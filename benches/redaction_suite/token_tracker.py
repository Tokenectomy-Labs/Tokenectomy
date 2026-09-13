#!/usr/bin/env python3
"""
Token Reduction Tracker for Tokenectomy Redaction Benchmark Suite.
Measures before/after token counts per sample using OpenAI tiktoken (cl100k_base),
reporting mean, median (P50), P90, min, max, and full token distribution.
Outputs structured metrics to corpus/token_reduction_report.json.
"""

import os
import sys
import json
import statistics
from typing import Dict, Any, List

BASE_DIR = os.path.dirname(os.path.abspath(__file__))
CORPUS_DIR = os.path.join(BASE_DIR, "corpus")
RUNNER_RESULTS_JSON = os.path.join(CORPUS_DIR, "runner_results.json")
DATASET_JSON = os.path.join(CORPUS_DIR, "dataset.json")
TOKEN_REPORT_JSON = os.path.join(CORPUS_DIR, "token_reduction_report.json")

def track_tokens():
    try:
        import tiktoken
    except ImportError:
        print("[ERROR] tiktoken is not installed in the current environment.")
        print("Please run: pip install tiktoken")
        sys.exit(1)

    if not os.path.exists(RUNNER_RESULTS_JSON):
        print(f"[ERROR] Runner results not found at: {RUNNER_RESULTS_JSON}")
        print("Please run runner.py first.")
        sys.exit(1)

    with open(DATASET_JSON, "r", encoding="utf-8") as f:
        dataset_list = json.load(f)
    dataset_map = {item["id"]: item for item in dataset_list}

    with open(RUNNER_RESULTS_JSON, "r", encoding="utf-8") as f:
        runner_data = json.load(f)

    results = runner_data.get("results", [])
    encoding = tiktoken.get_encoding("cl100k_base")

    per_sample_stats = []
    reduction_percentages = []

    total_raw_tokens = 0
    total_scrubbed_tokens = 0

    for item in results:
        sample_id = item["id"]
        raw_text = dataset_map[sample_id]["raw_text"]
        scrubbed_text = item["razor"]["scrubbed_text"]

        raw_tokens = len(encoding.encode(raw_text))
        scrubbed_tokens = len(encoding.encode(scrubbed_text))
        tokens_saved = raw_tokens - scrubbed_tokens

        pct = (tokens_saved / raw_tokens * 100.0) if raw_tokens > 0 else 0.0

        total_raw_tokens += raw_tokens
        total_scrubbed_tokens += scrubbed_tokens
        reduction_percentages.append(pct)

        per_sample_stats.append({
            "id": sample_id,
            "filename": item["filename"],
            "language": item["language"],
            "category": item["category"],
            "raw_tokens": raw_tokens,
            "scrubbed_tokens": scrubbed_tokens,
            "tokens_saved": tokens_saved,
            "reduction_pct": round(pct, 2),
            "ground_truth_count": item["ground_truth_count"]
        })

    # Statistical distribution
    reduction_percentages_sorted = sorted(reduction_percentages)
    mean_pct = statistics.mean(reduction_percentages)
    median_pct = statistics.median(reduction_percentages)
    min_pct = min(reduction_percentages)
    max_pct = max(reduction_percentages)

    # 90th percentile
    p90_idx = int(0.90 * len(reduction_percentages_sorted))
    p90_pct = reduction_percentages_sorted[min(p90_idx, len(reduction_percentages_sorted) - 1)]

    overall_reduction_pct = ((total_raw_tokens - total_scrubbed_tokens) / total_raw_tokens * 100.0) if total_raw_tokens > 0 else 0.0

    report = {
        "tokenizer": "cl100k_base (tiktoken)",
        "samples_evaluated": len(per_sample_stats),
        "aggregate": {
            "total_raw_tokens": total_raw_tokens,
            "total_scrubbed_tokens": total_scrubbed_tokens,
            "total_tokens_saved": total_raw_tokens - total_scrubbed_tokens,
            "overall_reduction_pct": round(overall_reduction_pct, 2)
        },
        "distribution": {
            "mean_pct": round(mean_pct, 2),
            "median_p50_pct": round(median_pct, 2),
            "p90_pct": round(p90_pct, 2),
            "min_pct": round(min_pct, 2),
            "max_pct": round(max_pct, 2)
        },
        "samples": per_sample_stats
    }

    with open(TOKEN_REPORT_JSON, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)

    print(f"Token reduction tracking complete. Report written to: {TOKEN_REPORT_JSON}")
    print("\n--- Token Reduction Distribution ---")
    print(f"Total Raw Tokens:      {total_raw_tokens}")
    print(f"Total Scrubbed Tokens: {total_scrubbed_tokens}")
    print(f"Total Tokens Saved:    {total_raw_tokens - total_scrubbed_tokens} ({overall_reduction_pct:.2f}% aggregate reduction)")
    print(f"Mean Reduction:        {mean_pct:.2f}%")
    print(f"Median (P50):          {median_pct:.2f}%")
    print(f"90th Percentile (P90): {p90_pct:.2f}%")
    print(f"Range [Min - Max]:     [{min_pct:.2f}% - {max_pct:.2f}%]")

if __name__ == "__main__":
    track_tokens()
