#!/usr/bin/env python3
"""
Parse cached W3C/WHATWG WebIDL files and generate WIT interface definitions.

Reads from : target/tairitsu-wit/webidl-cache/*.idl
Writes to  : packages/browser-worlds/wit/generated/*.wit

Each output file is a self-contained WIT package with name
    tairitsu-browser:<domain>@0.2.0
so it does not collide with the hand-written 0.1.0 files.

All browser objects are represented as opaque u64 handles crossing the WASM
boundary — this is the same "handle table" pattern used in the Phase-0
hand-written files.

Usage:
    python scripts/generate_browser_wit.py                   # generate all domains
    python scripts/generate_browser_wit.py --domains dom css # specific domains only
    python scripts/generate_browser_wit.py --dry-run         # print without writing
    python scripts/generate_browser_wit.py --stats           # show coverage stats
"""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Dict, List, Optional, Set, Tuple


def log_info(message: str) -> None:
    print(f"[INFO] {message}")


def log_ok(message: str) -> None:
    print(f"[OK] {message}")


def log_warn(message: str) -> None:
    print(f"[WARN] {message}", file=sys.stderr)


def log_error(message: str) -> None:
    print(f"[ERROR] {message}", file=sys.stderr)

# ---------------------------------------------------------------------------
# Data model
# ---------------------------------------------------------------------------


@dataclass
class WebIDLParam:
    name: str
    idl_type: str
    optional: bool = False
    variadic: bool = False


@dataclass
class WebIDLMember:
    kind: str           # 'attribute' | 'operation' | 'const'
    name: str
    idl_type: str       # raw WebIDL return/value type string
    readonly: bool = False
    static: bool = False
    params: List[WebIDLParam] = field(default_factory=list)


@dataclass
class WebIDLInterface:
    name: str
    inheritance: Optional[str]
    members: List[WebIDLMember]
    is_partial: bool = False
    is_mixin: bool = False
    source_spec: str = ""


# ---------------------------------------------------------------------------
# Spec → domain mapping  (must match fetch_webidl.py TARGET_SPECS)
# ---------------------------------------------------------------------------

SPEC_TO_DOMAIN: Dict[str, str] = {
    "dom": "dom",
    "fullscreen": "dom",
    "html": "html",
    "uievents": "events",
    "pointerevents": "events",
    "touch-events": "events",
    "clipboard-apis": "events",
    "fetch": "fetch",
    "xhr": "fetch",
    "streams": "fetch",
    "fileapi": "fetch",
    "url": "url",
    "cssom": "css",
    "cssom-view": "css",
    "css-animations": "css",
    "css-transitions": "css",
    "css-fonts": "css",
    "css-conditional": "css",
    "storage": "storage",
    "websockets": "websocket",
    "service-workers": "workers",
    "webcrypto": "crypto",
    "mediacapture-streams": "media",
    "media-capabilities": "media",
    "mediasession": "media",
    "mediastream-recording": "media",
    "speech-api": "media",
    "screen-capture": "media",
    "webrtc": "webrtc",
    "webgl1": "canvas",
    "webgl2": "canvas",
    "webcodecs": "canvas",
    "intersection-observer": "observers",
    "resize-observer": "observers",
    "performance-timeline": "performance",
    "hr-time": "performance",
    "resource-timing": "performance",
    "navigation-timing": "performance",
    "user-timing": "performance",
    "notifications": "notifications",
    "permissions": "permissions",
    "geolocation": "device",
    "screen-orientation": "device",
    "battery-status": "device",
    "vibration": "device",
    "gamepad": "device",
    "credential-management": "auth",
    "payment-request": "payments",
    "wasm-js-api": "wasm",
}

# Desired output order for domains in the "full" world.
DOMAIN_ORDER = [
    "dom", "events", "html", "css", "fetch", "url", "storage",
    "websocket", "workers", "crypto", "canvas", "media", "webrtc",
    "observers", "performance", "notifications", "permissions",
    "device", "auth", "payments", "wasm",
]

# ---------------------------------------------------------------------------
# WebIDL → WIT type mapping
# ---------------------------------------------------------------------------

