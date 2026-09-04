#!/usr/bin/env bash
# fuzz_scanner.sh — Static Hazard & Attack Surface Scanner
# Scans source code for dangerous patterns prone to crashing or exploits.

set -euo pipefail

TARGET_DIR="${1:-.}"

echo "========================================================"
echo "🩸 Adversary Bug Hunter — Static Hazard Scanner"
echo "Target Directory: $TARGET_DIR"
echo "========================================================"

echo ""
echo "[*] Scanning for unchecked panics and unwraps..."
if command -v rg &>/dev/null; then
    rg --line-number --color=always "\.unwrap\(|\.expect\(|panic!\(" "$TARGET_DIR" || echo "  [+] No unwraps/panics detected."
else
    grep -rnE "\.unwrap\(|\.expect\(|panic!\(" "$TARGET_DIR" || echo "  [+] No unwraps/panics detected."
fi

echo ""
echo "[*] Scanning for potential ReDoS patterns (nested quantifiers)..."
if command -v rg &>/dev/null; then
    rg --line-number --color=always "\(\.\*\)\+|\(\.\+\)\+|\(\.\*\)\*|\(\[^\"]\*\)\*" "$TARGET_DIR" || echo "  [+] No obvious nested quantifiers found."
else
    grep -rnE "\(\.\*\)\+|\(\.\+\)\+|\(\.\*\)\*|\(\[^\"]\*\)\*" "$TARGET_DIR" || echo "  [+] No obvious nested quantifiers found."
fi

echo ""
echo "[*] Scanning for raw shell executions and command injections..."
if command -v rg &>/dev/null; then
    rg --line-number --color=always "Command::new\(\"sh\"|Command::new\(\"bash\"|child_process\.exec|os\.system|subprocess\.Popen\(.*shell=True" "$TARGET_DIR" || echo "  [+] No raw shell spawning detected."
else
    grep -rnE "Command::new\(\"sh\"|Command::new\(\"bash\"|child_process\.exec|os\.system|subprocess\.Popen\(.*shell=True" "$TARGET_DIR" || echo "  [+] No raw shell spawning detected."
fi

echo ""
echo "[*] Scanning for unbounded stdin or buffer reads..."
if command -v rg &>/dev/null; then
    rg --line-number --color=always "read_to_end|read_to_string|read_line" "$TARGET_DIR" | grep -v "\.take(" || echo "  [+] All reads appear bounded by .take() or none found."
else
    grep -rnE "read_to_end|read_to_string|read_line" "$TARGET_DIR" || echo "  [+] No unbounded reads detected."
fi

echo ""
echo "========================================================"
echo "Scan Complete. Target these locations for exploit test construction."
echo "========================================================"
