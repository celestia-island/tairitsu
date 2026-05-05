#!/usr/bin/env python3
"""
WIT Generation Test Suite

Tests for WebIDL parsing and WIT generation correctness.

Run with: pytest scripts/test_wit_generation.py -v
"""

from __future__ import annotations

import sys
from pathlib import Path
from typing import Dict, List

import pytest

# Add scripts directory to path
scripts_dir = Path(__file__).parent
sys.path.insert(0, str(scripts_dir))

from generate_browser_wit import (
    WebIDLInterface,
    WebIDLMember,
    WebIDLParam,
    convert_type,
    camel_to_kebab,
    sanitize_wit_ident,
    parse_webidl_file,
    _parse_params,
    _parse_member,
    TYPE_ALIASES,
    WIT_KEYWORDS,
    WEBIDL_TO_WIT,
)

# =============================================================================
# Fixtures
# =============================================================================


@pytest.fixture(autouse=True)
def reset_type_aliases() -> None:
    """Reset TYPE_ALIASES before each test."""
    TYPE_ALIASES.clear()


@pytest.fixture
def sample_webidl_simple() -> str:
    """Simple WebIDL interface for testing."""
    return """
    interface ExampleInterface {
        attribute DOMString name;
        readonly attribute long id;
        void doSomething();
        long calculate(in long a, in long b);
    };
    """


@pytest.fixture
def sample_webidl_with_inheritance() -> str:
    """WebIDL interface with inheritance."""
    return """
    interface BaseInterface {
        attribute DOMString baseProp;
    };

    interface DerivedInterface : BaseInterface {
        attribute DOMString derivedProp;
        void derivedMethod();
    };
    """


@pytest.fixture
def sample_webidl_partial() -> str:
    """WebIDL with partial interface."""
    return """
    interface MouseEvent {
        readonly attribute long screenX;
        readonly attribute long screenY;
    };

    partial interface MouseEvent {
        readonly attribute long clientX;
        readonly attribute long clientY;
    };
    """


@pytest.fixture
def sample_webidl_with_params() -> str:
    """WebIDL with complex parameter types."""
    return """
    interface ComplexInterface {
        void methodWithOptional(optional DOMString name = "");
        void methodWithVariadic(DOMString... values);
        void methodWithMixed(long required, optional boolean flag = false);
    };
    """


# =============================================================================
# Type Conversion Tests (convert_type function)
# =============================================================================


