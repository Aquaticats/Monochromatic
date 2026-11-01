# Translation repair handover 2026-09-06: checkpoints of 2026-09-11 and 2026-09-12

The record of the sessions of 2026-09-10 to 2026-09-15 for the V4.1 Flash judge admission,
the verified serving checkpoint,
and the preparation of the writer calibration that followed,
kept as those sessions wrote it and not as the current state.

Part of [Translation repair handover snapshot: 2026-09-06](translation-repair-handover-2026-09-06.md),
which lists every file of this record.

## Latest checkpoint: V4.1 judge admission on 2026-09-11

Task 31's paid judge calibration and post-admission verification are complete.
V4.1 Flash is seated for judging and general text/preparation.
No writer or image calibration has run;
those independent holds remain for tasks 35 and 36.
Tasks 32 and 34's reference and driver checks remain verified.

Task 31 launched at 2026-09-11 14:13:40 UTC through `proc_02bc`.
It completed at 14:19:10.310 UTC;
wrapper PID 2149953 and Node PID 2150064 are historical,
not active work.
Run state is `~/temp/agent/v41-reviewed-run-20260911-vbOoMQ`;
the private log is `~/temp/agent/v41-reviewed-judge-calibration-20260911.log`.
All 140 cells returned schema-valid replies from one model POST each,
with all nine peers heard on every question and no warnings or abandoned-call record.
All 140 actual ballot reasons were fully read.
V4.1 chose the reference fourteen times with no damaged pick or decline,
matching the peer median of fourteen and beating the maximum damaged count of one.
An independent review confirmed admission under the unchanged rule.
`83e632127` removed only its judge/general-roster hold.
Writer/reader arrays,
existing peer order,
provider order,
caps and quorum formulas are unchanged across the compiled sixteen-state boundary comparison.
The registered plan is `~/temp/agent/v41-reviewed-plan-20260911-9eo8in/plan.json`.
Its live budget read reports every provider wet.
The 14:07:49 UTC sample recorded Synthetic 13.350341871212121 percent weekly remaining,
2750 of 2750 five-hour units,
Hyper 250 credits,
Bedrock USD 185.38 and OpenRouter USD 267.73.
Full-entry readiness is still false.
Current post-admission runtime is frozen at
`package/module/translation-repair/node_modules/.frozen-dist-45e64e411`.
Build/types pass;
`v41-judge-seat-final-lint-20260911.out` reports zero warnings/errors;
`v41-judge-seat-final-unit-20260911.out` ends `unit exit 0` at line 9164.
Its digest is `sha256-tree-v1:3a48d50c5b1e329f6b642af6ba2b3e3fae5d7b37ed05bf04607b9d7c843ce3f9`.
`v41-judge-seat-final-boundary-20260911.json` compares this frozen artifact with pre-admission `993583ad5`.
The detailed reading is `v41-reviewed-judge-results-20260911.json` and `v41-reviewed-judge-ballots-20260911.out`.
Spend helpers ran:
OpenRouter reported USD 0.00050599 across fourteen calls,
including thirteen explicit zero costs;
Bedrock logged USD 0.00770297 from its pricing calculation across twenty-eight calls;
Hyper's fifty-six calls estimate 2.674203982384 credits including reported cache reads,
not a reported debit;
Synthetic's forty-two calls remain subscription usage without a per-call dollar price.

The source-reviewed replacement tuples are fixed in
[the reviewed fixture plan](../planning/translation-repair-reviewed-calibration-fixtures-2026-09-11.md).
They cover `gqt`,
`MTF_0615` and locally corrected `Y1Ran`,
with all seven deliberate variants independently read.
The corpus remains unchanged.
`fidelity-reference-*` now owns ranges,
hashes,
local-edit provenance,
donors and deterministic damage verification.
The compiled loader matched the reviewed tuple artifact in
`~/temp/agent/reviewed-fidelity-loader-verification-20260911.json`.
Targeted build,
types and reference/CLI tests passed before the subsequent matrix hardening.

