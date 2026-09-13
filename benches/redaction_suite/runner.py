#!/usr/bin/env python3
"""
Dual Runner for Tokenectomy Redaction Benchmark Suite.
Executes both `target/release/razor --scrub` and `gitleaks detect` on the synthetic corpus,
captures raw scrubbed outputs and findings, and measures sub-millisecond execution times.
"""

import os
import sys
import json
import time
import subprocess
from typing import Dict, Any, List

BASE_DIR = os.path.dirname(os.path.abspath(__file__))
REPO_ROOT = os.path.abspath(os.path.join(BASE_DIR, "..", ".."))
CORPUS_DIR = os.path.join(BASE_DIR, "corpus")
SAMPLES_DIR = os.path.join(CORPUS_DIR, "samples")
DATASET_JSON = os.path.join(CORPUS_DIR, "dataset.json")
RUNNER_RESULTS_JSON = os.path.join(CORPUS_DIR, "runner_results.json")
GITLEAKS_REPORT_JSON = os.path.join(CORPUS_DIR, "gitleaks_report.json")

RAZOR_BIN = os.path.join(REPO_ROOT, "target", "release", "razor")
GITLEAKS_BIN = os.path.join(BASE_DIR, "bin", "gitleaks")

def check_binaries():
    if not os.path.exists(RAZOR_BIN):
        print(f"[ERROR] Razor binary not found at: {RAZOR_BIN}")
        print("Please build it first: cargo build --release --bin razor")
        sys.exit(1)
    if not os.path.exists(GITLEAKS_BIN):
        print(f"[ERROR] Gitleaks binary not found at: {GITLEAKS_BIN}")
        print("Please ensure gitleaks binary is placed in benches/redaction_suite/bin/gitleaks")
        sys.exit(1)
    if not os.path.exists(DATASET_JSON):
        print(f"[ERROR] Dataset JSON not found at: {DATASET_JSON}")
        print("Please run gen_corpus.py first.")
        sys.exit(1)

def run_gitleaks_batch() -> Dict[str, List[Dict[str, Any]]]:
    """
    Runs gitleaks across all samples in one batch for high accuracy,
    and returns a mapping of filename -> list of findings.
    """
    cmd = [
        GITLEAKS_BIN,
        "detect",
        "--no-git",
        "--source", SAMPLES_DIR,
        "--report-format", "json",
        "--report-path", GITLEAKS_REPORT_JSON,
        "--exit-code", "0"
    ]
    t0 = time.perf_counter()
    res = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    duration_ms = (time.perf_counter() - t0) * 1000.0

    findings_by_file: Dict[str, List[Dict[str, Any]]] = {}
    if os.path.exists(GITLEAKS_REPORT_JSON):
        with open(GITLEAKS_REPORT_JSON, "r", encoding="utf-8") as f:
            try:
                findings = json.load(f)
                for item in findings:
                    filepath = item.get("File", "")
                    fname = os.path.basename(filepath)
                    findings_by_file.setdefault(fname, []).append(item)
            except json.JSONDecodeError:
                pass

    return findings_by_file, duration_ms

def run_razor_on_sample(raw_text: str) -> (str, float):
    """
    Runs razor --scrub on raw_text via stdin, returns (scrubbed_text, latency_ms).
    """
    cmd = [RAZOR_BIN, "--scrub"]
    t0 = time.perf_counter()
    proc = subprocess.run(
        cmd,
        input=raw_text,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )
    latency_ms = (time.perf_counter() - t0) * 1000.0
    if proc.returncode != 0:
        print(f"[WARN] razor --scrub returned non-zero code: {proc.returncode}")
        print(proc.stderr)
    return proc.stdout, latency_ms

def run_gitleaks_on_single_sample(sample_path: str) -> float:
    """
    Measures single-file execution time for Gitleaks.
    """
    cmd = [
        GITLEAKS_BIN,
        "detect",
        "--no-git",
        "--source", sample_path,
        "--exit-code", "0"
    ]
    t0 = time.perf_counter()
    subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    return (time.perf_counter() - t0) * 1000.0

