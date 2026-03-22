#!/usr/bin/env python3
"""
Generate interface-level wrapper files for jco transpiled imports.

jco generates imports like:
    import { body, createElement } from 'tairitsu-browser:full/document';

This script generates minimal wrapper files for the interfaces used by Rust code.
"""

import re
from pathlib import Path
from typing import Dict, List, Optional


# Interfaces used by Rust code (from wit_platform.rs)
USED_INTERFACES = {
    "document": [
        "create-element",
        "create-text-node", 
        "get-body",
    ],
    "non-element-parent-node": [
        "get-element-by-id",
    ],
    "element": [
        "set-attribute",
        "remove-attribute",
    ],
    "node": [
        "append-child",
        "remove-child",
        "set-text-content",
        "get-text-content",
    ],
    "window": [
        "get-inner-width",
        "get-inner-height",
    ],
    "console": [
        "log",
        "warn",
        "error",
    ],
    "style": [
        "set-style-property",
        "get-style-property",
        "remove-style-property",
    ],
    "event-target": [
        "add-event-listener",
        "remove-event-listener",
        "prevent-default",
        "stop-propagation",
    ],
}


def kebab_to_camel(name: str) -> str:
    """Convert kebab-case to camelCase."""
    parts = name.split("-")
    return parts[0] + "".join(p.capitalize() for p in parts[1:])


def find_function_in_glue(func_name: str, glue_files: List[Path]) -> Optional[tuple]:
    """Find which glue file contains a function and return (glue_file, ts_name)."""
    ts_name = kebab_to_camel(func_name)
    
    for glue_file in sorted(glue_files):
        content = glue_file.read_text(encoding="utf-8")
        # Look for export function declaration in JS
        if re.search(rf"export\s+function\s+{ts_name}\s*\(", content):
            return (glue_file.stem, ts_name)
    
    return None


def generate_wrappers(output_dir: Path, glue_dist_dir: Path) -> None:
    """Generate interface wrapper files."""
    
    glue_files = list(glue_dist_dir.glob("*Glue.js"))
    wrapper_dir = output_dir / "browser-glue"
    wrapper_dir.mkdir(parents=True, exist_ok=True)
    
    print(f"Generating wrappers for {len(USED_INTERFACES)} interfaces...")
    print(f"Searching in {len(glue_files)} glue files...")
    
    for iface_name, functions in USED_INTERFACES.items():
        domain_imports: Dict[str, List[str]] = {}
        
        for func in functions:
            result = find_function_in_glue(func, glue_files)
            if result:
                glue_file, ts_name = result
                if glue_file not in domain_imports:
                    domain_imports[glue_file] = []
                if ts_name not in domain_imports[glue_file]:
                    domain_imports[glue_file].append(ts_name)
        
        if not domain_imports:
            print(f"  Skipping {iface_name} - no functions found")
            continue
        
        # Generate wrapper content (JS for browser)
        import_lines = []
        for domain in sorted(domain_imports.keys()):
            names = sorted(set(domain_imports[domain]))
            import_lines.append(f"import {{ {', '.join(names)} }} from './{domain}.js';")
        
        export_lines = []
        for func in functions:
            ts_name = kebab_to_camel(func)
            result = find_function_in_glue(func, glue_files)
            if result:
                export_lines.append(f"export {{ {ts_name} }};")
            else:
                export_lines.append(f"// export {{ {ts_name} }}; // TODO: not found")
        
        wrapper_content = f"""// Auto-generated wrapper for {iface_name} interface
// DO NOT EDIT MANUALLY

{chr(10).join(import_lines)}
{chr(10).join(export_lines)}
"""
        
        wrapper_path = wrapper_dir / f"{iface_name}.js"
        wrapper_path.write_text(wrapper_content, encoding="utf-8")
        print(f"  Generated {iface_name}.js ({len(functions)} functions)")


def main():
    project_root = Path(__file__).parent.parent
    output_dir = project_root / "packages" / "browser-glue" / "dist"
    glue_dist_dir = project_root / "packages" / "browser-glue" / "dist"
    
    print("=" * 64)
    print("Interface Wrapper Generator")
    print("=" * 64)
    
    generate_wrappers(output_dir, glue_dist_dir)


if __name__ == "__main__":
    main()
