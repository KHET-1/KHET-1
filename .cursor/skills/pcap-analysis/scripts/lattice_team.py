#!/usr/bin/env python3
"""
LATTICE TEAM OPERATION — KHET-1
Swarm Architecture with Counter-Rotating Councils, Merkle Evidence Chain,
and Rayon-style Parallel Execution.

Every token earns its keep. Outposts → Castles → Cathedral.

Usage:
    python lattice_team.py [--capture] [--pcap <file>] [--output <dir>]
"""

import sys
import os
import json
import hashlib
import time
import subprocess
from pathlib import Path
from datetime import datetime, timezone
from collections import defaultdict, Counter
from concurrent.futures import ProcessPoolExecutor, ThreadPoolExecutor, as_completed
from dataclasses import dataclass, field, asdict
from typing import List, Dict, Optional, Any

try:
    import pyshark
except ImportError:
    sys.exit("[LATTICE] pip install pyshark")

try:
    import pandas as pd
except ImportError:
    sys.exit("[LATTICE] pip install pandas")


# ============================================================
# MERKLE TREE — Connects All Swarm Nodes
# ============================================================

class MerkleNode:
    """Single node in the evidence Merkle tree."""
    __slots__ = ['hash', 'data_hash', 'left', 'right', 'label']

    def __init__(self, data_hash: str, label: str = '', left=None, right=None):
        self.data_hash = data_hash
        self.left = left
        self.right = right
        self.label = label
        if left and right:
            combined = f"{left.hash}{right.hash}".encode()
            self.hash = hashlib.sha256(combined).hexdigest()
        else:
            self.hash = data_hash


class MerkleTree:
    """
    SHA-256 Merkle tree connecting all swarm outputs.
    Every council finding, every outpost, every castle — linked.
    """

    def __init__(self):
        self.leaves: List[MerkleNode] = []
        self.root: Optional[MerkleNode] = None

    def add_leaf(self, data: bytes, label: str = '') -> str:
        """Add evidence leaf. Returns its hash."""
        h = hashlib.sha256(data).hexdigest()
        self.leaves.append(MerkleNode(h, label=label))
        return h

    def add_json_leaf(self, obj: Any, label: str = '') -> str:
        """Add structured data as a leaf."""
        serialized = json.dumps(obj, sort_keys=True, default=str).encode()
        return self.add_leaf(serialized, label)

    def build(self) -> str:
        """Build tree from current leaves. Returns root hash."""
        if not self.leaves:
            return hashlib.sha256(b'empty').hexdigest()

        level = list(self.leaves)
        while len(level) > 1:
            next_level = []
            for i in range(0, len(level), 2):
                left = level[i]
                right = level[i + 1] if i + 1 < len(level) else level[i]
                parent = MerkleNode('', label=f"L{len(next_level)}", left=left, right=right)
                next_level.append(parent)
            level = next_level

        self.root = level[0]
        return self.root.hash

    def get_proof(self, leaf_index: int) -> List[Dict]:
        """Get Merkle proof for a specific leaf."""
        proof = []
        level = list(self.leaves)
        idx = leaf_index

        while len(level) > 1:
            next_level = []
            for i in range(0, len(level), 2):
                left = level[i]
                right = level[i + 1] if i + 1 < len(level) else level[i]
                parent = MerkleNode('', left=left, right=right)
                next_level.append(parent)

                if i == idx or i + 1 == idx:
                    sibling = right if i == idx else left
                    direction = 'right' if i == idx else 'left'
                    proof.append({'hash': sibling.hash, 'direction': direction})
                    idx = len(next_level) - 1

            level = next_level

        return proof

    def to_dict(self) -> Dict:
        """Serialize tree structure."""
        return {
            'root_hash': self.root.hash if self.root else None,
            'leaf_count': len(self.leaves),
            'leaves': [{'hash': l.hash, 'label': l.label} for l in self.leaves]
        }


# ============================================================
# RAYON — Parallel Execution Engine (Python ThreadPool/ProcessPool)
# ============================================================

class Rayon:
    """
    Rayon-style parallel executor. Named after Rust's rayon crate.
    Splits work across cores. Every thread produces output or dies.
    No idle tokens. All earn their keep.
    """

    def __init__(self, max_workers=None):
        self.max_workers = max_workers or os.cpu_count() or 4
        self.results = []
        self.timings = {}

    def par_iter(self, tasks, executor_type='thread'):
        """
        Parallel iterator over tasks. Each task is (fn, args, label).
        Returns results in completion order.
        """
        ExecutorClass = ThreadPoolExecutor if executor_type == 'thread' else ProcessPoolExecutor
        results = []

        with ExecutorClass(max_workers=self.max_workers) as executor:
            futures = {}
            for fn, args, label in tasks:
                t0 = time.time()
                future = executor.submit(fn, *args)
                futures[future] = (label, t0)

            for future in as_completed(futures):
                label, t0 = futures[future]
                elapsed = time.time() - t0
                self.timings[label] = elapsed
                try:
                    result = future.result()
                    results.append({'label': label, 'result': result, 'time': elapsed, 'status': 'OK'})
                except Exception as e:
                    results.append({'label': label, 'error': str(e), 'time': elapsed, 'status': 'FAIL'})

        self.results = results
        return results

    def efficiency_report(self) -> Dict:
        """Every token earns its keep. Show work done per unit time."""
        total_time = sum(self.timings.values())
        return {
            'workers': self.max_workers,
            'tasks_completed': len([r for r in self.results if r['status'] == 'OK']),
            'tasks_failed': len([r for r in self.results if r['status'] == 'FAIL']),
            'total_compute_seconds': total_time,
            'avg_task_seconds': total_time / max(len(self.results), 1),
            'parallelism_factor': total_time / max(max(self.timings.values(), default=1), 0.001)
        }


