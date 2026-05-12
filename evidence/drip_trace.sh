#!/bin/bash
# LATTICE TEAM — KHET-1 — Local Drip Trace Script
# Run on: 192.168.2.46 (Parrot box / rathin)
# Target process: gvfsd-http (PID 181723)

echo "╔══════════════════════════════════════════════════════════╗"
echo "║  LATTICE TEAM — LOCAL DRIP TRACE                        ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""

PID=181723

# ── 1. CONFIRM PROCESS IS ALIVE ──
echo "=== [1] PROCESS STATUS ==="
if [ -d "/proc/$PID" ]; then
    echo "  PID $PID: ALIVE"
    echo "  Binary: $(readlink /proc/$PID/exe)"
    echo "  Cmdline: $(cat /proc/$PID/cmdline | tr '\0' ' ')"
    echo "  CWD: $(readlink /proc/$PID/cwd)"
    echo "  Started: $(stat -c '%y' /proc/$PID)"
    echo "  User: $(stat -c '%U' /proc/$PID)"
else
    echo "  PID $PID: DEAD — finding current gvfsd-http..."
    PID=$(pgrep -f gvfsd-http | head -1)
    if [ -z "$PID" ]; then
        echo "  No gvfsd-http running. Check: pgrep -fa gvfs"
    else
        echo "  Found new PID: $PID"
        echo "  Binary: $(readlink /proc/$PID/exe)"
        echo "  Cmdline: $(cat /proc/$PID/cmdline | tr '\0' ' ')"
    fi
fi
echo ""

# ── 2. WHAT TRIGGERED gvfsd-http? ──
echo "=== [2] PARENT CHAIN (who spawned it) ==="
current=$PID
for i in $(seq 1 10); do
    if [ -z "$current" ] || [ "$current" == "1" ]; then break; fi
    comm=$(cat /proc/$current/comm 2>/dev/null)
    cmdline=$(cat /proc/$current/cmdline 2>/dev/null | tr '\0' ' ' | cut -c1-100)
    ppid=$(grep PPid /proc/$current/status 2>/dev/null | awk '{print $2}')
    echo "  [$i] PID=$current ($comm) → $cmdline"
    current=$ppid
done
echo ""

# ── 3. WHAT'S LISTENING ON THOSE PORTS ──
echo "=== [3] SERVICES ON PORTS 7734, 8082, 8083 ==="
echo "  Port 7734:"
sudo ss -tlnp 'sport == 7734' 2>/dev/null || sudo netstat -tlnp 2>/dev/null | grep 7734
echo "  Port 8082:"
sudo ss -tlnp 'sport == 8082' 2>/dev/null || sudo netstat -tlnp 2>/dev/null | grep 8082
echo "  Port 8083:"
sudo ss -tlnp 'sport == 8083' 2>/dev/null || sudo netstat -tlnp 2>/dev/null | grep 8083
echo ""

# ── 4. FILE DESCRIPTORS (what's gvfsd-http connected to) ──
echo "=== [4] OPEN FILE DESCRIPTORS ==="
sudo ls -la /proc/$PID/fd 2>/dev/null | grep socket | head -20
echo ""
echo "  Socket details:"
sudo cat /proc/$PID/net/tcp 2>/dev/null | head -20
echo ""

# ── 5. WHO IS MAKING REQUESTS TO THESE SERVICES ──
echo "=== [5] ALL CLIENTS CONNECTED TO 7734/8082/8083 ==="
sudo ss -tupn 'dport == 7734 or dport == 8082 or dport == 8083 or sport == 7734 or sport == 8082 or sport == 8083'
echo ""

# ── 6. WHAT TRIGGERED THE GIO/GVFS MOUNT ──
echo "=== [6] GVFS MOUNTS (what HTTP resources are mounted) ==="
gio mount -l 2>/dev/null | grep -i http
echo ""
echo "  GVFS mount points:"
ls -la /run/user/$(id -u)/gvfs/ 2>/dev/null
echo ""
echo "  GIO monitor (file access triggers):"
dbus-send --session --dest=org.gtk.vfs.Daemon \
  --print-reply /org/gtk/vfs/mount_tracker \
  org.gtk.vfs.MountTracker.ListMountableInfo 2>/dev/null | head -30
echo ""

# ── 7. WHAT DBUS ACTIVATED gvfsd-http ──
echo "=== [7] DBUS ACTIVATION ==="
grep -r "gvfsd-http" /usr/share/dbus-1/ 2>/dev/null
echo ""
echo "  Session bus connections:"
dbus-send --session --dest=org.freedesktop.DBus \
  --print-reply /org/freedesktop/DBus \
  org.freedesktop.DBus.ListNames 2>/dev/null | grep -i gvfs
echo ""

# ── 8. FIND WHAT APP IS USING GIO HTTP ──
echo "=== [8] APPS USING GIO/GVFS HTTP ==="
echo "  All gvfs processes:"
pgrep -fa gvfs
echo ""
echo "  GTK apps with open HTTP handles:"
for pid in $(pgrep -f "gvfsd-http"); do
    echo "  PID $pid parents:"
    pstree -p $pid 2>/dev/null | head -3
done
echo ""

# ── 9. PYTHON SCRIPTS USING URLLIB (the UA from pcap) ──
echo "=== [9] PYTHON PROCESSES ==="
pgrep -fa python
echo ""
echo "  Python scripts accessing 7734/8082/8083:"
sudo lsof -i :7734 -i :8082 -i :8083 2>/dev/null
echo ""

# ── 10. FIND THE POLLING SCRIPT ──
echo "=== [10] FILE SEARCH — Script hitting these endpoints ==="
echo "  Searching for scripts referencing 7734/8082/status/api/notes..."
find /home/rathin -name "*.py" -exec grep -l "7734\|8082\|/status\|/api/notes" {} \; 2>/dev/null
echo ""
echo "  Searching for scripts referencing 192.168.2.151..."
find /home/rathin -name "*.py" -exec grep -l "192.168.2.151\|192.168.1.46" {} \; 2>/dev/null
echo ""
echo "  Crontab:"
crontab -l 2>/dev/null
echo ""
echo "  Systemd user services:"
systemctl --user list-units --type=service --state=running 2>/dev/null
echo ""
echo "  Autostart entries:"
ls ~/.config/autostart/ 2>/dev/null
echo ""

# ── 11. CURL THE SERVICES DIRECTLY ──
echo "=== [11] SERVICE RESPONSES ==="
echo "  GET http://127.0.0.1:7734/status"
curl -s -v http://127.0.0.1:7734/status 2>&1 | head -15
echo ""
echo "  GET http://127.0.0.1:8082/api/notes"
curl -s -v http://127.0.0.1:8082/api/notes 2>&1 | head -15
echo ""
echo "  GET http://127.0.0.1:8083/"
curl -s -v http://127.0.0.1:8083/ 2>&1 | head -15
echo ""

# ── 12. STRACE (if still running) ──
echo "=== [12] LIVE TRACE (5 seconds) ==="
if [ -d "/proc/$PID" ]; then
    timeout 5 sudo strace -e trace=network -fp $PID 2>&1 | head -30
fi
echo ""

echo "╔══════════════════════════════════════════════════════════╗"
echo "║  TRACE COMPLETE — Pipe output to: ./drip_trace.sh > trace_output.txt 2>&1"
echo "╚══════════════════════════════════════════════════════════╝"
