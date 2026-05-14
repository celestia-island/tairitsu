#!/usr/bin/env python3
"""
Build CDN-mode demo for tairitsu website.

Creates a cdn-index.html that uses static import maps and CDN-loaded glue packages
instead of the monolithic IIFE bundle. Also copies modular wasm components.

Usage:
    python scripts/build_cdn_demo.py [--dist DIR] [--cdn-mode local|esm-sh]
"""

import argparse
import json
import os
import shutil
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
NPM_DIR = REPO_ROOT / "packages" / "npm"
DEFAULT_DIST = REPO_ROOT / "target" / "tairitsu-dist"

CDN_SHIMS_DIR = "cdn-shims"
CDN_MODULES_DIR = "cdn-modules"

GLUE_SPECIFIER_MAP = {
    "@tairitsu-glue/document": {
        "package": "browser-glue",
        "export_obj": "document_exports",
        "functions": ["createElement", "createTextNode", "getBody"],
    },
    "@tairitsu-glue/element": {
        "package": "browser-glue",
        "export_obj": "element_exports",
        "functions": [
            "setAttribute", "getBoundingClientRect", "getClassList",
            "getClientHeight", "getScrollHeight", "getScrollTop",
            "setInnerHtml", "setScrollTop", "getAttribute", "getTagName",
            "removeAttribute",
        ],
    },
    "@tairitsu-glue/node": {
        "package": "browser-glue",
        "export_obj": "node_exports",
        "functions": [
            "appendChild", "getFirstChild", "removeChild", "setTextContent",
            "getTextContent", "getParentElement",
        ],
    },
    "@tairitsu-glue/non-element-parent-node": {
        "package": "browser-glue",
        "export_obj": "nonElementParentNode_exports",
        "functions": ["getElementById"],
    },
    "@tairitsu-glue/parent-node": {
        "package": "browser-glue",
        "export_obj": "parentNode_exports",
        "functions": ["querySelector", "querySelectorAll"],
    },
    "@tairitsu-glue/event": {
        "package": "browser-glue",
        "export_obj": "event_exports",
        "functions": [
            "getCurrentTarget", "getTarget", "getEventType",
            "getSrcElement", "getEventPhase", "eventStopPropagation",
            "stopImmediatePropagation", "getBubbles", "getCancelable",
            "getDefaultPrevented", "getTimeStamp", "getIsTrusted",
            "getCancelBubble", "setCancelBubble", "composedPath",
        ],
    },
    "@tairitsu-glue/event-target": {
        "package": "browser-glue",
        "export_obj": "eventTarget_exports",
        "functions": ["addEventListener", "removeEventListener", "preventDefault", "stopPropagation"],
    },
    "@tairitsu-glue/css-style-declaration": {
        "package": "browser-glue",
        "export_obj": "cssStyleDeclaration_exports",
        "functions": [
            "getCssText", "setCssText", "getLength", "item",
            "getPropertyValue", "getPropertyPriority", "setProperty",
            "removeProperty", "getParentRule",
        ],
    },
    "@tairitsu-glue/element-css-inline-style": {
        "package": "browser-glue",
        "export_obj": "elementCssInlineStyle_exports",
        "functions": ["getStyle"],
    },
    "@tairitsu-glue/dom-token-list": {
        "package": "browser-glue",
        "export_obj": "domTokenList_exports",
        "functions": ["add", "remove", "contains"],
    },
    "@tairitsu-glue/platform-helpers": {
        "package": "browser-glue",
        "export_obj": "platformHelpers_exports",
        "functions": [
            "setTimeout", "clearTimeout", "requestAnimationFrame",
            "cancelAnimationFrame", "innerWidth", "innerHeight",
        ],
    },
    "@tairitsu-glue/window": {
        "package": "browser-glue",
        "export_obj": "window_exports",
        "functions": ["getComputedStyle", "getInnerWidth", "getInnerHeight"],
    },
}

