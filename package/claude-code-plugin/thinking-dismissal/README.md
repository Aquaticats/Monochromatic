# thinking-dismissal

Claude Code `Stop` hook that shadow-records cost dismissals found in the
session's thinking blocks.

## What it does

Reads the tail of the session transcript at every stop,
looks at each thinking block's sentences,
and flags the ones that carry algorithmic cost and a dismissal together
(for example a quadratic scan waved away because the input is short).

Everything runs on this machine.
The record carries a block hash,
a sentence index,
and a pattern id;
never reasoning text,
because the log must stay safe to keep and read back.

The hook only records.
It never blocks a stop,
never injects context,
and never contacts a provider.

## Where records land

One JSONL line per stop at
`~/.local/state/monochromatic/thinking-dismissal/shadow.jsonl`,
overridable with `MONOCHROMATIC_THINKING_DISMISSAL_LOG` for tests.
Hits a previous stop already recorded are dropped,
so overlapping tail rescans do not repeat themselves.

## Pattern

`size-dismissal`:
one sentence carrying a cost term and a dismissal term together.
The vocabularies are calibrated against the two known dismissal blocks of
session `73f0639e` and an 11-transcript negative sample
(2 of 2 positives caught,
128 of 48,252 negative blocks flagged),
recorded in `doc/handover/superlinear-cost-dismissal-gate.md`
under `Local calibration, 2026-10-29`.