WEBIDL_TO_WIT: Dict[str, str] = {
    "boolean": "bool",
    "byte": "s8",
    "octet": "u8",
    "short": "s16",
    "unsigned short": "u16",
    # WebIDL 'long' / 'unsigned long' are guaranteed 32-bit signed/unsigned integers.
    # NOTE: Some browser APIs use these types for values that conceptually fit in 32
    # bits (e.g. event codes, DOM tree depth). For sizes and byte offsets browsers
    # typically use 'unsigned long long' or 'long long', which map to u64/s64 below.
    # If a specific API requires wider storage, override it in the hand-written 0.1.x
    # WIT files rather than widening the global mapping.
    "long": "s32",
    "unsigned long": "u32",
    "long long": "s64",
    "unsigned long long": "u64",
    "float": "f32",
    "unrestricted float": "f32",
    "double": "f64",
    "unrestricted double": "f64",
    "DOMString": "string",
    "USVString": "string",
    "ByteString": "string",
    "CSSOMString": "string",
    "DOMHighResTimeStamp": "f64",
    "DOMTimeStamp": "u64",
    # Event handler types - use u64 as opaque handles
    "EventHandler": "u64",
    "OnErrorEventHandler": "u64",
    "OnBeforeUnloadEventHandler": "u64",
    "VoidFunction": "u64",
    # Callback/function types that are handles (async callbacks, etc.)
    "Function": "u64",
    "MutationCallback": "u64",
    "IntersectionObserverCallback": "u64",
    "ResizeObserverCallback": "u64",
    "PerformanceObserverCallback": "u64",
    "FrameRequestCallback": "u64",
    "IdleRequestCallback": "u64",
    "TimerHandler": "u64",
    # Void / undefined
    "undefined": "_",
    "void": "_",
    # Generic
    "any": "string",
    "object": "u64",
    "symbol": "u64",
    "bigint": "u64",
    # Buffer / TypedArray types
    "ArrayBuffer": "list<u8>",
    "SharedArrayBuffer": "list<u8>",
    "ArrayBufferView": "list<u8>",
    "BufferSource": "list<u8>",
    "DataView": "list<u8>",
    "Int8Array": "list<s8>",
    "Int16Array": "list<s16>",
    "Int32Array": "list<s32>",
    "Uint8Array": "list<u8>",
    "Uint16Array": "list<u16>",
    "Uint32Array": "list<u32>",
    "BigInt64Array": "list<s64>",
    "BigUint64Array": "list<u64>",
    "Float32Array": "list<f32>",
    "Float64Array": "list<f64>",
    "Uint8ClampedArray": "list<u8>",
    # String list
    "DOMStringList": "list<string>",
}

WIT_KEYWORDS: Set[str] = {
    "use", "type", "resource", "func", "record", "enum", "flags", "variant",
    "interface", "world", "import", "export", "package", "include", "with",
    "constructor", "static", "borrow", "own", "option", "result", "list",
    "tuple", "string", "bool", "char",
    "u8", "u16", "u32", "u64", "s8", "s16", "s32", "s64", "f32", "f64",
    # Additional keywords that may appear in parameter names
    "from", "as", "where", "let", "true", "false", "async", "await",
}

# ---------------------------------------------------------------------------
# Identifier conversion
# ---------------------------------------------------------------------------


def camel_to_kebab(name: str) -> str:
    """Convert CamelCase / PascalCase / camelCase to kebab-case."""
    # Handles acronyms: HTMLElement → html-element, XMLHttpRequest → xml-http-request
    s1 = re.sub(r"([A-Z]+)([A-Z][a-z])", r"\1-\2", name)
    s2 = re.sub(r"([a-z0-9])([A-Z])", r"\1-\2", s1)
    result = s2.lower().replace("_", "-")
    result = re.sub(r"-+", "-", result).strip("-")
    if result and result[0].isdigit():
        result = "n-" + result
    return result or "unknown"


def sanitize_wit_ident(name: str) -> str:
    """Convert an arbitrary identifier to a valid WIT identifier."""
    kebab = camel_to_kebab(name)
    # Remove any characters not allowed in WIT identifiers
    kebab = re.sub(r"[^a-z0-9-]", "", kebab)
    kebab = re.sub(r"-+", "-", kebab).strip("-")
    if not kebab:
        return "unknown"
    if kebab in WIT_KEYWORDS:
        return f"%{kebab}"
    return kebab


# ---------------------------------------------------------------------------
# Type converter
# ---------------------------------------------------------------------------

def convert_type(type_str: str) -> str:
    """Convert a WebIDL type string to a WIT type string."""
    type_str = re.sub(r"\[[^\]]*\]\s*", "", type_str).strip()

    # Nullable: T?
    nullable = type_str.endswith("?")
    if nullable:
        type_str = type_str[:-1].strip()

    # Union: (T1 or T2 or ...)
    if type_str.startswith("(") and type_str.endswith(")"):
        inner = type_str[1:-1]
        parts = [p.strip() for p in re.split(r"\bor\b", inner)]
        real = [p for p in parts if p not in ("undefined", "null", "")]
        converted = convert_type(real[0]) if real else "string"
        if converted == "_":
            converted = "string"
        return f"option<{converted}>" if nullable else converted

    # sequence<T> / FrozenArray<T> / ObservableArray<T>
    m = re.match(r"(?:sequence|FrozenArray|ObservableArray)<(.+)>$", type_str)
    if m:
        inner = convert_type(m.group(1).strip())
        inner = "u8" if inner == "_" else inner
        result = f"list<{inner}>"
        return f"option<{result}>" if nullable else result

    # Promise<T>
    if re.match(r"Promise<", type_str):
        return "option<u64>" if nullable else "u64"

    # record<K, V>
    if re.match(r"record<", type_str):
        return "option<string>" if nullable else "string"

    # Exact primitive / known mapping
    if type_str in WEBIDL_TO_WIT:
        result = WEBIDL_TO_WIT[type_str]
        if result == "_":
            return "_"
        return f"option<{result}>" if nullable else result

    # Interface reference → opaque u64 handle
    result = "u64"
    return f"option<{result}>" if nullable else result


