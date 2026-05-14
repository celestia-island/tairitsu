#!/usr/bin/env python3
"""Build the unified @celestia/tairitsu-browser-glue npm package.

Reads runtime modules from packages/browser-glue/src/runtime/
and generates a single aggregated npm package at packages/npm/browser-glue/.

Also generates the backward-compatible glue-full shim.

Usage:
    python scripts/build_npm_glue_packages.py          # Generate + build
    python scripts/build_npm_glue_packages.py --gen     # Generate only
    python scripts/build_npm_glue_packages.py --build   # Build only (must generate first)
"""

import json
import os
import subprocess
import sys
from pathlib import Path

WORKSPACE_ROOT = Path(__file__).resolve().parent.parent
RUNTIME_DIR = WORKSPACE_ROOT / "packages" / "browser-glue" / "src" / "runtime"
NPM_DIR = WORKSPACE_ROOT / "packages" / "npm"
VERSION = "0.5.0"
SCOPE = "@celestia"

DOMAIN_MAP = {
    "dom": {
        "description": "DOM manipulation glue — document, element, node operations",
        "interfaces": {
            "document": "document",
            "element": "element",
            "node": "node",
            "non-element-parent-node": "nonElementParentNode",
            "parent-node": "parentNode",
        },
        "runtime_modules": ["document", "element", "node", "nonElementParentNode", "parentNode"],
    },
    "events": {
        "description": "Event handling glue — DOM events, event targets",
        "interfaces": {
            "event": "event",
            "event-target": "eventTarget",
        },
        "runtime_modules": ["event", "eventTarget"],
    },
    "css": {
        "description": "CSS glue — style declarations, inline styles, class lists",
        "interfaces": {
            "css-style-declaration": "cssStyleDeclaration",
            "element-css-inline-style": "elementCssInlineStyle",
        },
        "runtime_modules": ["cssStyleDeclaration", "elementCssInlineStyle"],
    },
    "html": {
        "description": "HTML glue — forms, input elements, selection",
        "interfaces": {
            "html-element": "htmlElement",
        },
        "runtime_modules": ["htmlElement"],
    },
    "observers": {
        "description": "Observer glue — MutationObserver, IntersectionObserver",
        "interfaces": {
            "mutation-observer": "mutationObserver",
            "intersection-observer": "intersectionObserver",
        },
        "runtime_modules": ["mutationObserver", "intersectionObserver"],
    },
    "resize-observer": {
        "description": "ResizeObserver glue",
        "interfaces": {"resize-observer": "resizeObserver"},
        "runtime_modules": ["resizeObserver"],
    },
    "platform": {
        "description": "Platform glue — viewport, timing, RAF",
        "interfaces": {
            "performance": "performance",
            "animation-frame": "animationFrame",
            "timer": "timer",
            "clipboard": "clipboard",
            "content-editable": "contentEditable",
            "scroll": "scroll",
            "resize": "resize",
            "media-query": "mediaQuery",
            "query-selector": "querySelector",
        },
        "runtime_modules": [
            "performance", "animationFrame", "timer", "clipboard",
            "contentEditable", "scroll", "resize", "mediaQuery", "querySelector",
        ],
    },
    "auth": {
        "description": "Web Authentication glue",
        "interfaces": {"web-authentication": "webAuthentication"},
        "runtime_modules": ["webAuthentication"],
    },
    "canvas": {
        "description": "Canvas 2D glue",
        "interfaces": {"canvas": "canvas"},
        "runtime_modules": ["canvas"],
    },
    "crypto": {
        "description": "Web Crypto API glue",
        "interfaces": {"crypto": "crypto"},
        "runtime_modules": ["crypto"],
    },
    "device": {
        "description": "Device APIs glue",
        "interfaces": {"device": "device"},
        "runtime_modules": ["device"],
    },
    "fetch": {
        "description": "Fetch API glue",
        "interfaces": {"fetch": "fetch"},
        "runtime_modules": ["fetch"],
    },
    "file-api": {
        "description": "File API glue",
        "interfaces": {"file": "file"},
        "runtime_modules": [],
    },
    "geolocation": {
        "description": "Geolocation API glue",
        "interfaces": {"geolocation": "geolocation"},
        "runtime_modules": ["geolocation"],
    },
    "indexed-db": {
        "description": "IndexedDB glue",
        "interfaces": {"indexed-db": "indexedDb"},
        "runtime_modules": [],
    },
    "media": {
        "description": "Media APIs glue — video, audio, media stream",
        "interfaces": {"media": "media"},
        "runtime_modules": ["media"],
    },
    "misc": {
        "description": "Miscellaneous browser APIs glue",
        "interfaces": {"misc": "misc"},
        "runtime_modules": ["misc"],
    },
    "notifications": {
        "description": "Notifications API glue",
        "interfaces": {"notifications": "notifications"},
        "runtime_modules": ["notifications"],
    },
    "payments": {
        "description": "Payment Request API glue",
        "interfaces": {"payments": "payments"},
        "runtime_modules": ["payments"],
    },
    "performance": {
        "description": "Performance APIs glue",
        "interfaces": {"performance-ext": "performanceExt"},
        "runtime_modules": ["performanceExt"],
    },
    "permissions": {
        "description": "Permissions API glue",
        "interfaces": {"permissions": "permissions"},
        "runtime_modules": ["permissions"],
    },
    "service-workers": {
        "description": "Service Worker glue",
        "interfaces": {"service-workers": "serviceWorkers"},
        "runtime_modules": [],
    },
    "storage": {
        "description": "Web Storage glue",
        "interfaces": {"storage": "storage"},
        "runtime_modules": ["storage"],
    },
    "streams": {
        "description": "Streams API glue",
        "interfaces": {"streams": "streams"},
        "runtime_modules": [],
    },
    "svg": {
        "description": "SVG glue",
        "interfaces": {"svg": "svg"},
        "runtime_modules": ["svg"],
    },
    "url": {
        "description": "URL API glue",
        "interfaces": {"url": "url"},
        "runtime_modules": ["url"],
    },
    "wasm": {
        "description": "WebAssembly glue",
        "interfaces": {"wasm": "wasm"},
        "runtime_modules": ["wasm"],
    },
    "web-animations": {
        "description": "Web Animations API glue",
        "interfaces": {"web-animations": "webAnimations"},
        "runtime_modules": [],
    },
    "webrtc": {
        "description": "WebRTC glue",
        "interfaces": {"webrtc": "webrtc"},
        "runtime_modules": ["webrtc"],
    },
    "websocket": {
        "description": "WebSocket glue",
        "interfaces": {"websocket": "websocket"},
        "runtime_modules": ["websocket"],
    },
    "websockets": {
        "description": "WebSocket streams glue",
        "interfaces": {"websockets": "websockets"},
        "runtime_modules": ["websockets"],
    },
    "workers": {
        "description": "Web Workers glue",
        "interfaces": {"workers": "workers"},
        "runtime_modules": ["workers"],
    },
}

