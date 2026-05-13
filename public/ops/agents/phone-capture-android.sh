#!/bin/bash
# ╔══════════════════════════════════════════════════════════════╗
# ║  LATTICE TEAM — KHET-1 — ANDROID PHONE AGENT               ║
# ║  Self-capture → Direct POST to Firestar                     ║
# ║  No router, no desktop, no LAN hub required                 ║
# ╚══════════════════════════════════════════════════════════════╝
#
# USAGE (Termux):
#   curl -sL https://firestar-defense.vercel.app/agents/phone-capture-android.sh | bash
#
# USAGE (with custom duration):
#   curl -sL https://firestar-defense.vercel.app/agents/phone-capture-android.sh | bash -s 60
#
# WHAT IT CAPTURES:
#   - All active TCP/UDP connections with process/app mapping
#   - DNS resolver config
#   - Network interfaces and IPs
#   - Per-app traffic byte counters
#   - Connection timing (for beacon detection)
#   - Packet capture (if tcpdump available, otherwise /proc polling)
#
# REQUIREMENTS:
#   - Termux (https://f-droid.org/packages/com.termux/)
#   - curl (pkg install curl)
#   - Optional: tcpdump (pkg install root-repo && pkg install tcpdump)
#   - Optional: python3 (pkg install python)

LATTICE_ENDPOINT="https://firestar-defense.vercel.app/api/ingest"
DURATION=${1:-30}
CAPTURE_DIR="/tmp/lattice_$$"
DEVICE_ID="android-$(getprop ro.serialno 2>/dev/null || cat /proc/sys/kernel/random/uuid 2>/dev/null | cut -d- -f1-2 || echo unk-$$)"

echo ""
echo "╔══════════════════════════════════════════════════════════╗"
echo "║  LATTICE AGENT — Phone Direct-to-Cloud                  ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""
echo "  Device:   $DEVICE_ID"
echo "  Duration: ${DURATION}s"
echo "  Target:   $LATTICE_ENDPOINT"
echo ""

mkdir -p "$CAPTURE_DIR"

# ── 1. DEVICE IDENTITY ──
echo "[1/7] Device info..."
MODEL=$(getprop ro.product.model 2>/dev/null || echo "unknown")
ANDROID_VER=$(getprop ro.build.version.release 2>/dev/null || echo "?")
CARRIER=$(getprop gsm.operator.alpha 2>/dev/null || echo "unknown")
WIFI_SSID=$(dumpsys wifi 2>/dev/null | grep "mWifiInfo" | grep -o 'SSID: [^,]*' | cut -d' ' -f2 || echo "?")
NET_TYPE="unknown"
# Detect if on WiFi or cellular
if ip addr show wlan0 2>/dev/null | grep -q "inet "; then
    NET_TYPE="wifi"
elif ip addr show rmnet0 2>/dev/null | grep -q "inet "; then
    NET_TYPE="cellular"
fi

cat > "$CAPTURE_DIR/device.json" << EOF
{
  "device_id": "$DEVICE_ID",
  "model": "$MODEL",
  "android_version": "$ANDROID_VER",
  "carrier": "$CARRIER",
  "network_type": "$NET_TYPE",
  "wifi_ssid": "$WIFI_SSID",
  "kernel": "$(uname -r 2>/dev/null)",
  "capture_time": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "duration_seconds": $DURATION
}
EOF
echo "  $MODEL | Android $ANDROID_VER | $CARRIER | $NET_TYPE"

# ── 2. ALL ACTIVE CONNECTIONS ──
echo "[2/7] Active connections..."
CONN_FILE="$CAPTURE_DIR/connections.json"

python3 -c "
import os, json, struct

def parse_addr(hex_addr):
    ip_hex, port_hex = hex_addr.split(':')
    port = int(port_hex, 16)
    # Little-endian IP for /proc/net
    ip_bytes = bytes.fromhex(ip_hex)
    ip = '.'.join(str(b) for b in reversed(ip_bytes))
    return ip, port

def parse_proc_net(path, proto):
    conns = []
    try:
        with open(path) as f:
            for line in f.readlines()[1:]:
                parts = line.strip().split()
                if len(parts) < 10: continue
                local_ip, local_port = parse_addr(parts[1])
                remote_ip, remote_port = parse_addr(parts[2])
                state = int(parts[3], 16)
                uid = int(parts[7])
                inode = parts[9]
                if remote_port == 0 and state == 10:  # LISTEN
                    continue
                conns.append({
                    'proto': proto,
                    'local': f'{local_ip}:{local_port}',
                    'remote': f'{remote_ip}:{remote_port}',
                    'state': state,
                    'uid': uid,
                    'inode': inode
                })
    except: pass
    return conns

