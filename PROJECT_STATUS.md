# LATTICE TEAM — KHET-1 — Project Status Report

**Date:** 13 May 2026  
**Operation:** LATTICE TEAM KHET-1  
**Repository:** github.com/KHET-1/KHET-1  
**Branch:** cursor/starfire-forensics-skill-051a  
**Live Domain:** firestar-defense.vercel.app  
**Commits:** 20  
**Files:** 58  
**Lines of Code:** ~29,900  
**Tests:** 6/6 passing  

---

## 1. System Inventory

### 1.1 Toolkit (Offline/Airgap — `toolkit/`)

| Module | File | LOC | Status | Function |
|--------|------|-----|--------|----------|
| CLI | `starfire` | 157 | Working | Main entrypoint: analyze, full, seal, verify |
| Merkle | `merkle/tree.py` | 157 | Tested (6/6) | SHA-256 tree, proofs, manifests, verify |
| Merkle Utils | `merkle/utils.py` | 22 | Working | hash_file, hash_string, verify_file |
| Evidence | `evidence/sealer.py` | 119 | Working | First-touch seal, chain of custody, verify_all |
| Red Council | `councils/red.py` | 123 | Working | Beaconing, volume, port scan, DNS tunnel |
| Blue Council | `councils/blue.py` | 120 | Working | HTTPS normal, infra, low vol, keepalives |
| Final Maker | `councils/final_maker.py` | 115 | Working | Conflict resolution, debate log, alignment |
| Parser | `parser/pcap_parser.py` | 75 | Working | pcap/pcapng → DataFrame |
| Report | `report/starfire.py` | 397 | Working | 10-tab HTML, JSON, Merkle, zero CDN |
| Mobile | `mobile/starfire-mobile.sh` | 200 | Working | Termux capture/analyze/upload |
| Config | `config.toml` | 30 | Reference | All settings, network blocked by default |
| Tests | `tests/test_merkle.py` | 95 | 6/6 passing | Merkle tree verification suite |
| Architecture | `CATHEDRAL.md` | 140 | Reference | Reverse Socratic design document |

**Total toolkit Python:** ~1,300 LOC (clean, modular, no duplication)

### 1.2 Website (Live — `public/`)

| Page | URL Path | Function |
|------|----------|----------|
| Lattice IDE | `/` | Main dashboard: fold7 sections, topology graph, tracked drip, 8 tabs |
| Formal Report | `/report-formal.html` | Court-ready, print CSS, triple-anchored, signature block |
| StarFire Report | `/starfire_report_final.html` | Interactive tabbed analysis (8 tabs + evidence) |
| Agents Hub | `/agents/` | Download page: Android, iOS, Linux agents with one-liners |
| Results | `/agents/results.html` | Auto-refresh (10s) incoming capture dashboard |
| Correlation | `/agents/correlation.html` | Multi-device drip correlation methodology |
| Infrastructure | `/agents/infrastructure.html` | Network setup guide, triangulation, Android vs iOS matrix |
| Vault Seal | `/vault-seal.json` | Machine-verifiable artifact registry |
| API | `/api/ingest` | POST captures / GET results |

**All pages:** 200 OK. Zero external CDN dependencies. Work fully offline (save HTML).

### 1.3 Evidence (Analyzed — `evidence/`)

| File | Size | Packets | SHA-256 (prefix) |
|------|------|---------|------------------|
| ppppppppppppppp.pcapng | 12.98 MB | 6,323 | 0e0764b8... |
| 2.pcapng | 12.69 MB | 2,925 | b6ea0520... |
| 1.pcapng | 1.29 MB | 280 | 035d86fc... |

**Merkle Root:** 7a2631ab7394377a7997f6ddad26c32812101c2f  
**Vault Seal:** 172c29fa04fe5559048edf9f021f2f2ce8441a677ff193e7e026b3f09d2baa37

### 1.4 Cursor Skill (`.cursor/skills/pcap-analysis/`)

