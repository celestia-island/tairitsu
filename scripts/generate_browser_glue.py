#!/usr/bin/env python3
"""
Parse generated WIT files and produce TypeScript glue code.

Reads from  : packages/browser-worlds/wit/generated/*.wit
Writes to   : packages/browser-glue/src/generated/*-glue.ts

This generator creates TypeScript modules that implement WIT import
interfaces by bridging to actual browser DOM/Web APIs.

Usage:
    python3 scripts/generate_browser_glue.py              # generate all domains
    python3 scripts/generate_browser_glue.py --domains storage url  # specific domains
    python3 scripts/generate_browser_glue.py --dry-run    # preview without writing
    python3 scripts/generate_browser_glue.py --stats      # show coverage stats
"""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Set, Tuple

# Import local modules
from wit_parser import (
    parse_wit_file, WitPackage, WitInterface, WitFunction,
    WitParam, WitType, WitPrimitive, WitHandle, WitOption,
    WitList, WitResult, WitTypeAlias, kebab_to_camel, kebab_to_pascal,
    wit_type_to_string
)
from type_mapper import (
    TypeScriptTypeMapper, JavaScriptMarshaler,
    is_async_function, generate_result_type
)


# ---------------------------------------------------------------------------
# Logging utilities
# ---------------------------------------------------------------------------

def log_info(message: str) -> None:
    print(f"[INFO] {message}")


def log_ok(message: str) -> None:
    print(f"[OK] {message}")


def log_warn(message: str) -> None:
    print(f"[WARN] {message}", file=sys.stderr)


def log_error(message: str) -> None:
    print(f"[ERROR] {message}", file=sys.stderr)


# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

# Interface name → browser class mapping
INTERFACE_TO_BROWSER_CLASS: Dict[str, str] = {
    # DOM
    "node": "Node",
    "element": "Element",
    "document": "Document",
    "window": "Window",

    # Fetch
    "headers": "Headers",
    "request": "Request",
    "response": "Response",
    "body": "Body",

    # Storage
    "storage-manager": "StorageManager",
    "navigator-storage": "NavigatorStorage",
    "storage": "Storage",

    # URL
    "url": "URL",
    "url-search-params": "URLSearchParams",

    # Crypto
    "crypto": "Crypto",
    "subtle-crypto": "SubtleCrypto",
    "crypto-key": "CryptoKey",

    # Events
    "event": "Event",
    "event-target": "EventTarget",

    # Canvas
    "canvas-rendering-context": "CanvasRenderingContext2D",
    "webgl-rendering-context": "WebGLRenderingContext",

    # Notifications
    "notification": "Notification",

    # Permissions
    "permissions": "Permissions",
    "permission-status": "PermissionStatus",

    # Streams
    "readable-stream": "ReadableStream",
    "writable-stream": "WritableStream",
    "transform-stream": "TransformStream",

    # Workers
    "worker": "Worker",
    "service-worker": "ServiceWorker",

    # IndexedDB
    "idb-database": "IDBDatabase",
    "idb-request": "IDBRequest",
    "idb-transaction": "IDBTransaction",

    # Media
    "media-stream": "MediaStream",
    "media-devices": "MediaDevices",

    # Geolocation
    "geolocation": "Geolocation",
    "geolocation-position": "GeolocationPosition",

    # Performance
    "performance": "Performance",

    # WebRTC
    "rtc-peer-connection": "RTCPeerConnection",
}

# Known async operation patterns (return Promises in browser)
# Note: Only include base operation names, NOT poll-* functions
ASYNC_PATTERNS = [
    "estimate", "persist", "persisted", "fetch", "respond",
    "array-buffer", "blob", "text", "json", "bytes", "form-data",
    "read", "write", "cancel", "close", "abort", "flush",
    "get-reader", "get-writer", "pipe-to", "pipe-through",
    "clone", "from", "create-image-bitmap",
    "get-user-media", "enumerate-devices",
    "query", "request",
]

# Attribute getters that return handles (need wrapping)
HANDLE_GETTER_PATTERNS = [
    "get-headers", "get-body", "get-signal",
    "get-storage", "get-crypto", "get-performance",
]


# ---------------------------------------------------------------------------
# Code Generation Data Structures
# ---------------------------------------------------------------------------