# ---------------------------------------------------------------------------
# WebIDL text parser
# ---------------------------------------------------------------------------

def _remove_comments(text: str) -> str:
    text = re.sub(r"/\*.*?\*/", " ", text, flags=re.DOTALL)
    text = re.sub(r"//[^\n]*", "", text)
    return text


def _remove_extended_attrs(text: str) -> str:
    """Strip [ExtendedAttribute] blocks."""
    result: list[str] = []
    depth = 0
    for ch in text:
        if ch == "[":
            depth += 1
        elif ch == "]":
            if depth > 0:
                depth -= 1
        elif depth == 0:
            result.append(ch)
    return "".join(result)


def _split_statements(body: str) -> List[str]:
    """Split interface body text by semicolons, respecting <> and ()."""
    stmts: list[str] = []
    current: list[str] = []
    depth = 0
    for ch in body:
        if ch in "<(":
            depth += 1
            current.append(ch)
        elif ch in ">)":
            depth -= 1
            current.append(ch)
        elif ch == "{":
            depth += 1
            current.append(ch)
        elif ch == "}":
            depth = max(0, depth - 1)
            current.append(ch)
        elif ch == ";" and depth == 0:
            s = "".join(current).strip()
            if s:
                stmts.append(s)
            current = []
        else:
            current.append(ch)
    tail = "".join(current).strip()
    if tail:
        stmts.append(tail)
    return stmts


def _parse_params(params_str: str) -> List[WebIDLParam]:
    """Parse a WebIDL parameter list string into WebIDLParam objects."""
    params: list[WebIDLParam] = []
    if not params_str.strip():
        return params

    # Split by commas respecting generics
    parts: list[str] = []
    current: list[str] = []
    depth = 0
    for ch in params_str:
        if ch in "<(":
            depth += 1
            current.append(ch)
        elif ch in ">)":
            depth -= 1
            current.append(ch)
        elif ch == "," and depth == 0:
            parts.append("".join(current).strip())
            current = []
        else:
            current.append(ch)
    if current:
        parts.append("".join(current).strip())

    for part in parts:
        part = re.sub(r"\[[^\]]*\]\s*", "", part).strip()
        if not part:
            continue

        optional = part.startswith("optional ")
        if optional:
            part = part[9:].lstrip()

        variadic = "..." in part
        part = part.replace("...", "").strip()

        # Remove default value
        eq_pos = -1
        depth_t = 0
        for idx, ch in enumerate(part):
            if ch in "<(":
                depth_t += 1
            elif ch in ">)":
                depth_t -= 1
            elif ch == "=" and depth_t == 0:
                eq_pos = idx
                break
        if eq_pos != -1:
            part = part[:eq_pos].strip()

        m = re.match(r"(.+)\s+(\w+)$", part.strip())
        if m:
            params.append(
                WebIDLParam(
                    name=m.group(2),
                    idl_type=m.group(1).strip(),
                    optional=optional,
                    variadic=variadic,
                )
            )
    return params


