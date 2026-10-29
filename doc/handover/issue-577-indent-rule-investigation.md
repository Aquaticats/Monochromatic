# Issue 577 indent rule investigation: handover

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/dev-script/task-util` is now `package/monorepo-task/core-exec`.

## Purpose

Close out the question "how hard is it to actually check indentation in our oxlint plugin"
(issue [#577](https://github.com/Aquaticats/Monochromatic/issues/577)),
ship the rule,
and leave the
follow-up queue in a state a fresh session can resume.

## What changed

The rule `stylistic/indentation` now ships in `package/oxlint-plugin/stylistic` with check and per-line
whitespace autofix,
engine and autofix tests covering every route,
and corpus-verified convergence.
The full record (model,
measurements,
retracted hypotheses,
workarounds,
upstream decision) is
`doc/troubleshooting/oxlint-typescript-indent-rule.md`.
The chain renderer was unified with the indent model in the same effort to stop a measured `--fix`
oscillation;
that history is in "Queue" and the troubleshooting doc's "Root cause".
Both corpus sweeps are recorded there,
and the results-only comment landed on issue #577 as `issuecomment-6339427646`.
The separate `stylistic/comma-dangle` type-literal corruption found during the sweeps is now fixed
(`0a49269a48`),
recorded in `doc/troubleshooting/oxlint-stylistic-comma-dangle-type-literal-fix.md`.
The queue's remaining items then landed:
`package/config/oxlint/src/rule/style.ts` enables `stylistic/indentation` at `warn` (`a0de3e051f`),
and one normalization sweep (`68415aeb53`) rewrote 21,441 leading-whitespace lines
in 1,845 git-clean files,
whitespace only.
The sweep's rules-only config pitfall is recorded in
`doc/troubleshooting/oxlint-config-default-categories.md`.

The format catch-up then landed as `3da31bdc3a`
(190 files,
36,532 insertions and 8,412 deletions,
9 fix passes),
and together with `68415aeb53` it was reported as issue
[#1794](https://github.com/Aquaticats/Monochromatic/issues/1794).
Of the 52 reported files,
32 were `lint:parse` gate artifacts
and 20 were real `.mjs` corruptions by the `no-restricted-syntax/no-arrow-function` autofix
(`3da31bdc3a` broke all 20;
`68415aeb53` broke 0).
The fixer fix landed with a guard test (`5544041a52`),
the parse gate as a mise task (`9e3f228e65`),
the 20 restorations as `20a3a73271` and `0c2f35836d`,
the glued-async regression as `b7b416ae2f`,
the `.d.*` transpile panic dodge as `97e7252959`,
and `Closes #1794` on `60d4795f95`.
The record is
`doc/troubleshooting/oxlint-no-arrow-function-unparseable-autofix.md`.

The leftover semantic-bridge teardown crash
(uncaught `Error: EBADF` at shutdown after the native channel died)
is fixed in `72773af2f5`.
`openSemanticFile` now classifies terminal channel failures at the adapter boundary,
retires the bridge once,
and teardown closes failure-safe with tagged warnings only (CXL).
The lifecycle state moved to
`package/oxlint-plugin/prefer-readonly-parameter-type/src/prefer-readonly-parameter-types/semantic-bridge-lifecycle.ts`.
A subagent's visitor-catch wiring collided with that module split and stood down;
adapter-wrapper retirement (PP4) superseded it.
The record is
`doc/troubleshooting/oxlint-typescript-semantic-bridge-epipe-teardown.md`.

The 2026-10-22 GAP proposals from these sessions were all adopted
through the question tool and landed in `AGENTS.md` (`70cc839d92`),
with the two oversized adopted clauses split into fresh `TPF` and `TRV` rules (RLM).
The measured landing record is `doc/planning/agents-md-gap-proposals.md`,
corrected in `1a59666c68`
after its first RLM claims proved word-half-only,
with the commit-message numbers corrected as commit comment `206152288`.

## Findings

Implementation size landed at 3,395 insertions and 165 deletions across 16 files (commits in "Commits"),
inside the 2,000 to 3,400 LoC estimate and about 3 times under the user's 10,000 LoC close-as-not-planned
threshold,
so issue #577 stayed open and was implemented on the user's go-ahead.