class TestTypeConversion:
    """Test WebIDL to WIT type conversions."""

    @pytest.mark.parametrize(
        "webidl_type,expected_wit",
        [
            # Boolean types
            ("boolean", "bool"),
            # Integer types
            ("byte", "s8"),
            ("octet", "u8"),
            ("short", "s16"),
            ("unsigned short", "u16"),
            ("long", "s32"),
            ("unsigned long", "u32"),
            ("long long", "s64"),
            ("unsigned long long", "u64"),
            # Float types
            ("float", "f32"),
            ("unrestricted float", "f32"),
            ("double", "f64"),
            ("unrestricted double", "f64"),
            # String types
            ("DOMString", "string"),
            ("USVString", "string"),
            ("ByteString", "string"),
            ("CSSOMString", "string"),
            # Void/undefined
            ("void", "_"),
            ("undefined", "_"),
            # Special numeric types
            ("DOMHighResTimeStamp", "f64"),
            ("DOMTimeStamp", "u64"),
            # Buffer types
            ("ArrayBuffer", "list<u8>"),
            ("SharedArrayBuffer", "list<u8>"),
            ("DataView", "list<u8>"),
            ("Int8Array", "list<s8>"),
            ("Uint8Array", "list<u8>"),
            ("Uint16Array", "list<u16>"),
            ("Uint32Array", "list<u32>"),
            ("Float32Array", "list<f32>"),
            ("Float64Array", "list<f64>"),
            # String lists
            ("DOMStringList", "list<string>"),
            # Callback/handle types
            ("EventHandler", "u64"),
            ("Function", "u64"),
            ("MutationCallback", "u64"),
            # Generic types
            ("any", "string"),
            ("object", "u64"),
            ("symbol", "u64"),
            ("bigint", "u64"),
        ],
    )
    def test_basic_type_mapping(self, webidl_type: str, expected_wit: str) -> None:
        """Test basic WebIDL to WIT type mappings."""
        assert convert_type(webidl_type) == expected_wit

    @pytest.mark.parametrize(
        "webidl_type,expected_wit",
        [
            # Nullable primitives
            ("boolean?", "option<bool>"),
            ("long?", "option<s32>"),
            ("unsigned long?", "option<u32>"),
            ("double?", "option<f64>"),
            ("DOMString?", "option<string>"),
            # Note: void? returns option<_> which is the actual behavior
            # but _ is a special marker for void return, not really a type
            ("void?", "_"),  # void is special, nullable void is still void
            # Nullable interface types
            ("Event?", "option<u64>"),
            ("Element?", "option<u64>"),
            ("Node?", "option<u64>"),
            # Nullable callback types
            ("EventHandler?", "option<u64>"),
        ],
    )
    def test_nullable_types(self, webidl_type: str, expected_wit: str) -> None:
        """Test nullable type conversion (T?)."""
        assert convert_type(webidl_type) == expected_wit

    @pytest.mark.parametrize(
        "webidl_type,expected_wit",
        [
            # Boolean priority in unions
            ("(boolean or DOMString)", "bool"),
            ("(DOMString or boolean)", "bool"),
            ("(boolean or long)", "bool"),
            # String priority (when no boolean)
            ("(DOMString or USVString)", "string"),
            ("(USVString or ByteString)", "string"),
            ("(DOMString or long)", "string"),
            # Numeric priority
            ("(long or double)", "f64"),
            ("(short or unsigned long)", "f64"),
            # With undefined/null (should be ignored)
            ("(boolean or undefined)", "bool"),
            ("(DOMString or null)", "string"),
            # Nullable unions
            ("(boolean or DOMString)?", "option<bool>"),
            ("(DOMString or long)?", "option<string>"),
            # Complex unions
            ("(boolean or DOMString or long)", "bool"),
            ("(DOMString or long or double)", "string"),
        ],
    )
    def test_union_types(self, webidl_type: str, expected_wit: str) -> None:
        """Test union type conversion (T1 or T2)."""
        assert convert_type(webidl_type) == expected_wit

    @pytest.mark.parametrize(
        "webidl_type,expected_wit",
        [
            # Basic sequences
            ("sequence<boolean>", "list<bool>"),
            ("sequence<long>", "list<s32>"),
            ("sequence<DOMString>", "list<string>"),
            ("sequence<Event>", "list<u64>"),
            # FrozenArray
            ("FrozenArray<long>", "list<s32>"),
            ("FrozenArray<DOMString>", "list<string>"),
            # ObservableArray
            ("ObservableArray<double>", "list<f64>"),
            # Nested sequences (simplified)
            ("sequence<sequence<long>>", "list<list<s32>>"),
            # Nullable sequences
            ("sequence<DOMString>?", "option<list<string>>"),
            ("sequence<long>?", "option<list<s32>>"),
        ],
    )
    def test_sequence_types(self, webidl_type: str, expected_wit: str) -> None:
        """Test sequence type conversion (sequence<T>)."""
        assert convert_type(webidl_type) == expected_wit

    @pytest.mark.parametrize(
        "webidl_type,expected_wit",
        [
            # Basic Promise
            ("Promise<void>", "u64"),
            ("Promise<DOMString>", "u64"),
            ("Promise<Event>", "u64"),
            ("Promise<boolean>", "u64"),
            # Nullable Promise
            ("Promise<void>?", "option<u64>"),
            ("Promise<DOMString>?", "option<u64>"),
        ],
    )
    def test_promise_types(self, webidl_type: str, expected_wit: str) -> None:
        """Test Promise type conversion."""
        assert convert_type(webidl_type) == expected_wit

    @pytest.mark.parametrize(
        "webidl_type,expected_wit",
        [
            # Basic record
            ("record<DOMString, DOMString>", "string"),
            ("record<USVString, long>", "string"),
            # Nullable record
            ("record<DOMString, DOMString>?", "option<string>"),
        ],
    )
    def test_record_types(self, webidl_type: str, expected_wit: str) -> None:
        """Test record type conversion."""
        assert convert_type(webidl_type) == expected_wit

    @pytest.mark.parametrize(
        "webidl_type,expected_wit",
        [
            # DOMRect special handling
            ("DOMRect", "dom-rect"),
            ("DOMRectReadOnly", "dom-rect"),
            ("DOMRectInit", "dom-rect"),
            ("TextRectangle", "dom-rect"),
            # Nullable DOMRect
            ("DOMRect?", "option<dom-rect>"),
            ("DOMRectReadOnly?", "option<dom-rect>"),
        ],
    )
    def test_special_record_types(self, webidl_type: str, expected_wit: str) -> None:
        """Test special record type overrides (DOMRect variants)."""
        assert convert_type(webidl_type) == expected_wit

    def test_interface_types_default_to_u64(self) -> None:
        """Test that unknown interface types default to u64 handles."""
        # Common browser interfaces
        for iface in ["Event", "Element", "Node", "Document", "Window"]:
            assert convert_type(iface) == "u64"
            assert convert_type(f"{iface}?") == "option<u64>"

    def test_extended_attribute_removal(self) -> None:
        """Test that extended attributes in [ ] are removed."""
        # Extended attributes should be stripped
        assert convert_type("[EnforceRange] long") == "s32"
        assert convert_type("[Clamp] unsigned long") == "u32"