def _parse_member(stmt: str) -> Optional[WebIDLMember]:
    """Parse one WebIDL interface body statement into a WebIDLMember."""
    stmt = stmt.strip()
    if not stmt:
        return None

    # Skip const
    if stmt.startswith("const "):
        return None

    # Skip iterable / maplike / setlike / async iterable
    if re.match(r"(?:async\s+)?(?:iterable|maplike|setlike)\s*[<;]", stmt):
        return None

    # Skip inherit attribute
    if "inherit " in stmt:
        return None

    is_static = stmt.startswith("static ")
    if is_static:
        stmt = stmt[7:].lstrip()

    # Attribute
    if "attribute " in stmt:
        readonly = stmt.startswith("readonly ")
        if readonly:
            stmt = stmt[9:].lstrip()
        
        # Remove stringifier prefix (e.g., "stringifier attribute USVString href")
        if stmt.startswith("stringifier "):
            stmt = stmt[12:].lstrip()
        if not stmt.startswith("attribute "):
            # e.g. "readonly attribute type name" already stripped "readonly"
            pass
        stmt_body = re.sub(r"^attribute\s+", "", stmt)
        m = re.match(r"(.+)\s+(\w+)$", stmt_body.strip())
        if m:
            return WebIDLMember(
                kind="attribute",
                name=m.group(2),
                idl_type=m.group(1).strip(),
                readonly=readonly,
                static=is_static,
            )
        return None

    # Operation (must have parentheses)
    if "(" not in stmt:
        return None

    # Remove special prefixes
    for kw in ("getter", "setter", "deleter", "stringifier", "legacycaller"):
        stmt = re.sub(rf"^{kw}\s+", "", stmt)
    stmt = stmt.strip()

    paren_open = stmt.find("(")
    paren_close = stmt.rfind(")")
    if paren_open == -1 or paren_close == -1:
        return None

    params_str = stmt[paren_open + 1: paren_close]
    before = stmt[:paren_open].strip()

    m = re.match(r"(.+)\s+(\w+)$", before)
    if not m:
        # Might be just a name (stringifier without return type)
        name_m = re.match(r"^(\w+)$", before)
        if name_m:
            name = name_m.group(1)
            if name == "constructor":
                return None  # Skip WebIDL constructors
            return WebIDLMember(
                kind="operation",
                name=name,
                idl_type="undefined",
                static=is_static,
                params=_parse_params(params_str),
            )
        return None

    name = m.group(2)
    if name == "constructor":
        return None

    return WebIDLMember(
        kind="operation",
        name=name,
        idl_type=m.group(1).strip(),
        static=is_static,
        params=_parse_params(params_str),
    )


def parse_webidl_file(
    text: str, source_spec: str = ""
) -> Dict[str, WebIDLInterface]:
    """Parse a WebIDL text and return {interface_name: WebIDLInterface}."""
    text = _remove_comments(text)
    text = _remove_extended_attrs(text)

    interfaces: Dict[str, WebIDLInterface] = {}
    pos = 0

    while pos < len(text):
        # Find 'interface' keyword (may be preceded by 'partial')
        m = re.search(r"\b(partial\s+)?interface(\s+mixin)?\s+", text[pos:])
        if not m:
            break

        decl_start = pos + m.start()
        name_start = pos + m.end()

        is_partial = bool(m.group(1))
        is_mixin = bool(m.group(2))

        name_m = re.match(r"(\w+)", text[name_start:])
        if not name_m:
            pos = decl_start + 1
            continue

        iface_name = name_m.group(1)
        after_name = name_start + name_m.end()

        # Inheritance
        inherit_m = re.match(r"\s*:\s*(\w+)", text[after_name:])
        inheritance: Optional[str] = None
        if inherit_m:
            inheritance = inherit_m.group(1)
            after_name += inherit_m.end()

        # Find opening brace
        brace_m = re.search(r"\{", text[after_name:])
        if not brace_m:
            pos = decl_start + 1
            continue

        brace_open = after_name + brace_m.start()

        # Find matching closing brace
        depth = 1
        j = brace_open + 1
        while j < len(text) and depth > 0:
            if text[j] == "{":
                depth += 1
            elif text[j] == "}":
                depth -= 1
            j += 1

        body = text[brace_open + 1: j - 1]
        members: List[WebIDLMember] = []
        for stmt in _split_statements(body):
            member = _parse_member(stmt)
            if member:
                members.append(member)

        if iface_name not in interfaces:
            interfaces[iface_name] = WebIDLInterface(
                name=iface_name,
                inheritance=inheritance,
                members=[],
                is_partial=is_partial,
                is_mixin=is_mixin,
                source_spec=source_spec,
            )
        interfaces[iface_name].members.extend(members)

        pos = j

    return interfaces


# ---------------------------------------------------------------------------
# WIT generation
# ---------------------------------------------------------------------------

# Global singleton interfaces - these don't need self parameters
# because they represent global browser objects (window, document, navigator)
GLOBAL_SINGLETON_INTERFACES = {
    "window",
    "document", 
    "navigator",
    "location",
    "history",
    "screen",
    "console",
    "performance",
    "crypto",
    "fetch",
}


