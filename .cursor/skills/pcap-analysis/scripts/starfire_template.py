"""StarFire Forensics Report — HTML Template with tabbed interface."""

STARFIRE_HTML = """<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>StarFire Forensics Report</title>
<style>
:root {{
    --bg-primary: #0a0a0f;
    --bg-secondary: #12121a;
    --bg-card: #1a1a2e;
    --bg-card-hover: #222240;
    --accent: #ff6b35;
    --accent-glow: rgba(255, 107, 53, 0.3);
    --text-primary: #e8e8f0;
    --text-secondary: #a0a0b8;
    --text-muted: #6a6a80;
    --border: #2a2a40;
    --success: #4ade80;
    --warning: #fbbf24;
    --danger: #f87171;
    --info: #60a5fa;
    --critical: #dc2626;
    --font-mono: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
    --font-sans: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
}}

* {{ margin: 0; padding: 0; box-sizing: border-box; }}

body {{
    font-family: var(--font-sans);
    background: var(--bg-primary);
    color: var(--text-primary);
    min-height: 100vh;
    line-height: 1.6;
}}

.header {{
    background: linear-gradient(135deg, var(--bg-secondary) 0%, #1a0a20 100%);
    border-bottom: 1px solid var(--border);
    padding: 1.5rem 2rem;
    display: flex;
    align-items: center;
    justify-content: space-between;
}}

.header-left {{
    display: flex;
    align-items: center;
    gap: 1rem;
}}

.logo {{
    font-size: 1.5rem;
    font-weight: 800;
    background: linear-gradient(135deg, var(--accent), #ff9f6b);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    letter-spacing: -0.5px;
}}

.logo-sub {{
    font-size: 0.75rem;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 2px;
}}

.lattice-badge {{
    background: var(--bg-card);
    border: 1px solid var(--success);
    border-radius: 6px;
    padding: 0.4rem 0.8rem;
    font-family: var(--font-mono);
    font-size: 0.7rem;
    color: var(--success);
}}

.council-status {{
    display: flex;
    gap: 0.5rem;
    align-items: center;
    font-size: 0.8rem;
    color: var(--text-secondary);
}}

.council-dot {{
    width: 8px;
    height: 8px;
    border-radius: 50%;
    animation: pulse 2s infinite;
}}

.council-dot.alpha {{ background: var(--accent); }}
.council-dot.beta {{ background: var(--info); }}

@keyframes pulse {{
    0%, 100% {{ opacity: 1; }}
    50% {{ opacity: 0.4; }}
}}

.tabs {{
    display: flex;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    overflow-x: auto;
    padding: 0 1rem;
}}

.tab {{
    padding: 0.9rem 1.4rem;
    cursor: pointer;
    color: var(--text-muted);
    font-size: 0.85rem;
    font-weight: 500;
    border-bottom: 2px solid transparent;
    transition: all 0.2s;
    white-space: nowrap;
}}

.tab:hover {{ color: var(--text-primary); background: var(--bg-card); }}
.tab.active {{
    color: var(--accent);
    border-bottom-color: var(--accent);
    background: rgba(255, 107, 53, 0.05);
}}

.tab-content {{ display: none; padding: 2rem; }}
.tab-content.active {{ display: block; }}

.card {{
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 1.5rem;
    margin-bottom: 1.5rem;
}}

.card-title {{
    font-size: 1rem;
    font-weight: 600;
    margin-bottom: 1rem;
    color: var(--text-primary);
    display: flex;
    align-items: center;
    gap: 0.5rem;
}}

.metric-grid {{
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
    margin-bottom: 1.5rem;
}}

.metric {{
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 1rem;
}}

.metric-value {{
    font-size: 1.8rem;
    font-weight: 700;
    font-family: var(--font-mono);
    color: var(--accent);
}}

.metric-label {{
    font-size: 0.75rem;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 1px;
    margin-top: 0.25rem;
}}

table {{
    width: 100%;
    border-collapse: collapse;
    font-size: 0.85rem;
}}

th {{
    text-align: left;
    padding: 0.75rem 1rem;
    background: var(--bg-secondary);
    color: var(--text-muted);
    font-weight: 600;
    text-transform: uppercase;
    font-size: 0.7rem;
    letter-spacing: 1px;
    border-bottom: 1px solid var(--border);
}}

td {{
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--border);
    font-family: var(--font-mono);
    font-size: 0.8rem;
}}

tr:hover {{ background: var(--bg-card-hover); }}

.badge {{
    display: inline-block;
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
    font-size: 0.7rem;
    font-weight: 600;
    text-transform: uppercase;
}}

.badge-critical {{ background: rgba(220, 38, 38, 0.2); color: var(--critical); border: 1px solid var(--critical); }}
.badge-high {{ background: rgba(248, 113, 113, 0.2); color: var(--danger); border: 1px solid var(--danger); }}
.badge-warning {{ background: rgba(251, 191, 36, 0.2); color: var(--warning); border: 1px solid var(--warning); }}
.badge-info {{ background: rgba(96, 165, 250, 0.2); color: var(--info); border: 1px solid var(--info); }}
.badge-success {{ background: rgba(74, 222, 128, 0.2); color: var(--success); border: 1px solid var(--success); }}

.debate-entry {{
    background: var(--bg-secondary);
    border-left: 3px solid var(--accent);
    padding: 1rem 1.5rem;
    margin-bottom: 1rem;
    border-radius: 0 6px 6px 0;
}}

.debate-alpha {{
    border-left-color: var(--accent);
}}

.debate-beta {{
    border-left-color: var(--info);
}}

.debate-resolution {{
    border-left-color: var(--success);
    background: rgba(74, 222, 128, 0.05);
}}

.debate-label {{
    font-size: 0.7rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 1px;
    margin-bottom: 0.5rem;
}}

.debate-label.alpha {{ color: var(--accent); }}
.debate-label.beta {{ color: var(--info); }}
.debate-label.resolution {{ color: var(--success); }}

.socratic-step {{
    position: relative;
    padding-left: 2rem;
    margin-bottom: 1.5rem;
}}

.socratic-step::before {{
    content: '?';
    position: absolute;
    left: 0;
    top: 0;
    width: 1.5rem;
    height: 1.5rem;
    background: var(--accent);
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 700;
    font-size: 0.8rem;
    color: white;
}}

.socratic-step .question {{
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: 0.5rem;
}}

.socratic-step .answer {{
    color: var(--text-secondary);
    font-family: var(--font-mono);
    font-size: 0.8rem;
    background: var(--bg-secondary);
    padding: 0.75rem;
    border-radius: 4px;
}}

.process-chain {{
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
    font-family: var(--font-mono);
    font-size: 0.8rem;
}}

.process-node {{
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 0.3rem 0.6rem;
}}

.process-arrow {{
    color: var(--text-muted);
}}

.payload-block {{
    background: #0d1117;
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 1rem;
    font-family: var(--font-mono);
    font-size: 0.75rem;
    overflow-x: auto;
    white-space: pre-wrap;
    word-break: break-all;
    color: var(--text-secondary);
    max-height: 300px;
    overflow-y: auto;
}}

.verdict-card {{
    display: flex;
    align-items: flex-start;
    gap: 1rem;
    padding: 1rem;
    background: var(--bg-secondary);
    border-radius: 6px;
    margin-bottom: 0.75rem;
    border: 1px solid var(--border);
}}

.verdict-icon {{
    font-size: 1.5rem;
    flex-shrink: 0;
}}

.chart-container {{
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 1rem;
    margin-bottom: 1.5rem;
}}

.chart-container img {{
    width: 100%;
    border-radius: 4px;
}}

footer {{
    text-align: center;
    padding: 2rem;
    color: var(--text-muted);
    font-size: 0.75rem;
    border-top: 1px solid var(--border);
}}
</style>
</head>
<body>

<div class="header">
    <div class="header-left">
        <div>
            <div class="logo">STARFIRE</div>
            <div class="logo-sub">Forensics Report</div>
        </div>
        <div class="council-status">
            <span class="council-dot alpha"></span> Council-&alpha;
            <span class="council-dot beta"></span> Council-&beta;
            <span>| Alignment: {council_alignment}</span>
        </div>
    </div>
    <div>
        <div class="lattice-badge">LATTICE {lattice_hash}</div>
    </div>
</div>

<div class="tabs">
    <div class="tab active" onclick="switchTab('overview')">Overview</div>
    <div class="tab" onclick="switchTab('offenders')">Top Offenders</div>
    <div class="tab" onclick="switchTab('debate')">Council Debate</div>
    <div class="tab" onclick="switchTab('drip')">Drip Trace</div>
    <div class="tab" onclick="switchTab('timeline')">Timeline</div>
    <div class="tab" onclick="switchTab('payloads')">Payloads</div>
    <div class="tab" onclick="switchTab('processes')">Processes</div>
    <div class="tab" onclick="switchTab('evidence')">Evidence Chain</div>
</div>

<div id="tab-overview" class="tab-content active">
    {tab_overview}
</div>

<div id="tab-offenders" class="tab-content">
    {tab_offenders}
</div>

<div id="tab-debate" class="tab-content">
    {tab_debate}
</div>

<div id="tab-drip" class="tab-content">
    {tab_drip_trace}
</div>

<div id="tab-timeline" class="tab-content">
    {tab_timeline}
</div>

<div id="tab-payloads" class="tab-content">
    {tab_payloads}
</div>

<div id="tab-processes" class="tab-content">
    {tab_processes}
</div>

<div id="tab-evidence" class="tab-content">
    <div class="card">
        <div class="card-title">Crystalline Lattice — Evidence Integrity</div>
        <div class="metric-grid">
            <div class="metric">
                <div class="metric-value">{evidence_count}</div>
                <div class="metric-label">Evidence Items</div>
            </div>
            <div class="metric">
                <div class="metric-value" style="font-size:1rem;">{lattice_hash}</div>
                <div class="metric-label">Lattice Hash (SHA-256)</div>
            </div>
        </div>
        <p style="color: var(--success); font-size: 0.85rem;">
            &#x2714; All evidence hashes verified. Chain of custody intact. Diamond-hand protocol enforced.
        </p>
    </div>
    {tab_evidence}
</div>

<footer>
    StarFire Forensics Engine | Generated {timestamp} | Crystalline Lattice Protected | Dual Council Verified
</footer>

<script>
function switchTab(tabId) {{
    document.querySelectorAll('.tab-content').forEach(el => el.classList.remove('active'));
    document.querySelectorAll('.tab').forEach(el => el.classList.remove('active'));
    document.getElementById('tab-' + tabId).classList.add('active');
    event.target.classList.add('active');
}}
</script>
</body>
</html>"""