Task 33 is complete.
An agent-created worktree from the linked source triggered the intended ignored-state copier,
which retained the global settlement lease after checkout.
The owned copier was stopped,
the active stale lease recovered through the wrapper,
and only the owned staging tree and abandoned pending claim were removed.
Commits recovered at `9f4034d15`.
The clean detached guard target was recreated from main without ignored-state copying.
`076a0cf0d` records the incident in
[the copy recovery record](../troubleshooting/cli-git-worktree-copy-settlement-timeout.md).
Older stages and concurrent changes were preserved.
The proposed `WCD` clarification is not applied.

The first guard target was detached at `9f4034d15`:
`~/temp/agent/translation-repair-guard-20260911`.
It used explicit dependency links and private proof files,
not a copied secret file,
and has now been removed.
Mise's process-scoped trust and native write/network sandbox were exercised:
inside writes succeed;
outside writes and writes through an outward symlink fail;
INET socket creation is denied;
provider credentials are absent from the child environment.
This is not a CPU/memory sandbox or a confidentiality claim about every readable file.

The first guard-removal run rebuilt every mutation successfully.
Nine categories produced the expected assertion failures;
coordinate validation,
edit overlap and donor overlap survived because downstream hashes masked the old tests.
`a0ec042a5` commits otherwise-valid controls isolating those guards.
A fresh main-created target at `82168f031` now verifies those controls:
`~/temp/agent/translation-repair-reference-guard-r2-20260911`.
The strict run checked thirty-three individual guard mutations,
including every coordinate predicate,
and required the intended named assertion for each ordinary nonzero test exit.
All thirty-three were detected;
restored build and tests passed.
Evidence is `~/temp/agent/fidelity-guard-r2-20260911/mutations.json` and `mutations.out`.
A string-anchor preflight failure was corrected before any mutation and is retained separately,
not counted as guard detection.
Both owned worktrees were subsequently removed through normal main-context commands without force.
Root-sentinel,
ignore and dry-run checks were repeated;
only the known helper symlink and proof files needed explicit removal.
`reviewed-guard-cleanup-20260911.json` records verified absence.
An attempted wrapped switch of the disposable target was refused as `config-untrusted`;
HEAD did not advance and no persistent trust was granted.
The second proof target was created from main at the required commit instead of granting persistent trust.

Independent review found that a mixed damage request could silently omit its unavailable family.
`deed1022d` produced the actual library,
trial-builder and native CLI failures in `reviewed-matrix-red-unit-20260911.out`.
`3b03c5d01` requires every requested family somewhere in the selected population.
`634cca76d` also binds metadata-only preflight to the configured corpus revision.
`reviewed-matrix-green-*` records passing build,
types and all reference/CLI suites after those changes.
`869f1fe69` isolates each positive native probe's payload namespace,
persists its logical plan and completed rows,
and labels capped prefixes as incomplete requested matrices.
Its build/types pass.
The native CLI consumed the actual pinned references through a wholly simulated transport:
identical questions in repeated invocations caused fresh exchanges in separate namespaces;
old state remained unchanged;
and SIGTERM before the second ballot left the plan and first row without a completed artifact.
The individual clean ballot remained separate from the underweight merged decline.
`fidelity-native-artifact-verification-20260911.json` records this boundary check,
with zero real model calls.
The verifier's first attempt overlooked the legitimate outer ledger directory;
that assertion was corrected without changing the successful native behavior.
Final package verification and corrected plan derivation now pass.
The current registered plan is `~/temp/agent/v41-reviewed-plan-20260911-eiI5so/plan.json`:
twenty-eight logical arrangements,
fourteen message-plus-schema questions,
and 140 model cells across the unchanged nine peers plus V4.1.
All 280 synthetic planner calls checked actual model-specific prompt equality;
none is model evidence.
The frozen loader's complete tuples match the independently reviewed artifact,
and the regenerated source/reference/damage set was fully read again.

