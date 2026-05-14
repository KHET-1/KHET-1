# Dual-Council Swarm — Generic Builder

Two adversarial teams. One builds. One breaks. Final Maker ships only what survives both.

## Usage

```python
from swarm import RedCouncil, BlueCouncil, FinalMaker

red = RedCouncil()   # Breaks things. Finds bugs. Challenges assumptions.
blue = BlueCouncil() # Builds things. Writes code. Defends decisions.
maker = FinalMaker() # Ships only what both agree on.

# Feed a task
result = maker.run(task="Build authentication module", red=red, blue=blue)
```

## How It Works

```
TASK IN
   │
   ├──→ BLUE COUNCIL (Build)
   │      Writes code. Makes architectural decisions.
   │      Produces: implementation + rationale
   │
   ├──→ RED COUNCIL (Break)
   │      Reviews Blue's output. Finds:
   │      - Bugs, edge cases, race conditions
   │      - Security holes
   │      - Missing error handling
   │      - Overcomplexity / YAGNI violations
   │      Produces: objections + severity scores
   │
   └──→ FINAL MAKER (Ship)
          Resolves: Blue fixes Red's valid objections.
          Rejects: Red objections that are nitpicks or wrong.
          Ships: Code that survived adversarial review.
```

## Principles

1. **Blue builds first.** Red doesn't build — Red only breaks.
2. **Red must cite evidence.** "This is bad" isn't an objection. "Line 47 has a null deref when X is empty" is.
3. **Final Maker is biased toward shipping.** Tie goes to Blue. Red must prove the defect.
4. **Every cycle produces output.** No infinite loops. Max 3 Red/Blue rounds per task.
5. **All tokens earn their keep.** No speculative code. No "might need later."
