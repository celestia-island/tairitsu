# Tairitsu Automation Scripts

This directory contains Python scripts for project automation and analysis.

## Scripts

### project_stats.py

Analyzes the project structure and provides statistics about code organization.

**Usage:**
```bash
python3 scripts/project_stats.py
```

or via just:
```bash
just stats
```

**Output:**
- Number of Rust files and lines of code
- Number of WIT interface files
- Number of configuration files (TOML, Markdown)
- Total project statistics

## Requirements

- Python 3.6 or later (standard library only, no external dependencies)

## Adding New Scripts

When adding new automation scripts to this directory:

1. Make the script executable: `chmod +x scripts/your_script.py`
2. Add a shebang line: `#!/usr/bin/env python3`
3. Document the script in this README
4. Add a recipe to the justfile if appropriate
