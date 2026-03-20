#!/usr/bin/env python3
"""WIT AST parsing and transformation logic for browser glue generator."""

import sys
from pathlib import Path
from typing import List, Optional

from wit_parser import (
    parse_wit_file, WitPackage, WitInterface, WitFunction,
    WitParam, WitType, WitHandle, WitOption, WitTypeAlias,
    kebab_to_camel, kebab_to_pascal, wit_type_to_string
)
from type_mapper import TypeScriptTypeMapper

from .config import (
    INTERFACE_TO_BROWSER_CLASS,
    MISSING_TYPES_IN_DOM,
    GLOBAL_SINGLETONS,
    BROWSER_API_NAME_MAPPINGS,
    PARAMETER_HANDLE_MAPPING,
    DICTIONARY_PARAMETER_TYPES,
    STATIC_METHOD_RETURN_OVERRIDES,
    STATIC_METHOD_NEEDS_TYPE_ASSERTION,
    HANDLE_RETURNING_FUNCTIONS,
    JS_RESERVED_WORDS,
    ASYNC_PATTERNS,
    GETTER_BUT_ACTUALLY_METHOD,
    ENUM_PROPERTIES,
)
from .models import (
    GeneratedParam, GeneratedFunction, GeneratedTypeAlias,
    GeneratedInterface, GeneratedDomain,
)
from .config import correct_type_casing


