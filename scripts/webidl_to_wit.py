#!/usr/bin/env python3
"""
WebIDL → WIT interface generator.

Reads WebIDL files from scripts/idl-cache/ (downloaded by fetch_w3c_idl.py),
parses the interface/dictionary/enum definitions, and emits WIT files under
packages/browser-worlds/wit/generated/.

Data source
-----------
W3C WebRef (curated branch): https://github.com/w3c/webref
  - Curated, machine-readable IDL extracts for all browser specs
  - Covers W3C, WHATWG, WICG and other standards-track specs

Design decisions
----------------
* Each browser-facing *object* interface (one that has a constructor or
  instance methods) is represented as an opaque handle (u64) because WIT
  does not support pass-by-reference objects natively.  The naming
  convention follows the existing hand-crafted WIT worlds:
    type ws-handle = u64;
    connect: func(url: string) -> result<ws-handle, string>;
* Event-handler attributes (onXxx) are skipped on the host side; a
  companion ``*-callbacks`` interface is generated for the guest-export
  side, matching the pattern in events.wit.
* Unsupported types (Promise<T>, complex unions, …) are mapped to
  ``string`` as a safe fallback with a comment.
* WIT identifiers use kebab-case; this script performs camelCase →
  kebab-case conversion automatically.

Usage
-----
    python scripts/webidl_to_wit.py              # generate all target specs
    python scripts/webidl_to_wit.py websockets   # generate specific specs
    python scripts/webidl_to_wit.py --dry-run    # preview without writing

Output
------
    packages/browser-worlds/wit/generated/<spec>.wit
"""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

# ---------------------------------------------------------------------------
# Type mapping: WebIDL → WIT
# ---------------------------------------------------------------------------

_PRIMITIVE: dict[str, str] = {
    "DOMString":              "string",
    "USVString":              "string",
    "ByteString":             "string",
    "CSSOMString":            "string",
    "DOMHighResTimeStamp":    "f64",
    "DOMTimeStamp":           "u64",
    "boolean":                "bool",
    "byte":                   "s8",
    "octet":                  "u8",
    "short":                  "s16",
    "unsigned short":         "u16",
    "long":                   "s32",
    "unsigned long":          "u32",
    "long long":              "s64",
    "unsigned long long":     "u64",
    "float":                  "f32",
    "unrestricted float":     "f32",
    "double":                 "f64",
    "unrestricted double":    "f64",
    # void-like
    "void":                   "",
    "undefined":              "",
    # binary / buffer types
    "ArrayBuffer":            "list<u8>",
    "ArrayBufferView":        "list<u8>",
    "BufferSource":           "list<u8>",
    "Uint8Array":             "list<u8>",
    "Uint8ClampedArray":      "list<u8>",
    "Int8Array":              "list<s8>",
    "Uint16Array":            "list<u16>",
    "Int16Array":             "list<s16>",
    "Uint32Array":            "list<u32>",
    "Int32Array":             "list<s32>",
    "Float32Array":           "list<f32>",
    "Float64Array":           "list<f64>",
    # DOM structures already modelled as strings/handles
    "object":                 "list<u8>",
    "any":                    "string",
    # event-handler types — skipped at call sites
    "EventHandler":           None,
    "OnErrorEventHandler":    None,
    "VoidFunction":           None,
}


def map_type(idl_type: str) -> Optional[str]:
    """Map a WebIDL type string to a WIT type. Returns None to skip the member."""
    t = idl_type.strip()

    # nullable: T?
    if t.endswith("?"):
        inner = map_type(t[:-1].strip())
        if inner is None or inner == "":
            return None
        return f"option<{inner}>"

    # Promise<T> — not representable in sync WIT; skip
    if t.startswith("Promise<"):
        return None

    # sequence<T> / FrozenArray<T> / ObservableArray<T>
    m = re.match(r"(?:sequence|FrozenArray|ObservableArray)<(.+)>$", t)
    if m:
        inner = map_type(m.group(1).strip())
        if inner is None or inner == "":
            return "list<u8>"      # safe fallback
        return f"list<{inner}>"

    # record<K, V> — approximate as list of pairs; simplify to string
    if re.match(r"record<", t):
        return "string"

    # union (T or U) — approximate as string
    if " or " in t:
        return "string"

    if t in _PRIMITIVE:
        # "" means void/undefined (no return), None means skip (event handlers)
        return _PRIMITIVE[t]

    # Unknown type — will be treated as an opaque handle reference or skipped
    return None


# ---------------------------------------------------------------------------
# Name conversion
# ---------------------------------------------------------------------------

