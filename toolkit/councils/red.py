"""Red Council — Attribution / Hostile Intent Lens"""

from dataclasses import dataclass, field
from typing import List, Dict
import statistics


@dataclass
class RedFinding:
    id: str
    severity: str  # CRITICAL, HIGH, MEDIUM, LOW
    category: str  # beaconing, volume, port_scan, dns_tunnel, high_entropy
    target: str
    hypothesis: str
    evidence: str
    confidence: float


class RedCouncil:
    """
    Counter-clockwise council. Assumes malicious intent.
    Looks for: beaconing, exfiltration, port scanning, DNS tunneling, C2.
    """

    def __init__(self):
        self.findings: List[RedFinding] = []

    def analyze(self, packets_df, local_ips: set) -> List[RedFinding]:
        """Run all red analysis passes."""
        self.findings = []
        # Normalize port columns to handle NaN (float64 from pyshark)
        df = packets_df.copy()
        df['dst_port'] = df['dst_port'].fillna(0).astype(int)
        df['src_port'] = df['src_port'].fillna(0).astype(int)

        self._detect_beaconing(df, local_ips)
        self._detect_volume_anomaly(df, local_ips)
        self._detect_unusual_ports(df, local_ips)
        self._detect_dns_tunnel(df, local_ips)
        return self.findings

    def _detect_beaconing(self, df, local_ips):
        """Regular interval traffic = possible C2 callback or automated polling."""
        outbound = df[df['src_ip'].isin(local_ips)]
        if len(outbound) < 10:
            return

        # Detect beaconing per (dst_ip, dst_port) pair — connection-level, not packet-level
        # Use SYN packets or first-packet-per-flow to measure connection initiation intervals
        for (dst_ip, dst_port), group in outbound.groupby(['dst_ip', 'dst_port']):
            if len(group) < 5 or dst_port == 0:
                continue

            sorted_times = group['time'].sort_values()
            # For flow-level: use gaps > 1s to identify distinct connection starts
            intervals = sorted_times.diff().dt.total_seconds().dropna()
            flow_intervals = intervals[intervals > 1.0]

            if len(flow_intervals) < 4:
                continue

            mean = flow_intervals.mean()
            std = flow_intervals.std()

            if mean > 2 and std < mean * 0.4 and mean < 600:
                regularity = 1 - (std / mean)
                self.findings.append(RedFinding(
                    id=f"RED-BEACON-{dst_ip}-{dst_port}",
                    severity='HIGH',
                    category='beaconing',
                    target=f"{dst_ip}:{dst_port}",
                    hypothesis=f"Regular beaconing to {dst_ip}:{dst_port} (interval {mean:.1f}s, sigma {std:.1f}s)",
                    evidence=f"{len(group)} packets in {len(flow_intervals)} flows, regularity={regularity:.2f}",
                    confidence=min(0.9, regularity)
                ))

        # Also check per-destination (ignoring port) with larger gap threshold
        for dst_ip in outbound['dst_ip'].unique():
            subset = outbound[outbound['dst_ip'] == dst_ip].sort_values('time')
            if len(subset) < 20:
                continue
            intervals = subset['time'].diff().dt.total_seconds().dropna()
            flow_gaps = intervals[intervals > 3.0]
            if len(flow_gaps) < 4:
                continue

            mean = flow_gaps.mean()
            std = flow_gaps.std()
            if mean > 3 and std < mean * 0.3 and mean < 600:
                regularity = 1 - (std / mean)
                if not any(f.target.startswith(dst_ip) for f in self.findings):
                    self.findings.append(RedFinding(
                        id=f"RED-BEACON-AGG-{dst_ip}",
                        severity='HIGH',
                        category='beaconing',
                        target=dst_ip,
                        hypothesis=f"Aggregate beaconing to {dst_ip} (interval {mean:.1f}s, sigma {std:.1f}s)",
                        evidence=f"{len(subset)} total packets, {len(flow_gaps)} flow gaps measured",
                        confidence=min(0.85, regularity)
                    ))

    def _detect_volume_anomaly(self, df, local_ips):
        """Single destination receiving disproportionate outbound traffic."""
        outbound = df[df['src_ip'].isin(local_ips) & ~df['dst_ip'].isin(local_ips)]
        if len(outbound) == 0:
            return

        dst_vol = outbound.groupby('dst_ip')['length'].sum()
        total = dst_vol.sum()
        if total < 10000:
            return

        for ip, vol in dst_vol.items():
            if vol > total * 0.4:
                self.findings.append(RedFinding(
                    id=f"RED-VOLUME-{ip}",
                    severity='HIGH',
                    category='volume',
                    target=ip,
                    hypothesis=f"{ip} receiving {vol/total*100:.1f}% of all outbound ({vol/1024:.1f}KB)",
                    evidence=f"Disproportionate data flow to single destination",
                    confidence=vol / total
                ))

    def _detect_unusual_ports(self, df, local_ips):
        """Non-standard ports in use."""
        outbound = df[df['src_ip'].isin(local_ips) & ~df['dst_ip'].isin(local_ips)]
        standard = {80, 443, 53, 22, 123, 993, 995, 587, 465, 8080, 8443}
        unusual = outbound[~outbound['dst_port'].isin(standard)]
        port_counts = unusual.groupby('dst_port').size()

        if len(port_counts) > 5:
            top_ports = port_counts.sort_values(ascending=False).head(5)
            self.findings.append(RedFinding(
                id="RED-PORTS",
                severity='MEDIUM',
                category='port_scan',
                target='Multiple',
                hypothesis=f"{len(port_counts)} non-standard ports in use: {', '.join(str(int(p)) for p in top_ports.index)}",
                evidence=f"May indicate port-hopping evasion or tunneling",
                confidence=0.5
            ))

    def _detect_dns_tunnel(self, df, local_ips):
        """Oversized DNS payloads suggest tunneling."""
        dns = df[(df['dst_port'] == 53) & (df['src_ip'].isin(local_ips))]
        if len(dns) == 0:
            return
        large = dns[dns['payload_size'] > 50]
        if len(large) > len(dns) * 0.3:
            self.findings.append(RedFinding(
                id="RED-DNS-TUNNEL",
                severity='CRITICAL',
                category='dns_tunnel',
                target='DNS infrastructure',
                hypothesis=f"Oversized DNS payloads ({len(large)}/{len(dns)} > 50B)",
                evidence="DNS tunneling typically uses large TXT queries to encode data",
                confidence=len(large) / max(len(dns), 1)
            ))
