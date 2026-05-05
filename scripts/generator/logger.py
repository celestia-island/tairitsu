#!/usr/bin/env python3
"""Logging utilities for the browser glue generator."""

import sys


def log_info(message: str) -> None:
    """Print an info message."""
    print(f"[INFO] {message}")


def log_ok(message: str) -> None:
    """Print a success message."""
    print(f"[OK] {message}")


def log_warn(message: str) -> None:
    """Print a warning message to stderr."""
    print(f"[WARN] {message}", file=sys.stderr)


def log_error(message: str) -> None:
    """Print an error message to stderr."""
    print(f"[ERROR] {message}", file=sys.stderr)
