---
name: pcap-analysis
description: "StarFire Forensics: Perform forensic-grade PCAP analysis with dual adversarial council methodology, crystalline lattice evidence protection, local process drip tracing via reverse Socratic reasoning, and generate interactive tabbed HTML reports. Use when the user provides pcap files, asks about network drip/leak analysis, local forensics, or needs to identify what process on the local machine is causing data exfiltration."
disable-model-invocation: true
---

# StarFire Forensics — PCAP Analysis & Network Drip Investigation

## Architecture Overview

```
┌──────────────────────────────────────────────────────────────┐
│                    STARFIRE FORENSICS                          │
├──────────────────────────────────────────────────────────────┤
│                                                               │
│   ┌─────────────┐    ┌─────────────────┐    ┌────────────┐  │
│   │  CRYSTALLINE │    │  DUAL COUNCILS  │    │  STARFIRE   │  │
│   │  LATTICE     │───▶│  (Counter-Rot)  │───▶│  REPORT    │  │
│   │  EVIDENCE    │    │                 │    │  (HTML)     │  │
│   │  PROTECTION  │    │  Council-α ↻    │    │            │  │
│   └─────────────┘    │  Council-β ↺    │    └────────────┘  │
│         │             │        │        │          ▲         │
│         ▼             │   Opus 4.6      │          │         │
│   ┌─────────────┐    │   Final Maker   │    ┌────────────┐  │
│   │  REVERSE     │    └─────────────────┘    │  TABBED    │  │
│   │  SOCRATIC    │                           │  WEBSITE   │  │
│   │  TRACE       │──────────────────────────▶│  OUTPUT    │  │
│   └─────────────┘                            └────────────┘  │
│                                                               │
└──────────────────────────────────────────────────────────────┘
```

---

## Phase 0: Crystalline Lattice Clap — Evidence Protection

Before ANY analysis begins, establish immutable evidence integrity.

```python
import hashlib
import json
from datetime import datetime, timezone
from pathlib import Path

def crystalline_lattice_clap(pcap_paths):
    """
    Establish evidence chain. Hash every pcap, create lattice manifest.
    Diamond-hand rule: originals are NEVER modified.
    """
    lattice = {
        'created': datetime.now(timezone.utc).isoformat(),
        'analyst': 'StarFire Forensics Engine',
        'evidence_items': [],
        'lattice_hash': None
    }

    for path in pcap_paths:
        p = Path(path)
        sha256 = hashlib.sha256(p.read_bytes()).hexdigest()
        md5 = hashlib.md5(p.read_bytes()).hexdigest()
        lattice['evidence_items'].append({
            'filename': p.name,
            'path': str(p.absolute()),
            'size_bytes': p.stat().st_size,
            'sha256': sha256,
            'md5': md5,
            'timestamp_accessed': datetime.now(timezone.utc).isoformat()
        })

    # Lattice self-seal
    lattice_content = json.dumps(lattice['evidence_items'], sort_keys=True)
    lattice['lattice_hash'] = hashlib.sha256(lattice_content.encode()).hexdigest()

    manifest_path = Path('starfire_lattice_manifest.json')
    manifest_path.write_text(json.dumps(lattice, indent=2))
    print(f"[CRYSTALLINE LATTICE] Evidence sealed. Lattice hash: {lattice['lattice_hash'][:16]}...")
    print(f"[DIAMOND HAND] {len(lattice['evidence_items'])} items protected. Originals untouched.")
    return lattice
```

**Rules:**
- NEVER write to or modify original pcap files
- ALL analysis operates on read-only file handles
- Lattice manifest is generated FIRST and verified LAST
- Any hash mismatch = abort and report tampering

---

## Phase 1: Dual Council Swarm — Counter-Rotating Analysis

Two adversarial analysis councils examine the same data from opposing perspectives.

### Council-α (Clockwise / Attribution Council)
**Mandate:** Assume the drip is intentional. Find the actor, the method, the destination.
- Focus: Who is exfiltrating? What data? Where is it going?
- Lens: Malicious insider, compromised process, C2 channel