PACKAGE_TO_CDN_NAME = {
    "browser-glue": "@celestia/tairitsu-browser-glue",
    "glue-full": "@celestia/tairitsu-glue-full",
    "runtime": "@celestia/tairitsu-runtime",
    "tairitsu-vdom-wasm": "@celestia/tairitsu-vdom-wasm",
    "tairitsu-hooks-wasm": "@celestia/tairitsu-hooks-wasm",
    "tairitsu-style-wasm": "@celestia/tairitsu-style-wasm",
    "hikari-palette-wasm": "@celestia/hikari-palette-wasm",
}

VERSION = "0.5.0"


def build_glue_source_modules(glue_pkg_dir: Path) -> dict[str, str]:
    """Read the source index.ts from a glue package and extract export objects."""
    src_file = glue_pkg_dir / "src" / "index.ts"
    if not src_file.exists():
        return {}
    return {"src": src_file.read_text(encoding="utf-8")}


def generate_per_specifier_modules(dist_dir: Path) -> dict[str, str]:
    """
    Generate per-specifier re-export ES modules.
    Each module re-exports individual functions from the parent glue package's _exports object.
    Returns a dict mapping specifier -> local module path.
    """
    shims_dir = dist_dir / CDN_SHIMS_DIR
    shims_dir.mkdir(parents=True, exist_ok=True)

    # Create a pure IIFE version of glue-core (no ES module exports)
    # that just initializes globalThis handle tables and helpers.
    core_dir = shims_dir / "glue-core"
    core_dir.mkdir(parents=True, exist_ok=True)
    core_src = NPM_DIR / "browser-glue" / "dist" / "index.js"
    if core_src.exists():
        core_code = core_src.read_text(encoding="utf-8")
        # Strip ES module exports: remove everything from the last 'export{' to end
        export_idx = core_code.rfind("export{")
        if export_idx != -1:
            core_iife = core_code[:export_idx].rstrip().rstrip(";")
        else:
            core_iife = core_code
        # Wrap in IIFE and add __wasmExports / __setWasmExports setup
        iife_code = (
            "(function(){\n"
            f"{core_iife}\n"
            "globalThis.__wasmExports = globalThis.__wasmExports || null;\n"
            "globalThis.__setWasmExports = globalThis.__setWasmExports || function(exports) {\n"
            "  globalThis.__wasmExports = exports;\n"
            "};\n"
            "})();\n"
        )
        (core_dir / "index.js").write_text(iife_code, encoding="utf-8")

    specifier_to_path = {}
    
    for specifier, info in GLUE_SPECIFIER_MAP.items():
        pkg_name = info["package"]
        export_obj = info["export_obj"]
        functions = info["functions"]
        
        # Create a safe filename from the specifier
        safe_name = specifier.replace("@tairitsu-glue/", "").replace("/", "-")
        module_file = shims_dir / f"{safe_name}.js"
        
        # Read the source to inline the actual function implementations
        pkg_src_file = NPM_DIR / pkg_name / "src" / "index.ts"
        
        # Instead of trying to parse TS, we'll build the shim to import from
        # a local copy of the glue package
        pkg_local_dir = shims_dir / pkg_name
        pkg_local_dir.mkdir(parents=True, exist_ok=True)
        
        # Build the re-export module
        # For local mode: import from relative local package
        re_exports = ", ".join(functions)
        
        # We need to generate a module that exports the individual functions.
        # Strategy: import the _exports object and destructure it.
        module_code = (
            f"// Auto-generated re-export shim for {specifier}\n"
            f"import {{ {export_obj} }} from './{pkg_name}/index.js';\n"
        )
        for fn in functions:
            module_code += f"export const {fn} = {export_obj}.{fn};\n"
        
        module_file.write_text(module_code, encoding="utf-8")
        specifier_to_path[specifier] = f"/{CDN_SHIMS_DIR}/{safe_name}.js"
    
    return specifier_to_path