Driver simulation exposed an incorrect assumption:
stage retries may repeat identical messages,
and `STAGE_RETRY_ROUNDS = 3` permits four total stage calls per cell.
That correction covered only the quorum loop.
Task 34's full source read found the separate existing unreadable-answer recovery round,
which appends `RECOVERY_NUDGE` after the loop.
The actual maximum is five stage calls per cell and 700 for the fixed matrix.
No production round is added or removed.
The 170-call logical simulation remains explicitly simulated evidence of the loop path,
not proof of the post-loop recovery or transport.
The old plan is superseded by the driver changes.

Task 34 now captures initial and recovery bodies through compiled provider clients without network,
derives physical-call bounds from the existing routing and HTTP retry layers,
and compares live bodies against registered routes.
Actual-client transport simulation passes:
140 cells,
144 stage calls and 146 fake model POSTs across all four gateways,
with one final ballot per identity,
HTTP retry,
cached stage retry,
final nudge and Hyper-to-OpenRouter budget failover exercised.
The production prompt cache delegates through `chatText`,
so the direct router's schema re-ask is not on this caller path;
no production re-ask was added.
Eleven positive/negative controls verify route/body checks,
global and per-cell bounds,
persistence refusal,
post-call failure and provider absence.
Simulated rows cannot become admission evidence.
See [driver verification](../planning/translation-repair-calibration-driver-2026-09-11.md).
The package formatter is `format:oxlint`,
not `format`;
`reviewed-reference-format-r3-20260911.out` passed before the latest changes.

Task 37's preparation failure is fixed and verified.
Task 35 is now paused for task 38's separate benchmark unit-scope defect;
no writer generation has run.
The deterministic sampler launches corpus-entry reads concurrently,
so preflight runs in a 2 GiB,
2 CPU,
512-PID container with no network and read-only corpus/source mounts.
The available terminal image supplies Git 2.52.0 with working `--no-lazy-fetch`;
Node 26.7.0 is bound read-only with its required `libatomic.so.1`.

The original sampler and a minimized 186-concurrent-read case exhausted V8's heap near 1 GiB.
Each read re-resolved Git,
whose shim check decoded the 4537488-byte executable.
An explicit resolved pin made both cases pass.
`d6be6e978` now resolves the executable once per sample batch without changing corpus data,
reader protections or sample selection.
The regression measures actual executable reads in a separate process:
three before,
one after,
zero with an explicit path.
All forty fixed sample rows exactly match the old resolved-pin control.
The logger EROFS warning was separately removed by a writable private log overlay,
not by changing the read-only corpus/source mounts.
Workspace is `~/temp/agent/v41-writer-20260911.sEDtepyo`;
state is in [writer preflight](../planning/translation-repair-writer-preflight-2026-09-11.md).
No host exhaustion probe or corpus edit was used.
Current writer-preparation runtime is frozen at
`package/module/translation-repair/node_modules/.frozen-dist-67823bc55`.
Build/types/Oxlint pass;
`v41-bench-resolution-final-unit-20260911.out` ends `unit exit 0` at line 9169.
The source trace is in
[bench Git resolution](../troubleshooting/translation-repair-bench-git-resolution.md).
The unstaged `mise.lock` change (tool-version downgrades only) was discarded on 2026-09-26 at the owner's word "Commit or discard mise.lock,
it doesn't matter";
a copy stays in the session scratchpad.

The exact frozen container sample and budget setup passed,
but full sample reading exposed a different issue.
`windward0032#14` has source heading only and incumbent heading plus three body paragraphs;
its body source is in sibling `windward0032#15`.
The compiled archive-floor validator rejects a heading-only candidate,
while retaining the incumbent passes.
A valid `lintong#0` heading-to-heading control passes.
The earlier conditional advice to keep the old sample was withdrawn after this counterexample.
The old launch script now refuses paid `run` mode.

