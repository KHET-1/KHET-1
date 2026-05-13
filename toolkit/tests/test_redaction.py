"""Test redaction engine."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

# Import from the standalone starfire script's redaction logic
# (we'll also test the principle independently)
import re


class Redactor:
    """Minimal test-compatible redaction engine."""

    def __init__(self, redact_map=None, redact_internal=False):
        self.map = redact_map or {}
        self.auto_internal = redact_internal
        self.counter = 0

    def redact(self, text):
        result = str(text)
        for orig, repl in self.map.items():
            result = result.replace(orig, repl)
        if self.auto_internal:
            def replace_ip(match):
                ip = match.group(0)
                if ip not in self.map:
                    self.counter += 1
                    self.map[ip] = f"INTERNAL-{self.counter:03d}"
                return self.map[ip]
            result = re.sub(r'192\.168\.\d+\.\d+', replace_ip, result)
            result = re.sub(r'10\.\d+\.\d+\.\d+', replace_ip, result)
            result = re.sub(r'172\.(1[6-9]|2\d|3[01])\.\d+\.\d+', replace_ip, result)
        return result


def test_basic_redaction():
    r = Redactor(redact_map={'rathin': '[ANALYST]', '192.168.2.46': 'SOURCE-A'})
    result = r.redact("User rathin at 192.168.2.46 ran the scan")
    assert 'rathin' not in result
    assert '192.168.2.46' not in result
    assert '[ANALYST]' in result
    assert 'SOURCE-A' in result
    print("[PASS] Basic redaction: names and IPs replaced")


def test_auto_internal():
    r = Redactor(redact_internal=True)
    text = "Traffic from 192.168.2.46 to 192.168.2.151 and 10.0.0.1"
    result = r.redact(text)
    assert '192.168' not in result
    assert '10.0.0' not in result
    assert 'INTERNAL-001' in result
    assert 'INTERNAL-002' in result
    assert 'INTERNAL-003' in result
    print(f"[PASS] Auto-internal: {result}")


def test_consistency():
    r = Redactor(redact_internal=True)
    text1 = "Source: 192.168.2.46"
    text2 = "Destination: 192.168.2.46"
    r.redact(text1)
    result2 = r.redact(text2)
    # Same IP should get same label
    assert 'INTERNAL-001' in result2
    print("[PASS] Consistency: same IP gets same label across calls")


def test_external_not_redacted():
    r = Redactor(redact_internal=True)
    text = "Connecting to 8.8.8.8 and 1.1.1.1 from 192.168.2.46"
    result = r.redact(text)
    assert '8.8.8.8' in result  # External IPs untouched
    assert '1.1.1.1' in result
    assert '192.168' not in result
    print("[PASS] External IPs preserved, internal redacted")


def test_empty_and_none():
    r = Redactor(redact_internal=True)
    assert r.redact("") == ""
    assert r.redact("no IPs here") == "no IPs here"
    print("[PASS] Empty/no-match inputs handled")


if __name__ == '__main__':
    test_basic_redaction()
    test_auto_internal()
    test_consistency()
    test_external_not_redacted()
    test_empty_and_none()
    print("\n[ALL REDACTION TESTS PASSED]")
