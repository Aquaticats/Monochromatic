# Translation repair handover snapshot: 2026-08-29

Part of the [current translation repair handover](translation-repair.md).

## Current stop condition

Historical evidence only.
Candidate M failed on 2026-09-01,
and no corpus run,
model call,
spent-prompt retry,
or successor implementation is authorized by this snapshot.

## Superseded handover snapshot from 2026-08-29

This section describes the pre-redesign pipeline and stopped generation-12 roots.
It is historical evidence,
not current instruction or authorization for another run.

### Repository state

- Worktree:
  `/var/home/user/worktrees/translation-repair`.
- Branch:
  `translation-repair-rebased`.
- Current pipeline commit:
  `0dc6e510c`.
- Naturalness-confirmation commit:
  `97fda9f95b424d8e16326e97169c06ab471ece97`.
- Exact-half corrective commit:
  `68c37da59c43529386cad78f3f8078d180d57f35`.
- Generation-13 continuous correction,
  prompt payload reuse,
  and one-provider operation landed in `1d16d89c4`.
- Generation-14 checkpoint,
  reviewed-rejection strategy history,
  and required-provider gate landed in `cf14b379b`.
- Deterministic correction-cycle guard landed in `6369228d5`.
- Generation-15 contributor-authority floor landed in `5211c54dd`.
- Visual-evidence guard landed in `d14e641e6`:
  distinct image-reading responsibilities,
  reviewed positives,
  and unresolved-asset pause before lanes.
- One-sided front-matter support landed in `56a47cb81`:
  source metadata insertion and exact target-only preservation.
- Translate version 10 prior-decline challenge landed in `504a407fc`.
- Translate version 11 unfilled-passage continuation landed in `ed756993b`;
  exact placement-cycle test landed in `1649c480d`.
- Archive-only block repair landed in `ccaad1f53`;
  contributor-floor test landed in `78ab244a2`.
- Consolidation version 16 final-selection recovery landed in `a84bb3a7a`;
  role-alias guard test landed in `f858ab538`.
- Translate version 12 and source-destination recovery landed in `0dc6e510c`.
- Unrelated dirty path:
  `.idea/.name`;
  keep it excluded.
- Pre-commit generation-13 verification emitted 862 `PASS` lines and zero `FAIL` lines.
- Final generation-14 verification emitted 865 `PASS` lines and zero `FAIL` lines.
- Final generation-15 verification emitted 866 `PASS` lines and zero `FAIL` lines.
- Final visual-evidence verification emitted 867 `PASS` lines and zero `FAIL` lines.
- Final one-sided front-matter verification emitted 867 `PASS` lines and zero `FAIL` lines.
- Final translate-version-11 verification emitted 869 `PASS` lines and zero `FAIL` lines.
- Final archive-block verification emitted 870 `PASS` lines and zero `FAIL` lines.
- Final-selection recovery verification emitted 871 `PASS` lines and zero `FAIL` lines.
- Final destination-recovery verification emitted 871 `PASS` lines and zero `FAIL` lines;
  its log contains both repair-standing and complete-pass destination-recovery tests.
- Continuous-repair audit is `doc/audit/translation-repair-continuous-repair-invariant.md`.
  Focused verification emitted 17 `PASS` lines and zero `FAIL` lines.
  All 82 handover-named GFP logs exist:
  41 red logs contain `] FAIL `,
  and 41 green logs contain `] PASS ` with no `] FAIL `.
- OXLint,
  TypeScript,
  and `git diff --check` passed before commit.
- Pre-commit advisor reviews identified generation-13 blockers;
  remediation followed,
  but no clean post-remediation advisor verdict was recorded before `1d16d89c4`.
- Generation-14 and correction-cycle commits each received clean pre-commit advisor verdict.
- Post-commit GFP mutations proved prompt reuse,
  stopped whole-entry retry,
  and third-correction guards fail when removed,
  then pass after restoration.
- GFP logs:
  `~/temp/agent/gfp-prompt-reuse-red-20260829.log`,
  `~/temp/agent/gfp-prompt-reuse-green-20260829.log`,
  `~/temp/agent/gfp-stopped-entry-red-20260829.log`,
  `~/temp/agent/gfp-stopped-entry-green-20260829.log`,
  `~/temp/agent/gfp-continuous-correction-red-20260829.log`,
  and `~/temp/agent/gfp-continuous-correction-green-20260829.log`.