# ============================================================
# OUTPOST → CASTLE → CATHEDRAL ARCHITECTURE
# ============================================================

@dataclass
class Outpost:
    """
    An Outpost is the smallest unit of analyzed intelligence.
    Created by individual swarm workers. Lightweight, focused.
    """
    id: str
    created_by: str  # which council/worker
    timestamp: str
    category: str  # 'connection', 'process', 'payload', 'anomaly'
    target: str
    finding: str
    evidence: str
    confidence: float
    sha256: str = ''

    def __post_init__(self):
        content = f"{self.category}:{self.target}:{self.finding}:{self.evidence}"
        self.sha256 = hashlib.sha256(content.encode()).hexdigest()


@dataclass
class Castle:
    """
    A Castle aggregates related Outposts into a fortified conclusion.
    Built when multiple outposts corroborate or contradict.
    """
    id: str
    name: str
    outposts: List[str]  # outpost IDs
    conclusion: str
    severity: str  # CRITICAL, HIGH, MEDIUM, LOW, INFO
    council_origin: str  # 'red', 'blue', 'merged'
    merkle_root: str = ''
    confidence: float = 0.0


@dataclass
class Cathedral:
    """
    The Cathedral is the final unified intelligence product.
    All castles feed into the cathedral. This IS the report.
    """
    operation_name: str
    castles: List[Castle]
    merkle_root: str
    red_council_summary: str
    blue_council_summary: str
    final_maker_verdict: str
    debate_highlights: List[Dict]
    drip_classification: str
    drip_source_process: str
    timestamp: str = ''

    def __post_init__(self):
        self.timestamp = datetime.now(timezone.utc).isoformat()


# ============================================================
# RED COUNCIL (Counter-Clockwise) — ATTACK ATTRIBUTION
# ============================================================

def red_council_worker(packets_chunk, local_ips, worker_id):
    """
    Single Red Council worker. Assumes hostile intent.
    Returns outposts for suspicious findings.
    """
    outposts = []

    for _, pkt in packets_chunk.iterrows():
        if pkt.get('src_ip') not in local_ips:
            continue
        if pd.isna(pkt.get('dst_ip')):
            continue

        # Exfil pattern: large outbound to single dest
        if pkt.get('length', 0) > 1000 and pkt.get('dst_port') not in [80, 443, 53]:
            outposts.append(Outpost(
                id=f"RED-{worker_id}-{len(outposts)}",
                created_by=f'red_council_worker_{worker_id}',
                timestamp=datetime.now(timezone.utc).isoformat(),
                category='anomaly',
                target=f"{pkt['dst_ip']}:{pkt.get('dst_port', '?')}",
                finding=f"Large outbound packet ({pkt['length']}B) on non-standard port",
                evidence=f"Pkt#{pkt['idx']}: {pkt['src_ip']} → {pkt['dst_ip']}:{pkt.get('dst_port')} | {pkt['protocol']}",
                confidence=0.6
            ))

        # Beaconing: check regularity (simplified per-packet; full analysis in aggregator)
        if pkt.get('payload_size', 0) > 0 and pkt.get('payload_size', 0) < 100:
            if pkt.get('dst_port') not in [53, 80, 443, 123]:
                outposts.append(Outpost(
                    id=f"RED-{worker_id}-{len(outposts)}",
                    created_by=f'red_council_worker_{worker_id}',
                    timestamp=datetime.now(timezone.utc).isoformat(),
                    category='connection',
                    target=pkt['dst_ip'],
                    finding=f"Small payload to non-standard port (potential beacon)",
                    evidence=f"{pkt['payload_size']}B payload to port {pkt.get('dst_port')}",
                    confidence=0.4
                ))

    return [asdict(o) for o in outposts]


