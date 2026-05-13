"""Blue Council — Innocence / Benign Lens"""

from dataclasses import dataclass
from typing import List


@dataclass
class BlueFinding:
    id: str
    severity: str  # INFO, LOW
    category: str  # https_normal, infrastructure, low_volume, keepalive, known_service
    target: str
    hypothesis: str
    evidence: str
    explanation: str
    confidence: float


class BlueCouncil:
    """
    Clockwise council. Assumes benign activity.
    Looks for: standard HTTPS, infrastructure DNS/NTP, known services, keepalives, low bandwidth.
    """

    def __init__(self):
        self.findings: List[BlueFinding] = []

    def analyze(self, packets_df, local_ips: set) -> List[BlueFinding]:
        """Run all blue analysis passes."""
        self.findings = []
        self._assess_https_dominance(packets_df, local_ips)
        self._assess_infrastructure(packets_df, local_ips)
        self._assess_bandwidth(packets_df, local_ips)
        self._assess_keepalives(packets_df, local_ips)
        return self.findings

    def _assess_https_dominance(self, df, local_ips):
        """HTTPS-dominant traffic = normal browsing/apps."""
        outbound = df[df['src_ip'].isin(local_ips) & ~df['dst_ip'].isin(local_ips)]
        if len(outbound) == 0:
            return
        https_pct = len(outbound[outbound['dst_port'] == 443]) / len(outbound)
        if https_pct > 0.4:
            self.findings.append(BlueFinding(
                id="BLUE-HTTPS",
                severity='INFO',
                category='https_normal',
                target='Multiple',
                hypothesis=f"{https_pct*100:.0f}% of outbound is HTTPS — standard encrypted traffic",
                evidence=f"Consistent with normal browser/application behavior",
                explanation="Modern apps use HTTPS for all communication. High HTTPS percentage is expected.",
                confidence=https_pct
            ))

    def _assess_infrastructure(self, df, local_ips):
        """DNS, NTP, DHCP are required network services."""
        outbound = df[df['src_ip'].isin(local_ips)]
        infra_ports = {53, 123, 67, 68, 5353}
        infra = outbound[outbound['dst_port'].isin(infra_ports)]
        if len(infra) > 0:
            self.findings.append(BlueFinding(
                id="BLUE-INFRA",
                severity='INFO',
                category='infrastructure',
                target='DNS/NTP/DHCP',
                hypothesis=f"{len(infra)} infrastructure packets — required for network operation",
                evidence="Name resolution and time synchronization are baseline requirements",
                explanation="Every device on a network must resolve names (DNS) and sync time (NTP).",
                confidence=0.95
            ))

    def _assess_bandwidth(self, df, local_ips):
        """Low bandwidth = not consistent with exfiltration."""
        outbound = df[df['src_ip'].isin(local_ips) & ~df['dst_ip'].isin(local_ips)]
        if len(df) < 2:
            return
        total_bytes = outbound['length'].sum()
        duration = (df['time'].max() - df['time'].min()).total_seconds()
        if duration < 1:
            return
        rate = total_bytes / duration
        if rate < 5000:
            self.findings.append(BlueFinding(
                id="BLUE-LOWVOL",
                severity='INFO',
                category='low_volume',
                target='Overall',
                hypothesis=f"Average outbound rate: {rate:.0f} B/s ({rate/1024:.2f} KB/s) — below exfil thresholds",
                evidence="Active exfiltration typically requires sustained high throughput",
                explanation="This bandwidth level is consistent with idle telemetry, not data theft.",
                confidence=0.8
            ))

    def _assess_keepalives(self, df, local_ips):
        """Regular intervals matching known keepalive periods."""
        outbound = df[df['src_ip'].isin(local_ips) & ~df['dst_ip'].isin(local_ips)]
        common_keepalives = [15, 30, 60, 120, 300]

        for dst_ip in outbound['dst_ip'].unique():
            subset = outbound[outbound['dst_ip'] == dst_ip].sort_values('time')
            if len(subset) < 5:
                continue
            intervals = subset['time'].diff().dt.total_seconds().dropna()
            intervals = intervals[intervals > 1]
            if len(intervals) < 3:
                continue
            mean = intervals.mean()
            for ka in common_keepalives:
                if abs(mean - ka) < ka * 0.2:
                    self.findings.append(BlueFinding(
                        id=f"BLUE-KA-{dst_ip}",
                        severity='INFO',
                        category='keepalive',
                        target=dst_ip,
                        hypothesis=f"Interval ~{mean:.0f}s matches standard {ka}s keepalive",
                        evidence="TCP keepalive and application heartbeats use fixed intervals",
                        explanation=f"Applications maintain connections with periodic pings at {ka}s intervals.",
                        confidence=0.7
                    ))
                    break