AUTOGEN_DOMAINS = [
    "file-api",
    "indexed-db",
    "service-workers",
    "streams",
    "web-animations",
]


def find_esbuild():
    """Find esbuild binary, trying local node_modules first, then npx."""
    candidates = []
    if os.name == "nt":
        candidates.append(("npx.cmd", ["npx.cmd", "esbuild", "--version"]))
        candidates.append(("esbuild.cmd", ["esbuild.cmd", "--version"]))
    candidates.append(("npx", ["npx", "esbuild", "--version"]))
    candidates.append(("esbuild", ["esbuild", "--version"]))
    for name, cmd in candidates:
        try:
            result = subprocess.run(cmd, capture_output=True, timeout=30)
            if result.returncode == 0:
                if name in ("npx", "npx.cmd"):
                    return [name, "esbuild"]
                return [name]
        except (FileNotFoundError, subprocess.TimeoutExpired):
            continue
    raise RuntimeError("esbuild not found. Install with: npm install -g esbuild")


def generate_browser_glue():
    """Generate the unified browser-glue package with all domains."""
    pkg_dir = NPM_DIR / "browser-glue"
    src_dir = pkg_dir / "src"
    src_dir.mkdir(parents=True, exist_ok=True)

    handles_src = RUNTIME_DIR / "handles.ts"
    helpers_src = RUNTIME_DIR / "helpers.ts"
    async_src = WORKSPACE_ROOT / "packages" / "browser-glue" / "src" / "async.ts"

    core_lines = [
        "// Auto-generated by build_npm_glue_packages.py — DO NOT EDIT",
        "// @ts-nocheck",
        "",
    ]

    if handles_src.exists():
        content = handles_src.read_text(encoding="utf-8").replace("// @ts-nocheck\n", "")
        core_lines.append("// === handles ===")
        core_lines.append(content.strip())
        core_lines.append("")

    if helpers_src.exists():
        content = helpers_src.read_text(encoding="utf-8").replace("// @ts-nocheck\n", "")
        core_lines.append("// === helpers ===")
        core_lines.append(content.strip())
        core_lines.append("")

    if async_src.exists():
        content = async_src.read_text(encoding="utf-8").replace("// @ts-nocheck\n", "")
        core_lines.append("// === async ===")
        core_lines.append(content.strip())
        core_lines.append("")

    (src_dir / "_core.ts").write_text("\n".join(core_lines), encoding="utf-8")

    all_interfaces = {}
    module_count = 0

    for domain_name, domain_info in DOMAIN_MAP.items():
        domain_lines = [
            "// @ts-nocheck",
            f"// Domain: {domain_name}",
            "",
        ]

        runtime_modules = domain_info["runtime_modules"]

        if runtime_modules:
            for module_name in runtime_modules:
                src_file = RUNTIME_DIR / f"{module_name}.ts"
                if not src_file.exists():
                    continue
                content = src_file.read_text(encoding="utf-8").replace("// @ts-nocheck\n", "").strip()
                domain_lines.append(f"// === {module_name} ===")
                domain_lines.append(content)
                domain_lines.append("")
                module_count += 1
        else:
            glue_file = WORKSPACE_ROOT / "packages" / "browser-glue" / "src" / "glue" / f"{domain_name}.ts"
            if glue_file.exists():
                content = glue_file.read_text(encoding="utf-8").replace("// @ts-nocheck\n", "").strip()
                domain_lines.append(content)
                domain_lines.append("")

        iface_entries = []
        for wit_name, module_name in domain_info["interfaces"].items():
            export_name = f"{module_name}_exports"
            iface_entries.append(f'  "@tairitsu-glue/{wit_name}": {export_name},')

        domain_lines.append("// === Registry ===")
        domain_lines.append("export const INTERFACES = {")
        for entry in iface_entries:
            domain_lines.append(entry)
        domain_lines.append("};")
        domain_lines.append("")

        safe_name = domain_name.replace("-", "_")
        (src_dir / f"{safe_name}.ts").write_text("\n".join(domain_lines), encoding="utf-8")
        all_interfaces[domain_name] = safe_name

    index_lines = [
        "// @ts-nocheck",
        "// Unified browser glue — all WIT domain implementations",
        "",
        'import "./_core.js";',
        "",
    ]

    for domain_name, safe_name in all_interfaces.items():
        index_lines.append(f'import {{ INTERFACES as {safe_name} }} from "./{safe_name}.js";')

    index_lines.append("")
    index_lines.append("export const INTERFACES = {")
    for safe_name in all_interfaces.values():
        index_lines.append(f"  ...{safe_name},")
    index_lines.append("};")
    index_lines.append("")

    (src_dir / "index.ts").write_text("\n".join(index_lines), encoding="utf-8")

    pkg_json = {
        "name": f"{SCOPE}/tairitsu-browser-glue",
        "version": VERSION,
        "description": "Tairitsu browser glue — all WIT domain implementations in a single package",
        "license": "MIT OR Apache-2.0",
        "type": "module",
        "sideEffects": False,
        "main": "./dist/index.js",
        "types": "./dist/index.d.ts",
        "exports": {
            ".": {
                "import": "./dist/index.js",
                "types": "./dist/index.d.ts",
            }
        },
        "files": ["dist/**/*.js", "dist/**/*.d.ts"],
        "repository": {
            "type": "git",
            "url": "https://github.com/celestia-island/tairitsu.git",
            "directory": "packages/npm/browser-glue",
        },
        "publishConfig": {
            "access": "public",
            "registry": "https://registry.npmjs.org/",
        },
        "scripts": {
            "build": "esbuild src/index.ts --bundle --outfile=dist/index.js --format=esm --platform=browser --minify",
            "clean": "rimraf dist",
            "prepublishOnly": "npm run build",
        },
        "devDependencies": {
            "esbuild": "^0.25.0",
            "rimraf": "^5.0.0",
        },
    }
    (pkg_dir / "package.json").write_text(
        json.dumps(pkg_json, indent=2) + "\n", encoding="utf-8"
    )

    print(f"  OK browser-glue: {module_count} modules across {len(DOMAIN_MAP)} domains")