def to_kebab(name: str) -> str:
    """camelCase / PascalCase → kebab-case."""
    s = re.sub(r"([A-Z]+)([A-Z][a-z])", r"\1-\2", name)
    s = re.sub(r"([a-z0-9])([A-Z])", r"\1-\2", s)
    return s.lower().replace("_", "-")


def wit_name(idl_name: str) -> str:
    """Convert an IDL identifier to a valid WIT identifier."""
    k = to_kebab(idl_name)
    # WIT reserved words
    _reserved = {"type", "interface", "world", "record", "enum", "variant",
                 "resource", "use", "import", "export", "package", "func",
                 "result", "option", "list", "tuple", "bool", "string",
                 "u8", "u16", "u32", "u64", "s8", "s16", "s32", "s64", "f32", "f64",
                 "from", "as", "where", "let", "static", "borrow", "own"}
    if k in _reserved:
        k = k + "-val"
    return k


# ---------------------------------------------------------------------------
# WebIDL data structures
# ---------------------------------------------------------------------------

@dataclass
class IDLParam:
    idl_type: str
    name: str
    optional: bool = False


@dataclass
class IDLMember:
    kind: str          # 'attribute' | 'operation' | 'constructor' | 'const'
    name: str
    return_type: str
    params: list[IDLParam] = field(default_factory=list)
    readonly: bool = False
    static: bool = False


@dataclass
class IDLInterface:
    name: str
    parent: Optional[str]
    members: list[IDLMember] = field(default_factory=list)
    partial: bool = False


@dataclass
class IDLDictionary:
    name: str
    parent: Optional[str]
    fields: list[tuple[str, str, bool]] = field(default_factory=list)  # (type, name, required)


@dataclass
class IDLEnum:
    name: str
    values: list[str] = field(default_factory=list)


@dataclass
class IDLTypedef:
    name: str
    idl_type: str


# ---------------------------------------------------------------------------
# WebIDL parser (regex-based, handles common subset)
# ---------------------------------------------------------------------------

def _strip_comments(text: str) -> str:
    text = re.sub(r"/\*.*?\*/", " ", text, flags=re.DOTALL)
    text = re.sub(r"//[^\n]*", "", text)
    return text


def _strip_extended_attrs(text: str) -> str:
    """Remove [ExtendedAttribute ...] blocks."""
    result: list[str] = []
    depth = 0
    for ch in text:
        if ch == "[":
            depth += 1
        elif ch == "]":
            depth -= 1
        elif depth == 0:
            result.append(ch)
    return "".join(result)


def _parse_params(raw: str) -> list[IDLParam]:
    if not raw.strip():
        return []
    # Split by top-level commas
    parts: list[str] = []
    depth = 0
    cur = ""
    for ch in raw:
        if ch in "<(":
            depth += 1
        elif ch in ">)":
            depth -= 1
        if ch == "," and depth == 0:
            parts.append(cur.strip())
            cur = ""
        else:
            cur += ch
    if cur.strip():
        parts.append(cur.strip())

    params: list[IDLParam] = []
    for part in parts:
        part = part.strip()
        if not part:
            continue
        optional = part.startswith("optional ")
        part = re.sub(r"^optional\s+", "", part).strip()
        # Remove default value aggressively: strip `= anything` to end of string.
        # This covers `= []`, `= {}`, `= ""`, `= 0`, `= ` (empty after ext-attr strip).
        # It is safe because `=` never appears in IDL type or parameter names.
        part = re.sub(r"\s*=.*$", "", part).strip()
        # Remove [ExtendedAttribute] markers
        part = re.sub(r"\[[^\]]*\]\s*", "", part).strip()
        if not part:
            continue
        # Split into type and name: last token is the name
        tokens = part.rsplit(None, 1)
        if len(tokens) == 2:
            t_type = tokens[0].strip()
            t_name = tokens[1].strip()
            # Guard: name must be a valid identifier (not `=` or similar)
            if t_name and re.match(r"^[a-zA-Z_]\w*$", t_name):
                params.append(IDLParam(idl_type=t_type,
                                       name=t_name,
                                       optional=optional))
    return params


def _extract_parens(text: str) -> Optional[tuple[str, str]]:
    """Find the content of the outermost (...) at the start of `text`.

    Returns (inner_content, rest_after_close) or None if no `(` found.
    """
    i = text.find("(")
    if i < 0:
        return None
    depth = 0
    for j in range(i, len(text)):
        if text[j] == "(":
            depth += 1
        elif text[j] == ")":
            depth -= 1
            if depth == 0:
                return text[i + 1:j], text[j + 1:]
    return None