Next actions:
complete task 38 using frozen production-prepared groups,
without weakening the archive floor,
changing the forty-round pooled-null rule or adding writer rounds;
freeze and read the revised forty-unit plan before any paid writer call.
The whole-parent prototype is rejected:
it changes task sizes and loses a child line-structure flag in `hulicaijia` section 7.
Shape-filtering parents leaves a different coverage distribution and is also rejected.
Use the existing configured block-pairing preparation on a preselected bounded pool,
account for those calls separately and refuse an uncorroborated fallback rather than dropping a parent.
Preparation and writer execution need separate plan/review gates.
Neither paid preparation nor writer generation has run for this new plan.

The first interface change retains final pairing seat outcomes in `BlockPairingOutcome`.
The outcomes-only `06862482e` snapshot passed build,
types,
zero-warning lint and the actual full-suite `unit exit 0`.
The temporary whole-parent type scaffold was removed.

The subsequent `readBlockPairingOutcomes` helper reuses the live reader and two-voice relation rule
for provider-free recipe interpretation.
`assertPairingSeats` rejects duplicated/empty electorates before live calls,
and unconfigured,
duplicated or reordered recorded identities before endorsements are counted.
Sparse ordered subsets and empty outcomes remain valid.
`PairingEvidenceError` has closed diagnostics naming only failure kind and numeric input position.
At `edb40e09d`,
build,
types,
zero-warning lint and the full suite pass;
`writer-pairing-identities-final-unit-r2-20260911.out` ends with `unit exit 0` at line 9189.
Five individual guard removals rebuilt and failed at the intended assertions in the bounded container;
restored source rebuilt and passed.
Proof is retained at `~/temp/agent/pairing-guard-proof-20260911`.
The owned guard worktree was removed and its absence verified.

A provider-free,
manually source-reviewed control recipe through actual production preparation
pairs the windward heading source 2069 to 2079 with target 11101 to 11126,
and its body source 2081 to 2419 with target 11128 to 12232.
Both unchanged-validator controls pass and all parent node memberships remain exactly once.
The legacy heading still fails,
while `lintong#0` passes.
This proves the mechanism,
not automatic acquisition or clean incumbent wording.
The remaining implementation must reuse production subdivision,
child line flags,
footnote relabeling and the actual `settleTranslateSlice` surface,
including target-only protection and supplied context.
Do not call the entire `preparePassEntry` as a shortcut:
its archive prose repair could let a benchmark peer author an incumbent without contributor attribution.
Preparation here buys correspondence evidence,
not uncredited baseline prose.
The native producer CLI is not changed yet.
The shared indexed-parent operation `prepareBlockPairing` and its production-shell integration
are implemented and verified at `97599f638`.
Build,
types,
zero-warning lint and full unit pass at line 9237 of
`writer-parent-preparation-final-unit-20260911.out`.
Fourteen cold/warm before/after fixture cases match frozen `06862482e` in complete results,
exact wire bodies and serialized cache bytes;
positive controls cover wire bytes,
cache bytes and line-structure Sets.
Snapshot `.frozen-dist-97599f638` has digest
`sha256-tree-v1:4a9ba31bddbc9cbf190830cf17625212f10d12a9c9c40263cb95d14a165e620d`.

The normalized provider-free pool prototype contains 283 parents;
a proposed forty-parent pool spans 32 entries.
`preparation-pool-prototype.json` and `preparation-pool-reading/` in the owned writer workspace
hold its exact inputs and full entry contexts.
Parent scope reading has begun in `preparation-pool-scope-reading.json`:
`AmbeR_the_anpa` and `ArtsEpiphany` have been read.
`ArtsEpiphany` is a preservation-only control,
not positive writing-quality evidence;
shared-incumbent accounting was checked provider-free with a positive control.
Full-entry parent scope reading has reached eighteen of the thirty-two selected entries.
No paid pool approval exists yet.