# =============================================================================
# Identifier Conversion Tests
# =============================================================================


class TestCamelToKebab:
    """Test CamelCase to kebab-case conversion."""

    @pytest.mark.parametrize(
        "input_name,expected",
        [
            # Simple cases
            ("HTMLElement", "html-element"),
            ("XMLHttpRequest", "xml-http-request"),
            ("CSSStyleDeclaration", "css-style-declaration"),
            # camelCase
            ("addEventListener", "add-event-listener"),
            ("getElementById", "get-element-by-id"),
            ("innerHTML", "inner-html"),
            # Single word
            ("name", "name"),
            ("Name", "name"),
            # Numbers
            ("node123", "node123"),
            ("test123Value", "test123-value"),
            # Multiple consecutive caps
            ("XMLParser", "xml-parser"),
            ("IOStream", "io-stream"),
            # Underscores (should be replaced with hyphens)
            ("my_variable", "my-variable"),
            ("__private__", "private"),
            # Already kebab
            ("already-kebab", "already-kebab"),
            # Empty after cleaning
            ("", "unknown"),
            ("___", "unknown"),
        ],
    )
    def test_camel_to_kebab(self, input_name: str, expected: str) -> None:
        """Test camelCase to kebab-case conversion."""
        assert camel_to_kebab(input_name) == expected

    def test_leading_digit_handling(self) -> None:
        """Test that identifiers starting with digits get 'n-' prefix."""
        assert camel_to_kebab("123test") == "n-123test"
        assert camel_to_kebab("0abc") == "n-0abc"

    def test_multiple_consecutive_hyphens_collapsed(self) -> None:
        """Test that multiple consecutive hyphens are collapsed."""
        assert camel_to_kebab("test---case") == "test-case"
        assert camel_to_kebab("test___case") == "test-case"


class TestSanitizeWitIdent:
    """Test WIT identifier sanitization."""

    @pytest.mark.parametrize(
        "input_name,expected",
        [
            # Normal identifiers
            ("normal", "normal"),
            ("CamelCase", "camel-case"),
            ("already-kebab", "already-kebab"),
            # WIT keywords should be escaped
            ("use", "%use"),
            ("type", "%type"),
            ("func", "%func"),
            ("list", "%list"),
            ("string", "%string"),
            ("bool", "%bool"),
            ("u8", "%u8"),
            ("u64", "%u64"),
            ("from", "%from"),
            ("true", "%true"),
            # Non-alphanumeric characters removed
            # Note: spaces are also removed, not just replaced
            ("test@var", "testvar"),
            ("test#var", "testvar"),
            ("test$var", "testvar"),
            ("test var", "testvar"),
            # Multiple special chars
            ("test@#$var", "testvar"),
            # Empty after sanitization
            ("@#$", "unknown"),
            ("", "unknown"),
        ],
    )
    def test_sanitize_wit_ident(self, input_name: str, expected: str) -> None:
        """Test WIT identifier sanitization."""
        assert sanitize_wit_ident(input_name) == expected

    def test_all_wit_keywords_are_escaped(self) -> None:
        """Verify all WIT keywords are properly escaped."""
        for keyword in WIT_KEYWORDS:
            result = sanitize_wit_ident(keyword)
            assert result == f"%{keyword}", f"Keyword '{keyword}' should be escaped"


# =============================================================================
# WebIDL Parsing Tests
# =============================================================================


