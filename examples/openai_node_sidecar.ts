/**
 * Tokenectomy AI Gateway — OpenAI Node.js / TypeScript SDK Drop-in Example
 * Demonstrates zero-cost prompt caching, secret redaction, and 429 resiliency.
 *
 * Prerequisites:
 *   1. Start the Tokenectomy Gateway:
 *      npx -y tokenectomy-razor --proxy
 *   2. Run with: npx tsx examples/openai_node_sidecar.ts
 */

import OpenAI from "openai";

// Simply point baseURL to the local Tokenectomy Gateway (127.0.0.1:8080/v1)
const openai = new OpenAI({
  baseURL: "http://127.0.0.1:8080/v1",
  apiKey: process.env.OPENAI_API_KEY || "sk-placeholder",
});

async function main() {
  const prompt = "Review this crash: failed at postgresql://admin:secret_pass@db.internal:5432/main";

  console.log("--- Request 1: Fresh Request (Cache MISS + Secret Redaction) ---");
  const t0 = performance.now();
  const res1 = await openai.chat.completions.create({
    model: "gpt-4o-mini",
    messages: [{ role: "user", content: prompt }],
  });
  console.log(`Latency: ${(performance.now() - t0).toFixed(2)}ms`);
  console.log(`Response: ${res1.choices[0].message.content?.slice(0, 100)}...\n`);

  console.log("--- Request 2: Identical Request (Zero-Cost Cache HIT) ---");
  const t1 = performance.now();
  const res2 = await openai.chat.completions.create({
    model: "gpt-4o-mini",
    messages: [{ role: "user", content: prompt }],
  });
  console.log(`Latency: ${(performance.now() - t1).toFixed(2)}ms (Served locally in <1ms)`);
  console.log("Observe live FinOps metrics at: http://127.0.0.1:8080/dashboard");
}

main().catch(console.error);
