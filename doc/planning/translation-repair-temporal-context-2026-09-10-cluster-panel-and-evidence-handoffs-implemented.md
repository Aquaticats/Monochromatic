# Temporal and participant source context: the cluster panel and evidence handoffs, implemented

Tasks 22 and 23 as implemented:
cluster review packets,
evidence roles and the source evidence handoff to selectors,
with the compiled full-path check that remained insufficient.

Part of [Temporal and participant source context](translation-repair-temporal-context-2026-09-10.md),
which lists every file of this record.

## Implemented packet and evidence boundaries

Task 22 implements the measured cluster review unit in `panel-stage.ts`,
re-exported from `repair-stages.ts`.
All packets and local claim/group maps are materialized before the first panel call.
`mapOverlapped` with overlap one bounds packet execution;
no vote creates another packet or generation round.
Every packet uses the unchanged electorate and independent resolver/tally.
The union of heard identities is reporting only,
never a quorum basis.

`repair-evidence-role.ts` supplies the measured current-coverage versus factual-evidence distinction
in critic and panel context blocks.
The panel accepts optional `documentSourceText`;
local critic quote anchoring remains unchanged.
The three red packet guards fail on the old implementation in `06d120f74`.
Implementation `53a249bb1` and subsequent fixture/formatting commits are locally verified through `5c10ed0eb`.
The full suite ends `unit exit 0` in `~/temp/agent/cluster-panel-final-unit-20260910.out`.
The built public stage reproduces thirty cached stock-window requests with zero misses
and exactly twenty-one measured issue decisions:
`~/temp/agent/cluster-panel-runtime-replay-20260910.out`.
Ordinary no-context critic/panel prompts remain byte-identical in
`~/temp/agent/panel-prompt-parity-20260910.out`.

Task 23 threads same-entry source evidence from `repair-slice-settle.ts`
through the cache key,
twin key,
purchase,
repair chunk,
panel and existing repair selectors.
An unchanged full source equal to the current slice is not redundantly supplied by the corpus caller.
A changed document outside the local window changes the repair key.
It does not change initial translation-writer prompts.

`repair-selection-evidence.ts` supplies the measured nearby/document evidence to both selector granularities.
Whole-chunk selection also receives the existing English before repair from its known untouched fallback.
This is comparison context,
not an extra candidate or independent factual authority.
`editor-envelope-context.ts` extracts the existing bounded English-context helper to preserve the module line budget.

The three red evidence/key guards in `6b8921898` fail before implementation `05c736e4e`.
Additional guards exercise the actual prepared-document and editor-stage handoffs,
not only helper strings.
Build,
types,
zero-warning lint and the complete suite pass through `8adb77fb9`.
`~/temp/agent/evidence-handoff-unit-20260910.out` ends `unit exit 0` at line 9125;
post-format focused tests also pass.
`~/temp/agent/repair-selection-parity-20260910.out`
proves the compiled regional and whole-chunk prompts match the measured treatments.
No-context regional prompts remain unchanged.

One type-check was started before changed built declarations were ready and emitted TS2353.
The after-build type-check passes;
future public-API verification must rebuild before checking tests that import built declarations.
Proposed instruction clarification:
`CM6` should mention this ordering when public APIs change.
No `AGENTS.md` amendment was made.

The implementation-review advisor request timed out;
no endorsement is attributed to that call.
Verification rests on the named tests,
parity checks and public-stage replay.

A read-only census of the unchanged pinned corpus finds ninety-two source pages;
`XingZ60` is largest at 16733 UTF-16 characters and 41720 bytes.
The census is `~/temp/agent/original-evidence-size-20260910.json`.
This is size evidence,
not proof of model comprehension or a new truncation policy.

## Completed compiled full-path check remains insufficient

`proc_ccdb` completed `translation-repair-temporal-implemented-path-probe-20260910` in 971 seconds.
Script:
`~/temp/agent/probe-temporal-implemented-path-20260910.mjs`.
Plan:
`~/temp/agent/temporal-implemented-path-plan-20260910.out`.
Log:
`~/temp/agent/temporal-implemented-path-probe-20260910.log`.
Report:
`~/temp/agent/temporal-implemented-path-probe-20260910/report.json`.

It executes actual `repairChunk` on frozen `.frozen-dist-8adb77fb9`,
without intercepting prompts.
The same-entry original is supplied through the implemented argument.
Completed prompt stores are merged into a new disposable store;
conflicting records stop setup instead of silently overwriting a response.
The bound remains one slice,
120 JSON requests,
360000 ms per exchange and 1200000 ms globally.
The run returned after eighty-three JSON calls.
It retained the second friendship item but removed the disclosure item,
and changed the group name to source script plus an explanatory gloss.
This does not complete task 23 or task 19.

The actual regional slate contained only deletion and a duplicate second-friendship item;
no correct disclosure draft survived from the two usable editors.
The regional selector declined that slate.
The whole-chunk slate contained those flawed drafts and a composite retaining the old incorrect disclosure.
It tied and used the existing indecision fallback.
Thus this result is not a selector silently discarding an available correct candidate.
Evidence:

- `~/temp/agent/temporal-implemented-path-results-20260910.out`
- `~/temp/agent/temporal-implemented-slates-20260910.out`
- `~/temp/agent/temporal-implemented-decisions-20260910.out`

The run logged 0.00506431 USD on Bedrock,
nine OpenRouter calls reporting zero,
and twenty-two unpriced Hyper plus thirty-three unpriced Synthetic calls.
The daily helper ran afterward.
Task 21 remains separate and no full Mio pass is active.
