# Translation repair history, first handover document: package completeness pass, dist-import alignment and milestone three scoping, 2026-07-23

The package completeness pass and dist-import alignment of 2026-07-23,
and the scoping of milestone three with the option the user picked.

Part of [Translation repair history: the first handover document, to 2026-08-15](translation-repair-history-first-handover-to-2026-08-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Package completeness pass and dist-import alignment

PKG COMPLETENESS PASS COMPLETE (2026-07-23,
user "Continue."
after
closure;
the handover's NEXT AFTER CLOSURE named this phase).
README refreshed with milestone-two completion and the
editorRuleAddendum contract knob (commit `479bf5a28`).
API surface audit (mechanical barrel-versus-module diff,
scratchpad
`api-surface-audit.ts`):
three accidental omissions surfaced into
`index.ts` (commit `0482bb9e6`):
`locateQuote` + `QuoteLocation`
(whole module missed the barrel),
`CategoryRemap` (return type of
public `remapCategoryLeaf`),
`MIN_DISPATCH_BUDGET_MS` (parity with
the public repair floor).
The audit's stale flag on `BenchmarkEntry`
was a false positive:
`benchmark.ts` re-exports it from
`prepare-entry.ts`.
Coverage gap map (scratchpad `coverage-map.ts`,
exported symbol to
importing test):
direct tests added across six commits
(`6bd1dccb7`,
`959b81df8`,
`14b3a3fcd`,
`235a28583`,
`5ca27b00b`,
`601f3bbb1`) for locateQuote,
normalizePunctuation,
the JSON
guards,
remapCategoryLeaf,
isPanelVoteState,
parseModelJson,
formatUsageNote,
extractCompletion + readUsage + SyntheticHttpError (renamed `ProviderHttpError` on 2026-10-19)
(every contract-violation detail),
armCallDeadline (expiry,
forwarded abort,
pre-aborted caller,
disposal),
fetchTransport
(previously zero test references;
header copy,
GET body omission,
status passthrough,
dependent-signal abort),
prepareBenchmarkEntry
(previously zero test references),
all four prompt sheet builders
(critic,
derivability,
resolution,
restoration judge) with their
verdict guards,
buildFootnoteGraph (both conventions,
every
finding kind),
parseMarkdownBody,
and a DEFAULT_JUDGE_MODEL_IDS
catalog-membership invariant.
Indirect coverage judged adequate per TC2 (branch-named tests in
callers):
repairChunk and the four pipeline stages via
repair-translation's end-to-end suite (happy,
checker-refusal,
no-issues,
non-translation block),
exchangeWithRetry and
attemptStageCall via the client and quorum suites,
computeScorecard
via the benchmark suite.
Closing verification:
85 unit suites pass,
oxlint 0 warnings 0
errors,
lint:types exit 0.
DIST-IMPORT ALIGNMENT COMPLETE (2026-07-23,
user "Align it.",
resolving the open question this section used to carry).
Every unit
test now imports package behavior from the built bundle per the
testing-practices skill;
none import sibling source anymore.
Enablers:
the package was scaffolded without its sibling one-liner
rolldown config so no dist bundle could ever build (source-importing
tests hid this);
the config landed and the bundle plus `.d.mts` now
build.
SUPERSEDED 2026-08-06 (user "Please turn this pkg into `node`
builds only."):
the target moved from `neutral` to `node`,
so the
import path is now `dist/final/node/index.mjs`.
Rolldown's `cleanDir`
clears only the output directory it writes,
so a checkout that built
before this change keeps a stale `dist/final/neutral/` that nothing
removes.
Harmless,
since no manifest entry and no import points at it
now,
but delete it rather than wondering why both exist.
A pre-alignment audit proved every test-imported symbol
already public (the PKG surface pass had closed the last gaps).
Mechanical codemod (scratchpad `dist-import-codemod.ts`) merged
each file's relative imports into one dist import with inline type
markers.
Verification:
85 suites pass against dist,
oxlint 0/0,
lint:types exit 0,
and the `buildAndTest` task exercises the same
sequence end to end.

## Milestone three scoping and the chosen real-corpus pass

MILESTONE THREE SCOPING PROPOSED (2026-07-23,
user "Scope milestone
3").
Deliberation only;
no implementation authorized yet.
Grounding facts measured this session:
`derive-seeds.ts` emits only `kind: 'deletion'` seeds while
`seeded-error.ts` already plants deletion,
replacement,
and
insertion;
no checkpoint serialization,
policy-file seam,
or
dossier (entities,
terminology) module exists in src
("checkpoint" appears only in design comments,
and
`tally-votes.ts` notes adjudication is replayable).
Options,
ranked A > B > C > D:
- A (RECOMMENDED),
  real-corpus production pass:
  run the proven pure
  fn unseeded over all 92 pinned pairs through the budgeted
  accumulation loop until every entry carries a settled status
  (repaired,
  unchanged,
  or blocked-non-translation;
  known-hard
  entries such as BI4PBV produce honest degraded statuses,
  nothing
  is excluded).
  Artifacts (issues with fates,
  repaired text,
  findings,
  status) stay outside the repo because the corpus is
  UNLICENSED;
  the handover records content-free tallies only.
  Headline gate:
  precision of accepted issues on a human-graded
  uniform sample (proposed 50 issues at a proposed 0.9 bar,
  user
  sets both),
  because a judge ensemble drawn from the same seven
  models re-affirming its own panel's acceptances is circular;
  a zh-anchored judge crosscheck over all accepted issues is the
  secondary,
  machine-graded number.
  Safety invariants:
  zero
  deterministic-gate violations,
  zero regression-majority
  selections,
  unchanged or blocked wherever nothing beats the
  input.
  Cost extrapolated from the measured DarlinChit
  full-pipeline datum (523 s,
  ~8.5 weighted units):
  92 entries on
  the order of 13 hours wall and 800 weighted units spread over
  days of budgeted runs,
  inside regeneration;
  larger entries chunk
  into more calls,
  so that extrapolation is a floor.
  Pro:
  measures the one unmeasured dimension that matters
  (real-error precision;
  milestone one deliberately skipped
  precision grading),
  meets the recorded deferral condition
  ("until the pure fn proves itself"),
  and produces both the
  corpus deliverable and the data that would justify B,
  C,
  or D.
  Con:
  the headline sample needs user grading time,
  and artifacts
  must live outside the repo.
- B,
  broadened seeded benchmark:
  derive replacement and insertion
  seed classes modeled on the real error seed bank (meaning
  inversion,
  fabricated specifics,
  policy-violating additions);
  grade detection recall,
  within-region precision,
  and repair
  against exact planted truth.
  Pro:
  the planting substrate already
  exists,
  ground truth is exact,
  no human grading needed.
  Con:
  a synthetic proxy for the same unknown A measures directly,
  and it spends another accumulation loop before any real-corpus
  value ships.
- C,
  interactive steering driver:
  checkpoint serialization at
  stage boundaries plus typed steering operations (approve/strike
  issue,
  correct alignment,
  lock wording,
  force verdict) per the
  settled architecture.
  Pro:
  designed,
  and required for real
  adoption.
  Con:
  which steering operations matter is best learned
  from A's real output,
  and the consumer form is deliberately
  open.
- D,
  calibration bundle:
  canary calibration feeding panel weights,
  MiniMax scheduler weighting,
  judge-universe exclusion of blocked
  entries,
  per-model editor slates.
  Pro:
  all are recorded
  follow-ups.
  Con:
  explicitly "none yet requested",
  no driving
  number,
  and the ensemble currently absorbs the quirks.
Adjacent-pair reasons:
  A over B because A measures the real error
distribution and delivers corpus value while B proxies it;
B over C because B extends a proven harness toward a measurable
gate while C's requirements stay unknown until real output exists;
C over D because C has a designed contract while D lacks any
driving number.
USER PICK (2026-07-23):
  A,
  the real-corpus production pass,
  chosen
from the ranked options;
  the proposed defaults (50-issue uniform
sample,
  0.9 precision bar) were accepted without notes.
MILESTONE THREE IS THE REAL-CORPUS PRODUCTION PASS.
  Execution
follows the milestone-two accumulation pattern:
  budgeted runs,
per-entry artifacts outside the repo (UNLICENSED corpus) under
`~/temp/translation-repair-corpus/`,
  content-free tallies here.