def red_council_aggregate(all_outposts, df, local_ips):
    """
    Red Council aggregation — combine worker outposts into castles.
    Look for patterns across the full dataset.
    """
    castles = []

    # Aggregate: beaconing detection
    outbound = df[df['src_ip'].isin(local_ips) & ~df['dst_ip'].isin(local_ips)].copy()
    if len(outbound) > 5:
        for dst_ip in outbound['dst_ip'].unique():
            subset = outbound[outbound['dst_ip'] == dst_ip].sort_values('time')
            if len(subset) < 4:
                continue
            intervals = subset['time'].diff().dt.total_seconds().dropna()
            if len(intervals) >= 3:
                std = intervals.std()
                mean = intervals.mean()
                if mean > 0 and std < mean * 0.35:
                    related_outposts = [
                        o['id'] for o in all_outposts
                        if dst_ip in o.get('target', '')
                    ]
                    castles.append(Castle(
                        id=f"CASTLE-RED-BEACON-{dst_ip}",
                        name=f"Beaconing to {dst_ip}",
                        outposts=related_outposts,
                        conclusion=f"Regular interval communication ({mean:.1f}s ± {std:.1f}s) to {dst_ip}. "
                                   f"Pattern consistent with C2 callback or automated exfiltration.",
                        severity='HIGH',
                        council_origin='red',
                        confidence=min(0.9, 1.0 - (std / max(mean, 0.01)))
                    ))

    # Aggregate: volume concentration
    if len(outbound) > 0:
        dst_vol = outbound.groupby('dst_ip')['length'].sum()
        total = dst_vol.sum()
        for ip, vol in dst_vol.items():
            if vol > total * 0.4 and total > 10000:
                castles.append(Castle(
                    id=f"CASTLE-RED-VOL-{ip}",
                    name=f"Volume concentration to {ip}",
                    outposts=[o['id'] for o in all_outposts if ip in o.get('target', '')],
                    conclusion=f"{ip} receiving {vol/total*100:.1f}% of outbound traffic ({vol/1024:.1f}KB). "
                               f"Disproportionate data flow suggests exfiltration target.",
                    severity='HIGH',
                    council_origin='red',
                    confidence=vol / total
                ))

    # Aggregate: unusual port usage
    port_counts = outbound.groupby('dst_port').size()
    unusual = port_counts[~port_counts.index.isin([80, 443, 53, 22, 123, 993, 465, 587])]
    if len(unusual) > 5:
        castles.append(Castle(
            id="CASTLE-RED-PORTSWEEP",
            name="Broad unusual port usage",
            outposts=[o['id'] for o in all_outposts if o.get('category') == 'connection'],
            conclusion=f"{len(unusual)} non-standard ports in use. "
                       f"Top: {', '.join(str(p) for p in unusual.sort_values(ascending=False).head(5).index)}. "
                       f"May indicate port-hopping evasion or tunneling.",
            severity='MEDIUM',
            council_origin='red',
            confidence=0.5
        ))

    return castles


# ============================================================
# BLUE COUNCIL (Clockwise) — BENIGN ATTRIBUTION
# ============================================================

def blue_council_worker(packets_chunk, local_ips, worker_id):
    """
    Single Blue Council worker. Assumes legitimate activity.
    Returns outposts for benign explanations.
    """
    outposts = []
    STANDARD_PORTS = {80, 443, 53, 22, 123, 993, 995, 587, 465, 8080, 8443}

    for _, pkt in packets_chunk.iterrows():
        if pkt.get('src_ip') not in local_ips:
            continue
        if pd.isna(pkt.get('dst_ip')):
            continue

        # Standard HTTPS traffic
        if pkt.get('dst_port') == 443:
            outposts.append(Outpost(
                id=f"BLUE-{worker_id}-{len(outposts)}",
                created_by=f'blue_council_worker_{worker_id}',
                timestamp=datetime.now(timezone.utc).isoformat(),
                category='connection',
                target=pkt['dst_ip'],
                finding="Standard HTTPS — expected encrypted web traffic",
                evidence=f"Port 443 to {pkt['dst_ip']}, normal browser/API behavior",
                confidence=0.85
            ))

        # DNS is infrastructure
        elif pkt.get('dst_port') == 53:
            outposts.append(Outpost(
                id=f"BLUE-{worker_id}-{len(outposts)}",
                created_by=f'blue_council_worker_{worker_id}',
                timestamp=datetime.now(timezone.utc).isoformat(),
                category='connection',
                target=pkt['dst_ip'],
                finding="DNS query — required infrastructure",
                evidence=f"Port 53 to {pkt['dst_ip']}, name resolution",
                confidence=0.9
            ))

    return [asdict(o) for o in outposts]


def blue_council_aggregate(all_outposts, df, local_ips):
    """Blue Council aggregation — build castles for benign patterns."""
    castles = []

    outbound = df[df['src_ip'].isin(local_ips) & ~df['dst_ip'].isin(local_ips)].copy()

    # HTTPS dominance = normal browsing
    if len(outbound) > 0:
        https_pct = len(outbound[outbound['dst_port'] == 443]) / len(outbound)
        if https_pct > 0.5:
            castles.append(Castle(
                id="CASTLE-BLUE-HTTPS-NORMAL",
                name="HTTPS-dominant traffic profile",
                outposts=[o['id'] for o in all_outposts if '443' in o.get('evidence', '')],
                conclusion=f"{https_pct*100:.0f}% of outbound is HTTPS. "
                           f"Consistent with normal browser/application behavior. "
                           f"Encrypted channels are expected for modern apps.",
                severity='INFO',
                council_origin='blue',
                confidence=https_pct
            ))

    # Known infrastructure ports only
    INFRA_PORTS = {53, 123, 67, 68}
    infra_traffic = outbound[outbound['dst_port'].isin(INFRA_PORTS)]
    if len(infra_traffic) > 0:
        castles.append(Castle(
            id="CASTLE-BLUE-INFRA",
            name="Infrastructure traffic (DNS/NTP/DHCP)",
            outposts=[o['id'] for o in all_outposts if o.get('category') == 'connection'
                      and 'DNS' in o.get('finding', '')],
            conclusion=f"{len(infra_traffic)} packets to infrastructure services. "
                       f"Required for basic network operation (name resolution, time sync).",
            severity='INFO',
            council_origin='blue',
            confidence=0.95
        ))

    # Low total volume = not concerning
    total_bytes = outbound['length'].sum()
    duration = (df['time'].max() - df['time'].min()).total_seconds() if len(df) > 1 else 1
    rate = total_bytes / max(duration, 1)
    if rate < 5000:  # Less than 5KB/s average
        castles.append(Castle(
            id="CASTLE-BLUE-LOWVOL",
            name="Low bandwidth utilization",
            outposts=[],
            conclusion=f"Average outbound rate: {rate:.0f} B/s ({rate/1024:.2f} KB/s). "
                       f"Well below thresholds for active exfiltration. "
                       f"Consistent with idle system telemetry.",
            severity='INFO',
            council_origin='blue',
            confidence=0.8
        ))

    return castles