# Map UIDs to package names
uid_to_pkg = {}
try:
    for line in open('/data/system/packages.list'):
        parts = line.strip().split()
        if len(parts) >= 2:
            uid_to_pkg[int(parts[1])] = parts[0]
except:
    # Termux fallback: parse pm list packages
    import subprocess
    try:
        out = subprocess.run(['pm', 'list', 'packages', '-U'], capture_output=True, text=True, timeout=5).stdout
        for line in out.split('\n'):
            if 'uid:' in line:
                pkg = line.split(':')[1].split(' ')[0]
                uid = int(line.split('uid:')[1].strip())
                uid_to_pkg[uid] = pkg
    except: pass

# Map inodes to PIDs
inode_to_pid = {}
for pid_dir in os.listdir('/proc'):
    if not pid_dir.isdigit(): continue
    try:
        for fd in os.listdir(f'/proc/{pid_dir}/fd'):
            try:
                link = os.readlink(f'/proc/{pid_dir}/fd/{fd}')
                if 'socket:[' in link:
                    ino = link.split('[')[1].rstrip(']')
                    cmdline = open(f'/proc/{pid_dir}/cmdline').read().replace('\x00',' ')[:80]
                    inode_to_pid[ino] = {'pid': pid_dir, 'cmd': cmdline}
            except: pass
    except: pass

conns = parse_proc_net('/proc/net/tcp', 'tcp')
conns += parse_proc_net('/proc/net/tcp6', 'tcp6')
conns += parse_proc_net('/proc/net/udp', 'udp')

# Enrich with app/process info
for c in conns:
    c['app'] = uid_to_pkg.get(c['uid'], f\"uid:{c['uid']}\")
    if c['inode'] in inode_to_pid:
        c['process'] = inode_to_pid[c['inode']]

# Filter out loopback
conns = [c for c in conns if not c['remote'].startswith('127.') and not c['remote'].startswith('0.0.0.0')]

with open('$CONN_FILE', 'w') as f:
    json.dump(conns, f, indent=2)
print(f'  {len(conns)} active connections mapped')
" 2>/dev/null || {
    # Fallback without python
    echo "  (no python — raw /proc capture)"
    cat /proc/net/tcp > "$CAPTURE_DIR/proc_net_tcp.txt" 2>/dev/null
    cat /proc/net/tcp6 > "$CAPTURE_DIR/proc_net_tcp6.txt" 2>/dev/null
    cat /proc/net/udp > "$CAPTURE_DIR/proc_net_udp.txt" 2>/dev/null
}

# ── 3. DNS CONFIG ──
echo "[3/7] DNS configuration..."
cat > "$CAPTURE_DIR/dns.json" << EOF
{
  "dns1": "$(getprop net.dns1 2>/dev/null || echo '?')",
  "dns2": "$(getprop net.dns2 2>/dev/null || echo '?')",
  "private_dns": "$(getprop net.dns.tls_hostname 2>/dev/null || echo 'none')",
  "resolv": "$(cat /etc/resolv.conf 2>/dev/null | grep nameserver | tr '\n' ' ')"
}
EOF

# ── 4. NETWORK INTERFACES ──
echo "[4/7] Network interfaces..."
ip addr show 2>/dev/null > "$CAPTURE_DIR/interfaces.txt" || ifconfig > "$CAPTURE_DIR/interfaces.txt" 2>/dev/null
ip route show 2>/dev/null > "$CAPTURE_DIR/routes.txt"

# ── 5. PER-APP TRAFFIC STATS ──
echo "[5/7] Per-app traffic counters..."
python3 -c "
import os, json

stats = []
uid_stat_dir = '/proc/uid_stat'
if os.path.isdir(uid_stat_dir):
    for uid in os.listdir(uid_stat_dir):
        try:
            rx = int(open(f'{uid_stat_dir}/{uid}/tcp_rcv').read().strip())
            tx = int(open(f'{uid_stat_dir}/{uid}/tcp_snd').read().strip())
            if rx > 1024 or tx > 1024:
                stats.append({'uid': int(uid), 'rx_bytes': rx, 'tx_bytes': tx})
        except: pass

stats.sort(key=lambda x: x['tx_bytes'], reverse=True)
with open('$CAPTURE_DIR/app_traffic.json', 'w') as f:
    json.dump(stats[:50], f, indent=2)
print(f'  {len(stats)} apps with traffic')
" 2>/dev/null || echo "  (uid_stat not available)"

# ── 6. CONNECTION POLLING (beacon detection) ──
echo "[6/7] Polling connections for ${DURATION}s (beacon detection)..."
POLL_FILE="$CAPTURE_DIR/conn_poll.json"
python3 -c "
import time, json