def copy_glue_packages_to_shims(dist_dir: Path):
    """Copy built glue package dists to the shims directory for local serving."""
    packages_needed = set()
    for info in GLUE_SPECIFIER_MAP.values():
        packages_needed.add(info["package"])
    
    shims_dir = dist_dir / CDN_SHIMS_DIR
    
    for pkg_name in packages_needed:
        pkg_dist = NPM_DIR / pkg_name / "dist"
        pkg_src = NPM_DIR / pkg_name / "src"
        target_dir = shims_dir / pkg_name
        
        if target_dir.exists():
            shutil.rmtree(target_dir)
        target_dir.mkdir(parents=True, exist_ok=True)
        
        # Check if there's a built dist, otherwise use source directly
        if pkg_dist.exists():
            for f in pkg_dist.iterdir():
                shutil.copy2(f, target_dir / f.name)
        elif pkg_src.exists():
            for f in pkg_src.iterdir():
                if f.suffix in ('.ts', '.js'):
                    # Convert .ts to .js extension
                    target_name = f.stem + ".js"
                    shutil.copy2(f, target_dir / target_name)


def copy_modular_wasm_packages(dist_dir: Path):
    """Copy modular wasm component packages to the dist directory."""
    modules_dir = dist_dir / CDN_MODULES_DIR
    modules_dir.mkdir(parents=True, exist_ok=True)
    
    wasm_packages = [
        "tairitsu-vdom-wasm",
        "tairitsu-hooks-wasm",
        "tairitsu-style-wasm",
        "hikari-palette-wasm",
    ]
    
    copied = []
    for pkg_name in wasm_packages:
        pkg_dir = NPM_DIR / pkg_name
        if not pkg_dir.exists():
            print(f"  Warning: {pkg_name} not found, skipping")
            continue
        
        target_dir = modules_dir / pkg_name
        if target_dir.exists():
            shutil.rmtree(target_dir)
        
        # Copy the entire dist directory
        dist_src = pkg_dir / "dist"
        if dist_src.exists():
            shutil.copytree(dist_src, target_dir)
            # Count files
            wasm_files = list(target_dir.rglob("*.wasm"))
            js_files = list(target_dir.rglob("*.js"))
            total_size = sum(f.stat().st_size for f in target_dir.rglob("*") if f.is_file())
            print(f"  {pkg_name}: {len(wasm_files)} wasm, {len(js_files)} js, {total_size // 1024}KB")
            copied.append(pkg_name)
    
    return copied


def build_import_map(specifier_map: dict[str, str], cdn_mode: str) -> dict[str, str]:
    """Build the import map entries."""
    imports = {}
    
    if cdn_mode == "esm-sh":
        # For esm.sh mode, point to CDN URLs
        for specifier, info in GLUE_SPECIFIER_MAP.items():
            pkg = info["package"]
            cdn_name = PACKAGE_TO_CDN_NAME.get(pkg, pkg)
            imports[specifier] = f"https://esm.sh/{cdn_name}@{VERSION}"
    else:
        # For local mode, use the generated shim modules
        imports.update(specifier_map)
    
    # Always add WASI preview2-shim CDN URLs (these are needed by jco wrappers)
    imports["@bytecodealliance/preview2-shim/cli"] = "https://esm.sh/@bytecodealliance/preview2-shim/cli"
    imports["@bytecodealliance/preview2-shim/filesystem"] = "https://esm.sh/@bytecodealliance/preview2-shim/filesystem"
    imports["@bytecodealliance/preview2-shim/io"] = "https://esm.sh/@bytecodealliance/preview2-shim/io"
    imports["@bytecodealliance/preview2-shim/random"] = "https://esm.sh/@bytecodealliance/preview2-shim/random"
    
    return imports


