#!/usr/bin/env python3
"""
TypeScript Type Mapper for WIT Types

Maps WIT types to their TypeScript equivalents for glue code generation.

WIT Type          → TypeScript Type
--------------------------------------------
u8, u16, u32      → number
s8, s16, s32      → number
u64, s64          → bigint
f32, f64          → number
bool              → boolean
string            → string
char              → string
option<T>         → T | undefined
list<T>           → T[]
result<T, E>      → { ok: true; value: T } | { ok: false; error: E }
result<T>         → { ok: true; value: T } | { ok: false; error: string }
result<_, E>      → { ok: true } | { ok: false; error: E }
handle types      → bigint
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Dict, Optional, Tuple, List

# Import from wit_parser
from wit_parser import (
    WitType, WitPrimitive, WitHandle, WitOption, WitList,
    WitResult, WitTuple, WitRecord, WitEnum, WitVariant, WitFlags
)


# ---------------------------------------------------------------------------
# Type Mapping Tables
# ---------------------------------------------------------------------------

WIT_TO_TYPESCRIPT_PRIMITIVE: Dict[str, str] = {
    # Unsigned integers
    "u8": "number",
    "u16": "number",
    "u32": "number",
    "u64": "bigint",

    # Signed integers
    "s8": "number",
    "s16": "number",
    "s32": "number",
    "s64": "bigint",

    # Floating point
    "f32": "number",
    "f64": "number",

    # Boolean
    "bool": "boolean",

    # String types
    "string": "string",
    "char": "string",

    # Unit type
    "_": "void",
}

# JavaScript runtime type checks for validation
TYPESCRIPT_RUNTIME_CHECK: Dict[str, str] = {
    "number": "typeof value === 'number'",
    "bigint": "typeof value === 'bigint'",
    "boolean": "typeof value === 'boolean'",
    "string": "typeof value === 'string'",
    "void": "true",  # No check needed
}


# ---------------------------------------------------------------------------
# Result Type Generation
# ---------------------------------------------------------------------------

@dataclass
class ResultTypeConfig:
    """Configuration for result type generation."""
    ok_type: Optional[str]  # TypeScript type string for ok case
    err_type: Optional[str]  # TypeScript type string for error case

    @property
    def has_ok_value(self) -> bool:
        return self.ok_type is not None and self.ok_type != "void"

    @property
    def has_err_type(self) -> bool:
        return self.err_type is not None


def generate_result_type(ok_type: Optional[str], err_type: Optional[str]) -> str:
    """
    Generate TypeScript discriminated union for result type.

    result<T, E>  → { ok: true; value: T } | { ok: false; error: E }
    result<T>     → { ok: true; value: T } | { ok: false; error: string }
    result<_, E>  → { ok: true } | { ok: false; error: E }
    """
    if ok_type is None or ok_type == "void":
        ok_clause = "{ ok: true }"
    else:
        ok_clause = f"{{ ok: true; value: {ok_type} }}"

    if err_type is None:
        err_clause = "{ ok: false; error: string }"
    else:
        err_clause = f"{{ ok: false; error: {err_type} }}"

    return f"{ok_clause} | {err_clause}"


# ---------------------------------------------------------------------------
# Main Type Mapper Class
# ---------------------------------------------------------------------------

class TypeScriptTypeMapper:
    """Maps WIT types to TypeScript types."""

    def __init__(self):
        # Cache for computed types
        self._cache: Dict[int, str] = {}

    def map_type(self, wit_type: WitType) -> str:
        """
        Convert a WIT type to TypeScript type string.

        Args:
            wit_type: The WIT type AST node

        Returns:
            TypeScript type string
        """
        # Use cache for complex types
        type_id = id(wit_type)
        if type_id in self._cache:
            return self._cache[type_id]

        result = self._map_type_impl(wit_type)
        self._cache[type_id] = result
        return result

    def _map_type_impl(self, wit_type: WitType) -> str:
        """Internal implementation of type mapping."""

        # Primitive types
        if isinstance(wit_type, WitPrimitive):
            return WIT_TO_TYPESCRIPT_PRIMITIVE.get(wit_type.name, "unknown")

        # Handle types (opaque u64)
        if isinstance(wit_type, WitHandle):
            return "bigint"

        # Option<T> → T | undefined
        if isinstance(wit_type, WitOption):
            inner = self.map_type(wit_type.inner)
            return f"{inner} | undefined"

        # list<T> → T[]
        if isinstance(wit_type, WitList):
            inner = self.map_type(wit_type.inner)
            # Special case: list<u8> → Uint8Array for binary data
            if inner == "number" and isinstance(wit_type.inner, WitPrimitive):
                if wit_type.inner.name == "u8":
                    return "Uint8Array"
            return f"({inner})[]"

        # result<T, E> → discriminated union
        if isinstance(wit_type, WitResult):
            ok_type = None if wit_type.ok is None else self.map_type(wit_type.ok)
            err_type = None if wit_type.err is None else self.map_type(wit_type.err)
            return generate_result_type(ok_type, err_type)

        # tuple<T1, T2, ...> → [T1, T2, ...]
        if isinstance(wit_type, WitTuple):
            elements = [self.map_type(e) for e in wit_type.elements]
            return f"[{', '.join(elements)}]"

        # Record/Enum/Variant/Flags → use name as-is (assume defined elsewhere)
        if isinstance(wit_type, WitRecord):
            # Convert kebab-case to PascalCase for TypeScript
            return self._to_pascal_case(wit_type.name)

        if isinstance(wit_type, WitEnum):
            return self._to_pascal_case(wit_type.name)

        if isinstance(wit_type, WitVariant):
            return self._to_pascal_case(wit_type.name)

        if isinstance(wit_type, WitFlags):
            return self._to_pascal_case(wit_type.name)

        # Unknown type
        return "unknown"

    def map_param(self, wit_type: WitType) -> str:
        """
        Map WIT type for function parameters.

        For result types in parameters, we use a simpler representation
        (just bigint handle for async operations).
        """
        return self.map_type(wit_type)

    def map_return(self, wit_type: WitType, is_async: bool = False) -> str:
        """
        Map WIT type for function return values.

        For async operations, the return type is a request ID (bigint).
        """
        if is_async:
            return "bigint"
        return self.map_type(wit_type)

    @staticmethod
    def _to_pascal_case(name: str) -> str:
        """Convert kebab-case to PascalCase."""
        parts = name.replace('_', '-').split('-')
        return ''.join(p.capitalize() for p in parts)

    @staticmethod
    def _to_camel_case(name: str) -> str:
        """Convert kebab-case to camelCase."""
        parts = name.replace('_', '-').split('-')
        if not parts:
            return name
        return parts[0] + ''.join(p.capitalize() for p in parts[1:])


# ---------------------------------------------------------------------------
# JavaScript Value Marshaling
# ---------------------------------------------------------------------------

class JavaScriptMarshaler:
    """Generates JavaScript code for marshaling values between WIT and JS."""

    @staticmethod
    def to_js(wit_type: WitType, var_name: str, mapper: TypeScriptTypeMapper) -> str:
        """
        Generate code to convert WIT value to JavaScript.

        Args:
            wit_type: The WIT type
            var_name: The variable name holding the WIT value
            mapper: Type mapper instance

        Returns:
            JavaScript expression
        """
        if isinstance(wit_type, WitPrimitive):
            if wit_type.name in ("u64", "s64"):
                return f"Number({var_name})"  # Convert bigint to number if needed
            return var_name

        if isinstance(wit_type, WitHandle):
            return var_name  # Handles stay as bigint

        if isinstance(wit_type, WitOption):
            inner_expr = JavaScriptMarshaler.to_js(wit_type.inner, var_name, mapper)
            return f"({var_name} === null || {var_name} === undefined ? undefined : {inner_expr})"

        if isinstance(wit_type, WitList):
            if isinstance(wit_type.inner, WitPrimitive) and wit_type.inner.name == "u8":
                return f"new Uint8Array({var_name})"
            return var_name

        if isinstance(wit_type, WitResult):
            # Results are represented as objects
            return var_name

        return var_name

    @staticmethod
    def from_js(wit_type: WitType, var_name: str, mapper: TypeScriptTypeMapper) -> str:
        """
        Generate code to convert JavaScript value to WIT-compatible format.

        Args:
            wit_type: The WIT type
            var_name: The variable name holding the JS value
            mapper: Type mapper instance

        Returns:
            JavaScript expression
        """
        if isinstance(wit_type, WitPrimitive):
            if wit_type.name in ("u64", "s64"):
                return f"BigInt({var_name})"
            return var_name

        if isinstance(wit_type, WitHandle):
            return var_name  # Handles are already bigint

        if isinstance(wit_type, WitOption):
            # Handle null/undefined
            inner_expr = JavaScriptMarshaler.from_js(wit_type.inner, var_name, mapper)
            return f"({var_name} === undefined ? null : {inner_expr})"

        if isinstance(wit_type, WitList):
            if isinstance(wit_type.inner, WitPrimitive) and wit_type.inner.name == "u8":
                return f"Array.from({var_name})"
            return var_name

        return var_name


# ---------------------------------------------------------------------------
# Utility Functions
# ---------------------------------------------------------------------------

def is_async_function(func_name: str, func_params: List, func_result) -> bool:
    """
    Determine if a function should use async poll pattern.

    Functions returning Promise<T> or certain browser APIs that are
    inherently async should use the poll pattern.
    """
    # Common async patterns in browser APIs
    async_patterns = [
        "fetch", "estimate", "persist", "persisted",
        "array-buffer", "blob", "text", "json", "bytes", "form-data",
        "read", "write", "cancel", "close", "abort",
        "get-reader", "get-writer", "pipe-to", "pipe-through",
        "response", "request",
    ]

    func_lower = func_name.lower()
    for pattern in async_patterns:
        if pattern in func_lower:
            return True

    return False


def get_browser_api_call(iface_name: str, func_name: str) -> str:
    """
    Map WIT interface/function to browser API call.

    Returns the JavaScript code to call the corresponding browser API.
    """
    # This is a simplified mapping - in practice this would need
    # more sophisticated handling based on the interface type
    return f"/* browser API call for {iface_name}.{func_name} */"


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    import sys
    from wit_parser import parse_wit_text

    def main():
        if len(sys.argv) < 2:
            print("Usage: python type_mapper.py '<wit-type>'")
            print("Example: python type_mapper.py 'option<list<u8>>'")
            sys.exit(1)

        # Parse the type from command line (simplified)
        type_str = sys.argv[1]

        # Create a minimal WIT snippet to parse
        wit_snippet = f"""
        package test:types@0.0.1;
        interface test {{
            test-func: func() -> {type_str};
        }}
        """

        try:
            pkg = parse_wit_text(wit_snippet)
            if pkg.interfaces and pkg.interfaces[0].functions:
                wit_type = pkg.interfaces[0].functions[0].result
                if wit_type:
                    mapper = TypeScriptTypeMapper()
                    ts_type = mapper.map_type(wit_type)
                    print(f"WIT:   {type_str}")
                    print(f"TS:    {ts_type}")
                else:
                    print("No return type found")
            else:
                print("Failed to parse type")
        except Exception as e:
            print(f"Error: {e}")
            sys.exit(1)

    main()