def generate_glue_full():
    """Generate the backward-compatible glue-full shim."""
    pkg_dir = NPM_DIR / "glue-full"
    src_dir = pkg_dir / "src"
    src_dir.mkdir(parents=True, exist_ok=True)

    (src_dir / "index.ts").write_text(
        "// @ts-nocheck\n"
        "// Re-export from the unified browser-glue package\n"
        'export { INTERFACES } from "@celestia/tairitsu-browser-glue";\n',
        encoding="utf-8",
    )

    pkg_json = {
        "name": f"{SCOPE}/tairitsu-glue-full",
        "version": VERSION,
        "description": "Tairitsu browser glue — all WIT domain implementations aggregated (deprecated: use @celestia/tairitsu-browser-glue)",
        "deprecated": "Use @celestia/tairitsu-browser-glue instead",
        "license": "MIT OR Apache-2.0",
        "type": "module",
        "sideEffects": False,
        "main": "./dist/index.js",
        "types": "./dist/index.d.ts",
        "exports": {
            ".": {
                "import": "./dist/index.js",
                "types": "./dist/index.d.ts",
            }
        },
        "files": ["dist/**/*.js", "dist/**/*.d.ts"],
        "repository": {
            "type": "git",
            "url": "https://github.com/celestia-island/tairitsu.git",
            "directory": "packages/npm/glue-full",
        },
        "publishConfig": {
            "access": "public",
            "registry": "https://registry.npmjs.org/",
        },
        "dependencies": {
            f"{SCOPE}/tairitsu-browser-glue": f"^{VERSION}",
        },
        "scripts": {
            "build": "esbuild src/index.ts --bundle --outfile=dist/index.js --format=esm --platform=browser --minify",
            "clean": "rimraf dist",
            "prepublishOnly": "npm run build",
        },
        "devDependencies": {
            "esbuild": "^0.25.0",
            "rimraf": "^5.0.0",
        },
    }
    (pkg_dir / "package.json").write_text(
        json.dumps(pkg_json, indent=2) + "\n", encoding="utf-8"
    )
    print(f"  OK glue-full: shim -> @celestia/tairitsu-browser-glue")