- Generation-14 GFP logs:
  `~/temp/agent/gfp-generation14-durable-reuse-red-20260829.log`,
  `~/temp/agent/gfp-generation14-durable-reuse-green-20260829.log`,
  `~/temp/agent/gfp-generation14-reviewed-history-red-v2-20260829.log`,
  `~/temp/agent/gfp-generation14-reviewed-history-green-20260829.log`,
  `~/temp/agent/gfp-generation14-required-providers-red-20260829.log`,
  `~/temp/agent/gfp-generation14-required-providers-green-20260829.log`,
  `~/temp/agent/gfp-generation14-payload-validation-red-20260829.log`,
  and `~/temp/agent/gfp-generation14-payload-validation-green-20260829.log`.
- Generation-15 GFP logs:
  `~/temp/agent/gfp-v15-contributor-floor-red-20260829.log`,
  `~/temp/agent/gfp-v15-contributor-floor-green-20260829.log`,
  `~/temp/agent/gfp-v15-contributor-exclusion-red-20260829.log`,
  `~/temp/agent/gfp-v15-contributor-exclusion-green-20260829.log`,
  `~/temp/agent/gfp-v15-contributor-baseline-red-20260829.log`,
  and `~/temp/agent/gfp-v15-contributor-baseline-green-20260829.log`.
- Visual-evidence GFP logs:
  `~/temp/agent/gfp-visual-pass-boundary-red-20260829.log`,
  `~/temp/agent/gfp-visual-pass-boundary-green-20260829.log`,
  `~/temp/agent/gfp-visual-stopped-classification-red-20260829.log`,
  `~/temp/agent/gfp-visual-stopped-classification-green-20260829.log`,
  `~/temp/agent/gfp-visual-distinct-perspectives-red-20260829.log`,
  and `~/temp/agent/gfp-visual-distinct-perspectives-green-20260829.log`.
- One-sided front-matter GFP logs:
  `~/temp/agent/gfp-front-matter-insertion-red-20260829.log`,
  `~/temp/agent/gfp-front-matter-insertion-green-20260829.log`,
  `~/temp/agent/gfp-front-matter-admission-red-20260829.log`,
  `~/temp/agent/gfp-front-matter-admission-green-20260829.log`,
  `~/temp/agent/gfp-front-matter-source-shape-red-20260829.log`,
  `~/temp/agent/gfp-front-matter-source-shape-green-20260829.log`,
  `~/temp/agent/gfp-front-matter-stopped-red-20260829.log`,
  and `~/temp/agent/gfp-front-matter-stopped-green-20260829.log`.
- Translate version-10 GFP logs:
  `~/temp/agent/gfp-translate-v10-decline-challenge-red-20260829.log`
  and `~/temp/agent/gfp-translate-v10-decline-challenge-green-20260829.log`.
- Destination-recovery GFP logs:
  `~/temp/agent/gfp-translate-incumbent-source-floor-red-20260829.log`,
  `~/temp/agent/gfp-translate-incumbent-source-floor-green-20260829.log`,
  `~/temp/agent/gfp-consolidation-standing-source-floor-red-20260829.log`,
  `~/temp/agent/gfp-consolidation-standing-source-floor-green-20260829.log`,
  `~/temp/agent/gfp-destination-prewrite-guard-red-20260829.log`,
  `~/temp/agent/gfp-destination-prewrite-guard-green-20260829.log`,
  `~/temp/agent/gfp-destination-scheduler-red-20260829.log`,
  and `~/temp/agent/gfp-destination-scheduler-green-20260829.log`.
- Final-selection recovery GFP logs:
  `~/temp/agent/gfp-final-selection-recovery-condition-red-20260829.log`,
  `~/temp/agent/gfp-final-selection-recovery-condition-green-20260829.log`,
  `~/temp/agent/gfp-final-selection-cycle-red-20260829.log`,
  `~/temp/agent/gfp-final-selection-cycle-green-20260829.log`,
  `~/temp/agent/gfp-final-selection-provider-red-20260829.log`,
  `~/temp/agent/gfp-final-selection-provider-green-20260829.log`,
  `~/temp/agent/gfp-final-selection-alias-red-20260829.log`,
  `~/temp/agent/gfp-final-selection-alias-green-20260829.log`,
  `~/temp/agent/gfp-final-selection-evidence-red-20260829.log`,
  `~/temp/agent/gfp-final-selection-evidence-green-20260829.log`,
  `~/temp/agent/gfp-final-selection-scheduler-red-20260829.log`,
  `~/temp/agent/gfp-final-selection-scheduler-green-20260829.log`,
  `~/temp/agent/gfp-final-selection-twin-red-20260829.log`,
  and `~/temp/agent/gfp-final-selection-twin-green-20260829.log`.