class TestWebIDLParsing:
    """Test WebIDL parsing functionality."""

    def test_parse_simple_interface(self, sample_webidl_simple: str) -> None:
        """Test parsing a simple interface."""
        interfaces = parse_webidl_file(sample_webidl_simple, "test")

        assert "ExampleInterface" in interfaces
        iface = interfaces["ExampleInterface"]
        assert iface.name == "ExampleInterface"
        assert iface.inheritance is None
        assert not iface.is_partial
        assert not iface.is_mixin

    def test_parse_interface_members(self, sample_webidl_simple: str) -> None:
        """Test parsing interface members (attributes and operations)."""
        interfaces = parse_webidl_file(sample_webidl_simple, "test")
        iface = interfaces["ExampleInterface"]

        # Should have attributes and operations
        assert len(iface.members) >= 4

        # Check for attribute members
        attrs = [m for m in iface.members if m.kind == "attribute"]
        assert len(attrs) >= 2

        # Check for operation members
        ops = [m for m in iface.members if m.kind == "operation"]
        assert len(ops) >= 2

    def test_parse_interface_inheritance(self, sample_webidl_with_inheritance: str) -> None:
        """Test parsing interface with inheritance."""
        interfaces = parse_webidl_file(sample_webidl_with_inheritance, "test")

        # Check base interface
        assert "BaseInterface" in interfaces
        base = interfaces["BaseInterface"]
        assert base.inheritance is None

        # Check derived interface
        assert "DerivedInterface" in interfaces
        derived = interfaces["DerivedInterface"]
        assert derived.inheritance == "BaseInterface"

    def test_parse_partial_interface(self, sample_webidl_partial: str) -> None:
        """Test parsing partial interfaces."""
        interfaces = parse_webidl_file(sample_webidl_partial, "test")

        assert "MouseEvent" in interfaces
        mouse_event = interfaces["MouseEvent"]

        # Should have members from both base and partial
        assert len(mouse_event.members) >= 4

        # Check for expected attributes
        attr_names = {m.name for m in mouse_event.members if m.kind == "attribute"}
        assert "screenX" in attr_names
        assert "screenY" in attr_names
        assert "clientX" in attr_names
        assert "clientY" in attr_names

    def test_parse_readonly_attribute(self) -> None:
        """Test parsing readonly attributes."""
        webidl = """
        interface Example {
            readonly attribute DOMString readOnlyProp;
            attribute DOMString writableProp;
        };
        """
        interfaces = parse_webidl_file(webidl, "test")
        iface = interfaces["Example"]

        readonly_attrs = [m for m in iface.members if m.kind == "attribute" and m.readonly]
        writable_attrs = [m for m in iface.members if m.kind == "attribute" and not m.readonly]

        assert len(readonly_attrs) == 1
        assert readonly_attrs[0].name == "readOnlyProp"

        assert len(writable_attrs) == 1
        assert writable_attrs[0].name == "writableProp"

    def test_parse_static_members(self) -> None:
        """Test parsing static members."""
        webidl = """
        interface Example {
            static DOMString staticMethod();
            static readonly attribute long staticProp;
        };
        """
        interfaces = parse_webidl_file(webidl, "test")
        iface = interfaces["Example"]

        static_members = [m for m in iface.members if m.static]
        assert len(static_members) >= 2

    def test_parse_mixin_interface(self) -> None:
        """Test parsing mixin interfaces."""
        webidl = """
        interface mixin ExampleMixin {
            void mixinMethod();
        };

        interface Example {
            void regularMethod();
        };
        """
        interfaces = parse_webidl_file(webidl, "test")

        assert "ExampleMixin" in interfaces
        mixin = interfaces["ExampleMixin"]
        assert mixin.is_mixin

        assert "Example" in interfaces
        regular = interfaces["Example"]
        assert not regular.is_mixin


