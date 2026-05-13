#!/data/data/com.termux/files/usr/bin/bash
# ╔══════════════════════════════════════════════════════════════╗
# ║  STARFIRE MOBILE — Forensics Toolkit for Android (Termux)   ║
# ║  Fully offline. Atomic tools. No cloud required.            ║
# ╚══════════════════════════════════════════════════════════════╝
#
# Install in Termux:
#   pkg install curl && curl -sL https://firestar-defense.vercel.app/agents/starfire-mobile.sh -o ~/starfire && chmod +x ~/starfire
#
# Or airgap install: copy this file to phone via USB/adb
#
# MODES:
#   ~/starfire capture    — capture connections + optional pcap
#   ~/starfire analyze    — analyze a local pcap (offline)
#   ~/starfire upload     — send results to Firestar (optional, needs internet)
#   ~/starfire tools      — install/check all atomic tools
#   ~/starfire full       — capture + analyze + seal (one shot)

set -e

VERSION="1.0.0"
TOOLKIT_DIR="${HOME}/.starfire"
EVIDENCE_DIR="${TOOLKIT_DIR}/evidence"
REPORTS_DIR="${TOOLKIT_DIR}/reports"
TOOLS_DIR="${TOOLKIT_DIR}/tools"

mkdir -p "$TOOLKIT_DIR" "$EVIDENCE_DIR" "$REPORTS_DIR" "$TOOLS_DIR"

# ── ATOMIC TOOLS (each does one thing, composable) ──

install_tools() {
    echo "[TOOLS] Checking/installing atomic components..."
    
    # Core (always needed)
    pkg install -y python 2>/dev/null || true
    pkg install -y openssl-tool 2>/dev/null || true  # for sha256
    
    # Optional capture tools
    pkg install -y tcpdump 2>/dev/null || echo "  tcpdump: needs root-repo (pkg install root-repo first)"
    pkg install -y nmap 2>/dev/null || true
    pkg install -y termux-api 2>/dev/null || true  # for device info
    
    # Optional analysis
    pip install pyshark 2>/dev/null || true
    pip install pandas 2>/dev/null || true
    
    # Optional local AI
    echo "  Local LLM: install Ollama via proot-distro if needed"
    
    echo "[TOOLS] Done. Run './starfire tools check' to verify."
}

check_tools() {
    echo "╔══════════════════════════════════════════════════════════╗"
    echo "║  STARFIRE MOBILE — Tool Status                          ║"
    echo "╚══════════════════════════════════════════════════════════╝"
    echo ""
    
    check() {
        if command -v "$1" &>/dev/null; then
            echo "  [OK]  $1 $(command -v $1)"
        else
            echo "  [--]  $1 (not installed: $2)"
        fi
    }
    
    echo "  CORE:"
    check python3 "pkg install python"
    check openssl "pkg install openssl-tool"
    check sha256sum "pkg install coreutils"
    
    echo ""
    echo "  CAPTURE:"
    check tcpdump "pkg install root-repo && pkg install tcpdump"
    check nmap "pkg install nmap"
    check termux-wifi-connectioninfo "pkg install termux-api"
    
    echo ""
    echo "  ANALYSIS:"
    python3 -c "import pyshark" 2>/dev/null && echo "  [OK]  pyshark" || echo "  [--]  pyshark (pip install pyshark)"
    python3 -c "import pandas" 2>/dev/null && echo "  [OK]  pandas" || echo "  [--]  pandas (pip install pandas)"
    
    echo ""
    echo "  OPTIONAL:"
    check ollama "See: https://ollama.com (via proot-distro)"
    check curl "pkg install curl"
    check tshark "pkg install tshark"
}

# ── CAPTURE (atomic) ──

