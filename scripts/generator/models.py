#!/usr/bin/env python3
"""Data structures for browser glue code generation."""

from dataclasses import dataclass, field
from typing import List, Optional


@dataclass
class GeneratedParam:
    """Generated function parameter."""
    name: str
    ts_type: str
    wit_type_str: str
    needs_handle_lookup: bool = False
    target_handle_pascal: str = ""
    wit_name: str = ""


@dataclass
class GeneratedFunction:
    """Generated function info."""
    wit_name: str
    ts_name: str
    pascal_name: str
    params: List[GeneratedParam]
    ts_return: str
    ts_return_inner: str = ""
    ts_return_inner_original: str = ""
    is_async: bool = False
    is_getter: bool = False
    is_setter: bool = False
    is_getter_but_method: bool = False
    is_setter_but_method: bool = False
    is_static: bool = False
    return_is_void: bool = False
    return_is_optional: bool = False
    return_is_handle: bool = False
    browser_method: str = ""
    browser_attr: str = ""
    browser_args: str = ""
    browser_class: str = ""
    self_param: str = "self"
    value_param: str = "value"
    has_explicit_poll: bool = False
    is_global_singleton: bool = False
    skip_first_param: bool = False
    docs: List[str] = field(default_factory=list)
    
    exported_name: str = ""


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
    js_type_for_handles: str = ""
    functions: List[GeneratedFunction] = field(default_factory=list)
    type_aliases: List[GeneratedTypeAlias] = field(default_factory=list)
    create_function: bool = False


@dataclass
class GeneratedDomain:
    """Generated domain info."""
    name: str
    export_name: str
    interfaces: List[GeneratedInterface]
    interface_count: int
