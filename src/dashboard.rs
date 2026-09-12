// src/dashboard.rs — Enterprise FinOps & Token Economics Dashboard for Tokenectomy Razor

/// Returns the embedded self-contained HTML/CSS/JS FinOps Dashboard.
/// Runs completely offline with zero external network dependencies (no external CDNs).
pub fn render_dashboard_html() -> &'static str {
    DASHBOARD_HTML
}

const DASHBOARD_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Tokenectomy Razor — FinOps & AI Gateway Dashboard</title>
  <style>
    :root {
      --bg: #09090b;
      --card-bg: #121215;
      --card-border: #27272a;
      --card-hover: #18181b;
      --text-main: #f4f4f5;
      --text-muted: #a1a1aa;
      --accent-green: #10b981;
      --accent-green-bg: rgba(16, 185, 129, 0.1);
      --accent-cyan: #06b6d4;
      --accent-cyan-bg: rgba(6, 182, 212, 0.1);
      --accent-purple: #8b5cf6;
      --accent-purple-bg: rgba(139, 92, 246, 0.1);
      --accent-yellow: #f59e0b;
      --font-sans: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
      --font-mono: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    }

    * { box-sizing: border-box; margin: 0; padding: 0; }

    body {
      background-color: var(--bg);
      color: var(--text-main);
      font-family: var(--font-sans);
      min-height: 100vh;
      padding: 24px;
      line-height: 1.5;
    }

    .container {
      max-width: 1100px;
      margin: 0 auto;
    }

    /* Top Navigation / Header */
    header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding-bottom: 24px;
      border-bottom: 1px solid var(--card-border);
      margin-bottom: 32px;
      flex-wrap: wrap;
      gap: 16px;
    }

    .brand {
      display: flex;
      align-items: center;
      gap: 12px;
    }

    .brand-icon {
      font-size: 28px;
    }

    .brand-title {
      font-size: 20px;
      font-weight: 700;
      letter-spacing: -0.02em;
    }

    .brand-badge {
      font-size: 11px;
      font-family: var(--font-mono);
      background: #27272a;
      color: #d4d4d8;
      padding: 2px 8px;
      border-radius: 9999px;
      font-weight: 600;
    }

    .status-pill {
      display: flex;
      align-items: center;
      gap: 8px;
      background: var(--accent-green-bg);
      border: 1px solid rgba(16, 185, 129, 0.2);
      color: var(--accent-green);
      padding: 6px 14px;
      border-radius: 9999px;
      font-size: 12px;
      font-weight: 600;
      font-family: var(--font-mono);
    }

    .pulse {
      width: 8px;
      height: 8px;
      border-radius: 50%;
      background: var(--accent-green);
      box-shadow: 0 0 0 0 rgba(16, 185, 129, 0.7);
      animation: pulse 2s infinite;
    }

    @keyframes pulse {
      0% { box-shadow: 0 0 0 0 rgba(16, 185, 129, 0.7); }
      70% { box-shadow: 0 0 0 6px rgba(16, 185, 129, 0); }
      100% { box-shadow: 0 0 0 0 rgba(16, 185, 129, 0); }
    }

    /* KPI Grid */
    .kpi-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
      gap: 16px;
      margin-bottom: 32px;
    }

    .kpi-card {
      background: var(--card-bg);
      border: 1px solid var(--card-border);
      border-radius: 12px;
      padding: 20px;
      transition: border-color 0.2s;
    }

    .kpi-card:hover {
      border-color: #3f3f46;
    }

    .kpi-label {
      font-size: 12px;
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.05em;
      color: var(--text-muted);
      margin-bottom: 8px;
      display: flex;
      align-items: center;
      justify-content: space-between;
    }

    .kpi-value {
      font-size: 32px;
      font-weight: 700;
      font-family: var(--font-mono);
      letter-spacing: -0.03em;
      font-variant-numeric: tabular-nums;
    }

    .kpi-value.green { color: var(--accent-green); }
    .kpi-value.cyan { color: var(--accent-cyan); }
    .kpi-value.purple { color: var(--accent-purple); }
    .kpi-value.yellow { color: var(--accent-yellow); }

    .kpi-subtext {
      font-size: 12px;
      color: var(--text-muted);
      margin-top: 6px;
    }

    /* Comparison / Pruning Progress */
    .section-card {
      background: var(--card-bg);
      border: 1px solid var(--card-border);
      border-radius: 12px;
      padding: 24px;
      margin-bottom: 32px;
    }

    .section-title {
      font-size: 16px;
      font-weight: 600;
      margin-bottom: 16px;
      display: flex;
      align-items: center;
      gap: 8px;
    }

    .progress-bar-wrap {
      width: 100%;
      height: 28px;
      background: #1c1917;
      border-radius: 8px;
      overflow: hidden;
      display: flex;
      margin-bottom: 12px;
      border: 1px solid #292524;
    }

    .bar-sanitized {
      background: var(--accent-cyan);
      height: 100%;
      transition: width 0.4s ease;
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 11px;
      font-weight: 700;
      color: #082f49;
    }

    .bar-saved {
      background: #27272a;
      height: 100%;
      flex-grow: 1;
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 11px;
      font-weight: 600;
      color: #a1a1aa;
    }

    .stats-legend {
      display: flex;
      justify-content: space-between;
      font-size: 13px;
      color: var(--text-muted);
      flex-wrap: wrap;
      gap: 8px;
    }

    .legend-item {
      display: flex;
      align-items: center;
      gap: 8px;
    }

    .legend-dot {
      width: 8px;
      height: 8px;
      border-radius: 50%;
    }

    /* Setup / Code snippet guide */
    .guide-tabs {
      display: flex;
      gap: 8px;
      margin-bottom: 16px;
      border-bottom: 1px solid var(--card-border);
      padding-bottom: 8px;
      overflow-x: auto;
    }

    .tab-btn {
      background: transparent;
      border: none;
      color: var(--text-muted);
      padding: 6px 12px;
      font-size: 13px;
      font-weight: 600;
      cursor: pointer;
      border-radius: 6px;
      transition: all 0.2s;
    }

    .tab-btn.active {
      background: #27272a;
      color: #fff;
    }

    .code-box {
      background: #000;
      border: 1px solid #27272a;
      border-radius: 8px;
      padding: 16px;
      font-family: var(--font-mono);
      font-size: 13px;
      color: #e4e4e7;
      overflow-x: auto;
      position: relative;
    }

    .copy-btn {
      position: absolute;
      top: 10px;
      right: 10px;
      background: #27272a;
      color: #d4d4d8;
      border: 1px solid #3f3f46;
      border-radius: 6px;
      padding: 4px 10px;
      font-size: 11px;
      font-weight: 600;
      cursor: pointer;
      transition: background 0.2s;
    }

    .copy-btn:hover {
      background: #3f3f46;
      color: #fff;
    }

    /* Footer */
    footer {
      display: flex;
      justify-content: space-between;
      font-size: 12px;
      color: var(--text-muted);
      padding-top: 24px;
      border-top: 1px solid var(--card-border);
      flex-wrap: wrap;
      gap: 12px;
    }

    footer a {
      color: var(--text-muted);
      text-decoration: none;
    }

    footer a:hover {
      color: var(--text-main);
    }
  </style>
