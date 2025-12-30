#!/usr/bin/env python3
"""
Project Statistics Generator for Tairitsu

This script analyzes the project structure and provides statistics
about code organization, file counts, and lines of code.
"""

import os
import sys
from pathlib import Path
from collections import defaultdict


def count_lines(file_path):
    """Count lines in a file."""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            return len(f.readlines())
    except Exception:
        return 0


def analyze_project(root_dir):
    """Analyze project structure and gather statistics."""
    stats = {
        'rust': defaultdict(int),
        'wit': defaultdict(int),
        'other': defaultdict(int),
        'total_files': 0,
        'total_lines': 0,
    }
    
    # File extensions to analyze
    extensions = {
        '.rs': 'rust',
        '.wit': 'wit',
        '.toml': 'other',
        '.md': 'other',
    }
    
    root_path = Path(root_dir)
    
    # Skip these directories
    skip_dirs = {'target', '.git', 'node_modules', '__pycache__'}
    
    for path in root_path.rglob('*'):
        # Skip directories in skip list
        if any(skip_dir in path.parts for skip_dir in skip_dirs):
            continue
            
        if path.is_file():
            ext = path.suffix
            if ext in extensions:
                category = extensions[ext]
                lines = count_lines(path)
                
                stats[category]['files'] += 1
                stats[category]['lines'] += lines
                stats['total_files'] += 1
                stats['total_lines'] += lines
    
    return stats


def print_stats(stats):
    """Pretty print statistics."""
    print("\n" + "="*60)
    print(" "*20 + "Tairitsu Project Statistics")
    print("="*60 + "\n")
    
    # Rust files
    if stats['rust']['files'] > 0:
        print(f"📦 Rust Files:")
        print(f"   Files: {stats['rust']['files']}")
        print(f"   Lines: {stats['rust']['lines']:,}")
        print()
    
    # WIT files
    if stats['wit']['files'] > 0:
        print(f"🔗 WIT Interface Files:")
        print(f"   Files: {stats['wit']['files']}")
        print(f"   Lines: {stats['wit']['lines']:,}")
        print()
    
    # Other files
    if stats['other']['files'] > 0:
        print(f"📄 Other Files (TOML, MD):")
        print(f"   Files: {stats['other']['files']}")
        print(f"   Lines: {stats['other']['lines']:,}")
        print()
    
    # Total
    print(f"📊 Total:")
    print(f"   Files: {stats['total_files']}")
    print(f"   Lines: {stats['total_lines']:,}")
    print("\n" + "="*60 + "\n")


def main():
    """Main entry point."""
    # Get repository root (script is in scripts/ dir)
    script_dir = Path(__file__).parent
    repo_root = script_dir.parent
    
    print(f"Analyzing project at: {repo_root}")
    
    stats = analyze_project(repo_root)
    print_stats(stats)
    
    return 0


if __name__ == '__main__':
    sys.exit(main())