def _parse_members(body: str) -> list[IDLMember]:
    body = _strip_extended_attrs(body)
    members: list[IDLMember] = []
    # Track emitted operation names to deduplicate overloads
    emitted_ops: set[str] = set()
    # Split on semicolons to get individual declarations
    for stmt in body.split(";"):
        stmt = stmt.strip()
        if not stmt:
            continue

        # const — skip
        if stmt.startswith("const "):
            continue

        # iterable, maplike, setlike — skip
        if re.match(r"(?:iterable|maplike|setlike)<", stmt):
            continue

        # constructor — must be parsed before checking for operations
        if re.match(r"constructor\s*\(", stmt):
            paren = _extract_parens(stmt)
            if paren is not None:
                params_str, _ = paren
                members.append(IDLMember(
                    kind="constructor", name="constructor",
                    return_type="self",
                    params=_parse_params(params_str),
                ))
            continue

        static = bool(re.match(r"static\s+", stmt))
        stmt = re.sub(r"^static\s+", "", stmt).strip()

        readonly = bool(re.match(r"readonly\s+", stmt))
        stmt = re.sub(r"^readonly\s+", "", stmt).strip()

        # attribute
        m = re.match(r"attribute\s+(.+)", stmt)
        if m:
            rest = m.group(1).strip()
            # type is everything but last token
            tokens = rest.rsplit(None, 1)
            if len(tokens) == 2:
                members.append(IDLMember(
                    kind="attribute", name=tokens[1], return_type=tokens[0],
                    readonly=readonly, static=static,
                ))
            continue

        # inherit attribute (rare)
        m = re.match(r"inherit\s+attribute\s+(.+)", stmt)
        if m:
            rest = m.group(1).strip()
            tokens = rest.rsplit(None, 1)
            if len(tokens) == 2:
                members.append(IDLMember(
                    kind="attribute", name=tokens[1], return_type=tokens[0],
                    readonly=True, static=False,
                ))
            continue

        # operation: returnType methodName(params)
        # Use _extract_parens to handle nested parens in params
        paren_idx = stmt.find("(")
        if paren_idx > 0:
            before_paren = stmt[:paren_idx].strip()
            tokens = before_paren.rsplit(None, 1)
            if len(tokens) == 2:
                ret_type = tokens[0].strip()
                op_name = tokens[1].strip()
                if op_name in ("stringifier", "serializer", "getter", "setter",
                               "deleter", "legacycaller"):
                    continue
                # Deduplicate overloads: keep first occurrence
                if op_name in emitted_ops:
                    continue
                emitted_ops.add(op_name)
                paren_result = _extract_parens(stmt[paren_idx:])
                params_str = paren_result[0] if paren_result else ""
                members.append(IDLMember(
                    kind="operation", name=op_name, return_type=ret_type,
                    params=_parse_params(params_str),
                    static=static,
                ))

    return members


def _parse_dict_fields(body: str) -> list[tuple[str, str, bool]]:
    """Parse field declarations from a WebIDL dictionary body.

    Handles multi-word primitive types (``unsigned long``, ``unsigned long long``,
    ``unrestricted double``, etc.) by normalising them before splitting.

    Returns list of (idl_type, field_name, required).
    """
    # Normalize multi-word primitive type tokens so they appear as single tokens
    # when we split on whitespace.  Restore them after splitting.
    _MULTI_WORD = [
        ("unsigned long long", "%%ULL%%"),
        ("unsigned long",      "%%UL%%"),
        ("unsigned short",     "%%US%%"),
        ("unrestricted double", "%%URD%%"),
        ("unrestricted float",  "%%URF%%"),
        ("long long",          "%%LL%%"),
    ]
    fields: list[tuple[str, str, bool]] = []
    for stmt in body.split(";"):
        stmt = stmt.strip()
        if not stmt:
            continue
        required = stmt.startswith("required ")
        stmt = re.sub(r"^required\s+", "", stmt).strip()
        # Strip extended attributes
        stmt = _strip_extended_attrs(stmt).strip()
        # Strip default value (= ...)
        stmt = re.sub(r"\s*=.*$", "", stmt).strip()
        if not stmt:
            continue
        # Normalize multi-word types
        for long_form, token in _MULTI_WORD:
            stmt = re.sub(r"\b" + long_form.replace(" ", r"\s+") + r"\b",
                          token, stmt)
        # Split into tokens: type [nullable?] name
        tokens = stmt.split()
        if len(tokens) < 2:
            continue
        f_name = tokens[-1].strip("?")
        f_type = " ".join(tokens[:-1]).strip()
        # Restore multi-word types
        for long_form, token in _MULTI_WORD:
            f_type = f_type.replace(token, long_form)
        if f_name and re.match(r"^[a-zA-Z_]\w*$", f_name) and f_type:
            fields.append((f_type, f_name, required))
    return fields


