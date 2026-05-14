"""
Dual-Council Swarm — Generic Code Builder
Two teams. One builds. One breaks. Final Maker ships what survives.
"""

from dataclasses import dataclass, field
from typing import List, Dict, Optional, Callable
from datetime import datetime, timezone
import hashlib
import json


@dataclass
class BuildOutput:
    """What Blue Council produces."""
    task: str
    code: str
    language: str
    rationale: str
    files: Dict[str, str] = field(default_factory=dict)  # filename → content
    dependencies: List[str] = field(default_factory=list)
    confidence: float = 0.8


@dataclass
class Objection:
    """What Red Council raises against Blue's output."""
    id: str
    severity: str  # CRITICAL, HIGH, MEDIUM, LOW, NITPICK
    category: str  # bug, security, performance, complexity, missing_error_handling, yagni
    location: str  # file:line or description of where
    description: str
    evidence: str  # specific proof (not opinion)
    suggested_fix: str = ''
    valid: Optional[bool] = None  # Final Maker sets this


@dataclass
class ShipDecision:
    """Final Maker's verdict on each objection."""
    objection_id: str
    accepted: bool
    reasoning: str


@dataclass
class BuildResult:
    """Final shipped output."""
    task: str
    code: str
    files: Dict[str, str]
    objections_raised: int
    objections_accepted: int
    objections_rejected: int
    rounds: int
    merkle_hash: str
    timestamp: str


class BlueCouncil:
    """
    Builds code. Defends decisions. Fixes valid objections.
    Bias: toward shipping working code. Minimal. Correct.
    """

    def __init__(self):
        self.builds: List[BuildOutput] = []

    def build(self, task: str, constraints: List[str] = None, 
              fix_objections: List[Objection] = None) -> BuildOutput:
        """
        Generate code for a task. If fix_objections provided, 
        revise previous build to address them.
        
        Override this method with actual code generation logic
        (LLM call, template engine, or manual implementation).
        """
        output = BuildOutput(
            task=task,
            code="",
            language="python",
            rationale="",
            files={},
            dependencies=[],
            confidence=0.8
        )

        if fix_objections:
            output.rationale = f"Revised to address {len(fix_objections)} objection(s): " + \
                             ", ".join(o.id for o in fix_objections)

        self.builds.append(output)
        return output

    def defend(self, objection: Objection, build: BuildOutput) -> str:
        """
        Blue's response to a Red objection. Returns defense or concession.
        Override for actual logic.
        """
        return f"Acknowledged: {objection.id}. Will fix in next iteration."


class RedCouncil:
    """
    Breaks code. Finds defects. Challenges assumptions.
    Bias: toward finding real problems. Must cite evidence.
    Rules: No building. No "I would do it differently." Only defects with proof.
    """

    def __init__(self):
        self.objections: List[Objection] = []
        self._counter = 0

    def review(self, build: BuildOutput) -> List[Objection]:
        """
        Review Blue's output. Return list of objections.
        Each objection MUST have evidence (not opinion).
        
        Override this method with actual review logic.
        """
        objections = []

        # Basic automated checks (these always run)
        objections.extend(self._check_error_handling(build))
        objections.extend(self._check_complexity(build))
        objections.extend(self._check_security(build))

        self.objections.extend(objections)
        return objections

    def _check_error_handling(self, build: BuildOutput) -> List[Objection]:
        """Check for missing error handling."""
        issues = []
        for filename, content in build.files.items():
            lines = content.split('\n')
            for i, line in enumerate(lines, 1):
                # Bare except
                if 'except:' in line and 'except Exception' not in line:
                    self._counter += 1
                    issues.append(Objection(
                        id=f"RED-{self._counter}",
                        severity="MEDIUM",
                        category="missing_error_handling",
                        location=f"{filename}:{i}",
                        description="Bare except clause catches SystemExit, KeyboardInterrupt",
                        evidence=f"Line {i}: {line.strip()}"
                    ))
                # No error handling on file operations
                if 'open(' in line and 'try' not in '\n'.join(lines[max(0,i-3):i]):
                    self._counter += 1
                    issues.append(Objection(
                        id=f"RED-{self._counter}",
                        severity="LOW",
                        category="missing_error_handling",
                        location=f"{filename}:{i}",
                        description="File operation without try/except (FileNotFoundError risk)",
                        evidence=f"Line {i}: {line.strip()}"
                    ))
        return issues

    def _check_complexity(self, build: BuildOutput) -> List[Objection]:
        """Check for overcomplexity."""
        issues = []
        for filename, content in build.files.items():
            lines = content.split('\n')
            # Function too long
            in_func = False
            func_start = 0
            func_name = ""
            for i, line in enumerate(lines, 1):
                if line.strip().startswith('def ') or line.strip().startswith('async def '):
                    if in_func and (i - func_start) > 50:
                        self._counter += 1
                        issues.append(Objection(
                            id=f"RED-{self._counter}",
                            severity="MEDIUM",
                            category="complexity",
                            location=f"{filename}:{func_start}",
                            description=f"Function '{func_name}' is {i - func_start} lines (>50). Split it.",
                            evidence=f"Lines {func_start}-{i}"
                        ))
                    in_func = True
                    func_start = i
                    func_name = line.strip().split('(')[0].replace('def ', '').replace('async ', '')

            # Too many imports (over-engineering signal)
            import_count = sum(1 for l in lines if l.startswith('import ') or l.startswith('from '))
            if import_count > 15:
                self._counter += 1
                issues.append(Objection(
                    id=f"RED-{self._counter}",
                    severity="LOW",
                    category="yagni",
                    location=f"{filename}:1-{import_count}",
                    description=f"{import_count} imports. Likely over-engineered.",
                    evidence=f"{import_count} import statements"
                ))
        return issues

    def _check_security(self, build: BuildOutput) -> List[Objection]:
        """Check for security issues."""
        issues = []
        for filename, content in build.files.items():
            lines = content.split('\n')
            for i, line in enumerate(lines, 1):
                # SQL injection risk
                if 'execute(' in line and ('f"' in line or "f'" in line or '%s' in line):
                    self._counter += 1
                    issues.append(Objection(
                        id=f"RED-{self._counter}",
                        severity="CRITICAL",
                        category="security",
                        location=f"{filename}:{i}",
                        description="Possible SQL injection — string interpolation in execute()",
                        evidence=f"Line {i}: {line.strip()}"
                    ))
                # Hardcoded secrets
                if any(kw in line.lower() for kw in ['password', 'secret', 'api_key', 'token']) and '=' in line and '#' not in line:
                    if any(c in line for c in ['"', "'"]) and 'env' not in line.lower() and 'getenv' not in line.lower():
                        self._counter += 1
                        issues.append(Objection(
                            id=f"RED-{self._counter}",
                            severity="CRITICAL",
                            category="security",
                            location=f"{filename}:{i}",
                            description="Possible hardcoded secret",
                            evidence=f"Line {i}: {line.strip()[:60]}..."
                        ))
                # eval/exec
                if 'eval(' in line or 'exec(' in line:
                    self._counter += 1
                    issues.append(Objection(
                        id=f"RED-{self._counter}",
                        severity="HIGH",
                        category="security",
                        location=f"{filename}:{i}",
                        description="eval()/exec() — arbitrary code execution risk",
                        evidence=f"Line {i}: {line.strip()}"
                    ))
        return issues


