"""Dual-Council Swarm. Blue builds. Red breaks. Final Maker ships."""

from dataclasses import dataclass, field
from typing import List, Dict, Optional
from datetime import datetime, timezone
import hashlib, json, re


@dataclass
class Objection:
    id: str
    severity: str      # CRITICAL, HIGH, MEDIUM, LOW
    category: str      # bug, security, complexity, error_handling, yagni
    location: str
    description: str
    evidence: str
    valid: Optional[bool] = None


@dataclass
class BuildResult:
    task: str
    files: Dict[str, str]
    objections_raised: int
    objections_fixed: int
    objections_rejected: int
    rounds: int
    seal: str
    timestamp: str


class BlueCouncil:
    """Builds code. Fixes valid objections."""

    def build(self, task: str, constraints=None, fixes=None) -> Dict[str, str]:
        """Override this. Return dict of filename→code."""
        return {}


class RedCouncil:
    """Breaks code. Single-pass static analysis. Must cite evidence."""

    PATTERNS = [
        (r'except\s*:', 'MEDIUM', 'error_handling', 'Bare except catches SystemExit/KeyboardInterrupt'),
        (r'eval\s*\(', 'HIGH', 'security', 'eval() — arbitrary code execution'),
        (r'exec\s*\(', 'HIGH', 'security', 'exec() — arbitrary code execution'),
        (r'execute\s*\(.*(?:f["\']|%s|%d|\.format)', 'CRITICAL', 'security', 'SQL injection — interpolation in execute()'),
    ]

    SECRET_KEYWORDS = ('password', 'secret', 'api_key', 'token', 'private_key')

    def __init__(self):
        self._n = 0

    def review(self, files: Dict[str, str]) -> List[Objection]:
        """Single pass over all files. Returns objections with evidence."""
        objs = []
        for fname, content in files.items():
            lines = content.split('\n')
            func_start, func_name, func_depth = 0, '', 0

            for i, line in enumerate(lines, 1):
                stripped = line.strip()

                # Pattern matching
                for pat, sev, cat, desc in self.PATTERNS:
                    if re.search(pat, line):
                        objs.append(self._obj(sev, cat, f'{fname}:{i}', desc, stripped))

                # Hardcoded secrets
                if any(k in line.lower() for k in self.SECRET_KEYWORDS):
                    if '=' in line and any(q in line for q in ('"', "'")) and 'env' not in line.lower() and 'getenv' not in line.lower():
                        objs.append(self._obj('CRITICAL', 'security', f'{fname}:{i}', 'Hardcoded secret', stripped[:60]))

                # Function length tracking
                if stripped.startswith(('def ', 'async def ')):
                    if func_name and (i - func_start) > 50:
                        objs.append(self._obj('MEDIUM', 'complexity', f'{fname}:{func_start}', f'{func_name}() is {i-func_start} lines', f'Lines {func_start}-{i}'))
                    func_start, func_name = i, stripped.split('(')[0].replace('def ','').replace('async ','')

            # Check last function
            if func_name and (len(lines) - func_start) > 50:
                objs.append(self._obj('MEDIUM', 'complexity', f'{fname}:{func_start}', f'{func_name}() is {len(lines)-func_start} lines', f'Lines {func_start}-{len(lines)}'))

        return objs

    def _obj(self, sev, cat, loc, desc, evidence) -> Objection:
        self._n += 1
        return Objection(id=f'R{self._n}', severity=sev, category=cat, location=loc, description=desc, evidence=evidence)


class FinalMaker:
    """Ships what survives. Max 3 rounds. Tie goes to Blue."""

    def run(self, task: str, blue: BlueCouncil, red: RedCouncil, constraints=None) -> BuildResult:
        fixes_needed = None
        all_objs = []
        accepted, rejected = 0, 0

        for rnd in range(1, 4):
            files = blue.build(task, constraints, fixes=fixes_needed)
            objs = red.review(files)
            all_objs.extend(objs)

            if not objs:
                break

            fixes_needed = []
            for o in objs:
                if o.severity in ('CRITICAL', 'HIGH') and o.evidence:
                    o.valid = True
                    fixes_needed.append(o)
                    accepted += 1
                elif o.severity == 'MEDIUM' and o.evidence:
                    o.valid = True
                    fixes_needed.append(o)
                    accepted += 1
                else:
                    o.valid = False
                    rejected += 1

            if not fixes_needed:
                break

        code_blob = json.dumps(files, sort_keys=True)
        return BuildResult(
            task=task,
            files=files,
            objections_raised=len(all_objs),
            objections_fixed=accepted,
            objections_rejected=rejected,
            rounds=rnd,
            seal=hashlib.sha256(code_blob.encode()).hexdigest(),
            timestamp=datetime.now(timezone.utc).isoformat()
        )