Task 39's footnote boundary is verified at `086fd9f6e`.
Task 38's preparation population and reviewed-plan gates remain unapproved.
The old `Y1Ran` definition map `2` to `1` and `3` to `2` could not close
because unmatched archive `1` occupied the destination.
The implemented closure retains that unmatched note under fresh label `4`,
separate from supplied correspondence and existing forced elimination.
Identity evidence is retained rather than inferred from equal spelling.

The syntax rewrite now uses strict document grammar,
exact raw marker spans,
normalized identifiers,
injective destinations and a reparsed occurrence-graph check.
Code,
metadata,
comments,
escaped literals,
URLs and attributes remain unchanged.
Declared English-original spans and their note anchors block the entire operation when touched.
Definition movement preserves distinct gaps and withholds changes across annotations or shared container delimiters.
The whole-document parser must not use the slice parser's lone-tag rescue:
the malformed-JSX regression exposed that false syntax proof.

Source through `75b769026` passed build,
types and the full unit process with exit zero.
`footnote-full-unit-20260911.out` has 9253 lines and ends with the completed lane-contest suite;
this direct invocation did not append the verification helper's separate `unit exit 0` marker.
The final wrapper-based verification supersedes that intermediate run.
The second formatter run reported zero warnings and errors;
`footnote-final-lint-20260912.out` also checks the subsequent boundary tests with zero warnings and errors.

Frozen `.frozen-dist-b3b6ee29c` passed the actual pinned `Y1Ran` consumer control.
All three definitions and references remain,
with final labels `1`,
`2` and `4`.
Front matter is unchanged;
all 45 target nodes appear once across 23 rebuilt slices with current ranges.
The shared `translateSliceInput` projection was exercised,
masked MDX compiled without execution,
and a repeat operation was a no-op.
`footnote-Y1-consumer-r3-20260911.out` records zero fetches and 194 ms inside the configured
2 GiB/2 CPU/512 PID no-network container.
This is a manually reviewed relation mechanism control,
not acquired correspondence or calibration approval.
The earlier attempts failed in the private harness's compiler resolution and assumed preparation-object shape,
not the relabel algorithm.

The intermediate guard proof at `2ce03f9a1` detected 22 individual removals,
each after a successful rebuild and at an intended normal assertion.
Restored build and registered tests passed;
peak memory was 726011904 bytes with no OOM/PID events.
Proof is retained at `~/temp/agent/footnote-guard-proof-20260911`.
The owned `translation-repair-footnote-guard-20260911` worktree was retained for the updated proof,
then removed normally after source restoration and artifact retention.

Independent review then identified further boundaries.
Actual default-budget preparation produced a marker-bearing container half at offsets 0 to 450;
reparsing that slice as a whole document failed.
Missing map domains could also silently no-op or partially apply.
Through `9196cb780`,
map domains are checked before edits,
and `footnoteRelabelOf` now takes complete source/target text plus slices,
projecting full-document markers through checked range/text boundaries.
Build,
types and review regressions pass.
The proposed astral-length defect was disproved by actual parser-positive controls:
the installed parser and lexer agree at 999 UTF-16 units and reject 1000.
NFC/NFD identifiers stay distinct.

Final footnote verification is complete at `086fd9f6e`,
with source code at `2cc8cfec1`.
Build,
types,
zero-warning lint and the full suite pass;
`footnote-final-unit-20260912.out` ends with actual `unit exit 0` at line 9308.
Real `preparePassEntry` fixtures cover cold/warm persisted caches,
current node/range/hash/line metadata,
protected withholding and forced elimination.
The existing monotone agreement leaves one crossing relation unaccepted,
so that incomplete original question is correctly not cached and is reacquired on replay.
Fully claimed cached questions add no replay calls.
The tests do not invoke archive prose repair or title lookup I/O.

