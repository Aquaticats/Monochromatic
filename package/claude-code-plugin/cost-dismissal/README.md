# cost-dismissal

Claude Code hook that shadow-records cost dismissals found in the session's
thinking blocks,
at `PreToolUse`,
`PostToolUse`,
and `Stop` from one handler.

## Why three hook events

Each event sees a part of the turn the others miss,
so one dismissal can be observed from three angles:

- `PreToolUse` names the action a dismissal led to before it runs,
  and it is where a graduated gate could demand a proof first.
- `PostToolUse` proves the action actually ran,
  since a denial at `PreToolUse` never produces one.
- `Stop` is the only event that sees turn-ending thinking
  with no tool call after it.

Each observation is tagged with its event
and kept once per event type,
so the log holds the intent view,
the completion view,
and the residual view of the same hit,
joinable on the block hash.

## What it does

Reads the tail of the session transcript at each event,
looks at each thinking block's sentences,
and flags the ones that carry algorithmic cost and a dismissal together
(for example a quadratic scan waved away because the input is short).

Everything runs on this machine.
The record carries a block hash,
a sentence index,
a pattern id,
and on the tool events the tool name and call id;
never reasoning text,
because the log must stay safe to keep and read back.

The hook only records.
It never denies a tool call,
never injects context,
and never contacts a provider.

## Where records land

One JSONL line per event at
`~/.local/state/monochromatic/cost-dismissal/shadow.jsonl`,
overridable with `MONOCHROMATIC_COST_DISMISSAL_LOG` for tests.

## Pattern

`size-dismissal`:
one sentence carrying a cost term and a dismissal term together.
The vocabularies are calibrated against the two known dismissal blocks of
session `73f0639e` and an 11-transcript negative sample
(2 of 2 positives caught,
128 of 48,252 negative blocks flagged),
recorded in `doc/handover/superlinear-cost-dismissal-gate.md`
under `Local calibration, 2026-10-29`.
