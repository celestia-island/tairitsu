#!/usr/bin/env python3
"""Clean IDL cache directories."""
from __future__ import annotations

import shutil
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent


def main():
    print("Removing IDL caches...")
    for d in [
        REPO_ROOT / "scripts" / "idl-cache",
        REPO_ROOT / "target" / "tairitsu-wit" / "webidl-cache",
    ]:
        if d.exists():
            shutil.rmtree(str(d))
            print(f"  ✓ Removed {d}")
        else:
            print(f"  - {d} (not found)")


if __name__ == "__main__":
    main()