def build_all():
    """Run esbuild on all generated packages."""
    npm_packages = sorted(NPM_DIR.iterdir())
    built = 0
    failed = []

    for pkg_dir in npm_packages:
        if not pkg_dir.is_dir():
            continue
        if not (pkg_dir / "package.json").exists():
            continue
        if pkg_dir.name == "runtime":
            continue

        pkg_json = json.loads((pkg_dir / "package.json").read_text(encoding="utf-8"))
        name = pkg_json.get("name", pkg_dir.name)

        if not (pkg_dir / "src" / "index.ts").exists():
            continue

        dist_dir = pkg_dir / "dist"
        dist_dir.mkdir(exist_ok=True)

        src_file = str(pkg_dir / "src" / "index.ts")
        out_file = str(pkg_dir / "dist" / "index.js")

        try:
            esbuild_cmd = find_esbuild()
            result = subprocess.run(
                esbuild_cmd + [
                    src_file,
                    "--bundle",
                    f"--outfile={out_file}",
                    "--format=esm",
                    "--platform=browser",
                    "--minify",
                    "--tree-shaking=true",
                ],
                capture_output=True,
                text=True,
                encoding="utf-8",
                errors="replace",
                cwd=str(pkg_dir),
                timeout=30,
            )
            if result.returncode == 0:
                out_size = (pkg_dir / "dist" / "index.js").stat().st_size
                print(f"  OK {name}: {out_size:,} bytes")
                built += 1
            else:
                print(f"  X {name}: {result.stderr.strip()}")
                failed.append(name)
        except Exception as e:
            print(f"  X {name}: {e}")
            failed.append(name)

    print(f"\n  Built: {built} packages, Failed: {len(failed)}")
    if failed:
        print(f"  Failed: {', '.join(failed)}")


def main():
    args = set(sys.argv[1:])

    if "--build" not in args:
        print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
        print("Generating unified browser-glue npm package...")
        print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")

        print("\n[1/2] Unified browser-glue package:")
        generate_browser_glue()

        print("\n[2/2] Backward-compatible glue-full shim:")
        generate_glue_full()

        print("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
        print(f"Generated unified package with {len(DOMAIN_MAP)} domains")
        print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")

    if "--gen" not in args:
        print("\nBuilding all packages with esbuild (minified)...")
        build_all()


if __name__ == "__main__":
    main()