def evaluate_sample(sample: Dict[str, Any], gitleaks_findings: List[Dict[str, Any]]) -> Dict[str, Any]:
    raw_text = sample["raw_text"]
    ground_truth = sample.get("ground_truth", [])

    # 1. Run Razor
    scrubbed_text, razor_ms = run_razor_on_sample(raw_text)

    # Evaluate Razor against Ground Truth
    razor_hits = []
    razor_misses = []
    for gt in ground_truth:
        secret = gt["raw"]
        sec_type = gt["type"]
        # If the raw secret is no longer in scrubbed text, it was redacted
        if secret not in scrubbed_text:
            razor_hits.append({
                "type": sec_type,
                "raw": secret,
                "status": "REDACTED"
            })
        else:
            razor_misses.append({
                "type": sec_type,
                "raw": secret,
                "status": "MISSED"
            })

    # Check for False Positives in Razor (especially on negative controls)
    razor_fp = 0
    if len(ground_truth) == 0:
        # Negative control sample: should NOT contain redaction markers
        markers = ["[REDACTED]", "_REDACTED]"]
        if any(m in scrubbed_text for m in markers):
            razor_fp = 1

    # 2. Evaluate Gitleaks against Ground Truth
    gitleaks_hits = []
    gitleaks_misses = []
    matched_gitleaks_indices = set()

    for gt in ground_truth:
        secret = gt["raw"]
        sec_type = gt["type"]
        found = False
        for idx, gl in enumerate(gitleaks_findings):
            gl_secret = gl.get("Secret", "")
            gl_match = gl.get("Match", "")
            # Check if secret matches or is contained in gitleaks detection
            if gl_secret and (gl_secret in secret or secret in gl_secret):
                found = True
                matched_gitleaks_indices.add(idx)
                break
            elif gl_match and (secret in gl_match or gl_match in secret):
                found = True
                matched_gitleaks_indices.add(idx)
                break
        if found:
            gitleaks_hits.append({
                "type": sec_type,
                "raw": secret,
                "status": "DETECTED"
            })
        else:
            gitleaks_misses.append({
                "type": sec_type,
                "raw": secret,
                "status": "MISSED"
            })

    gitleaks_fp = 0
    # Any gitleaks findings not matching any ground truth
    unmatched_gl = len(gitleaks_findings) - len(matched_gitleaks_indices)
    if len(ground_truth) == 0:
        gitleaks_fp = len(gitleaks_findings)
    elif unmatched_gl > 0:
        gitleaks_fp = unmatched_gl

    return {
        "id": sample["id"],
        "filename": sample["filename"],
        "language": sample["language"],
        "category": sample["category"],
        "description": sample["description"],
        "ground_truth_count": len(ground_truth),
        "ground_truth": ground_truth,
        "razor": {
            "latency_ms": razor_ms,
            "scrubbed_text": scrubbed_text,
            "hits": razor_hits,
            "misses": razor_misses,
            "false_positives": razor_fp
        },
        "gitleaks": {
            "findings_count": len(gitleaks_findings),
            "raw_findings": gitleaks_findings,
            "hits": gitleaks_hits,
            "misses": gitleaks_misses,
            "false_positives": gitleaks_fp
        }
    }

def main():
    check_binaries()
    print("=== Tokenectomy Dual Runner (Razor vs Gitleaks) ===")
    with open(DATASET_JSON, "r", encoding="utf-8") as f:
        dataset = json.load(f)

    print(f"Loaded {len(dataset)} samples from {DATASET_JSON}")
    print("Executing Gitleaks batch scan on corpus...")
    gitleaks_batch_findings, total_gl_time = run_gitleaks_batch()
    print(f"Gitleaks batch scan completed in {total_gl_time:.2f} ms")

    results = []
    total_razor_time = 0.0

    print("Running dual evaluation per sample...")
    for item in dataset:
        fname = item["filename"]
        sample_path = os.path.join(SAMPLES_DIR, fname)
        gl_findings = gitleaks_batch_findings.get(fname, [])

        # Measure individual sample gitleaks time on a subset or estimate
        evaluated = evaluate_sample(item, gl_findings)
        total_razor_time += evaluated["razor"]["latency_ms"]
        results.append(evaluated)

    output_payload = {
        "timestamp": time.time(),
        "total_samples": len(results),
        "total_razor_time_ms": total_razor_time,
        "total_gitleaks_time_ms": total_gl_time,
        "results": results
    }

    with open(RUNNER_RESULTS_JSON, "w", encoding="utf-8") as f:
        json.dump(output_payload, f, indent=2)

    print(f"Dual evaluation complete. Results written to: {RUNNER_RESULTS_JSON}")
    print(f"  Razor Total Time: {total_razor_time:.3f} ms (Avg: {total_razor_time / len(results):.3f} ms/sample)")
    print(f"  Gitleaks Batch Time: {total_gl_time:.2f} ms")

if __name__ == "__main__":
    main()
