#!/bin/bash
# ╔══════════════════════════════════════════════════════════════╗
# ║  LATTICE TEAM — KHET-1 — ANDROID PHONE FORENSICS AGENT     ║
# ║  Download → Run → Capture → Return to Lattice IDE          ║
# ╚══════════════════════════════════════════════════════════════╝
#
# REQUIREMENTS:
#   - Termux or rooted shell on Android
#   - tcpdump (pkg install root-repo && pkg install tcpdump) OR
#   - tshark (pkg install tshark)
#   - curl (pre-installed on Termux)
#
# USAGE:
#   curl -sL https://workspace-star-far.vercel.app/agents/phone-capture-android.sh | bash
#   OR download and run:
#   chmod +x phone-capture-android.sh && ./phone-capture-android.sh
#
# For non-root: uses /proc/net + dumpsys instead of tcpdump

LATTICE_ENDPOINT="https://workspace-star-far.vercel.app/api/ingest"
DEVICE_ID="android-$(getprop ro.serialno 2>/dev/null || echo $(cat /proc/sys/kernel/random/uuid | cut -d- -f1))"
CAPTURE_DIR="/tmp/lattice_capture_$$"
DURATION=${1:-30}

echo "╔══════════════════════════════════════════════════════════╗"
echo "║  LATTICE AGENT — Android Phone Forensics                ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""
echo "  Device: $DEVICE_ID"
echo "  Duration: ${DURATION}s"
echo "  Endpoint: $LATTICE_ENDPOINT"
echo ""

mkdir -p "$CAPTURE_DIR"

# ── Phase 1: System Info ──
echo "[1/6] Collecting system info..."
cat > "$CAPTURE_DIR/device_info.json" << DEVEOF
{
  "device_id": "$DEVICE_ID",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "model": "$(getprop ro.product.model 2>/dev/null || echo 'unknown')",
  "android_version": "$(getprop ro.build.version.release 2>/dev/null || echo 'unknown')",
  "kernel": "$(uname -r)",
  "hostname": "$(hostname)",
  "uptime": "$(uptime)"
}
DEVEOF

# ── Phase 2: Network State ──
echo "[2/6] Capturing network state..."

# Active connections
cat /proc/net/tcp 2>/dev/null > "$CAPTURE_DIR/proc_net_tcp.txt"
cat /proc/net/tcp6 2>/dev/null > "$CAPTURE_DIR/proc_net_tcp6.txt"
cat /proc/net/udp 2>/dev/null > "$CAPTURE_DIR/proc_net_udp.txt"

# IP addresses
ip addr show 2>/dev/null > "$CAPTURE_DIR/ip_addr.txt" || ifconfig > "$CAPTURE_DIR/ip_addr.txt" 2>/dev/null

# Routing
ip route show 2>/dev/null > "$CAPTURE_DIR/routes.txt"

# DNS
cat /etc/resolv.conf 2>/dev/null > "$CAPTURE_DIR/dns.txt"
getprop net.dns1 2>/dev/null >> "$CAPTURE_DIR/dns.txt"
getprop net.dns2 2>/dev/null >> "$CAPTURE_DIR/dns.txt"

# ── Phase 3: Process Network Map ──
echo "[3/6] Mapping processes to connections..."

# Parse /proc/net/tcp into readable form
python3 -c "
import struct, os, json

def parse_proc_net(path):
    connections = []
    try:
        with open(path) as f:
            for line in f.readlines()[1:]:
                parts = line.strip().split()
                if len(parts) < 10: continue
                local = parts[1].split(':')
                remote = parts[2].split(':')
                local_ip = '.'.join(str(int(local[0][i:i+2], 16)) for i in range(6,-2,-2))
                local_port = int(local[1], 16)
                remote_ip = '.'.join(str(int(remote[0][i:i+2], 16)) for i in range(6,-2,-2))
                remote_port = int(remote[1], 16)
                state = int(parts[3], 16)
                uid = parts[7]
                inode = parts[9]
                if remote_port > 0:
                    connections.append({
                        'local': f'{local_ip}:{local_port}',
                        'remote': f'{remote_ip}:{remote_port}',
                        'state': state,
                        'uid': uid,
                        'inode': inode
                    })
    except: pass
    return connections

conns = parse_proc_net('/proc/net/tcp')
conns += parse_proc_net('/proc/net/tcp6')

# Try to map inodes to PIDs
pid_map = {}
for pid_dir in os.listdir('/proc'):
    if not pid_dir.isdigit(): continue
    try:
        fd_dir = f'/proc/{pid_dir}/fd'
        cmdline = open(f'/proc/{pid_dir}/cmdline').read().replace('\x00', ' ').strip()
        for fd in os.listdir(fd_dir):
            try:
                link = os.readlink(f'{fd_dir}/{fd}')
                if 'socket:' in link:
                    inode = link.split('[')[1].rstrip(']')
                    pid_map[inode] = {'pid': pid_dir, 'cmd': cmdline[:100]}
            except: pass
    except: pass