Current `.frozen-dist-086fd9f6e` has digest
`sha256-tree-v1:6ec56fc725bc08c481d040f16a9d358d4826384c1a820879e2c73611472b83be`.
Its pinned `Y1Ran` rerun passes the complete mechanism control,
with unchanged expected output hash,
zero fetch calls and 307 ms measured in that run.
No timing comparison is inferred.
`footnote-Y1-current-consumer-20260912.out` is the current artifact-consumer evidence.

The updated proof detects 30 registered guard removals and binding mutations:
all mutant builds succeed,
the designated normal assertions fail,
restored source hashes match,
and restored build/tests pass.
Peak memory is 775876608 bytes with no OOM/PID-limit event.
Proof is retained at `~/temp/agent/footnote-guard-proof-r2-20260912`.
The owned guard worktree's root sentinels were absent,
its ignore rules were recorded,
and normal removal plus filesystem/worktree-list absence is verified.
Independent re-review found no concrete remaining footnote defect.
No additional speculative cases were added after that review.
The next action is task 38's policy-eligible population and reviewed preparation plan.
The [footnote work record](../planning/translation-repair-archive-footnote-collisions-2026-09-11.md)
is canonical.
No corpus edits or paid preparation,
writer or image calls occurred.

The unapproved preparation prototype also includes a production-declined entry:
`gqt/source-section/6/target-section/6` reads as a whole-page English original.
The selected `hakureico` parent carries a declared-original span.
Task 38 must apply the existing pre-purchase decline and sealed-span rules before freezing its population,
then redraw deterministically and revalidate reading anchors.
This is not post-outcome deletion or the rejected shape filter.
The provider-free record is `prototype-original-eligibility.json` and
`writer-prototype-original-eligibility-20260911.out`.

Task 38 now has explicit children:
40 for the eligible pool and source reading,
41 for qualified current-evidence journals,
42 for the native reviewed-input execution gate,
and 43 for bounded acquisition and final prepared-task review.
Task 40 has complete reading coverage and a verified frozen parent selection.
Task 41 is active;
35 remains blocked by 38.

The new provider-free policy population on frozen `086fd9f6e` contains 272 parents from 90 eligible entries.
It excludes whole-page originals `cheonwoomaeng` and `gqt` before selection and records missing-side `tdor`.
The unchanged spread rule draws forty parents across thirty entries,
with 20467 source and 71446 incumbent characters.
Population digest:
`17bef29954c1db3e0242b28fbdfe77c3c57e6f53e56e7a29003ed7d2cfdfaaab`.
Pool digest:
`235e18fe08d07b39c056e12f134479cdd803756cd4b601c533a289f2fd4404a2`.
Seven primary parents carry over from the original prototype selection.
`ArtsEpiphany` is still in the eligible population but is not selected by the new deterministic draw.
No selected parent intersects a declared-original span;
the population keeps that annotated parent rather than applying a shape filter.

All eighteen original reading records were revalidated.
Twelve complete-entry readings and six parent-scope readings carry to the new pool by exact hashes.
The journal now covers all thirty selected entries and forty parent scopes,
including reread changed scopes in carried entries and a bounded reread of the formerly truncated `shihai4h` file.
Current artifacts are `preparation-pool-policy-20260912.json`,
`preparation-pool-policy-reading-20260912/` and `preparation-policy-reading-journal-20260912.json`
in the owned writer workspace.
The independent truncation concern was withdrawn after its reviewer distinguished
its serialized context from the original visible tool response.

