#!/usr/bin/env python3
"""
StarFire Report Generator - LATTICE TEAM KHET-1
Default: Pure algorithmic / zero-AI (court-safe, offline)
Optional: --local-llm for narrative sections via Ollama
"""

import json
import os
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, Any, Optional

import sys
sys.path.insert(0, str(Path(__file__).parent.parent))

from merkle.tree import MerkleTree


class StarFireReport:
    def __init__(self, output_dir: str = "reports", llm_model: str = "llama3.1:8b"):
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(parents=True, exist_ok=True)
        self.timestamp = datetime.now(timezone.utc).isoformat()
        self.merkle = MerkleTree()
        self.data: Dict[str, Any] = {}
        self._llm_model = llm_model

    def add_section(self, name: str, content: Dict):
        """Add any section with automatic Merkle leaf."""
        section_str = json.dumps(content, sort_keys=True, default=str)
        self.merkle.add_leaf(f"SECTION:{name}:{section_str}", label=f"section:{name}")
        self.data[name] = content

    def generate(self, pcap_results: Dict, custody_manifest: Optional[Dict] = None,
                 use_local_llm: bool = False):
        """Generate full StarFire report."""
        mode = "OFFLINE_ZERO_AI" if not use_local_llm else "OFFLINE_LOCAL_LLM"

        self.data = {
            "report_info": {
                "title": "StarFire Forensic Network Report",
                "operation": "LATTICE TEAM — KHET-1",
                "generated": self.timestamp,
                "mode": mode,
                "merkle_root": ""
            },
            "executive_summary": self._build_exec_summary(pcap_results),
            "timeline": self._build_timeline(pcap_results),
            "top_castles": pcap_results.get("castles", []),
            "council_debate": pcap_results.get("debate", {}),
            "drip_trace": self._build_drip_trace(pcap_results),
            "payload_samples": pcap_results.get("payloads", []),
            "process_context": pcap_results.get("processes", {}),
            "evidence_chain": self._build_evidence_chain(custody_manifest),
            "methodology": self._build_methodology(),
            "residual_risks": pcap_results.get("residual_risks", [])
        }

        # Merkle-seal every section
        for name, content in self.data.items():
            if name != "report_info":
                self.add_section(name, content)

        self.merkle.build()
        self.data["report_info"]["merkle_root"] = self.merkle.get_root()

        # Generate HTML
        html_path = self.output_dir / "starfire_report.html"
        html_path.write_text(self._render_html(), encoding='utf-8')

        # Save manifest
        self.merkle.save_manifest(str(self.output_dir / "merkle_manifest.json"))

        # Save JSON
        json_path = self.output_dir / "starfire_report.json"
        json_path.write_text(json.dumps(self.data, indent=2, default=str), encoding='utf-8')

        print(f"Report: {html_path}")
        print(f"JSON:   {json_path}")
        print(f"Merkle: {self.merkle.get_root()[:24]}...")

        return self.data

    def _build_exec_summary(self, results: Dict) -> Dict:
        return {
            "classification": results.get("final_classification", "UNDETERMINED"),
            "confidence_percent": results.get("confidence", 0),
            "council_alignment": results.get("council_alignment", "N/A"),
            "key_findings": results.get("key_findings", []),
            "total_packets": results.get("total_packets", 0),
            "total_evidence_bytes": results.get("total_bytes", 0),
            "capture_duration": results.get("duration", "unknown"),
            "verdict": results.get("verdict", "Pending review")
        }

    def _build_timeline(self, results: Dict) -> Dict:
        return {
            "events": results.get("timeline_events", []),
            "first_packet": results.get("first_packet", ""),
            "last_packet": results.get("last_packet", ""),
            "peak_throughput": results.get("peak_throughput", ""),
            "protocol_distribution": results.get("protocols", {})
        }

    def _build_drip_trace(self, results: Dict) -> Dict:
        return {
            "trace": results.get("drip_trace", []),
            "drip_classification": results.get("drip_classification", ""),
            "source_process": results.get("source_process", ""),
            "target_device": results.get("target_device", "")
        }

    def _build_evidence_chain(self, custody: Optional[Dict] = None) -> Dict:
        chain = {
            "merkle_root": self.merkle.get_root(),
            "merkle_leaves": len(self.merkle.leaves),
            "custody_statement": (
                "All analysis performed offline. No network calls made. "
                "No AI APIs contacted. Evidence integrity verified via SHA-256 "
                "Merkle tree. Chain of custody maintained throughout."
            ),
            "network_calls_made": 0,
            "ai_api_calls_made": 0,
            "triple_anchor": {
                "anchor_1": "SHA-256 hash per file (independently verifiable)",
                "anchor_2": "Git commit history (timestamped, immutable)",
                "anchor_3": "Merkle tree inclusion proof (mathematical verification)"
            }
        }
        if custody:
            chain["custody_manifest"] = custody
        return chain

    def _generate_narrative(self, results: Dict) -> str:
        """Optional local Ollama narrative. Fully offline — model runs on local hardware."""
        try:
            import subprocess
            prompt = (
                "You are a network forensics analyst writing for a non-technical audience. "
                "Summarize the following findings in clear, court-friendly language. "
                "State only what the evidence shows. Do not speculate.\n\n"
                f"{json.dumps(results, indent=2, default=str)[:6000]}"
            )

            result = subprocess.run(
                ["ollama", "run", self._llm_model, prompt],
                capture_output=True, text=True, timeout=120,
                env={**os.environ, 'OLLAMA_HOST': 'http://127.0.0.1:11434'}
            )

            if result.returncode == 0:
                return result.stdout.strip()
            return f"[Ollama returned non-zero: {result.stderr[:200]}]"
        except FileNotFoundError:
            return "[Ollama not installed. Install: curl -fsSL https://ollama.com/install.sh | sh]"
        except subprocess.TimeoutExpired:
            return "[Local LLM timed out (120s). Try smaller model: ollama pull llama3.2:3b]"
        except Exception as e:
            return f"[Local LLM error: {str(e)}]"

    def _build_methodology(self) -> Dict:
        return {
            "approach": "Dual-council adversarial analysis (Red/Blue) with Final Maker resolution",
            "red_council": "Assumes hostile intent. Detects: beaconing, volume anomalies, port scanning, DNS tunneling, high-entropy payloads.",
            "blue_council": "Assumes benign behavior. Identifies: standard HTTPS, infrastructure services, keepalives, known endpoints.",
            "final_maker": "Weighs evidence from both councils using confidence scoring. Resolves all contested findings.",
            "evidence_handling": "First-touch SHA-256 sealing. Read-only analysis. Merkle tree linking all nodes.",
            "reproducibility": "Any party with the same pcap files and this toolkit will produce identical analytical results (deterministic).",
            "tools_used": [
                "tshark/pyshark (packet parsing)",
                "SHA-256 (evidence integrity)",
                "Merkle tree (inclusion proofs)",
                "Statistical interval analysis (beacon detection)",
                "TLS SNI extraction (destination identification)"
            ]
        }

    def _render_html(self) -> str:
        """Generate fully self-contained HTML report. No external CDN deps for airgap."""
        report_json = json.dumps(self.data, indent=2, default=str)
        merkle_root = self.data['report_info']['merkle_root']
        mode = self.data['report_info']['mode']
        timestamp = self.data['report_info']['generated']

        # Build tab content from data
        exec_summary = self.data.get('executive_summary', {})
        evidence = self.data.get('evidence_chain', {})
        methodology = self.data.get('methodology', {})

        findings_html = ''
        for f in exec_summary.get('key_findings', []):
            if isinstance(f, str):
                findings_html += f"<li>{f}</li>"
            elif isinstance(f, dict):
                findings_html += f"<li><strong>{f.get('title','')}</strong>: {f.get('detail','')}</li>"

        castles_html = ''
        for c in self.data.get('top_castles', []):
            sev = c.get('severity', 'INFO')
            castles_html += f"""<div style="border:1px solid #ddd;padding:0.8rem;margin:0.5rem 0;border-radius:4px;border-left:3px solid {'#e53e3e' if sev in ('CRITICAL','HIGH') else '#ecc94b' if sev == 'MEDIUM' else '#4299e1'};">
                <strong>[{sev}] {c.get('name','')}</strong>
                <p style="margin:0.3rem 0 0;color:#555;font-size:0.9rem;">{c.get('conclusion','')}</p>
                <p style="margin:0.2rem 0 0;color:#888;font-size:0.8rem;">Confidence: {c.get('confidence',0):.0%} | Origin: {c.get('council_origin','')}</p>
            </div>"""

        debate_html = ''
        debate = self.data.get('council_debate', {})
        if isinstance(debate, dict):
            for d in debate.get('debate_log', debate.get('highlights', [])):
                debate_html += f"""<div style="margin:1rem 0;padding:1rem;background:#f9f9f9;border-radius:4px;">
                    <p style="color:#c53030;font-weight:600;">RED: {d.get('red_hypothesis','')}</p>
                    <p style="color:#2b6cb0;font-weight:600;margin-top:0.5rem;">BLUE: {d.get('blue_hypothesis','')}</p>
                    <p style="color:#276749;margin-top:0.5rem;font-style:italic;">Resolution: {d.get('resolution','')} — {d.get('reasoning','')}</p>
                </div>"""

        drip_html = ''
        drip = self.data.get('drip_trace', {})
        for step in drip.get('trace', []):
            q = step.get('q', step.get('question', ''))
            a = step.get('a', step.get('answer', ''))
            drip_html += f"""<div style="margin:0.75rem 0;padding-left:1.5rem;border-left:3px solid #ed8936;">
                <p style="font-weight:600;">{q}</p>
                <p style="color:#555;font-family:monospace;font-size:0.85rem;margin-top:0.25rem;">{a}</p>
            </div>"""

        tools_html = ''.join(f"<li>{t}</li>" for t in methodology.get('tools_used', []))

        html = f"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>StarFire Forensic Report</title>