def _wit_interface_block(iface: WebIDLInterface) -> Optional[str]:
    """Render one WebIDL interface as a WIT interface block. Returns None if empty."""
    wit_name = sanitize_wit_ident(iface.name)
    handle_type = f"{wit_name}-handle"
    is_singleton = iface.name.lower().replace("-", "") in {n.replace("-", "") for n in GLOBAL_SINGLETON_INTERFACES}

    lines: list[str] = []
    lines.append(f"/// WebIDL interface: `{iface.name}`")
    if iface.inheritance:
        lines.append(f"/// Inherits: `{iface.inheritance}`")
    if is_singleton:
        lines.append(f"/// Note: Global singleton - no self parameter needed")
    lines.append(
        f"/// Source: https://github.com/w3c/webref/tree/main/ed/idl/{iface.source_spec}.idl"
    )
    lines.append(f"interface {wit_name} {{")
    lines.append(
        f"    /// Opaque handle to a host-side `{iface.name}` instance.")
    lines.append(f"    type {handle_type} = u64;")

    emitted: Set[str] = set()
    has_non_handle = False

    for member in iface.members:
        if member.kind == "attribute":
            suffix = camel_to_kebab(member.name)
            getter = f"get-{suffix}"
            setter = f"set-{suffix}"
            wit_type = convert_type(member.idl_type)

            if wit_type == "_":
                continue

            if getter not in emitted:
                lines.append("")
                lines.append(f"    /// `{member.name}` attribute — getter.")
                if member.static or is_singleton:
                    lines.append(f"    {getter}: func() -> {wit_type};")
                else:
                    lines.append(
                        f"    {getter}: func(self: {handle_type}) -> {wit_type};"
                    )
                emitted.add(getter)
                has_non_handle = True

            if not member.readonly and setter not in emitted:
                lines.append(f"    /// `{member.name}` attribute — setter.")
                if member.static or is_singleton:
                    lines.append(f"    {setter}: func(value: {wit_type});")
                else:
                    lines.append(
                        f"    {setter}: func(self: {handle_type}, value: {wit_type});"
                    )
                emitted.add(setter)

        elif member.kind == "operation":
            op_name = sanitize_wit_ident(member.name)
            if op_name in ("%constructor", "constructor"):
                continue
            ret_type = convert_type(member.idl_type)

            params: list[str] = []
            if not member.static and not is_singleton:
                params.append(f"self: {handle_type}")

            for p in member.params:
                p_name = sanitize_wit_ident(p.name)
                p_type = convert_type(p.idl_type)
                if p_type == "_":
                    continue
                if p.optional and not p_type.startswith("option<"):
                    p_type = f"option<{p_type}>"
                if p.variadic and not p_type.startswith("list<"):
                    p_type = f"list<{p_type}>"
                params.append(f"{p_name}: {p_type}")

            if op_name not in emitted:
                sig = f"    {op_name}: func({', '.join(params)})"
                if ret_type != "_":
                    sig += f" -> {ret_type}"
                sig += ";"
                lines.append("")
                lines.append(f"    /// `{member.name}()` operation.")
                lines.append(sig)
                emitted.add(op_name)
                has_non_handle = True

    lines.append("}")

    if not has_non_handle:
        return None

    return "\n".join(lines)

def _generate_special_type_defs(interfaces: List[WebIDLInterface]) -> List[str]:
    """Generate type definitions for special types like event-handler-record."""
    lines = []
    
    # Find the global-event-handlers interface
    global_event_handlers_iface = next(
        (iface for iface in interfaces if iface.name == "global-event-handlers"),
        None
    )
    
    if global_event_handlers_iface:
        lines.append("/// Event handler function type")
        lines.append("///")
        lines.append("type event-handler-record = record {")
        
        # Collect all event handler names from the interface
        event_handlers = []
        for member in global_event_handlers_iface.members:
            if member.kind == "attribute" and member.name.startswith("on"):
                # Convert kebab-case to camelCase
                parts = member.name.split("-")
                handler_name = parts[0] + "".join(p.capitalize() for p in parts[1:])
                event_handlers.append(handler_name)
        
        # Generate record fields (sorted for consistency)
        for handler in sorted(event_handlers):
            lines.append(f"    {handler}: option<event-handler-handle>;")
        
        lines.append("};")
        lines.append("")
        lines.append("/// Event handler function handle")
        lines.append("type event-handler-handle = u64;")
        lines.append("")
    
    return lines

def generate_domain_wit(
    domain: str,
    interfaces: List[WebIDLInterface],
    source_specs: List[str],
) -> str:
    """Render all interfaces for *domain* as a complete WIT file."""
    wit_interface_names: list[str] = []
    interface_blocks: list[str] = []

    # De-duplicate by name (keep merge already done by parse_webidl_file)
    seen_names: Set[str] = set()
    for iface in interfaces:
        if iface.name in seen_names:
            continue
        seen_names.add(iface.name)

        block = _wit_interface_block(iface)
        if block:
            interface_blocks.append(block)
            wit_interface_names.append(sanitize_wit_ident(iface.name))

    if not interface_blocks:
        return ""

    # Header
    header_lines = [
        f"/// Auto-generated WIT interfaces — domain: {domain}",
        "///",
        "/// Generated by: scripts/generate_browser_wit.py",
        f"/// Source specs: {', '.join(source_specs)}",
        "/// W3C/WHATWG webref: https://github.com/w3c/webref (MIT)",
        "///",
        "/// All browser objects are represented as opaque u64 handles.",
        "/// Regenerate with: just wit-gen",
        "///",
        "/// Status: auto-generated (Phase A) — review before use in production",
        f"package tairitsu-browser:{domain}@0.2.0;",
        "",
    ]

    # Generate type definitions for special types (e.g., event handlers)
    type_defs_lines = _generate_special_type_defs(interfaces)

    # Build world block
    world_imports = "\n".join(
        f"    import {n};" for n in wit_interface_names
    )
    world_block = (
        f"/// Browser {domain} world — all {len(wit_interface_names)} "
        f"auto-generated interfaces.\n"
        f"world {sanitize_wit_ident(domain)}-world {{\n"
        f"{world_imports}\n"
        f"}}"
    )

    sections = (
        "\n".join(header_lines)
        + "\n"
        + "\n\n".join(interface_blocks)
        + "\n\n"
        + world_block
        + "\n"
    )
    return sections