### Council-β (Counter-Clockwise / Innocence Council)  
**Mandate:** Assume the drip is benign. Find the legitimate explanation.
- Focus: What normal process would produce this traffic pattern?
- Lens: Telemetry, auto-updates, cloud sync, DNS prefetch, keepalives

### Council Execution Framework

```python
def council_alpha_analysis(packets_df, local_ip, offenders):
    """Attribution council - assumes malicious intent."""
    findings = []

    # Pattern: sustained low-bandwidth outbound (classic exfil)
    for ip, stats in offenders.items():
        if stats['bytes_out'] > 0 and stats['packet_regularity'] > 0.8:
            findings.append({
                'severity': 'HIGH',
                'hypothesis': f'{ip} shows metered exfiltration pattern',
                'evidence': f"Regularity={stats['packet_regularity']:.2f}, "
                           f"Avg payload={stats['avg_payload']}B, "
                           f"Duration={stats['duration']}s",
                'process_suspect': stats.get('process', 'unknown'),
                'confidence': 0.0  # scored later
            })

    # Pattern: DNS tunneling
    dns_queries = packets_df[packets_df['dst_port'] == 53]
    if len(dns_queries) > 0:
        avg_query_len = dns_queries['payload_size'].mean()
        if avg_query_len > 50:
            findings.append({
                'severity': 'CRITICAL',
                'hypothesis': 'Possible DNS tunneling - oversized queries',
                'evidence': f"Avg DNS payload={avg_query_len:.0f}B (normal <40B)",
                'confidence': 0.0
            })

    # Pattern: beaconing (periodic callbacks)
    # ... interval analysis ...

    return {'council': 'alpha', 'mandate': 'attribution', 'findings': findings}


def council_beta_analysis(packets_df, local_ip, offenders):
    """Innocence council - assumes benign behavior."""
    findings = []

    for ip, stats in offenders.items():
        # Check against known benign destinations
        if is_known_cdn(ip) or is_known_telemetry(ip):
            findings.append({
                'severity': 'LOW',
                'hypothesis': f'{ip} is known telemetry/CDN endpoint',
                'evidence': f"Resolves to {reverse_dns(ip)}, matches known infra",
                'explanation': 'Normal OS/app telemetry',
                'confidence': 0.0
            })

        # Regular intervals = cron/heartbeat, not necessarily C2
        if stats['packet_regularity'] > 0.8:
            findings.append({
                'severity': 'INFO',
                'hypothesis': f'{ip} regularity consistent with keepalive/healthcheck',
                'evidence': f"Interval matches common heartbeat periods (30s/60s/300s)",
                'confidence': 0.0
            })

    return {'council': 'beta', 'mandate': 'innocence', 'findings': findings}
```

### Opus 4.6 Final Maker — Conflict Resolution

When councils disagree, the Final Maker resolves:

```python
def final_maker_resolution(alpha_findings, beta_findings):
    """
    Opus 4.6 Final Maker: Resolves conflicts between councils.
    Uses weight-of-evidence, not majority vote.
    """
    debate_log = []
    final_verdicts = []

    for alpha_f in alpha_findings:
        # Find contradicting beta finding for same target
        beta_counter = find_counter_argument(beta_findings, alpha_f)

        if beta_counter:
            debate_entry = {
                'target': alpha_f.get('hypothesis'),
                'alpha_position': alpha_f,
                'beta_position': beta_counter,
                'resolution': None,
                'reasoning': None
            }

            # Weight evidence
            alpha_score = score_evidence(alpha_f)
            beta_score = score_evidence(beta_counter)

            if alpha_score > beta_score * 1.5:
                debate_entry['resolution'] = 'ALPHA_PREVAILS'
                debate_entry['reasoning'] = (
                    f"Attribution evidence ({alpha_score:.1f}) substantially "
                    f"outweighs benign explanation ({beta_score:.1f})"
                )
                final_verdicts.append({**alpha_f, 'verdict': 'SUSPICIOUS'})
            elif beta_score > alpha_score * 1.5:
                debate_entry['resolution'] = 'BETA_PREVAILS'
                debate_entry['reasoning'] = (
                    f"Benign explanation ({beta_score:.1f}) substantially "
                    f"outweighs attribution claim ({alpha_score:.1f})"
                )
                final_verdicts.append({**beta_counter, 'verdict': 'BENIGN'})
            else:
                debate_entry['resolution'] = 'UNRESOLVED'
                debate_entry['reasoning'] = (
                    f"Scores too close (α={alpha_score:.1f} vs β={beta_score:.1f}). "
                    f"Requires manual review. Flagged for analyst attention."
                )
                final_verdicts.append({
                    **alpha_f, 'verdict': 'REQUIRES_REVIEW',
                    'beta_counter': beta_counter
                })

            debate_log.append(debate_entry)
        else:
            # No counter-argument from beta
            final_verdicts.append({**alpha_f, 'verdict': 'UNCHALLENGED'})

    return {
        'verdicts': final_verdicts,
        'debate_log': debate_log,
        'council_alignment': len([d for d in debate_log if d['resolution'] != 'UNRESOLVED']) / max(len(debate_log), 1)
    }
```

