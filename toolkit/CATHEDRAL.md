# The Cathedral — Reverse Socratic Architecture

What does the finished forensics operation look like, and what bricks build it?

## The Cathedral (End State)

```
You can hand someone a single link, a single file, or a single command.
They get: complete evidence, chain of custody, analysis, narrative, redaction.
Court-ready. Client-ready. Zero questions about provenance.
Independently verifiable. With or without AI. Your choice per engagement.
```

## Reverse Socratic: Questioning Backwards from the Cathedral

### Q: What does the client/court need to see?
A: A report that says: here's what happened, here's the proof, here's who verified it, here's the hash chain proving nothing was altered.

### Q: What proves nothing was altered?
A: SHA-256 hash of every file, sealed at time of first access. Merkle tree linking all hashes. Git commit history as second anchor. Vault seal as third.

### Q: What generates that report?
A: The toolkit. One command. Offline or online. Their choice.

### Q: What feeds the toolkit?
A: Evidence. PCAPs from phones, laptops, routers. Connection logs. Process maps.

### Q: What collects that evidence?
A: Atomic agents. Each does one thing. Composable. Phone agent. Desktop agent. Router agent.

### Q: What ties the agents together?
A: The Firestar endpoint (optional). Or a USB stick. Or a local network share. Or nothing — each stands alone.

### Q: What if they need explanation, not just data?
A: Local LLM generates narrative. Or they hire an analyst. Or the report is self-explanatory.

### Q: What if names can't be known?
A: Redaction engine. Maps real values to anonymous labels. Configurable per engagement.

### Q: What if the phone has no root?
A: /proc/net/tcp is readable without root. UID→package mapping works without root. Connection polling works without root. Only raw pcap needs root (or PCAPdroid app).

### Q: What if there's no internet?
A: Everything works offline. Upload is opt-in. Analysis is local. Report generates locally.

---

## The Bricks (Atomic Tools)

Each tool does ONE thing. They compose into the cathedral.

| Tool | Function | Platform | Needs Internet | Needs Root | Needs AI |
|------|----------|----------|----------------|------------|----------|
| `capture-connections` | Snapshot /proc/net/tcp,udp → JSON | Android/Linux | No | No | No |
| `capture-pcap` | tcpdump wrapper with auto-seal | Android/Linux/Mac | No | Yes | No |
| `capture-dns` | Log DNS queries from /proc or tcpdump | Any | No | Varies | No |
| `capture-processes` | Map PIDs to network sockets | Android/Linux | No | No | No |
| `capture-apps` | UID→package name resolution | Android | No | No | No |
| `capture-polling` | Time-series connection sampling | Any | No | No | No |
| `analyze-beacons` | Detect regular-interval traffic | Any | No | No | No |
| `analyze-destinations` | Cluster/classify destination IPs | Any | No | No | No |
| `analyze-correlate` | Cross-device shared destination finding | Any | No | No | No |
| `analyze-tls-sni` | Extract server names from TLS ClientHello | Any | No | No | No |
| `seal-evidence` | SHA-256 hash + Merkle tree | Any | No | No | No |
| `seal-vault` | Generate vault seal over all artifacts | Any | No | No | No |
| `redact` | Apply redaction map to any output | Any | No | No | No |
| `report-html` | Generate self-contained HTML report | Any | No | No | No |
| `report-pdf` | Convert HTML to PDF (via wkhtmltopdf) | Any | No | No | No |
| `narrative-local` | Generate plain-language via Ollama | Any | No | No | Local only |
| `upload` | POST results to Firestar (OPT-IN) | Any | Yes | No | No |

## What Else Should You Think About

### Evidence Handling
- **First-touch hash**: the MOMENT a file enters your custody, hash it. Before anything else.
- **Read-only mount**: if analyzing from USB/SD, mount it read-only (`mount -o ro`).
- **No analysis on originals**: copy first, analyze the copy, keep original sealed.

### Temporal Correlation
- **NTP sync all devices** before capture. If timestamps don't align, correlation fails.
- **Record timezone** of each capture explicitly. UTC everywhere.
- **Wall-clock photo**: photograph a clock next to the device at capture start (physical timestamp anchor).

### Multi-Device Correlation
- Same destination across N devices = shared service/app
- Same timing across N devices = centralized push/config
- Unique destination on ONE device = investigate that device specifically
- Build the correlation matrix EVERY TIME: devices × destinations × timing

### Legal Posture
- **Offline mode** means you can truthfully state: "No AI was involved in the analysis"
- **Local LLM mode** means: "AI-assisted narrative was generated locally with no data transmission"
- **Chain of custody** file is your attestation that evidence wasn't tampered with
- **Vault seal** is the single hash a court can verify: if it matches, nothing changed
- **Redaction** protects uninvolved parties — you control what names appear

### Operational Security
- Agents are disposable. Run, collect, delete.
- No persistent connections from agents to any server (fire-and-forget POST or no POST at all)
- Local analysis leaves no cloud trace
- Git history proves WHEN analysis was done (timestamped commits)

### What This Doesn't Do (Scope Limits)
- Cannot break TLS 1.3 (PFS prevents passive decryption — need SSLKEYLOGFILE from the device during capture)
- Cannot identify processes on iOS without jailbreak or Mac+USB
- Cannot capture baseband/carrier-level signaling (that's below the OS)
- Cannot attribute traffic to a specific user action (only to an app/process)
- PCAPdroid is the closest to "full capture without root" on Android — recommend it for sustained monitoring

---

## Cathedral Assembly Order

```
FOUNDATION:
  1. Atomic capture tools (each phone gets: connections + apps + polling)
  2. Evidence sealing (SHA-256 on first touch)
  3. Chain of custody (custody.json per capture)

WALLS:
  4. Analysis engine (beacons, destinations, ports, DNS, TLS SNI)
  5. Correlation engine (cross-device matching)
  6. Redaction engine (anonymize before output)

ROOF:
  7. Report generator (HTML, self-contained)
  8. Vault seal (single hash covers everything)
  9. Optional narrative (local LLM or manual)

SPIRE:
  10. Delivery (link, file, USB, print — your choice per engagement)
```
