"""PCAP Parser — Converts pcap/pcapng to structured DataFrame."""

import pandas as pd
from pathlib import Path
from typing import Optional


class PcapParser:
    """Parse pcap files into analysis-ready DataFrames. No network calls."""

    def __init__(self):
        self.packets_parsed = 0

    def parse(self, pcap_path: str) -> pd.DataFrame:
        """Parse a pcap file into a DataFrame with standard columns."""
        import pyshark

        cap = pyshark.FileCapture(str(pcap_path), keep_packets=True)
        cap.load_packets()

        records = []
        for i, pkt in enumerate(cap):
            rec = {
                'idx': i,
                'time': pkt.sniff_time,
                'length': int(pkt.length),
                'protocol': pkt.highest_layer,
                'src_ip': None,
                'dst_ip': None,
                'src_port': None,
                'dst_port': None,
                'transport': None,
                'payload_size': 0
            }

            try:
                rec['src_ip'] = pkt.ip.src
                rec['dst_ip'] = pkt.ip.dst
            except AttributeError:
                pass

            try:
                transport = pkt.transport_layer
                if transport:
                    rec['transport'] = transport
                    rec['src_port'] = int(pkt[transport].srcport)
                    rec['dst_port'] = int(pkt[transport].dstport)
            except (AttributeError, TypeError, ValueError):
                pass

            try:
                if hasattr(pkt, 'data') and hasattr(pkt.data, 'data'):
                    rec['payload_size'] = len(pkt.data.data.replace(':', '')) // 2
                elif hasattr(pkt, 'tcp') and hasattr(pkt.tcp, 'payload'):
                    rec['payload_size'] = len(pkt.tcp.payload.replace(':', '')) // 2
            except (ValueError, AttributeError):
                pass

            records.append(rec)

        cap.close()
        self.packets_parsed += len(records)

        df = pd.DataFrame(records)
        return df

    def parse_multiple(self, paths: list) -> pd.DataFrame:
        """Parse multiple pcap files into a single DataFrame."""
        dfs = []
        for p in paths:
            if Path(p).exists():
                dfs.append(self.parse(p))
        if dfs:
            return pd.concat(dfs, ignore_index=True)
        return pd.DataFrame()