</head>
<body>
  <div class="container">
    <header>
      <div class="brand">
        <span class="brand-icon">🗡️</span>
        <div>
          <span class="brand-title">Tokenectomy Razor</span>
          <span class="brand-badge" id="version-badge">v1.2.1</span>
        </div>
      </div>
      <div class="status-pill">
        <span class="pulse"></span>
        <span>GATEWAY ACTIVE (127.0.0.1:8080)</span>
      </div>
    </header>

    <!-- Top KPI Row -->
    <div class="kpi-grid">
      <div class="kpi-card">
        <div class="kpi-label">
          <span>Estimated Savings</span>
          <span>💰</span>
        </div>
        <div class="kpi-value green" id="val-cost">$0.00</div>
        <div class="kpi-subtext">Based on $3.00/M blended LLM prompt rate</div>
      </div>

      <div class="kpi-card">
        <div class="kpi-label">
          <span>Tokens Sliced</span>
          <span>✂️</span>
        </div>
        <div class="kpi-value cyan" id="val-tokens">0</div>
        <div class="kpi-subtext" id="sub-tokens">0 raw tokens intercepted</div>
      </div>

      <div class="kpi-card">
        <div class="kpi-label">
          <span>Noise Pruning</span>
          <span>📉</span>
        </div>
        <div class="kpi-value purple" id="val-reduction">0%</div>
        <div class="kpi-subtext">Average prompt context compression</div>
      </div>

      <div class="kpi-card">
        <div class="kpi-label">
          <span>Credentials Shielded</span>
          <span>🛡️</span>
        </div>
        <div class="kpi-value yellow" id="val-secrets">0</div>
        <div class="kpi-subtext">API keys, JWTs, and DB URIs redacted</div>
      </div>
    </div>

    <!-- Live Context Pruning Ratio -->
    <div class="section-card">
      <div class="section-title">
        <span>Context Compression Breakdown</span>
      </div>
      <div class="progress-bar-wrap">
        <div class="bar-sanitized" id="bar-sanitized" style="width: 100%;">Waiting for requests...</div>
        <div class="bar-saved" id="bar-saved" style="width: 0%;">Pruned Noise</div>
      </div>
      <div class="stats-legend">
        <div class="legend-item">
          <span class="legend-dot" style="background: var(--accent-cyan);"></span>
          <span id="txt-actual-tokens">Payload Sent to Upstream LLM: 0 tokens</span>
        </div>
        <div class="legend-item">
          <span class="legend-dot" style="background: #52525b;"></span>
          <span id="txt-pruned-tokens">Pruned Framework Boilerplate: 0 tokens</span>
        </div>
        <div class="legend-item">
          <span id="txt-requests">Requests Processed: 0</span>
        </div>
      </div>
    </div>

    <!-- Agent Connection Guide -->
    <div class="section-card">
      <div class="section-title">
        <span>Route Coding Agents Through Tokenectomy Gateway</span>
      </div>
      <div class="guide-tabs">
        <button class="tab-btn active" onclick="switchTab('cursor')">Cursor / Windsurf</button>
        <button class="tab-btn" onclick="switchTab('cline')">Cline / Roo Code</button>
        <button class="tab-btn" onclick="switchTab('python')">Python SDK</button>
        <button class="tab-btn" onclick="switchTab('curl')">cURL</button>
      </div>

      <div class="code-box" id="code-content">
        <button class="copy-btn" onclick="copyCode()">Copy</button>
        <pre><code id="code-text">// Point your OpenAI Base URL to Tokenectomy Gateway:
