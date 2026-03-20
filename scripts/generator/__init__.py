#!/usr/bin/env python3
"""Browser glue generator package.

This package provides modular components for generating TypeScript glue code
from WIT interface definitions.

The generator is organized by functionality:
- config: Type mappings and configuration constants
- models: Data structures for generated code
- ast_parser: WIT AST parsing and transformation
- code_gen: TypeScript code rendering
- logger: Logging utilities
"""

from .logger import log_info, log_ok, log_warn, log_error
from .config import (
    INTERFACE_TO_BROWSER_CLASS,
    MISSING_TYPES_IN_DOM,
    GLOBAL_SINGLETONS,
    ASYNC_PATTERNS,
    BROWSER_API_NAME_MAPPINGS,
    PARAMETER_HANDLE_MAPPING,
    DICTIONARY_PARAMETER_TYPES,
    PARAMETER_BIGINT_TO_NUMBER,
    ENUM_PROPERTIES,
    ENUM_VALUE_MAPPINGS,
    GETTER_BUT_ACTUALLY_METHOD,
    CUSTOM_TYPE_DEFINITIONS,
    TYPE_NAME_CASING_OVERRIDES,
    STATIC_METHOD_RETURN_OVERRIDES,
    STATIC_METHOD_NEEDS_TYPE_ASSERTION,
    HANDLE_RETURNING_FUNCTIONS,
    JS_RESERVED_WORDS,
    correct_type_casing,
)
from .models import (
    GeneratedParam,
    GeneratedFunction,
    GeneratedTypeAlias,
    GeneratedInterface,
    GeneratedDomain,
)
from .ast_parser import WitParser
from .code_gen import CodeGenerator

__all__ = [
    # Logging
    "log_info",
    "log_ok",
    "log_warn",
    "log_error",
    # Config
    "INTERFACE_TO_BROWSER_CLASS",
    "MISSING_TYPES_IN_DOM",
    "GLOBAL_SINGLETONS",
    "ASYNC_PATTERNS",
    "BROWSER_API_NAME_MAPPINGS",
    "PARAMETER_HANDLE_MAPPING",
    "DICTIONARY_PARAMETER_TYPES",
    "PARAMETER_BIGINT_TO_NUMBER",
    "ENUM_PROPERTIES",
    "ENUM_VALUE_MAPPINGS",
    "GETTER_BUT_ACTUALLY_METHOD",
    "CUSTOM_TYPE_DEFINITIONS",
    "TYPE_NAME_CASING_OVERRIDES",
    "STATIC_METHOD_RETURN_OVERRIDES",
    "STATIC_METHOD_NEEDS_TYPE_ASSERTION",
    "HANDLE_RETURNING_FUNCTIONS",
    "JS_RESERVED_WORDS",
    "correct_type_casing",
    # Models
    "GeneratedParam",
    "GeneratedFunction",
    "GeneratedTypeAlias",
    "GeneratedInterface",
    "GeneratedDomain",
    # Parsers/Generators
    "WitParser",
    "CodeGenerator",
]