- Archive-block GFP logs:
  `~/temp/agent/gfp-archive-anchor-red-20260829.log`,
  `~/temp/agent/gfp-archive-anchor-green-20260829.log`,
  `~/temp/agent/gfp-archive-post-anchor-quorum-red-20260829.log`,
  `~/temp/agent/gfp-archive-post-anchor-quorum-green-20260829.log`,
  `~/temp/agent/gfp-archive-revise-blocks-retention-red-20260829.log`,
  `~/temp/agent/gfp-archive-revise-blocks-retention-green-20260829.log`,
  `~/temp/agent/gfp-archive-naturalness-red-20260829.log`,
  `~/temp/agent/gfp-archive-naturalness-green-20260829.log`,
  `~/temp/agent/gfp-archive-inner-cycle-red-20260829.log`,
  `~/temp/agent/gfp-archive-inner-cycle-green-20260829.log`,
  `~/temp/agent/gfp-archive-outer-cycle-red-20260829.log`,
  `~/temp/agent/gfp-archive-outer-cycle-green-20260829.log`,
  `~/temp/agent/gfp-archive-contributor-floor-red-20260829.log`,
  and `~/temp/agent/gfp-archive-contributor-floor-green-20260829.log`.
- Translate version-11 GFP logs:
  `~/temp/agent/gfp-unfilled-followup-evidence-red-20260829.log`,
  `~/temp/agent/gfp-unfilled-followup-evidence-green-20260829.log`,
  `~/temp/agent/gfp-carried-insertion-final-guard-red-20260829.log`,
  `~/temp/agent/gfp-carried-insertion-final-guard-green-20260829.log`,
  `~/temp/agent/gfp-insertion-placement-cycle-red-20260829.log`,
  `~/temp/agent/gfp-insertion-placement-cycle-green-20260829.log`,
  `~/temp/agent/gfp-translation-provider-interruption-red-20260829.log`,
  `~/temp/agent/gfp-translation-provider-interruption-green-20260829.log`,
  `~/temp/agent/gfp-unfilled-scheduler-stopped-red-20260829.log`,
  and `~/temp/agent/gfp-unfilled-scheduler-stopped-green-20260829.log`.

Artifact schema remains 9.
At `0dc6e510c`,
cache generations are consolidation 16,
lane contest 5,
translate 12,
refine 4,
repair 30,
and pairing 2.
Preparation identity remains v2.

### Current quality mechanism

Absolute naturalness acceptance requires defect-discovery review followed by
substantively distinct prior-acceptance challenge of exact candidate.
Each responsibility settles at exact-half usable quorum,
rounded up for odd rosters.
Any usable rejection heard before bounded settlement remains decisive and enters continuous correction.
Correction uses latest exact rejected text and structured findings,
has no finite quality ceiling,
and records any failed strategy into materially different next prompt.
Exact repeated correction task is detected before dispatch and pauses as `INCOMPLETE`;
this guard exists because durable payload replay otherwise makes history-loss cycle costless enough to starve timers.
Schema-9 reader accepts arbitrary complete digest-bound correction chains.

Absent-passage translation now continues from latest exact rejected slate and findings.
Insertion placement continues from latest coverage verdict,
anchored target evidence,
missing destinations,
and shortfall outcome.
Exact task cycles and provider silence pause as `INCOMPLETE` without whole-entry retry.
Full coverage records passage carried elsewhere and final would-ship page must retain every exact anchored proof region.
This retention is intentionally conservative:
semantically equivalent rewrite that removes exact region can pause until strategy changes.
Restarting unchanged cycle replays durable payloads and re-derives same pause;
new strategy or evidence is required before relaunch.

Archive-only wording no longer terminates preparation unreviewed.
Source-supported retention requires substantive exact quote inside expected aligned section;
editorial retention requires contributor,
citation,
media,
or comment shape.
Post-filter exact-half participation,
any-revise rejection,
two distinct naturalness responsibilities,
contributor survival,
and independent correction selection all apply.
Selected corrections reprepare document and corrected preparation drives lanes,
artifact,
and page.
Repeated correction task or archive state pauses as `INCOMPLETE`.
Remaining unclaimed blocks are intentionally reviewed again after sibling revision,
because parser locations and source context may have changed.

Unendorsed final standing now remains inside consolidation.
Version 16 carries prior selection slate,
role-aliased producer and judge relations,
selection ballots,
gate ballots,
terminal,
and findings into a distinct recovery producer prompt.
Raw known producer and judge ids are replaced inside candidate text,
ballot reasons,
gate reasons,
and findings while self-vote relations remain readable.
A roster id belonging only to a lost voice does not enter alias registry;
this known bound carries no candidate or ballot relation.
Only gate-endorsed fresh consolidation may return from unsafe standing path.
Provider silence,
caller abort,
and exact failed-evidence cycle pause as `INCOMPLETE`;
identical twins share final safe chain and no unsafe intermediate settlement persists.
`assertFinalSelectionSettled` remains defensive and maps to stopped work.

