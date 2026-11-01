# Claude route: probe figures and review handles

Source comments of the Claude route cite figures from live probes and handles from its reviews.
The probe logs and the review folders sit outside this repository,
so this note carries what the comments rely on.
Written 2026-10-22 as the F8 entry of the fourth fix round of the Claude route review.
The rulings recorded here are the lead's and are open to the owner's veto.

## Live probes of 2026-10-22

The probes called the owner's Claude key;
this repository holds only what they measured.

### Probe 2, short prompts, 22:02 to 22:03 UTC

Output rates in tokens a second,
thinking counted,
two short-prompt samples for each model and no spread measured:

- Haiku 5.5:
  170.2 and 152.6.
- Sonnet 5.5:
  113.8 and 101.3.

`CLAUDE_FLOOR_TOKENS_PER_SECOND` (`shared-contract/src/claude-card-sides.ts`) is 80,
a fifth under the slowest measured call.
It is a floor to size a deadline by and not a prediction.

The longest pause between stream events (`maxGap` in `reportStreamProgress`) over the six calls was
40,
112,
1,990,
3,429,
3,278 and 3,934 ms,
and the longest call ran 38 s.
`provider/src/claude-body.ts` keeps the 120,000 ms idle window on that evidence,
and the Claude `maxGap` lines of the first corpus pass are to be read against 120 s.

### Probe 3, response formats, run by the lead

Twenty-six calls to claude-haiku-5-5,
estimated cost 0.0020 USD.
All 24 formats of the closed-vocabulary census answered HTTP 200 with stop reason `end_turn` and client outcome `ok`.
The probe's validator was a plain object check and not the stage guards.
The `pattern` control and the 25-label enum control answered HTTP 200.
HTTP 200 shows the schema was accepted and not that a keyword changes the reply.

Round 4 added four controls to the probe script for the keyed run that followed:
`minItems` 1,
`minItems` 2 sent raw,
a listed `format` and an unlisted `format` sent raw.

A first run with them,
on 2026-10-23,
made 30 calls for an estimated 0.00215 USD and sent no baseline.
A second run,
on 2026-10-23,
made 34 calls for an estimated 0.0023 USD,
and sent a baseline for each accepted control,
the same ask with the keyword removed.
Its result is in `doc/decision/translation-repair-direct-providers.md`,
in the addendum on the keyed Claude probes.
In this sample each keyword changed the reply (one reply for each arm),
and the probe cannot tell the compiled grammar from the model reading the schema text.

### Structured outputs page

<https://platform.claude.com/docs/en/build-with-claude/structured-outputs>,
read 2026-10-22.
The page lists `pattern` as supported without backreferences,
lookahead,
lookbehind and word boundaries,
`minItems` as supported for 0 and 1 only,
and ten string formats.
`provider/src/claude-schema.ts` quotes the page in its header.

## Handles the comments cite

The reviews of the Claude route numbered their findings.
A comment that cites one of these means the finding it names.

First review:

- C4:
  the run's editor list holds Sonnet 5.5.
- C6:
  the eight Claude cards share two wire models,
  so a bench must not seat two cards of one model.

Second review (R) and its closed-vocabulary item (P1):

- R1:
  a Claude call is armed with the larger of the run's deadline and one sized from its completion ceiling.
- R2:
  the 120 s idle window was checked against the six probe calls.
- R3:
  a Claude key that goes dry drops Haiku 5.5 from the next seating.
- R4:
  no per-call text in a system prompt,
  so the cached prefix is the same on every call.
- R5:
  the run bench comment names its load-time loop.
- R6:
  a stripped schema bound is told to the model in the field's description.
- R7:
  a header-stated wait past the timer range never arms a timer.
- R8:
  no digest or cache key includes a response format.
- R9:
  the `max_tokens` stub is the completion cap.
- R10:
  a schema keyword the feature refuses stops the body with `ClaudeSchemaKeywordError`.
- R11:
  the editor calibration chairs ask the one-wire-model rule where they are built.
- P1:
  closed-vocabulary fields list their labels as an enum built from the guard's own constant.

Third review:
R4 again,
for the fence length,
the rendered-break counts and the consistency marks moving to the user turn.

Re-check of the second fix round (F1 to F12),
answered in the fourth fix round:

- F1:
  no caller-side timeout signal on a Claude call in the two probe commands.
- F2:
  the call-config stamp,
  the START line and the entry-cap check name the Claude deadline.
- F3:
  the straggler grace usually ends a slow Claude voice long before 460 s (recorded,
  existing design).
- F4:
  `@throws` for `ClaudeSchemaKeywordError` on the callers that let it escape.
- F5:
  a request the route refused as malformed ends the round and is no lost voice.
- F6:
  cache generations;
  the schema changes ride inside the numbers round 3 set.
- F7:
  `pattern`,
  `minItems` and `format` follow the Structured outputs page.
- F8:
  this note,
  and no articles in `@param` and `@returns` lines.
- F9:
  a header-stated hold is capped at the run's hard cap.
- F10:
  the vocabulary tests feed the guard every enum member and one non-member.
- F11:
  the sheet-builder census matches an import or a call expression.
- F12:
  the reseat case folds holds through the production budgets.