@dataclass
class GeneratedParam:
    """Generated function parameter."""
    name: str
    ts_type: str
    wit_type_str: str


@dataclass
class GeneratedFunction:
    """Generated function info."""
    wit_name: str
    ts_name: str
    pascal_name: str
    params: List[GeneratedParam]
    ts_return: str
    ts_return_inner: str = ""
    is_async: bool = False
    is_getter: bool = False
    is_setter: bool = False
    is_static: bool = False
    return_is_void: bool = False
    return_is_optional: bool = False
    browser_method: str = ""
    browser_attr: str = ""
    browser_args: str = ""
    browser_class: str = ""
    self_param: str = "self"
    value_param: str = "value"
    has_explicit_poll: bool = False  # If True, don't generate auto-poll function
    docs: List[str] = field(default_factory=list)


@dataclass
class GeneratedTypeAlias:
    """Generated type alias."""
    name: str
    ts_name: str
    ts_type: str
    docs: List[str] = field(default_factory=list)


@dataclass
class GeneratedInterface:
    """Generated interface info."""
    wit_name: str
    name: str
    handle_type: Optional[str]
    handle_var: str
    handle_pascal: str
    browser_class: str
    functions: List[GeneratedFunction]
    type_aliases: List[GeneratedTypeAlias]
    create_function: bool = False


@dataclass
class GeneratedDomain:
    """Generated domain info."""
    name: str
    export_name: str
    interfaces: List[GeneratedInterface]
    interface_count: int


# ---------------------------------------------------------------------------
# Generator Class
# ---------------------------------------------------------------------------

