#!/usr/bin/env python3
"""Validate agent is working in correct viewyard context"""

import json
import os
import sys
from pathlib import Path


def validate_view_context(target_path: str = ".") -> dict:
    """Validate current working directory has proper view context"""
    
    # Look for .viewyard-context file
    context_file = Path(target_path) / ".viewyard-context"
    
    if not context_file.exists():
        return {
            "valid": False,
            "error": "No .viewyard-context file found",
            "suggestion": "Are you in a viewyard view directory?"
        }
    
    try:
        with open(context_file) as f:
            context = json.load(f)
    except Exception as e:
        return {
            "valid": False,
            "error": f"Failed to read context file: {e}",
            "suggestion": "Context file may be corrupted"
        }
    
    # Validate we're in the correct directory or subdirectory
    current_path = Path(target_path).resolve()
    expected_path = Path(context["view_root"]).resolve()

    # Allow current path to be the view root or any subdirectory within it
    try:
        current_path.relative_to(expected_path)
    except ValueError:
        return {
            "valid": False,
            "error": f"Working directory mismatch",
            "current": str(current_path),
            "expected": str(expected_path),
            "suggestion": f"cd {context['view_root']}"
        }
    
    return {
        "valid": True,
        "view_name": context["view_name"],
        "view_root": context["view_root"],
        "active_repos": context["active_repos"],
        "allowed_paths": context["workspace_boundary"]["allowed_paths"],
        "forbidden_paths": context["workspace_boundary"]["forbidden_paths"]
    }


def main():
    """CLI interface for context validation"""
    target = sys.argv[1] if len(sys.argv) > 1 else "."
    result = validate_view_context(target)
    
    if result["valid"]:
        print(f"✓ Valid viewyard context: {result['view_name']}")
        print(f"  Root: {result['view_root']}")
        print(f"  Repos: {', '.join(result['active_repos'])}")
    else:
        print(f"✗ Invalid context: {result['error']}")
        if "suggestion" in result:
            print(f"  Suggestion: {result['suggestion']}")
        sys.exit(1)


if __name__ == "__main__":
    main()