export OPENAI_BASE_URL="http://127.0.0.1:8080/v1"
export OPENAI_API_KEY="your-real-openai-api-key"</code></pre>
      </div>
    </div>

    <footer>
      <div>Tokenectomy Razor — Autonomous Sub-Cortex Infrastructure</div>
      <div>
        <a href="/v1/metrics" target="_blank">JSON Metrics API</a> · 
        <a href="https://github.com/Tokenectomy-Labs/Tokenectomy" target="_blank">GitHub</a> · 
        <a href="https://crates.io/crates/tokenectomy" target="_blank">Crates.io</a>
      </div>
    </footer>
  </div>

  <script>
    const snippets = {
      cursor: `// Cursor / Windsurf custom OpenAI Base URL:
// In Settings -> Models -> OpenAI API Base URL:
http://127.0.0.1:8080/v1`,
      cline: `// In Cline / Roo Code Settings:
// API Provider: OpenAI Compatible
// Base URL: http://127.0.0.1:8080/v1
// API Key: (Your standard OpenAI or Anthropic API Key)`,
      python: `from openai import OpenAI

# Transparently route all prompts through Tokenectomy Gateway
client = OpenAI(
    base_url="http://127.0.0.1:8080/v1",
    api_key="your-api-key"
)`,
      curl: `curl http://127.0.0.1:8080/v1/chat/completions \\
  -H "Content-Type: application/json" \\
  -H "Authorization: Bearer $OPENAI_API_KEY" \\
  -d '{"model": "gpt-4o", "messages": [{"role": "user", "content": "Analyze build error..."}]}'`
    };

    function switchTab(tab) {
      document.querySelectorAll('.tab-btn').forEach(btn => btn.classList.remove('active'));
      event.target.classList.add('active');
      document.getElementById('code-text').textContent = snippets[tab];
    }

    function copyCode() {
      const text = document.getElementById('code-text').textContent;
      navigator.clipboard.writeText(text).then(() => {
        const btn = document.querySelector('.copy-btn');
        btn.textContent = 'Copied!';
        setTimeout(() => btn.textContent = 'Copy', 2000);
      });
    }

    async function updateMetrics() {
      try {
        const res = await fetch('/v1/metrics');
        if (!res.ok) return;
        const data = await res.json();

        // Update KPIs
        document.getElementById('val-cost').textContent = '$' + (data.estimated_cost_saved_usd || 0).toFixed(2);
        document.getElementById('val-tokens').textContent = (data.estimated_tokens_saved || 0).toLocaleString();
        document.getElementById('sub-tokens').textContent = (data.estimated_raw_tokens || 0).toLocaleString() + ' raw tokens intercepted';
        document.getElementById('val-reduction').textContent = (data.reduction_percentage || 0).toFixed(1) + '%';
        document.getElementById('val-secrets').textContent = (data.secrets_redacted || 0).toLocaleString();
        document.getElementById('version-badge').textContent = 'v' + (data.version || '1.2.1');

        // Update Progress bar
        const raw = data.estimated_raw_tokens || 0;
        const sanitized = data.estimated_sanitized_tokens || 0;
        const saved = data.estimated_tokens_saved || 0;

        if (raw > 0) {
          const sanitizedPct = Math.max(5, (sanitized / raw) * 100);
          const savedPct = 100 - sanitizedPct;
          document.getElementById('bar-sanitized').style.width = sanitizedPct + '%';
          document.getElementById('bar-sanitized').textContent = sanitizedPct.toFixed(0) + '% Payload';
          document.getElementById('bar-saved').style.width = savedPct + '%';
          document.getElementById('bar-saved').textContent = '-' + savedPct.toFixed(0) + '% Pruned';
        }

        document.getElementById('txt-actual-tokens').textContent = 'Payload Sent: ' + sanitized.toLocaleString() + ' tokens';
        document.getElementById('txt-pruned-tokens').textContent = 'Pruned Noise: ' + saved.toLocaleString() + ' tokens';
        document.getElementById('txt-requests').textContent = 'Requests Processed: ' + (data.total_requests || 0).toLocaleString();
      } catch (err) {
        console.debug('Polling metrics:', err);
      }
    }

    // Initial fetch and 1.5s live polling
    updateMetrics();
    setInterval(updateMetrics, 1500);
  </script>
</body>
</html>"#;
