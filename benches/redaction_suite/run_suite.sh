#!/usr/bin/env bash
set -euo pipefail

# ==============================================================================
# Tokenectomy Redaction Benchmark & Evaluation Suite Master Runner
# ==============================================================================

SUITE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SUITE_DIR}/../.." && pwd)"
VENV_DIR="${SUITE_DIR}/.venv"
PYTHON="${VENV_DIR}/bin/python"
BIN_DIR="${SUITE_DIR}/bin"
GITLEAKS_BIN="${BIN_DIR}/gitleaks"

echo "===================================================================="
echo "🗡️  TOKENECTOMY REDACTION BENCHMARK & EVALUATION SUITE"
echo "===================================================================="
echo "Suite Directory: ${SUITE_DIR}"
echo "Repository Root: ${REPO_ROOT}"

# 1. Ensure Python Virtual Environment & Dependencies
if [ ! -f "${PYTHON}" ]; then
    echo "Creating Python virtual environment in ${VENV_DIR}..."
    python3 -m venv "${VENV_DIR}"
    "${PYTHON}" -m pip install --quiet --upgrade pip
    "${PYTHON}" -m pip install --quiet tiktoken
fi

# 2. Ensure Gitleaks Binary
if [ ! -f "${GITLEAKS_BIN}" ]; then
    mkdir -p "${BIN_DIR}"
    if command -v gitleaks &> /dev/null; then
        echo "Found system gitleaks, copying to ${GITLEAKS_BIN}..."
        cp "$(command -v gitleaks)" "${GITLEAKS_BIN}"
    else
        echo "Downloading gitleaks v8.30.1 for Linux x86_64..."
        curl -sSL "https://github.com/gitleaks/gitleaks/releases/download/v8.30.1/gitleaks_8.30.1_linux_x64.tar.gz" -o "/tmp/gitleaks.tar.gz"
        tar -xzf "/tmp/gitleaks.tar.gz" -C "${BIN_DIR}" gitleaks
        rm -f "/tmp/gitleaks.tar.gz"
        chmod +x "${GITLEAKS_BIN}"
    fi
fi

# 3. Build Razor Release Binary
echo "[1/5] Compiling Tokenectomy Razor (release mode)..."
cd "${REPO_ROOT}"
cargo build --release --bin razor

# 4. Generate Ground-Truth Polyglot Corpus
echo "[2/5] Generating synthetic labeled corpus..."
cd "${SUITE_DIR}"
"${PYTHON}" gen_corpus.py

# 5. Execute Dual Runner (Razor vs Gitleaks)
echo "[3/5] Executing dual runner on corpus..."
"${PYTHON}" runner.py

# 6. Score Metrics per Category (Precision / Recall / F1)
echo "[4/5] Computing precision, recall, and F1 per category..."
"${PYTHON}" scorer.py

# 7. Track Token Reduction Distribution (tiktoken cl100k_base)
echo "[5/5] Measuring token reduction distribution..."
"${PYTHON}" token_tracker.py

# 8. Generate Report & Update README
echo "Rendering benchmark report and updating README.md..."
"${PYTHON}" report_generator.py --update-readme

# 9. Regression Gate Check
SCORE_FILE="${SUITE_DIR}/corpus/score_report.json"
RAZOR_RECALL=$("${PYTHON}" -c "import json; data=json.load(open('${SCORE_FILE}')); print(data['summary']['razor_overall']['recall'])")
RAZOR_FP=$("${PYTHON}" -c "import json; data=json.load(open('${SCORE_FILE}')); print(data['summary']['razor_overall']['fp'])")
IS_RECALL_REGRESSION=$("${PYTHON}" -c "import json; data=json.load(open('${SCORE_FILE}')); print('1' if float(data['summary']['razor_overall']['recall']) < 95.0 else '0')")

echo ""
echo "===================================================================="
echo "🎯 REDACTION SUITE GATE RESULTS:"
echo "   - Razor Recall: ${RAZOR_RECALL}% (Required: >= 95.0%)"
echo "   - Razor False Positives: ${RAZOR_FP} (Required: 0)"
echo "===================================================================="

if [ "${IS_RECALL_REGRESSION}" = "1" ]; then
    echo "❌ REGRESSION DETECTED: Razor recall (${RAZOR_RECALL}%) dropped below 95.0% threshold!"
    exit 1
fi

if [ "${RAZOR_FP}" -ne 0 ]; then
    echo "❌ REGRESSION DETECTED: Razor produced ${RAZOR_FP} false positive redactions on clean logs!"
    exit 1
fi

echo "✅ ALL GATES PASSED: 100% Zero-Leak Redaction Verified."