class FinalMaker:
    """
    Resolves Red vs Blue. Ships what survives.
    Bias: toward shipping. Red must PROVE the defect. Tie goes to Blue.
    Max rounds: 3. After that, ship with known issues documented.
    """

    MAX_ROUNDS = 3

    def __init__(self):
        self.decisions: List[ShipDecision] = []
        self.rounds_completed = 0

    def run(self, task: str, blue: BlueCouncil, red: RedCouncil,
            constraints: List[str] = None) -> BuildResult:
        """
        Full build cycle: Blue builds, Red reviews, iterate, ship.
        """
        accepted_objections = []
        rejected_objections = []
        all_objections = []

        # Round 1: Blue builds
        build = blue.build(task, constraints)

        for round_num in range(1, self.MAX_ROUNDS + 1):
            # Red reviews
            objections = red.review(build)
            all_objections.extend(objections)

            if not objections:
                break  # Clean build, ship it

            # Final Maker decides each objection
            valid_objections = []
            for obj in objections:
                decision = self._decide(obj, build)
                self.decisions.append(decision)

                if decision.accepted:
                    obj.valid = True
                    valid_objections.append(obj)
                    accepted_objections.append(obj)
                else:
                    obj.valid = False
                    rejected_objections.append(obj)

            if not valid_objections:
                break  # All objections rejected, ship as-is

            # Blue fixes valid objections
            build = blue.build(task, constraints, fix_objections=valid_objections)

        self.rounds_completed = round_num

        # Seal the output
        all_code = json.dumps(build.files, sort_keys=True) + build.code
        merkle_hash = hashlib.sha256(all_code.encode()).hexdigest()

        return BuildResult(
            task=task,
            code=build.code,
            files=build.files,
            objections_raised=len(all_objections),
            objections_accepted=len(accepted_objections),
            objections_rejected=len(rejected_objections),
            rounds=self.rounds_completed,
            merkle_hash=merkle_hash,
            timestamp=datetime.now(timezone.utc).isoformat()
        )

    def _decide(self, objection: Objection, build: BuildOutput) -> ShipDecision:
        """
        Decide if an objection is valid.
        CRITICAL/HIGH with evidence → accept.
        NITPICK or no evidence → reject.
        """
        if objection.severity == 'NITPICK':
            return ShipDecision(
                objection_id=objection.id,
                accepted=False,
                reasoning="Nitpick. Tie goes to Blue. Ship it."
            )

        if not objection.evidence:
            return ShipDecision(
                objection_id=objection.id,
                accepted=False,
                reasoning="No evidence provided. Red must cite specific proof."
            )

        if objection.severity in ('CRITICAL', 'HIGH'):
            return ShipDecision(
                objection_id=objection.id,
                accepted=True,
                reasoning=f"{objection.severity} with evidence. Must fix before ship."
            )

        if objection.severity == 'MEDIUM':
            return ShipDecision(
                objection_id=objection.id,
                accepted=True,
                reasoning="Medium severity with evidence. Fix to reduce risk."
            )

        # LOW — accept only if trivial to fix
        return ShipDecision(
            objection_id=objection.id,
            accepted=False,
            reasoning="Low severity. Document and ship. Fix in next iteration."
        )

    def get_report(self) -> Dict:
        """Summary of the build cycle."""
        return {
            'rounds': self.rounds_completed,
            'decisions': len(self.decisions),
            'accepted': sum(1 for d in self.decisions if d.accepted),
            'rejected': sum(1 for d in self.decisions if not d.accepted),
            'details': [
                {'id': d.objection_id, 'accepted': d.accepted, 'reasoning': d.reasoning}
                for d in self.decisions
            ]
        }
