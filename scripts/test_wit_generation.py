#!/usr/bin/env python3
"""
WIT Generation Test Suite

Tests for WebIDL parsing and WIT generation correctness.
"""

from __future__ import annotations

import sys
from pathlib import Path

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
    TYPE_ALIASES,
)


def test_type_conversions():
    """Test WebIDL to WIT type conversions."""
    tests = [
        # Basic types
        ("boolean", "bool"),
        ("octet", "u8"),
        ("unsigned long", "u32"),
        ("unsigned long long", "u64"),
        ("DOMString", "string"),
        # Nullable types
        ("boolean?", "option<bool>"),
        ("DOMString?", "option<string>"),
        # Sequence types
        ("sequence<boolean>", "list<bool>"),
        ("sequence<DOMString>", "list<string>"),
        # Union types - boolean has higher priority than string
        ("(DOMString or boolean)", "option<bool>"),  # boolean priority in union
        ("(boolean or DOMString)", "bool"),  # boolean priority
        # Interface types
        ("Event", "u64"),
        ("Element", "u64"),
        # Nullable interface
        ("Event?", "option<u64>"),
    ]

    passed = 0
    failed = 0
    for webidl, expected_wit in tests:
        result = convert_type(webidl)
        if result == expected_wit:
            passed += 1
        else:
            print(f"FAIL: {webidl} -> {result} (expected {expected_wit})")
            failed += 1

    print(f"Type conversions: {passed}/{len(tests)} passed")
    return failed == 0


def test_identifier_conversion():
    """Test CamelCase to kebab-case conversion."""
    tests = [
        ("HTMLElement", "html-element"),
        ("XMLHttpRequest", "xml-http-request"),
        ("CSSStyleDeclaration", "css-style-declaration"),
        ("NodeList", "node-list"),
        ("addEventListener", "add-event-listener"),
    ]

    passed = 0
    failed = 0
    for input_name, expected in tests:
        result = camel_to_kebab(input_name)
        if result == expected:
            passed += 1
        else:
            print(f"FAIL: {input_name} -> {result} (expected {expected})")
            failed += 1

    print(f"Identifier conversions: {passed}/{len(tests)} passed")
    return failed == 0


def test_wit_ident_sanitization():
    """Test WIT identifier sanitization."""
    tests = [
        ("normal", "normal"),
        ("CamelCase", "camel-case"),
        ("123number", "n-123-number"),
        ("class", "%class"),  # WIT keyword
        ("string", "%string"),  # WIT keyword
    ]

    passed = 0
    failed = 0
    for input_name, expected in tests:
        result = sanitize_wit_ident(input_name)
        if result == expected:
            passed += 1
        else:
            print(f"FAIL: {input_name} -> {result} (expected {expected})")
            failed += 1

    print(f"WIT ident sanitization: {passed}/{len(tests)} passed")
    return failed == 0


def test_webidl_parsing():
    """Test WebIDL parsing functionality."""
    sample_webidl = """
        interface Event {
            readonly attribute DOMString type;
            readonly attribute EventTarget? target;
            void stopPropagation();
            void preventDefault();
        };

        interface MouseEvent : Event {
            readonly attribute long screenX;
            readonly attribute long screenY;
        };

        partial interface MouseEvent {
            readonly attribute long clientX;
            readonly attribute long clientY;
        };
    """

    interfaces = parse_webidl_file(sample_webidl, "test")

    # Check Event interface
    assert "Event" in interfaces, "Event interface not found"
    event = interfaces["Event"]
    assert len(event.members) == 3, f"Expected 3 members, got {len(event.members)}"
    assert event.inheritance is None, "Event should not have inheritance"

    # Check MouseEvent interface
    assert "MouseEvent" in interfaces, "MouseEvent interface not found"
    mouse_event = interfaces["MouseEvent"]
    assert mouse_event.inheritance == "Event", "MouseEvent should inherit from Event"
    # Should have 2 original members + 3 partial members = 5 total
    assert len(mouse_event.members) == 5, f"Expected 5 members, got {len(mouse_event.members)}"

    print("WebIDL parsing: All tests passed")
    return True


def test_typedef_resolution():
    """Test typedef (type alias) resolution."""
    TYPE_ALIASES.clear()
    TYPE_ALIASES["BinaryType"] = "DOMString"
    TYPE_ALIASES["ID"] = "DOMString"

    # Test alias resolution
    result = convert_type("BinaryType")
    expected = "string"  # DOMString -> string
    if result == expected:
        print("Typedef resolution: All tests passed")
        return True
    else:
        print(f"FAIL: BinaryType -> {result} (expected {expected})")
        return False


def run_all_tests() -> bool:
    """Run all test suites."""
    print("=" * 60)
    print("WIT Generation Test Suite")
    print("=" * 60)
    print()

    results = []
    results.append(test_type_conversions())
    results.append(test_identifier_conversion())
    results.append(test_wit_ident_sanitization())
    results.append(test_webidl_parsing())
    results.append(test_typedef_resolution())

    print()
    print("=" * 60)
    if all(results):
        print("All tests passed!")
        return True
    else:
        print(f"Some tests failed: {sum(results)}/{len(results)} passed")
        return False


if __name__ == "__main__":
    success = run_all_tests()
    sys.exit(0 if success else 1)
