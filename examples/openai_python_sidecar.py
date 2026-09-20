"""
Tokenectomy AI Gateway — OpenAI Python SDK Drop-in Example
Demonstrates zero-cost prompt caching, secret redaction, and 429 resiliency
using the official OpenAI Python SDK.

Prerequisites:
    1. Start the Tokenectomy Gateway:
       npx -y tokenectomy-razor --proxy
    2. Set your OPENAI_API_KEY environment variable.
"""

import os
import time
from openai import OpenAI

# Point base_url to the local Tokenectomy Gateway (127.0.0.1:8080/v1)
client = OpenAI(
    base_url="http://127.0.0.1:8080/v1",
    api_key=os.environ.get("OPENAI_API_KEY", "sk-placeholder-if-routing-local"),
)

prompt = "Analyze this error trace: connection failed at postgresql://admin:super_secret_password@db.prod.internal:5432/primary"

print("--- Request 1: Fresh Request (Cache MISS + In-Flight Secret Redaction) ---")
start = time.time()
resp1 = client.chat.completions.create(
    model="gpt-4o-mini",
    messages=[{"role": "user", "content": prompt}],
)
elapsed1 = (time.time() - start) * 1000
print(f"Latency: {elapsed1:.2f}ms")
print(f"Response preview: {resp1.choices[0].message.content[:100]}...\n")

print("--- Request 2: Identical Request (Zero-Cost Cache HIT) ---")
start = time.time()
resp2 = client.chat.completions.create(
    model="gpt-4o-mini",
    messages=[{"role": "user", "content": prompt}],
)
elapsed2 = (time.time() - start) * 1000
print(f"Latency: {elapsed2:.2f}ms (Served from local Rust cache without consuming API credits)")
print(f"Speedup: {elapsed1 / max(elapsed2, 0.1):.1f}x faster\n")

print("Observe live FinOps token savings & redacted credentials at: http://127.0.0.1:8080/dashboard")
