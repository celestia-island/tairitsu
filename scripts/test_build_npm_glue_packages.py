#!/usr/bin/env python3
"""Fail-loud tests for build_npm_glue_packages.py (W1 reproducible build).

The wasm copy step used to SKIP silently when the wasm32-wasip2 target
directory was missing — that is how npm package 0.5.4 shipped with zero
.wasm files and zero instantiate references. These tests pin the new
contract: a full build without wasm components FAILS; skipping must be
an explicit --gen choice.

Run with: pytest scripts/test_build_npm_glue_packages.py -v
"""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

scripts_dir = Path(__file__).parent
sys.path.insert(0, str(scripts_dir))

import build_npm_glue_packages as glue  # noqa: E402  (imports scripts/ via sys.path above)


@pytest.fixture()
def fake_workspace(tmp_path, monkeypatch):
    """Redirect the module's workspace constants at a throwaway tree."""
    monkeypatch.setattr(glue, "WORKSPACE_ROOT", tmp_path)
    monkeypatch.setattr(glue, "DIST_DIR", tmp_path / "pkg" / "dist")
    return tmp_path


def test_full_build_fails_loud_when_wasm_target_dir_missing(fake_workspace):
    with pytest.raises(SystemExit) as exc:
        glue.copy_wasm_components()
    assert exc.value.code != 0
    assert "wasm32-wasip2" in exc.value.code


def test_full_build_fails_loud_when_target_dir_has_no_wasm(fake_workspace, capsys):
    (fake_workspace / "target" / "wasm32-wasip2" / "release").mkdir(parents=True)
    with pytest.raises(SystemExit) as exc:
        glue.copy_wasm_components()
    assert exc.value.code != 0
    assert "no .wasm files" in exc.value.code


def test_full_build_error_names_the_explicit_opt_out(fake_workspace):
    with pytest.raises(SystemExit) as exc:
        glue.copy_wasm_components()
    assert "--gen" in exc.value.code, "the error must name the explicit skip flag"


def test_wasm_copy_succeeds_when_components_exist(fake_workspace, capsys):
    release = fake_workspace / "target" / "wasm32-wasip2" / "release"
    release.mkdir(parents=True)
    (release / "demo.wasm").write_bytes(b"\0asm-component-bytes")
    glue.copy_wasm_components()  # must not raise
    copied = glue.DIST_DIR / "wasm" / "demo.wasm"
    assert copied.read_bytes() == b"\0asm-component-bytes"
    assert "demo.wasm" in capsys.readouterr().out


def test_gen_mode_records_explicit_skip(fake_workspace, monkeypatch, capsys):
    # --gen must BOTH skip the wasm copy AND print the recorded notice —
    # pinning the main() wiring, not just copy_wasm_components.
    monkeypatch.setattr(glue, "build_domain", lambda name: True)

    def _must_not_run():
        raise AssertionError("wasm copy must not run in --gen mode")

    monkeypatch.setattr(glue, "copy_wasm_components", _must_not_run)
    monkeypatch.setattr(sys, "argv", ["build_npm_glue_packages.py", "--gen"])
    glue.main()  # must not raise
    assert "explicit opt-out (--gen" in capsys.readouterr().out


def test_default_mode_requires_wasm_copy(fake_workspace, monkeypatch):
    # The default (full) mode must invoke the fail-loud wasm copy — a
    # mutation that silently skips it in main() fails here.
    monkeypatch.setattr(glue, "build_domain", lambda name: True)

    def _sentinel():
        sys.exit("copy_wasm_components was invoked")

    monkeypatch.setattr(glue, "copy_wasm_components", _sentinel)
    monkeypatch.setattr(sys, "argv", ["build_npm_glue_packages.py"])
    with pytest.raises(SystemExit) as exc:
        glue.main()
    assert "invoked" in str(exc.value.code)