| File | LOC | Purpose |
|------|-----|---------|
| SKILL.md | 170 | Skill specification for Cursor agent invocation |
| scripts/lattice_team.py | 1,141 | Original monolithic swarm engine |
| scripts/starfire_analyze.py | 893 | Standalone StarFire analyzer |
| scripts/starfire_template.py | 457 | HTML template |
| scripts/analyze_pcap.py | 197 | Quick CLI summary tool |

---

## 2. Capabilities Matrix

| Capability | Offline | Local LLM | Cloud | Status |
|------------|---------|-----------|-------|--------|
| PCAP parsing | Yes | Yes | Yes | Working |
| Connection mapping | Yes | Yes | Yes | Working |
| Beacon detection | Yes | Yes | Yes | Working |
| Dual-council analysis | Yes | Yes | Yes | Working |
| Merkle evidence chain | Yes | Yes | Yes | Tested |
| SHA-256 vault seal | Yes | Yes | Yes | Working |
| HTML report generation | Yes | Yes | Yes | Working |
| Redaction (IP/name) | Yes | Yes | Yes | Working |
| Plain-language narrative | No | Yes (Ollama) | Yes | Hook ready |
| Phone capture (Android) | Yes | — | Optional upload | Working |
| Phone capture (iOS) | Yes | — | Optional upload | Working |
| Multi-device correlation | Yes | Yes | Yes | Guide ready |
| Results ingestion API | — | — | Yes | Working (Vercel) |
| Auto-refresh dashboard | — | — | Yes | Working |
| Court-ready formal report | Yes | Yes | Yes | Working |
| Print/PDF export | Yes | Yes | Yes | Browser print |

---

## 3. What Was Accomplished

1. **Identified network drip** from 3 pcap files (9,528 packets): Python 3.13 polling script via gvfsd-http, targeting Mac Mini services on ports 7734/8082.

2. **Built dual-council adversarial analysis** (Red vs Blue) with Final Maker resolution. Council alignment: 100%.

3. **Established triple-anchored chain of custody**: SHA-256 + Git + Merkle tree. Vault sealed.

4. **Created standalone toolkit** that runs fully offline with zero AI, zero network, zero telemetry. Court-safe by default.

5. **Deployed live website** at firestar-defense.vercel.app with formal report, interactive dashboard, agent distribution, and results collection.

6. **Built phone forensics agents** (Android/iOS) that self-capture and optionally POST to cloud.

7. **Documented architecture** (CATHEDRAL.md) with reverse Socratic reasoning from end-state backwards to atomic bricks.

8. **Integrated Grok thread findings** confirming drip sources (Claude CLI, Cursor IDE, Python polling).

---

## 4. Grade

| Dimension | Score | Notes |
|-----------|-------|-------|
| **Functionality** | 9/10 | Full pipeline works end-to-end. Missing: PDF export (browser-dependent), deep payload decode. |
| **Architecture** | 9/10 | Clean modular design. Each module independent. Cathedral doc solid. Slight coupling between old monolithic scripts and new toolkit. |
| **Evidence Integrity** | 10/10 | Triple-anchored. Merkle tree. Vault seal. Chain of custody. First-touch hashing. Read-only enforcement. |
| **Court Readiness** | 9/10 | Formal report clean. Methodology documented. Redaction working. Missing: digital signature (GPG) on vault seal for non-repudiation. |
| **Offline/Airgap** | 10/10 | Fully functional without network. Confirmed 0 network calls. Config blocks outbound by default. |
| **Mobile** | 7/10 | Android agent works in Termux. iOS requires Mac+USB or jailbreak (platform limitation). No native app. |
| **Test Coverage** | 6/10 | Merkle tested (6/6). Other modules tested via integration but lack unit tests. |
| **Documentation** | 8/10 | CATHEDRAL.md, README, inline docs, correlation guide. Could use: API docs, contribution guide. |
| **UX/Presentation** | 8/10 | Dark theme IDE, fold7, topology graph. Formal report is print-ready. Could use: Plotly charts, PDF auto-generation. |
| **Operational Security** | 9/10 | No telemetry. No cloud in default mode. Redaction engine. Agents are fire-and-forget. One gap: Vercel token was used in-session (now expired). |