The four-route model (template exemption,
ancestor plus two with body-owner remap,
closer at the bracket
base with brace remaps,
suffix keyword at the owner line) matches 99.04% of the repository's 1,052,606 code
lines (7,091 `.ts` files,
probe `full4.json`,
2026-10-22).
A first `--fix` sweep with the rule alone rewrites 20,722 leading-whitespace runs in 1,792 files,
touches
nothing but leading whitespace,
reaches the fixed point in one pass,
and leaves zero diagnostics.
A second sweep with every fixture rule enabled reaches a fixed point at pass 12
(an eight-pass cap reads `stable=false` only because a few files need three cascade layers),
with zero `stylistic/indentation` or `stylistic(chain-per-line)` diagnostics in parseable files.
Of the changed lines,
9,921 are direct model disagreements and 10,801 are cascade followers of a moved
anchor line.

The landed normalization sweep (`68415aeb53`,
2026-10-22) ran over 7,333 candidates
(the corpus had grown from 7,091),
changed 1,845 files and 21,441 leading-whitespace lines with zero content-line
edits and zero line-count changes
(verified per file against `HEAD`),
and converged in one fix pass.
It ran the indentation rule alone with
`categories: { "correctness": "off" }`,
so the default-category fixes that a rules-only config would also apply
stayed out of the commit
(see `doc/troubleshooting/oxlint-config-default-categories.md`).

## Assumptions and limits

- Sweeping normalization is accepted (user:
  "I don't care about huge sweeping changes").
- Corpus continuation styles that write binary operands,
  union bars,
  and hanging function-expression
  bodies flat are treated as drift to normalize,
  not as conventions to preserve.
- `package/config/oxlint/src/rule/style.ts` enables `stylistic/indentation` at `warn` (`a0de3e051f`)
  and the one normalization sweep is committed (`68415aeb53`).
  The sweep snapshot `git status --porcelain` immediately before running and
  skipped the three concurrently dirty paths
  (`mise.toml`,
  `package/config/pnpr/config.yaml`,
  `scratchpad/`),
  so concurrent sessions' uncommitted work stayed untouched.
  Remaining limit:
  files that drifted under other already-enabled rules are queued for a
  format catch-up under "Queue",
  not normalized here.
- Scratch assets under `~/temp/agent/indent-probe-2026-10-22/` (`probe4.mjs`,
  `probe4b.mjs`,
  `sweep.mjs`,
  `spot.mjs`,
  `probe-render.mjs`,
  `pass-probe.mjs`,
  `full4.json`,
  `full4b.json`,
  `residuals.json`,
  `residuals-2.json`,
  `sweep-report.txt`,
  `sweep-report-2.txt`,
  `issue-577-comment.md`,
  `comma-repro/`,
  `pass-probe/`,
  `gfp-mutation-1.patch`,
  `gfp-mutation-2.patch`) are
  disposable;
  the troubleshooting docs carry everything needed to rebuild them.
- Measurement lesson:
  a lint-based corpus run is only as current as the built `dist` bundle it loads
  (a stale bundle once reported a false `stylistic(indentation)` versus `stylistic(chain-per-line)`
  oscillation;
  see "Measurement correction" in
  `doc/troubleshooting/oxlint-typescript-indent-rule.md`).
  Rebuild with `mise run //package/oxlint-plugin/stylistic:test:unit` before measuring.
- Test-helper lesson for future engine tests:
  an `offsetOf`-style `after` parameter must mean "strictly
  after",
  or chained marker lookups re-find the previous marker and stub trees collapse (this cost one
  false engine failure;
  see `package/oxlint-plugin/stylistic/src/utility/indent-expectation.unit.test.ts`).
- Assume the tree is never quiet:
  sweep only git-clean paths from a pre-run `git status --short` snapshot,
  never gate a sweep on a quiet tree.
- TypeScript 7.1 correction (user right,
  agent wrong):
  the classic API including `transpileModule` is present at `typescript/sync`
  and `typescript/async`;
  the claim that 7.1 has "no transpileModule" is retracted and must not repeat.

## Queue

1.  Done:
    the `stylistic/comma-dangle` type-literal corruption is fixed (`0a49269a48`),
    before any enablement or whole-set `--fix` sweep as required.
    The trigger,
    root cause,
    and landed fix live in
    `doc/troubleshooting/oxlint-stylistic-comma-dangle-type-literal-fix.md`.
1.  Done:
    `stylistic/indentation` is enabled at `warn` in
    `package/config/oxlint/src/rule/style.ts` (`a0de3e051f`),
    and the normalization sweep is committed separately (`68415aeb53`):
    21,441 leading-whitespace lines in 1,845 of 7,333 git-clean candidates,
    whitespace only,
    one fix pass.