def parse_webidl(
    idl_text: str,
) -> tuple[list[IDLInterface], list[IDLDictionary], list[IDLEnum], list[IDLTypedef]]:
    """Parse WebIDL text into structured data."""
    clean = _strip_comments(idl_text)

    typedefs: list[IDLTypedef] = []
    for m in re.finditer(r"typedef\s+(.+?)\s+(\w+)\s*;", clean):
        typedefs.append(IDLTypedef(name=m.group(2), idl_type=m.group(1).strip()))

    enums: list[IDLEnum] = []
    for m in re.finditer(r"enum\s+(\w+)\s*\{([^}]*)\}", clean, re.DOTALL):
        values = re.findall(r'"([^"]+)"', m.group(2))
        enums.append(IDLEnum(name=m.group(1), values=values))

    dictionaries: list[IDLDictionary] = []
    for m in re.finditer(
        r"(?:partial\s+)?dictionary\s+(\w+)(?:\s*:\s*(\w+))?\s*\{([^}]*)\}",
        clean, re.DOTALL
    ):
        fields = _parse_dict_fields(m.group(3))
        dictionaries.append(IDLDictionary(
            name=m.group(1), parent=m.group(2), fields=fields,
        ))

    interfaces: list[IDLInterface] = []
    for m in re.finditer(
        r"(partial\s+)?(?:interface(?:\s+mixin)?)\s+(\w+)"
        r"(?:\s*:\s*(\w+))?\s*\{([^}]*)\}",
        clean, re.DOTALL,
    ):
        partial = m.group(1) is not None
        members = _parse_members(m.group(4))
        interfaces.append(IDLInterface(
            name=m.group(2), parent=m.group(3),
            members=members, partial=partial,
        ))

    return interfaces, dictionaries, enums, typedefs


# ---------------------------------------------------------------------------
# WIT emitter
# ---------------------------------------------------------------------------

_INDENT = "    "


def _wit_type(idl_type: str, *, fallback: str = "string") -> str:
    """Return WIT type string; use *fallback* when mapping is unknown."""
    t = map_type(idl_type)
    return t if (t is not None and t != "") else fallback


class WITWriter:
    """Builds a .wit file as a list of lines."""

    def __init__(self) -> None:
        self._lines: list[str] = []

    def blank(self, n: int = 1) -> None:
        self._lines.extend([""] * n)

    def line(self, text: str = "") -> None:
        self._lines.append(text)

    def comment(self, text: str) -> None:
        self._lines.append(f"/// {text}")

    def render(self) -> str:
        return "\n".join(self._lines) + "\n"


def emit_enum(w: WITWriter, idl_enum: IDLEnum) -> None:
    if not idl_enum.values:
        return
    w.comment(f"WebIDL enum {idl_enum.name}.")
    w.line(f"enum {wit_name(idl_enum.name)} {{")
    for i, v in enumerate(idl_enum.values):
        safe = wit_name(v.replace("-", "_"))
        w.line(f"{_INDENT}{safe},")
    w.line("}")
    w.blank()


def emit_record(w: WITWriter, d: IDLDictionary) -> None:
    w.comment(f"WebIDL dictionary {d.name}.")
    w.line(f"record {wit_name(d.name)} {{")
    for f_type, f_name, _required in d.fields:
        wt = map_type(f_type)
        if wt is None:
            wt = "string"     # fallback
        if wt == "":
            continue
        w.line(f"{_INDENT}{wit_name(f_name)}: {wt},")
    w.line("}")
    w.blank()