for conn in conns:
    if conn['inode'] in pid_map:
        conn['process'] = pid_map[conn['inode']]

with open('$CAPTURE_DIR/connections.json', 'w') as f:
    json.dump(conns, f, indent=2)
print(f'  Mapped {len(conns)} connections, {sum(1 for c in conns if \"process\" in c)} with PIDs')
" 2>/dev/null || echo "  (Python not available — using raw /proc data)"

# ── Phase 4: Packet Capture ──
echo "[4/6] Capturing packets (${DURATION}s)..."

PCAP_FILE="$CAPTURE_DIR/capture.pcap"
if command -v tcpdump &>/dev/null; then
    timeout $DURATION tcpdump -i any -c 5000 -w "$PCAP_FILE" 2>/dev/null &
    CAP_PID=$!
    sleep $DURATION
    kill $CAP_PID 2>/dev/null
    wait $CAP_PID 2>/dev/null
    echo "  Captured: $(wc -c < "$PCAP_FILE" 2>/dev/null || echo 0) bytes"
elif command -v tshark &>/dev/null; then
    timeout $DURATION tshark -i any -c 5000 -w "$PCAP_FILE" 2>/dev/null
    echo "  Captured: $(wc -c < "$PCAP_FILE" 2>/dev/null || echo 0) bytes"
else
    echo "  [!] No capture tool. Using /proc/net polling instead..."
    # Poll /proc/net/tcp every second for connection changes
    for i in $(seq 1 $DURATION); do
        echo "--- t=$i ---" >> "$CAPTURE_DIR/net_poll.txt"
        cat /proc/net/tcp >> "$CAPTURE_DIR/net_poll.txt" 2>/dev/null
        sleep 1
    done
    echo "  Polled /proc/net for ${DURATION}s"
fi

# ── Phase 5: App-level network usage (Android-specific) ──
echo "[5/6] Collecting app network stats..."

# dumpsys for network stats (Android)
dumpsys netstats 2>/dev/null | head -200 > "$CAPTURE_DIR/netstats.txt"
dumpsys connectivity 2>/dev/null | head -100 > "$CAPTURE_DIR/connectivity.txt"

# Per-UID traffic stats
if [ -d "/proc/uid_stat" ]; then
    ls /proc/uid_stat/ | while read uid; do
        rx=$(cat /proc/uid_stat/$uid/tcp_rcv 2>/dev/null || echo 0)
        tx=$(cat /proc/uid_stat/$uid/tcp_snd 2>/dev/null || echo 0)
        if [ "$rx" -gt 1000 ] || [ "$tx" -gt 1000 ]; then
            echo "$uid: rx=$rx tx=$tx"
        fi
    done > "$CAPTURE_DIR/uid_traffic.txt"
fi

# ── Phase 6: Package & Upload ──
echo "[6/6] Packaging and uploading results..."

# Create tarball
TARBALL="$CAPTURE_DIR/lattice_results_${DEVICE_ID}.tar.gz"
cd "$CAPTURE_DIR" && tar -czf "$TARBALL" --exclude="*.tar.gz" . 2>/dev/null

FILESIZE=$(wc -c < "$TARBALL" 2>/dev/null || echo 0)
echo "  Package: $TARBALL ($FILESIZE bytes)"

# Upload to Lattice endpoint
echo "  Uploading to Lattice IDE..."
RESPONSE=$(curl -s -w "\n%{http_code}" \
    -X POST "$LATTICE_ENDPOINT" \
    -H "Content-Type: application/octet-stream" \
    -H "X-Device-ID: $DEVICE_ID" \
    -H "X-Capture-Time: $(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --data-binary "@$TARBALL" 2>/dev/null)

HTTP_CODE=$(echo "$RESPONSE" | tail -1)
BODY=$(echo "$RESPONSE" | head -n -1)

if [ "$HTTP_CODE" = "200" ] || [ "$HTTP_CODE" = "201" ]; then
    echo "  ✓ Upload successful (HTTP $HTTP_CODE)"
    echo "  Response: $BODY"
else
    echo "  ✗ Upload failed (HTTP $HTTP_CODE)"
    echo "  Saving locally: $TARBALL"
    echo ""
    echo "  Manual upload:"
    echo "  curl -X POST $LATTICE_ENDPOINT \\"
    echo "    -H 'Content-Type: application/octet-stream' \\"
    echo "    -H 'X-Device-ID: $DEVICE_ID' \\"
    echo "    --data-binary @$TARBALL"
fi

echo ""
echo "╔══════════════════════════════════════════════════════════╗"
echo "║  CAPTURE COMPLETE                                        ║"
echo "║  Results: $CAPTURE_DIR                                   ║"
echo "╚══════════════════════════════════════════════════════════╝"