class BrowserGlueGenerator:
    """Generates TypeScript glue code from WIT files."""

    def __init__(self):
        self.type_mapper = TypeScriptTypeMapper()
        self.marshaler = JavaScriptMarshaler()

    # JavaScript/TypeScript reserved keywords
    JS_RESERVED_WORDS = {
        "break", "case", "catch", "continue", "debugger", "default", "delete",
        "do", "else", "finally", "for", "function", "if", "in", "instanceof",
        "new", "return", "switch", "this", "throw", "try", "typeof", "var",
        "void", "while", "with", "class", "const", "enum", "export", "extends",
        "import", "super", "implements", "interface", "let", "package", "private",
        "protected", "public", "static", "yield", "null", "true", "false",
        "undefined", "NaN", "Infinity", "await", "async", "of", "get", "set",
        # Also reserved in strict mode
        "arguments", "eval",
    }

    def generate_function(self, func: WitFunction, interface: WitInterface,
                          browser_class: str, handle_type: Optional[str]) -> GeneratedFunction:
        """Generate code for a single WIT function."""

        # Convert function name
        wit_name = func.name
        ts_name = kebab_to_camel(wit_name)
        # Escape reserved words by prefixing with underscore
        if ts_name in self.JS_RESERVED_WORDS:
            ts_name = f"_{ts_name}"
        pascal_name = kebab_to_pascal(wit_name)

        # Process parameters
        params: List[GeneratedParam] = []
        self_param = "self"
        browser_args_list: List[str] = []

        for i, p in enumerate(func.params):
            param_name = kebab_to_camel(p.name)
            # Escape reserved words by prefixing with underscore
            if param_name in self.JS_RESERVED_WORDS:
                param_name = f"_{param_name}"
            ts_type = self.type_mapper.map_type(p.type_)
            wit_type_str = wit_type_to_string(p.type_)

            params.append(GeneratedParam(param_name, ts_type, wit_type_str))

            # First param is often 'self' for instance methods
            if i == 0 and p.name == "self":
                self_param = param_name
            else:
                browser_args_list.append(param_name)

        # Determine return type
        ts_return = "void"
        ts_return_inner = ""
        return_is_void = True
        return_is_optional = False

        if func.result:
            ts_return = self.type_mapper.map_type(func.result)
            return_is_void = ts_return == "void"
            return_is_optional = isinstance(func.result, WitOption)
            if not return_is_void:
                ts_return_inner = ts_return

        # Determine function characteristics
        is_getter = wit_name.startswith("get-")
        is_setter = wit_name.startswith("set-")
        is_async = self._is_async_function(wit_name, func, interface.name)
        is_static = func.is_static

        # Determine browser API mapping
        browser_method = ts_name
        browser_attr = ""
        browser_args = ", ".join(browser_args_list)

        if is_getter:
            # get-foo → obj.foo
            attr_name = wit_name[4:]  # Remove "get-"
            browser_attr = kebab_to_camel(attr_name)
        elif is_setter:
            # set-foo → obj.foo = value
            attr_name = wit_name[4:]  # Remove "set-"
            browser_attr = kebab_to_camel(attr_name)
            if params:
                value_param = params[-1].name

        return GeneratedFunction(
            wit_name=wit_name,
            ts_name=ts_name,
            pascal_name=pascal_name,
            params=params,
            ts_return=ts_return,
            ts_return_inner=ts_return_inner,
            is_async=is_async,
            is_getter=is_getter,
            is_setter=is_setter,
            is_static=is_static,
            return_is_void=return_is_void,
            return_is_optional=return_is_optional,
            browser_method=browser_method,
            browser_attr=browser_attr,
            browser_args=browser_args,
            browser_class=browser_class if is_static else "",
            self_param=self_param,
            docs=func.docs,
        )

    def generate_interface(self, iface: WitInterface) -> GeneratedInterface:
        """Generate code for a WIT interface."""

        wit_name = iface.name
        handle_type = None
        handle_var = ""
        handle_pascal = ""

        # Find handle type alias
        for ta in iface.type_aliases:
            if ta.name.endswith("-handle"):
                handle_type = ta.name
                handle_var = kebab_to_camel(ta.name.replace("-handle", "Handles"))
                handle_pascal = kebab_to_pascal(ta.name.replace("-handle", ""))
                break

        # Map to browser class
        browser_class = INTERFACE_TO_BROWSER_CLASS.get(wit_name, wit_name.replace("-", ""))

        # Generate type aliases
        type_aliases: List[GeneratedTypeAlias] = []
        for ta in iface.type_aliases:
            ts_name = kebab_to_pascal(ta.name)
            ts_type = self.type_mapper.map_type(ta.target)
            type_aliases.append(GeneratedTypeAlias(
                name=ta.name,
                ts_name=ts_name,
                ts_type=ts_type,
                docs=ta.docs,
            ))

        # Generate functions
        functions: List[GeneratedFunction] = []

        # First, collect all function names to check for existing poll functions
        func_names = {func.name for func in iface.functions}
        poll_names = {name for name in func_names if name.startswith("poll-")}
        # Map of base function name -> has explicit poll function
        has_explicit_poll = {}
        for name in func_names:
            if not name.startswith("poll-"):
                poll_name = f"poll-{name}"
                has_explicit_poll[name] = poll_name in poll_names

        for func in iface.functions:
            gf = self.generate_function(func, iface, browser_class, handle_type)
            # Mark if this function should NOT generate auto-poll (has explicit poll)
            gf.has_explicit_poll = has_explicit_poll.get(func.name, False)
            functions.append(gf)

        # Check if interface has a create/constructor function
        create_function = any(
            f.wit_name in ("create", "constructor") or
            "create" in f.wit_name.lower()
            for f in functions
        )

        return GeneratedInterface(
            wit_name=wit_name,
            name=kebab_to_pascal(wit_name),
            handle_type=handle_type,
            handle_var=handle_var,
            handle_pascal=handle_pascal,
            browser_class=browser_class,
            functions=functions,
            type_aliases=type_aliases,
            create_function=create_function,
        )

    def generate_domain(self, domain: str, wit_path: Path) -> Optional[GeneratedDomain]:
        """Generate glue code for a domain."""

        try:
            pkg = parse_wit_file(wit_path)
        except Exception as e:
            log_warn(f"Failed to parse {wit_path}: {e}")
            return None

        interfaces: List[GeneratedInterface] = []
        for iface in pkg.interfaces:
            # Skip empty interfaces (only handle type, no functions)
            if not iface.functions:
                continue
            gi = self.generate_interface(iface)
            interfaces.append(gi)

        if not interfaces:
            return None

        return GeneratedDomain(
            name=domain,
            export_name=domain.replace("-", ""),
            interfaces=interfaces,
            interface_count=len(interfaces),
        )

    def render_module(self, domain: GeneratedDomain, source_file: str) -> str:
        """Render TypeScript module for a domain using simple string templating."""

        lines: List[str] = []

        # Header
        lines.append("/**")
        lines.append(f" * {domain.name} glue — implements the `tairitsu-browser:{domain.name}` WIT import interfaces.")
        lines.append(" *")
        lines.append(f" * Auto-generated from: {source_file}")
        lines.append(" * Generated by: scripts/generate_browser_glue.py")
        lines.append(" *")
        lines.append(" * DO NOT EDIT MANUALLY - regenerate with: just glue-gen")
        lines.append(" *")
        lines.append(" * All browser objects are represented as opaque bigint handles.")
        lines.append(" */")
        lines.append("")

        # Check for async functions
        has_async = any(f.is_async for iface in domain.interfaces for f in iface.functions)

        if has_async:
            lines.append("// ---------------------------------------------------------------------------")
            lines.append("// Async handle table for Promise-based operations")
            lines.append("// ---------------------------------------------------------------------------")
            lines.append("")
            lines.append("let _nextAsyncHandle = 1n;")
            lines.append("")
            lines.append("interface AsyncHandle<T> {")
            lines.append("  promise: Promise<T>;")
            lines.append("  result: { ok: true; value: T } | { ok: false; error: string } | null;")
            lines.append("}")
            lines.append("")
            lines.append("const _asyncHandles = new Map<bigint, AsyncHandle<unknown>>();")
            lines.append("")

        # Generate each interface
        for iface in domain.interfaces:
            lines.append("// ---------------------------------------------------------------------------")
            lines.append(f"// WIT interface: {iface.wit_name}")
            lines.append("// ---------------------------------------------------------------------------")
            lines.append("")

            # Type aliases
            if iface.type_aliases:
                for ta in iface.type_aliases:
                    lines.append(f"/** {ta.docs[0] if ta.docs else 'Type alias'} */")
                    lines.append(f"export type {ta.ts_name} = {ta.ts_type};")
                    lines.append("")

            # Handle table
            if iface.handle_type:
                lines.append(f"/** Handle table for {iface.browser_class} instances */")
                lines.append(f"const _{iface.handle_var} = new Map<bigint, {iface.browser_class}>();")
                lines.append(f"let _next{iface.handle_pascal} = 1n;")
                lines.append("")

                if iface.create_function:
                    lines.append(f"/** Register a new {iface.browser_class} and return its handle. */")
                    lines.append(f"function register{iface.handle_pascal}(obj: {iface.browser_class}): bigint {{")
                    lines.append(f"  const handle = _next{iface.handle_pascal}++;")
                    lines.append(f"  _{iface.handle_var}.set(handle, obj);")
                    lines.append("  return handle;")
                    lines.append("}")
                    lines.append("")

                lines.append(f"/** Get a {iface.browser_class} by handle, throwing if not found. */")
                lines.append(f"function get{iface.handle_pascal}(handle: bigint): {iface.browser_class} {{")
                lines.append(f"  const obj = _{iface.handle_var}.get(handle);")
                lines.append("  if (!obj) {")
                lines.append(f"    throw new Error(`{iface.browser_class} handle ${{handle}} not found`);")
                lines.append("  }")
                lines.append("  return obj;")
                lines.append("}")
                lines.append("")

            # Functions
            for func in iface.functions:
                self._render_function(lines, func, iface, has_async)

        # Export default object
        lines.append("// ---------------------------------------------------------------------------")
        lines.append("// Exports")
        lines.append("// ---------------------------------------------------------------------------")
        lines.append("")
        lines.append("export default {")
        exports = []
        for iface in domain.interfaces:
            for func in iface.functions:
                exports.append(f"  {func.ts_name}")
                if func.is_async and not func.has_explicit_poll:
                    exports.append(f"  poll{func.pascal_name}")
        lines.append(",\n".join(exports))
        lines.append("};")
        lines.append("")

        return "\n".join(lines)

    def _render_function(self, lines: List[str], func: GeneratedFunction,
                         iface: GeneratedInterface, has_async_table: bool) -> None:
        """Render a single function."""

        # Documentation
        docs = func.docs[0] if func.docs else f"`{func.wit_name}()` operation."
        lines.append("/**")
        lines.append(f" * {docs}")
        if func.is_async:
            lines.append(" *")
            lines.append(" * Async operation: returns request ID, poll with " +
                        f"`poll{func.pascal_name}()`")
        lines.append(" */")

        # Function signature
        params_str = ", ".join(f"{p.name}: {p.ts_type}" for p in func.params)
        lines.append(f"export function {func.ts_name}({params_str}): {func.ts_return} {{")

        # Function body
        if func.is_async:
            self._render_async_body(lines, func, iface)
        elif func.is_getter:
            self._render_getter_body(lines, func, iface)
        elif func.is_setter:
            self._render_setter_body(lines, func, iface)
        elif func.is_static:
            self._render_static_body(lines, func, iface)
        else:
            self._render_method_body(lines, func, iface)

        lines.append("}")
        lines.append("")

        # Poll function for async operations (only if no explicit poll function exists)
        if func.is_async and not func.has_explicit_poll:
            self._render_poll_function(lines, func)
            lines.append("")

    def _render_async_body(self, lines: List[str], func: GeneratedFunction,
                          iface: GeneratedInterface) -> None:
        """Render async function body."""
        lines.append("  const requestId = _nextAsyncHandle++;")
        if iface.handle_type and func.params:
            lines.append(f"  const obj = get{iface.handle_pascal}({func.self_param});")
        else:
            lines.append("  // No handle lookup needed")

        obj_ref = "obj" if iface.handle_type else ""
        args = func.browser_args if func.browser_args else ""

        if obj_ref:
            lines.append(f"  const promise = {obj_ref}.{func.browser_method}({args})")
        else:
            # Static or global async function
            lines.append(f"  const promise = {func.browser_class}.{func.browser_method}({args})"
                        if func.browser_class else
                        f"  // TODO: Implement async operation {func.wit_name}")

        lines.append("    .then((result) => {")
        lines.append("      const entry = _asyncHandles.get(requestId);")
        lines.append("      if (entry) {")
        if func.return_is_void:
            lines.append("        entry.result = { ok: true };")
        else:
            lines.append("        entry.result = { ok: true, value: result };")
        lines.append("      }")
        lines.append("    })")
        lines.append("    .catch((err: Error) => {")
        lines.append("      const entry = _asyncHandles.get(requestId);")
        lines.append("      if (entry) {")
        lines.append("        entry.result = { ok: false, error: err.message };")
        lines.append("      }")
        lines.append("    });")
        lines.append("")
        lines.append("  _asyncHandles.set(requestId, { promise, result: null });")
        lines.append("  return requestId;")

    def _render_getter_body(self, lines: List[str], func: GeneratedFunction,
                           iface: GeneratedInterface) -> None:
        """Render getter function body."""
        if iface.handle_type and func.params:
            lines.append(f"  const obj = get{iface.handle_pascal}({func.self_param});")
        if func.return_is_optional:
            lines.append(f"  return obj.{func.browser_attr} ?? undefined;")
        else:
            lines.append(f"  return obj.{func.browser_attr};")

    def _render_setter_body(self, lines: List[str], func: GeneratedFunction,
                           iface: GeneratedInterface) -> None:
        """Render setter function body."""
        if iface.handle_type and func.params:
            lines.append(f"  const obj = get{iface.handle_pascal}({func.self_param});")
        if func.params:
            value_param = func.params[-1].name
            lines.append(f"  obj.{func.browser_attr} = {value_param};")

    def _render_static_body(self, lines: List[str], func: GeneratedFunction,
                           iface: GeneratedInterface) -> None:
        """Render static function body."""
        if func.browser_class:
            lines.append(f"  return {func.browser_class}.{func.browser_method}({func.browser_args});")
        else:
            lines.append(f"  // Static operation: {func.wit_name}")
            lines.append(f"  throw new Error('Static operation not implemented: {func.wit_name}');")

    def _render_method_body(self, lines: List[str], func: GeneratedFunction,
                           iface: GeneratedInterface) -> None:
        """Render instance method body."""
        if iface.handle_type and func.params:
            lines.append(f"  const obj = get{iface.handle_pascal}({func.self_param});")

        obj_ref = "obj" if iface.handle_type else ""
        args = func.browser_args

        if func.return_is_void:
            lines.append(f"  {obj_ref}.{func.browser_method}({args});")
        elif func.return_is_optional:
            lines.append(f"  return {obj_ref}.{func.browser_method}({args}) ?? undefined;")
        else:
            lines.append(f"  return {obj_ref}.{func.browser_method}({args});")

    def _render_poll_function(self, lines: List[str], func: GeneratedFunction) -> None:
        """Render poll function for async operations."""
        lines.append("/**")
        lines.append(f" * Poll an async `{func.ts_name}()` operation.")
        lines.append(" * Returns undefined if still pending, or the result if complete.")
        lines.append(" */")

        result_type = "{ ok: true"
        if not func.return_is_void:
            result_type += f"; value: {func.ts_return_inner}"
        result_type += " } | { ok: false; error: string } | undefined"

        lines.append(f"export function poll{func.pascal_name}(requestId: bigint): {result_type} {{")
        lines.append("  const entry = _asyncHandles.get(requestId);")
        lines.append("  if (!entry) {")
        lines.append("    return { ok: false, error: `Unknown request ID ${requestId}` };")
        lines.append("  }")
        lines.append("  return entry.result ?? undefined;")
        lines.append("}")

    def render_index(self, domains: List[GeneratedDomain]) -> str:
        """Render the main index.ts file."""

        total_interfaces = sum(d.interface_count for d in domains)
        total_functions = sum(
            len(f) for d in domains
            for iface in d.interfaces
            for f in [iface.functions]
        )

        lines: List[str] = []

        lines.append("/**")
        lines.append(" * @tairitsu/browser-glue/generated")
        lines.append(" *")
        lines.append(" * Auto-generated TypeScript glue code for browser WIT interfaces.")
        lines.append(" *")
        lines.append(" * Generated by: scripts/generate_browser_glue.py")
        lines.append(" * Source: packages/browser-worlds/wit/generated/*.wit")
        lines.append(" *")
        lines.append(" * DO NOT EDIT MANUALLY - regenerate with: just glue-gen")
        lines.append(" *")
        lines.append(" * ## Package layout")
        lines.append(" *")

        for domain in domains:
            lines.append(f" * - `./{domain.name}` — `tairitsu-browser:{domain.name}` " +
                        f"({domain.interface_count} interfaces)")

        lines.append(" */")
        lines.append("")
        lines.append("// Re-export all generated modules")

        for domain in domains:
            lines.append(f"export * as {domain.export_name} from \"./generated/{domain.name}-glue.js\";")

        lines.append("")
        lines.append("// Statistics")
        lines.append(f"export const GLUE_STATS = {{")
        lines.append(f"  totalDomains: {len(domains)},")
        lines.append(f"  totalInterfaces: {total_interfaces},")
        lines.append(f"  totalFunctions: {total_functions},")
        lines.append(f"  generatedAt: \"{datetime.now().isoformat()}\",")
        lines.append("};")
        lines.append("")

        return "\n".join(lines)

    def _is_async_function(self, wit_name: str, func: WitFunction, iface_name: str) -> bool:
        """Determine if a function should use async poll pattern."""

        # Poll functions (start with "poll-") are synchronous polling operations
        # They should NOT be treated as async functions themselves
        if wit_name.startswith("poll-"):
            return False

        # Getters/setters are always synchronous
        if wit_name.startswith("get-") or wit_name.startswith("set-"):
            return False

        # Check known async patterns - only match exact function names or
        # functions where the pattern is the main action (not part of "poll-X")
        wit_lower = wit_name.lower()
        for pattern in ASYNC_PATTERNS:
            # Exact match
            if wit_lower == pattern:
                return True
            # Function name like "array-buffer" matching "array-buffer" pattern
            if wit_lower == pattern:
                return True

        return False