`frozen-parent-selection-20260912.json` binds sixty-three files with digest
`853fd30daef7464865356ac1ffa901de4ba5cb35b4ef80d28cf6a043c36be3be`.
The complete provider-free population rebuild matches the recorded pool,
with order,
coordinate and protection positive controls;
`writer-policy-selection-freeze-20260912.out` records exit zero.
The check ran without network in the 2 GiB,
2 CPU,
512 PID container.
This freezes parent identities and baseline coordinates,
not acquisition approval or final writer units.
Continue task 41's owning current-evidence journal and source-channel manifests.
The pairing-only replay seam is implemented:
`blockPairingQuestion` shares numbering/key construction,
`queriedBlockPairingDetails` preserves full production findings,
and `qualifyPreparedBlockPairing` verifies current outcomes,
usable quorum,
complete handoff,
media authority and source/target accounting.
All results explicitly remain `pairing-only`.
Historical cache and unresolved fallback cannot qualify.

The full `qualified-pairing-verification-r5-20260912.out` run passes build,
types,
formatting,
zero-warning lint,
full tests and Markdown lint;
its inspected full-unit terminal is line 9508.
`c62c6b37c` also composes actual relabel/reprepare with stale-question refusal and fresh qualification.
`c95318ebe` adds zero-question snapshot ownership coverage.
The final owning journal still needs exact documents,
ranges,
containers,
acquisition rosters/attempts,
source channels and persisted recipes.
The native reviewed CLI gate remains task 42,
not implemented by this helper.

Guard proof completed in the disposable
`~/temp/agent/translation-repair-qualified-guard-20260912`,
base `519e87092` plus the recorded `c95318ebe` test overlay.
The tool probe verified no credentials and the configured 2 GiB,
2 CPU,
512 PID environment.
The launcher uses no network.
`~/temp/agent/qualified-guard-mutations-20260912.out` records twenty-two successful mutant builds,
each designated normal assertion failure,
and passing rebuilt restored tests.
Peak memory was 667873280 bytes with no OOM/PID-limit events.
The target-key change altered refusal classification rather than admitting stale evidence;
the retained report distinguishes diagnostic bindings from acceptance constraints.
Proof is retained under `~/temp/agent/qualified-guard-proof-20260912`.
The cleanup audit verifies restored source,
the committed test overlay and owned generated paths,
with no root Git sentinels.
Only that owned worktree was removed through Git from the main worktree.
`removal.json` verifies filesystem and registration absence;
retained proof and unrelated worktrees remain intact.
The next implementation is the owning journal's exact document,
roster,
attempt and source-channel bindings,
not paid acquisition or writer execution.
Any eligibility-changing finding invalidates this freeze;
abort or reopen task 40 rather than replacing a parent.
Later target rewrites require fresh parse,
ranges,
nodes,
protection,
pairing and cache identity.
The `Mio` introduction keeps an explicit final-unit coverage/context question for its forward lead-in and English summary.
`hulicaijia` needs complete definition correspondence before preserving its extra brand footnote under a fresh label.

`XingZ60` parent 6 prompted a source-content review:
source `补` plus attribution faces an archive epitaph introduced by an archive-only revision.
An initial suggestion to exclude unsupported incumbent prose was withdrawn before any code or redraw:
it would wrongly exclude ordinary correctable writer incumbents.
Unknown nonempty source remains text unless a generic existing policy supplies another role.
The exact prepared-unit probe accepts a same-shape historical `Supplement` wording witness,
while a heading-only control fails the archive block floor.
The witness is not gold;
the epitaph is not protected as source-authoritative prose.
The parent remains in the unchanged draft,
with revised notes in `reading-XingZ60-floor-reviewed-20260912.json`.
Evidence is `source-floor-probe-20260912.json` and `writer-source-floor-control-20260912.out`.
Apply source-channel availability and modality qualification,
not incumbent-accuracy filtering,
before acquisition and final task admission.

No paid preparation,
writer generation or image calibration has run.