def _emit_operation(
    w: WITWriter,
    member: IDLMember,
    handle_type: Optional[str],
    prefix: str = "",
) -> None:
    """Emit a single WIT function for an IDL operation."""
    func_name = wit_name(prefix + member.name if prefix else member.name)

    params: list[str] = []
    if handle_type and not member.static and member.kind != "constructor":
        params.append(f"handle: {handle_type}")

    skip = False
    for p in member.params:
        wt = map_type(p.idl_type)
        if wt is None:
            # Skip members whose param types can't be mapped
            skip = True
            break
        if wt == "":
            continue
        params.append(f"{wit_name(p.name)}: {wt}")

    if skip:
        w.line(f"{_INDENT}// {func_name}: skipped (unmappable parameter type)")
        return

    ret = map_type(member.return_type)
    if ret is None:
        if handle_type and member.kind == "constructor":
            ret_clause = f" -> result<{handle_type}, string>"
        else:
            w.line(f"{_INDENT}// {func_name}: skipped (unmappable return type)")
            return
    elif ret == "":
        if handle_type and member.kind == "constructor":
            ret_clause = f" -> result<{handle_type}, string>"
        else:
            ret_clause = ""
    else:
        if handle_type and member.kind == "constructor":
            ret_clause = f" -> result<{handle_type}, string>"
        else:
            ret_clause = f" -> {ret}"

    param_str = ", ".join(params)
    w.line(f"{_INDENT}{func_name}: func({param_str}){ret_clause};")


def emit_interface(
    w: WITWriter,
    iface: IDLInterface,
    handle_type: Optional[str] = None,
    constructor_func: Optional[str] = None,
    skipped_attrs: Optional[set[str]] = None,
) -> None:
    """Emit a WIT interface block for an IDL interface.

    Parameters
    ----------
    handle_type:
        WIT type alias name for the opaque handle (e.g. ``ws-handle``).
        If None the interface is emitted without a handle.
    constructor_func:
        Override name for the constructor function (e.g. ``connect``).
    skipped_attrs:
        Set of attribute names that should be silently dropped.
    """
    if skipped_attrs is None:
        skipped_attrs = set()

    iface_wit_name = wit_name(iface.name)
    w.comment(f"WebIDL interface {iface.name}.")
    if iface.parent:
        w.comment(f"Extends: {iface.parent}.")
    w.line(f"interface {iface_wit_name} {{")

    if handle_type:
        w.comment(f"Opaque handle to a {iface.name} instance.")
        w.line(f"{_INDENT}type {handle_type} = u64;")
        w.blank()

    for member in iface.members:
        if member.name in skipped_attrs:
            continue

        if member.kind == "const":
            continue

        # Skip EventHandler attributes — they go in the callbacks interface
        if member.kind == "attribute" and map_type(member.return_type) is None:
            continue

        if member.kind == "constructor":
            ctor_name = constructor_func or f"new-{iface_wit_name}"
            # Rewrite the operation with the custom name
            pseudo = IDLMember(
                kind="constructor", name=ctor_name,
                return_type=member.return_type,
                params=member.params,
            )
            # Emit without handle prefix (constructor returns the handle)
            _emit_operation(w, pseudo, handle_type=handle_type)
            continue

        if member.kind == "attribute":
            # getter
            getter = IDLMember(
                kind="operation",
                name=f"get-{member.name}" if not member.readonly else member.name,
                return_type=member.return_type,
                params=[],
                static=member.static,
            )
            _emit_operation(w, getter, handle_type=handle_type)
            # setter (only for non-readonly)
            if not member.readonly:
                setter = IDLMember(
                    kind="operation",
                    name=f"set-{member.name}",
                    return_type="undefined",
                    params=[IDLParam(idl_type=member.return_type, name="value")],
                    static=member.static,
                )
                _emit_operation(w, setter, handle_type=handle_type)
            continue

        if member.kind == "operation":
            _emit_operation(w, member, handle_type=handle_type)

    w.line("}")
    w.blank()


# ---------------------------------------------------------------------------
# Spec-specific generation targets
# ---------------------------------------------------------------------------

@dataclass
class SpecTarget:
    """Describes how to convert one IDL file into a WIT package file."""
    spec_name: str       # key in SPECS dict / idl-cache filename stem
    package: str         # WIT package identifier, e.g. "tairitsu-browser:websocket"
    version: str         # WIT package version
    description: str     # one-line description for the file header
    # Which IDL interfaces to include, with optional renaming
    # Value: (handle_type, constructor_func_override)
    interfaces: dict[str, tuple[Optional[str], Optional[str]]] = field(
        default_factory=dict
    )
    dicts: list[str] = field(default_factory=list)      # dictionary names to include
    enums: list[str] = field(default_factory=list)      # enum names to include
    # Name of the WIT world to emit
    world_name: Optional[str] = None
    world_imports: list[str] = field(default_factory=list)
    world_exports: list[str] = field(default_factory=list)