---

## Phase 2: Reverse Socratic Trace — Local Process Drip Origin

Work BACKWARDS from the observed drip to its root cause on this machine.

### The Reverse Socratic Method

Start from the network observation and question backwards:

```
DRIP OBSERVED → What port? → What socket? → What PID? → What binary? 
→ What parent? → What triggered it? → WHY does it exist?
```

```python
import subprocess
import os

def reverse_socratic_trace(target_ip=None, target_port=None):
    """
    Trace from network drip backwards to root process.
    Each step asks: 'But WHY is this happening?'
    """
    trace = {'questions': [], 'answers': []}

    # Q1: What connections match the drip pattern?
    trace['questions'].append("What active connections match the drip destination?")
    ss_out = subprocess.run(
        ['ss', '-tupn', 'state', 'established'],
        capture_output=True, text=True
    ).stdout
    matching_conns = [l for l in ss_out.split('\n')
                      if (target_ip and target_ip in l) or
                         (target_port and f':{target_port}' in l)]
    trace['answers'].append(matching_conns)

    # Q2: What process owns these sockets?
    trace['questions'].append("What process owns these sockets?")
    pids = set()
    for conn in matching_conns:
        if 'pid=' in conn:
            pid = conn.split('pid=')[1].split(',')[0].split(')')[0]
            pids.add(pid)
    processes = {}
    for pid in pids:
        try:
            cmdline = Path(f'/proc/{pid}/cmdline').read_text().replace('\x00', ' ')
            exe = os.readlink(f'/proc/{pid}/exe')
            cwd = os.readlink(f'/proc/{pid}/cwd')
            processes[pid] = {'cmdline': cmdline, 'exe': exe, 'cwd': cwd}
        except (FileNotFoundError, PermissionError):
            processes[pid] = {'cmdline': 'ACCESS_DENIED', 'exe': 'unknown'}
    trace['answers'].append(processes)

    # Q3: What is the parent chain?
    trace['questions'].append("What spawned these processes? (parent chain)")
    for pid in pids:
        chain = get_process_chain(pid)
        trace['answers'].append({pid: chain})

    # Q4: What files are these processes touching?
    trace['questions'].append("What files/resources are these processes accessing?")
    for pid in pids:
        try:
            fds = subprocess.run(['ls', '-la', f'/proc/{pid}/fd'],
                               capture_output=True, text=True).stdout
            trace['answers'].append({pid: fds})
        except:
            pass

    # Q5: What is the actual payload content?
    trace['questions'].append("What data is actually being sent? (the drip substance)")
    # This comes from pcap payload analysis

    # Q6: WHY does this process need to send this data?
    trace['questions'].append(
        "Is this communication essential to the process's stated function, "
        "or is it extraneous/suspicious?"
    )

    return trace


def get_process_chain(pid):
    """Walk up the process tree to init."""
    chain = []
    current = pid
    while current and current != '1' and current != '0':
        try:
            stat = Path(f'/proc/{current}/stat').read_text()
            comm = Path(f'/proc/{current}/comm').read_text().strip()
            ppid = stat.split(')')[1].split()[1]
            chain.append({'pid': current, 'name': comm, 'ppid': ppid})
            current = ppid
        except (FileNotFoundError, PermissionError, IndexError):
            break
    return chain
```

