#!/usr/bin/env python3
"""
Scorer for Tokenectomy Redaction Benchmark Suite.
Calculates Precision, Recall, and F1-Score per secret category (JWT, AWS key, DB connection string, etc.)
comparing Tokenectomy Razor vs Gitleaks.
Outputs structured scoring metrics to corpus/score_report.json.
"""

import os
import sys
import json
from collections import defaultdict
from typing import Dict, Any, List

BASE_DIR = os.path.dirname(os.path.abspath(__file__))
CORPUS_DIR = os.path.join(BASE_DIR, "corpus")
RUNNER_RESULTS_JSON = os.path.join(CORPUS_DIR, "runner_results.json")
SCORE_REPORT_JSON = os.path.join(CORPUS_DIR, "score_report.json")

def calculate_metrics(tp: int, fp: int, fn: int) -> Dict[str, float]:
    precision = tp / (tp + fp) if (tp + fp) > 0 else (1.0 if fn == 0 else 0.0)
    recall = tp / (tp + fn) if (tp + fn) > 0 else 1.0
    f1 = (2 * precision * recall) / (precision + recall) if (precision + recall) > 0 else 0.0
    return {
        "precision": round(precision * 100.0, 2),
        "recall": round(recall * 100.0, 2),
        "f1": round(f1 * 100.0, 2)
    }

def score():
    if not os.path.exists(RUNNER_RESULTS_JSON):
        print(f"[ERROR] Runner results not found at: {RUNNER_RESULTS_JSON}")
        print("Please execute runner.py first.")
        sys.exit(1)

    with open(RUNNER_RESULTS_JSON, "r", encoding="utf-8") as f:
        data = json.load(f)

    results = data.get("results", [])

    # Groupings by category
    razor_by_cat = defaultdict(lambda: {"tp": 0, "fn": 0, "fp": 0, "total": 0})
    gitleaks_by_cat = defaultdict(lambda: {"tp": 0, "fn": 0, "fp": 0, "total": 0})

    razor_overall = {"tp": 0, "fn": 0, "fp": 0, "total": 0}
    gitleaks_overall = {"tp": 0, "fn": 0, "fp": 0, "total": 0}

    # Track negative control results (clean logs)
    negative_controls = []

    for item in results:
        ground_truth = item.get("ground_truth", [])
        razor_res = item["razor"]
        gl_res = item["gitleaks"]

        if len(ground_truth) == 0:
            # Negative control
            negative_controls.append({
                "id": item["id"],
                "filename": item["filename"],
                "language": item["language"],
                "razor_fp": razor_res.get("false_positives", 0),
                "gitleaks_fp": gl_res.get("false_positives", 0)
            })
            razor_overall["fp"] += razor_res.get("false_positives", 0)
            gitleaks_overall["fp"] += gl_res.get("false_positives", 0)
            continue

        # Evaluate Ground Truth per category
        for gt in ground_truth:
            cat = gt["type"]
            raw = gt["raw"]

            razor_by_cat[cat]["total"] += 1
            gitleaks_by_cat[cat]["total"] += 1
            razor_overall["total"] += 1
            gitleaks_overall["total"] += 1

            # Razor hit check
            is_razor_hit = any(h["raw"] == raw for h in razor_res.get("hits", []))
            if is_razor_hit:
                razor_by_cat[cat]["tp"] += 1
                razor_overall["tp"] += 1
            else:
                razor_by_cat[cat]["fn"] += 1
                razor_overall["fn"] += 1

            # Gitleaks hit check
            is_gl_hit = any(h["raw"] == raw for h in gl_res.get("hits", []))
            if is_gl_hit:
                gitleaks_by_cat[cat]["tp"] += 1
                gitleaks_overall["tp"] += 1
            else:
                gitleaks_by_cat[cat]["fn"] += 1
                gitleaks_overall["fn"] += 1

        # Gitleaks false positives in non-empty samples
        gl_fp = gl_res.get("false_positives", 0)
        gitleaks_overall["fp"] += gl_fp

    # Build category breakdown
    category_scores = {}
    all_categories = sorted(list(set(list(razor_by_cat.keys()) + list(gitleaks_by_cat.keys()))))

    for cat in all_categories:
        r_stats = razor_by_cat[cat]
        gl_stats = gitleaks_by_cat[cat]

        r_metrics = calculate_metrics(r_stats["tp"], r_stats["fp"], r_stats["fn"])
        gl_metrics = calculate_metrics(gl_stats["tp"], gl_stats["fp"], gl_stats["fn"])

        category_scores[cat] = {
            "total_ground_truth": r_stats["total"],
            "razor": {
                "tp": r_stats["tp"],
                "fn": r_stats["fn"],
                "fp": r_stats["fp"],
                "precision": r_metrics["precision"],
                "recall": r_metrics["recall"],
                "f1": r_metrics["f1"]
            },
            "gitleaks": {
                "tp": gl_stats["tp"],
                "fn": gl_stats["fn"],
                "fp": gl_stats["fp"],
                "precision": gl_metrics["precision"],
                "recall": gl_metrics["recall"],
                "f1": gl_metrics["f1"]
            }
        }

    # Calculate overall metrics
    r_overall_metrics = calculate_metrics(razor_overall["tp"], razor_overall["fp"], razor_overall["fn"])
    gl_overall_metrics = calculate_metrics(gitleaks_overall["tp"], gitleaks_overall["fp"], gitleaks_overall["fn"])

    score_report = {
        "summary": {
            "total_samples": len(results),
            "total_ground_truth_secrets": razor_overall["total"],
            "negative_control_samples": len(negative_controls),
            "razor_overall": {
                "tp": razor_overall["tp"],
                "fn": razor_overall["fn"],
                "fp": razor_overall["fp"],
                "precision": r_overall_metrics["precision"],
                "recall": r_overall_metrics["recall"],
                "f1": r_overall_metrics["f1"]
            },
            "gitleaks_overall": {
                "tp": gitleaks_overall["tp"],
                "fn": gitleaks_overall["fn"],
                "fp": gitleaks_overall["fp"],
                "precision": gl_overall_metrics["precision"],
                "recall": gl_overall_metrics["recall"],
                "f1": gl_overall_metrics["f1"]
            }
        },
        "category_breakdown": category_scores,
        "negative_controls": negative_controls
    }

    with open(SCORE_REPORT_JSON, "w", encoding="utf-8") as f:
        json.dump(score_report, f, indent=2)

    print(f"Scoring complete. Report written to: {SCORE_REPORT_JSON}")
    print("\n--- Summary Performance ---")
    print(f"Total Secrets in Corpus: {razor_overall['total']}")
    print(f"Razor Overall:    Recall: {r_overall_metrics['recall']}% | Precision: {r_overall_metrics['precision']}% | F1: {r_overall_metrics['f1']}% (TP: {razor_overall['tp']}, FN: {razor_overall['fn']}, FP: {razor_overall['fp']})")
    print(f"Gitleaks Overall: Recall: {gl_overall_metrics['recall']}% | Precision: {gl_overall_metrics['precision']}% | F1: {gl_overall_metrics['f1']}% (TP: {gitleaks_overall['tp']}, FN: {gitleaks_overall['fn']}, FP: {gitleaks_overall['fp']})")

if __name__ == "__main__":
    score()