def generate_cdn_html(dist_dir: Path, import_map: dict, cdn_mode: str):
    """Generate cdn-index.html from the existing index.html, replacing the boot sequence."""
    source_html = dist_dir / "index.html"
    if not source_html.exists():
        print(f"Error: {source_html} not found. Run 'tairitsu build' first.")
        sys.exit(1)
    
    html = source_html.read_text(encoding="utf-8")
    
    # Find the boot script section
    # The current HTML has:
    # 1. <script src="/browser-glue/__tairitsu_glue__.js?v=..."></script>
    # 2. <script type="module"> ... (big boot script) ... </script>
    
    import_map_json = json.dumps({"imports": import_map}, indent=2)
    
    # Build the CDN boot HTML by replacing the boot section
    # Strategy: replace everything from the glue script tag to the end of the module script
    
    # Find the comment + glue script section
    comment_marker = '    <!-- Import map (WASI preview2-shim + tairitsu-glue interfaces)'
    glue_marker = '<script src="/browser-glue/__tairitsu_glue__.js'
    module_end_marker = '</script>\n</body>'
    
    # Start from the comment if found, otherwise from the glue script
    comment_start = html.find(comment_marker)
    glue_start = html.find(glue_marker)
    replace_start = comment_start if comment_start != -1 else glue_start
    
    module_end = html.rfind(module_end_marker)
    
    if replace_start == -1 or module_end == -1:
        print("Error: Could not find boot script markers in index.html")
        sys.exit(1)
    
    head_part = html[:replace_start]
    # tail_part starts after </body> tag
    
    # Build CDN boot script
    v = int(__import__('time').time())
    
    # Read the existing module script to extract post-boot utilities
    # (fixSvgNamespaces, waitForDomSettle, glow effect, scrollbar, etc.)
    existing_module_start = html.find('<script type="module">\n        import { instantiateWithWrapper }', glue_start)
    if existing_module_start == -1:
        existing_module_start = html.find('<script type="module">', glue_start)
    
    # Extract post-boot code (everything after the wasm instantiation)
    # Look for the stylesReady function and everything after
    post_boot_marker = "const stylesReady = ()"
    post_boot_idx = html.find(post_boot_marker, existing_module_start)
    
    post_boot_code = ""
    if post_boot_idx != -1:
        # Find the end of the module script
        module_close = html.rfind('</script>', post_boot_idx, module_end + len('</script>'))
        if module_close != -1:
            # Extract from stylesReady to end of script
            raw_post = html[post_boot_idx:module_close]
            # Dedent by 8 spaces (the module script indentation)
            lines = raw_post.split('\n')
            dedented = []
            for line in lines:
                if line.startswith('        '):
                    dedented.append(line[8:])
                else:
                    dedented.append(line)
            post_boot_code = '\n'.join(dedented)
    
    cdn_boot = f'''    <!-- CDN Boot Mode: Static import map + modular glue packages -->
    <script type="importmap">
    {import_map_json}
    </script>
    <script src="/{CDN_SHIMS_DIR}/glue-core/index.js"></script>
    <script type="module">
        // CDN Boot: glue-core handles are already initialized by the IIFE above.
        // The import map resolves @tairitsu-glue/* to local CDN shim modules,
        // which re-export individual functions from per-domain glue packages.

        // --- Boot utilities ---
        const appRoot = document.getElementById('app');
        const setAppStatus = (text) => {{
            if (!appRoot) return;
            const current = (appRoot.textContent || '').trim();
            if (current === 'Loading...') {{
                appRoot.textContent = text;
            }}
        }};

        const clearLoadingIfUnchanged = () => {{
            if (!appRoot) return;
            const current = (appRoot.textContent || '').trim();
            if (current === 'Loading...') {{
                appRoot.textContent = '';
            }}
        }};

        const tryInvokeBootExports = async (result) => {{
            const normalizeBootName = (name) => {{
                const lowered = String(name || '').toLowerCase();
                if (lowered === 'run') return 'run';
                if (lowered === 'main') return 'main';
                if (lowered === 'init') return 'init';
                if (lowered === 'start') return 'start';
                return null;
            }};

            const seenObjects = new Set();
            const seenFunctions = new Set();
            const discovered = [];

            const collect = (obj, depth = 0) => {{
                if (!obj || typeof obj !== 'object' || depth > 3) return;
                if (seenObjects.has(obj)) return;
                seenObjects.add(obj);

                for (const [name, value] of Object.entries(obj)) {{
                    if (typeof value !== 'function') continue;
                    const kind = normalizeBootName(name);
                    if (!kind) continue;
                    if (seenFunctions.has(value)) continue;
                    seenFunctions.add(value);
                    discovered.push({{ kind, fn: value }});
                }}

                for (const [, value] of Object.entries(obj)) {{
                    if (value && typeof value === 'object') {{
                        collect(value, depth + 1);
                    }}
                }}
            }};

            const targets = [
                result,
                result && result.instance,
                result && result.exports,
                result && result.instance && result.instance.exports,
            ];

            for (const target of targets) {{
                collect(target);
                if (target && target.exports) collect(target.exports);
            }}

            let invoked = false;

            for (const preferred of ['run', 'main', 'init']) {{
                for (const entry of discovered) {{
                    if (entry.kind !== preferred) continue;
                    await entry.fn();
                    invoked = true;
                }}
            }}

            if (!invoked) {{
                const fallbackStart = discovered.find((entry) => entry.kind === 'start');
                if (fallbackStart) {{
                    await fallbackStart.fn();
                    invoked = true;
                }}
            }}

            return invoked;
        }};

        const buildImports = () => {{
            const imports = {{}};
            if (globalThis.__TAIRITSU_GLUE__ && globalThis.__TAIRITSU_GLUE__.INTERFACES) {{
                for (const [shortName, exports] of Object.entries(globalThis.__TAIRITSU_GLUE__.INTERFACES)) {{
                    const ifaceName = shortName.replace('@tairitsu-glue/', '');
                    const fullName = `tairitsu-browser:full/${{ifaceName}}@0.2.0`;
                    imports[fullName] = exports;
                }}
            }}
            return imports;
        }};

        // --- Register glue interfaces from CDN modules ---
        // Import the unified browser-glue package
        const glueModule = await import('/{CDN_SHIMS_DIR}/browser-glue/index.js');

        // Merge all INTERFACES into global registry
        globalThis.__TAIRITSU_GLUE__ = {{
            INTERFACES: {{
                ...glueModule.INTERFACES,
            }},
        }};

        // --- Load modular wasm components (pre-cache) ---
        const MODULAR_COMPONENTS = {{
            'tairitsu-vdom': '/{CDN_MODULES_DIR}/tairitsu-vdom-wasm/wrapper/tairitsu_vdom.js',
            'tairitsu-hooks': '/{CDN_MODULES_DIR}/tairitsu-hooks-wasm/wrapper/tairitsu_hooks.js',
            'tairitsu-style': '/{CDN_MODULES_DIR}/tairitsu-style-wasm/wrapper/tairitsu_style.js',
            'hikari-palette': '/{CDN_MODULES_DIR}/hikari-palette-wasm/wrapper/hikari_palette.js',
        }};

        // Pre-load modular components in parallel (non-blocking)
        const componentPromises = Object.entries(MODULAR_COMPONENTS).map(async ([name, url]) => {{
            try {{
                const mod = await import(url);
                console.log(`[cdn] Pre-loaded component: ${{name}}`);
                return {{ name, mod }};
            }} catch (err) {{
                console.warn(`[cdn] Failed to pre-load component ${{name}}:`, err);
                return {{ name, err }};
            }}
        }});

        // --- Load and boot main application wasm ---
        let bootInvoked = false;
        const CANDIDATES = [
            './component-wrapper/tairitsu_website.js',
            './component-wrapper/index.js',
        ];

        async function tryLoadWrapper(imports) {{
            for (const path of CANDIDATES) {{
                try {{
                    const mod = await import(path);
                    const instantiate = mod.instantiate || mod.default || mod.init;
                    if (typeof instantiate === 'function') {{
                        try {{
                            return await instantiate(imports);
                        }} catch (_e1) {{ /* ignore first attempt */ }}
                        try {{
                            return await instantiate(async (modulePath) => {{
                                const resolved = new URL(modulePath, import.meta.url);
                                const response = await fetch(resolved);
                                if (!response.ok) {{
                                    throw new Error(`Failed to fetch core module: ${{modulePath}}`);
                                }}
                                return WebAssembly.compileStreaming(response);
                            }}, imports);
                        }} catch (_e2) {{ /* ignore second attempt */ }}
                    }}
                    return mod;
                }} catch (error) {{
                    console.warn(`[cdn] Wrapper candidate ${{path}} failed:`, error);
                }}
            }}
            return null;
        }}

        try {{
            const wrapperResult = await tryLoadWrapper(buildImports());
            if (wrapperResult) {{
                if (globalThis.__setWasmExports && wrapperResult) {{
                    globalThis.__setWasmExports(wrapperResult);
                }}
                bootInvoked = await tryInvokeBootExports(wrapperResult);
            }} else {{
                // Fallback: direct wasm load
                const response = await fetch('/tairitsu_website.wasm');
                const bytes = await response.arrayBuffer();
                const module = await WebAssembly.compile(bytes);
                const moduleResult = await WebAssembly.instantiate(module, buildImports());
                if (globalThis.__setWasmExports && moduleResult) {{
                    globalThis.__setWasmExports(moduleResult);
                }}
                bootInvoked = await tryInvokeBootExports(moduleResult);
            }}
        }} catch (err) {{
            console.error('[cdn] Boot failed:', err);
            setAppStatus('Failed to load: ' + (err.message || err));
        }}

        if (!bootInvoked) {{
            setAppStatus('Component initialized (no exported run/start entry found).');
        }} else {{
            clearLoadingIfUnchanged();
        }}

        // Wait for component pre-loads to settle (non-blocking)
        Promise.allSettled(componentPromises).then(results => {{
            const loaded = results.filter(r => r.status === 'fulfilled' && !r.value.err).length;
            console.log(`[cdn] ${{loaded}}/${{Object.keys(MODULAR_COMPONENTS).length}} modular components pre-loaded`);
        }});

        {post_boot_code}
    </script>'''

    new_html = head_part + cdn_boot + '\n</body>\n</html>'
    
    output_path = dist_dir / "cdn-index.html"
    output_path.write_text(new_html, encoding="utf-8")
    print(f"  Generated: {output_path.relative_to(dist_dir)}")
    return output_path


