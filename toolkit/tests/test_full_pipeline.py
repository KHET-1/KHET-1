"""End-to-end pipeline test using real evidence."""

import sys
import os
import json
import shutil
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent))

from evidence.sealer import EvidenceSealer
from parser.pcap_parser import PcapParser
from councils.red import RedCouncil
from councils.blue import BlueCouncil
from councils.final_maker import FinalMaker
from report.starfire import StarFireReport

EVIDENCE_DIR = Path(__file__).parent.parent.parent / 'evidence'
TEST_OUTPUT = Path('/tmp/test_pipeline_output')


def test_evidence_sealing():
    pcap = EVIDENCE_DIR / '1.pcapng'
    if not pcap.exists():
        print("[SKIP] 1.pcapng not found")
        return

    sealer = EvidenceSealer()
    item = sealer.ingest(str(pcap))
    assert item.sha256 == '035d86fcc6fcff58e59845df7af94ae0a4850e74a941bf95099b422aa0837508'
    assert item.size_bytes == 1294604

    root = sealer.seal()
    assert len(root) == 64
    assert sealer.verify(str(pcap)) is True
    print(f"[PASS] Evidence sealing: {item.filename} → {root[:16]}...")


def test_parser():
    pcap = EVIDENCE_DIR / '1.pcapng'
    if not pcap.exists():
        print("[SKIP] 1.pcapng not found")
        return

    parser = PcapParser()
    df = parser.parse(str(pcap))
    assert len(df) == 280
    assert 'src_ip' in df.columns
    assert 'dst_ip' in df.columns
    assert 'time' in df.columns
    assert df['src_ip'].value_counts().index[0] == '192.168.2.46'
    print(f"[PASS] Parser: {len(df)} packets, top src = {df['src_ip'].value_counts().index[0]}")


def test_councils():
    pcap = EVIDENCE_DIR / '1.pcapng'
    if not pcap.exists():
        print("[SKIP] 1.pcapng not found")
        return

    parser = PcapParser()
    df = parser.parse(str(pcap))
    local_ips = {'192.168.2.46'}

    red = RedCouncil()
    red_findings = red.analyze(df, local_ips)

    blue = BlueCouncil()
    blue_findings = blue.analyze(df, local_ips)

    assert len(blue_findings) > 0, "Blue should find something"
    assert all(hasattr(f, 'confidence') for f in blue_findings)
    assert all(hasattr(f, 'hypothesis') for f in red_findings)

    print(f"[PASS] Councils: Red={len(red_findings)}, Blue={len(blue_findings)}")


def test_final_maker():
    from councils.red import RedFinding
    from councils.blue import BlueFinding

    # Clear blue win: 0.95 > 0.5 * 1.3 (=0.65) → BLUE_PREVAILS
    red_findings = [
        RedFinding(id='R1', severity='HIGH', category='beaconing',
                  target='10.0.0.1', hypothesis='Regular 5s beaconing',
                  evidence='20 packets', confidence=0.5)
    ]
    blue_findings = [
        BlueFinding(id='B1', severity='INFO', category='infrastructure',
                   target='DNS/NTP', hypothesis='Infrastructure traffic normal',
                   evidence='Common pattern', explanation='Normal', confidence=0.95)
    ]

    maker = FinalMaker()
    verdicts, debate = maker.resolve(red_findings, blue_findings)

    assert len(debate) == 1
    assert debate[0]['resolution'] == 'BLUE_PREVAILS'
    assert maker.get_alignment() == 1.0
    print(f"[PASS] Final Maker: {debate[0]['resolution']} (alignment={maker.get_alignment():.0%})")

    # Contested case: 0.7 vs 0.8 — margin too small
    red2 = [RedFinding(id='R2', severity='MEDIUM', category='port_scan',
                       target='x', hypothesis='Port scan', evidence='8 ports', confidence=0.7)]
    blue2 = [BlueFinding(id='B2', severity='INFO', category='https_normal',
                        target='x', hypothesis='Normal HTTPS', evidence='70%',
                        explanation='Standard', confidence=0.8)]
    maker2 = FinalMaker()
    _, debate2 = maker2.resolve(red2, blue2)
    assert debate2[0]['resolution'] == 'CONTESTED'
    print(f"[PASS] Final Maker (contested): {debate2[0]['resolution']}")


def test_report_generation():
    if TEST_OUTPUT.exists():
        shutil.rmtree(TEST_OUTPUT)

    results = {
        'final_classification': 'TEST',
        'confidence': 99,
        'council_alignment': '100%',
        'verdict': 'TEST VERDICT',
        'total_packets': 100,
        'total_bytes': 50000,
        'key_findings': [{'title': 'Test', 'detail': 'This is a test'}],
        'castles': [],
        'debate': {'debate_log': []},
        'drip_trace': [],
        'payloads': [],
        'processes': {},
    }

    report = StarFireReport(output_dir=str(TEST_OUTPUT))
    report.generate(results)

    assert (TEST_OUTPUT / 'starfire_report.html').exists()
    assert (TEST_OUTPUT / 'starfire_report.json').exists()
    assert (TEST_OUTPUT / 'merkle_manifest.json').exists()

    html = (TEST_OUTPUT / 'starfire_report.html').read_text()
    assert 'TEST VERDICT' in html
    assert 'OFFLINE_ZERO_AI' in html

    manifest = json.loads((TEST_OUTPUT / 'merkle_manifest.json').read_text())
    assert manifest['leaf_count'] > 0
    assert len(manifest['merkle_root']) == 64

    print(f"[PASS] Report generation: HTML + JSON + Merkle manifest")


if __name__ == '__main__':
    test_evidence_sealing()
    test_parser()
    test_councils()
    test_final_maker()
    test_report_generation()
    print("\n[ALL PIPELINE TESTS PASSED]")
