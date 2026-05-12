---
name: pcap-analysis
description: "LATTICE TEAM OPERATION — KHET-1: Forensic PCAP analysis with dual counter-rotating councils (Red/Blue adversarial swarm), SHA-256 Merkle-connected evidence chains, Rayon parallel execution, reverse Socratic drip tracing, and StarFire tabbed HTML report. Outposts → Castles → Cathedral architecture. Use when analyzing pcap files, investigating network drip/leaks, identifying root offenders, or performing local forensics on this machine."
disable-model-invocation: true
---

# LATTICE TEAM OPERATION — KHET-1

## Architecture

```
╔══════════════════════════════════════════════════════════════╗
║           LATTICE TEAM — SWARM TOPOLOGY                      ║
╠══════════════════════════════════════════════════════════════╣
║                                                              ║
║   ┌──────────────┐          MERKLE          ┌────────────┐  ║
║   │  RED COUNCIL │◄────── SHA-256 ──────────►│ BLUE COUNCIL│ ║
║   │  (↺ Counter) │         TREE              │ (↻ Clock)  │  ║
║   │              │           │               │             │  ║
║   │  Workers x N │           │               │  Workers x N│  ║
║   │  (Rayon)     │           │               │  (Rayon)    │  ║
║   └──────┬───────┘           │               └──────┬──────┘  ║
║          │                   │                      │         ║
║          ▼                   ▼                      ▼         ║
║   ┌──────────┐       ┌────────────┐         ┌──────────┐    ║
║   │ OUTPOSTS │──────►│   CASTLES  │◄────────│ OUTPOSTS │    ║
║   └──────────┘       └─────┬──────┘         └──────────┘    ║
║                             │                                 ║
║                      ┌──────▼───────┐                        ║
║                      │  FINAL MAKER │                        ║
║                      │  (Opus 4.6)  │                        ║
║                      └──────┬───────┘                        ║
║                             │                                 ║
║                      ┌──────▼───────┐                        ║
║                      │  CATHEDRAL   │                        ║
║                      │  (Report)    │                        ║
║                      └──────────────┘                        ║
╚══════════════════════════════════════════════════════════════╝
```

## Core Principles

| Principle | Implementation |
|-----------|---------------|
| **All tokens earn their keep** | Every worker produces outposts. No idle compute. |
| **Merkle connects all** | Every outpost, castle, debate → SHA-256 leaf in Merkle tree |
| **Rayon parallel** | ThreadPoolExecutor splits work across cores |
| **Diamond hand** | Evidence never modified. Read-only. Sealed on first touch. |
| **Outposts → Castles → Cathedral** | Progressive intelligence refinement |

## Execution

### Quick Start — Full Operation

```bash
# Auto-capture from local box + analyze
python ~/.cursor/skills/pcap-analysis/scripts/lattice_team.py --output ./report

# Analyze existing pcaps (no live capture)
python ~/.cursor/skills/pcap-analysis/scripts/lattice_team.py --pcap ~/Desktop/*.pcap --no-capture

# Both: capture + existing files
python ~/.cursor/skills/pcap-analysis/scripts/lattice_team.py --pcap /tmp/evidence.pcap --output ./forensics
```

### Outputs

| File | Contents |
|------|----------|
| `lattice_starfire_report.html` | Full tabbed interactive report |
| `lattice_report.json` | Machine-readable structured data |
| `merkle_manifest.json` | Complete Merkle tree with all leaf hashes |

---

## Phase Breakdown

### Phase 0: Crystalline Lattice Clap
- SHA-256 + MD5 hash every evidence file
- Create Merkle leaf for each pcap
- Enforce read-only (diamond hand)
- Generate lattice manifest

### Phase 1: Parse (Rayon)
- Load packets via pyshark
- Extract: IP src/dst, ports, protocols, payloads
- Add parse summary to Merkle tree

### Phase 2: Dual Council Swarm

**Red Council (↺ Counter-Clockwise — Attribution)**
- Assumes malicious intent
- Detects: beaconing, volume anomalies, unusual ports, DNS tunneling, high-entropy payloads
- Each worker chunk produces outposts independently
- Aggregator builds castles from correlated outposts

**Blue Council (↻ Clockwise — Innocence)**  
- Assumes legitimate behavior
- Identifies: known telemetry, standard HTTPS, keepalives, infrastructure traffic, low volume
- Same parallel architecture, opposing lens
- Aggregator builds defensive castles

### Phase 3: Castle Construction
- Related outposts merge into castles
- Each castle = fortified conclusion with evidence chain
- Merkle leaf for every castle

### Phase 4: Final Maker (Opus 4.6)
- Pits red castles against blue castles
- Confidence-weighted resolution
- Logs high-signal debate points
- Resolution types: RED_PREVAILS, BLUE_PREVAILS, CONTESTED

### Phase 5: Cathedral
- All castles assembled into unified intelligence product
- Drip classified (C2_BEACON, DATA_EXFIL, TELEMETRY, etc.)
- Source process identified (when live)
- HTML report generated

### Phase 6: Output + Verify
- Generate HTML (StarFire tabbed report)
- Generate JSON (machine-readable)
- Generate Merkle manifest
- Verify all hashes intact

---

## Merkle Tree Structure

```
                    [ROOT HASH]
                   /           \
          [Internal]           [Internal]
         /         \          /          \
    [evidence:     [evidence:  [outpost:    [outpost:
     file1.pcap]   file2.pcap]  RED-0-0]    BLUE-0-0]
                                    ...
    [castle:       [castle:    [debate:
     RED-BEACON]   BLUE-HTTPS]  resolution]
```

Every node SHA-256 hashed. Parent = SHA-256(left.hash + right.hash).
Proof of inclusion available for any leaf via `merkle.get_proof(index)`.

---

## Drip Classification Table

| Classification | Red Indicators | Blue Counter |
|---|---|---|
| **C2_BEACON** | Regular intervals, small payloads, non-standard ports | Keepalive, heartbeat |
| **DATA_EXFIL** | Volume concentration, high entropy, single dest >40% | Cloud sync, backup |
| **DNS_TUNNEL** | Oversized DNS payloads (>50B avg) | Complex legitimate queries |
| **PORT_HOPPING** | Many non-standard ports, low per-port volume | Service mesh, microservices |
| **TELEMETRY** | Periodic small sends to unknown IPs | Known telemetry endpoints |
| **LIKELY_BENIGN** | — | HTTPS dominant, low volume, known infra |

---

## Report Tabs (StarFire HTML)

| Tab | Contents |
|-----|----------|
| **Overview** | Metrics, drip classification, rayon efficiency |
| **Top Offenders** | Final castles with severity + confidence |
| **Council Debate** | Red vs Blue exchanges, Final Maker resolutions |
| **Drip Trace** | Reverse Socratic: What → Where → Who → Why |
| **Timeline** | Protocol distribution, traffic profile |
| **Payloads** | Decoded payload samples from captured packets |
| **Processes** | Live `ss -tupn` local process state |
| **Evidence Chain** | Merkle tree leaves, root hash, integrity status |

---

## Dependencies

```bash
pip install pyshark pandas
sudo apt install tcpdump tshark
```

## File Structure

```
pcap-analysis/
├── SKILL.md                     (this file)
└── scripts/
    ├── lattice_team.py          (MAIN — full Lattice Team operation)
    ├── starfire_analyze.py      (standalone StarFire analyzer)
    ├── starfire_template.py     (HTML template)
    └── analyze_pcap.py          (quick CLI summary)
```
