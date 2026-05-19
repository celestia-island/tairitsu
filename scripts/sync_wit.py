#!/usr/bin/env python3
"""Sync composed WIT files from browser-worlds into tairitsu-web for crates.io publishing."""
from __future__ import annotations

import shutil
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

SRC = REPO_ROOT / "packages" / "browser-worlds" / "wit" / "composed"
DST = REPO_ROOT / "packages" / "web" / "wit" / "composed"


def main():
    if DST.exists():
        shutil.rmtree(str(DST))

    shutil.copytree(str(SRC), str(DST))

    wit_files = list(DST.glob("*.wit"))
    print(f"Synced {len(wit_files)} WIT files -> {DST}")


if __name__ == "__main__":
    main()