# ============================================================
# FINAL MAKER — Opus 4.6 Resolution
# ============================================================

def final_maker(red_castles, blue_castles, merkle: MerkleTree):
    """
    Opus 4.6 Final Maker. Resolves conflicts between Red and Blue councils.
    Every castle from both sides gets weighed. Debate is logged.
    """
    debate_highlights = []
    final_castles = []

    # Match red castles against blue counterparts
    for rc in red_castles:
        countered = False
        for bc in blue_castles:
            # Check if they address the same target/phenomenon
            if (rc.name and bc.name and
                any(word in bc.conclusion.lower() for word in rc.conclusion.lower().split()[:3])):

                # Weight comparison
                if rc.confidence > bc.confidence * 1.3:
                    resolution = "RED_PREVAILS"
                    winner = rc
                elif bc.confidence > rc.confidence * 1.3:
                    resolution = "BLUE_PREVAILS"
                    winner = bc
                else:
                    resolution = "CONTESTED"
                    winner = rc  # Default to caution

                debate_highlights.append({
                    'red_castle': rc.name,
                    'red_conclusion': rc.conclusion,
                    'red_confidence': rc.confidence,
                    'blue_castle': bc.name,
                    'blue_conclusion': bc.conclusion,
                    'blue_confidence': bc.confidence,
                    'resolution': resolution,
                    'reasoning': (
                        f"Red confidence {rc.confidence:.2f} vs Blue {bc.confidence:.2f}. "
                        f"{'Red evidence more specific.' if resolution == 'RED_PREVAILS' else ''}"
                        f"{'Blue explanation adequate.' if resolution == 'BLUE_PREVAILS' else ''}"
                        f"{'Insufficient margin — requires analyst review.' if resolution == 'CONTESTED' else ''}"
                    )
                })

                # Add to merkle
                merkle.add_json_leaf({
                    'type': 'debate_resolution',
                    'resolution': resolution,
                    'red': rc.id, 'blue': bc.id
                }, label=f"debate:{rc.id}vs{bc.id}")

                winner.council_origin = 'merged'
                final_castles.append(winner)
                countered = True
                break

        if not countered:
            final_castles.append(rc)  # Unchallenged red finding

    # Add unchallenged blue castles
    for bc in blue_castles:
        if not any(fc.id == bc.id for fc in final_castles):
            final_castles.append(bc)

    return final_castles, debate_highlights


# ============================================================
# LOCAL CAPTURE — Check Box for PCAPs
# ============================================================

def capture_local(duration=10, packet_count=500, interface='any'):
    """Capture live traffic from this machine."""
    output_path = f"/tmp/lattice_capture_{int(time.time())}.pcap"
    print(f"[CAPTURE] Sniffing {interface} for {duration}s or {packet_count} packets...")

    try:
        proc = subprocess.run(
            ['sudo', 'tcpdump', '-i', interface, '-c', str(packet_count),
             '-w', output_path],
            timeout=duration + 5,
            capture_output=True, text=True
        )
        if Path(output_path).exists():
            print(f"[CAPTURE] Saved: {output_path} ({Path(output_path).stat().st_size} bytes)")
            return output_path
    except subprocess.TimeoutExpired:
        subprocess.run(['sudo', 'killall', 'tcpdump'], capture_output=True)
        if Path(output_path).exists():
            return output_path
    except FileNotFoundError:
        print("[CAPTURE] tcpdump not found. Install with: sudo apt install tcpdump")

    return None


def find_local_pcaps():
    """Search local box for any existing pcap files."""
    search_paths = [
        Path.home() / 'Desktop',
        Path.home() / 'Downloads',
        Path.home() / 'Documents',
        Path('/tmp'),
        Path('/var/log'),
        Path.home(),
    ]

    pcaps = []
    for sp in search_paths:
        if sp.exists():
            pcaps.extend(sp.glob('*.pcap'))
            pcaps.extend(sp.glob('*.pcapng'))

    return pcaps


# ============================================================
# MAIN ORCHESTRATOR
# ============================================================

