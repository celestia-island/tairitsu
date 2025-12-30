#!/usr/bin/env python3
"""
Download WASI adapter files from GitHub that match the current Wasmtime version.

This script reads the Wasmtime version from Cargo.toml and downloads the
corresponding WASI adapter files from the Wasmtime GitHub releases.
"""

import re
import sys
import urllib.request
import urllib.error
import json
from pathlib import Path


def get_wasmtime_version(cargo_toml_path: str) -> str:
    """Read Wasmtime version from Cargo.toml."""
    with open(cargo_toml_path, 'r', encoding='utf-8') as f:
        content = f.read()

    # Match wasmtime = { version = "^25" } or wasmtime = "25"
    # Try to find version inside braces first
    match = re.search(
        r'wasmtime\s*=\s*\{[^}]*version\s*=\s*["\']?\^?(\d+)', content)
    if not match:
        # Try simple version format
        match = re.search(r'wasmtime\s*=\s*["\']\^?(\d+)["\']', content)

    if not match:
        raise ValueError("Could not find Wasmtime version in Cargo.toml")

    version = match.group(1)
    return f"{version}.0.0"  # Convert to semver format


def get_github_release_url(version: str) -> str:
    """Get the download URL for a specific Wasmtime version."""
    # Wasmtime releases are at: https://github.com/bytecodealliance/wasmtime/releases/tag/v{version}
    # The WASI adapters are in the assets
    return f"https://api.github.com/repos/bytecodealliance/wasmtime/releases/tags/v{version}"


def download_file(url: str, dest_path: Path) -> None:
    """Download a file from URL to destination path."""
    print(f"Downloading {url}...")
    print(f"  to {dest_path}")

    dest_path.parent.mkdir(parents=True, exist_ok=True)

    with urllib.request.urlopen(url) as response:
        dest_path.write_bytes(response.read())


def download_wasi_adapters(version: str, res_dir: Path) -> None:
    """Download WASI adapter files for the given Wasmtime version."""
    api_url = get_github_release_url(version)

    print(f"Fetching release info for Wasmtime {version}...")
    print(f"  API URL: {api_url}")

    try:
        with urllib.request.urlopen(api_url) as response:
            release_data = json.loads(response.read())
    except urllib.error.HTTPError as e:
        print(f"[ERROR] Failed to fetch release info: {e}")
        print(f"Please check if version {version} exists at:")
        print("  https://github.com/bytecodealliance/wasmtime/releases")
        sys.exit(1)

    # Find the WASI adapter assets
    wasi_files = {
        'wasi_snapshot_preview1.command.wasm': None,
        'wasi_snapshot_preview1.reactor.wasm': None,
    }

    for asset in release_data.get('assets', []):
        asset_name = asset['name']
        if asset_name in wasi_files:
            wasi_files[asset_name] = asset['browser_download_url']

    # Download missing files
    downloaded = False
    for filename, url in wasi_files.items():
        if url is None:
            print(f"[WARNING] Could not find {filename} in release assets")
            continue

        dest_path = res_dir / filename
        download_file(url, dest_path)
        downloaded = True

    if not downloaded:
        print("[ERROR] No WASI adapter files were downloaded")
        sys.exit(1)


def main():
    """Main entry point."""
    script_dir = Path(__file__).parent
    project_root = script_dir.parent
    res_dir = project_root / "packages" / "runtime" / "res"
    cargo_toml_path = project_root / "packages" / "runtime" / "Cargo.toml"

    if not cargo_toml_path.exists():
        print(f"[ERROR] Cargo.toml not found at {cargo_toml_path}")
        sys.exit(1)

    try:
        wasmtime_version = get_wasmtime_version(str(cargo_toml_path))
        print(f"Detected Wasmtime version: {wasmtime_version}")
    except ValueError as e:
        print(f"[ERROR] {e}")
        sys.exit(1)

    download_wasi_adapters(wasmtime_version, res_dir)

    # Use ASCII-safe output for Windows compatibility
    print("[OK] WASI adapters downloaded successfully")


if __name__ == "__main__":
    main()