class WitParser:
    """Parses WIT files and transforms them into generated code data structures."""

    def __init__(self):
        self.type_mapper = TypeScriptTypeMapper()

    def generate_function(
        self,
        func: WitFunction,
        interface: WitInterface,
        browser_class: str,
        handle_type: Optional[str]
    ) -> GeneratedFunction:
        """Generate code for a single WIT function."""

        wit_name = func.name
        ts_name = kebab_to_camel(wit_name)
        browser_method = ts_name
        
        if wit_name in BROWSER_API_NAME_MAPPINGS:
            browser_method = BROWSER_API_NAME_MAPPINGS[wit_name]
        
        if ts_name in JS_RESERVED_WORDS:
            ts_name = f"_{ts_name}"
        
        pascal_name = kebab_to_pascal(wit_name)
        is_global_singleton = interface.name in GLOBAL_SINGLETONS

        params: List[GeneratedParam] = []
        self_param = "self"
        browser_args_list: List[str] = []
        skip_first_param = False

        for i, p in enumerate(func.params):
            param_name = kebab_to_camel(p.name)
            
            if param_name in JS_RESERVED_WORDS:
                param_name = f"_{param_name}"
            
            key = (interface.name, wit_name, p.name)
            needs_lookup = False
            target_pascal = ""
            
            if isinstance(p.type_, WitHandle):
                ts_type = "bigint"
            elif key in DICTIONARY_PARAMETER_TYPES:
                ts_type = DICTIONARY_PARAMETER_TYPES[key]
            else:
                ts_type = self.type_mapper.map_type(p.type_)
                wit_type_str_check = wit_type_to_string(p.type_)
                if wit_type_str_check.endswith("-handle"):
                    needs_lookup = True
                    handle_iface = wit_type_str_check[:-7]
                    target_pascal = correct_type_casing(kebab_to_pascal(handle_iface))
            
            wit_type_str = wit_type_to_string(p.type_)
            
            if is_global_singleton and i == 0 and p.name == "self":
                skip_first_param = True
                continue
            
            params.append(GeneratedParam(param_name, ts_type, wit_type_str, needs_lookup, target_pascal))

            if i == 0 and p.name == "self":
                self_param = param_name
            elif not (i == 0 and is_global_singleton):
                browser_args_list.append(param_name)

        ts_return = "void"
        ts_return_inner = ""
        ts_return_inner_original = ""
        return_is_void = True
        return_is_optional = False
        return_is_handle = False

        if func.result:
            ts_return = self.type_mapper.map_type(func.result)
            return_is_void = ts_return == "void"
            return_is_optional = isinstance(func.result, WitOption)
            if not return_is_void:
                ts_return_inner = ts_return
                return_is_handle = (ts_return == "bigint" and 
                                   (isinstance(func.result, WitHandle) or
                                    any(ta.name == wit_type_to_string(func.result) and ta.name.endswith("-handle")
                                        for ta in interface.type_aliases)))
                
                key = (interface.name, kebab_to_camel(wit_name))
                if key in HANDLE_RETURNING_FUNCTIONS:
                    return_is_handle = True

        is_getter = wit_name.startswith("get-")
        is_setter = wit_name.startswith("set-")
        is_async = self._is_async_function(wit_name, func, interface.name)
        is_getter_but_method = is_getter and wit_name[4:] in GETTER_BUT_ACTUALLY_METHOD
        
        has_self_param = any(p.name == "self" for p in func.params)
        is_static = func.is_static or (not has_self_param and not is_global_singleton)
  
        if is_async:
            ts_return_inner_original = ts_return_inner
            ts_return = "bigint"
            return_is_void = False
 
        browser_attr = ""
        browser_args = ", ".join(browser_args_list)
        value_param = "value"
        
        if is_global_singleton:
            browser_class = GLOBAL_SINGLETONS[interface.name]
        
        key = (interface.name, ts_name)
        if key in STATIC_METHOD_RETURN_OVERRIDES:
            ts_return = STATIC_METHOD_RETURN_OVERRIDES[key]
            if ts_return.startswith("Promise<"):
                is_async = True
                ts_return_inner = ts_return[8:-1]
                ts_return_inner_original = ts_return
                ts_return = "bigint"
                return_is_void = False

        if is_getter:
            attr_name = wit_name[4:]
            browser_attr = kebab_to_camel(attr_name)
        elif is_setter:
            attr_name = wit_name[4:]
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
            ts_return_inner_original=ts_return_inner_original,
            is_async=is_async,
            is_getter=is_getter,
            is_setter=is_setter,
            is_static=is_static,
            is_getter_but_method=is_getter_but_method,
            return_is_void=return_is_void,
            return_is_optional=return_is_optional,
            return_is_handle=return_is_handle,
            browser_method=browser_method,
            browser_attr=browser_attr,
            browser_args=browser_args,
            browser_class=browser_class if is_static or is_global_singleton else "",
            self_param=self_param,
            value_param=value_param,
            has_explicit_poll=False,
            is_global_singleton=is_global_singleton,
            skip_first_param=skip_first_param,
            docs=func.docs,
        )

    def generate_interface(self, iface: WitInterface) -> GeneratedInterface:
        """Generate code for a WIT interface."""

        wit_name = iface.name
        handle_type = None
        handle_var = ""
        handle_pascal = ""

        for ta in iface.type_aliases:
            if ta.name.endswith("-handle"):
                handle_type = ta.name
                handle_var = kebab_to_camel(ta.name.replace("-handle", "Handles"))
                handle_pascal = correct_type_casing(kebab_to_pascal(ta.name.replace("-handle", "")))
                break

        browser_class = INTERFACE_TO_BROWSER_CLASS.get(wit_name, correct_type_casing(kebab_to_pascal(wit_name)))
        js_type_for_handles = MISSING_TYPES_IN_DOM.get(wit_name, browser_class)

        type_aliases: List[GeneratedTypeAlias] = []
        for ta in iface.type_aliases:
            ts_name = correct_type_casing(kebab_to_pascal(ta.name))
            if ta.name.endswith("-handle") or isinstance(ta.target, WitHandle):
                ts_type = "bigint"
            else:
                ts_type = self.type_mapper.map_type(ta.target)
            type_aliases.append(GeneratedTypeAlias(
                name=ta.name,
                ts_name=ts_name,
                ts_type=ts_type,
                docs=ta.docs,
            ))

        functions: List[GeneratedFunction] = []

        func_names = {func.name for func in iface.functions}
        poll_names = {name for name in func_names if name.startswith("poll-")}
        has_explicit_poll = {}
        for name in func_names:
            if not name.startswith("poll-"):
                poll_name = f"poll-{name}"
                has_explicit_poll[name] = poll_name in poll_names

        for func in iface.functions:
            gf = self.generate_function(func, iface, browser_class, handle_type)
            gf.has_explicit_poll = has_explicit_poll.get(func.name, False)
            functions.append(gf)

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
            js_type_for_handles=js_type_for_handles,
            functions=functions,
            type_aliases=type_aliases,
            create_function=create_function,
        )

    def generate_domain(self, domain: str, wit_path: Path) -> Optional[GeneratedDomain]:
        """Generate glue code for a domain."""

        try:
            pkg = parse_wit_file(wit_path)
        except Exception as e:
            print(f"[WARN] Failed to parse {wit_path}: {e}", file=sys.stderr)
            return None

        interfaces: List[GeneratedInterface] = []
        for iface in pkg.interfaces:
            if not iface.functions:
                continue
            gi = self.generate_interface(iface)
            interfaces.append(gi)

        if not interfaces:
            return None

        return GeneratedDomain(
            name=domain,
            export_name=kebab_to_camel(domain),
            interfaces=interfaces,
            interface_count=len(interfaces),
        )

    def _is_async_function(self, wit_name: str, func: WitFunction, iface_name: str) -> bool:
        """Determine if a function should use async poll pattern."""

        if wit_name.startswith("poll-"):
            return False

        if wit_name.startswith("get-") or wit_name.startswith("set-"):
            return False

        wit_lower = wit_name.lower()
        for pattern in ASYNC_PATTERNS:
            if wit_lower == pattern:
                return True
            if wit_lower == pattern:
                return True

        return False