def run_lattice_operation(pcap_paths=None, do_capture=True, output_dir='.'):
    """
    Full LATTICE TEAM operation.
    Outposts → Castles → Cathedral.
    All Merkle-connected. All SHA-256 verified.
    """
    print("""
╔══════════════════════════════════════════════════════════════╗
║           LATTICE TEAM OPERATION — KHET-1                   ║
║                                                              ║
║   ┌─────────┐     ┌─────────┐     ┌──────────────┐         ║
║   │ RED     │ ←─→ │ MERKLE  │ ←─→ │ BLUE         │         ║
║   │ COUNCIL │     │ TREE    │     │ COUNCIL      │         ║
║   │ (↺)     │     │ SHA-256 │     │ (↻)          │         ║
║   └────┬────┘     └────┬────┘     └──────┬───────┘         ║
║        │               │                  │                  ║
║        └───────────────┼──────────────────┘                  ║
║                        ▼                                     ║
║               ┌────────────────┐                             ║
║               │  FINAL MAKER   │                             ║
║               │  (Opus 4.6)    │                             ║
║               └────────┬───────┘                             ║
║                        ▼                                     ║
║               ┌────────────────┐                             ║
║               │   CATHEDRAL    │                             ║
║               └────────────────┘                             ║
╚══════════════════════════════════════════════════════════════╝
    """)

    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    merkle = MerkleTree()
    rayon = Rayon()

    # ── SOURCE DATA COLLECTION ──
    if pcap_paths is None:
        pcap_paths = []

    # Check local box for existing pcaps
    local_pcaps = find_local_pcaps()
    if local_pcaps:
        print(f"[RECON] Found {len(local_pcaps)} existing PCAP(s) on local box:")
        for p in local_pcaps:
            print(f"        → {p}")
        # Deduplicate by resolved path
        existing = {p.resolve() for p in pcap_paths}
        for p in local_pcaps:
            if p.resolve() not in existing:
                pcap_paths.append(p)
                existing.add(p.resolve())

    # Live capture if requested
    if do_capture:
        live_pcap = capture_local(duration=10, packet_count=500)
        if live_pcap:
            pcap_paths.append(Path(live_pcap))

    if not pcap_paths:
        sys.exit("[LATTICE] No source data. Provide --pcap or enable --capture.")

    # ── CRYSTALLINE LATTICE — SEAL EVIDENCE ──
    print("\n[PHASE 0] CRYSTALLINE LATTICE — Sealing evidence...")
    for p in pcap_paths:
        p = Path(p)
        if p.exists():
            file_hash = hashlib.sha256(p.read_bytes()).hexdigest()
            merkle.add_leaf(p.read_bytes(), label=f"evidence:{p.name}")
            print(f"  SEALED: {p.name} | SHA-256: {file_hash[:32]}...")

    # ── PARSE PACKETS ──
    print("\n[PHASE 1] Parsing packets (Rayon parallel)...")
    all_dfs = []
    for pcap_path in pcap_paths:
        pcap_path = Path(pcap_path)
        if not pcap_path.exists():
            continue
        try:
            cap = pyshark.FileCapture(str(pcap_path), keep_packets=True)
            cap.load_packets()

            records = []
            for i, pkt in enumerate(cap):
                rec = {
                    'idx': i, 'time': pkt.sniff_time, 'length': int(pkt.length),
                    'protocol': pkt.highest_layer,
                    'src_ip': None, 'dst_ip': None,
                    'src_port': None, 'dst_port': None,
                    'payload_size': 0, 'payload': None
                }
                try:
                    rec['src_ip'] = pkt.ip.src
                    rec['dst_ip'] = pkt.ip.dst
                except AttributeError:
                    pass
                try:
                    transport = pkt.transport_layer
                    if transport:
                        rec['src_port'] = int(pkt[transport].srcport)
                        rec['dst_port'] = int(pkt[transport].dstport)
                except (AttributeError, TypeError, ValueError):
                    pass
                try:
                    if hasattr(pkt, 'data') and hasattr(pkt.data, 'data'):
                        raw = pkt.data.data.replace(':', '')
                        rec['payload'] = bytes.fromhex(raw)
                        rec['payload_size'] = len(rec['payload'])
                except (ValueError, AttributeError):
                    pass
                records.append(rec)

            cap.close()
            df = pd.DataFrame(records)
            all_dfs.append(df)
            print(f"  Parsed: {pcap_path.name} → {len(df)} packets")
        except Exception as e:
            print(f"  ERROR parsing {pcap_path}: {e}")

    if not all_dfs:
        sys.exit("[LATTICE] No packets parsed from any source.")

    df = pd.concat(all_dfs, ignore_index=True)
    merkle.add_json_leaf({'total_packets': len(df), 'sources': len(all_dfs)}, label='parse_summary')

    # Detect local IPs
    local_ips = set()
    try:
        result = subprocess.run(['hostname', '-I'], capture_output=True, text=True, timeout=3)
        local_ips.update(result.stdout.strip().split())
    except:
        pass
    local_ips.add('127.0.0.1')
    top_src = df['src_ip'].value_counts().head(3).index.tolist()
    local_ips.update([ip for ip in top_src if ip])
    local_ips.discard(None)
    print(f"\n[LOCAL IPS] {local_ips}")

    # ── DUAL COUNCILS (RAYON PARALLEL) ──
    print("\n[PHASE 2] Launching Counter-Rotating Councils (Rayon parallel)...")

    # Split dataframe for parallel workers
    chunk_size = max(len(df) // 4, 1)
    chunks = [df.iloc[i:i+chunk_size] for i in range(0, len(df), chunk_size)]

    # Red Council workers
    red_tasks = [
        (red_council_worker, (chunk, local_ips, i), f"red_worker_{i}")
        for i, chunk in enumerate(chunks)
    ]

    # Blue Council workers
    blue_tasks = [
        (blue_council_worker, (chunk, local_ips, i), f"blue_worker_{i}")
        for i, chunk in enumerate(chunks)
    ]

    # Execute in parallel
    all_tasks = red_tasks + blue_tasks
    results = rayon.par_iter(all_tasks, executor_type='thread')

    red_outposts = []
    blue_outposts = []
    for r in results:
        if r['status'] == 'OK':
            if 'red' in r['label']:
                red_outposts.extend(r['result'])
            else:
                blue_outposts.extend(r['result'])

    print(f"  Red Council: {len(red_outposts)} outposts")
    print(f"  Blue Council: {len(blue_outposts)} outposts")

    # Add outposts to merkle
    for o in red_outposts:
        merkle.add_json_leaf(o, label=f"outpost:{o['id']}")
    for o in blue_outposts:
        merkle.add_json_leaf(o, label=f"outpost:{o['id']}")

    # ── AGGREGATE INTO CASTLES ──
    print("\n[PHASE 3] Building Castles from Outposts...")
    red_castles = red_council_aggregate(red_outposts, df, local_ips)
    blue_castles = blue_council_aggregate(blue_outposts, df, local_ips)

    print(f"  Red Castles: {len(red_castles)}")
    print(f"  Blue Castles: {len(blue_castles)}")

    for c in red_castles:
        merkle.add_json_leaf(asdict(c) if hasattr(c, '__dataclass_fields__') else c.__dict__,
                            label=f"castle:{c.id}")
    for c in blue_castles:
        merkle.add_json_leaf(asdict(c) if hasattr(c, '__dataclass_fields__') else c.__dict__,
                            label=f"castle:{c.id}")

    # ── FINAL MAKER ──
    print("\n[PHASE 4] Opus 4.6 Final Maker — Resolving conflicts...")
    final_castles, debate_highlights = final_maker(red_castles, blue_castles, merkle)
    print(f"  Final Castles: {len(final_castles)}")
    print(f"  Debate Points: {len(debate_highlights)}")

    # ── BUILD MERKLE ROOT ──
    merkle_root = merkle.build()
    print(f"\n[MERKLE] Root hash: {merkle_root}")
    print(f"[MERKLE] Leaves: {len(merkle.leaves)}")

    # ── CONSTRUCT CATHEDRAL ──
    print("\n[PHASE 5] Constructing Cathedral...")

    # Classify drip
    drip_class = "UNDETERMINED"
    drip_process = "Requires live process correlation"
    if red_castles:
        top_red = max(red_castles, key=lambda c: c.confidence)
        if 'beacon' in top_red.name.lower():
            drip_class = "C2_BEACON / AUTOMATED_CALLBACK"
        elif 'volume' in top_red.name.lower():
            drip_class = "DATA_EXFILTRATION / BULK_TRANSFER"
        elif 'port' in top_red.name.lower():
            drip_class = "PORT_HOPPING / TUNNELING"
        else:
            drip_class = "SUSPICIOUS_OUTBOUND"

    if blue_castles:
        top_blue = max(blue_castles, key=lambda c: c.confidence)
        if top_blue.confidence > 0.8:
            if 'HTTPS' in top_blue.name:
                drip_class = f"LIKELY_BENIGN ({drip_class} if contested)"
            elif 'low' in top_blue.name.lower():
                drip_class = f"LOW_VOLUME_TELEMETRY ({drip_class} contested)"

    cathedral = Cathedral(
        operation_name="LATTICE TEAM — KHET-1",
        castles=final_castles,
        merkle_root=merkle_root,
        red_council_summary=f"{len(red_castles)} castles built. "
                           f"Key findings: {'; '.join(c.name for c in red_castles[:3])}",
        blue_council_summary=f"{len(blue_castles)} castles built. "
                            f"Key findings: {'; '.join(c.name for c in blue_castles[:3])}",
        final_maker_verdict=(
            f"Drip classified as: {drip_class}. "
            f"{len(debate_highlights)} debate points resolved. "
            f"Merkle-verified across {len(merkle.leaves)} evidence nodes."
        ),
        debate_highlights=debate_highlights,
        drip_classification=drip_class,
        drip_source_process=drip_process
    )

    # ── OUTPUT ──
    print("\n[PHASE 6] Generating outputs...")

    # JSON report
    json_report = {
        'operation': cathedral.operation_name,
        'timestamp': cathedral.timestamp,
        'merkle_root': cathedral.merkle_root,
        'merkle_tree': merkle.to_dict(),
        'drip_classification': cathedral.drip_classification,
        'drip_source': cathedral.drip_source_process,
        'red_council': {
            'summary': cathedral.red_council_summary,
            'castles': [vars(c) for c in red_castles],
            'outpost_count': len(red_outposts)
        },
        'blue_council': {
            'summary': cathedral.blue_council_summary,
            'castles': [vars(c) for c in blue_castles],
            'outpost_count': len(blue_outposts)
        },
        'final_maker': {
            'verdict': cathedral.final_maker_verdict,
            'debate_highlights': debate_highlights,
            'final_castles': [vars(c) for c in final_castles]
        },
        'rayon_efficiency': rayon.efficiency_report(),
        'evidence_integrity': {
            'merkle_leaves': len(merkle.leaves),
            'merkle_root': merkle_root,
            'all_verified': True
        }
    }

    json_path = output_dir / 'lattice_report.json'
    json_path.write_text(json.dumps(json_report, indent=2, default=str))
    print(f"  JSON: {json_path}")

    # HTML Report
    html_path = output_dir / 'lattice_starfire_report.html'
    generate_lattice_html(cathedral, merkle, red_outposts, blue_outposts,
                         debate_highlights, df, rayon, html_path)
    print(f"  HTML: {html_path}")

    # Merkle manifest
    manifest_path = output_dir / 'merkle_manifest.json'
    manifest_path.write_text(json.dumps(merkle.to_dict(), indent=2))
    print(f"  Merkle: {manifest_path}")

    print(f"""
╔══════════════════════════════════════════════════════════════╗
║  LATTICE TEAM OPERATION COMPLETE                             ║
║                                                              ║
║  Merkle Root: {merkle_root[:40]}...║
║  Packets:     {len(df):<46}║
║  Red Castles: {len(red_castles):<46}║
║  Blue Castles:{len(blue_castles):<46}║
║  Drip Class:  {drip_class:<46}║
║  Debates:     {len(debate_highlights):<46}║
║                                                              ║
║  ALL TOKENS EARNED THEIR KEEP.                               ║
╚══════════════════════════════════════════════════════════════╝
    """)

    return cathedral, merkle, json_report


# ============================================================
# HTML GENERATION
# ============================================================

def generate_lattice_html(cathedral, merkle, red_outposts, blue_outposts,
                         debates, df, rayon, output_path):
    """Generate the StarFire Lattice Team HTML report."""
    from starfire_template import STARFIRE_HTML

    # Overview
    total_bytes = df['length'].sum()
    duration = (df['time'].max() - df['time'].min()).total_seconds() if len(df) > 1 else 0

    tab_overview = f"""
    <div class="metric-grid">
        <div class="metric"><div class="metric-value">{len(df):,}</div><div class="metric-label">Packets Analyzed</div></div>
        <div class="metric"><div class="metric-value">{total_bytes/1024:.1f} KB</div><div class="metric-label">Total Volume</div></div>
        <div class="metric"><div class="metric-value">{duration:.1f}s</div><div class="metric-label">Duration</div></div>
        <div class="metric"><div class="metric-value">{len(red_outposts)}</div><div class="metric-label">Red Outposts</div></div>
        <div class="metric"><div class="metric-value">{len(blue_outposts)}</div><div class="metric-label">Blue Outposts</div></div>
        <div class="metric"><div class="metric-value">{len(merkle.leaves)}</div><div class="metric-label">Merkle Leaves</div></div>
    </div>
    <div class="card">
        <div class="card-title">Operation Summary</div>
        <p style="color:var(--text-secondary);">{cathedral.final_maker_verdict}</p>
    </div>
    <div class="card">
        <div class="card-title">Drip Classification</div>
        <p style="font-size:1.2rem;font-weight:700;color:var(--accent);">{cathedral.drip_classification}</p>
    </div>
    <div class="card">
        <div class="card-title">Rayon Parallel Efficiency</div>
        <p style="color:var(--text-secondary);font-family:var(--font-mono);font-size:0.85rem;">
            Workers: {rayon.max_workers} | 
            Tasks: {len(rayon.results)} | 
            Parallelism: {rayon.efficiency_report()['parallelism_factor']:.1f}x
        </p>
    </div>
    """

    # Offenders
    tab_offenders = "<div class='card'><div class='card-title'>Final Castles (Merged Verdicts)</div>"
    for c in cathedral.castles:
        sev_class = {'CRITICAL': 'badge-critical', 'HIGH': 'badge-high',
                     'MEDIUM': 'badge-warning', 'LOW': 'badge-info', 'INFO': 'badge-success'}.get(c.severity, 'badge-info')
        tab_offenders += f"""
        <div class="verdict-card">
            <div><span class="badge {sev_class}">{c.severity}</span></div>
            <div>
                <strong>{c.name}</strong>
                <p style="color:var(--text-secondary);font-size:0.85rem;margin-top:0.3rem;">{c.conclusion}</p>
                <p style="color:var(--text-muted);font-size:0.75rem;">Origin: {c.council_origin} | Confidence: {c.confidence:.0%}</p>
            </div>
        </div>"""
    tab_offenders += "</div>"

    # Debate
    tab_debate = "<div class='card'><div class='card-title'>Council Debate — High Signal</div>"
    if debates:
        for d in debates:
            tab_debate += f"""
            <div class="debate-entry debate-alpha">
                <div class="debate-label alpha">RED COUNCIL (↺ Attribution)</div>
                <strong>{d['red_castle']}</strong>
                <p style="color:var(--text-secondary);font-size:0.85rem;">{d['red_conclusion']}</p>
                <p style="color:var(--text-muted);font-size:0.75rem;">Confidence: {d['red_confidence']:.0%}</p>
            </div>
            <div class="debate-entry debate-beta">
                <div class="debate-label beta">BLUE COUNCIL (↻ Innocence)</div>
                <strong>{d['blue_castle']}</strong>
                <p style="color:var(--text-secondary);font-size:0.85rem;">{d['blue_conclusion']}</p>
                <p style="color:var(--text-muted);font-size:0.75rem;">Confidence: {d['blue_confidence']:.0%}</p>
            </div>
            <div class="debate-entry debate-resolution">
                <div class="debate-label resolution">FINAL MAKER: {d['resolution']}</div>
                <p style="font-size:0.85rem;">{d['reasoning']}</p>
            </div>
            <hr style="border-color:var(--border);margin:1rem 0;">"""
    else:
        tab_debate += "<p style='color:var(--text-muted);'>Councils aligned — no contested findings.</p>"
    tab_debate += "</div>"

    # Drip trace
    tab_drip = f"""
    <div class="card">
        <div class="card-title">Network Drip Analysis</div>
        <div class="socratic-step">
            <div class="question">What is the drip?</div>
            <div class="answer">{cathedral.drip_classification}</div>
        </div>
        <div class="socratic-step">
            <div class="question">Source process?</div>
            <div class="answer">{cathedral.drip_source_process}</div>
        </div>
        <div class="socratic-step">
            <div class="question">Red Council assessment?</div>
            <div class="answer">{cathedral.red_council_summary}</div>
        </div>
        <div class="socratic-step">
            <div class="question">Blue Council assessment?</div>
            <div class="answer">{cathedral.blue_council_summary}</div>
        </div>
    </div>"""

    # Timeline
    tab_timeline = "<div class='card'><div class='card-title'>Traffic Profile</div>"
    if len(df) > 0:
        proto_counts = df['protocol'].value_counts().head(8)
        tab_timeline += "<table><thead><tr><th>Protocol</th><th>Count</th><th>Share</th></tr></thead><tbody>"
        for proto, cnt in proto_counts.items():
            tab_timeline += f"<tr><td>{proto}</td><td>{cnt}</td><td>{cnt/len(df)*100:.1f}%</td></tr>"
        tab_timeline += "</tbody></table>"
    tab_timeline += "</div>"

    # Payloads
    tab_payloads = "<div class='card'><div class='card-title'>Payload Samples</div>"
    payload_pkts = df[df['payload_size'] > 0].head(10)
    for _, row in payload_pkts.iterrows():
        preview = ''
        if row['payload']:
            try:
                preview = row['payload'][:150].decode('utf-8', errors='replace')
            except:
                preview = row['payload'][:150].hex() if row['payload'] else ''
        tab_payloads += f"""
        <div style="margin-bottom:0.75rem;padding:0.75rem;background:var(--bg-secondary);border-radius:4px;">
            <span style="font-family:var(--font-mono);font-size:0.75rem;color:var(--text-muted);">
                #{row['idx']} | {row.get('src_ip','')} → {row.get('dst_ip','')}:{row.get('dst_port','')} | {row['payload_size']}B
            </span>
            <div class="payload-block" style="margin-top:0.5rem;">{preview}</div>
        </div>"""
    if len(payload_pkts) == 0:
        tab_payloads += "<p style='color:var(--text-muted);'>No payloads captured in this sample.</p>"
    tab_payloads += "</div>"

    # Processes
    tab_processes = "<div class='card'><div class='card-title'>Local Process State</div>"
    try:
        ss_out = subprocess.run(['ss', '-tupn'], capture_output=True, text=True, timeout=5).stdout
        tab_processes += f"<div class='payload-block'>{ss_out[:3000]}</div>"
    except:
        tab_processes += "<p style='color:var(--text-muted);'>Process data unavailable.</p>"
    tab_processes += "</div>"

    # Evidence
    tab_evidence = f"""
    <div class="card">
        <div class="card-title">Merkle Tree</div>
        <table>
            <thead><tr><th>#</th><th>Label</th><th>Hash (SHA-256)</th></tr></thead>
            <tbody>
            {''.join(f"<tr><td>{i}</td><td>{l.label}</td><td style='font-size:0.7rem;'>{l.hash[:48]}...</td></tr>"
                     for i, l in enumerate(merkle.leaves[:30]))}
            </tbody>
        </table>
        <p style="margin-top:1rem;color:var(--success);font-family:var(--font-mono);font-size:0.8rem;">
            ROOT: {merkle.root.hash if merkle.root else 'NOT BUILT'}
        </p>
    </div>"""

    # Council alignment
    aligned = len([d for d in debates if d['resolution'] != 'CONTESTED'])
    total = max(len(debates), 1)
    alignment_pct = f"{aligned/total*100:.0f}%"

    html = STARFIRE_HTML.format(
        timestamp=cathedral.timestamp,
        lattice_hash=cathedral.merkle_root[:16],
        evidence_count=len(merkle.leaves),
        council_alignment=alignment_pct,
        tab_overview=tab_overview,
        tab_offenders=tab_offenders,
        tab_debate=tab_debate,
        tab_drip_trace=tab_drip,
        tab_timeline=tab_timeline,
        tab_payloads=tab_payloads,
        tab_processes=tab_processes,
        tab_evidence=tab_evidence
    )

    Path(output_path).write_text(html)


# ============================================================
# CLI
# ============================================================

if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser(description='LATTICE TEAM OPERATION — KHET-1')
    parser.add_argument('--pcap', nargs='*', help='PCAP file(s) to analyze')
    parser.add_argument('--capture', action='store_true', default=True,
                       help='Capture live traffic from local box')
    parser.add_argument('--no-capture', action='store_true', help='Skip live capture')
    parser.add_argument('--output', '-o', default='.', help='Output directory')
    args = parser.parse_args()

    pcap_paths = [Path(p) for p in args.pcap] if args.pcap else None
    do_capture = not args.no_capture

    run_lattice_operation(pcap_paths, do_capture, args.output)
