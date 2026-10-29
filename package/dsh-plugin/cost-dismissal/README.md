# cost-dismissal (DeepSeek Harness)

DeepSeek Harness plugin that shadow-records cost dismissals found in session
thinking blocks,
at `tools/pre-execute`,
`tools/result`,
and `agent/turn-stopping`.
The Claude Code twin is the `cccd` hook of
`package/claude-code-plugin/cost-dismissal`;
both write one shared log.

## Why these events

Each event sees a part of the turn the others miss:

- `tools/pre-execute` names the action a dismissal led to before it runs.
- `tools/result` proves the action ran,
  since a denial never produces one.
- `agent/turn-stopping` is the only point that sees turn-ending thinking
  with no tool call after it.

## How it reads thinking

A `ctx.sessionProjections` unit folds every committed `assistant/message`
event,
one pass per event over its `reasoning` blocks.
Detection flags the sentences that carry algorithmic cost and a dismissal
together;
the record carries a block hash,
a sentence index,
a pattern id,
and on the tool events the tool name and call id.
Never reasoning text,
so the log is safe to keep.

The hook only records.
It never denies a tool call,
never holds a turn,
and never contacts a provider.

## Where records land

One JSONL line per event at
`~/.local/state/monochromatic/cost-dismissal/shadow.jsonl`,
overridable with `MONOCHROMATIC_COST_DISMISSAL_LOG` for tests.

## Pattern

`size-dismissal`,
calibrated against the two known dismissal blocks of session `73f0639e` and
an 11-transcript negative sample
(2 of 2 positives caught,
128 of 48,252 negative blocks flagged);
see `doc/handover/superlinear-cost-dismissal-gate.md`
under `Local calibration, 2026-10-29`.
