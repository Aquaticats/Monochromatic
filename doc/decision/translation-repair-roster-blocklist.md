# Roster refresh: calibrate then seat, under an owner blocklist

## Status

Decided by the owner on 2026-09-01,
answering the roster decision point in
[`translation-repair-no-loop-design.md`](../planning/translation-repair-no-loop-design.md):
option R3,
"Calibrate then seat",
with a blocklist stated in the same answer.

The shape decision point was answered separately and is NOT a decision:
"There's no point of asking me to set something in stone.
You're free to prototype and measure and do anything.
I am neither explicitly approving it nor explicitly disapproving it
because I know we'd get stuck in a box either way."
Design authority over the pipeline shape is therefore delegated:
prototype,
measure,
and adjust without treating the written shape as ratified.

## The blocklist, verbatim with the owner's reasons

- `hf:nvidia/NVIDIA-Nemotron-3-Super-120B-A12B-NVFP4`:
  "can't stick to its own viewpoint".
- `hf:zai-org/GLM-4.7-Flash`,
  `hf:zai-org/GLM-5.2`,
  GLM5,
  GLM5.1,
  Kimi K2.5,
  Kimi K2.6,
  Kimi K2.7 Code,
  Llama,
  MiniMax M2.7,
  Qwen3.6,
  Qwen3.7,
  Qwen3:
  "too outdated".
- `deepseek-v4-flash`,
  `deepseek-v4-pro`:
  "the undated versions",
  so dated aliases such as `deepseek-v4-pro-0813` stayed eligible
  until 2026-09-16.
- `deepseek-v4-pro-0813`,
  `deepseek-v4-flash-0731`
  and their OpenRouter spellings,
  added 2026-09-16:
  "DeepSeek V4.1 Flash is much better than both V4 Pro and V4 Flash,
  so please remove V4 Pro and V4 Flash."
  Record:
  the 2026-09-16 addendum of
  `doc/decision/translation-repair-roster-seating-2026-09-01.md`.
- Qwen 3.8 Max
  (`qwen3.8-max`):
  "absurd cost in money".

Reading notes,
mine:
bare "Qwen3" names the Qwen3 series,
not Qwen3.8,
because the owner blocklists Qwen 3.8 Max separately on cost,
which presupposes other Qwen3.8 variants remain eligible.
Names such as `qwen3-coder-480b-a35b-instruct-int4-mixed-ar` and
`qwen3-next-80b-a3b-instruct` are Qwen3-series spellings and are blocked.
Bare "GLM5" names the model Hyper serves as `glm-5`,
not the GLM5 series,
by the same presupposition:
the owner lists GLM5.1 and GLM-5.2 separately,
which a series reading would make redundant,
and GLM-5.3-Flash was a seated production model when the owner wrote the list,
so a series reading would have the owner silently blocking their own
seated model under a reason ("too outdated") that cannot describe the
newest member.
`glm-5.3` and `glm-5.3-flash` therefore stay eligible.
"Llama" blocks the whole family.
`qwen3.8-max` was already culled from the whole roster on 2026-08-28
at the owner's instruction
(recorded at `HYPER_ORIGIN_ROSTER_IDS` in
`package/translation-repair/shared-contract/src/roster-id.ts`),
so this blocklist entry confirms an existing cull rather than unseating
a currently seated model;
an earlier revision of this document said "currently seated",
which was stale when written.
Hyper's live catalog on 2026-09-01 serves `qwen3.8-2.4t-a95b` at exactly
the credit rate `qwen3.8-max` carried at its cull
(input 40,
output 120 credits per million tokens):
the owner's entry names the Max model,
not a price rule,
so the 2.4t model stayed an eligible candidate.
The same-day conformance probe then culled it and `qwen3.8-flash` before
seating on a separate,
non-blocklist ground:
both answer plain text and tools under automatic choice with HTTP 200
but reject `tool_choice: {type: 'tool'}` with HTTP 400,
the automatic-only constraint recorded for the culled `qwen3.8-max`,
and every structured stage forces its tool.
Non-conformance is a catalog fact,
never an owner instruction,
so neither joins the blocklist;
the probe is recorded on the `HYPER_MODELS` entry comment in
`package/translation-repair/shared-contract/src/hyper-catalog.ts`.

## Advisory attached to the same answer

"DeepSeek models sometimes treats prompts extremely literally."
Carry this into sheet and prompt design wherever a DeepSeek route is a candidate:
avoid figurative instructions,
spell out edge handling,
and treat over-literal readings as a known failure shape rather than model noise.

Review pass performed 2026-09-01 over the calibration-facing sheets
(`translate-wire.ts` system rules and user framing,
`translate-selection-sheet.ts` criteria,
`refine-prompt.ts` system and correction policy,
`house-policy.ts` shared blocks):
no edits required.
Every precedence conflict is stated literally
("THIS RULE OUTRANKS ...",
"WHERE A CRITERION AND A HOUSE RULE DISAGREE,
THE HOUSE RULE WINS"),
absent-input and empty-answer edges are named in place
("(none:
this passage has no translation yet)",
"Returning an empty list is a correct and common answer"),
and the house policy instructs against character-literal readings outright
("rendered by its conventional community meaning is correct even when a
literal reading of the characters says otherwise").
The residual figurative phrases
("should not see it churn",
"carries warmth")
each sit beside a literal operative sentence that carries the action,
so the literal and intended readings coincide.

## Consequences

- The calibration candidate set is the live Hyper and Synthetic catalogs
  minus this blocklist,
  mapped to exact catalog ids in a named constant with the reason per entry.
- Seating follows the calibration measurements,
  per the R3 order recorded in the no-loop design:
  code lands verified,
  calibration seats the roster,
  four-entry pass,
  reading.