# ---------------------------------------------------------------------------
# Main orchestration
# ---------------------------------------------------------------------------

def load_all_interfaces(
    cache_dir: Path,
) -> Tuple[Dict[str, List[WebIDLInterface]], Dict[str, List[str]]]:
    """
    Load all cached WebIDL files, parse them, and group by domain.

    Returns:
        domain_interfaces: {domain: [WebIDLInterface, ...]}
        domain_specs:      {domain: [spec_name, ...]}
    """
    domain_interfaces: Dict[str, List[WebIDLInterface]] = {}
    domain_specs: Dict[str, List[str]] = {}

    idl_files = sorted(cache_dir.glob("*.idl"))
    if not idl_files:
        log_error(
            f"No .idl files found in {cache_dir}. Run `just wit-fetch-idl` first."
        )
        return domain_interfaces, domain_specs

    # Track interface name → last seen to merge partial interfaces
    merged: Dict[str, WebIDLInterface] = {}

    for idl_file in idl_files:
        spec = idl_file.stem
        domain = SPEC_TO_DOMAIN.get(spec)
        if not domain:
            continue  # skip specs not in our map

        try:
            text = idl_file.read_text(encoding="utf-8")
        except OSError as exc:
            log_warn(f"Cannot read {idl_file}: {exc}")
            continue

        ifaces = parse_webidl_file(text, source_spec=spec)
        domain_specs.setdefault(domain, [])
        if spec not in domain_specs[domain]:
            domain_specs[domain].append(spec)

        for name, iface in ifaces.items():
            if name in merged:
                # Merge partial/mixin members into existing
                merged[name].members.extend(iface.members)
            else:
                iface.source_spec = spec
                merged[name] = iface

    # Group by domain: use the domain of the first-seen spec for each interface
    for name, iface in merged.items():
        spec = iface.source_spec
        domain = SPEC_TO_DOMAIN.get(spec, "unknown")
        domain_interfaces.setdefault(domain, [])
        domain_interfaces[domain].append(iface)

    return domain_interfaces, domain_specs


def run_generate(
    cache_dir: Path,
    output_dir: Path,
    domains: Optional[List[str]] = None,
    *,
    dry_run: bool = False,
    stats: bool = False,
) -> None:
    """Parse cached WebIDL and write generated WIT files."""
    log_info(f"Cache  : {cache_dir}")
    log_info(f"Output : {output_dir}")
    print()

    domain_interfaces, domain_specs = load_all_interfaces(cache_dir)

    if not domain_interfaces:
        return

    if stats:
        total_ifaces = sum(len(v) for v in domain_interfaces.values())
        log_info(
            f"Parsed {total_ifaces} interfaces across {len(domain_interfaces)} domains:"
        )
        for dom in DOMAIN_ORDER:
            ifaces = domain_interfaces.get(dom, [])
            specs = domain_specs.get(dom, [])
            print(f"  {dom:<15} {len(ifaces):3d} interfaces  ← {', '.join(specs)}")
        unknown = domain_interfaces.get("unknown", [])
        if unknown:
            print(f"  {'unknown':<15} {len(unknown):3d} interfaces")
        return

    target_domains = domains if domains else DOMAIN_ORDER

    all_world_imports: list[str] = []
    written = skipped = 0

    if not dry_run:
        output_dir.mkdir(parents=True, exist_ok=True)

    for domain in target_domains:
        ifaces = domain_interfaces.get(domain, [])
        if not ifaces:
            log_warn(f"skip {domain} — no interfaces in cache")
            skipped += 1
            continue

        specs = domain_specs.get(domain, [])
        wit_content = generate_domain_wit(domain, ifaces, specs)
        if not wit_content:
            log_warn(f"skip {domain} — all interfaces were empty")
            skipped += 1
            continue

        dest = output_dir / f"{domain}.wit"
        iface_count = wit_content.count("\ninterface ")

        if dry_run:
            log_info(
                f"dry-run write {dest.name} ({iface_count} interfaces, {len(wit_content):,} bytes)"
            )
            continue

        dest.write_text(wit_content, encoding="utf-8")
        log_ok(
            f"Wrote {dest.name:<30} {iface_count:3d} interfaces ({len(wit_content):,} bytes)"
        )
        written += 1

        # Collect import lines for the full world
        wit_domain_name = sanitize_wit_ident(domain)
        all_world_imports.append(
            f'    import {wit_domain_name}-world from "{domain}";'
        )

    if not dry_run and written > 0:
        print()
        log_info(f"Result : {written} files written, {skipped} skipped")

        # Generate unified browser-full.wit
        generate_full_world(output_dir, dry_run)