**Overall: 8.5/10**

---

## 5. Recommendations

### Immediate (High Value, Low Effort)

1. **Add GPG signing to vault seal** — makes the seal non-repudiable. One command: `gpg --sign vault-seal.json`. A court can verify the analyst's identity signed it.

2. **Add unit tests for councils and parser** — `test_red.py`, `test_blue.py`, `test_parser.py`. Each council should have test cases with known-bad and known-good traffic.

3. **Remove `__pycache__` from git** — add to .gitignore. These are build artifacts.

4. **Consolidate old scripts** — `.cursor/skills/pcap-analysis/scripts/` contains the monolithic prototypes. These should either be deprecated (moved to `legacy/`) or deleted now that `toolkit/` is the canonical implementation.

### Medium-Term (Cathedral Completion)

5. **Report PDF generation** — Add `wkhtmltopdf` or `weasyprint` for `--report pdf`. Currently requires browser print.

6. **Plotly/chart integration** — Timeline visualization, port heatmap, traffic volume chart. Embed as base64 images in HTML for airgap compat.

7. **Payload decoder** — Expand parser to extract HTTP bodies, decode base64/gzip, identify file types (magic bytes).

8. **Multi-file correlation engine** — `correlate.py` module that takes multiple pcap DataFrames and identifies shared destinations, synchronized timing, cross-device patterns.

9. **Local LLM model recommendations** — Test and document which Ollama models give best forensics narratives (llama3.1:8b vs mistral vs codellama). Include model-specific prompt tuning.

10. **Android native wrapper** — Package starfire-mobile.sh as a Termux Boot script or Tasker integration for scheduled automated captures.

### Long-Term (Scaling)

11. **Persistent results store** — Replace Vercel /tmp with Vercel Blob or KV for results that survive cold starts.

12. **Multi-analyst workflow** — Multiple analysts can submit findings, Merkle tree grows, vault seal updates. Needs conflict resolution for concurrent submissions.

13. **Encrypted evidence transport** — Age/GPG encrypt evidence bundles for transfer between airgapped analysis box and cloud reporting.

14. **Compliance mapping** — Map findings to NIST, MITRE ATT&CK, ISO 27001 controls. Auto-tag each finding with relevant framework references.

15. **Automated scheduled phone capture** — Cron-style repeated captures that build longitudinal data for trend analysis (is the drip increasing? new destinations appearing?).

---

## 6. Tool Analysis Summary

| Tool/Component | Maturity | Keep/Replace/Improve |
|---------------|----------|---------------------|
| `toolkit/starfire` (CLI) | Production-ready | **Keep** — wire remaining features |
| `toolkit/merkle/` | Production-ready | **Keep** — add proof verification tests |
| `toolkit/councils/` | Functional | **Improve** — more detection signatures, test suite |
| `toolkit/parser/` | Functional | **Improve** — add payload extraction, DNS parsing |
| `toolkit/report/` | Functional | **Improve** — Plotly charts, PDF output |
| `toolkit/evidence/` | Production-ready | **Keep** — add GPG signing |
| `toolkit/mobile/` | Beta | **Improve** — add iOS Shortcuts integration, Tasker auto |
| `.cursor/skills/` (old scripts) | Legacy | **Deprecate** — toolkit/ is the canonical source |
| `public/` (website) | Live | **Improve** — persistent storage, real-time charts |
| `api/ingest.js` | Functional | **Improve** — persistent store, auth, rate limiting |
| Vercel deployment | Live | **Keep** — add custom domain if available |

---

*Report generated by LATTICE Team oversight process. Vault integrity verified.*
