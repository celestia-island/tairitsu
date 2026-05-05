#!/usr/bin/env python3
"""
WIT (WebAssembly Interface Types) Parser

Parses WIT syntax into an AST representation for code generation.
Supports the WIT format used by wasm-tools/wit-component.

Usage:
    from wit_parser import parse_wit_file, WitPackage

    pkg = parse_wit_file(Path("storage.wit"))
    for iface in pkg.interfaces:
        for func in iface.functions:
            print(f"{iface.name}.{func.name}: {func.params} -> {func.result}")
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import List, Optional, Dict, Tuple, Union


# ---------------------------------------------------------------------------
# AST Types
# ---------------------------------------------------------------------------

@dataclass
class WitType:
    """Base class for WIT types."""
    pass


@dataclass
class WitPrimitive(WitType):
    """Primitive types: u8, u16, u32, u64, s8, s16, s32, s64, f32, f64, bool, string, char."""
    name: str


@dataclass
class WitHandle(WitType):
    """Opaque handle type (alias to u64)."""
    name: str  # e.g., "storage-manager-handle"


@dataclass
class WitOption(WitType):
    """Option type: option<T>"""
    inner: WitType


@dataclass
class WitList(WitType):
    """List type: list<T>"""
    inner: WitType


@dataclass
class WitResult(WitType):
    """Result type: result<T, E> or result<T> or result<_, E>"""
    ok: Optional[WitType]  # None means "_"
    err: Optional[WitType]  # None means no error type


@dataclass
class WitTuple(WitType):
    """Tuple type: tuple<T1, T2, ...>"""
    elements: List[WitType]


@dataclass
class WitRecord(WitType):
    """Record type reference."""
    name: str


@dataclass
class WitEnum(WitType):
    """Enum type reference."""
    name: str


@dataclass
class WitVariant(WitType):
    """Variant type reference."""
    name: str


@dataclass
class WitFlags(WitType):
    """Flags type reference."""
    name: str


@dataclass
class WitParam:
    """Function parameter."""
    name: str
    type_: WitType


@dataclass
class WitFunction:
    """Interface function."""
    name: str
    params: List[WitParam]
    result: Optional[WitType]
    is_static: bool = False
    docs: List[str] = field(default_factory=list)


@dataclass
class WitTypeAlias:
    """Type alias definition."""
    name: str
    target: WitType
    docs: List[str] = field(default_factory=list)


@dataclass
class WitInterface:
    """WIT interface definition."""
    name: str
    functions: List[WitFunction] = field(default_factory=list)
    type_aliases: List[WitTypeAlias] = field(default_factory=list)
    docs: List[str] = field(default_factory=list)
    source_file: str = ""


@dataclass
class WitWorld:
    """WIT world definition."""
    name: str
    imports: List[str] = field(default_factory=list)  # Interface names
    exports: List[str] = field(default_factory=list)  # Interface names
    docs: List[str] = field(default_factory=list)


@dataclass
class WitPackage:
    """Complete WIT package."""
    name: str
    version: str
    interfaces: List[WitInterface] = field(default_factory=list)
    worlds: List[WitWorld] = field(default_factory=list)
    docs: List[str] = field(default_factory=list)
    source_file: str = ""


# ---------------------------------------------------------------------------
# Tokenizer
# ---------------------------------------------------------------------------

TOKEN_PATTERNS = [
    ('COMMENT_LINE', r'///[^\n]*'),
    ('COMMENT_REGULAR', r'//[^\n]*'),  # Regular single-line comments
    ('COMMENT_BLOCK', r'/\*[\s\S]*?\*/'),
    ('WHITESPACE', r'\s+'),
    ('PACKAGE', r'\bpackage\b(?!\-)'),  # Not followed by hyphen
    ('INTERFACE', r'\binterface\b(?!\-)'),
    ('WORLD', r'\bworld\b(?!\-)'),
    ('TYPE', r'\btype\b(?!\-)'),
    ('FUNC', r'\bfunc\b(?!\-)'),
    ('IMPORT', r'\bimport\b(?!\-)'),
    ('EXPORT', r'\bexport\b(?!\-)'),
    ('FROM', r'\bfrom\b(?!\-)'),
    ('INCLUDE', r'\binclude\b(?!\-)'),
    ('USE', r'\buse\b(?!\-)'),
    ('OPTION', r'\boption\b(?!\-)'),
    ('LIST', r'\blist\b(?!\-)'),
    ('RESULT', r'\bresult\b(?!\-)'),
    ('TUPLE', r'\btuple\b(?!\-)'),
    ('RECORD', r'\brecord\b(?!\-)'),
    ('ENUM', r'\benum\b(?!\-)'),
    ('VARIANT', r'\bvariant\b(?!\-)'),
    ('FLAGS', r'\bflags\b(?!\-)'),
    ('RESOURCE', r'\bresource\b(?!\-)'),
    ('STATIC', r'\bstatic\b(?!\-)'),
    ('ESCAPED_IDENT', r'%[a-zA-Z_][a-zA-Z0-9_-]*'),  # # e.g., %type, %use
    ('PRIMITIVE', r'\b(u8|u16|u32|u64|s8|s16|s32|s64|f32|f64|bool|string|char|_)\b(?!\-)'),
    ('IDENT', r'[a-zA-Z_][a-zA-Z0-9_-]*'),
    ('STRING', r'"[^"]*"'),
    ('VERSION', r'\d+\.\d+\.\d+'),  # semver: 0.2.0
    ('NUMBER', r'\d+'),
    ('LBRACE', r'\{'),
    ('RBRACE', r'\}'),
    ('LPAREN', r'\('),
    ('RPAREN', r'\)'),
    ('LBRACKET', r'\['),
    ('RBRACKET', r'\]'),
    ('LANGLE', r'<'),
    ('RANGLE', r'>'),
    ('COLON', r':'),
    ('SEMICOLON', r';'),
    ('COMMA', r','),
    ('DOT', r'\.'),
    ('EQUALS', r'='),
    ('ARROW', r'->'),
    ('AT', r'@'),
]


@dataclass
class Token:
    kind: str
    value: str
    line: int
    column: int


def tokenize(text: str) -> List[Token]:
    """Tokenize WIT source text."""
    combined = '|'.join(f'(?P<{name}>{pattern})' for name, pattern in TOKEN_PATTERNS)
    regex = re.compile(combined)

    tokens: List[Token] = []
    line = 1
    column = 1

    for match in regex.finditer(text):
        kind = match.lastgroup
        value = match.group()

        if kind == 'WHITESPACE':
            # Track position but don't emit
            line += value.count('\n')
            if '\n' in value:
                column = len(value.rsplit('\n', 1)[-1]) + 1
            else:
                column += len(value)
            continue

        if kind in ('COMMENT_LINE', 'COMMENT_BLOCK', 'COMMENT_REGULAR'):
            # Track position but don't emit (comments are handled separately)
            line += value.count('\n')
            if '\n' in value:
                column = len(value.rsplit('\n', 1)[-1]) + 1
            else:
                column += len(value)
            continue

        tokens.append(Token(kind, value, line, column))
        column += len(value)

    # Post-process: handle keywords that should be treated as identifiers
    # 1. keyword + hyphen + identifier -> merge into single IDENT (e.g., "type-val")
    # 2. keyword followed by colon -> convert to IDENT (e.g., "from:" as param name)
    merged_tokens: List[Token] = []
    i = 0
    KEYWORDS = {'PACKAGE', 'INTERFACE', 'WORLD', 'TYPE', 'FUNC', 'IMPORT', 'EXPORT',
                'FROM', 'INCLUDE', 'USE', 'OPTION', 'LIST', 'RESULT', 'TUPLE',
                'RECORD', 'ENUM', 'VARIANT', 'FLAGS', 'RESOURCE', 'STATIC',
                'PRIMITIVE'}

    while i < len(tokens):
        tok = tokens[i]

        # Check if this is a keyword that should be treated as identifier
        if tok.kind in KEYWORDS:
            # Case 1: keyword + hyphen + identifier -> merge into single IDENT
            if (i + 2 < len(tokens) and
                tokens[i + 1].value == '-' and
                tokens[i + 2].kind in ('IDENT', 'PRIMITIVE', 'ESCAPED_IDENT', *KEYWORDS)):
                # Merge into a single IDENT token
                merged_value = tok.value + '-' + tokens[i + 2].value
                merged_tokens.append(Token('IDENT', merged_value, tok.line, tok.column))
                i += 3
                continue

            # Case 2: keyword followed by colon (parameter name context)
            if i + 1 < len(tokens) and tokens[i + 1].kind == 'COLON':
                # Convert keyword to IDENT for parameter names like "from:"
                merged_tokens.append(Token('IDENT', tok.value, tok.line, tok.column))
                i += 1
                continue

        merged_tokens.append(tok)
        i += 1

    return merged_tokens


# ---------------------------------------------------------------------------
# Parser
# ---------------------------------------------------------------------------

class WitParser:
    """Recursive descent parser for WIT files."""

    def __init__(self, tokens: List[Token], source_file: str = ""):
        self.tokens = tokens
        self.pos = 0
        self.source_file = source_file
        self.current_docs: List[str] = []

    def current(self) -> Optional[Token]:
        if self.pos < len(self.tokens):
            return self.tokens[self.pos]
        return None

    def peek(self, offset: int = 0) -> Optional[Token]:
        pos = self.pos + offset
        if pos < len(self.tokens):
            return self.tokens[pos]
        return None

    def consume(self, expected_kind: Optional[str] = None) -> Token:
        tok = self.current()
        if tok is None:
            raise SyntaxError(f"Unexpected end of input, expected {expected_kind}")
        if expected_kind and tok.kind != expected_kind:
            raise SyntaxError(f"Expected {expected_kind}, got {tok.kind} '{tok.value}' at line {tok.line}")
        self.pos += 1
        return tok

    def match(self, *kinds: str) -> bool:
        tok = self.current()
        return tok is not None and tok.kind in kinds

    def collect_docs(self) -> List[str]:
        """Collect documentation comments preceding current position."""
        # Look backwards for doc comments (simplified - we extract from source)
        docs = self.current_docs.copy()
        self.current_docs = []
        return docs

    def parse_type(self) -> WitType:
        """Parse a WIT type expression."""
        tok = self.current()
        if tok is None:
            raise SyntaxError("Expected type")

        # Primitive types
        if tok.kind == 'PRIMITIVE':
            self.consume()
            return WitPrimitive(tok.value)

        # option<T>
        if tok.kind == 'OPTION':
            self.consume()
            self.consume('LANGLE')
            inner = self.parse_type()
            self.consume('RANGLE')
            return WitOption(inner)

        # list<T>
        if tok.kind == 'LIST':
            self.consume()
            self.consume('LANGLE')
            inner = self.parse_type()
            self.consume('RANGLE')
            return WitList(inner)

        # result<T, E> or result<T> or result<_, E>
        if tok.kind == 'RESULT':
            self.consume()
            self.consume('LANGLE')
            ok_type: Optional[WitType] = None
            err_type: Optional[WitType] = None

            # Check for underscore (no ok type)
            if self.match('PRIMITIVE') and self.current().value == '_':
                self.consume()
            else:
                ok_type = self.parse_type()

            if self.match('COMMA'):
                self.consume()
                err_type = self.parse_type()

            self.consume('RANGLE')
            return WitResult(ok_type, err_type)

        # tuple<T1, T2, ...>
        if tok.kind == 'TUPLE':
            self.consume()
            self.consume('LANGLE')
            elements = [self.parse_type()]
            while self.match('COMMA'):
                self.consume()
                elements.append(self.parse_type())
            self.consume('RANGLE')
            return WitTuple(elements)

        # Identifier (handle type, record, enum, variant, flags)
        if tok.kind == 'IDENT':
            self.consume()
            name = tok.value
            # Check for handle types (usually end with -handle)
            if name.endswith('-handle') or name.endswith('Handle'):
                return WitHandle(name)
            # Could be a record/enum/variant/flags reference
            return WitRecord(name)  # Treat as record for now

        raise SyntaxError(f"Unexpected token in type: {tok.kind} '{tok.value}'")

    def parse_params(self) -> List[WitParam]:
        """Parse function parameters: (name: type, name2: type2)"""
        params: List[WitParam] = []
        self.consume('LPAREN')

        if not self.match('RPAREN'):
            while True:
                # Accept both IDENT and ESCAPED_IDENT for param names
                if self.match('ESCAPED_IDENT'):
                    name_tok = self.consume()
                    name = name_tok.value[1:]  # Remove leading %
                else:
                    name_tok = self.consume('IDENT')
                    name = name_tok.value
                self.consume('COLON')
                type_ = self.parse_type()
                params.append(WitParam(name, type_))
                if not self.match('COMMA'):
                    break
                self.consume('COMMA')
                # Handle trailing comma - stop if next is RPAREN
                if self.match('RPAREN'):
                    break

        self.consume('RPAREN')
        return params

    def parse_function(self) -> WitFunction:
        """Parse a function definition: name: func(params) -> result;"""
        docs = self.collect_docs()
        # Accept both IDENT and ESCAPED_IDENT for function names
        if self.match('ESCAPED_IDENT'):
            name_tok = self.consume()
            name = name_tok.value[1:]  # Remove leading %
        else:
            name_tok = self.consume('IDENT')
            name = name_tok.value
        self.consume('COLON')
        self.consume('FUNC')

        params = self.parse_params()

        result: Optional[WitType] = None
        if self.match('ARROW'):
            self.consume()
            result = self.parse_type()

        self.consume('SEMICOLON')

        return WitFunction(name, params, result, docs=docs)

    def parse_type_alias(self) -> WitTypeAlias:
        """Parse a type alias: type name = target;"""
        docs = self.collect_docs()
        self.consume('TYPE')
        name_tok = self.consume('IDENT')
        name = name_tok.value
        self.consume('EQUALS')
        target = self.parse_type()
        self.consume('SEMICOLON')
        return WitTypeAlias(name, target, docs)

    def parse_interface(self) -> WitInterface:
        """Parse an interface definition."""
        docs = self.collect_docs()
        self.consume('INTERFACE')
        name_tok = self.consume('IDENT')
        name = name_tok.value
        self.consume('LBRACE')

        functions: List[WitFunction] = []
        type_aliases: List[WitTypeAlias] = []

        while not self.match('RBRACE'):
            if self.match('TYPE'):
                type_aliases.append(self.parse_type_alias())
            elif self.match('IDENT') or self.match('ESCAPED_IDENT'):
                functions.append(self.parse_function())
            elif self.match('INCLUDE') or self.match('USE') or self.match('IMPORT') or self.match('FROM'):
                # Skip include/use/import statements in interfaces
                self._skip_include()
            elif self.match('ENUM'):
                # Skip enum definitions inside interfaces
                self._skip_enum()
            elif self.match('RECORD'):
                # Skip record definitions inside interfaces
                self._skip_record()
            elif self.match('VARIANT'):
                # Skip variant definitions inside interfaces
                self._skip_variant()
            elif self.match('FLAGS'):
                # Skip flags definitions inside interfaces
                self._skip_flags()
            elif self.match('RESOURCE'):
                # Skip resource definitions inside interfaces
                self._skip_resource()
            else:
                tok = self.current()
                raise SyntaxError(f"Unexpected token in interface: {tok.kind if tok else 'EOF'}")

        self.consume('RBRACE')

        return WitInterface(name, functions, type_aliases, docs, self.source_file)

    def parse_world(self) -> WitWorld:
        """Parse a world definition."""
        docs = self.collect_docs()
        self.consume('WORLD')
        name_tok = self.consume('IDENT')
        name = name_tok.value
        self.consume('LBRACE')

        imports: List[str] = []
        exports: List[str] = []

        while not self.match('RBRACE'):
            if self.match('IMPORT'):
                self.consume()
                iface_name = self.consume('IDENT').value
                # Skip optional from clause
                if self.match('FROM'):
                    self.consume()
                    self.consume('STRING')
                if self.match('SEMICOLON'):
                    self.consume()
                imports.append(iface_name)
            elif self.match('EXPORT'):
                self.consume()
                iface_name = self.consume('IDENT').value
                if self.match('SEMICOLON'):
                    self.consume()
                exports.append(iface_name)
            else:
                tok = self.current()
                raise SyntaxError(f"Unexpected token in world: {tok.kind if tok else 'EOF'}")

        self.consume('RBRACE')
        return WitWorld(name, imports, exports, docs)

    def parse_package(self) -> WitPackage:
        """Parse a complete WIT package."""
        docs = self.collect_docs()

        self.consume('PACKAGE')
        name_tok = self.consume('IDENT')
        # Handle package names like "tairitsu-browser:storage"
        pkg_name = name_tok.value
        if self.match('COLON'):
            self.consume()
            next_tok = self.consume('IDENT')
            pkg_name += ':' + next_tok.value

        version = ""
        if self.match('AT'):
            self.consume()
            # Version can be IDENT or VERSION (semver like 0.2.0)
            if self.match('VERSION'):
                version = self.consume().value
            elif self.match('IDENT'):
                version = self.consume().value
            else:
                # Handle numeric version parts
                version_parts = []
                while self.match('NUMBER') or self.match('DOT'):
                    tok = self.consume()
                    version_parts.append(tok.value)
                version = ''.join(version_parts)

        self.consume('SEMICOLON')

        interfaces: List[WitInterface] = []
        worlds: List[WitWorld] = []

        while self.current() is not None:
            if self.match('INTERFACE'):
                interfaces.append(self.parse_interface())
            elif self.match('WORLD'):
                worlds.append(self.parse_world())
            elif self.match('ENUM'):
                # Skip top-level enum definitions
                self._skip_enum()
            elif self.match('RECORD'):
                # Skip top-level record definitions
                self._skip_record()
            elif self.match('VARIANT'):
                # Skip top-level variant definitions
                self._skip_variant()
            elif self.match('FLAGS'):
                # Skip top-level flags definitions
                self._skip_flags()
            elif self.match('TYPE'):
                # Skip top-level type aliases
                self._skip_type_alias()
            else:
                tok = self.current()
                raise SyntaxError(f"Unexpected token at package level: {tok.kind if tok else 'EOF'}")

        return WitPackage(pkg_name, version, interfaces, worlds, docs, self.source_file)

    def _skip_enum(self) -> None:
        """Skip an enum definition."""
        self.consume('ENUM')
        self.consume('IDENT')  # enum name
        self.consume('LBRACE')
        depth = 1
        while depth > 0:
            tok = self.consume()
            if tok is None:
                break
            if tok.kind == 'LBRACE':
                depth += 1
            elif tok.kind == 'RBRACE':
                depth -= 1

    def _skip_record(self) -> None:
        """Skip a record definition."""
        self.consume('RECORD')
        self.consume('IDENT')  # record name
        self.consume('LBRACE')
        depth = 1
        while depth > 0:
            tok = self.consume()
            if tok is None:
                break
            if tok.kind == 'LBRACE':
                depth += 1
            elif tok.kind == 'RBRACE':
                depth -= 1

    def _skip_variant(self) -> None:
        """Skip a variant definition."""
        self.consume('VARIANT')
        self.consume('IDENT')  # variant name
        self.consume('LBRACE')
        depth = 1
        while depth > 0:
            tok = self.consume()
            if tok is None:
                break
            if tok.kind == 'LBRACE':
                depth += 1
            elif tok.kind == 'RBRACE':
                depth -= 1

    def _skip_flags(self) -> None:
        """Skip a flags definition."""
        self.consume('FLAGS')
        self.consume('IDENT')  # flags name
        self.consume('LBRACE')
        depth = 1
        while depth > 0:
            tok = self.consume()
            if tok is None:
                break
            if tok.kind == 'LBRACE':
                depth += 1
            elif tok.kind == 'RBRACE':
                depth -= 1

    def _skip_resource(self) -> None:
        """Skip a resource definition."""
        self.consume('RESOURCE')
        self.consume('IDENT')  # resource name
        # Resource can have optional body with { ... }
        if self.match('LBRACE'):
            self.consume()
            depth = 1
            while depth > 0:
                tok = self.consume()
                if tok is None:
                    break
                if tok.kind == 'LBRACE':
                    depth += 1
                elif tok.kind == 'RBRACE':
                    depth -= 1
        elif self.match('SEMICOLON'):
            self.consume()

    def _skip_type_alias(self) -> None:
        """Skip a top-level type alias."""
        self.consume('TYPE')
        self.consume('IDENT')  # type name
        self.consume('EQUALS')
        # Skip until semicolon
        while self.current() and self.current().kind != 'SEMICOLON':
            self.consume()
        self.consume('SEMICOLON')

    def _skip_include(self) -> None:
        """Skip include/use/import statements within interfaces."""
        tok = self.current()
        if tok is None:
            return
        # include/use/import [name] [from "path"];
        while self.current() and self.current().kind != 'SEMICOLON':
            self.consume()
        if self.current() and self.current().kind == 'SEMICOLON':
            self.consume()

    def parse(self) -> WitPackage:
        """Parse the complete WIT file."""
        return self.parse_package()


def extract_doc_comments(text: str) -> Dict[int, List[str]]:
    """Extract doc comments (///) from source and map to line numbers."""
    docs: Dict[int, List[str]] = {}
    lines = text.split('\n')
    pending: List[str] = []

    for i, line in enumerate(lines):
        stripped = line.strip()
        if stripped.startswith('///'):
            # Extract comment content (remove /// and leading space)
            content = stripped[3:].strip()
            pending.append(content)
        elif stripped and not stripped.startswith('//'):
            # Non-comment, non-empty line - attach pending docs
            if pending:
                docs[i + 1] = pending.copy()  # 1-indexed line number
                pending = []
        # else: empty line or block comment, reset pending
        elif not stripped:
            pending = []

    return docs


def parse_wit_file(path: Path) -> WitPackage:
    """Parse a WIT file and return the package AST."""
    text = path.read_text(encoding='utf-8')
    return parse_wit_text(text, str(path))


def parse_wit_text(text: str, source_file: str = "") -> WitPackage:
    """Parse WIT source text and return the package AST."""
    # Extract doc comments first
    doc_map = extract_doc_comments(text)

    # Tokenize
    tokens = tokenize(text)

    # Parse
    parser = WitParser(tokens, source_file)

    # Map docs to constructs (simplified approach - we'll enhance this)
    pkg = parser.parse()

    return pkg


# ---------------------------------------------------------------------------
# Utilities
# ---------------------------------------------------------------------------

def wit_type_to_string(t: WitType) -> str:
    """Convert a WitType back to WIT syntax string."""
    if isinstance(t, WitPrimitive):
        return t.name
    elif isinstance(t, WitHandle):
        return t.name
    elif isinstance(t, WitOption):
        return f"option<{wit_type_to_string(t.inner)}>"
    elif isinstance(t, WitList):
        return f"list<{wit_type_to_string(t.inner)}>"
    elif isinstance(t, WitResult):
        ok_str = "_" if t.ok is None else wit_type_to_string(t.ok)
        if t.err is None:
            return f"result<{ok_str}>"
        return f"result<{ok_str}, {wit_type_to_string(t.err)}>"
    elif isinstance(t, WitTuple):
        elems = ", ".join(wit_type_to_string(e) for e in t.elements)
        return f"tuple<{elems}>"
    elif isinstance(t, WitRecord):
        return t.name
    elif isinstance(t, WitEnum):
        return t.name
    elif isinstance(t, WitVariant):
        return t.name
    elif isinstance(t, WitFlags):
        return t.name
    else:
        return str(t)


def kebab_to_camel(name: str) -> str:
    """Convert kebab-case to camelCase."""
    parts = name.split('-')
    if not parts:
        return name
    return parts[0] + ''.join(p.capitalize() for p in parts[1:])


def kebab_to_pascal(name: str) -> str:
    """Convert kebab-case to PascalCase."""
    parts = name.split('-')
    return ''.join(p.capitalize() for p in parts)


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    import argparse
    import sys

    def main():
        parser = argparse.ArgumentParser(description="Parse WIT files and dump AST")
        parser.add_argument("file", help="WIT file to parse")
        parser.add_argument("--json", action="store_true", help="Output as JSON")
        args = parser.parse_args()

        path = Path(args.file)
        if not path.exists():
            print(f"Error: File not found: {path}", file=sys.stderr)
            sys.exit(1)

        pkg = parse_wit_file(path)

        if args.json:
            import json
            from dataclasses import asdict
            print(json.dumps(asdict(pkg), indent=2))
        else:
            print(f"Package: {pkg.name}@{pkg.version}")
            print(f"Source: {pkg.source_file}")
            print()
            for iface in pkg.interfaces:
                print(f"Interface: {iface.name}")
                for ta in iface.type_aliases:
                    print(f"  type {ta.name} = {wit_type_to_string(ta.target)};")
                for func in iface.functions:
                    params = ", ".join(f"{p.name}: {wit_type_to_string(p.type_)}" for p in func.params)
                    result = f" -> {wit_type_to_string(func.result)}" if func.result else ""
                    print(f"  {func.name}({params}){result};")
                print()
            for world in pkg.worlds:
                print(f"World: {world.name}")
                print(f"  imports: {', '.join(world.imports)}")
                print(f"  exports: {', '.join(world.exports)}")

    main()