Source destinations now have two stage-local recovery paths.
Translate version 12 excludes archive incumbent that fails deterministic source atoms and treats fallback as absent.
Consolidation validates ordinary standing as well as syntax-bearing standing,
so contest-endorsed repair wording missing a source destination enters version-16 recovery.
Incumbent validation intentionally uses archive text as its own page-shape baseline;
source-derived atoms are the meaningful floor there.
Final page destination comparison occurs before filesystem mutation and is a defensive `INCOMPLETE` invariant.

The task-24 terminal-quality register is closed at code level:
scripted integration,
full suite,
and GFP pass.
Live provider rejection has not yet traversed every new recovery loop;
that is task 26 and no production-readiness claim precedes it.
Exact deterministic cycle is accepted policy:
it blocks publication indefinitely until prompt strategy,
roster,
or evidence changes,
and is not fallback authorization.
`UnansweredContestSliceError` remains ordinary resumable `ERROR` because it is artifact/tally inconsistency,
not quality verdict.

Every direct roster round now starts straggler grace at exact-half participation.
Grace may collect more responses,
but no stage requires every provider seat.
Participation quorum remains distinct from two-vote corroboration thresholds in pairing and comparative gates.
Either provider may supply entire exact half;
missing provider key marks that provider dry rather than refusing launch.
Both keys absent still refuse launch.

Run client memoizes first model-plus-canonical-prompt payload.
Exact duplicate calls reuse in-flight or completed payload rather than calling provider again.
Corpus pass stores raw payloads beneath run-root `prompt-payloads/`;
restart reconstructs correction state without provider resend and continues at first unseen prompt.
These records contain corpus/model wording and must remain unquoted and outside Git.

Schema-9 `confirmations` binds earlier acceptable review to exact candidate,
paragraph digests,
roster order,
and decisive-round order.
Runtime produces confirmation evidence by construction.
Reader accepts missing field only for legacy schema-9 readability,
so fresh validation must come from current writer rather than reconstructed artifact.

`hf:nvidia/NVIDIA-Nemotron-3-Super-120B-A12B-NVFP4` and
`hf:zai-org/GLM-4.7-Flash` remain unreachable from every active stage.
`hf:zai-org/GLM-5.3-Flash` uses verified Synthetic route without inherited GLM-5.2 standing.

### Stopped validation snapshot

Snapshot began 2026-08-29 at 12:14 UTC.
Both generation-12 processes were later stopped after non-conforming behavior was identified.
Neither root is resumable under generation 13 or publication evidence.

#### `Weideriche_`

- Process:
  `proc_a259`.
- Pipeline digest:
  `sha256-tree-v1:dc788a666af6bce37b37c55d4d674ac99185656795327d120413c3e1304aec9f58`.
- Run root:
  `~/temp/agent/validation-Weideriche-schema9-half-quorum-v12-20260829`.
- Log:
  `~/temp/agent/validation-Weideriche-schema9-half-quorum-v12-20260829.log`.
- Snapshot state:
  repair and translate complete;
  lane contest active.
- Post-snapshot first-attempt tally:
  `status=ERROR ms=4840305 aborted=false error=slice 1 did not meet absolute naturalness floor`.
- First attempt wrote no artifact or page and queued automatic reattempt with 13 additional cache records.
- Process was stopped during incorrect whole-entry second attempt.
- Expected artifact if published:
  `~/temp/agent/validation-Weideriche-schema9-half-quorum-v12-20260829/artifacts/Weideriche_.json`.
- Expected page if published:
  `~/temp/agent/validation-Weideriche-schema9-half-quorum-v12-20260829/fixed/people/Weideriche_/page.en.md`.
- Verification log to create:
  `~/temp/agent/verify-Weideriche-schema9-half-quorum-v12-20260829.log`.
- Reading root to create:
  `~/temp/agent/read-Weideriche-schema9-half-quorum-v12-20260829`.

Stopped all-seat partial root is
`~/temp/agent/validation-Weideriche-schema9-confirmed-naturalness-v11-20260829`;
its log is
`~/temp/agent/validation-Weideriche-schema9-confirmed-naturalness-v11-20260829.log`.
That root,
failed fresh schema-9 roots,
and observability-only root are not publication or readiness evidence.