# ---------------------------------------------------------------------------
# Main Orchestration
# ---------------------------------------------------------------------------

def find_wit_files(wit_dir: Path) -> List[Tuple[str, Path]]:
    """Find all WIT files in the generated directory."""
    files = []
    for wit_file in sorted(wit_dir.glob("*.wit")):
        domain = wit_file.stem
        files.append((domain, wit_file))
    return files


def run_generate(
    wit_dir: Path,
    output_dir: Path,
    domains: Optional[List[str]] = None,
    *,
    dry_run: bool = False,
    stats: bool = False,
) -> None:
    """Parse WIT files and generate TypeScript glue code."""

    log_info(f"WIT source : {wit_dir}")
    log_info(f"Output     : {output_dir}")
    print()

    # Find WIT files
    wit_files = find_wit_files(wit_dir)

    if not wit_files:
        log_error(f"No WIT files found in {wit_dir}")
        return

    # Filter to requested domains
    if domains:
        wit_files = [(d, p) for d, p in wit_files if d in domains]
        if not wit_files:
            log_error(f"No WIT files found for domains: {domains}")
            return

    # Generate
    generator = BrowserGlueGenerator()
    generated_domains: List[GeneratedDomain] = []

    if stats:
        log_info("WIT → TypeScript Glue Coverage Statistics")
        print("=" * 60)

    total_interfaces = 0
    total_functions = 0

    for domain, wit_path in wit_files:
        gen_domain = generator.generate_domain(domain, wit_path)
        if gen_domain:
            generated_domains.append(gen_domain)
            total_interfaces += gen_domain.interface_count
            total_functions += sum(len(i.functions) for i in gen_domain.interfaces)

            if stats:
                func_count = sum(len(i.functions) for i in gen_domain.interfaces)
                print(f"  {domain:<20} {gen_domain.interface_count:3d} interfaces, "
                      f"{func_count:3d} functions")

    if stats:
        print("=" * 60)
        print(f"  {'TOTAL':<20} {total_interfaces:3d} interfaces, "
              f"{total_functions:3d} functions")
        print()
        return

    # Write output
    if not dry_run:
        output_dir.mkdir(parents=True, exist_ok=True)
        generated_dir = output_dir / "generated"
        generated_dir.mkdir(parents=True, exist_ok=True)

    for gen_domain in generated_domains:
        wit_path = wit_dir / f"{gen_domain.name}.wit"
        content = generator.render_module(gen_domain, str(wit_path))
        dest = output_dir / "generated" / f"{gen_domain.name}-glue.ts"

        if dry_run:
            log_info(f"dry-run: {dest.name} ({len(content):,} bytes)")
        else:
            dest.write_text(content, encoding="utf-8")
            func_count = sum(len(i.functions) for i in gen_domain.interfaces)
            log_ok(f"Wrote {dest.name:<25} {gen_domain.interface_count:2d} interfaces, "
                   f"{func_count:3d} functions")

    # Write index
    if generated_domains:
        index_content = generator.render_index(generated_domains)
        index_dest = output_dir / "generated-index.ts"

        if dry_run:
            log_info(f"dry-run: {index_dest.name} ({len(index_content):,} bytes)")
        else:
            index_dest.write_text(index_content, encoding="utf-8")
            log_ok(f"Wrote {index_dest.name}")

    if not dry_run:
        print()
        log_info(f"Result: {len(generated_domains)} domains generated")


# ---------------------------------------------------------------------------
# CLI Entry Point
# ---------------------------------------------------------------------------

def main() -> None:
    parser = argparse.ArgumentParser(
        description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter,
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
        "--wit-dir",
        metavar="DIR",
        help="Override WIT source directory",
    )
    parser.add_argument(
        "--output-dir",
        metavar="DIR",
        help="Override TypeScript output directory",
    )
    args = parser.parse_args()

    project_root = Path(__file__).parent.parent

    wit_dir = (
        Path(args.wit_dir)
        if args.wit_dir
        else project_root / "packages" / "browser-worlds" / "wit" / "generated"
    )
    output_dir = (
        Path(args.output_dir)
        if args.output_dir
        else project_root / "packages" / "browser-glue" / "src"
    )

    print("=" * 64)
    log_info("Tairitsu WIT -> TypeScript Glue Generator")
    print("=" * 64)
    if args.dry_run:
        log_info("Mode: dry run (no files written)")

    run_generate(
        wit_dir,
        output_dir,
        domains=args.domains,
        dry_run=args.dry_run,
        stats=args.stats,
    )


if __name__ == "__main__":
    main()
