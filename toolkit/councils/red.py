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
        self._detect_beaconing(packets_df, local_ips)
        self._detect_volume_anomaly(packets_df, local_ips)
        self._detect_unusual_ports(packets_df, local_ips)
        self._detect_dns_tunnel(packets_df, local_ips)
        return self.findings

    def _detect_beaconing(self, df, local_ips):
        """Regular interval outbound = possible C2 callback."""
        outbound = df[df['src_ip'].isin(local_ips) & ~df['dst_ip'].isin(local_ips)]
        if len(outbound) < 10:
            return

        for dst_ip in outbound['dst_ip'].unique():
            subset = outbound[outbound['dst_ip'] == dst_ip].sort_values('time')
            if len(subset) < 5:
                continue
            intervals = subset['time'].diff().dt.total_seconds().dropna()
            intervals = intervals[intervals > 0.5]
            if len(intervals) < 4:
                continue

            mean = intervals.mean()
            std = intervals.std()
            if mean > 0 and std < mean * 0.35 and mean < 600:
                regularity = 1 - (std / mean)
                self.findings.append(RedFinding(
                    id=f"RED-BEACON-{dst_ip}",
                    severity='HIGH',
                    category='beaconing',
                    target=dst_ip,
                    hypothesis=f"Regular beaconing to {dst_ip} (interval {mean:.1f}s, sigma {std:.1f}s)",
                    evidence=f"{len(subset)} packets, regularity={regularity:.2f}, total {subset['length'].sum()/1024:.1f}KB",
                    confidence=min(0.9, regularity)
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