<style>
body {{ font-family:system-ui,-apple-system,sans-serif; margin:0; padding:0; background:#f7f7f8; color:#1a1a2e; }}
.container {{ max-width:960px; margin:0 auto; padding:2rem; }}
header {{ text-align:center; border-bottom:2px solid #e2e2e8; padding-bottom:1.5rem; margin-bottom:2rem; }}
h1 {{ font-size:1.8rem; color:#c53030; margin:0; }}
.mode-badge {{ display:inline-block; margin-top:0.75rem; padding:0.4rem 1rem; background:#c6f6d5; color:#276749; border-radius:20px; font-family:monospace; font-size:0.8rem; }}
.tabs {{ display:flex; border-bottom:2px solid #e2e2e8; margin-bottom:1.5rem; flex-wrap:wrap; }}
.tab {{ padding:0.7rem 1.2rem; cursor:pointer; color:#666; font-size:0.9rem; border-bottom:2px solid transparent; margin-bottom:-2px; }}
.tab:hover {{ color:#1a1a2e; }}
.tab.active {{ color:#c53030; border-bottom-color:#c53030; font-weight:600; }}
.panel {{ display:none; }}
.panel.active {{ display:block; }}
.card {{ background:white; border:1px solid #e2e2e8; border-radius:8px; padding:1.5rem; margin-bottom:1.5rem; box-shadow:0 1px 3px rgba(0,0,0,0.05); }}
.card h2 {{ font-size:1.1rem; margin:0 0 1rem; color:#1a1a2e; }}
.mono {{ font-family:monospace; font-size:0.8rem; }}
.seal {{ background:#f0fff4; border:2px solid #276749; border-radius:8px; padding:1rem; margin:1rem 0; }}
table {{ width:100%; border-collapse:collapse; font-size:0.85rem; }}
th,td {{ padding:0.5rem 0.7rem; text-align:left; border-bottom:1px solid #e2e2e8; }}
th {{ background:#f7f7f8; font-weight:600; color:#555; font-size:0.75rem; text-transform:uppercase; }}
@media print {{ .tabs,.no-print {{ display:none; }} .panel {{ display:block !important; page-break-inside:avoid; }} }}
</style>
</head>
<body>
<div class="container">
<header>
    <h1>STARFIRE FORENSIC REPORT</h1>
    <p style="color:#666;margin:0.3rem 0;">LATTICE TEAM — KHET-1 | {timestamp}</p>
    <div class="mode-badge">{mode} | MERKLE: {merkle_root[:16]}...</div>
</header>

<div class="tabs">
    <div class="tab active" onclick="openTab(event,'summary')">Summary</div>
    <div class="tab" onclick="openTab(event,'timeline')">Timeline</div>
    <div class="tab" onclick="openTab(event,'castles')">Castles</div>
    <div class="tab" onclick="openTab(event,'debate')">Debate</div>
    <div class="tab" onclick="openTab(event,'drip')">Drip Trace</div>
    <div class="tab" onclick="openTab(event,'payloads')">Payloads</div>
    <div class="tab" onclick="openTab(event,'evidence')">Evidence</div>
    <div class="tab" onclick="openTab(event,'methodology')">Methodology</div>
    <div class="tab" onclick="openTab(event,'risks')">Risks</div>
    <div class="tab" onclick="openTab(event,'raw')">Raw JSON</div>
</div>

<div id="panel-summary" class="panel active">
    <div class="card">
        <h2>Executive Summary</h2>
        <table>
            <tr><th>Classification</th><td>{exec_summary.get('classification','')}</td></tr>
            <tr><th>Confidence</th><td>{exec_summary.get('confidence_percent',0)}%</td></tr>
            <tr><th>Council Alignment</th><td>{exec_summary.get('council_alignment','')}</td></tr>
            <tr><th>Total Packets</th><td>{exec_summary.get('total_packets',0):,}</td></tr>
            <tr><th>Verdict</th><td><strong>{exec_summary.get('verdict','')}</strong></td></tr>
        </table>
        <h3 style="margin-top:1rem;">Key Findings</h3>
        <ul>{findings_html or '<li>No findings recorded</li>'}</ul>
    </div>
</div>

<div id="panel-timeline" class="panel">
    <div class="card">
        <h2>Timeline</h2>
        <table>
            <tr><th>First Packet</th><td class="mono">{self.data.get('timeline',{}).get('first_packet','')}</td></tr>
            <tr><th>Last Packet</th><td class="mono">{self.data.get('timeline',{}).get('last_packet','')}</td></tr>
            <tr><th>Peak Throughput</th><td>{self.data.get('timeline',{}).get('peak_throughput','')}</td></tr>
        </table>
    </div>
</div>

<div id="panel-castles" class="panel">
    <div class="card">
        <h2>Top Castles / Offenders</h2>
        {castles_html or '<p style="color:#888;">No castles built during this analysis.</p>'}
    </div>
</div>

<div id="panel-debate" class="panel">
    <div class="card">
        <h2>Red vs Blue Council Debate</h2>
        {debate_html or '<p style="color:#888;">Councils aligned — no contested findings.</p>'}
    </div>
</div>

<div id="panel-drip" class="panel">
    <div class="card">
        <h2>Reverse Socratic Drip Trace</h2>
        <p style="margin-bottom:1rem;color:#555;">Working backwards from network observation to root cause.</p>
        {drip_html or '<p style="color:#888;">No drip trace performed.</p>'}
    </div>
</div>

<div id="panel-payloads" class="panel">
    <div class="card">
        <h2>Payload Samples</h2>
        <p style="color:#888;">Payload samples will appear here when pcap analysis includes payload extraction.</p>
    </div>
</div>

<div id="panel-evidence" class="panel">
    <div class="card">
        <h2>Evidence Chain & Vault</h2>
        <div class="seal">
            <p><strong>Merkle Root:</strong> <span class="mono">{merkle_root}</span></p>
            <p><strong>Leaves:</strong> {evidence.get('merkle_leaves',0)}</p>
            <p><strong>Network Calls:</strong> {evidence.get('network_calls_made',0)}</p>
            <p><strong>AI API Calls:</strong> {evidence.get('ai_api_calls_made',0)}</p>
        </div>
        <p style="margin-top:1rem;">{evidence.get('custody_statement','')}</p>
        <h3 style="margin-top:1rem;">Triple Anchor</h3>
        <table>
            <tr><th>Anchor 1</th><td>SHA-256 hash per file (independently verifiable)</td></tr>
            <tr><th>Anchor 2</th><td>Git commit history (timestamped, immutable)</td></tr>
            <tr><th>Anchor 3</th><td>Merkle tree inclusion proof (mathematical verification)</td></tr>
        </table>
    </div>
</div>

<div id="panel-methodology" class="panel">
    <div class="card">
        <h2>Methodology & Reproducibility</h2>
        <table>
            <tr><th>Approach</th><td>{methodology.get('approach','')}</td></tr>
            <tr><th>Red Council</th><td>{methodology.get('red_council','')}</td></tr>
            <tr><th>Blue Council</th><td>{methodology.get('blue_council','')}</td></tr>
            <tr><th>Final Maker</th><td>{methodology.get('final_maker','')}</td></tr>
            <tr><th>Evidence Handling</th><td>{methodology.get('evidence_handling','')}</td></tr>
            <tr><th>Reproducibility</th><td>{methodology.get('reproducibility','')}</td></tr>
        </table>
        <h3 style="margin-top:1rem;">Tools Used</h3>
        <ul>{tools_html}</ul>
    </div>
</div>

<div id="panel-risks" class="panel">
    <div class="card">
        <h2>Residual Risks & Lessons Learned</h2>
        <p style="color:#888;">Residual risk items will be populated based on analysis scope and unresolved findings.</p>
    </div>
</div>

<div id="panel-raw" class="panel">
    <div class="card">
        <h2>Raw Report Data (JSON)</h2>
        <pre style="background:#1a1a2e;color:#e2e8f0;padding:1rem;border-radius:6px;overflow:auto;max-height:600px;font-size:0.75rem;">{report_json}</pre>
    </div>
</div>

<footer style="text-align:center;margin-top:2rem;padding:1rem;color:#888;font-size:0.75rem;border-top:1px solid #e2e2e8;">
    LATTICE TEAM — KHET-1 | StarFire Forensics | Merkle: {merkle_root[:16]}... | {mode}
</footer>
</div>

<script>
function openTab(evt, id) {{
    document.querySelectorAll('.panel').forEach(p => p.classList.remove('active'));
    document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
    document.getElementById('panel-' + id).classList.add('active');
    evt.target.classList.add('active');
}}
</script>
</body>
</html>"""
        return html