class TestParameterParsing:
    """Test WebIDL parameter parsing."""

    def test_parse_simple_parameters(self) -> None:
        """Test parsing simple parameters."""
        params = _parse_params("in long a, in DOMString b, in boolean c")

        assert len(params) == 3
        assert params[0].name == "a"
        # Note: 'in' keyword is preserved in the idl_type
        assert params[0].idl_type == "in long"
        assert not params[0].optional

        assert params[1].name == "b"
        assert params[1].idl_type == "in DOMString"

        assert params[2].name == "c"
        assert params[2].idl_type == "in boolean"

    def test_parse_optional_parameters(self) -> None:
        """Test parsing optional parameters."""
        params = _parse_params("optional DOMString name, optional long value")

        assert len(params) == 2
        assert params[0].optional
        assert params[0].idl_type == "DOMString"

        assert params[1].optional
        assert params[1].idl_type == "long"

    def test_parse_variadic_parameters(self) -> None:
        """Test parsing variadic (rest) parameters."""
        params = _parse_params("DOMString... values")

        assert len(params) == 1
        assert params[0].variadic
        assert params[0].name == "values"
        assert params[0].idl_type == "DOMString"

    def test_parse_parameters_with_defaults(self) -> None:
        """Test parsing parameters with default values."""
        params = _parse_params('DOMString name = "", long count = 0')

        assert len(params) == 2
        # Default values should be stripped
        assert params[0].name == "name"
        assert params[0].idl_type == "DOMString"

        assert params[1].name == "count"
        assert params[1].idl_type == "long"

    def test_parse_complex_parameters(self) -> None:
        """Test parsing complex parameter types."""
        params = _parse_params("sequence<long> numbers, record<DOMString, long> map")

        assert len(params) == 2
        assert params[0].idl_type == "sequence<long>"
        assert params[1].idl_type == "record<DOMString, long>"

    def test_parse_empty_parameters(self) -> None:
        """Test parsing empty parameter list."""
        params = _parse_params("")
        assert len(params) == 0

        params = _parse_params("   ")
        assert len(params) == 0


class TestMemberParsing:
    """Test individual member parsing."""

    def test_parse_attribute_member(self) -> None:
        """Test parsing attribute members."""
        stmt = "readonly attribute DOMString href"
        member = _parse_member(stmt)

        assert member is not None
        assert member.kind == "attribute"
        assert member.name == "href"
        assert member.idl_type == "DOMString"
        assert member.readonly
        assert not member.static

    def test_parse_writable_attribute(self) -> None:
        """Test parsing writable (non-readonly) attributes."""
        stmt = "attribute DOMString title"
        member = _parse_member(stmt)

        assert member is not None
        assert member.kind == "attribute"
        assert member.name == "title"
        assert not member.readonly

    def test_parse_static_attribute(self) -> None:
        """Test parsing static attributes."""
        stmt = "static readonly attribute long length"
        member = _parse_member(stmt)

        assert member is not None
        assert member.kind == "attribute"
        assert member.static
        assert member.readonly

    def test_parse_operation_member(self) -> None:
        """Test parsing operation members."""
        stmt = "void stopPropagation()"
        member = _parse_member(stmt)

        assert member is not None
        assert member.kind == "operation"
        assert member.name == "stopPropagation"
        assert member.idl_type == "void"

    def test_parse_operation_with_return_type(self) -> None:
        """Test parsing operations with return types."""
        stmt = "DOMString getItem(in long index)"
        member = _parse_member(stmt)

        assert member is not None
        assert member.kind == "operation"
        assert member.name == "getItem"
        assert member.idl_type == "DOMString"
        assert len(member.params) == 1
        assert member.params[0].name == "index"

    def test_parse_static_operation(self) -> None:
        """Test parsing static operations."""
        stmt = "static DOMString create()"
        member = _parse_member(stmt)

        assert member is not None
        assert member.kind == "operation"
        assert member.static

    def test_parse_constructor_is_skipped(self) -> None:
        """Test that constructors are skipped."""
        stmt = "constructor()"
        member = _parse_member(stmt)

        assert member is None

    def test_parse_const_is_skipped(self) -> None:
        """Test that const members are skipped."""
        stmt = "const unsigned short NODE_TYPE = 1"
        member = _parse_member(stmt)

        assert member is None

    def test_parse_iterable_is_skipped(self) -> None:
        """Test that iterable declarations are skipped."""
        stmt = "iterable<long>"
        member = _parse_member(stmt)

        assert member is None

    def test_parse_stringifier_attribute(self) -> None:
        """Test parsing stringifier attributes."""
        stmt = "stringifier attribute USVString href"
        member = _parse_member(stmt)

        assert member is not None
        assert member.kind == "attribute"
        assert member.name == "href"


# =============================================================================
# Typedef Resolution Tests
# =============================================================================