TARGETS: list[SpecTarget] = [
    SpecTarget(
        spec_name="websockets",
        package="tairitsu-browser:websocket",
        version="0.1.0",
        description="WebSocket API — https://websockets.spec.whatwg.org/",
        interfaces={
            "WebSocket": ("ws-handle", "connect"),
        },
        enums=["BinaryType"],
        world_name="websocket-world",
        # world_imports auto-derived from interface names → ["web-socket"]
    ),
    SpecTarget(
        spec_name="streams",
        package="tairitsu-browser:streams",
        version="0.1.0",
        description="WHATWG Streams Standard — https://streams.spec.whatwg.org/",
        interfaces={
            "ReadableStream":  ("readable-handle",  "new-readable-stream"),
            "WritableStream":  ("writable-handle",  "new-writable-stream"),
            "TransformStream": ("transform-handle", "new-transform-stream"),
        },
        enums=["ReadableStreamType"],
        world_name="streams-world",
        # world_imports auto-derived → ["readable-stream", "writable-stream", "transform-stream"]
    ),
    SpecTarget(
        spec_name="html",
        package="tairitsu-browser:storage",
        version="0.1.0",
        description=(
            "Web Storage API (localStorage / sessionStorage) — "
            "https://html.spec.whatwg.org/multipage/webstorage.html"
        ),
        interfaces={
            "Storage": ("storage-handle", "open-storage"),
        },
        world_name="storage-world",
        # world_imports auto-derived → ["storage"]
    ),
    SpecTarget(
        spec_name="service-workers",
        package="tairitsu-browser:workers",
        version="0.1.0",
        description=(
            "Service Workers + Worker API — "
            "https://w3c.github.io/ServiceWorker/"
        ),
        interfaces={
            "ServiceWorkerRegistration": ("sw-reg-handle", "register-service-worker"),
            "ServiceWorker":             ("sw-handle",     None),
        },
        world_name="workers-world",
        # world_imports auto-derived → ["service-worker-registration", "service-worker"]
    ),
    SpecTarget(
        spec_name="file-api",
        package="tairitsu-browser:file-api",
        version="0.1.0",
        description="File API — https://w3c.github.io/FileAPI/",
        interfaces={
            "Blob":       ("blob-handle",       "new-blob"),
            "File":       ("file-handle",       None),
            "FileReader": ("file-reader-handle", "new-file-reader"),
            "FileList":   ("file-list-handle",  None),
        },
        enums=["EndingType"],
        dicts=["BlobPropertyBag", "FilePropertyBag"],
        world_name="file-api-world",
        # world_imports auto-derived → ["blob", "file", "file-reader", "file-list"]
    ),
    SpecTarget(
        spec_name="indexed-db",
        package="tairitsu-browser:indexed-db",
        version="0.1.0",
        description="IndexedDB API — https://www.w3.org/TR/IndexedDB/",
        interfaces={
            "IDBFactory":          ("idb-factory-handle",    None),
            "IDBDatabase":         ("idb-db-handle",         "idb-open"),
            "IDBTransaction":      ("idb-tx-handle",         None),
            "IDBObjectStore":      ("idb-store-handle",      None),
            "IDBIndex":            ("idb-index-handle",      None),
            "IDBCursor":           ("idb-cursor-handle",     None),
            "IDBRequest":          ("idb-request-handle",    None),
        },
        enums=["IDBTransactionMode", "IDBRequestReadyState", "IDBCursorDirection"],
        dicts=["IDBVersionChangeEventInit"],
        world_name="indexed-db-world",
        # world_imports auto-derived
    ),
    SpecTarget(
        spec_name="geolocation",
        package="tairitsu-browser:geolocation",
        version="0.1.0",
        description="Geolocation API — https://www.w3.org/TR/geolocation/",
        interfaces={
            "Geolocation": ("geo-handle", None),
        },
        dicts=["PositionOptions", "GeolocationCoordinates",
               "GeolocationPosition", "GeolocationPositionError"],
        world_name="geolocation-world",
        # world_imports auto-derived → ["geolocation"]
    ),
    SpecTarget(
        spec_name="intersection-observer",
        package="tairitsu-browser:intersection-observer",
        version="0.1.0",
        description="Intersection Observer — https://w3c.github.io/IntersectionObserver/",
        interfaces={
            "IntersectionObserver": ("io-handle", "new-intersection-observer"),
        },
        dicts=["IntersectionObserverEntry", "IntersectionObserverInit"],
        world_name="intersection-observer-world",
        # world_imports auto-derived → ["intersection-observer"]
    ),
    SpecTarget(
        spec_name="resize-observer",
        package="tairitsu-browser:resize-observer",
        version="0.1.0",
        description="Resize Observer — https://drafts.csswg.org/resize-observer/",
        interfaces={
            "ResizeObserver": ("ro-handle", "new-resize-observer"),
        },
        dicts=["ResizeObserverEntry", "ResizeObserverOptions"],
        enums=["ResizeObserverBoxOptions"],
        world_name="resize-observer-world",
        # world_imports auto-derived → ["resize-observer"]
    ),
    SpecTarget(
        spec_name="web-animations",
        package="tairitsu-browser:web-animations",
        version="0.1.0",
        description="Web Animations — https://drafts.csswg.org/web-animations-1/",
        interfaces={
            "Animation":         ("animation-handle",  "new-animation"),
            "KeyframeEffect":    ("keyframe-handle",   "new-keyframe-effect"),
            "AnimationTimeline": ("timeline-handle",   None),
        },
        enums=["FillMode", "PlaybackDirection", "CompositeOperation",
               "AnimationPlayState", "AnimationReplaceState"],
        dicts=["KeyframeEffectOptions", "KeyframeAnimationOptions",
               "DocumentTimelineOptions"],
        world_name="web-animations-world",
        # world_imports auto-derived → ["animation", "keyframe-effect", "animation-timeline"]
    ),
]


