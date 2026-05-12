#!/bin/bash
# ╔══════════════════════════════════════════════════════════════╗
# ║  LATTICE TEAM — KHET-1 — iOS PHONE FORENSICS AGENT         ║
# ║  For jailbroken iOS OR via macOS relay (rvictl)             ║
# ╚══════════════════════════════════════════════════════════════╝
#
# METHOD A: Run on jailbroken iOS (via SSH/NewTerm)
# METHOD B: Run on macOS connected to iPhone via USB (uses rvictl)
#
# USAGE:
#   # On Mac with iPhone connected via USB:
#   curl -sL https://workspace-star-far.vercel.app/agents/phone-capture-ios.sh | bash
#
#   # On jailbroken iOS:
#   curl -sL https://workspace-star-far.vercel.app/agents/phone-capture-ios.sh | bash -s -- --jailbreak

LATTICE_ENDPOINT="https://workspace-star-far.vercel.app/api/ingest"
DURATION=${1:-30}
MODE="relay"
CAPTURE_DIR="/tmp/lattice_ios_$$"

if [ "$1" = "--jailbreak" ] || [ "$2" = "--jailbreak" ]; then
    MODE="jailbreak"
    DURATION=${2:-30}
fi

echo "╔══════════════════════════════════════════════════════════╗"
echo "║  LATTICE AGENT — iOS Phone Forensics                    ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""
echo "  Mode: $MODE"
echo "  Duration: ${DURATION}s"
echo ""

mkdir -p "$CAPTURE_DIR"

if [ "$MODE" = "relay" ]; then
    # ── macOS USB Relay Method ──
    echo "[1/5] Finding connected iOS device..."
    
    # Get device UDID
    UDID=$(system_profiler SPUSBDataType 2>/dev/null | grep -A5 "iPhone\|iPad" | grep "Serial Number" | awk '{print $3}')
    if [ -z "$UDID" ]; then
        UDID=$(idevice_id -l 2>/dev/null | head -1)
    fi
    if [ -z "$UDID" ]; then
        echo "  [!] No iOS device found. Connect via USB and trust this computer."
        exit 1
    fi
    echo "  Device UDID: $UDID"
    DEVICE_ID="ios-${UDID:0:8}"
    
    # Get device info
    echo "[2/5] Collecting device info..."
    ideviceinfo 2>/dev/null > "$CAPTURE_DIR/device_info.txt" || true
    cat > "$CAPTURE_DIR/device_meta.json" << DEVEOF
{
  "device_id": "$DEVICE_ID",
  "udid": "$UDID",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "method": "rvictl_relay",
  "host_os": "$(sw_vers -productVersion 2>/dev/null || echo 'unknown')"
}
DEVEOF

    # Create virtual interface
    echo "[3/5] Creating remote virtual interface (rvictl)..."
    rvictl -s "$UDID" 2>/dev/null
    sleep 2
    
    # Find the rvi interface
    RVI_IF=$(ifconfig -l | tr ' ' '\n' | grep rvi)
    if [ -z "$RVI_IF" ]; then
        echo "  [!] rvictl failed. Ensure Xcode is installed and device is trusted."
        echo "  Try: sudo rvictl -s $UDID"
        exit 1
    fi
    echo "  Interface: $RVI_IF"
    
    # Capture
    echo "[4/5] Capturing iPhone traffic via $RVI_IF (${DURATION}s)..."
    PCAP_FILE="$CAPTURE_DIR/ios_capture.pcap"
    sudo tcpdump -i "$RVI_IF" -c 5000 -w "$PCAP_FILE" &
    CAP_PID=$!
    sleep $DURATION
    sudo kill $CAP_PID 2>/dev/null
    wait $CAP_PID 2>/dev/null
    
    # Cleanup virtual interface
    rvictl -x "$UDID" 2>/dev/null
    
    echo "  Captured: $(wc -c < "$PCAP_FILE" 2>/dev/null | tr -d ' ') bytes"

else
    # ── Jailbroken iOS Direct Method ──
    DEVICE_ID="ios-jb-$(hostname)"
    
    echo "[1/5] System info..."
    cat > "$CAPTURE_DIR/device_meta.json" << DEVEOF
{
  "device_id": "$DEVICE_ID",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "method": "jailbreak_direct",
  "ios_version": "$(sw_vers -productVersion 2>/dev/null || echo 'unknown')",
  "hostname": "$(hostname)"
}
DEVEOF

    echo "[2/5] Network state..."
    netstat -an > "$CAPTURE_DIR/netstat.txt" 2>/dev/null
    ifconfig > "$CAPTURE_DIR/ifconfig.txt" 2>/dev/null
    
    echo "[3/5] Process network map..."
    lsof -i -n -P > "$CAPTURE_DIR/lsof_network.txt" 2>/dev/null
    ps aux > "$CAPTURE_DIR/ps.txt" 2>/dev/null
    
    echo "[4/5] Packet capture (${DURATION}s)..."
    PCAP_FILE="$CAPTURE_DIR/ios_capture.pcap"
    if command -v tcpdump &>/dev/null; then
        timeout $DURATION tcpdump -i any -c 5000 -w "$PCAP_FILE" 2>/dev/null
    else
        echo "  [!] No tcpdump. Install via Cydia/Sileo: tcpdump"
        # Fallback: poll netstat
        for i in $(seq 1 $DURATION); do
            echo "--- t=$i ---" >> "$CAPTURE_DIR/netstat_poll.txt"
            netstat -an >> "$CAPTURE_DIR/netstat_poll.txt" 2>/dev/null
            sleep 1
        done
    fi
fi

# ── Phase 5: Package & Upload ──
echo "[5/5] Packaging and uploading..."

TARBALL="$CAPTURE_DIR/lattice_ios_${DEVICE_ID}.tar.gz"
cd "$CAPTURE_DIR" && tar -czf "$TARBALL" --exclude="*.tar.gz" . 2>/dev/null

FILESIZE=$(wc -c < "$TARBALL" 2>/dev/null | tr -d ' ')
echo "  Package: $FILESIZE bytes"

RESPONSE=$(curl -s -w "\n%{http_code}" \
    -X POST "$LATTICE_ENDPOINT" \
    -H "Content-Type: application/octet-stream" \
    -H "X-Device-ID: $DEVICE_ID" \
    -H "X-Capture-Time: $(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --data-binary "@$TARBALL" 2>/dev/null)

HTTP_CODE=$(echo "$RESPONSE" | tail -1)
if [ "$HTTP_CODE" = "200" ] || [ "$HTTP_CODE" = "201" ]; then
    echo "  ✓ Uploaded to Lattice IDE"
else
    echo "  ✗ Upload failed (HTTP $HTTP_CODE) — saved locally: $TARBALL"
    echo ""
    echo "  Manual: curl -X POST $LATTICE_ENDPOINT -H 'X-Device-ID: $DEVICE_ID' --data-binary @$TARBALL"
fi

echo ""
echo "╔══════════════════════════════════════════════════════════╗"
echo "║  iOS CAPTURE COMPLETE — Results in $CAPTURE_DIR         ║"
echo "╚══════════════════════════════════════════════════════════╝"