1.  Done:
    the format catch-up landed as `3da31bdc3a`
    (190 files,
    36,532 insertions and 8,412 deletions,
    9 fix passes under the full root config,
    reviewed and committed separately).
    Sweep practice it established:
    assume the tree is never quiet;
    snapshot `git status --short` before the run,
    sweep only git-clean paths from that snapshot,
    and never gate the sweep on a quiet tree.
    Its parse fallout is recorded in
    `doc/troubleshooting/oxlint-no-arrow-function-unparseable-autofix.md`.
1.  Root `lint:parse` alias:
    add a root-mise alias for `//package/dev-script/task-util:lint:parse`
    once the concurrent root `mise.toml` work lands
    (that file is concurrently dirty and stays untouched here).
1.  `doc/*.mjs` files have no format owner:
    `mise run format:oxlint` does not reach them
    and no doc-family formatter claims them.
    Superseded by the user decision (2026-10-22)
    that no `.mjs` files should exist under `doc/` at all;
    the conversion or deletion is tracked as
    [#1795](https://github.com/Aquaticats/Monochromatic/issues/1795),
    so the format-owner question dissolves with the files.

## Commits

- `docs(troubleshooting): record the oxlint TypeScript indent rule feasibility investigation` (`48308b493b`)
- `docs(handover): queue the issue 577 indent rule decision` (`59cc14bb65`)
- `feat(oxlint-plugin-stylistic): add indentation checker and autofix` (`bee2e27eb7`,
  engine,
  rule,
  registration;
  its `Closes #577` auto-closed the issue on push)
- `test(*): add indentation fixtures and engine exports` (`2b4b5cefd1`)
- `fix(oxlint-plugin-stylistic): anchor unanchored root lines at column zero` (`1e6f2c0db3`)
- `fix(*): nest chain continuations on their anchor lines` (`310a3fa94c`,
  chain-per-line unification with
  the indent model,
  mixed-fixture convergence tests)
- `docs(troubleshooting): update the indent rule record to the shipped implementation` (`5edda3c34d`)
- `docs(handover): record the shipped indent rule and remaining queue` (`b3cad61ec6`)
- `docs(troubleshooting): apply semantic line breaks to the indent rule record` (`c9daef5380`)
- `docs(handover): apply semantic line breaks to the indent rule handover` (`68822cf102`)
- `docs(troubleshooting): record the comma-dangle type-literal fix corruption` (`3f1759034f`)
- `docs(handover): queue the comma-dangle fix and record the stale-bundle lesson` (`b5b6c17431`)
- `fix(*): stop comma-dangle inserting inside type-annotated pattern annotations` (`0a49269a48`,
  red-first fixtures,
  close-delimiter lookup skips the pattern's type annotation)
- `chore(config-oxlint): enable stylistic/indentation at warn` (`a0de3e051f`)
- `style(*): normalize leading indentation to the indent model` (`68415aeb53`,
  1,845 git-clean files,
  leading whitespace only,
  one fix pass)
- `fix(oxlint-plugin-no-restricted-syntax): make the no-arrow-function autofix emit parseable declarations`
  (`5544041a52`,
  guard test in the fix suite)
- `feat(dev-script-task-util): add lint:parse, the formatting-commit parse gate` (`9e3f228e65`)
- `fix(*): restore three .mjs files that 3da31bdc3a left unparseable` (`20a3a73271`)
- `fix(desktop-app-ide): restore the 17 .mjs files 3da31bdc3a left unparseable` (`0c2f35836d`)
- `test(oxlint-plugin-no-restricted-syntax): cover the glued async arrow form in the fix suite` (`b7b416ae2f`)
- `fix(dev-script-task-util): keep lint:parse alive on 7.1 transpile panics and parse .d.* safely` (`97e7252959`)
- `style(*): re-apply parse-checked catch-up formatting to the 20 restored .mjs files`
  (`60d4795f95`,
  closes #1794)
- `fix(oxlint-plugin-prefer-readonly-parameter-type): retire the semantic bridge instead of writing to a dead channel`
  (`72773af2f5`,
  lifecycle split,
  CXL-clean teardown,
  GFP-proven)
- `docs(troubleshooting): record the no-arrow-function unparseable autofix incident and workaround` (`312d80e321`)
- `docs(troubleshooting): record the semantic bridge EPIPE teardown failure mode` (`2c5478a8a2`)
- `docs(planning): record the AGENTS.md gap proposals from the 2026-10-22 sessions` (`c6c9a3dcff`)
- `docs(AGENTS.md): adopt the facade-probe, suite-reach, and bridge-exception tightenings` (`70cc839d92`,
  `CLAUDE.md` regenerated via `mise run sync:files`)
- `docs(planning): correct the gap-proposal RLM claims and record the landed split` (`1a59666c68`)