class TestTypedefResolution:
    """Test typedef (type alias) resolution."""

    def test_typedef_resolution_in_convert_type(self) -> None:
        """Test that typedefs are resolved in convert_type."""
        TYPE_ALIASES["BinaryType"] = "DOMString"
        TYPE_ALIASES["CustomID"] = "unsigned long"

        assert convert_type("BinaryType") == "string"
        assert convert_type("CustomID") == "u32"

    def test_nullable_typedef(self) -> None:
        """Test nullable typedef types.

        Note: The current implementation does NOT resolve typedefs
        when they have a nullable suffix. The typedef lookup happens
        before nullable processing, so "MaybeString?" won't match
        "MaybeString" in TYPE_ALIASES.
        """
        TYPE_ALIASES["MaybeString"] = "DOMString"

        # Without nullable, it resolves correctly
        assert convert_type("MaybeString") == "string"

        # With nullable, it doesn't resolve (limitation of current impl)
        # This is expected behavior - unknown type defaults to u64
        assert convert_type("MaybeString?") == "option<u64>"

    def test_typedef_in_sequence(self) -> None:
        """Test typedef used in sequence."""
        TYPE_ALIASES["CustomString"] = "DOMString"

        assert convert_type("sequence<CustomString>") == "list<string>"

    def test_typedef_in_union(self) -> None:
        """Test typedef used in union types.

        Note: Similar to nullable typedefs, typedefs inside unions
        are not resolved because the union processing happens before
        typedef resolution can be applied to individual union members.
        """
        TYPE_ALIASES["CustomBool"] = "boolean"

        # The typedef inside a union is not resolved
        # Union falls back to string (first non-undefined/null type)
        assert convert_type("(CustomBool or DOMString)") == "string"

        # But the typedef alone resolves correctly
        assert convert_type("CustomBool") == "bool"

    def test_parse_typedef_from_webidl(self) -> None:
        """Test parsing typedef from WebIDL text."""
        webidl = """
        typedef DOMString BinaryType;
        typedef unsigned long CustomID;

        interface Example {
            BinaryType getType();
            CustomID getID();
        };
        """
        interfaces = parse_webidl_file(webidl, "test")

        # Typedefs should be registered
        assert "BinaryType" in TYPE_ALIASES
        assert TYPE_ALIASES["BinaryType"] == "DOMString"
        assert "CustomID" in TYPE_ALIASES
        assert TYPE_ALIASES["CustomID"] == "unsigned long"

        # And convert_type should resolve them
        assert convert_type("BinaryType") == "string"
        assert convert_type("CustomID") == "u32"


# =============================================================================
# Integration Tests
# =============================================================================


class TestIntegration:
    """Integration tests for complete workflows."""

    def test_complete_parsing_and_conversion_workflow(self) -> None:
        """Test complete workflow from WebIDL to WIT types."""
        webidl = """
        typedef sequence<DOMString> StringList;

        interface Example {
            attribute DOMString name;
            readonly attribute long id;
            void doSomething(optional boolean flag = false);
            StringList getNames();
            long calculate(in long a, in long b);
        };
        """

        interfaces = parse_webidl_file(webidl, "test")
        assert "Example" in interfaces

        iface = interfaces["Example"]
        assert len(iface.members) >= 5

        # Verify type conversions work correctly
        for member in iface.members:
            if member.kind == "attribute":
                wit_type = convert_type(member.idl_type)
                assert wit_type in ("string", "s32", "_")
            elif member.kind == "operation":
                ret_type = convert_type(member.idl_type)
                assert ret_type in ("_", "string", "s32", "list<string>")

    def test_real_world_dom_interface(self) -> None:
        """Test parsing a realistic DOM interface."""
        webidl = """
        interface Event {
            readonly attribute DOMString type;
            readonly attribute EventTarget? target;
            readonly attribute boolean bubbles;
            void stopPropagation();
            void preventDefault();
        };

        interface EventTarget {
            void addEventListener(DOMString type, EventHandler callback);
            void removeEventListener(DOMString type, EventHandler callback);
        };
        """

        interfaces = parse_webidl_file(webidl, "test")

        # Check Event interface
        assert "Event" in interfaces
        event = interfaces["Event"]
        assert event.inheritance is None

        event_attrs = [m for m in event.members if m.kind == "attribute"]
        assert len(event_attrs) >= 3

        # Check EventTarget interface
        assert "EventTarget" in interfaces
        target = interfaces["EventTarget"]

        target_ops = [m for m in target.members if m.kind == "operation"]
        assert len(target_ops) >= 2