Generation-11 replay roots and logs remain mechanism evidence for rewriter prompting,
selector eligibility,
reviewer nondeterminism,
and repeated-acceptance diagnosis.
Key replay root is
`~/temp/agent/replay-Weideriche-correction-rewriters-v11-20260829`;
its private log and candidate-review logs remain usable only without quoting their wording.

#### Pull request 386, `Carena0442`

- Process:
  `proc_3a1c`.
- Pull request:
  `https://github.com/one-among-us/data/pull/386`.
- Pull-request head:
  `a80634a674f94861ea3b7056fba054ca9eab1a2c`.
- Fixture pipeline digest:
  `sha256-tree-v1:2231798fcc453ccb9fe7ff688f4690ec7662e22ae70b833f91d70c7ec5cc9f58`.
- Run root:
  `~/temp/agent/validation-pr386-Carena0442-schema9-half-quorum-v12-20260829`.
- Log:
  `~/temp/agent/validation-pr386-Carena0442-schema9-half-quorum-v12-20260829.log`.
- Provenance:
  `~/temp/agent/pr386-Carena0442-run-provenance-20260829.md`.
- Snapshot state:
  22 slices prepared;
  first non-metadata repair slice active.
- Process was stopped after same-prompt repetition was rejected as independence mechanism.
- It wrote no artifact or page.
- Expected artifact if published:
  `~/temp/agent/validation-pr386-Carena0442-schema9-half-quorum-v12-20260829/artifacts/Carena0442.json`.
- Expected page if published:
  `~/temp/agent/validation-pr386-Carena0442-schema9-half-quorum-v12-20260829/fixed/people/Carena0442/page.en.md`.
- Verification log to create:
  `~/temp/agent/verify-pr386-Carena0442-schema9-half-quorum-v12-20260829.log`.
- Reading root to create:
  `~/temp/agent/read-pr386-Carena0442-schema9-half-quorum-v12-20260829`.

Minimal Git fixture contains exact pull-request versions of six changed files.
Production `corpus-pass` has no pull-request input flag.
Uncommitted throwaway pipeline fork changes corpus commit and exposes clone location through
`TRANSLATION_REPAIR_CORPUS_DIR`.
That variable belonged to the fork alone.
Production has read `TRANSLATION_REPAIR_CORPUS_CLONE_DIR` and `TRANSLATION_REPAIR_CORPUS_COMMIT`
since `b0a79eb66` (2026-09-01), so later pull-request runs need no fork.

Startup process `proc_80b4` omitted mise;
stable log is
`~/temp/agent/validation-pr386-Carena0442-startup-without-mise-20260829.log`.
Startup process `proc_d541` used mise before encrypted local environment was linked;
stable log is
`~/temp/agent/validation-pr386-Carena0442-startup-without-local-env-20260829.log`.
Both made no model call,
produced no run root,
and are not evidence.

### Completion gates for future fresh runs

1.  Inspect terminal `TALLY`;
    process exit zero is insufficient.
2.  Count page and artifact outputs.
3.  Require current artifact schema 9,
    preparation identity v2,
    metadata slice and index zero,
    and matching corpus and pipeline identities.
4.  Run `verify-published` against exact run root.
5.  Read complete published page for fidelity,
    naturalness,
    structure,
    contributor authority,
    and image-derived claims.
6.  Retain logs,
    artifact,
    published page,
    verification output,
    timing report,
    slice-cost report,
    and provenance record.
7.  Treat quality rejection as stage-local repair work,
    never terminal `do not publish` answer.
    `status=INCOMPLETE` means operational work remains and is not success or quality verdict.

### Time-to-complete work before another Carena run

Owner requires fresh `Carena0442` median below two hours without weakening quality.
Stopped generation-12 run is phase evidence,
not completion-time sample.
Use `doc/planning/translation-repair-entry-time-to-complete.md` as measurement protocol.
No new validation starts before guard proofs,
commit,
and completion-path plan.

Stopped runs overlapped and shared provider capacity.
Their elapsed-time ratio is not matched concurrency evidence.
Separate time to quorum from post-quorum grace,
completed calls from abandoned calls,
and cold generation from resumed cache work.
Pair every timing result with actual publication-quality outcome.

### Next actions

1.  Analyze stopped Carena log by phase and define sub-two-hour matched measurement.
2.  Optimize only measured completion path without weakening quality.
3.  Rebuild pull-request fixture worktree and launch fresh roots only after quality and performance gates,
    passing `--require-providers synthetic,hyper`.
4.  Verify every successful page and artifact,
    then read complete output.

There is no release deadline.
Strict actual-output quality remains gate.
Mechanical,
schema,
modality,
replay,
and reconstructed-context checks do not establish readiness.