# ---------------------------------------------------------------------------
# File generator
# ---------------------------------------------------------------------------

def _file_header(pkg: str, version: str, description: str, spec_name: str) -> str:
    """Generate the top-of-file comment block."""
    return (
        f"/// {description}\n"
        f"///\n"
        f"/// THIS FILE IS AUTO-GENERATED by scripts/webidl_to_wit.py\n"
        f"/// Source: W3C WebRef (https://github.com/w3c/webref, curated branch)\n"
        f"/// IDL spec: scripts/idl-cache/{spec_name}.idl\n"
        f"///\n"
        f"/// Do not edit manually — re-run `just gen-wit` to regenerate.\n"
    )


def generate_wit(
    target: SpecTarget,
    idl_text: str,
) -> str:
    """Generate a WIT file string for the given target and IDL source."""
    interfaces, dictionaries, enums, _typedefs = parse_webidl(idl_text)

    # Index by name for fast lookup
    iface_by_name = {i.name: i for i in interfaces}
    dict_by_name  = {d.name: d for d in dictionaries}
    enum_by_name  = {e.name: e for e in enums}

    w = WITWriter()

    # File header
    for line in _file_header(
        target.package, target.version, target.description, target.spec_name
    ).splitlines():
        w.line(line)
    w.blank()

    # Package declaration
    w.line(f"package {target.package}@{target.version};")
    w.blank()

    # Enums and records must be inside an interface in WIT
    # Create a "types" interface if we have any enums or dicts
    has_types = target.enums or target.dicts
    if has_types:
        w.comment("Type definitions (enums and records from WebIDL).")
        w.line("interface types {")
        # Enums
        for enum_name in target.enums:
            e = enum_by_name.get(enum_name)
            if e:
                w.line(f"{_INDENT}/// WebIDL enum {e.name}.")
                w.line(f"{_INDENT}enum {wit_name(e.name)} {{")
                for v in e.values:
                    safe = wit_name(v.replace("-", "_"))
                    w.line(f"{_INDENT}{_INDENT}{safe},")
                w.line(f"{_INDENT}}}")
            else:
                w.line(f"{_INDENT}// NOTE: enum {enum_name} not found in IDL source")
        # Dictionaries / records
        for dict_name in target.dicts:
            d = dict_by_name.get(dict_name)
            if d:
                w.line(f"{_INDENT}/// WebIDL dictionary {d.name}.")
                w.line(f"{_INDENT}record {wit_name(d.name)} {{")
                for f_type, f_name, _required in d.fields:
                    wt = map_type(f_type)
                    if wt is None:
                        wt = "string"     # fallback
                    if wt == "":
                        continue
                    w.line(f"{_INDENT}{_INDENT}{wit_name(f_name)}: {wt},")
                w.line(f"{_INDENT}}}")
            else:
                w.line(f"{_INDENT}// NOTE: dictionary {dict_name} not found in IDL source")
        w.line("}")
        w.blank()

    # Interfaces
    for iface_name, (handle_type, ctor_override) in target.interfaces.items():
        iface = iface_by_name.get(iface_name)
        if iface:
            emit_interface(w, iface,
                           handle_type=handle_type,
                           constructor_func=ctor_override)
        else:
            iface_wit = wit_name(iface_name)
            w.comment(f"NOTE: interface {iface_name} not found in IDL source")
            w.line(f"interface {iface_wit} {{")
            if handle_type:
                w.comment(f"Opaque handle (IDL not available in this spec).")
                w.line(f"{_INDENT}type {handle_type} = u64;")
            w.line("}")
            w.blank()

    # World — auto-derive imports from interface names if not explicitly set
    if target.world_name:
        # Use target.world_imports if explicitly set, else derive from interface names
        world_imports = (
            target.world_imports
            if target.world_imports
            else [wit_name(n) for n in target.interfaces]
        )
        w.comment(
            f"WIT world for {target.package.split(':')[-1]} — "
            "import into your WASM component."
        )
        w.line(f"world {target.world_name} {{")
        for imp in world_imports:
            w.line(f"{_INDENT}import {imp};")
        for exp in target.world_exports:
            w.line(f"{_INDENT}export {exp};")
        w.line("}")
        w.blank()

    return w.render()


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

