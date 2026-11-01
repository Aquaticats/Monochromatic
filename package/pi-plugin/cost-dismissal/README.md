# pi-plugin-cost-dismissal

Shadow-records cost dismissals found in pi sessions.

## What it does

Folds every finished assistant message
(thinking and visible text alike,
since the detector measures overall dismissals)
for sentences that carry algorithmic cost together with a dismissal,
and records them at `tool_call`,
`tool_result`,
and `turn_end`:
the intended action,
the completed action,
and the turn that ends with no action.

Everything runs on this machine.
The record carries a block hash,
a sentence index,
a pattern id,
and on the tool events the tool name and call id;
never reasoning text.

## Where records land

The shared log of the whole detector family:
`~/.local/state/monochromatic/cost-dismissal/shadow.jsonl`,
overridable with `MONOCHROMATIC_COST_DISMISSAL_LOG`
and written beside the Claude Code `cccd` hook
and the DeepSeek Harness `cost-dismissal` plugin.

## Installation

Add this package to global pi settings at `~/.pi/agent/settings.json`:

```json
{
  "packages": [
    "/var/home/user/Monochromatic/package/pi-plugin/cost-dismissal"
  ]
}
```