def main():
    parser = argparse.ArgumentParser(description="Build CDN-mode demo")
    parser.add_argument("--dist", default=str(DEFAULT_DIST), help="Output directory")
    parser.add_argument(
        "--cdn-mode",
        choices=["local", "esm-sh"],
        default="local",
        help="CDN mode: local (served from dist) or esm-sh (from esm.sh CDN)",
    )
    args = parser.parse_args()

    dist_dir = Path(args.dist)
    if not dist_dir.exists():
        print(f"Error: {dist_dir} does not exist. Run 'tairitsu build' first.")
        sys.exit(1)

    print(f"[cdn-demo] Building CDN demo in {dist_dir} (mode: {args.cdn_mode})")

    # Step 1: Generate per-specifier re-export modules
    print("\n[1/4] Generating per-specifier re-export modules...")
    specifier_map = generate_per_specifier_modules(dist_dir)
    print(f"  Generated {len(specifier_map)} shim modules")

    # Step 2: Copy glue packages to shims directory
    print("\n[2/4] Copying glue packages...")
    copy_glue_packages_to_shims(dist_dir)
    
    # Step 3: Copy modular wasm packages
    print("\n[3/4] Copying modular wasm packages...")
    wasm_pkgs = copy_modular_wasm_packages(dist_dir)
    print(f"  Copied {len(wasm_pkgs)} wasm packages")

    # Step 4: Generate CDN HTML
    print("\n[4/4] Generating cdn-index.html...")
    import_map = build_import_map(specifier_map, args.cdn_mode)
    output = generate_cdn_html(dist_dir, import_map, args.cdn_mode)

    # Summary
    print(f"\n[cdn-demo] Done!")
    print(f"  Output: {output}")
    print(f"  CDN shims: {dist_dir / CDN_SHIMS_DIR}")
    print(f"  Modular components: {dist_dir / CDN_MODULES_DIR}")
    print(f"\n  To test:")
    print(f"    cd {dist_dir}")
    print(f"    python -m http.server 3002")
    print(f"    Open http://localhost:3002/cdn-index.html")


if __name__ == "__main__":
    main()