do_capture() {
    DURATION=${1:-30}
    TIMESTAMP=$(date -u +%Y%m%d_%H%M%S)
    CAPTURE_ID="mob_${TIMESTAMP}"
    OUTDIR="${EVIDENCE_DIR}/${CAPTURE_ID}"
    mkdir -p "$OUTDIR"
    
    echo "╔══════════════════════════════════════════════════════════╗"
    echo "║  CAPTURE — ${DURATION}s — ID: ${CAPTURE_ID}             "
    echo "╚══════════════════════════════════════════════════════════╝"
    echo ""
    
    # Device info
    echo "[1] Device identity..."
    python3 -c "
import json, subprocess, os

info = {
    'capture_id': '$CAPTURE_ID',
    'timestamp': '$(date -u +%Y-%m-%dT%H:%M:%SZ)',
    'kernel': os.uname().release,
    'hostname': os.uname().nodename
}

# Android props
props = ['ro.product.model', 'ro.build.version.release', 'gsm.operator.alpha', 'ro.serialno']
for p in props:
    try:
        r = subprocess.run(['getprop', p], capture_output=True, text=True, timeout=3)
        info[p.split('.')[-1]] = r.stdout.strip()
    except: pass

# Network type
try:
    r = subprocess.run(['termux-wifi-connectioninfo'], capture_output=True, text=True, timeout=5)
    info['wifi'] = json.loads(r.stdout) if r.returncode == 0 else None
except: pass

with open('$OUTDIR/device.json', 'w') as f:
    json.dump(info, f, indent=2)
print(f\"  Model: {info.get('model','?')} | Android: {info.get('release','?')} | Carrier: {info.get('alpha','?')}\")
" 2>/dev/null || echo "  (basic mode)"
    
    # Connections snapshot
    echo "[2] Connection map..."
    python3 << 'PYEOF' 2>/dev/null
import os, json

def parse_proc_net(path, proto):
    conns = []
    try:
        with open(path) as f:
            for line in f.readlines()[1:]:
                parts = line.strip().split()
                if len(parts) < 10: continue
                local_hex, local_port_hex = parts[1].split(':')
                remote_hex, remote_port_hex = parts[2].split(':')
                
                local_ip = '.'.join(str(int(local_hex[i:i+2], 16)) for i in range(6,-2,-2))
                remote_ip = '.'.join(str(int(remote_hex[i:i+2], 16)) for i in range(6,-2,-2))
                local_port = int(local_port_hex, 16)
                remote_port = int(remote_port_hex, 16)
                state = int(parts[3], 16)
                uid = int(parts[7])
                
                if remote_port == 0: continue
                conns.append({
                    'proto': proto, 'uid': uid, 'state': state,
                    'local': f'{local_ip}:{local_port}',
                    'remote': f'{remote_ip}:{remote_port}'
                })
    except: pass
    return conns

# Map UIDs to packages
uid_pkg = {}
try:
    import subprocess
    r = subprocess.run(['pm', 'list', 'packages', '-U'], capture_output=True, text=True, timeout=10)
    for line in r.stdout.split('\n'):
        if 'uid:' in line:
            pkg = line.split(':')[1].split(' ')[0]
            uid = int(line.split('uid:')[1].strip())
            uid_pkg[uid] = pkg
except: pass

conns = parse_proc_net('/proc/net/tcp', 'tcp')
conns += parse_proc_net('/proc/net/tcp6', 'tcp6')
conns += parse_proc_net('/proc/net/udp', 'udp')

# Filter loopback, enrich with app name
external = []
for c in conns:
    if c['remote'].startswith('127.') or c['remote'].startswith('0.0.0.'): continue
    c['app'] = uid_pkg.get(c['uid'], f"uid:{c['uid']}")
    external.append(c)

with open(os.environ.get('OUTDIR', '/tmp') + '/connections.json', 'w') as f:
    json.dump(external, f, indent=2)
print(f"  {len(external)} external connections mapped to {len(set(c['app'] for c in external))} apps")
PYEOF
    
    # Connection polling (beacon detection)
    echo "[3] Polling (${DURATION}s)..."
    python3 -c "
import time, json, os

samples = []
start = time.time()
while time.time() - start < $DURATION:
    remotes = set()
    try:
        with open('/proc/net/tcp') as f:
            for line in f.readlines()[1:]:
                parts = line.strip().split()
                if len(parts) > 3 and parts[3] == '01':
                    remotes.add(parts[2])
    except: pass
    samples.append({'t': round(time.time()-start,1), 'count': len(remotes), 'endpoints': list(remotes)[:10]})
    time.sleep(2)

with open('$OUTDIR/polling.json', 'w') as f:
    json.dump(samples, f)
print(f'  {len(samples)} samples')
" 2>/dev/null || echo "  (polling skipped)"
    
    # Packet capture (if available)
    echo "[4] Packet capture..."
    if command -v tcpdump &>/dev/null; then
        timeout $DURATION tcpdump -i any -c 3000 -s 128 -w "$OUTDIR/capture.pcap" 2>/dev/null || true
        echo "  pcap: $(wc -c < "$OUTDIR/capture.pcap" 2>/dev/null || echo 0) bytes"
    else
        echo "  (no tcpdump — connection map only)"
    fi
    
    # Seal evidence
    echo "[5] Sealing..."
    python3 -c "
import hashlib, json, os
from pathlib import Path

entries = []
for f in Path('$OUTDIR').iterdir():
    if f.is_file():
        h = hashlib.sha256(f.read_bytes()).hexdigest()
        entries.append({'file': f.name, 'sha256': h, 'size': f.stat().st_size})

seal_content = json.dumps(entries, sort_keys=True)
vault_hash = hashlib.sha256(seal_content.encode()).hexdigest()

manifest = {
    'capture_id': '$CAPTURE_ID',
    'sealed_at': '$(date -u +%Y-%m-%dT%H:%M:%SZ)',
    'vault_seal': vault_hash,
    'network_calls': 0,
    'ai_calls': 0,
    'entries': entries
}
with open('$OUTDIR/custody.json', 'w') as f:
    json.dump(manifest, f, indent=2)
print(f'  VAULT SEAL: {vault_hash[:32]}...')
print(f'  Files: {len(entries)} | Network calls: 0 | AI calls: 0')
" 2>/dev/null
    
    echo ""
    echo "Evidence: $OUTDIR/"
    ls -la "$OUTDIR/" 2>/dev/null
}

# ── ANALYZE (atomic, offline) ──

do_analyze() {
    TARGET=${1:-$(ls -td ${EVIDENCE_DIR}/mob_* 2>/dev/null | head -1)}
    
    if [ -z "$TARGET" ] || [ ! -d "$TARGET" ]; then
        echo "No evidence to analyze. Run: ./starfire capture first"
        exit 1
    fi
    
    echo "╔══════════════════════════════════════════════════════════╗"
    echo "║  ANALYZE — Offline — $(basename $TARGET)"
    echo "╚══════════════════════════════════════════════════════════╝"
    echo ""
    
    python3 << PYEOF
import json, os
from collections import Counter
from pathlib import Path

target = "$TARGET"
conns_file = os.path.join(target, 'connections.json')
poll_file = os.path.join(target, 'polling.json')

print("=== CONNECTION ANALYSIS ===\n")

if os.path.exists(conns_file):
    conns = json.load(open(conns_file))
    
    # Top destinations
    dsts = Counter(c['remote'].split(':')[0] for c in conns)
    print("Top Destinations:")
    for ip, cnt in dsts.most_common(10):
        print(f"  {ip:<20} {cnt} connections")
    
    # Top apps
    apps = Counter(c.get('app','?') for c in conns)
    print("\nTop Apps (by connection count):")
    for app, cnt in apps.most_common(10):
        print(f"  {app:<45} {cnt}")
    
    # Top ports
    ports = Counter(int(c['remote'].split(':')[1]) for c in conns)
    svc = {443:'HTTPS',80:'HTTP',53:'DNS',5228:'GCM-Push',5222:'XMPP',8443:'HTTPS-Alt'}
    print("\nTop Ports:")
    for port, cnt in ports.most_common(10):
        print(f"  :{port:<6} ({svc.get(port,'?'):<10}) {cnt}")
    
    print(f"\nTotal: {len(conns)} connections from {len(apps)} apps to {len(dsts)} destinations")
else:
    print("  No connections.json found")

# Beacon detection from polling
print("\n=== BEACON DETECTION ===\n")
if os.path.exists(poll_file):
    samples = json.load(open(poll_file))
    # Track endpoints appearing across multiple samples
    endpoint_freq = Counter()
    for s in samples:
        for ep in s.get('endpoints', []):
            endpoint_freq[ep] += 1
    
    persistent = {ep: cnt for ep, cnt in endpoint_freq.items() if cnt > len(samples) * 0.7}
    if persistent:
        print(f"Persistent connections (present in >70% of samples):")
        for ep, cnt in sorted(persistent.items(), key=lambda x: -x[1])[:10]:
            print(f"  {ep} — present in {cnt}/{len(samples)} samples")
    else:
        print("  No persistent beaconing detected")
else:
    print("  No polling data")

print("\n=== EVIDENCE INTEGRITY ===\n")
custody_file = os.path.join(target, 'custody.json')
if os.path.exists(custody_file):
    custody = json.load(open(custody_file))
    print(f"Vault Seal: {custody['vault_seal']}")
    print(f"Network calls: {custody['network_calls']}")
    print(f"AI calls: {custody['ai_calls']}")
    for e in custody['entries']:
        print(f"  {e['file']:<25} {e['sha256'][:24]}... ({e['size']} bytes)")
PYEOF
}

# ── UPLOAD (optional — only if you choose) ──

do_upload() {
    TARGET=${1:-$(ls -td ${EVIDENCE_DIR}/mob_* 2>/dev/null | head -1)}
    ENDPOINT="${2:-https://firestar-defense.vercel.app/api/ingest}"
    
    if [ -z "$TARGET" ] || [ ! -d "$TARGET" ]; then
        echo "Nothing to upload. Run: ./starfire capture first"
        exit 1
    fi
    
    echo "Uploading $(basename $TARGET) to $ENDPOINT..."
    
    # Combine all JSON into one payload
    PAYLOAD=$(python3 -c "
import json, os
from pathlib import Path
combined = {}
for f in Path('$TARGET').glob('*.json'):
    combined[f.stem] = json.load(open(f))
print(json.dumps(combined))
")
    
    DEVICE_ID=$(python3 -c "
import json
try:
    d = json.load(open('$TARGET/device.json'))
    print(d.get('capture_id', 'unknown'))
except: print('unknown')
")
    
    curl -s -X POST "$ENDPOINT" \
        -H "Content-Type: application/json" \
        -H "X-Device-ID: $DEVICE_ID" \
        -H "X-Capture-Time: $(date -u +%Y-%m-%dT%H:%M:%SZ)" \
        -d "$PAYLOAD"
    echo ""
}

# ── FULL (one shot: capture + analyze + seal) ──

do_full() {
    do_capture "${1:-30}"
    echo ""
    LATEST=$(ls -td ${EVIDENCE_DIR}/mob_* 2>/dev/null | head -1)
    do_analyze "$LATEST"
}

# ── CLI ROUTER ──

case "${1:-help}" in
    capture)  do_capture "${2:-30}" ;;
    analyze)  do_analyze "$2" ;;
    upload)   do_upload "$2" "$3" ;;
    full)     do_full "${2:-30}" ;;
    tools)
        case "${2:-check}" in
            install) install_tools ;;
            check|*) check_tools ;;
        esac ;;
    help|*)
        echo "StarFire Mobile v${VERSION} — Phone Forensics Toolkit"
        echo ""
        echo "Usage:"
        echo "  ./starfire capture [seconds]  — capture network state (default 30s)"
        echo "  ./starfire analyze [dir]      — analyze latest capture (offline)"
        echo "  ./starfire upload [dir] [url] — send results to Firestar (optional)"
        echo "  ./starfire full [seconds]     — capture + analyze in one shot"
        echo "  ./starfire tools install      — install all dependencies"
        echo "  ./starfire tools check        — verify tool availability"
        echo ""
        echo "All analysis is LOCAL. Upload is OPT-IN only."
        echo "Evidence dir: ${EVIDENCE_DIR}/"
        ;;
esac
