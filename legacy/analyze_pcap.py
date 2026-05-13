#!/usr/bin/env python3
"""Quick PCAP analysis script - produces a full report from a capture file."""

import sys
import json
from collections import defaultdict, Counter
from pathlib import Path

try:
    import pyshark
except ImportError:
    sys.exit("Install pyshark: pip install pyshark")

try:
    import pandas as pd
except ImportError:
    sys.exit("Install pandas: pip install pandas")


SERVICES = {
    20: 'FTP-Data', 21: 'FTP', 22: 'SSH', 23: 'Telnet', 25: 'SMTP',
    53: 'DNS', 67: 'DHCP', 68: 'DHCP', 80: 'HTTP', 110: 'POP3',
    123: 'NTP', 143: 'IMAP', 161: 'SNMP', 443: 'HTTPS', 445: 'SMB',
    993: 'IMAPS', 995: 'POP3S', 1433: 'MSSQL', 1521: 'Oracle',
    3306: 'MySQL', 3389: 'RDP', 5432: 'PostgreSQL', 5900: 'VNC',
    6379: 'Redis', 8080: 'HTTP-Alt', 8443: 'HTTPS-Alt', 27017: 'MongoDB'
}


def analyze(pcap_path, output_json=None):
    print(f"\n{'='*60}")
    print(f"  PCAP Analysis: {pcap_path}")
    print(f"{'='*60}\n")

    cap = pyshark.FileCapture(pcap_path, keep_packets=True)
    cap.load_packets()

    if len(cap) == 0:
        print("No packets found.")
        return

    # Metadata
    first_time = cap[0].sniff_time
    last_time = cap[-1].sniff_time
    duration = last_time - first_time

    print(f"[Overview]")
    print(f"  Packets:    {len(cap)}")
    print(f"  First:      {first_time}")
    print(f"  Last:       {last_time}")
    print(f"  Duration:   {duration}")
    print()

    # Protocol breakdown
    proto_counts = Counter(pkt.highest_layer for pkt in cap)
    print(f"[Protocol Distribution]")
    for proto, count in proto_counts.most_common(15):
        pct = count / len(cap) * 100
        print(f"  {proto:<15} {count:>7} ({pct:.1f}%)")
    print()

    # Source/Destination
    ip_stats = defaultdict(lambda: {'sent': 0, 'recv': 0, 'pkts_out': 0, 'pkts_in': 0})
    port_stats = defaultdict(lambda: {'count': 0, 'bytes': 0, 'sources': set()})
    offender_data = defaultdict(lambda: {
        'bytes': 0, 'packets': 0, 'dst_ports': set(),
        'dst_ips': set(), 'syn': 0, 'rst': 0
    })

    for pkt in cap:
        try:
            src = pkt.ip.src
            dst = pkt.ip.dst
            length = int(pkt.length)

            ip_stats[src]['sent'] += length
            ip_stats[src]['pkts_out'] += 1
            ip_stats[dst]['recv'] += length
            ip_stats[dst]['pkts_in'] += 1

            offender_data[src]['bytes'] += length
            offender_data[src]['packets'] += 1
            offender_data[src]['dst_ips'].add(dst)

            if hasattr(pkt, 'tcp') or hasattr(pkt, 'udp'):
                layer = pkt.transport_layer
                if layer:
                    try:
                        dport = int(pkt[layer].dstport)
                        port_stats[dport]['count'] += 1
                        port_stats[dport]['bytes'] += length
                        port_stats[dport]['sources'].add(src)
                        offender_data[src]['dst_ports'].add(dport)
                    except (AttributeError, TypeError, ValueError):
                        pass

            if hasattr(pkt, 'tcp'):
                try:
                    flags = int(pkt.tcp.flags, 16)
                    if flags & 0x02:
                        offender_data[src]['syn'] += 1
                    if flags & 0x04:
                        offender_data[src]['rst'] += 1
                except (ValueError, AttributeError):
                    pass

        except AttributeError:
            continue

    # Top talkers
    print(f"[Top Talkers - By Bytes Sent]")
    sorted_ips = sorted(ip_stats.items(), key=lambda x: x[1]['sent'], reverse=True)
    print(f"  {'IP':<18} {'Sent':>10} {'Recv':>10} {'Pkts Out':>9} {'Pkts In':>8}")
    print(f"  {'-'*16:<18} {'-'*10} {'-'*10} {'-'*9} {'-'*8}")
    for ip, stats in sorted_ips[:15]:
        print(f"  {ip:<18} {stats['sent']/1024:>9.1f}K {stats['recv']/1024:>9.1f}K "
              f"{stats['pkts_out']:>9} {stats['pkts_in']:>8}")
    print()

    # Port analysis
    print(f"[Port Analysis - Top 20 by Traffic]")
    sorted_ports = sorted(port_stats.items(), key=lambda x: x[1]['bytes'], reverse=True)
    print(f"  {'Port':<7} {'Service':<12} {'Packets':>8} {'Bytes':>10} {'Sources':>8}")
    print(f"  {'-'*5:<7} {'-'*10:<12} {'-'*8} {'-'*10} {'-'*8}")
    for port, stats in sorted_ports[:20]:
        svc = SERVICES.get(port, '-')
        print(f"  {port:<7} {svc:<12} {stats['count']:>8} {stats['bytes']/1024:>9.1f}K "
              f"{len(stats['sources']):>8}")
    print()

    # Root offenders
    print(f"[Root Offenders]")
    scored = []
    for ip, d in offender_data.items():
        score = d['bytes'] / (1024*1024)
        score += len(d['dst_ports']) * 0.5
        score += len(d['dst_ips']) * 0.3
        score += d['syn'] * 0.1
        score += d['rst'] * 0.2

        flags = []
        if len(d['dst_ports']) > 50:
            flags.append('PORT_SCAN')
        if d['syn'] > 100 and d['rst'] > 50:
            flags.append('SYN_FLOOD')
        if len(d['dst_ips']) > 20:
            flags.append('BROAD_SCAN')
        if d['bytes'] > 10 * 1024 * 1024:
            flags.append('BANDWIDTH_HOG')

        scored.append({'ip': ip, 'score': score, 'flags': flags, **d})

    scored.sort(key=lambda x: x['score'], reverse=True)
    for entry in scored[:10]:
        flag_str = f" [{', '.join(entry['flags'])}]" if entry['flags'] else ""
        print(f"  {entry['ip']:<18} score={entry['score']:.2f}  "
              f"{entry['bytes']/1024:.0f}KB  {entry['packets']} pkts  "
              f"{len(entry['dst_ports'])} ports  {len(entry['dst_ips'])} dsts{flag_str}")
    print()

    # JSON export
    if output_json:
        report = {
            'metadata': {
                'file': str(pcap_path),
                'packets': len(cap),
                'duration_seconds': duration.total_seconds(),
                'first_packet': str(first_time),
                'last_packet': str(last_time),
            },
            'protocols': dict(proto_counts.most_common()),
            'top_talkers': [{'ip': ip, **{k: v for k, v in s.items()}}
                           for ip, s in sorted_ips[:20]],
            'top_ports': [{'port': p, 'service': SERVICES.get(p, ''),
                          'packets': s['count'], 'bytes': s['bytes'],
                          'unique_sources': len(s['sources'])}
                         for p, s in sorted_ports[:30]],
            'offenders': [{'ip': e['ip'], 'score': e['score'], 'flags': e['flags'],
                          'bytes': e['bytes'], 'packets': e['packets'],
                          'dst_ports': len(e['dst_ports']),
                          'dst_ips': len(e['dst_ips'])}
                         for e in scored[:20]],
        }
        Path(output_json).write_text(json.dumps(report, indent=2, default=str))
        print(f"[Report saved to {output_json}]")

    cap.close()


if __name__ == '__main__':
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} <pcap_file> [output.json]")
        sys.exit(1)

    pcap_file = sys.argv[1]
    json_out = sys.argv[2] if len(sys.argv) > 2 else None
    analyze(pcap_file, json_out)