### Drip Classification

After the Socratic trace, classify the drip:

| Drip Type | Description | Indicators |
|-----------|-------------|------------|
| **Telemetry Drip** | App/OS phoning home | Known endpoints, small payloads, periodic |
| **Sync Drip** | Cloud sync (Dropbox, iCloud, etc.) | Burst patterns, large payloads, known IPs |
| **Exfil Drip** | Data exfiltration | Unknown destinations, encoded payloads, stealth timing |
| **C2 Drip** | Command & control | Beaconing, small regular packets, encrypted |
| **Leak Drip** | Misconfigured service | Broadcast, unintended exposure |
| **Ghost Drip** | Orphaned/zombie process | No parent, stale connections |

---

## Phase 3: StarFire Report Generation — Tabbed HTML Website

Generate the final forensics-grade interactive report.

```python
def generate_starfire_report(lattice, council_results, socratic_trace, 
                             packets_analysis, output_path='starfire_report.html'):
    """Generate the full StarFire Forensics tabbed HTML report."""

    debate_html = render_debate_log(council_results['debate_log'])
    verdicts_html = render_verdicts(council_results['verdicts'])
    trace_html = render_socratic_trace(socratic_trace)
    network_html = render_network_analysis(packets_analysis)
    timeline_html = render_timeline(packets_analysis)

    html = STARFIRE_TEMPLATE.format(
        timestamp=lattice['created'],
        lattice_hash=lattice['lattice_hash'],
        evidence_count=len(lattice['evidence_items']),
        tab_overview=network_html,
        tab_offenders=verdicts_html,
        tab_debate=debate_html,
        tab_drip_trace=trace_html,
        tab_timeline=timeline_html,
        tab_payloads=render_payloads(packets_analysis),
        tab_processes=render_processes(socratic_trace),
        council_alignment=f"{council_results['council_alignment']*100:.0f}%"
    )

    Path(output_path).write_text(html)
    print(f"[STARFIRE] Report generated: {output_path}")
    return output_path
```

The HTML template is in `scripts/starfire_template.py`. Run:

```bash
python scripts/generate_report.py <pcap_file> [--output starfire_report.html]
```

---

## Execution Order (Full Workflow)

```
1. Locate PCAPs (Desktop or specified path)
2. CRYSTALLINE LATTICE CLAP — hash & seal evidence
3. Parse packets into DataFrame
4. Launch Council-α (attribution/malicious lens)
5. Launch Council-β (innocence/benign lens)
6. FINAL MAKER resolves conflicts, logs debate
7. REVERSE SOCRATIC TRACE on local machine
8. Classify drip type
9. Generate StarFire HTML report with all tabs
10. Verify lattice hashes unchanged (evidence integrity check)
```

## Report Tabs

The StarFire Report contains these tabs:

| Tab | Contents |
|-----|----------|
| **Overview** | Executive summary, key metrics, protocol breakdown |
| **Top Offenders** | Scored/flagged IPs with council verdicts |
| **Council Debate** | High-signal disagreements, reasoning, resolutions |
| **Drip Trace** | Reverse Socratic chain from network → process → root cause |
| **Timeline** | Traffic volume over time, burst detection |
| **Payloads** | Decoded payload samples, content classification |
| **Processes** | Local process map, parent chains, file access |
| **Evidence** | Lattice manifest, hash verification, chain of custody |

---

## Quick Start

```bash
# Full analysis with report
python ~/.cursor/skills/pcap-analysis/scripts/starfire_analyze.py ~/Desktop/*.pcap

# Just the quick CLI summary
python ~/.cursor/skills/pcap-analysis/scripts/analyze_pcap.py target.pcap

# Live capture + immediate forensics (requires root)
sudo tcpdump -i any -w /tmp/live.pcap -c 10000 && \
python ~/.cursor/skills/pcap-analysis/scripts/starfire_analyze.py /tmp/live.pcap
```
