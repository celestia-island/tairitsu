#!/usr/bin/env python3
"""Logging utilities for the browser glue generator.

Delegates to the vendored unified logger (scripts/utils/logger.py) so generator
output matches the project-wide columnar format. Keeps the same public names
(log_info/log_ok/log_warn/log_error) — existing `from logger import ...` call
sites are unchanged.
"""
from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

# Load the vendored unified logger by explicit path — both files are named
# `logger.py`, so a bare `import logger` would re-import THIS module.
_unified = Path(__file__).resolve().parent.parent / "utils" / "logger.py"
_log = None
try:
    _spec = importlib.util.spec_from_file_location("_tairitsu_unified_logger", _unified)
    _mod = importlib.util.module_from_spec(_spec)
    assert _spec.loader is not None
    _spec.loader.exec_module(_mod)
    _log = _mod.Logger(source="tairitsu", module="gen")
except (OSError, ImportError, AssertionError):
    _log = None


def log_info(message: str) -> None:
    if _log:
        _log.info(message)
    else:
        print(f"[INFO] {message}")


def log_ok(message: str) -> None:
    if _log:
        _log.ok(message)
    else:
        print(f"[OK] {message}")


def log_warn(message: str) -> None:
    if _log:
        _log.warn(message)
    else:
        print(f"[WARN] {message}", file=sys.stderr)


def log_error(message: str) -> None:
    if _log:
        _log.error(message)
    else:
        print(f"[ERROR] {message}", file=sys.stderr)