def generate_full_world(output_dir: Path, dry_run: bool = False) -> None:
    """
    Generate a unified browser-full.wit that combines all domain worlds.
    
    This creates a single WIT file with package tairitsu-browser:full@0.2.0
    that imports all domain-specific interfaces, allowing Rust code to use
    a single world binding.
    """
    generated_dir = output_dir
    if not generated_dir.exists():
        log_warn("Cannot generate browser-full.wit: generated directory not found")
        return
    
    # Collect all interfaces from all domain WIT files
    all_interfaces: List[str] = []
    seen_interface_names: Set[str] = set()
    domain_files = sorted(generated_dir.glob("*.wit"))
    
    # Interfaces we define manually - skip these from collected interfaces
    manual_interfaces = {"event-callbacks", "lifecycle", "event-target"}
    
    for domain_file in domain_files:
        if domain_file.name == "browser-full.wit":
            continue
        
        content = domain_file.read_text(encoding="utf-8")
        
        # Extract interface blocks (between "interface" and closing "}")
        # Skip the world definition and package declaration
        in_interface = False
        brace_depth = 0
        current_block: List[str] = []
        
        for line in content.split("\n"):
            stripped = line.strip()
            
            # Skip package, world, and comments at file level
            if stripped.startswith("package "):
                continue
            if stripped.startswith("/// ") and not in_interface:
                continue
            if stripped.startswith("world "):
                break  # Stop at world definition
            
            # Track interface blocks
            if stripped.startswith("interface "):
                # Extract interface name
                m = re.match(r"interface\s+([\w-]+)", stripped)
                if m:
                    iface_name = m.group(1)
                    # Skip if we've seen this name before or it's a manual interface
                    if iface_name in seen_interface_names or iface_name in manual_interfaces:
                        in_interface = False
                        current_block = []
                        continue
                    seen_interface_names.add(iface_name)
                
                in_interface = True
                current_block = [line]
                brace_depth = line.count("{") - line.count("}")
                continue
            
            if in_interface:
                current_block.append(line)
                brace_depth += line.count("{") - line.count("}")
                
                if brace_depth == 0:
                    all_interfaces.append("\n".join(current_block))
                    in_interface = False
                    current_block = []
    
    if not all_interfaces:
        log_warn("No interfaces found to include in browser-full.wit")
        return
    
    # Build the unified WIT file
    header = """/// Unified browser world — combines all auto-generated domain interfaces.
///
/// Generated by: scripts/generate_browser_wit.py
/// W3C/WHATWG webref: https://github.com/w3c/webref (MIT)
///
/// All browser objects are represented as opaque u64 handles.
/// Regenerate with: just wit-gen
///
/// Status: auto-generated (Phase A) — review before use in production
package tairitsu-browser:full@0.2.0;

"""
    
    # Event callbacks interface (exported to host)
    event_callbacks = """/// Event callback interface - implemented by the component, called by host.
///
/// The host (browser-glue) calls these functions when DOM events fire.
interface event-callbacks {
    /// Opaque listener identifier returned by add-event-listener.
    type listener-id = u64;
    /// Opaque event handle for prevent-default/stop-propagation.
    type event-handle = u64;

    /// Mouse event data structure.
    record mouse-event-data {
        client-x: f64,
        client-y: f64,
        offset-x: f64,
        offset-y: f64,
        button: u8,
        buttons: u8,
        ctrl-key: bool,
        shift-key: bool,
        alt-key: bool,
        meta-key: bool,
    }

    /// Keyboard event data structure.
    record keyboard-event-data {
        key: string,
        code: string,
        key-code: u32,
        ctrl-key: bool,
        shift-key: bool,
        alt-key: bool,
        meta-key: bool,
        repeat: bool,
    }

    /// Focus event data structure.
    record focus-event-data {
        related-target: option<u64>,
    }

    /// Input event data structure.
    record input-event-data {
        data: option<string>,
        input-type: string,
    }

    /// Called when a mouse event fires on a registered listener.
    on-mouse-event: func(listener-id: listener-id, event: event-handle, data: mouse-event-data);
    /// Called when a keyboard event fires on a registered listener.
    on-keyboard-event: func(listener-id: listener-id, event: event-handle, data: keyboard-event-data);
    /// Called when a focus event fires on a registered listener.
    on-focus-event: func(listener-id: listener-id, event: event-handle, data: focus-event-data);
    /// Called when an input event fires on a registered listener.
    on-input-event: func(listener-id: listener-id, event: event-handle, data: input-event-data);
    /// Called for any other event type.
    on-generic-event: func(listener-id: listener-id, event: event-handle, event-type: string);
}

/// Component lifecycle interface - implemented by the component.
interface lifecycle {
    /// Called by the host after the component is instantiated.
    /// This is the entry point for the component's main logic.
    start: func() -> result<_, string>;
}

/// Console interface for logging.
interface console {
    /// Log a message to the console.
    log: func(message: string);
    /// Log a warning to the console.
    warn: func(message: string);
    /// Log an error to the console.
    error: func(message: string);
}

/// Event target interface for adding/removing event listeners.
interface event-target {
    /// Add an event listener to a target node.
    /// Returns a listener-id that can be used to remove the listener.
    add-event-listener: func(target: u64, event-type: string, use-capture: bool) -> result<u64, string>;
    /// Remove a previously registered event listener.
    remove-event-listener: func(target: u64, listener-id: u64) -> result<_, string>;
    /// Prevent the default action for this event.
    prevent-default: func(event: u64);
    /// Stop the event from propagating further.
    stop-propagation: func(event: u64);
}

/// Style interface - convenience wrapper for CSS operations.
/// This is a simplified interface for the most common style operations.
interface style {
    /// Set a CSS property on an element.
    set-style-property: func(element: u64, property: string, value: string) -> result<_, string>;
    /// Get a CSS property value from an element.
    get-style-property: func(element: u64, property: string) -> option<string>;
    /// Remove a CSS property from an element.
    remove-style-property: func(element: u64, property: string) -> result<_, string>;
}

"""
    
    # Collect import interface names (already filtered for duplicates)
    import_interface_names = list(seen_interface_names)
    # Sort for consistent output
    import_interface_names.sort()
    
    # Add special interfaces (including style wrapper and console)
    all_interface_names = import_interface_names + ["event-callbacks", "lifecycle", "event-target", "style", "console"]
    
    world_imports = "\n".join(f"    import {name};" for name in import_interface_names)
    world_exports = """    export event-callbacks;
    export lifecycle;"""
    
    world_block = f"""
/// Full browser world — all {len(all_interface_names)} auto-generated interfaces.
world browser-full {{
{world_imports}
    import event-target;
    import style;
    import console;
{world_exports}
}}
"""
    
    full_content = header + event_callbacks + "\n\n".join(all_interfaces) + "\n" + world_block + "\n"
    
    dest = output_dir.parent / "browser-full.wit"
    
    if dry_run:
        log_info(f"dry-run write {dest.name} ({len(all_interface_names)} interfaces, {len(full_content):,} bytes)")
        return
    
    dest.write_text(full_content, encoding="utf-8")
    log_ok(f"Wrote {dest.name:<30} {len(all_interface_names):3d} interfaces ({len(full_content):,} bytes)")


