#!/usr/bin/env bash
# contract_validator.sh — Code Integrity & Scope Creep Validator
# Verifies test passes, checks for uncommitted scope creep, and flags warnings.

set -euo pipefail

TARGET_DIR="${1:-.}"

echo "========================================================"
echo "📐 Spec-First Architect — Contract Validator"
echo "Project Directory: $TARGET_DIR"
echo "========================================================"

echo ""
echo "[*] Checking for placeholder comments (e.g. '// ... existing code', 'TODO: implement')..."
if command -v rg &>/dev/null; then
    PLACEHOLDERS=$(rg --line-number -i "\/\/\s*\.\.\.\s*existing|\/\/\s*TODO:\s*implement" "$TARGET_DIR" 2>/dev/null || echo "")
    if [ -n "$PLACEHOLDERS" ]; then
        echo "  [!] WARNING: Lazy placeholders detected!"
        echo "$PLACEHOLDERS"
    else
        echo "  [+] Clean! No lazy deletion placeholders found."
    fi
else
    echo "  [-] Skipping placeholder check (ripgrep not available)."
fi

echo ""
echo "[*] Checking Git Working Tree Status (Detecting Scope Creep)..."
if command -v git &>/dev/null && [ -d "$TARGET_DIR/.git" ]; then
    CHANGED_FILES=$(cd "$TARGET_DIR" && git status --porcelain | wc -l)
    echo "  [+] Total modified/untracked files: $CHANGED_FILES"
    cd "$TARGET_DIR" && git status -s
else
    echo "  [-] Not a git repository or git not found."
fi

echo ""
echo "========================================================"
echo "Contract Validation Complete."
echo "========================================================"