# Filename overrides for specs where the IDL filename doesn't match the
# spec_name exactly (copied from fetch_w3c_idl.py SPECS table)
_IDL_FILENAME_OVERRIDES: dict[str, str] = {
    "file-api":    "FileAPI.idl",
    "indexed-db":  "IndexedDB.idl",
    "cookie-store": "cookiestore.idl",
}


def _find_idl_file(cache_dir: Path, spec_name: str) -> Optional[Path]:
    """Locate the cached IDL file for a spec, trying common filename variants."""
    candidates = [
        cache_dir / f"{spec_name}.idl",
        cache_dir / f"{spec_name}.webidl",
    ]
    override = _IDL_FILENAME_OVERRIDES.get(spec_name)
    if override:
        candidates.insert(0, cache_dir / override)
    for c in candidates:
        if c.exists():
            return c
    return None


def main() -> int:
    script_dir = Path(__file__).parent
    default_cache   = script_dir / "idl-cache"
    default_out_dir = (script_dir.parent
                       / "packages" / "browser-worlds" / "wit" / "generated")

    parser = argparse.ArgumentParser(
        description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "specs", nargs="*",
        help=(
            "Target spec names to generate "
            "(default: all configured targets). "
            "Available: " + ", ".join(t.spec_name for t in TARGETS)
        ),
    )
    parser.add_argument(
        "--cache-dir", default=str(default_cache), metavar="DIR",
        help=f"Directory containing downloaded IDL files (default: {default_cache})",
    )
    parser.add_argument(
        "--out-dir", default=str(default_out_dir), metavar="DIR",
        help=f"Output directory for generated WIT files (default: {default_out_dir})",
    )
    parser.add_argument(
        "--dry-run", action="store_true",
        help="Print generated WIT to stdout instead of writing files",
    )
    parser.add_argument(
        "--list", action="store_true",
        help="List configured generation targets and exit",
    )
    args = parser.parse_args()

    if args.list:
        print("Configured generation targets:")
        for t in TARGETS:
            print(f"  {t.spec_name:<30} → {t.package}@{t.version}")
        return 0

    cache_dir = Path(args.cache_dir)
    out_dir   = Path(args.out_dir)

    # Filter targets
    requested = set(args.specs) if args.specs else {t.spec_name for t in TARGETS}
    unknown = requested - {t.spec_name for t in TARGETS}
    if unknown:
        print(f"[ERROR] Unknown target(s): {', '.join(sorted(unknown))}", file=sys.stderr)
        return 1

    active = [t for t in TARGETS if t.spec_name in requested]

    if not args.dry_run:
        out_dir.mkdir(parents=True, exist_ok=True)

    errors = 0
    for target in active:
        idl_path = _find_idl_file(cache_dir, target.spec_name)

        if idl_path is None or not idl_path.exists():
            print(
                f"  [WARN] IDL cache miss for {target.spec_name!r} — "
                f"run `just gen-wit-fetch` first",
                file=sys.stderr,
            )
            errors += 1
            continue

        idl_text = idl_path.read_text(encoding="utf-8")
        wit_text = generate_wit(target, idl_text)

        if args.dry_run:
            print(f"{'=' * 70}")
            print(f"# {target.spec_name} → {target.package}@{target.version}")
            print(f"{'=' * 70}")
            print(wit_text)
        else:
            out_path = out_dir / f"{target.spec_name}.wit"
            out_path.write_text(wit_text, encoding="utf-8")
            lines = wit_text.count("\n")
            print(f"  [OK] {target.spec_name:<30} → {out_path}  ({lines} lines)")

    print()
    if errors:
        print(f"[WARN] {errors} target(s) skipped (missing IDL cache)")
    else:
        print(f"[OK] Generated {len(active)} WIT file(s)")

    return 0 if errors == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