samples = []
duration = $DURATION
interval = 2  # sample every 2s
start = time.time()

while time.time() - start < duration:
    conns = set()
    try:
        with open('/proc/net/tcp') as f:
            for line in f.readlines()[1:]:
                parts = line.strip().split()
                if len(parts) >= 4:
                    remote = parts[2]
                    state = parts[3]
                    if state == '01':  # ESTABLISHED
                        conns.add(remote)
    except: pass
    samples.append({
        't': round(time.time() - start, 1),
        'established': len(conns),
        'remotes': list(conns)[:20]
    })
    time.sleep(interval)

with open('$POLL_FILE', 'w') as f:
    json.dump(samples, f)
print(f'  {len(samples)} samples taken over {duration}s')
" 2>/dev/null || {
    # No python fallback — just poll raw
    for i in $(seq 1 $((DURATION / 3))); do
        echo "=== t=$((i*3)) ===" >> "$CAPTURE_DIR/poll_raw.txt"
        grep " 01 " /proc/net/tcp >> "$CAPTURE_DIR/poll_raw.txt" 2>/dev/null
        sleep 3
    done
    echo "  polled raw for ${DURATION}s"
}

# ── 7. OPTIONAL: PACKET CAPTURE ──
echo "[7/7] Packet capture..."
PCAP_FILE="$CAPTURE_DIR/capture.pcap"
if command -v tcpdump &>/dev/null; then
    timeout $DURATION tcpdump -i any -c 3000 -s 128 -w "$PCAP_FILE" 2>/dev/null
    PCAP_SIZE=$(wc -c < "$PCAP_FILE" 2>/dev/null || echo 0)
    echo "  tcpdump: $PCAP_SIZE bytes"
else
    echo "  (tcpdump not available — using connection polling only)"
    echo "  Install: pkg install root-repo && pkg install tcpdump"
    echo "  Or use PCAPdroid app (no root needed)"
fi

# ── PACKAGE & UPLOAD ──
echo ""
echo "[UPLOAD] Packaging results..."

# Build JSON payload (no tarball dependency)
RESULT=$(python3 -c "
import json, os, hashlib

payload = {}

# Load all JSON files
for f in os.listdir('$CAPTURE_DIR'):
    path = os.path.join('$CAPTURE_DIR', f)
    if f.endswith('.json'):
        try:
            payload[f.replace('.json','')] = json.load(open(path))
        except:
            payload[f.replace('.json','')] = open(path).read()[:5000]
    elif f.endswith('.txt'):
        payload[f.replace('.txt','')] = open(path).read()[:5000]

# Add pcap hash if exists
pcap_path = '$PCAP_FILE'
if os.path.exists(pcap_path) and os.path.getsize(pcap_path) > 0:
    h = hashlib.sha256(open(pcap_path,'rb').read()).hexdigest()
    payload['pcap_sha256'] = h
    payload['pcap_size'] = os.path.getsize(pcap_path)

print(json.dumps(payload))
" 2>/dev/null)

if [ -z "$RESULT" ]; then
    # Fallback: just upload raw text
    RESULT="{\"raw_connections\": \"$(cat /proc/net/tcp 2>/dev/null | head -30 | tr '\n' '|')\"}"
fi

echo "[UPLOAD] Sending to Firestar..."
HTTP_CODE=$(curl -s -w "%{http_code}" -o /tmp/lattice_response.txt \
    -X POST "$LATTICE_ENDPOINT" \
    -H "Content-Type: application/json" \
    -H "X-Device-ID: $DEVICE_ID" \
    -H "X-Capture-Time: $(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    -d "$RESULT")

echo ""
if [ "$HTTP_CODE" = "200" ] || [ "$HTTP_CODE" = "201" ]; then
    echo "  ✓ UPLOADED (HTTP $HTTP_CODE)"
    cat /tmp/lattice_response.txt 2>/dev/null
else
    echo "  ✗ FAILED (HTTP $HTTP_CODE)"
    echo "  Response: $(cat /tmp/lattice_response.txt 2>/dev/null)"
    echo ""
    echo "  Saved locally: $CAPTURE_DIR/"
    echo "  Manual upload later:"
    echo "  curl -X POST $LATTICE_ENDPOINT -H 'Content-Type: application/json' -H 'X-Device-ID: $DEVICE_ID' -d @$CAPTURE_DIR/payload.json"
    echo "$RESULT" > "$CAPTURE_DIR/payload.json"
fi

echo ""
echo "╔══════════════════════════════════════════════════════════╗"
echo "║  DONE — Results at: https://firestar-defense.vercel.app/agents/results.html"
echo "╚══════════════════════════════════════════════════════════╝"
