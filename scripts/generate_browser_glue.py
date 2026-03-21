#!/usr/bin/env python3
"""
Parse generated WIT files and produce TypeScript glue code.

Reads from  : packages/browser-worlds/wit/generated/*.wit
Writes to   : packages/browser-glue/src/generated/*-glue.ts

This generator creates TypeScript modules that implement WIT import
interfaces by bridging to actual browser DOM/Web APIs.

Usage:
    python3 scripts/generate_browser_glue.py              # generate all domains
    python3 scripts/generate_browser_glue.py --domains storage url  # specific domains
    python3 scripts/generate_browser_glue.py --dry-run    # preview without writing
    python3 scripts/generate_browser_glue.py --stats      # show coverage stats
"""

import argparse
from pathlib import Path
from typing import List, Optional, Tuple

from generator import (
    log_info, log_ok, log_warn, log_error,
    WitParser, CodeGenerator,
    GeneratedDomain,
)


def find_wit_files(wit_dir: Path) -> List[Tuple[str, Path]]:
    """Find all WIT files in the generated directory."""
    files = []
    for wit_file in sorted(wit_dir.glob("*.wit")):
        domain = wit_file.stem
        files.append((domain, wit_file))
    return files


def run_generate(
    wit_dir: Path,
    output_dir: Path,
    domains: Optional[List[str]] = None,
    *,
    dry_run: bool = False,
    stats: bool = False,
) -> None:
    """Parse WIT files and generate TypeScript glue code."""

    log_info(f"WIT source : {wit_dir}")
    log_info(f"Output     : {output_dir}")
    print()

    wit_files = find_wit_files(wit_dir)

    if not wit_files:
        log_error(f"No WIT files found in {wit_dir}")
        return

    if domains:
        wit_files = [(d, p) for d, p in wit_files if d in domains]
        if not wit_files:
            log_error(f"No WIT files found for domains: {domains}")
            return

    parser = WitParser()
    code_gen = CodeGenerator()
    generated_domains: List[GeneratedDomain] = []

    if stats:
        log_info("WIT → TypeScript Glue Coverage Statistics")
        print("=" * 60)

    total_interfaces = 0
    total_functions = 0

    for domain, wit_path in wit_files:
        gen_domain = parser.generate_domain(domain, wit_path)
        if gen_domain:
            generated_domains.append(gen_domain)
            total_interfaces += gen_domain.interface_count
            total_functions += sum(len(i.functions) for i in gen_domain.interfaces)

            if stats:
                func_count = sum(len(i.functions) for i in gen_domain.interfaces)
                print(f"  {domain:<20} {gen_domain.interface_count:3d} interfaces, "
                      f"{func_count:3d} functions")

    if stats:
        print("=" * 60)
        print(f"  {'TOTAL':<20} {total_interfaces:3d} interfaces, "
              f"{total_functions:3d} functions")
        print()
        return

    if not dry_run:
        output_dir.mkdir(parents=True, exist_ok=True)

    for gen_domain in generated_domains:
        wit_path = wit_dir / f"{gen_domain.name}.wit"
        content = code_gen.render_module(gen_domain, str(wit_path))
        dest = output_dir / f"{gen_domain.export_name}Glue.ts"

        if dry_run:
            log_info(f"dry-run: {dest.name} ({len(content):,} bytes)")
        else:
            dest.write_text(content, encoding="utf-8")
            func_count = sum(len(i.functions) for i in gen_domain.interfaces)
            log_ok(f"Wrote {dest.name:<25} {gen_domain.interface_count:2d} interfaces, "
                   f"{func_count:3d} functions")

    if generated_domains:
        index_content = code_gen.render_index(generated_domains)
        index_dest = output_dir / "index.ts"

        if dry_run:
            log_info(f"dry-run: {index_dest.name} ({len(index_content):,} bytes)")
        else:
            index_dest.write_text(index_content, encoding="utf-8")
            log_ok(f"Wrote {index_dest.name}")

    if not dry_run:
        print()
        log_info(f"Result: {len(generated_domains)} domains generated")


def main() -> None:
    parser = argparse.ArgumentParser(
        description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print what would be written without actually writing files",
    )
    parser.add_argument(
        "--stats",
        action="store_true",
        help="Show coverage statistics and exit",
    )
    parser.add_argument(
        "--domains",
        nargs="+",
        metavar="DOMAIN",
        help="Generate only specific domain(s)",
    )
    parser.add_argument(
        "--wit-dir",
        metavar="DIR",
        help="Override WIT source directory",
    )
    parser.add_argument(
        "--output-dir",
        metavar="DIR",
        help="Override TypeScript output directory",
    )
    args = parser.parse_args()

    project_root = Path(__file__).parent.parent

    wit_dir = (
        Path(args.wit_dir)
        if args.wit_dir
        else project_root / "packages" / "browser-worlds" / "wit" / "generated"
    )
    output_dir = (
        Path(args.output_dir)
        if args.output_dir
        else project_root / "packages" / "browser-glue" / "src"
    )

    print("=" * 64)
    log_info("Tairitsu WIT -> TypeScript Glue Generator")
    print("=" * 64)
    if args.dry_run:
        log_info("Mode: dry run (no files written)")

    run_generate(
        wit_dir,
        output_dir,
        domains=args.domains,
        dry_run=args.dry_run,
        stats=args.stats,
    )


if __name__ == "__main__":
    main()