# ---------------------------------------------------------------------------
# CLI entry point
# ---------------------------------------------------------------------------

def main() -> None:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print what would be written without actually writing files",
    )
    parser.add_argument(
        "--stats",
        action="store_true",
        help="Show coverage statistics and exit",
    )
    parser.add_argument(
        "--domains",
        nargs="+",
        metavar="DOMAIN",
        help="Generate only specific domain(s)",
    )
    parser.add_argument(
        "--cache-dir",
        metavar="DIR",
        help="Override WebIDL cache directory",
    )
    parser.add_argument(
        "--output-dir",
        metavar="DIR",
        help="Override WIT output directory",
    )
    args = parser.parse_args()

    project_root = Path(__file__).parent.parent

    cache_dir = (
        Path(args.cache_dir)
        if args.cache_dir
        else project_root / "target" / "tairitsu-wit" / "webidl-cache"
    )
    output_dir = (
        Path(args.output_dir)
        if args.output_dir
        else project_root / "packages" / "browser-worlds" / "wit" / "generated"
    )

    print("=" * 64)
    log_info("Tairitsu WebIDL -> WIT Generator")
    print("=" * 64)
    if args.dry_run:
        log_info("Mode   : dry run (no files written)")

    run_generate(
        cache_dir,
        output_dir,
        domains=args.domains,
        dry_run=args.dry_run,
        stats=args.stats,
    )


if __name__ == "__main__":
    main()
