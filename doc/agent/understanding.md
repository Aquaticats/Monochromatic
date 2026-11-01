# Understanding before design

This practice records what a task's concepts mean before those meanings become design constraints.
It applies to familiar language as well as unfamiliar APIs.
“Graceful fallback” and “parallel batch” caused incorrect designs when their meanings were assumed rather than established.

## Contents of a working understanding

A useful record contains the meaning in this task, a concrete example, a counterexample,
the evidence or authority, the scope of that evidence, and remaining uncertainty.
The record lives beside the relevant plan or implementation;
the handover points to it instead of becoming its only home.

Statements have distinguishable status:

- A user requirement records the user's actual instruction or accepted choice.
  A proposal by the agent is not a shared decision.
- A verified fact names the source or real consumer check, its date or version, and the exercised surface.
- An inference names the evidence and the reasoning still connecting it to the conclusion.
- A proposal states behavior that has not been accepted.
- An unknown states the missing evidence or decision, without turning it into a prohibition.

These distinctions remain visible in examples, diagrams, and questions too.
A confident diagram does not upgrade an inference into a fact.

## Research depth

For a tool capability, the useful evidence is its current public contract,
the deciding implementation path, and an actual consumer exercising the relevant distinction.
A successful all-allow test and an all-deny test do not establish how mixed outcomes behave.
Tests of both member orders can distinguish per-call blocking from accidental whole-batch behavior.
Unexercised paths remain named limits rather than implied guarantees.

A constraint found in one prototype belongs to that prototype until evidence establishes a platform constraint.
Public alternatives need investigation before a private integration or capability-gap claim is proposed.
An unavailable public capability is considered under the user's fallback requirements;
it does not grant permission to invent a new architecture.

For user-defined terms and preferences, the user's explanation is the governing source.
External definitions may clarify a missing distinction; they do not replace an already explicit meaning.
The accepted meaning of graceful fallback in this repository is minimal degradation,
aiming to preserve the end-user experience entirely where possible.
It is not a synonym for stopping, prompting, or disabling functionality whenever a guarantee is unavailable.

## Corrections and propagation

A corrected understanding includes the mistaken claim and the evidence that supersedes it.
Every dependent proposal, example, diagram, test expectation, and handover entry needs reassessment.
For example, learning that a parallel batch permits selective blocking changes the reducer scope,
execution diagrams, manual-decision scope, and mixed-outcome fixtures, not just one paragraph.

The correction does not automatically decide adjacent questions.
Per-call blocking does not establish arbitrary cancellation after a call has started,
identify which model answer belongs to which call,
or choose when an allowed member should start while another remains under review.
Those distinctions stay explicit until evidence or a user decision resolves them.

## Relationship to source and domain documents

The record preserves meanings, decisions, evidence, and uncertainties; it is not a cached replacement for source.
Code and tool contracts still need fresh inspection for each probe.
This practice does not create `CONTEXT.md` or a separate vocabulary authority that can overrule the current code or user.
See [domain documentation](domain.md) and [independent correction review](self-review.md).
