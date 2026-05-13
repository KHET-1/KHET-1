"""Final Maker — Opus-grade conflict resolution between Red and Blue councils."""

from typing import List, Dict, Tuple
from dataclasses import dataclass

from .red import RedFinding
from .blue import BlueFinding


@dataclass
class Verdict:
    target: str
    red_finding: RedFinding
    blue_finding: BlueFinding
    resolution: str  # RED_PREVAILS, BLUE_PREVAILS, CONTESTED
    reasoning: str
    winning_confidence: float


class FinalMaker:
    """
    Resolves conflicts between Red and Blue councils.
    Every Red finding MUST face the strongest Blue counterargument.
    No finding passes unchallenged.
    """

    def __init__(self):
        self.verdicts: List[Verdict] = []
        self.debate_log: List[Dict] = []

    def resolve(self, red_findings: List[RedFinding], blue_findings: List[BlueFinding]) -> Tuple[List[Verdict], List[Dict]]:
        """Pit each red finding against the most relevant blue finding."""
        self.verdicts = []
        self.debate_log = []

        for rf in red_findings:
            best_blue = self._find_best_counter(rf, blue_findings)

            if best_blue:
                if rf.confidence > best_blue.confidence * 1.3:
                    resolution = "RED_PREVAILS"
                    reasoning = (f"Red confidence {rf.confidence:.2f} exceeds Blue {best_blue.confidence:.2f}. "
                                f"Suspicious pattern confirmed with stronger evidence.")
                elif best_blue.confidence > rf.confidence * 1.3:
                    resolution = "BLUE_PREVAILS"
                    reasoning = (f"Blue confidence {best_blue.confidence:.2f} exceeds Red {rf.confidence:.2f}. "
                                f"Benign explanation adequately accounts for observed behavior.")
                else:
                    resolution = "CONTESTED"
                    reasoning = (f"Margin insufficient (Red {rf.confidence:.2f} vs Blue {best_blue.confidence:.2f}). "
                                f"Both hypotheses plausible. Manual analyst review required.")

                verdict = Verdict(
                    target=rf.target,
                    red_finding=rf,
                    blue_finding=best_blue,
                    resolution=resolution,
                    reasoning=reasoning,
                    winning_confidence=max(rf.confidence, best_blue.confidence)
                )
                self.verdicts.append(verdict)

                self.debate_log.append({
                    'target': rf.target,
                    'red_id': rf.id,
                    'red_hypothesis': rf.hypothesis,
                    'red_confidence': rf.confidence,
                    'blue_id': best_blue.id,
                    'blue_hypothesis': best_blue.hypothesis,
                    'blue_confidence': best_blue.confidence,
                    'resolution': resolution,
                    'reasoning': reasoning
                })
            else:
                self.verdicts.append(Verdict(
                    target=rf.target,
                    red_finding=rf,
                    blue_finding=None,
                    resolution="UNCHALLENGED_RED",
                    reasoning="No blue counterargument available.",
                    winning_confidence=rf.confidence
                ))

        return self.verdicts, self.debate_log

    def _find_best_counter(self, red: RedFinding, blues: List[BlueFinding]):
        """Find the most relevant Blue finding to counter a Red finding."""
        if not blues:
            return None

        red_words = set(red.hypothesis.lower().split())
        network_terms = {'port', 'ports', 'traffic', 'https', 'http', 'tcp',
                        'bandwidth', 'outbound', 'connection', 'packets', 'dns'}

        best = None
        best_score = -1

        for bf in blues:
            blue_words = set(bf.hypothesis.lower().split())
            overlap = len(red_words & blue_words)
            domain = len((red_words | blue_words) & network_terms)
            score = overlap + domain * 0.5 + bf.confidence * 2

            if score > best_score:
                best_score = score
                best = bf

        return best

    def get_alignment(self) -> float:
        """Percentage of debates resolved (not CONTESTED)."""
        if not self.debate_log:
            return 1.0
        resolved = sum(1 for d in self.debate_log if d['resolution'] != 'CONTESTED')
        return resolved / len(self.debate_log)