The shared pure `translateSliceInput` projection is implemented and used by `settleTranslateSlice`.
At `895843508`,
build,
types,
zero-warning lint and the full suite pass at line 9244 of
`writer-slice-input-final-unit-20260911.out`.
Eight before/after settlement cases match frozen `97599f638` in exact stage inputs and complete results,
with positive controls for request bytes and returned text.
Snapshot `.frozen-dist-895843508` has digest
`sha256-tree-v1:56d8d472b1a526faef861068d41e224c3640457b18ef0802a58b50992d18273c`.
The acquired-pairing-through-slicing regression and durable media/cache-byte controls also pass.
The mixed-parent parity extension covers implicit,
cached,
queried,
fallback and definition-separated parents in one document.

Keep the benchmark's native `runTranslateStage` operation and scoring unchanged;
do not add publication-disposition guards to calibration.
Remaining task 38 work is bounded pool qualification,
the evidence journal,
reviewed-plan execution gate,
actual prompt/transport bounds and automatic acquisition controls.
Qualification must use current queried outcomes and the configured quorum number of usable voices,
keep the existing per-relation threshold,
reject fallback,
and separately account for source insertions and deterministic media claims.
Cache eligibility alone is not semantic qualification.
Pairing's `runWindowedRounds` has no separate post-loop recovery nudge;
do not borrow the different writer/judge-stage bound without tracing its caller.

The formatter accepts package scope but not appended file paths in its current Node inline task.
Use `format:oxlint` without extra paths;
[the verified task-argument diagnosis](../troubleshooting/mise-inline-node-arguments.md)
records the failed form and isolated controls.
[Writer unit scope](../planning/translation-repair-writer-unit-scope-2026-09-11.md) is the current work record.
Task 36 follows writer calibration.
Both independent holds remain.
Persist actual model-independent question identities and model/provider cell identities separately,
and grade each actual question once.
The native default cap of sixteen rows is not complete admission evidence for the full reviewed matrix.
Task 27 and the initial-use/pre-window naming integration remain paused,
so the fresh Mio pass remains blocked.

## Verified serving checkpoint on 2026-09-11

The owner approved DeepSeek V4.1 Flash on Hyper and OpenRouter.
Task 30 has registered one distinct identity across both providers,
verified both actual compiled clients and the production router,
and retained separate judge,
writer and reader calibration holds.
No V4 Flash 0731 rating or exclusion transfers.
All exported production role arrays,
the nested repair model configuration and provider-aware seats across sixteen budget states
match frozen `b9d3b2ea0` exactly,
with a positive control showing the catalog really gained the new version.

The serving checkpoint was frozen at
`package/module/translation-repair/node_modules/.frozen-dist-24b1cbdb7`.
Build and types pass;
`deepseek-v41-lint-r3-20260911.out` has zero warnings/errors;
`deepseek-v41-full-unit-r3-20260911.out` ends `unit exit 0` at line 9164.
All logs are under `~/temp/agent/`.
A disposable native `judge-fidelity-probe --cap 0` invocation accepts the held-out model as a calibration candidate
without model generation.
Task 31 must measure role eligibility before any hold is released.

The current Hyper price snapshot is 2026-09-11,
verified against all 34 live rows and the provider's documented USD 0.05 per credit.
V4.1 Flash is quoted at 6 input and 24 output credits per million,
not unpriced or free.
Those are dated estimates,
not wire-reported debits.
The [admission record](../planning/translation-repair-deepseek-v41-flash-2026-09-11.md)
contains serving,
pricing and calibration evidence.

Task 27 is paused for this model work,
not completed.
Its acquisition API now retains coordinates through actual archive normalization (`8c29f4df7`),
including duplicate-line origins and stub removal.
The first normalization red also exposed a void result passed to `mapOverlapped`;
`bd61f76ef` fixed that misuse,
and the corrected red isolates only the missing coordinate mapping.
Naming,
provenance,
normalization and existing stub tests pass.
Remaining work is the history/parser/environment fixture catalog,
guard mutation proofs,
real Mio acquisition and package documentation.
Task 29's production use classification and task 28's renderer/repair/cache/artifact handoff remain absent.
The full-entry queue is still blocked and no full-entry pass is active.
