# Depot trial interview

Note 2026-10-29: the runner scripts this document names under `package/git-policy` (`cli/bin`,
`cli/native-interop` and `cli.fuzz/bin`) are TypeScript files now. Read every `.mjs` file name here as the same
name ending in `.ts`; the driver `old-format-fixtures` is `old-format-driver`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/desktop-daemon/hall-monitor` is now `package/hall-monitor/daemon`;
> `package/dev-script/page-weight` is now `package/page-weight/cli`;
> `package/desktop-app/ide` is now `package/ide/app`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/rust-module/jsonc-edit.fuzz` is now `package/jsonc-edit/core-rust.fuzz`;
> `package/module/hyperscript` is now `package/hyperscript/core`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/forbidden-strings.fuzz` is now `package/forbidden-strings/cli.fuzz`;
> `package/rust-module/forbidden-regex.fuzz` is now `package/forbidden-strings/core.fuzz`;
> `package/cli/vmsync` is now `package/vmsync/cli`.

## Published research and accepted first interview round on 2026-10-18

Research tip `64d814b21cae7399270cfcc244ad98c78aa905ec` is published through the isolated merge worktree.
Direct remote readback is `3bd497b16a46b08f45a06d09b8c767ce83dbcad4`;
the requested tip is verified in its ancestry.
Publication used a normal push without force.
The private controller journal is
`latest-research-publication-64d814b21cae7399270cfcc244ad98c78aa905ec.private.json`.
The accepted first-round answers and permanent GitHub-hosted execution correction are subsequently published
at `8f6cc54eb38a46dc9c4702e06d56c80508fa3910`,
with requested tip `2b39f13df685e79a115f2f91ba388d0991a539b0` verified in its ancestry.
The matching private journal records a successful normal push and exact remote readback.
All four changed CI documents pass scoped Markdown lint and CommonMark rendering,
with no missing local links or italic spans.
The rendered provider-allocation paragraphs have been read.
The private renderer initially rejected the routing study because it was absent from its registered file list;
after registering that owned document,
the complete scoped render passes.
The Git policy replayed the formatting commit during concurrent landing;
the final published requested tip is the replayed identity,
not the discarded prepared identity `aad99ed01e36ac42152c5c389703dddf9960ba59`.
The finite factual execution and reconciliation queue is closed with the limits retained in the report.
This does not assert complete product-feature or source-branch coverage.
No fresh independent whole-report review ran after the agent service quota was exhausted.

Correction (2026-10-18):
the independent whole-report review has now run and is recorded in
`doc/planning/depot-expanded-ci-evaluation-independent-review.md`.
The later pending-review
statements in this handover are historical;
the record and the review verdict are current.

The first interview round is answered through the question tool:

- Q1 is accepted:
  use a regression ratchet.
  Enforce known-good checks and block new or changed failures.
  Setup,
  discovery and artifact-integrity failures always block.
  Existing source failures require exact baselines;
  raw `FAIL` log tokens are insufficient because the passing harness controls deliberately emit them.
- Q2 is accepted:
  core checks on PRs and the wider matrix weekly.
  This replaces the recommended nightly breadth.
  Private personal fixtures stay local.
  Dependency-based selection still needs qualification because the frozen task graph has a cycle.
- Q3 is accepted:
  staged native promotion.
  Introduce Depot-managed runners under GitHub orchestration,
  then qualify selected Linux workflows for native Depot CI against explicit acceptance criteria.
  This is a rollout order within a permanent hybrid,
  not a path to making Depot the default for all eligible Linux work.
  Native paused-session attachment is still unqualified.

The user explicitly clarified that Depot never fully replaces GitHub Actions.
GitHub-hosted runners themselves remain heavily used,
including ordinary eligible work rather than only platform exceptions or emergency fallback.
Retaining GitHub orchestration while moving its execution to Depot does not satisfy this requirement.
The permanent design has GitHub-hosted execution,
Depot-managed GitHub runners and selected native Depot CI workflows.
Provider placement follows workload suitability and measured benefit within that constraint.
The report's former description of native Depot as the Linux orchestration pool is retracted.
The user's correction confirms the intentional hybrid already recorded in the Q14 trial answer.
It does not reopen provider replacement as an interview choice.

The Developer baseline,
additional-pool approach and USD 200 all-provider planning ceiling remain settled.
No production implementation is unlocked by posting this round.
The second interview round is answered through the question tool.
Q4 accepts standing bounded authority for recurring overage.
Q5 accepts one reserved automatic recovery attempt for classified infrastructure failures.
Q6 keeps the correction in CI documents and leaves `AUT` unchanged.
The [concrete rollout plan](../planning/depot-expanded-ci-rollout.md) consolidates these answers.
The plan and accepted operating policies are published at `a08edd938abdcffb84cf902a458f14b42ca340a3`.
The publication journal verifies the exact requested tip and direct remote readback with a normal push.
The plan,
updated report and handover pass scoped Markdown lint and rendered checks;
all local links exist and no italic spans occur.
The complete rendered rollout plan has been read.
The private renderer now registers the new plan explicitly.

Q7 is posted through the question tool for confirmation of the linked concrete rollout plan.
The current interview on 2026-10-18 settled the operating model:
CI is advisory-only;
the baseline is Developer with overage disabled on every product;
native execution runs only in one bounded weekly matrix that covers every admitted native
capability category;
untrusted pull-request source stays on public GitHub;
Node validation defaults to GitHub-hosted runners;
cache writes come only from `main` and explicit dispatches;
the default-branch advisory run uses sliding-window rate limiting and a fast static,
build and unit subset,
with the complete profile weekly;
one independent whole-report review is recorded before any paid profile;
each optional product and cache format receives an explicit disposition;
and success targets are relative to the public GitHub baseline.
The blocking-enforcement answer in Q1 and the standing-overage authority in Q4 are superseded
by the advisory-only and no-overage answers above;
Q1's baseline discipline remains.
Q7,
the confirmation of the revised rollout plan,
is the only open confirmation question.
The revised plan is published at `d94e50bec4` after the closure controls.
The review record is at
`doc/planning/depot-expanded-ci-evaluation-independent-review.md`,
and the registration,
duplicate,
baseline and paused-session evidence is under `doc/audit/`.
Q7 was answered on 2026-10-18:
the user confirmed the revised plan at `d94e50bec4` and permitted
implementation.
The adoption proposal was accepted and moved to `doc/decision/depot-ci-adoption.md`;
the provider and cache supersession notes are in effect.
Implementation starts with the plan's work item 1;
no further spend or recovery authorization is required.

The posted second round covers independent decisions:

- Q4 is accepted:
  standing authority for bounded recurring overage under the existing USD 200 combined planning ceiling,
  without approval of each paid batch.
  Use standing authority for unattended CI,
  using conservative published charges and the repository-owned allocation ledger.
  This does not change subscriptions,
  authorize account-wide inspection or guarantee an aggregate invoice.
- Q5 is accepted:
  one bounded recovery attempt for a classified provider/startup failure,
  without requiring an individual recovery request.
  Reserve exposure for both attempts and retain the first failure.
  Source failures,
  discovery failures and artifact-integrity failures cannot become successful checks by being ignored.
- Q6 is accepted:
  retain the correction only in CI documents and leave the existing general `AUT` rule unchanged,
  consistent with the user's preference for narrowly scoped documentation changes.
  The rejected concrete alternative proposed under `AGENTS.md`'s correction rule was:
  "Execute requested work without expanding scope.
  Preserve accepted systems and workload allocations unless the user explicitly authorizes replacing them."
  No `AGENTS.md` edit has been applied.

The remaining technical acceptance criteria are engineering work,
not requests for the user to choose runner labels,
cache formats or retry timers.
Known-failure baselines must identify actual reproducible cases rather than waive whole failing tasks.
They must be qualified on the implementation candidate's immutable source;
historical failures cannot automatically become current waivers.
Native promotion qualifies the features each selected workflow needs;
workflows requiring unqualified paused-session attachment remain on GitHub until that boundary qualifies.
No numerical GitHub/Depot allocation or eventual Depot-majority target is inferred from the word "heavy".

## Final local capture and publication checkpoint on 2026-10-18

The corrected capture is terminal and its owned container is absent.
Its real stdout/stderr positive control passes.
The same twelve unchanged authored files produce 179 product source-case passes
and nine separate built-harness outer control passes on the already-admitted derived fixture topology.
All labels join actual per-case debug `PASS` records;
24 exact streams contain 9990 stdout bytes and 92794 stderr bytes.
No logger-internal diagnostic occurs.
Eight deliberate inner failure diagnostics belong to the passing harness controls.
All three private export identities match again,
without publishing their payloads,
filenames or hashes.
The source receipt is `doc/audit/depot-local-node-capture-correction-2026-10-18.evidence.json`.
Earlier empty successful-stderr fields remain unobserved placeholders.
The original four fixture-path failures and the no-with assertion gap remain qualified.
No new provider allocation or production source fix occurred.

The first capture invocation was rejected before runtime because its admission commit had not completed.
The still-prepared admission ran only after `b8c2dc9dcc` completed;
there was no duplicate container allocation.
The final capture archive,
its admission and prior evidence pass their byte/hash joins.
Scoped JSON formatting and Markdown checks pass with no remaining diagnostic.
The six approved report sections render with no missing local links or italic spans.
The completed local source-test worktree is removed normally after its ignored-output dry run
and root Git-dir entry checks.
All 162 generated output roots are retained privately;
the compiled harness identity and all three original/copy private fixture identities still match.
Canonical worktree absence is checked with the registered main worktree as a positive control
(`doc/audit/depot-local-source-worktree-retirement-2026-10-18.evidence.json`).
The remaining action is exact-tip isolated publication,
then the first interview frontier on failure enforcement,
workflow cadence and the order of Linux orchestration adoption.
The service quota still prevents a fresh independent whole-report sufficiency review.

## Local source-consumer checkpoint on 2026-10-18

Cloud work remains terminal;
no additional provider allocation is needed for this checkpoint.
The frozen Figma parser and converter now pass all 24 authored leaf labels,
including actual consumption of the three personal exports in a network-disabled container.
Payloads,
filenames and hashes remain private.
The receipt is `doc/audit/depot-local-figma-authored-consumer-2026-10-18.evidence.json`.
The remaining nine authored source-test files produced 151 passing leaf labels and four failures.
Eight files pass;
the import-attributes integration file fails during Rolldown entry resolution.
It requests `src/fixtures`,
while the frozen source contains `src/fixture`.
The fresh owning Mise Node-harness build passes,
and the existing built-export harness self-test passes its nine outer cases,
including assertions about deliberate failures,
timeouts and repeat termination.
Those controls are separate from product coverage.
Exact source identities and 22 process-output bodies are sealed in
`doc/audit/depot-remaining-local-unit-2026-10-18.evidence.json` and its source archive.
Source-import successes do not establish compiled package assertion coverage.

The committed three-leg local counterfactual admission is terminal:
`doc/audit/depot-local-node-counterfactual-admission-2026-10-18.evidence.json`.
The writable logger directory removes its diagnostic but retains all four entry failures.
Adding only `MONOCHROMATIC_VERBOSE=true` produces ten debug lifecycle lines in failed-child stderr.
Adding only the throwaway `fixtures -> fixture` alias then makes all four authored cases pass.
Each network-disabled container has two CPUs,
2 GiB RAM and a 150-second limit.
Keep the entry-resolution and secondary logger diagnostics as separate incidents.
The source-owned no-with case still does not assert absence of transformation on a plain import.
Successful-child stderr was discarded by the local wrapper;
its empty fields are placeholders,
not observed empty streams.
The correction is explicit in `doc/audit/depot-local-node-counterfactual-2026-10-18.evidence.json`.
All 223 semantic owner projections and 1818 original log-hash joins are now published in
`doc/audit/depot-semantic-package-publication-2026-10-18.evidence.json`.
The next action is scoped formatting and rendered checks,
then publication through the existing isolated merge worktree.
The agent service limit still prevents a fresh independent whole-report review.
The interview has not started.

## Terminal allocation closure on 2026-10-18

Every newly admitted job is terminal.
The final follow-on has 49 rows and 53 retained source bodies,
with 49 timestamp-pointer joins and exact BigInt arithmetic passing
(`doc/audit/depot-owned-allocation-final-source-verification-2026-10-18.evidence.json`).
Its nominal provider-window proxy is USD 1.685366666667 before allowances;
the separate BuildKit request-window proxy is USD 0.015897333333.
The frozen 223-row baseline and historical reservations remain separate.
Both SSH attempts are consumed;
paused-session attachment and final workflow readback remain unqualified.
Verified installation and source-inspected status behavior are recorded in
`doc/troubleshooting/depot-native-ssh-paused-session.md`.
No new independent whole-report review has run after the agent service limit.
Concrete Root source/hash/pointer/arithmetic checks do not substitute for that review.
The source/runtime join is now sealed in
`doc/audit/depot-source-contract-runtime-join-2026-10-18.evidence.json`.
It revalidates all 5878 frozen script paths and 219 README blob identities.
The corrected task classifier includes generic `test` and additional `test:*` names for 41 owners.
Of 208 active roots,
55 have no such named task;
this includes fixtures and configuration and is not a count of untested products.
Six roots contain 11 authored unit files without owning test execution in the existing 1818-command projection:
`desktop-daemon/hall-monitor`,
`dev-script/page-weight`,
`figma/kiwi`,
`figma/to-penpot`,
`module/hyperscript` and `rolldown-plugin/import-attributes`,
all under `package/`.
The next action is the local frozen Figma consumer before other new execution.
All three personal exports required by the authored converter tests exist at their exact source paths;
their private byte/hash probe is `personal-figma-fixture-path-probes.json` in Root's controller.
Use a disposable 2-CPU/2-GiB container with network disabled during fixture consumption.
Keep personal payloads local and preserve authored assertions unchanged.
The exact Node 26.12.0 image is cached;
the old coverage worktree has no `node_modules`.
Prepare isolated frozen dependencies and a source-bound admission before running the existing files.
The JavaScript/TypeScript source candidate parser does not establish every product feature contract.
Publication must use an isolated merge worktree because unrelated concurrent IDE changes block merging
`origin/main` in the shared checkout.

## Current execution checkpoint on 2026-10-18

Research remains active and the interview has not started.
The three active research agents reached their service usage limit on continuation.
Root now owns their durable queues and existing controller state;
no completed or uncertain allocation is redispatched because an agent stopped.
The seven-primary material controller reached terminal state:
six workflows succeed and design fails before Node startup with
`/research-node/node: error while loading shared libraries: libatomic.so.1: cannot open shared object file`.
Root collected all seven original artifact ZIPs,
verified their digests,
CRC and receipt/log hashes,
and retains the detailed contexts privately in `material-owner-collected-contexts.json`.
Workflow success does not yet supply parsed Nextest case counts.
Remaining work is explicit:
parse and seal the material owner identities,
diagnose and apply an admitted harness-only design prerequisite correction,
review/admit the matched GNU source before its single primary dispatch,
review/admit the prepared SSH correction before the remaining attempt,
join terminal intervals into a new allocation-ledger snapshot,
finish independent report sufficiency and renderer/publication checks.
The screenshot-publication boundary is already closed by
`doc/audit/depot-complete-coverage-desktop-receipts.json.gz`:
22756 bytes,
SHA-256 `8b9be6f1b2cfb29540226d0a8c4b520d867d8d6a24b0eddda516198b26b26885`.
Its scan covers 188 tracked audit files and 251 decoded containers,
finds zero original desktop ZIP/captured-PNG payloads and 836 hash references,
and detects both actual private ZIPs and all 110 PNG occurrences in the positive control.
No image inspection/masking/publication or new desktop launch is claimed.
The matched GNU and design continuations are now both terminal,
their watcher session `30378` is reaped at zero,
and no newly dispatched cloud job remains active.
GNU run `39165688015`,
job `117788040975`,
runs 469 seconds and executes the same 1494 identities as the public baseline:
1493 pass,
the unreadable-session-path/logging case fails,
and the watch case changes from public FAIL to Depot PASS.
One repeated final-failure status line is deduplicated;
one summary-only skip remains separate.
No provider cause or failure rate follows from this pair.
Design correction `39166144807`,
job `117789377100`,
runs 162 seconds on the same public shape,
passes all thirteen authored programs and verifies 997 selected inputs totaling 269996175 materialized bytes.
Both artifacts pass SHA-256,
CRC and member/receipt joins.
Their 42 text bodies and source-bound outcomes are sealed in
`doc/audit/depot-gnu-design-terminal-2026-10-18.evidence.json` and its source archive.
The primary archive remains unchanged.
Native CLI status omits timing fields;
source inspection identifies `CIService/GetWorkflow` as the actual attempt-timestamp consumer.
Exact owned workflow queries return first SSH attempt `18:37:37` to `18:39:53 UTC` (136 seconds)
and correction `20:58:40` to `21:00:58 UTC` (138 seconds).
At USD 0.006/minute their combined nominal attempt-window proxy is USD 0.0274,
separate from the historical USD 0.036 reservation and any invoice.
Raw bodies remain private pending the final selected-field/hash ledger projection.
The primary material outcome is now sealed in
`doc/audit/depot-material-owner-primary-2026-10-18.evidence.json`
with 126 original-artifact-derived text bodies in its hash-bound source archive.
Actual compiled inventories match executed identities:
IDE full 783 passes plus one ignored,
IDE library subset 316 passes plus the same ignored case,
terminal 16 passes,
GTK 13 passes,
sticky eight passes plus one ignored,
Qt one pass.
The 1137 observations contain 821 unique binary/case identities;
the separate IDE subset overlaps full and is not additive coverage.
IDE's real runtime producer records 2500 staged files.
The actual owner assertions remain unchanged.
Design's 13 programs were enumerated,
but zero executed before the locked Node loader failure.
The single-library correction is source `73edb149b0a051e8047e32b447d93af335f537a8`,
branch `research/depot-design-libatomic-20261018`.
Its actual ten bootstrap chunks reproduce the tracked Mise/config bytes;
reversing the `libatomic` package addition restores the complete original workflow object.
The original public profile and source assertions stay unchanged.
The initial preflight incorrectly ran the final receipt assertion against an absent fixture,
then omitted append chunks;
both preflight defects were corrected before dispatch.
The source branch is directly published,
and its separate correction admission is
`doc/audit/depot-design-libatomic-correction-admission-2026-10-18.evidence.json`.
Matched GNU run `39165688015` on exact source
`28f3167d8f0e731d13378d072dd5f23a7e392554` is active.
The actual branch is `research/depot-complete-coverage-20261018`,
not the descriptive unpushed branch label in the original proposal.
Parent admission independently compares the complete decoded job and serialized function,
retaining inherited inactive Android/desktop/material harness paths without dispatching them.
Native SSH correction `sn950h8xcj`,
job `lxkdfrx3dh`,
attempt `bv33fqpvnp`,
source `553bd7a8d2e79c1b907ac4181a28300a98285f9f`,
is terminal cancelled and its controller is reaped at exit one.
Its bounded tmate preinstall succeeds with the admitted archive/ELF hashes and version 2.4.0.
Direct SSH and CLI RPC again return matching proofs with client exits 255 and zero;
all recorded owned client identities are absent.
No paused transport or final readback is qualified.
Both admitted attempts are consumed;
no third attempt is authorized by this campaign.
The root main publisher has a real non-fast-forward rejection,
not a wait-notice timeout.
A normal merge refuses to overwrite concurrent edits in
`doc/handover/slint-ide-0x.md` and `package/desktop-app/ide/README.md`.
Those edits remain untouched.
Publish the local scoped docs through an isolated merge worktree,
preserving both histories and verifying direct remote ancestry;
do not stash,
reset or force-push the shared main worktree.
All parent graph cloud jobs are terminal,
all parent graph watchers and collectors are reaped,
and no parent cloud allocation remains active.
Corrected native run `fnvwqfg0bb` on `72af18d5b130dabef7ff5b656ffdc1b58df4fcde`
produced both complete 222-command logs;
actual attempt intervals are 390 and 388 seconds.
Both repetitions pass browser prerequisites,
Chromium and Firefox;
WebKit reaches the OPFS assertion and root E2E fails discovery before running cases.
Original and corrected native results are sealed in
`doc/audit/depot-expanded-graph-native-correction-2026-10-18.evidence.json` at `f53c920cb2`.
Repeated GitHub results are sealed in
`doc/audit/depot-expanded-graph-github-repetition-2026-10-18.evidence.json` at `05d02428a0`.
All eight originals independently pass ZIP scope checks and 1720 retained command-log byte/hash joins.
The source-failure classification is
`doc/planning/depot-expanded-graph-independent-review.md` through `31c4622c6a`.
Two original WebKit streams mix stderr notices with stdout JSON;
their explicitly derived cleaned views classify assertions without qualifying an original-JSON consumer.
Corrected-native Rust parity at `e0790e738d` retains 652 emitted PASS statuses,
with the same 89 Truepeak and 237 forbidden-strings identities per sample.
Changed-case/status controls reject equality;
skip identities remain unenumerated.

Completed-phase mean ratios are 2.86825 for shared builds and 2.21112 for Rust on Depot GHA.
Corrected native ratios are 3.37416 and 2.20532 respectively.
No complete-CI multiplier is claimed.
Critical-path receipt
`doc/audit/depot-expanded-graph-critical-path-2026-10-18.evidence.json`
is committed at `df360379b0` and binds the original public source archives.
Shared builds followed by a Node branch form all six observed longest paths;
Rust is not on those paths.
The cost note replaces its stale claim that no aggregate ledger exists.
The frozen 223-row baseline stays unchanged;
the independently frozen 30-row follow-on snapshot records a USD 0.6041 known-provider-window proxy,
including desktop's exact 263-second attempt at USD 0.1052.
The previous 279-second run envelope and USD 0.1116 estimate are superseded,
with their history retained in the allocation ledger.
The parent's exact nominal Depot graph subtotal is USD 0.487.
These figures are not invoices or evidence of VM/builder deallocation.

Coverage owns public OS recovery `39144499822`,
then the admitted matched Depot Mac/Windows controls,
semantic assertion/generator reconciliation and the metadata-only screenshot-publication receipt.
Rust owns exact desktop attempt boundaries,
the final follow-on ledger and deletion/absence checks for the three owned prepared native images.
Platform completed the narrow report capability-allocation completeness pass;
the final whole-report independent sufficiency review follows factual closure.
The parent owns its graph source-archive checks,
rendered report/handover checks,
independent cost-ledger reconciliation,
final capability/OS/semantic integration and publication ancestry checks.
Read the grilling skill and begin the interview only after those research queues close.

Coverage's semantic reconciliation corrects the original package census:
227 candidates included four documentation blobs;
the frozen tree has 223 package directories,
208 active,
13 paused and two archived.
Task and execution counts are unchanged.
The report records this correction;
the parent independently checks candidate types against the frozen tree before final acceptance.
The parent's three new graph receipts pass JSON formatting,
all 44 retained original source-body byte/hash joins and targeted Markdown lint.
All fifteen parent Markdown documents pass rendering/link checks;
the changed handover checkpoint and cost note have been read as actual rendered HTML.
The Bash builtin rejected the parent's invented `kill --signal` option;
the actual `help kill` contract supports `-s`,
which terminated the exact owned overbroad search and produced a reaped status of 143.
Platform owns the separate source-backed troubleshooting note;
this is an invocation error,
not an upstream defect or a Depot result.

The independent package census now passes at `16:59:07 UTC`:
`doc/audit/depot-package-directory-census-independent-2026-10-18.evidence.json`
joins all third-level package trees and the four excluded blob identities.
Prepared commit `ed8cf6e5b4` records that independent evidence.
The report's capability allocation now explicitly covers native/GHA scheduling,
Tests/report identities,
snapshots/OCI/Disk/SDK format separation,
BuildKit/registry lifecycle,
cache/PR/network/Dependabot trust and optional-product dispositions.
Its scoped publisher is reaped at zero and the landed commit is `cce76fb02e`.

Coverage's semantic join identifies material authored owners needing wider existing-evidence joins
before any new execution:
IDE,
file-manager variants,
terminal and design.
Static test attributes are not executed-case counts.
Coverage is preparing exact finite profiles for actual owner tests and real prerequisites,
including the IDE's pinned grammar producer and terminal font/Zig dependencies.
The concrete proposal is now reviewed and admitted through
`doc/audit/depot-material-owner-parent-admission-2026-10-18.evidence.json`.
The admission publisher is reaped at zero at `c297a446bc`.
Its bound proposal SHA-256 is `8ebd2766f5e64f13d180ea6027853a97efa969748ba4ec04e9f0d6795becb82e`.
The parent independently joins 24 frozen blob/hash/byte/object inputs across 18 distinct paths.
Seven primary jobs reserve USD 1.08 nominal compute;
two named conditional corrections reserve USD 2.88 and unpriced contingency is USD 1.04.
This totals a USD 5 nominal guard,
not an invoice guarantee.
The parent completed concrete rollout profiles for static checks,
Node/Rust tests,
browsers,
packaged artifacts,
OS/ARM and VM/desktop/Android consumers.
Each profile must name actual owner commands,
producer prerequisites,
retained evidence and source-failure disposition.
Their scoped publisher is reaped at zero at `884f42a9af`;
the new profiles were read as rendered HTML and targeted Markdown passes.
The admitted material-owner campaign retains
at most two concurrent cloud jobs,
with default Cargo/Nextest concurrency and existing no emulator/desktop relaunch boundary.
The raw-Git graph does not prove materialized font/media payload equivalence.
Coverage must name materialized LFS source hashes wherever those consumers need them.
Commit and preflight each executable harness before allocation,
including native argv/task checks and all cloud payload readers.
Coordinate the two slots across GNU,
material and Platform SSH jobs rather than treating a per-workflow matrix cap as a global reservation.
The widened legacy search covers 169 inventoried files,
168 searched files,
94 CRC-qualified ZIPs and 3558 members,
with 72 exact-alias controls and real music-player/Truepeak positives.
It finds no qualifying selected IDE/terminal/file-manager outcome in that retained corpus.
Design's authored owners are Node/ImageMagick fixtures,
not an authored Playwright suite;
only the real owners are admitted.
This research is not deferred to production acceptance.

Platform additionally source-checks a finite native debug-access consumer,
before any allocation,
with a proposed two-CPU three-minute job and USD 0.036 maximum including correction.
The installed `depot ci ssh` uses a PTY RPC;
its `--info --output json` alone is not remote-command execution.
The documented direct OpenSSH bridge and paused-step tmate injection are distinct transports.
The parent must review exact noninteractive identity and cleanup controls before dispatch.
Rust replaced desktop's 279-second envelope with exact workflow attempt evidence:
263 seconds,
status failed,
USD 0.1052 nominal,
giving a 30-row follow-on subtotal USD 0.6041.
The archive had retained full workflow responses;
Rust is inspecting all actual fields and replacing unnecessary public metadata with explicit selected projections,
while retaining private originals/hashes and correcting source pointers.
Current archive replacement does not erase Git history.
The parent independently reconciles the sanitized final ledger,
including subsequent OS/owner/debug attempts,
before publishing a final combined subtotal.

The sanitized follow-on snapshot independently passes at `17:14:07 UTC`:
30 exact timestamp-pointer/rate joins,
35 retained source-body hashes,
zero frozen-baseline duplicates and exact BigInt arithmetic.
Frozen coordinator receipt and nested original snapshot archive are
`doc/audit/depot-owned-allocation-follow-on-independent-2026-10-18.evidence.json`
and its sources archive,
committed at `846d2c1301`.
USD 0.6041 remains a nominal provider-window proxy;
USD 0.015897333333 remains a separate BuildKit request proxy.
The public metadata projection correction found no nonpublic actor/email,
credential,
signed URL or connection material.
Unnecessary public actor/email fields in the full GitHub REST body were replaced by explicit selected metadata.
The selected workflow responses and private-original hash boundaries remain distinct;
no Git history was rewritten.

The conditional waiting model is sealed at `6553733993`:
`doc/audit/depot-expanded-graph-conditional-waiting-value-2026-10-18.evidence.json`.
The original dated Bank of Canada source was re-fetched and retained.
CAD 8.975/hour divided by CAD 1.4154/USD gives USD 6.340963685/hour.
Completed shared-build phase differences have conditional full-blocking values USD 0.24117 for Depot GHA
and USD 0.26053 for corrected native.
Actual human blocking is unmeasured,
no overlapping Rust time is added and no whole-CI benefit or ROI is claimed.
Both new cost receipts pass hashes/bytes for six frozen wrapper bodies and 35 nested retained bodies,
JSON formatting and targeted Markdown lint.
All fifteen parent documents render with valid relative links and no unintended emphasis.
The new report allocation sections,
failure-classification paragraphs,
native parity,
follow-on figures and conditional cost note were read as actual HTML.
Direct remote main `3cddf3617afbde0c13fc532db20ae05a6c39b9f9`
contains all eight checked parent landed commits through `296ecf41c8` at `17:17:04 UTC`.
The later cost commits need a subsequent remote ancestry check.

Public OS recovery `39144499822` is terminal.
Actual Windows x64 and ARM each run 1494 Nextest cases,
1493 pass and one unreadable-session-path failure,
plus one skip.
Mac runs 1495 cases,
1468 pass and 27 fail,
plus one skip.
These are per-binary runtime outcomes,
not unique authored-assertion totals.
The GNU adapter duplicates the owner's `--no-fail-fast` and reaches zero test bodies;
Coverage retains that harness error and owns one GNU-only correction after the matched OS run.
Matched Depot OS `39148580870` is terminal on `aa501f4a5c0e74a0a993b668c98b00fe3be2da19`
under `doc/audit/depot-current-host-matched-admission-2026-10-18.evidence.json`,
two sequential 20-minute caps with 17-minute inner bounds,
USD 1.92 reserved under USD 2.
Mac job `117737397487` runs from `17:13:37` to `17:19:36 UTC`,
359 seconds;
Windows job `117737397295` runs from `17:19:37` to `17:30:51 UTC`,
674 seconds.
The combined nominal job-window proxy is USD 0.6584,
separate from its USD 1.92 reservation and invoice cost.
Its observed case counts match the public count boundaries;
Coverage seals exact original artifact/case/host evidence and actual OS/machine differences before final integration.
GNU-only correction `39152147229` is terminal failed,
on harness `b76aab1964bfe9ac21aec3e9c5d24de443ec4992`
and dedicated branch `research/depot-gnu-owner-correction-20261018`.
Its single standard public Windows job has a 30-minute cap and 27-minute inner bound,
with zero nominal public compute and finite watcher `11329` owning its 40-minute campaign guard.
Job `117747819418` ran from 18:00:26 to 18:10:52 UTC (626 seconds).
Coverage has sealed the actual GNU bodies in
`doc/audit/depot-current-host-gnu-owner-2026-10-18.evidence.json.gz`:
2046665 bytes,
SHA-256 `19e672e8a099c63b4a070e4619f7c6bea8b844629d80a91cb5f8f88b4c7c2372`.
Its 17 derived normalized text members retain original hashes;
original ZIP,
binary and media payloads remain private.
The collected GNU outcomes report 1494 distinct binary/case identities:
1492 pass and two fail,
with one summary-only skip.
The unreadable-session-path/logging failure overlaps MSVC;
`watch::tests::a_burst_of_changes_requests_one_rescan` is an additional GNU assertion failure.
Its cause and repeat failure rate remain unmeasured.
The original calls and diagnostics retain their separate source identities.
The parent also requests a concrete matched Depot-Windows GNU proposal,
preserving owner source/argv/tool setup and adding actual OS/CPU/memory metadata probes.
Its proposed scope is one four-CPU job with a 20-minute cap (USD 0.32 nominal),
plus at most one same-profile harness-only correction (USD 0.64 maximum).
Exact-source parent admission is still required before allocation.
This additional target comparison does not replace the admitted seven-owner material campaign.
No coverage cloud job remains active.
The decoded bootstrap and duplicate-versus-single-argument controls pass before dispatch.
Coverage sequences the material campaign after this GNU attempt is terminal.
Platform's pending SSH source includes a third native CLI PTY-RPC consumer,
separate from direct OpenSSH and pinned tmate.
The replacement seal is `3adb2516b1`,
fixture `b6a60493c16087043a7043f50cd31d8aebf4bdee`.
Parent source admission is recorded in
`doc/audit/depot-native-ssh-parent-admission-2026-10-18.evidence.json`.
It independently joins current and retained source bodies,
fixture Git blobs,
the direct remote branch and controller/helper/bridge/manifest hashes.
The parent admission publisher is reaped at zero at `dc62a35802`,
and the explicit ready message was sent after its JSON/Markdown checks passed.
Native SSH run `sqrz4sml30` used job `q74vcmxkk1`,
attempt `fhxx4s51rf` and sandbox `g64shw2rl3jzrxsmq92x`.
Dispatch started at `18:37:23.441 UTC`;
the 150-second watchdog cancelled the run.
Fresh terminal status is cancelled,
and controller `68688` is reaped at exit one at `18:40:00.760 UTC`.
Direct OpenSSH and native CLI RPC both emitted VM challenge proofs;
their client exits were 255 and zero respectively.
Both recorded owned client trees are reaped and absent.
No paused-tmate leg was selected and no final workflow resume/file consumer appeared.
This is partial transport execution,
not completed pause/resume qualification.
Read-only collection matches both source/boot/challenge responses.
Direct OpenSSH runs as UID 0/GID 0 in `/root`;
native CLI RPC runs as UID 1001/GID 117 in `/home/runner`.
Neither inherits the synthetic workflow environment.
All recorded owned identities and the askpass socket are freshly absent.
The final 276 log events show the paused action entering `apt-get update`
and emitting only the 237-byte `/etc/apt/apt-mirrors.txt` Mirrorlist line before cancellation.
No tmate target was logged.
Dependency installation is the last recorded phase;
the last APT line does not identify the blocking instruction or cause.
It is not evidence that tmate transport fails or is unsupported.
Pinned action source `src/index.js:82` through `src/index.js:110`
shows `install-dependencies: false` skips both package installation and static-binary download,
then uses `tmate` from `PATH`.
Platform prepares a source-bound same-binary correction without unconditional apt update;
Root must review changed identities before the remaining attempt.
No second dispatch is admitted automatically.
The single reserved slot is released,
so Coverage may use both global slots for admitted work.
The three-minute server job and USD 0.036 two-attempt guard remain unchanged.
The 210-second local point starts cleanup;
there is no independent whole-controller hard backstop.
Runtime outcomes use a successor receipt so the admitted proposal/source seal stays immutable.
Platform durably queues the read-only GitHub check-runs/commit-status consumer comparison in
`doc/planning/depot-native-ssh-runtime-research.md` at `0567953a15`.
The positive control is the owned GitHub graph source/run;
the native probe uses an exact owned source with actor/output bodies kept private.
Its exact `b6a60493c16087043a7043f50cd31d8aebf4bdee` SHA has GitHub check `117756320157`,
`native-ssh-acceptance / ssh-proof`,
completed/cancelled under app `219785`,
`depot-code-access` (Depot Code Access).
The known GitHub graph source has six checks:
four GitHub Actions checks for run `39140975786` and two Depot Code Access native-graph checks.
Both exact SHAs have zero legacy commit statuses.
The check details identify exact owned job `q74vcmxkk1`,
and fresh native status binds it to run `sqrz4sml30` and attempt `fhxx4s51rf`.
Selected URL/external-ID hashes and match booleans are sealed in
`doc/audit/depot-native-ssh-runtime-2026-10-18.evidence.json`;
the installed app name alone does not mean the optional Depot Code product was enabled.
No app/account settings were inspected or changed.
Coverage committed and pushed the seven-owner material harness at
`015ee75186e697a05160913dbd8006eceb90b2e5` before its actual bootstrap/Mise/Node preflight.
It preserves the admitted profile/prerequisite/argv boundaries and adds actual OS/CPU/memory probes.
The final executable admission is
`doc/audit/depot-material-owner-executable-admission-2026-10-18.evidence.json`.
It binds pushed harness `e270285361b526793ff1db3af9f8224630c28e70`
and frozen application `e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`,
after actual owner-variable,
Cargo-selection,
altered-PNG/LFS-pointer rejection and publication controls pass.
Finite controller session `55444` has started.
IDE full run `39156168838` uses `depot-ubuntu-24.04-4`,
four requested CPUs/documented 16 GB and a 30-minute/USD 0.36 maximum.
GTK release run `39156185253` uses public `ubuntu-24.04`,
30 minutes and nominal compute USD 0.
Those jobs own both global slots.
The private journal is
`/var/home/user/temp/agent/depot-complete-coverage-20261018/material-owner-campaign-controller.json`.
Its initial reservation is USD 1.08 primary,
USD 2.88 conditional and USD 1.04 unpriced headroom,
with fresh terminal predecessor rows required before subsequent dispatches.
Platform prepares the SSH correction read-only;
allocation requires Root coordination of a released/reserved slot.
Rust checked the actual locked Mise 2026.10.9 affected-task selector,
separately from the installed 2026.10.0 failure.
Its actual locked-version metadata probe now confirms the identical workspace cycle,
after changed-producer/development-consumer,
unchanged-revision empty-selection and synthetic-cycle controls.
All 419 frozen metadata hashes stay unchanged in an offline two-CPU,
two-GiB,
256-PID container.
No owner task,
credential,
installation or upgrade is involved.
Rust sealed exact binary/source/arguments and the existing troubleshooting update at `a9541b020c`:
`doc/audit/depot-mise-locked-affected-2026-10-18.evidence.json`,
SHA-256 `37bd8779e0046a9eab9c25d22f6c4f293f0c059083a3c5d502bef486fa4b5841`.
Its 16 source-body,
seven outcome and four deciding source-line joins pass,
with targeted Markdown and actual rendered-section checks.

Kotlin's claimed unexecuted six JVM methods are retracted and positively resolved.
The missing semantic projection alone did not prove missing execution.
Coverage and Rust match the exact frozen Kotlin subtree `fa0167ad3af17896374a22b40ede76e9e9f51250`
and all six `dev.monochromatic.detekt.RequireKDocTest` method tuples in four original warm/off JUnit reports.
All six pass in each sample.
The experiment executes the authored Gradle `test` task via Mise with cache-specific flags,
not the literal package Mise wrapper.
The separate `r328mp420m` emitted-JAR consumer is not the six-case basis.
No new Kotlin run is needed.
Rust sealed the source/task/JUnit projection at `c75fc4790b`:
`doc/audit/depot-kotlin-gradle-owner-join-2026-10-18.evidence.json`,
SHA-256 `a497e3f495e1818af77eb9a2cba8ab45cbab71b5b59476364fe9cb1993b36614`.
Four original ZIP CRCs and 102 member hashes pass;
Coverage joins that projection semantically.
Coverage also checks actual OS Nextest binary/package identities and older receipts for the other proposed owners.
The new owner campaign is admitted but not yet allocated;
its exact executable harness and payload preflight remain required before dispatch.
Platform's Bash diagnosis is complete at `3cddf3617a`:
`doc/troubleshooting/bash-builtin-kill-signal-option.md`,
exact patchlevel-nine source and safe signal-zero controls.
Its pending real-debug proposal remains separate.
Rust's documented short-lived OCI challenge exchange now authenticates exact prepared-image reads.
All three owned tags and their immutable digests return `404 MANIFEST_UNKNOWN`.
No deletion,
persistent credential or cloud allocation occurred.
Rust seals creator/host/repository/digest joins,
selected pull scope and an invalid-Bearer response control before final absence acceptance.
These reads do not establish expiration as the cause or physical storage reclamation.
The cloud cleanup fallback is unnecessary.
Rust sealed current absence in `49e5c84d1b`:
`doc/audit/depot-native-snapshot-absence-2026-10-18.evidence.json`,
SHA-256 `967abe64d26450f9754b00c387c742b1ea744f6ac8e58d7be605b240c1499956`.
The same-host/path invalid-Bearer control returns 401 while the valid selected pull token returns 404.
Its 20-body source archive retains exact creator/host/repository/tag/digest joins.
The follow-on now has 37 rows and 41 retained bodies after the public recovery and managed-host joins.
Its known nominal provider-window proxy is USD 1.2625,
with Depot GitHub USD 0.8381 and native CI USD 0.4244;
the parent graph subtotal remains USD 0.487.
Rust's successor verification is sealed at `92839f0386`:
`doc/audit/depot-owned-allocation-follow-on-hosts-verification-2026-10-18.evidence.json`.
The ledger SHA-256 is `9b536e96fabfe522fa0921dddfa7b7da3c14730c5c9625c99b7286850425cb17`;
the 41-body archive SHA-256 is `872651779e81a7ae6627c10c7392151fbe435cb97be456a8e409428ee6ca1b6d`.
Host comparison controls at `21f3133491` establish equality for 1495 Mac and 1494 Windows recorded cases,
with changed identity,
missing identity and changed status detected by the same comparator.
The initial host seal retains its derived normalized views at `d29c7887...`;
the derived-view comparison successor is
`doc/audit/depot-current-host-owner-comparison-2026-10-18.evidence.json.gz`,
SHA-256 `c11ce61387c8675f99c303cad6093d444a13bc7c42adf24eb77f2bdd1f1677b7`.
Depot Mac requested/catalog eight CPUs and 24 GB differs from observed available CPUs seven and 17 GiB.
Its actual OS build and CPU model remain unmeasured;
do not substitute a fresh VM or catalog M2 as the missing original observation.
The independent frozen 30-row review remains a distinct immutable snapshot;
the final larger ledger still needs the parent's new independent review.
The refreshed Cargo bootstrap troubleshooting section renders and passes its targeted check.
Its nine embedded-upstream-Rust warnings exactly match the pre-edit baseline;
an injected Rust-closure control raises the count to twelve.
The first stdout-only collector missed stderr diagnostics and is rejected,
as is the nonmoving heading control.
No broader token or account enumeration is authorized.
Coordinate at most two active cloud jobs across owners.
Root removed its completed graph and MVM worktrees after clean tracked/untracked status,
ignored-output dry runs,
root Git-name/ignore checks and confirmation that neither active owner needs their paths.
The three ignored-output directories were moved into the private Root controller's
`retained-completed-worktree-outputs` directory before ordinary non-forced removal.
No retained evidence or shared Git/LFS objects were deleted.
The physical roots resolve through `/var/mnt/encrypted/temp/agent`;
the first literal-path assertion was rejected and the corrected realpath comparison passes.
The rejected parent invocation `mise run --file ...` emitted `no task //:--file found`.
Running the existing task from its owning controller directory passes five receipt hashes,
44 source-body joins and targeted Markdown lint.

## Recorded setup and recovery history on 2026-10-18 at 16:33 UTC

Research remains active;
do not start the interview.
GitHub graph run `39135886118` is terminal failed:
all four jobs passed checkout and resource recording,
then failed selected-tool setup because the parent omitted configured `cargo-binstall`.
The exact mise diagnostic is
`requires configured install dependency 'cargo-binstall@latest', but its selected version is not installed`.
No graph workload command ran;
the planned 222 commands provide no execution or speed credit.
The parent retained terminal job metadata,
artifact identities and private failed logs,
then followed the existing
[Cargo installer troubleshooting entry](../troubleshooting/mise-cargo-backend-needs-rust-installed.md).
Correction `fdca95cc72` installs locked Node,
pnpm,
Rust and `cargo-binstall` before locked Nextest.
The Rust researcher independently preflights the actual final driver and setup locally before repeat dispatch.
The exact staged tool install passed in fresh isolated tool state under two CPU and two GiB limits.
Independent driver controls also found a staged-source detection gap and surviving detached commands after external
`SIGTERM`.
Correction `234ccc92dabb81e998e0a0bb5e838713924dd83f`
compares tracked state to `HEAD` and adds signal-driven cancellation,
bounded group termination and final cancellation receipts.
Independent rechecks of that exact correction pass all twelve controls,
including actual staged/unstaged rejection and resistant process cleanup after both signals.
The sealed preflight receipt is
`doc/audit/depot-expanded-graph-preflight-2026-10-18.evidence.json`,
with original-source/log joins and publication checks committed at `2ca0bda544`.
Corrected GitHub run `39140975786` was dispatched once at `15:32:53.075 UTC` on that exact source;
its finite terminal watcher is reaped with terminal failure at `16:24:13 UTC`.
Stub owner commands provide no application execution credit.
The browser branch already contains `prepare:matched-image`;
an incomplete-plan reading that alleged its omission was retracted.

Native automatic BuildKit authentication run `dzsf27t0xx` is terminal failed.
The CLI connected to the AMD64 builder with the injected native token,
then the fixture combined incompatible local export and `--save`,
emitting `multiple outputs currently unsupported`.
This is a fixture error already covered by the exporter troubleshooting documentation,
not a token rejection.
Local-export-only correction `pxx75mxnw2` is terminal finished with its actual frozen-source consumer passing.
The separate project-scoped registry auth run `38z4wlq9d3` is terminal finished:
anonymous and invented-token requests returned 401,
while the injected native token read the OCI graph and executed the extracted frozen queue consumer.
The platform owner independently rehashed index children,
config and layers from the original ZIP.
Its owned project `ng17dl3xrn` and image were deleted,
with manifest 200 changing to 404 and an exact project image query returning `not_found` at `15:38:29 UTC`.
This is logical deletion evidence,
not physical erasure.
The platform native slot is released.
Desktop continuation `0jpk5bwpvs` executed all six remaining authored sessions.
Its structured summaries contain 41 passing and one failing assertion,
with no skips;
duplicated raw output lines are not additional cases.
The failing dialog assertion is `Ctrl+Shift+O opens the system folder dialog`.
The owner verified zero retained app/compositor processes,
units or container,
restored sinks and absent credential.
The Rust cache campaign is terminal and its slot released at `15:48:32 UTC`.
Native graph `gb01kjx2qk` is dispatched on exact source `234ccc92dabb81e998e0a0bb5e838713924dd83f`,
with two sequential samples and one native job maximum.
Its finite terminal watcher process session `84977` is reaped:
native run is terminal failed at `16:04:04 UTC` and produced both graph artifacts.
Both exact job/attempt/sandbox identities are retained in the private terminal receipt.
The native collector process session `52194` is reaped zero;
both original ZIPs passed CRC/API-size checks and all 222 original command-log byte/hash joins per sample.
Shared builds passed all 42 commands in 61.660 and 61.439 seconds;
both Rust owners passed in 283.541 and 296.024 seconds.
Browser preparation reproduced the exact documented rootless Podman group-identity rejection in both samples.
Those samples give no browser-case credit.
The isolated correction `72af18d5b130dabef7ff5b656ffdc1b58df4fcde`
uses the existing verified `sudo --non-interactive` Podman boundary only for native Depot.
Independent execution of its exact bootstrap passed 24 bounded provider/command controls;
receipt `doc/audit/depot-expanded-graph-podman-preflight-2026-10-18.evidence.json`
is sealed at commit `5edbdadcc2`.
The native correction admission retains the existing two-sample USD 0.72 reserve within the USD 3 guard;
its cumulative conservative campaign proxy remains USD 2.9096.
The real corrected native consumer remains required;
local fake-executable routing controls give no application case credit.
The GitHub terminal watcher session `63906` is reaped.
Completed GitHub samples already have CRC/API-size/API-digest-verified original ZIPs:
GitHub one `12142265999`,
199494 bytes,
SHA256 `761b6d70731dc17c86771d34cb090644c30588c2e1c786c6db83f85f63c85635`;
Depot one `12142052987`,
222777 bytes,
SHA256 `48071e722f955b1be4b9758e2d5fef627845c83a1f6ca047dcb0f1bf2aa8234e`.
Private canonical sample bodies are `graph-corrected-github-1.json` and `graph-corrected-depot-1.json`.
Both completed all 42 shared builds and both actual Rust owners.
GitHub one has 28 commands not started because of the campaign budget and six command-budget outcomes;
Depot one has all 222 command logs and 25 failing commands requiring source/prerequisite classification.
Thus their full graph durations are not yet a like-for-like speed comparison.
Shared and Rust completed subgraphs can be compared after unchanged-input spread and output validation.
Never count a populated outcome `startedAt` or `log` field as proof of a child invocation:
unstarted budget placeholders also carry them;
join actual original ZIP member presence and explicit state.
Depot's second GHA sample is also collected:
artifact `12141519578`,
223331 bytes,
SHA256 `452edf15dabd5cf769e1fb3965db010518ceda8024dd832a28b94494462191a5`.
It retains all 222 original command logs,
with 196 passing commands and 26 requiring classification.
Shared builds passed in 72.591 seconds and Rust owners in 340.119 seconds.
The private inspector now counts actual ZIP members,
replacing its erroneous placeholder-based `actuallyStartedCommands` projection.
All 1082 retained original command logs across these five samples pass exact byte/hash joins.
The last public GitHub repetition is collected:
artifact `12142748843`,
199971 bytes,
SHA256 `90209f8bfe7778673f7f3b83ba851440542b045adc584360096ee4da228948b2`.
It also has 194 actual command logs,
six command-budget outcomes and 28 unstarted commands.
The independent platform reviewer verifies all six original CRC/ZIP-byte/digest records
and all 1276 retained command-log hashes.
Both complete shared-build programs and Rust owner programs match across all four GHA repetitions.
Shared-build mean times are GitHub 210.210 seconds versus Depot GHA 73.2885 seconds,
with observed spreads 6.962 and 1.395 seconds:
a 2.86825 ratio of means.
Rust owner means are 720.5195 versus 325.8625 seconds,
with observed spreads 10.631 and 28.513 seconds:
a 2.21112 ratio of means.
These compare completed phases,
not complete pipelines or a fixed randomized property-test corpus.
The first Rust identity projection erroneously included Nextest's changing progress ordinals;
the actual format was inspected and the corrected case-only projection matches all four samples.
It contains 89 Truepeak and 237 forbidden-strings passing identities per sample;
the two Truepeak skips are summary counts,
not enumerated skip identities in this projection.
The Rust owner independently validates that normalization and its negative controls before final publication.
Both original native ZIP digests also match their actual upload-action log digests;
the action's exact message is `SHA256 digest of uploaded artifact zip is ...`.
Native correction run `fnvwqfg0bb` is dispatched once on `72af18d5b1`;
its finite terminal watcher is process session `20838`.
Inspect all failure families and seal corrected native consumers before publication.

Packed consumers are sealed at `898fd6ab03`:
25 actual runtime imports,
one type-only package checked with positive and negative TypeScript consumers,
and all five functional consumers including real mutation CLI dry-run discovery.
Dry-run discovery is not executed mutation testing.
The type-only package's runtime-import rejection is a harness expectation error,
not a product failure.
The separate current-source Depot Windows and Startup-trial Mac controls retain their USD 2 campaign guard.
Current-source public host controls `39141147245` are dispatched on harness
`c50ed764237d45d5fc7aa123e86b3dc43891e9da`:
five sequential contexts,
locked Node 26.12,
frozen working-byte manifests and actual emitted-binary consumers.
The owner corrected GitHub's 21,000-character per-step limit using byte-identical runtime bootstrap generation,
then ran positive and negative source-hash controls before dispatch.
Depot Mac/Windows allocation remains separate and has not started.
The first public Mac/Windows contexts exposed a research logger filename containing `/`
and an omitted hidden-artifact upload option.
Their raw logs retain actual filesystem/native-debug/emitted-version successes,
but no artifact consumer or case-level Nextest acceptance follows from that failed handoff.
The coverage owner preserves the failed cohort and corrects only research logging/upload configuration.
Pinned uploader controls prove zero default-hidden matches versus four explicit-hidden matches;
logger controls accept hostile task names and reject an existing output before child execution.
The exact corrected public recovery is run `39144499822`,
harness `aa501f4a5c0e74a0a993b668c98b00fe3be2da19`,
with the same five sequential contexts and finite watcher.
Matched Depot allocation follows this actual public handoff.
The coverage owner also confirms whether public desktop source archives contain image bytes or only identities,
closing the existing screenshot publication/privacy boundary without relaunching desktop or emulator work.

The parent finished substantive review of 18 deployed native-CI documentation pages,
156,769 exact source bytes and 95 claim-level findings.
The sealed private source/hash/review join is `native-ci-complete-source-review.json`
under the parent's expansion controller.
The platform owner is merging those findings into the public source ledger,
preserving the separately reviewed split-tests page.
The parent owns a concise canonical interpretation note and the final report integration.
The parent finished all eleven remaining Cache integration pages,
69,290 verified source bytes and 50 claim-level findings,
with a sealed `cache-integration-complete-source-review.json` sent to the platform ledger owner.
The full 13,860-path frozen inventory supplies positive owner controls and exact named-manifest applicability.
The canonical interpretation is `doc/planning/depot-cache-capability-review.md`.
The GitHub Cache integration guide explicitly documents no branch isolation on Depot;
the Rust researcher owns a bounded cross-branch cache control before closing that trust-scope finding.
Its six sequential two-minute jobs plus one correction reserve use a separate USD 0.084 uniform proxy
under a USD 0.10 guard;
the parent coordinated dispatch after the platform slot release.
All six cache jobs completed without a retry:
GitHub seed `39141719724`,
same-branch `39141763438` and sibling `39141821223`;
Depot seed `39141909098`,
same-branch `39142023227` and sibling `39142130086`.
GitHub's verified same-branch exact hit is followed by a sibling miss;
Depot's sibling actually restores the exact 53-byte marker.
This proves that benign sibling-read scope,
not every fork/PR write-poisoning scenario.
The Rust owner seals the source archive and branch cleanup before final report integration.
No account setting or broad credential is needed.

The original graph admission's USD 1.20 Depot nominal full-window reservation was incorrect.
Current runner-types documentation gives four-CPU Linux USD 0.012 per minute,
so two native and two Depot GitHub jobs at thirty minutes each reserve USD 1.44 before allowances.
Public GitHub nominal compute remains zero.
The six-sample uniform Depot-rate proxy is USD 2.16,
not an observed all-provider invoice.
The original four failed job intervals total 148 seconds,
including 71 seconds on Depot runners:
USD 0.0142 nominal Depot compute and USD 0.0296 uniform admission proxy.
The corrected six-sample proxy plus USD 0.72 correction reserve and observed failed proxy is USD 2.9096,
within the existing USD 3 campaign guard.
The [failed setup receipt](../audit/depot-expanded-graph-setup-failure-2026-10-18.evidence.json)
retains the original admission correction and exact four artifact digests.

The combined allocation ledger at `acf8a8c195` freezes 223 records and known partial nominal compute
USD 6.1999959,
with BuildKit's USD 0.538584 request proxy kept separate from unmeasured actual BuildKit compute.
The parent independently verified its arithmetic at `15:41:16 UTC`:
all 411 original bodies,
223 unique records,
195 exact timestamp/rate/cost joins and separate exact BuildKit fractions pass.
The published independent receipt is
`doc/audit/depot-owned-allocation-independent-arithmetic-2026-10-18.evidence.json`,
committed with the cache interpretation as prepared `aa3427f187`,
landed `7eb1f51231` and verified as an ancestor of the directly queried remote main tip.
The Rust owner now appends a separate sourced follow-on ledger after sealing cache results,
deduplicating the frozen baseline and all later coverage/platform/cache/root identities.
The parent reviews that arithmetic when all graph/OS attempts are terminal.
Admission maxima are not observed charges or invoices.
The root report now integrates packed and desktop consumers,
native automatic BuildKit/registry success,
static/property/mutation dispositions and the baseline arithmetic,
while retaining pending graph/OS/source-semantic closure.
The platform owner has now finished all 120 deployed documentation rows:
119 full source-review objects plus the separately retained split-tests identity/adapter review.
No retrieval-only generic rows remain.
The source owner finishes publication and command/API applicability mapping without new allocations,
then remains available for independent report sufficiency review after factual queues close.
Its publication checks now pass,
with all 99 distinct CLI invocations,
31 protobuf services/208 method declarations and 63 deployed operations mapped with explicit runtime limits.
It closes two previously tracked local troubleshooting/prototype gates for empty image JSON and the old Semgrep wrapper,
using bounded isolated fixtures and no new cloud allocation or upstream message.
Material final constraints include soft BuildKit cache-size policy,
deprecated Dagger integration,
root-modifiable Linux-only egress and distinct GHA/container minute caps without a demonstrated combined dollar cap.
The Root cache note now integrates the sealed six-job result,
actual GitHub default `write` mode and untested default-branch sharing,
fork/write-poisoning and retention boundaries.
The private renderer crossed Node's default output buffer on the growing handover AST;
following the existing bounded-buffer troubleshooting entry fixed the parent capture policy.
All fourteen render targets now pass,
including the 1,067,303-byte handover AST and 215,831-byte HTML.
Actual cache,
allocation and Git-recursion HTML have been read;
the latest handover/report changes still need their final render/read.
The 16:03 rerender now passes all fifteen targets,
with current report 26,045 bytes,
handover HTML 218,574 bytes and handover AST 1,080,555 bytes.
The current report HTML,
latest handover top,
new buffer section and Git-worktree recurrence have now been read.
Later edits require scoped final rerender/read.
After factual queues close,
obtain an independent sufficiency review,
finish the approved report sections and only then start the decision interview.

## Historical execution checkpoint on 2026-10-18 at 14:26 UTC

The factual research queue remains active;
do not start the interview yet.
The parent owns the final report and repeated coherent Linux graph campaign.
The Rust researcher now owns the combined exact-allocation ledger and independent review of new parent evidence.
The coverage researcher owns remaining desktop sessions,
proper packed-install activation,
current-source standard GitHub macOS/Windows controls and the semantic assertion manifest.
The platform researcher owns substantive source-ledger completion and a new automatic native-job BuildKit auth probe.
That probe uses only the injected short-lived job token,
one owned temporary project and separate build/registry assertions under its USD 1 guard.
The parent holds its native graph allocation until that probe is terminal.

The stronger [guest-file receipt](../audit/depot-mvm-guest-files-2026-10-18.evidence.json)
is committed at `c65e3b8eeb`.
Native `25jpltfpkq` and both jobs in GitHub `39134227406` use harness
`1373e624739fbc93502b0c8ca502820c6ff99b45`.
Each actual guest has a distinct boot identity,
reads the exact 51-byte input,
creates an initially absent distinct output and returns the exact transformed 57 bytes.
All three original ZIPs and 198 file identities are retained;
the original host-shared-directory-only check remains separately qualified.
The corrupt-expression dispatch `39132577785` is a retained queued harness failure,
not a timed execution inferred from the API's populated `startedAt`.
No invoice or VM billing-release claim follows from workflow completion.

The repeated Linux graph uses clean pushed harness
`815591bf307b1237d31b1c4e644463d7c13654fa`
on `research/depot-expanded-graph-20261018`,
with unchanged application cohort `e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`.
GitHub dispatch is `39135886118`.
It contains sequential GitHub and Depot-GitHub repetitions;
the native workflow adds sequential native repetitions after its slot is released.
The graph invokes 222 actual owning commands:
a shared Node prerequisite phase,
parallel module/browser branches and an independent Rust branch.
This is one four-CPU Linux job per sample,
not a distributed cross-job artifact DAG or complete repository CI.
It preserves source failures and measures setup plus complete-report latency.
It restores no explicit repository/package/compiler cache or prepared snapshot;
provider image/tool/registry state is not assumed empty.
The runtime source comparison uses 54 frozen top-level Git object identities,
excluding declared research configuration directories,
and tracked working-tree changes without requiring a shallow checkout to fetch the application parent commit.
Local source controls observed unchanged success,
changed-object rejection,
a removed guard missing the difference and restored success.
The six planned jobs each have four CPUs,
16 GB and a thirty-minute limit,
with at most two parent-owned concurrent jobs.
The USD 3 gross proxy guard includes USD 2.16 full-window all-provider equivalents
and USD 0.72 for two bounded corrections;
the initial nominal Depot compute reservation is USD 1.20 before allowances.
Public GitHub compute is modeled separately at zero nominal charge.
The private controller retains the admission,
source verification and dispatch response.

Retain terminal graph job/attempt metadata and original evidence ZIPs,
measure unchanged-input spread and actual overlapping critical paths,
join them into the Rust researcher's allocation ledger,
finish remaining owner-specific factual queues and obtain an independent sufficiency review.
Then reconcile the approved six report sections and start the decision interview.

The coordinated native order is automatic BuildKit auth,
remaining desktop sessions,
then the repeated native graph;
the GitHub graph matrix proceeds independently.
The retained historical Mac/Windows owner evidence uses standard GitHub runners,
not Depot operating-system runners.
Equivalent current-source Depot Windows and Startup-trial Mac controls are now coverage-owned
under a separate USD 2 maximum reservation,
with Developer Windows eligibility and Startup Mac eligibility kept distinct.
The latest packed-install run `39136177894` reached all five functional consumers,
including the real mutation-test CLI's dry-run discovery.
Its remaining namespace-import failure concerns a type-only package;
the coverage owner is replacing that invalid runtime expectation with an actual compiler consumer.
No runtime import or executed mutation credit follows from the type-only package or CLI dry run.

## Historical execution checkpoint on 2026-10-18 at 12:20 UTC

The expanded evaluation remains research work;
the interview has not started.
The [working expanded report](../planning/depot-expanded-ci-evaluation.md)
now uses the approved six sections;
it is not the final sufficiency verdict.
The [combined pricing model](../audit/depot-combined-product-pricing-2026-10-18.evidence.json)
has executed separate-product scenarios,
including the counterexample to an all-product Developer cost-dominance claim.
Actual whole-graph allocation and critical paths remain parent-owned.
After completing its active Truepeak fixture and publication,
the Rust researcher owns independent review of the parent's eight policy ZIPs,
three process ZIPs,
eight VM/KVM ZIPs and combined pricing arithmetic.
The review also checks the working report outside that researcher's own Rust claims,
with no new cloud allocation.
The coverage researcher published its separate
[per-job budget ledger](../planning/depot-complete-coverage-budget-ledger.json).
The corrected standard-GitHub `mvm` lifecycle passed in `39125868595`,
source `64d6a666e63e4773ee56666148ae9c8ba371877a`.
It preserves runner UID/GID `1001`,
adds device group `993` only to the child,
executes the actual frozen owner,
checks guest stdout and exit-two stderr,
and roundtrips 51 bytes with SHA256 `07442f8c71b3a17811a517cd5fceea7977e97e6f4a17d6a95a7602b7dc7daae8`.
KVM is present and enabled;
final private domains are empty and both owned daemons exited without cleanup errors.
Its original 48783-byte ZIP and 97 member identities passed API-digest,
CRC and SHA verification.
The named-ACL removal writer remains unidentified.
The failed seventh attempt `39124932077` lacked workspace pnpm setup and did not exercise a guest;
the eighth attempt restored the original frozen install recipe before execution.
The [complete VM/KVM sequence](../audit/depot-mvm-kvm-2026-10-18.evidence.json)
and final troubleshooting correction are public.
All eight original ZIPs are retained.
The job-level ledger is 619 observed seconds,
USD 0.0498 gross Depot compute value before allowances,
and a separately labeled USD 0.1238 all-provider four-CPU equivalent proxy.
It does not establish an invoice or billable VM lifetime.

The per-test identity adapter passed actual `st40hvmw90`:
both physical shards found historical timings for all 34 candidates and passed their exact 17-case partitions.
Its [public receipt](../audit/depot-tests-identity-adapter-2026-10-18.evidence.json)
retains the original and derived report relation,
stored rows,
source and guard-removal controls.
The platform researcher owns remaining source-ledger reconciliation,
the queued repository ARM check and
[included-allowance reconciliation](../planning/depot-included-allowance-reconciliation.md).
Do not extend the older native-CI-only Developer/Startup arithmetic to combined product usage.

The fresh compiled artifact consumer verified 1036 declared hashes,
including the hidden manifest.
Actual runtime consumption initially failed because upload/extraction flattened producer symlinks.
The same immutable artifact passed permission and prohibition after restoring source-owned activation and
emitted dependency links in `39126304854`,
source `bb659f0f7fea71de9a2ba850bd04c5d0919fcb8e`.
No application build or unit suite ran in that consumer,
and both successful domain cases made zero HTTP/provider requests.
The remaining `none` case reached its preserved `unresolved` versus `unspecified` source assertion.
The coverage researcher owns complete matching-image browser/E2E receipts,
finite mutation and conformance completion and their canonical coverage note.
Separate remaining coverage items are actual desktop compositor/ImageMagick/PNG prerequisites and consumers,
proper packed-install consumers,
and semantic feature/assertion-manifest completion.
No other lane owns these items;
the coverage researcher must reconcile measured/reserved campaign cost before each additional bounded profile.

The broader Rust ledger records all 35 bounded ASan fuzz targets completed,
Truepeak's actual all-features Nextest result of 89 passing cases and two expected skips,
and independently parsed planted-fault results.
The Rust researcher owns remaining benchmark and privileged WG terminal evidence.

The [matched process receipt](../audit/depot-process-timeouts-2026-10-18.evidence.json)
contains all three original ZIPs.
All three providers allowed the finite seventy-second writer past a sixty-second step deadline.
Only the native observation establishes completion after the executor advanced to its next step;
both GitHub-orchestrated observers started after writer completion.
The [source-backed note](../troubleshooting/depot-step-timeout-process-boundary.md)
keeps that difference separate from unknown signal routing,
VM release and billing.

The parent queue is container Git/libatomic prerequisite notes,
job-environment recurrence documentation,
combined provider/cost/critical-path modeling,
the approved six-section expanded report and independent sufficiency review.
Disposable PR cleanup and ordinary/default/step continuation publication are complete.
Research snapshots remain available for their active consumers;
cleanup follows the last owning campaign.

## Historical execution checkpoint on 2026-10-18 at 11:38 UTC

The actual single-job retry of native run `xgq2429hqv` completed artifact download on attempt two.
Its consumer then correctly rejected the old fixture's failed assertion.
The corrected fresh run `1jqm5wjmlj`,
harness `bf3c4a98b32826c1c16e8755b8775256340fab79`,
passed its downstream consumer with the current producer hashes and rejection controls.
The subsequent full workflow rerun regenerated passing producer receipts,
but its downloader selected the first attempt's `allrun` archive alongside current state/default archives.
The consumer rejected the current-output hash mismatch.
All seven producer/consumer archives across both attempts are retained privately with CRC and SHA checks.
The published SDK selector sorts same-name artifacts by numeric ID,
and the actual older `allrun` ID is larger than the rerun's ID.
Both IDs are safe integers;
precision loss does not explain this pair.
The backend's ID-construction mechanism remains unknown.
The Action's raw pre-filter response was not captured;
the selector vulnerability is proven but a visibility race is not excluded as the incident mechanism.
The source trace and controls are in
[`depot-rerun-artifact-id-order.md`](../troubleshooting/depot-rerun-artifact-id-order.md).
Content-bound names at `46fab80ef500e7d857cdb822085c03f899e266ff`
passed their first fresh consumer in `w9jdpj5f1l`,
with all current producer hashes and six source/hash rejection controls.
Its full workflow rerun passed the same current-output and rejection controls.
Both attempts join their current producer archives and ten distinct provider sandbox identities.
The [public receipt](../audit/depot-native-orchestration-2026-10-18.evidence.json)
retains fifteen original ZIPs.
Do not infer passing aggregate workflow status from that consumer:
the same diagnostic workflow contains intentionally failed parallel jobs and a cancelled timeout job.
The isolated ordinary job-level comparison completed as native `kjf78stmkb` and GitHub `39122132487`:
both expose `needs.allowed_failure.result` as `success`,
while native aggregate and GitHub App suite report failure and the GitHub workflow reports success.
There is no cancelled timeout in that isolated workflow.
Default downstream eligibility and an isolated step-level workaround are now verified controls.
Their runs are `pfzfjrqdpq` and `lrhfzwrwl2`,
at `714a3cad36c20da75969454fe33f28dab7770e9b`.
The default-condition downstream ran successfully without `always()`;
the native aggregate still failed.
The separate one-operation step-level workaround finished its job and workflow successfully,
retaining step outcome `failure` and conclusion `success`.
The public paired policy/history receipt now contains all eight original ZIPs,
including these controls.
The semantics campaign's cumulative full-window reservations are USD 0.960 under the USD 1 guard.

The earlier [Depot Tests trial](../audit/depot-test-feature-2026-10-16.md)
already qualifies physical binary-level sharding,
failure/skip retrieval,
retry history and analytics on its own source.
The new current-source profile extends that evidence to 34 individual test names,
with unchanged-input seed repeats and separate physical shards.
Its exact Nextest selector controls and local CLI `2.102.23` preflight passed.
The first run `b7zbgvws7h` failed before Nextest because the child command inherited a directory
outside its explicitly discoverable Mise configuration.
The source-backed correction at `e1b2f76f62fbb7fea99eef4fca88443d54308455`
passed actual run `fm9d4wkbj9`.
Three seed executions passed 34 cases each;
two physical shards passed 17 cases each;
their downstream consumer verified the exact disjoint 34-case union.
Attempt-scoped result APIs independently returned all 136 passing events,
with 34 unique case names and four distinct provider sandbox identities.
Both shard commands found timings for zero candidates.
Physical execution and ingestion passed.
The later read-only control established a harness identity error:
the full test-name candidates do not match their JUnit classname,
`monochromatic-git-policy-cli`.
Depot found timings for that actual classname under both test-name and class-name timing granularity;
all full test-name candidate controls and the absent-class negative found zero.
Changing report-key variants did not change that observed identity distinction.
The current [splitting documentation](https://depot.dev/docs/ci/how-to-guides/split-tests.md)
requires file/class candidates even when timing granularity is `testname`.
The original zero result does not establish unavailable historical data or a vendor history defect.
The [paired policy/history receipt](../audit/depot-job-policy-history-2026-10-18.evidence.json)
retains all eight read-only requests and their actual outputs.
The two full profile reservations total USD 0.348 under the USD 1 campaign guard.
Original evidence ZIPs,
stored result rows and positive/negative transport-secret cleanup are public in
[`depot-physical-test-shards-2026-10-18.evidence.json`](../audit/depot-physical-test-shards-2026-10-18.evidence.json).
The first failure and verified directory remedy are in
[`mise-excluded-research-task-discovery.md`](../troubleshooting/mise-excluded-research-task-discovery.md).
Do not repeat the prior failure/skip/duplicate trials as though those capabilities were untested.
Issue `1046` remains open;
the current public service protocol still says `duplicate_invocation` is always false.

The broader Rust work executed 336 passing core cases with 13 skips among 349 enumerated cases.
The current-source Rust linter executed 415 passing cases with one skip among 416 enumerated cases.
Its standalone compiler consumer passed in `zxlt43t7g1`,
using Cargo's JSON-emitted matching rlib/rmeta paths.
That consumer remains separate from the 415 passing unit cases.
The actual frozen-source `wg-quicker-exempt` producer was consumed by GitHub run `39118628603`:
all 44 owning invocations passed,
including the formerly missing companion-presence boundary.
Those tests verify the actual sibling ELF's presence and hash,
not execution of the privileged BPF program.
The exact transport secret was removed and HTTP 404 verified.
The original missing-producer fixture remains retained.

The raw GitHub-hosted KVM diagnostic completed as run `39115506319`.
QEMU `q35` with KVM passed as the ordinary user after scoped device access was granted;
the later `none` machine user probe failed opening `/dev/kvm`,
while the root probe passed.
Libvirt's KVM capability probe still failed.
This disproves a GitHub-wide KVM-absence claim;
the access change between probes is not yet explained.
The parent owns its public receipt and deciding-source note.
The installed Libvirt source launches its KVM capability probe with machine `none`;
the changing ordinary-user access still needs per-probe state evidence before assigning a cause.
The bounded follow-up is GitHub `39123865841`,
at `fb143279d6adc8a2dd6f0b6c77ad584ae5a2daef`.
It captures device inode,
mode,
owner,
ACL and actual caller access around each raw QEMU control,
including root identity,
root QEMU,
idle and repeated ACL-grant controls.
Its process-discovery check must find a live paused TCG process before trusting cleanup absence.
Only one 128 MiB paused QEMU process may exist at a time;
there is no application guest.
The job has eight minutes,
with a six-minute inner deadline.
The measured five prior intervals plus its full reservation have nominal proxy cost USD 0.1796,
under the unchanged USD 1 campaign guard.

The default/step continuation notes and their updated rendered checks pass.
The disposable PR cleanup is public and freshly reverified:
the research base returns HTTP 404 and production branch protection still matches its baseline.
The remaining parent queue includes the latest KVM-note verification and evidence publication,
VM/process public receipts,
container Git/libatomic prerequisite notes,
job-environment recurrence documentation,
PR cleanup publication,
the provider/cost/critical-path model and the six-section evaluation report.
The routing researcher owns finite fuzz,
coverage,
conformance,
mutation-container and three-engine browser waves.
The Rust researcher owns GNU/ASan fuzz,
actual deterministic Codex host completion,
all-features Truepeak Nextest,
and source-qualified benchmark/privileged-BPF applicability.
The deterministic Codex host now has forty real successful command outputs in `1h6qf8j86f`;
Bubblewrap was installed in the disposable job without changing the signed Codex binary.
The TypeScript property/coverage/conformance wave reached all 69 owning invocations,
with 64 passing and five retained domain or baseline failures.
The routing researcher also owns the actual three-engine browser wave `39123458425`
at `6328647d8b7113a920d777044944612f3bc47a58`.
The platform researcher owns command/doc inventory reconciliation,
the stopped-SDK evidence and the per-test JUnit identity adapter with physical consumer checks.
Its adapter run is `st40hvmw90`,
source `c6d25d092678b0933f1e3b43765fc46c02593a9e`,
with one VM at a time and the unchanged USD 0.534 cumulative full-window admission.
After that campaign and source-ledger reconciliation,
the platform researcher owns an actual frozen Linux ARM64 repository CI check,
or reconciliation of sufficient existing immutable evidence.
The multi-platform BuildKit Node/ELF probe alone does not establish repository ARM CI execution.
The initial separate guard is USD 1,
one VM,
eight CPUs and fifteen minutes;
it provides functional evidence unless unchanged repetitions justify a performance comparison.
That adapter may use one parent VM slot;
the KVM follow-up reserves the other.

The raw KVM access follow-up completed successfully with 46 sequential snapshots.
The named runner ACL disappears after successful QEMU use,
and regranting restores both machine types.
The process-discovery positive and final cleanup controls passed.
Its deciding device-open source and the unknown ACL-writer boundary are recorded in
[`depot-ci-versus-github-runner-platforms.md`](../troubleshooting/depot-ci-versus-github-runner-platforms.md).
The separate descriptor/group lifecycle control is GitHub `39124932077`,
at `bafc75bb388a9f92dfc056f0bb409d86d48cc1ee`.
Its full reservation plus six measured intervals is nominal USD 0.3938 under the unchanged USD 1 guard.
It has a twenty-five-minute outer control envelope,
a four-minute permission step and an eighteen-minute actual VM step.
The actual guest remains four vCPUs and 8192 MiB;
the original application manifest's twenty-minute reference is retained separately from the new outer envelope.
Supplementary device-group access affects only the disposable child process,
with no group database edit.
Daemon log capture is now isolated from lifecycle cleanup before this full guest retry.

The broader Rust work now has completed bounded ASan execution for all 35 frozen fuzz targets.
Truepeak's actual all-features owning Nextest invocation passed 89 cases with two expected skips,
separate from the earlier build/cache-only zero-test result.
Git-policy planted faults were all detected,
with exact JUnit identities corrected by independent XML parsing.
The remaining benchmark and privileged WG campaigns remain active in the Rust-owned ledger.
The fresh native artifact consumer verified all 1036 declared compiled hashes,
including the formerly omitted hidden manifest,
then failed its first actual domain import before any permission/prohibition/none cases ran.
Its runtime topology diagnosis remains the routing researcher's active task;
artifact membership success does not establish runtime portability.
Independent sufficiency review precedes the interview.

Research remains open;
do not resume the interview or declare readiness from the completed module sample.
The frozen application source is `e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`.
Later main changes do not silently enlarge a frozen cohort.

The exact cloud APK pair now has a completed local headful consumer in
[`depot-android-consumer-2026-10-18.evidence.json`](../audit/depot-android-consumer-2026-10-18.evidence.json).
All 16 source-derived cases were reported:
14 passed,
two retained the known `SessionStoreTest` context-fixture failure,
and none skipped.
Private emulator/ADB cleanup completed.
The successful producer and its matching certificates remain distinct from the first APK-only launch.

Both native Depot and Depot's GitHub runner completed the frozen `mvm` Linux lifecycle.
GitHub-hosted Ubuntu failed libvirt's KVM capability boundary after a successful device-access adjustment.
The parent owns a further bounded raw QEMU/libvirt diagnostic,
the public VM/process receipts,
PR cleanup receipt completion,
and the associated source-backed troubleshooting notes.

The repaired container consumers completed successfully in native run `g02lgcbfgq`
and GitHub run `39113547458`,
at harness `5485cc0d12805f330d568b358e5ff0bb988d4d37`.
Native AMD64/Arm64 host and job-container jobs passed;
GitHub-hosted AMD64 and Depot AMD64/Arm64 host and job-container jobs passed.
The job image digest is
`sha256:853b99050d5420698a02d7cb10978be8824dc0999c9ace08ec453706da54ec31`.
The actual consumer checks the locked Node 26 binary hash and its loaded `libatomic` mapping,
separately from the image's base Node 24.
Earlier Git-absent and `libatomic.so.1`-absent fixtures stay retained as failed prerequisite controls.
The public receipt and all ten original ZIPs are retained in
[`depot-service-containers-2026-10-18.evidence.json`](../audit/depot-service-containers-2026-10-18.evidence.json)
and its linked compressed archive.
Commit `0a8a2979e0` retains them;
scoped dprint passed.
The exact GitHub secret and both owned native variants were removed,
with absence verified after positive lookup controls.
The builder researcher has been told the consumer hold is released.
This is secret-binding cleanup,
not proof of token revocation.

The matched lock/evidence campaign is complete on native Depot,
GitHub-hosted Ubuntu and Depot GitHub runners.
Every ordinary-directory repeat failed within the same 34-case cohort;
all nine tmpfs repeats passed 34 of 34.
Provider-labelled receipts and deciding source are in the cache report and
[`git-policy-replaced-lock-filesystem.md`](../troubleshooting/git-policy-replaced-lock-filesystem.md).
Do not attribute these failures to Depot alone.
The exact temporary signed-URL secrets were deleted and absence verified.
The cache audit now has 21 job-attempt records across 20 workflow runs,
with a conservative USD 1.407 request-to-job-terminal proxy,
not allocated CPU time or an invoice.

After finishing that evidence's lint/render and owned cleanup,
the cache researcher owns the broader frozen Rust/current-source linter and Rust fuzz area.
It must classify all 23 standalone Cargo manifests,
compare exact prior source/dependency/toolchain evidence for valid carryover,
and prepare documented bounded profiles before launches.
Its unchanged cumulative USD 5 proxy guard includes the completed USD 1.407;
at most two of its jobs may be active.
Main remains docs/evidence only;
heavy work stays in isolated cloud jobs or bounded containers.

The parent still owns physical Depot Tests shards/report consumers,
native retry/rerun and corrected state-artifact consumption,
the raw GitHub-hosted KVM diagnostic,
the cost/critical-path model,
and the approved final report.
The frozen inventory contains five Rust fuzz sidecars,
not only the two `rust-module` sidecars:
`cli/forbidden-strings.fuzz`,
`git-policy/cli.fuzz`,
`linter/monochromatic-lint.fuzz`,
`rust-module/forbidden-regex.fuzz`,
and `rust-module/jsonc-edit.fuzz` under `package/`.
Inspect each owning task before running;
some already implement container bounds and planted controls,
and some dual-corpus commands can mutate committed seed inputs.
The cache researcher owns their source preservation and explicit ASan/GNU target evidence.

The KVM follow-up completed as a fifth allocation after four terminal guest jobs.
The retained admission measures 358 preceding request-to-terminal seconds,
models them at USD 0.0716,
and reserves USD 0.12 for one ten-minute GitHub-hosted diagnostic.
The USD 0.1916 forecast stays under the unchanged USD 1 campaign guard.
Harness `0a2fd7678b` calls the source-bound raw QEMU/libvirt capability task;
it does not execute an application guest.
GitHub run `39115506319` completed at full harness
`0a2fd7678b719d9039df318003a43bbc4713cd7c`.
Both raw q35 KVM probes passed,
including the unprivileged process after the scoped ACL grant;
QMP reported KVM present and enabled.
Later unprivileged `machine=none` failed opening KVM with `Permission denied`,
while its root control passed and private libvirt rejected KVM capability.
The diagnostic retained TCG positives and observed both owned daemon PIDs gone.
This disproves a platform-wide absence-of-KVM conclusion.
Per-probe device permission snapshots were not captured,
so the changing access boundary and libvirt failure still need qualification.
The first dispatch's immediate run lookup failed after successful creation;
recovery adopted the existing run rather than dispatching another.

Native consumer retry was actually submitted for `xgq2429hqv` job `nw4mdr6b68`.
The API returned attempt 2 queued.
The original failed attempt and three original producer artifacts were retained first.
One additional two-CPU/eight-GB/five-minute job reserves USD 0.03
within the existing USD 1 semantics campaign guard.
Do not expect a passing aggregate from this unchanged source:
its default-parallel assertion assumes sibling absence,
which the stronger observation did not establish.
The retry qualifies lineage and the separate unexplained download-action boundary.

For physical Depot Tests,
the parent is preparing actual Nextest per-test JUnit from the source-bound 34-case lock diagnostic cohort,
using the retained executable/archive producer and explicitly owned tmpfs environment.
This will not disguise an owning-task wrapper row as an inner test case,
claim a filesystem source repair,
or qualify the whole Git policy suite.
Candidate selection,
report freshness,
history,
fail/skip/duplicate controls,
physical job identities and the artifact consumer remain required execution work.

The current-source Rust linter run `j50kf2jlqx` actually ran 415 passing tests,
with one ignored case among 416 enumerated cases across three binaries.
Its real CLI help/rule controls passed.
The aggregate remains red because the separate direct-rustc consumer encountered an rlib metadata-stub diagnostic;
that is not a linter test failure or a completed binary handoff.
The standalone Rust run `lwp0vbb2vq` is active,
including the frozen `wg-quicker-exempt` release producer needed by the routing researcher's actual default test.
The process/transport wave `39114552347` completed all 131 invocations:
128 passed and three default tasks failed.
Real compositor lookup/ImageMagick prerequisites and the CPU-cap watcher assertion are being qualified separately.

The capability researcher's additional admission is USD 2 cumulative timestamp compute,
with its existing two-builder/sixteen-CPU/thirty-two-GB bounds unchanged.
Actual Bake,
secret/SSH forwarding,
real Cargo cache mounts,
and remote-source controls remain its owned work.
Captured docs/repository enumeration is not complete substantive feature review.
Current source confirms `depot cargo` is a local Cargo wrapper for the same remote sccache backend,
not a separate remote Rust compiler.
The capability researcher is verifying its prerequisite,
authentication precedence and environment-forwarding boundaries.
The substantive chronology pass also covers Dockerfile lint,
Cache Explorer/export,
builder networking and Datadog's current availability.
Current project source defines `max_concurrent_builds` per builder instance,
not as a total-builder cap;
autoscaling stays off under the campaign's hard allocation bound.
Cache-policy `keep_gb` is per architecture,
so its configured value must not be described as whole-project storage usage or a shared-allowance guarantee.
The current Dockerfile-lint campaign has a working Hadolint positive and clean control,
but its missing Semgrep positive remains unqualified.

## Research reopened and interview withdrawn on 2026-10-18

The user rejected the readiness claim:
"Not enough research.
You're missing so much that it's hard to name them all."
They will return in the morning to inspect further research.
The parent withdrew the readiness claim and the three pending operating questions.
Do not wait for answers,
ask the user to enumerate omissions,
or restart grilling before adequate research.
The earlier independent review of selected evidence remains valid only for its checked boundaries;
its sufficiency conclusion is withdrawn.

New frozen source:
`e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`.
The [reopened plan](../planning/depot-developer-additional-pool-research.md)
records independently verifiable platform-surface,
complete-coverage,
reuse,
orchestration,
runtime,
Test,
economics and operations work.
The three research agents are active again.
Their new owned notes are `depot-platform-surface-audit.md`,
`depot-reuse-strategy-research.md` and `depot-complete-coverage-research.md`
in `doc/planning/`.
The parent owns actual event/status/gate probes,
runtime feasibility and integration.
All executable mutations remain isolated;
main receives documentation and evidence only.
Prior account-wide-access,
subscription,
production routing and external-provider restrictions persist.

Next action:
record and inspect the missing source/runtime surfaces,
execute the applicable current-source graph and deeper reuse/orchestration experiments,
and reconcile actual outcomes without declaring selected passing probes comprehensive.

## Active Developer additional-pool research on 2026-10-18

The user requested Depot Developer as an additional pool alongside GitHub,
then required comprehensive research before grilling.
They clarified that migration is the wrong framing:
use Depot's full capabilities and greatly expand the current lackluster CI setup.
The research now covers all CI-relevant Depot capabilities and desired repository coverage,
not only placement of existing jobs.
Developer remains the intended base plan;
record eligibility gaps rather than excluding capabilities from discovery.
Depot and GitHub Actions research experiments are authorized.
The [research plan](../planning/depot-developer-additional-pool-research.md)
tracks independently verifiable areas and the stop condition for beginning the interview.
Developer plus continued GitHub availability is settled;
pool placement and operational preferences will be grilled after factual research.

The capability and repository-routing research notes are complete.
Actual routing probes cover both pools,
single-pool selection,
intentional worker failure and cancellation propagation through a stable gate.
Fresh standalone Depot jobs verified automatic cache credentials,
compiler reuse and changed-flag invalidation,
plus concurrent persistent-disk consumers.
Historical standalone KVM guest execution has been reverified from its retained artifact.
SDK command and file consumers passed;
every created sandbox was independently confirmed finished.

The repeated fresh-job pnpm-store comparison is complete.
An initial cold cohort passed,
but seed run `39100149452` stalled in the parent-authored archive-size guard before saving.
Both old seed jobs and the workflow are confirmed `cancelled` at `05:32:36Z`.
This establishes terminal workflow state,
not VM destruction or stopped billing.
The corrected harness is `2a155eec49` on `research/depot-pnpm-cache-20261018`.
Its real success and intentional-failure controls pass.
The final cohort adds explicit cache-disabled controls,
uses unique scope `20261018-pnpm-cache-v3`
and has a 5-minute job timeout.
Parent monitor session `14886` completed all sequential phases successfully and retained terminal receipts
under private `depot-developer-pool-research/receipts-v3`.
The superseded seed monitor `59066` stopped on cancellation without dispatching warm consumers.
Initial v3 dispatch returned HTTP `422` for the unquoted `off` choice before allocating a job.
Its failed receipt is retained;
the quoted choice was accepted as run `39101112370`,
which passed all cache-disabled jobs.
Explicit-miss `39101194478`,
seed `39101280802`,
warm `39101349744` and branch probe `39101439840` also passed every job.
The [verified parent receipt](../audit/depot-pnpm-fresh-jobs-2026-10-18.evidence.json)
retains all 16 final-cohort jobs and verified artifact ZIP digests.
Restore plus install takes 11 to 12 seconds warm versus 9 to 10 seconds disabled on GitHub,
and 4 to 5 seconds warm versus 3 seconds disabled on Depot.
The apparent Depot whole-job improvement also includes checkout falling from 15 to 16 seconds
to 3 to 4 seconds before restore;
do not attribute that complete-job change to the pnpm archive.
The branch probe verifies a GitHub miss and Depot hit from a different branch at the same source.
It deliberately executes no installer or application consumer.
The GitHub archive was already absent before cleanup;
an exact key/ref query confirms zero entries.
No deletion command ran for that entry and no disappearance cause is established.
The worktree is safely removed and both pushed source branches remain.
The [pnpm research note](../planning/depot-pnpm-store-fresh-job-research.md)
records protocol,
storage ownership and failure details.
The independent review is complete and identifies no additional universal research prerequisite.
The operating frontier is spending authority,
qualified-check enforcement and infrastructure-recovery autonomy.
Use the [frontier review](../planning/depot-developer-interview-frontier-review.md)
for its option costs and rankings.
Final parent verification passed:
scoped Markdown lint,
rendered inspection,
JSON formatting,
source and cleanup assertions,
and 11 local links in the new parent notes.
The independent cache review verified 16 ZIPs,
67 member hashes,
224 step intervals and 42 successful build/consumer task invocations.
Its fresh GitHub API reads independently confirmed every artifact digest.

The entire first post-research frontier was asked together in the question tool on 2026-10-18:

- Q1:
  standing monthly owner-reserved headroom or approval of each paid batch.
- Q2:
  qualify new checks and then enforce them at publication boundaries,
  or keep them advisory.
- Q3:
  bounded automatic infrastructure retry and eligible-provider fallback,
  or require an explicit rerun.

The recommended choices are standing headroom,
qualified enforcement and bounded automatic recovery.
Answers are pending;
preselected options are not answers or authorization.
Recompute dependent decisions after the response.
Do not implement production routing until shared understanding is confirmed.
The main agent owns experiment harnesses and the combined decision tree.
The research agents own separate cited capability,
routing and cache notes.
Do not begin the interview while factual prerequisites remain investigable.
Do not change production routing or the subscription before shared understanding is reached.
Prior account-wide-access exclusions remain in force.
The user also requires performance claims to name the exact jobs and cache system.
The measured pnpm-store regression is GitHub `actions/cache`
in `readonly-semantic-bridge.yml`'s semantic-bridge jobs,
not Depot Cache and not separately measured logger/TOML regressions.
The cache decision now names that boundary.
The research plan retains a concrete proposed `QNB` tightening outside the Depot interview;
`AGENTS.md` is unchanged.

## Cache benefit clarification on 2026-10-18

The user asked how caching changes the performance assessment.
The approximately `3.26` GitHub/Depot elapsed-time ratio described cold compilation,
not the repeat-workload cache benefit.
No complete-graph cached speedup has been established.
The [fresh-job pnpm comparison](../planning/depot-pnpm-store-fresh-job-research.md)
has now completed for its named workload;
its restore overhead consumes the observed pnpm-command saving on both pools.

The corrected [default container builder trial](../audit/depot-runner-sizing-2026-10-15.md)
records two cold samples of `268.558` and `265.417` seconds,
averaging `266.987` seconds,
and unchanged repetitions of `22.297` to `37.654` seconds including local artifact export.
Dividing the cold mean by those warm observations gives approximately `7.09` to `11.97`.
That is a within-Depot cold-to-unchanged comparison,
not a provider speedup.

In the corrected sixteen-CPU/sixteen-job runner trial,
the source-change rebuild averages `122.5` seconds versus `246.6` seconds cold,
approximately a twofold elapsed-time ratio.
The changed CLI name is verified in the real executable.
This edit rebuilds both local packages under conservative content invalidation;
it is not a measurement of every possible source edit.

The [earlier GitHub reference](../audit/depot-trial-2026-10-15.md)
also records unchanged same-job builds of `1.716` and `1.975` seconds.
Those keep artifacts on the runner and do not measure cache restoration into a fresh job.
They cannot be compared directly with Depot's local-download boundary.
The native Mac/Windows follow-up records only artifact-cache misses and skipped cache saves,
so it establishes no warm-cache benefit on those platforms.

The [prepared desktop experiment](../audit/depot-prepared-desktop-2026-10-16.md)
does verify reuse across fresh consumers:
installed packages and source-bound binaries are reused without installation or compilation.
Fresh runtime services still need startup,
and changed build inputs require new preparation.
Preparation cost and storage cannot be omitted from a steady-workload model.

The [Cargo cache investigation](../troubleshooting/cargo-buildkit-source-timestamps.md)
caught an actual stale executable under timestamp-based reuse.
Content-aware package invalidation recovered the fixture,
and the corrected application matrices passed all 100 runtime output assertions.
Reject the stale subsecond source rebuild as a cache-performance result.
The workload's cold,
unchanged and edited-input frequencies remain needed to quantify total waiting benefit.

## Measured performance clarification on 2026-10-18

The user asked whether the measured benefit was a fourfold speedup.
No fourfold overall CI or complete-graph waiting improvement has been established.
The prior response's statement that measured benefit did not justify Startup was too definite:
the aggregate waiting-time saving needed for that return-on-cost conclusion remains unmeasured.
Developer remains the budget and CI-only pricing baseline,
not a conclusion that paid Mac capacity cannot repay its cost.

The [historical compilation observations](../audit/depot-trial-2026-10-15.md)
record GitHub's cold build at `869.508` seconds
and Depot's four-CPU runner at `757.394` seconds:
an observed ratio of approximately `1.15`,
with `112.114` seconds saved in those individual observations.
Each route has one successful cold observation;
they do not establish a repeated provider effect.

The [default-builder concurrency trial](../audit/depot-runner-sizing-2026-10-15.md)
records cold samples of `268.558` and `265.417` seconds,
averaging `266.987` seconds.
Comparing the earlier GitHub observation to that mean gives an elapsed-time ratio of approximately `3.26`
for that single build operation,
or about 14 minutes 30 seconds versus 4 minutes 27 seconds.
Hardware,
compilation parallelism,
harness revision and artifact export paths differ.
This is an observational elapsed-time ratio,
not an isolated provider effect or a complete-graph speedup.

Within the repeated Depot sizing experiment,
four CPUs/four Cargo jobs average `526.7` seconds
and sixteen CPUs/sixteen jobs average `246.6` seconds:
approximately `2.14` times as fast,
saving `280.1` seconds per measured cold build.
Both allocation and concurrency change;
each configuration's repetitions occur on one allocated runner.

The native Mac and Windows follow-up established capability and consumer outcomes.
It did not establish a controlled GitHub-versus-Depot speedup or monthly waiting-time reduction.
The private `recorded-provider-speed-comparison.json` retains the arithmetic inspection.
The prior final verification receipt remains a timestamped record of its original document bytes.

## Native follow-up terminal result on 2026-10-18

The authorized Depot native runner follow-up is complete at the consumer boundary.
The [expanded evaluation](../audit/tech-depot-complete-ci-trial-vet-2026-10-18.md)
records the final scoped verdict:
intentional GitHub/Depot routing,
with Developer plus selective overage as the planning baseline.
Developer includes Depot Windows runners;
Depot Mac runners require Startup.
Free GitHub-hosted native work remains applicable under the chosen budget.
All six workflow runs and 12 allocated job rows are terminal.
Ten rows use actual Depot runners;
the two preserved Windows ARM rows use GitHub.
No application source was fixed,
no production workflow routing was changed,
and no subscription or account-wide usage setting was inspected.

The [first wave](../audit/depot-native-runner-first-wave-2026-10-18.evidence.json),
[recovery wave](../audit/depot-native-recovery-wave-2026-10-18.evidence.json)
and [consumer reconciliation](../audit/depot-native-consumers-and-variation-2026-10-18.evidence.json)
retain source/common pairs,
Git-verified input hashes,
provider artifact digests,
per-step logs and individual nextest identities.
Private validator assertions were corrected after reading the frozen harnesses:
`caseName` is the `PROVENANCE_CASE` execution profile (`native` or `windows`),
while `commonManifest.caseId` identifies the frozen source case (`native` or `desktop`).
Both bindings and the exact source revision are checked independently.

- Native Mac,
  both repetitions:
  1382 executed identities,
  1356 passed,
  26 failed and one additional summary skip.
  Outcomes match the last GitHub native Mac retry.
- Desktop Mac,
  both repetitions:
  1433 executed identities,
  1406 passed,
  27 failed and one additional summary skip.
  Identities and outcomes match the GitHub reference.
- Native Windows:
  original release build and CLI consumer pass;
  test compilation retains the frozen source's `E0433` errors.
- Recovered desktop Windows:
  both Depot x64 variants and the GitHub ARM row each pass 1432 nextest identities,
  with one additional summary skip.
  Original PE,
  Bash runtime and launch consumers pass.
  Keep the failed original PowerShell file assertion beside its passing independent stream probes.

The [path measurements](../audit/depot-native-checkout-paths-2026-10-18.evidence.json)
establish a 260-character Depot harness fixture path,
versus 247 characters on GitHub ARM and 243 with the shorter Depot native harness path.
The long-path bootstrap fixes checkout at the original 260-character path.
The first failed allocations remain included in both label-proration estimates.

Mac hardware is heterogeneous:
the four probes report seven,
seven,
seven and nine CPUs,
all with `18253611008` memory bytes.
All native cache lookups miss and failed jobs skip post-save.
The desktop timing difference therefore cannot isolate a provider effect or warm-cache benefit.
The native Mac pair reports the same CPU count and memory but does not identify the physical CPU model.

The original recovery watcher session `46364` ended on local GitHub CLI TLS handshake timeouts;
the real provider runs remained active.
Read-only watch resumed in session `84033`,
which is now complete.
No workflow was redispatched for that transport failure.
All artifact collection and source/consumer checks are complete.

The native follow-up label-proration estimates total USD `3.231466666666667`
using GitHub job intervals,
or USD `3.089866666666667` using first-step-to-terminal intervals.
Neither is a provider invoice.
The [hybrid model](../audit/depot-native-hybrid-routing-model-2026-10-18.evidence.json)
uses the dated Bank of Canada observation `2026-10-16`,
CAD `1.4154` per USD.
For CI usage alone,
Developer plus overage is always cheaper than Startup at the published rates:
the extra USD 180 fee buys USD 108 more CI allowance.
Other product pools and native Mac access remain separate considerations.

The expanded report has the approved section list,
passes scoped Markdown checks and has verified local evidence links.
Its rendered output was inspected.
The [cleanup receipt](../audit/depot-native-worktree-cleanup-2026-10-18.evidence.json)
records removal of only this follow-up's disposable worktrees after clean-status,
ignored-file preview and stray Git-root checks.
The frozen branches remain available.
The [final verification receipt](../audit/depot-native-final-verification-2026-10-18.evidence.json)
binds the published report and native evidence bytes.

No native follow-up execution remains queued.
The earlier source-failure ownership,
historical resource gaps,
unexecuted surfaces and missing invoice remain explicit acceptance limits.
They are not a pending source-fix or migration queue.
No complete trial invoice or accepted complete-graph latency is claimed.

## Completed trial execution and remaining acceptance gaps

### Historical reopening of the native Depot runner evaluation

The user challenged the statement that macOS and Windows require another provider,
then instructed:
"So do the work!"
The statement was too broad and is withdrawn.
Standalone Depot CI has Linux sandboxes;
Depot separately offers Windows and macOS compute through GitHub Actions runners.
The [product-scope correction](../troubleshooting/depot-ci-versus-github-runner-platforms.md)
retains the fresh primary documentation and limits.

The real objective remains intentional GitHub/Depot routing under the USD $200 monthly ceiling,
with Developer preferred where practical and elapsed waiting valued at CAD $8.975 per hour.
The user did not ask for wholesale migration.
The original evaluation report did not finish the authorized native follow-up.

The authorized follow-up comprised these independently verifiable areas:

- Execute the missing native Windows and macOS consumers on Depot GitHub Actions runners.
  Preserve historical source revisions and original frozen harnesses;
  bind provider-specific follow-up harnesses and actual architectures separately.
- Reconcile original assertions,
  real artifact consumers,
  source/common provenance,
  all attempts and provider allocation for that follow-up.
- Measure unchanged repetitions where a performance comparison is claimed,
  and update the intentional-routing and combined-cost verdict from actual evidence.
- Complete the remaining authorized source-issue and evidence reconciliation;
  keep unrecoverable archived evidence distinct from unexecuted work.

The expanded evaluation and native terminal result record the completed follow-up and retained limits.

Account-wide access and subscription changes remain excluded.
The current Startup trial was already authorized for evaluation use.
A repository-scoped Actions runner query returned an empty list;
this does not establish failure of Depot's ephemeral runners.
An unrelated concurrent edit to `package/git-policy/cli.fuzz/bin/container.mjs` must be preserved.

The supplemental cohort is now dispatched as `native-runners-20261018T034256508Z`.
Its [bound manifest](../audit/depot-complete-cohort-native-runners-20261018T034256508Z.json)
preserves the historical case digests and binds the new provider-specific harness revisions.
The [dispatch and placement evidence](../audit/depot-native-runner-followup-2026-10-17.evidence.json)
retains all workflow bytes and initial provider job responses.
The original consumer step values are independently compared with the frozen harnesses;
only runner selection,
the native cache namespace and the added hardware probe differ.

- Native source:
  run `39094759518`,
  Depot Windows job `117578770519` and Depot macOS job `117578770694`.
- Desktop source on Depot macOS:
  run `39094764609`,
  job `117578785599`.
- Desktop Windows matrix:
  run `39094767696`,
  Depot GNU job `117578793896`,
  Depot MSVC x64 job `117578794032`,
  unchanged GitHub ARM job `117578794097`.

All six initial jobs are in progress.
All five Depot-labelled jobs have actual `depot-*` runner names.

The first desktop Mac route is terminal:
`1433` nextest cases run,
`1406` passed,
`27` failed and one summary skip,
plus the two existing Clippy `missing return` diagnostics.
Do not claim identity parity until the individual test names are compared.
Its hardware probe reports seven CPUs and `18253611008` memory bytes,
which differs from the published eight-CPU,
24-GB runner specification.
Record measured hardware separately from the priced label.

The first desktop Windows Depot rows stopped at harness checkout with `Filename too long`.
They receive no desktop consumer credit.
The original GitHub ARM row reached the existing PowerShell failure.
All logs and the ARM provenance archive are retained under
private `depot-native-followup-artifacts/` with provider digests checked.
The [long-path recovery](../troubleshooting/depot-windows-long-path-checkout.md)
uses new Windows harness `20b7e331ad5a4e29bd493aba1829fff5f07d27a8`.
Recovery cohort `native-recovery-20261018T035939491Z`
retains the original native harness for unchanged repetitions and all historical case digests.
The recovery [bound manifest](../audit/depot-complete-cohort-native-recovery-20261018T035939491Z.json)
is committed as `738e501267` and its published bytes are independently verified.
All recovery routes are dispatched:
native hosts run `39095637308`,
desktop Mac run `39095642563` and desktop Windows run `39095647113`.
The native harness remains `3e51a06848ba363a90d4ce30816760042153f349`.
The private `depot-native-long-path-publication.json`
and `depot-native-long-path-dispatches/` retain the authoritative clock and receipts.
Inspect those receipts before any repeated dispatch.

The first wave is fully terminal and command session `75523` has completed.
Its terminal receipts are under `depot-native-followup-dispatches/`.
The recovery `wait-native-long-path-terminals` task is active;
it writes terminal receipts under `depot-native-long-path-dispatches/`
and emits only terminal results.
Continue independent source-issue reconciliation while those inputs remain unchanged.
Then collect actual logs and archives,
validate hardware,
source/common pairs and every original consumer boundary,
and retain all allocation intervals before measuring unchanged repetitions.

The user-approved reader report is complete:
[Depot complete CI trial evaluation](../audit/tech-depot-complete-ci-trial-vet-2026-10-17.md).
It uses the approved sections and makes no adoption recommendation.
Its [final gate and interval registry](../audit/depot-final-gates-and-intervals-2026-10-17.evidence.json)
joins all 770 primary gates and 33 external gates to explicit outcomes.
The primary totals are 654 passes,
115 failures and one prerequisite-blocked skip.
The coherent GitHub accounting includes 21 executed job intervals and four skipped analysis jobs,
with the native macOS repeat counted and the carried-forward Windows alias excluded.
The common request reaches the final provider terminal after 5851263 milliseconds.
This is an observed terminal frontier,
not successful complete-graph latency or a reconstructed dependency critical path.

The report and handover pass targeted Markdown lint and actual CommonMark rendering.
An independent checker verifies the frozen gate identity sets,
the provider interval sums,
the alias exclusion and every internal report link.
The rendered report was read in full.
The repaired public-output digest schema is tracked in closed issue
[#1765](https://github.com/Aquaticats/Monochromatic/issues/1765)
with exact body readback.
Its resolution applies to the 105 retained primary failure outputs,
not an unperformed audit of older artifacts.

### Logger terminal reconciliation checkpoint

The [logger cohort evidence](../audit/depot-coherent-logger-2026-10-17.evidence.json)
retains all generic primary observations,
source/common metadata joins,
the actual final guard logs,
and the completed coherent Depot allocation ledger.
All 719 logger primary check IDs have terminal records with frozen task/argument identities.
Retained outputs have independently verified hashes.
They record 613 passes,
105 failures,
and one check skipped because its prerequisite build failed.
Only 199 logical checks have an observation with both source and cohort completion pairs.
The generic footer declarations are not final acceptance.
All 650 configured source-input files were independently hashed against frozen Git blobs.
The [primary failure evidence](../audit/depot-logger-primary-failures-2026-10-17.evidence.json)
now retains the text of all 105 failed-check outputs
with original byte counts and hashes independently verified across 10496598 bytes.
Personal home paths are redacted from public text;
the retained text has separate byte counts and hashes.
It includes parsed lint/type candidates and the reviewed tracker comments.
Issue mapping remains incomplete;
deliberately failing child controls are not terminal parent-suite failures.

Independent browser reconciliation now joins all 32 cells to the unchanged full and partition inventories.
All 118 authored cases have result records:
116 passed,
the known WebKit OPFS availability assertion failed,
and the authored WebKit fs-path case skipped.
The configured effective retries and actual retry indices are zero.
All browser cells have both source and common-cohort pairs.
Removing a selected case fails the independent inventory comparison.

Nine fresh TypeScript native mutation reports now independently verify 5542 selected identities
and their exact frozen-source spans across 117 files.
They separately retain 77 ignored entries and declare 863 shards.
The actual ZIP directories were relisted and matched the retained member inventories.
No real physical shard manifest or report files are present;
the similarly named private fixture manifests receive no shard credit.
Physical baseline/report/return reconciliation remains unestablished.
The distinct physical-shard retention gap is filed as `#1434` with exact body readback.
A read-only probe of the known completed logger TOML sandbox returned
`sandbox "gbwhw0dz0jzdgd6n346m" was not found`;
it recovered no temporary shard files.
The Rust native verification format is separate.
Its original mutation task stops at the parent Rust gate:
1806 library tests pass,
six fail,
and four are ignored.
The retained manifest declares two CPUs,
two GiB,
128 PIDs and no network.
The later mutation phase receives no execution credit.
Deliberately failing child controls are not additional parent-suite failures.
JSONC and file-enforcer mutation tasks failed without native JSON.
The logger TOML selection exactly matches the 3062-ID source fixture;
removing an ID fails the comparison.

The unchanged conf mutation task passes this cohort with 894 selected IDs:
405 killed,
92 survived,
four timed out,
393 failed compilation,
and no reported runtime error.
Its older red baseline in `#1156` remains a separate historical observation without an established fix.
Aggregate killed status receives no assertion credit.

Custom consumer reconciliation now verifies both source/common pairs and retained outputs.
Both Electron Wayland tasks pass through the private software compositor.
WireGuard route integration passes;
bypass reaches the known reserved-mark teardown failure `#1077`.
The Pi fixture loads all 22 configured extensions and no fixture extensions.
The original serializer assertion fails as `#1078`;
terminal answer/cancellation,
notification delivery and the offline tmux consumer pass.
The root Oxlint gate fails after its original full command.

The logger desktop consumer independently matches the 118-case authored inventory.
It observes 112 original checks:
110 pass,
the known folder-dialog assertion fails,
and one authored case skips.
The continued wide-session abort is a separate synthetic failure.
Six original wide-session assertions remain unexecuted.
All 17 retained session logs match their archived hashes and compositor calls.
The resource samples show zero recorded OOM and PID-limit rejections.
Removing an observed case changes the missing-case count from six to seven.

The combined route has 97 terminal jobs,
96 collected archives,
and all 97 retained attempt logs.
There are 94 source pairs and 93 cohort pairs.
Static and unit jobs rebuild tracked plugin bundle outputs;
the static job also changes the tracked IntelliJ JAR.
Their after-source and after-cohort guards correctly reject the resulting tracked differences.
These observations do not establish a change to authored task definitions.
The native TOML mutation task passed and retained a native report with verified IDs and source spans.
Its reported 462 physical shards remain unverified because their files are not retained in the archive.
Its local source pair is available,
but the later authenticated common-manifest lookup reports `gh: Bad credentials (HTTP 401)`.
The credential rejection cause is not established.
This is separate from the TOML case final guard whose diagnostic is unavailable.

Logger generated-consistency job `vkv8rwn4l1`,
attempt `5d611nw71d`,
ended with `Sandbox terminated before worker reported completion`.
It published no archive.
Its raw log retains exit-zero root generation and one sidecar command,
then a sidecar container adapter receiving `SIGKILL`.
The external driver and final source guard later report shell exit code 137.
These partial observations do not recover the full generated-difference evidence or final bindings.
Its declared runner was `16x64`;
the provider retains 133 memory samples and a peak memory-utilization value of `0.9975022728697344`.
That correlation alone does not establish the termination cause.
Do not retry this resource-consuming job without first reconciling its diagnostics and memory evidence.

The [repository accounting checkpoint](../audit/depot-repository-accounting-2026-10-17.evidence.json)
now retains the complete scoped terminal inventory and precise metrics for its attempts.
The [metadata troubleshooting record](../troubleshooting/depot-ci-workflow-metadata-accounting.md)
documents the endpoint precision and no-sandbox boundaries.
The 391 provider attempt records include 389 observed start-to-finish intervals
and two successful `no_sandbox` records without a start time.
Those two records receive no observed allocation interval or assumed invoice charge.
The workflow metadata timestamps have whole-second precision;
all observed intervals instead use the metrics endpoint's timestamps.
The 283 prior attempt metric records and 108 coherent records reconcile without collection errors.

All 108 coherent Depot attempt intervals are now closed.
Their 291275.192 allocated vCPU-seconds prorate to $14.5637596 at the published CI rate.
This is the coherent Depot compute subtotal.
Historical resource candidates produce a separate provisional compute proration of $47.2234074
across 263 observed intervals.
Exact historical uploaded workflow bytes are not established;
18 other allocated attempts remain unpriced because their resource sizes are unbound.
Do not present the provisional figure as verified historical spend.
The union of all observed attempt intervals is 28061005 milliseconds,
with peak concurrency of 108 attempts.
That union accounts for overlap;
it is not a reconstructed dependency critical path.

Current Depot documentation identifies the included-minute base as two CPUs and eight GB.
The coherent vCPU allocation equals 2427.2932666666666 smallest-sandbox minute equivalents.
No allowance netting was applied.
GitHub/local consumer allocation,
storage,
plan allowances,
and invoice reconciliation remain separate.

The reviewed comments for `#1160`,
`#1156`,
`#1052` and `#1131` were published on a later paced retry with exact body readback.
`#1160` is closed for the recovered aggregate native JSON.
The separate physical-shard retention issue `#1434` remains open.
The older conf red baseline in `#1156` is preserved without a fix claim.

Acceptance gaps and follow-up boundaries:

- Logger credential issue `#1180`,
  rebuilt tracked-output issue `#1181`,
  and terminated-worker issue `#1182` are filed with exact body readback.
  The [completion troubleshooting record](../troubleshooting/depot-logger-final-completion-gaps.md)
  keeps their boundaries separate.
- Reconcile source failures and prerequisite-blocked checks with existing issue threads.
- File scoped follow-up evidence for the repaired private verifier assumptions:
  zero-based native columns,
  separate ignored records,
  Rust's distinct archive format,
  browser versus DOM report roots,
  and captured desktop output versus provider progress logs.
- Finish the external-check registry and overlap-aware allocation/critical paths,
  including the already retained native macOS retry and Windows carry-forward alias.
- The repository-filtered Depot index contains 276 terminal records from the trial period.
  Every returned repository is `Aquaticats/Monochromatic`.
  The CLI paginates until the provider's next-page token is empty;
  the requested limit of 1000 was not reached.
  The accounting checkpoint joins all attempt metrics and retains verified or provisional resource bindings.
  The oldest indexed record has no workflow,
  so a run record alone proves no allocated CI job.
- File or add scoped evidence for the repaired private evidence-construction findings.
- Check and render the updated documents and inspect their actual output.
- The reader report is written and verified in the approved structure.
  Missing assertions,
  physical shard receipts,
  complete provenance and billing remain explicit acceptance conditions.
  The report does not authorize changing the frozen cohort,
  retrying resource-consuming jobs,
  source fixes or adoption.

The native desktop inventory has 114 authored cases,
108 observed original cases and six unexecuted originals.
Its original outcomes are 106 passes,
one failure and one authored skip.
The separate synthetic abort makes 109 total outcome records;
it does not replace any missing original assertion.
The desktop-case inventory similarly has 112 observed originals plus one synthetic abort,
or 113 outcome records for 118 authored cases with six unexecuted originals.
These counts were rechecked directly against the retained authored inventories and outcome arrays.

### Final guard and unchanged macOS recovery checkpoint

The [final boundary evidence](../audit/depot-coherent-final-boundaries-2026-10-17.evidence.json)
retains the verified macOS retry,
TOML final-guard diagnostics and independent log probes,
the incomplete allocation ledger,
and current primary pricing documents fetched through the saved global Codex MCP.

Native macOS attempt two retained a valid common/source before and after pair
and published provider-digest-matched artifact `12101426954`.
All 1382 observed identities match the first attempt.
The retry recorded 1356 passes and 26 known failures,
with one summary skip whose identity is not printed.
The decode-cache case in `#1178` passed on this unchanged-source repeat;
its first failure and open issue remain.
Transport incidents `#1175` and `#1176` remain separate and open.
The second-attempt Windows job ID aliases the original execution:
its complete step records and times are identical,
its artifact is the original,
and the retry log archive contains only macOS.
Do not count another Windows allocation.

The TOML route is terminal and all five artifacts are collected.
Its native mutation source task passed with verified bounded cleanup,
but the provider final source/cohort guard exited one.
Only its before record is archived.
Both finite retrieval paths returned 120742 records and 41239978 body bytes,
ending before the failed final guard diagnostic.
Lossless fractional-second normalization proves equal timestamp/body sequences,
and removing a record changes the hash.
Issue `#1179` remains open with unknown root cause and no paired-provenance completion credit.
All 24 TOML primary task terminals are now independently reconciled:
20 passed and four failed.
Eighty-five configured source files were hashed against frozen Git blobs.
The native report preserves all 3062 selected identities,
463 green-baseline shard reports,
all physical source returns,
and every frozen-source mutation span.
Its 37 selected full-suite test files match the prior same-source inventory.
No exact known spawn diagnostic was observed in the 41142381-byte source log;
the positive diagnostic control succeeds,
and killed status alone receives no assertion credit.
All 1269 metadata operations are retained with 2538 forwarding events.
Four nonzero inspections are expected already-removed objects;
the longest operation took 8926.082681 milliseconds under the 60000-millisecond metadata budget.
Root generated consistency failed after an exit-zero command changed disposable `mise.toml`.
The [complete TOML receipts](../audit/depot-coherent-toml-2026-10-17.evidence.json)
retain raw shard bytes,
the native report,
full native source output,
and every failed source-check log.
The missing final provenance pair still receives no completion credit.

All provider routes are now terminal.
Logger artifact collection and assertion reconciliation are running.
The retained allocation snapshot still has 107 closed Depot intervals and the then-active logger interval.
Its 268273.804 allocated vCPU-seconds prorate to $13.4136902 at the published CI rate.
This is an incomplete compute subtotal,
not an invoice or final trial cost.
The live pricing page names the $200 plan Startup.
Account-wide billing access remains outside scope.

Remaining independently verifiable work:

- Reconcile the completed TOML source failures with their issue threads.
- Collect and reconcile the terminal logger artifacts and authored assertions.
- Scope and file the repaired private macOS-verifier,
  timestamp-comparison,
  issue-readback,
  pricing-task quoting and JSONL-parsing construction findings.
- Add actual-cohort evidence to existing source and transport issues without closing unresolved failures.
- Finish all original and retry allocation intervals,
  storage limitations and overlap-aware critical paths.
- Reconcile the complete 770-primary and 33-external registry,
  preserving blocked and unexecuted branches.
- Await the already-asked reader-report outline answer before drafting that report.
- Check the changed Markdown and JSON and inspect their actual rendered output.

### Execution checkpoint on October 17

The Codex scope clarification and globally verified search/fetch MCP setup are complete.
The user selected the Codex document change only;
`AGENTS.md` remains unchanged.
The coherent provider replay is terminal;
artifact and assertion reconciliation continues.
Common cohort `complete-20261017T111555806Z` was published in `5a52255c8f10d2a2ab06ba50f125152d42227081`.
Authenticated Contents API decoding verified all 6191 exact bytes
and SHA256 `ec26a2edd105dc17fb7f27bbafbe3bd92e9d0f479901e9a5f8938a98b963fc50`
at `2026-10-17T11:16:24.579Z`,
before the shared request instant `2026-10-17T11:17:09.003Z`.
All six harness remotes were explicitly pushed and matched their frozen clean heads.
All 16 parallel routes and four sequential Scorecard cases have accepted dispatch receipts.
Each Scorecard dispatch followed the preceding case's terminal receipt.
All seven Depot run records independently confirm the bound harness SHA,
branch,
repository,
and `workflow_dispatch` event.
The [dispatch evidence](../audit/depot-complete-dispatch-2026-10-17.evidence.json)
retains commands,
inputs,
provider run IDs,
returned metadata,
and the actual dispatcher definitions.
A private driver argument mistake was rejected before any provider call;
the accepted path passes the selected route explicitly through the child environment.
All 20 routes now have accepted dispatch receipts.
Every Scorecard gate succeeded and each conditional analysis was skipped;
its exact observed rolling-window denial and paired records are retained.
The native feature profiles passed 58,
67,
and 71 tests with equal source and common before/after identities.
The original VM task reproduced source cwd failure `#1076`,
while its natural 502,093-byte bundle and isolated CLI consumer passed.
Both original Linux and Windows guest lifecycles remain unexecuted.
All four logger host runner labels and applicable original command outcomes are verified
against distinct provider-digest-matched archives and equal source/common identities.
Issue `#1168` is closed after actual `macos-15` verification.
The corrected dispatcher argument,
Android artifact filename,
and NOCOW cold-copy preparer incidents are filed and closed as `#1169`,
`#1170`,
and `#1171`.
The [verified closure receipts](../audit/depot-coherent-incident-closures-2026-10-17.evidence.json)
retain exact issue bodies and the logger closing comment.
The common-bound Android APK pair passed the held input validator.
Private AVD preparation then failed before launch on NOCOW snapshot/transient reflinks.
The filesystem controls and source trace are retained in
[the cold-copy troubleshooting record](../troubleshooting/android-emulator-cold-boot-reflink-nocow.md).
The revised local prepare task excludes unused snapshot and adb transient state;
all other staged device task bodies remain unchanged.
Consumer attempt two completed with the same bound producer.
The private AVD copy and metadata check passed,
and all 15 original instrumented assertions ran:
13 passed,
two `SessionStoreTest` cases failed with the null-context symptom in `#1002`,
and none skipped or missing.
The adb instrumentation command exited zero;
the aggregate task correctly failed.
Guest API `37`,
ABI `x86_64`,
page size `16384`,
and font scale `1.0` were measured before execution;
forced density and size were unset.
Graceful shutdown passed,
the unit is inactive,
the private adb server is stopped,
and all three private sockets are closed.
Owned AVD ini/config hashes remain unchanged.
The [actual Android evidence](../audit/depot-coherent-android-2026-10-17.evidence.json)
retains producer joins,
APK digests,
complete raw instrumentation,
all device lifecycle steps,
and the missing-assertion positive control.
The [early cohort evidence](../audit/depot-coherent-early-results-2026-10-17.evidence.json)
retains these boundaries and full structured receipts.

The actual native and desktop producer archives are independently verified.
All active primary checks have terminal records and their check-log byte counts and hashes match.
Source and common before/after identities match the frozen publication and provider attempt joins.
The native core wave passed five checks and failed Clippy plus changed-Rust style.
The native desktop and desktop-source Linux waves each passed their four package checks
and failed changed-Rust style.
The native style execution appears in both waves but counts once logically.
Both runtime consumers reproduced the folder setup and system-dialog failures in `#1005` and `#1004`.
Their original observed assertion counts are 108 of 114 and 112 of 118;
each still lacks the same six wide-session assertions.
The synthetic session-abort outcome is separate from authored assertions.
All 16 native and 17 desktop raw session logs match their archive manifests,
provider archive events,
and exact source session selectors.
Cleanup and all retained PID/OOM counters pass.
The exact changed native Cargo file passed formatting.
The desktop Cargo check ran and failed the documented TOML 1.1 inline-table grammar gap,
while its build,
Clippy,
and nextest passed against the same unchanged file.
The [actual native and desktop evidence](../audit/depot-coherent-native-desktop-2026-10-17.evidence.json)
retains full formatter records,
source/provider joins,
raw session logs,
and authored-inventory sensitivity controls.
Per-attempt provider allocation remains pending reconciliation.

Formatter harness issues `#1166` and `#1167` are now closed with live command and unchanged-source evidence.
Desktop grammar mismatch `#1172` remains open as a scoped follow-up to `#562`.
All four scanner gates passed with paired common/source artifacts.
Each Scorecard gate observed eight analyses already executed in the preceding day
and conditionally skipped its analysis job;
the exact timestamped notices and paired source/common records are durably retained.
Both Alpine architecture jobs succeeded.
The original PowerShell file assertion failed on all three Windows variants,
while their independent merged-stream probes captured the expected version.
Every uncaptured-console provider step independently printed the exact expected version.
Native host Windows and desktop macOS retain paired provenance,
and their nextest failures are reconciled against exact observed identities.
Native host macOS lost its after-provenance record to Node `UND_ERR_CONNECT_TIMEOUT` at `api.github.com:443`.
Its artifact action separately exhausted five CreateArtifact request attempts.
These are distinct transport incidents;
a shared cause is not established.
Only native macOS was rerun in run `39028181199` with the unchanged frozen harness and inputs.
Its second-attempt provenance and artifact publication are now independently verified.
The original failed attempt remains retained and receives no paired-provenance completion credit.
The retry and its orchestration gap must remain in provider allocation and critical-path accounting.
The transport and changed-Rust source-style incidents are filed.
Next actions are to retain and reconcile remaining provider terminals,
retain and reconcile all host consumer assertions,
collect the pending logger terminal and reconcile the completed TOML evidence,
and finish overlap-aware accounting.

The [GitHub gate checkpoint](../audit/depot-coherent-github-gates-2026-10-17.evidence.json)
retains all scanner/Scorecard source/common pairs,
initial host/platform outcomes,
the two distinct native macOS transport logs,
and unchanged-harness retry receipt.
It also retains exact changed-Rust diagnostics:
native has 19 semantic-line-break errors and 13 anonymous-function warnings;
desktop has 230 semantic-line-break errors,
33 missing-rustdoc errors,
and 15 anonymous-function warnings.
Their source scopes and issue reconciliation remain distinct.
Scoped document checks pass with source-snippet warnings retained.
The configured dprint Markdown plugin is disabled,
so dprint verification covers JSON artifacts only;
Markdown is checked by monochromatic-lint and CommonMark rendering.

The [host consumer evidence](../audit/depot-coherent-host-consumers-2026-10-17.evidence.json)
retains every observed nextest identity,
source failures,
all Windows independent version receipts,
and Alpine smoke logs and screenshot hashes.
Each Windows variant passed 1432 nextest tests and reported one source skip.
Each Alpine architecture passed 1461 nextest tests and reported one source skip.
All three Alpine audio/window scenarios and the PipeWire null-sink link passed on each architecture.
Desktop macOS recorded 1406 passes and 27 failures;
native macOS first attempt recorded 1355 passes and 27 failures.
Both actual identity sets equal the earlier same-source macOS inventories,
with a dropped-identity positive control.
Their summaries report one skip without naming that skipped identity;
the source-declared ignored benchmark remains separately identified.
Native Windows is source-compile-blocked before test execution.
The native macOS first attempt adds an untracked
`measure::tests::two_sweeps_of_one_load_decode_a_track_only_once` failure:
the decode count was two instead of one.
Its separately verified issue `#1178` is open without a cause claim;
the unchanged-source retry passed that case and retains the first-attempt failure without a cause claim.
Native and desktop style findings are open as `#1173` and `#1174`.
The separate macOS API and artifact transport incidents are open as `#1175` and `#1176`.
Private verifier record-classification repairs are filed and closed as `#1177`.
Source trace and executed historical-log retrieval are in
[the transport troubleshooting record](../troubleshooting/github-actions-native-macos-provenance-transports.md).

The four case manifests now have complete static registrations with `completeGraph: true`.
Their 24 committed copies are byte-identical across six isolated harnesses.
The registry binds 770 primary and 33 external checks to 20 routes.
Primary task definitions,
arguments,
selection closure,
and exclusions remain unchanged.
The logger semantic platform row now includes the authored macOS requirement,
and TOML now explicitly records the selected current Scorecard policy.
Execution acceptance remains pending.
The [static registration evidence](../audit/depot-complete-static-registration-2026-10-17.evidence.json)
retains each case digest,
harness revision,
route,
and actual helper-selection proof.
Root is `3d976fe06d22ae01c54aa206cf021a78374a3b06`;
native hosts `467dc0002d6080f8aa951f488430450debf3df3c`;
Windows `4f7ff1a739976a3b781e6056dd3b71b84f1e082f`;
Alpine `16ee27b3d6b115baa5f1bb4046d52d01057172cd`;
TOML `9cc7f28fb8db201d9774ff7cf5404ebec3ffc260`;
and VM `0fbf23d77e5a9b616763990845e36dc2c453f6d6`.
Verify clean remote equality before each actual dispatch.

Desktop continuation controls at harness `bb475f636f51a00864710454c6499fcd3db6b8a4`
are terminal and fully collected:
native `s3cmrhf2kp` recorded 106 passes,
two failures,
and one skip;
desktop `wxq11xm2hd` recorded 110 passes,
two failures,
and one skip.
Actual summary identities reach every source-selected route,
including the desktop source's four Auto-play checks.
The failures remain the source folder setup in `#1005`
and the zero-pixel folder chooser boundary already tracked in `#1004`.
The synthetic `trial/session-aborted/wide` outcome preserves the setup error;
checks after that setup failure within the wide session remain source-blocked.

The actual authored inventory has 114 native and 118 desktop assertion identities,
including each source's authored skip.
The retained summaries contain 108 and 112 original identities respectively;
synthetic continuation failures are separate.
The same 6 wide-session assertions remain unexecuted in both cases:

- `click-through/search-wide`.
- `click-through/toast-wide`.
- `lists/picker-follows-search-result`.
- `lists/track-list-keeps-place-across-breakpoint`.
- `layout/breakpoint-599`.
- `layout/breakpoint-600`.

The actual source inventory functions were exercised against committed source bytes.
Dropping a known observed assertion increases the missing count from six to seven.
`desktopRuntimeAuthoredAssertionInventory` retains all input hashes and the positive control.

All retained PID/OOM counters are zero,
source identities match before and after,
and cleanup is verified.
Provider allocations were 603.265 seconds and 575.693 seconds at `16x64`.
These development allocations do not constitute a coherent whole-case latency measurement.
The complete receipts are in the `desktopRuntimeSessionContinuationControl` evidence field.

The source controller writes its summary and then removes its scratch base.
The historical `bb475f636f51a00864710454c6499fcd3db6b8a4` artifacts therefore lack raw session logs:
the post-command collector runs after that removal.
Those historical artifacts retain their summaries and screenshots without retrospective log credit.
Trial archive implementation `b6b44d679853c29e39a3c801b64ca66a32d30a97`
copies regular session logs before the original source removal,
verifies their bytes and SHA256,
and keeps the original removal in `finally`.
Its generated hook passed 16 archive controls across the source cases,
including copy failures,
quoted/newline paths,
symlink rejection,
source-anchor failures,
and a removed-cleanup control that the actual post-run probe rejects.
The updated hook also passed 17 continuation/safety controls per source case.
The missing completed-run logs are separately filed as `#1133`.
The clean pushed final root worktree is
`~/temp/agent/depot-cohort-root-worktree-20261017`,
branch `trial/depot-cohort-root-20261017`.
Its `f8d090f36dbbbba5dee6e96a732a7da0cd2bc73e` preparation derives the receipt inventory
from each committed source selector.
Actual function controls accept six native and seven desktop routes;
the legacy six-route desktop negative fails.
Both archive/inventory controls were dispatched from clean,
explicitly pushed and remote-verified harness `2294b7f1171c2cd88d0ca50dd148404a3a0db4b6`:
native `w21t2hzv67` and desktop `d83q9lrgfx`.
Both are terminal and fully collected.
Native retains 16 raw log files totaling 2,022,505 bytes;
desktop retains 17 totaling 2,128,752 bytes,
including `auto-play.log`.
Every copied file matches its archive manifest's byte count and SHA256.
The provider's archive event matches that manifest,
and the post-run probe verifies removal of the original scratch directory.
Every raw log contains the actual headless software-compositor invocation.
The receipts match the committed source selectors' six native and seven desktop sessions.
Source identities match before and after,
all sampled PID/OOM counters remain zero,
and cleanup is verified.
Counts remain 106 passes/two failures/one skip and 110 passes/two failures/one skip respectively.
The authored-inventory comparison still finds the same six unexecuted wide-session assertions per case.
These controls resolve the trial archive and inventory acceptance boundaries in `#1133` and `#1120`;
they do not resolve the source failures or establish a passing whole-case replay.
Issues `#1133`,
`#1120`,
and `#1110` are closed after fresh comment-body and state verification.
The same evidence was added to open source issues `#1004`,
`#1005`,
and `#1006`.
Provider allocations were 521.934 seconds and 388.986 seconds at `16x64`.
`desktopRuntimeSessionLogArchiveControl` retains the actual proof,
including raw-log hashes,
source-selected inventories,
screenshots,
resource samples,
and complete compressed inspection receipts.
The unused source-compositor build step was removed from the new diagnostic and final Linux routes.
Historical allocations remain retained without adjustment.

The canonical common-manifest contract is in the plan's shared cohort identity section.
`#1119` tracks byte-preserving transport through every final route.
The authenticated object/base64 API control verifies exact bytes at a frozen repository ref;
raw JSON from GitHub CLI 2.102.0 changes escaped controls through its sanitizing transport.
`doc/troubleshooting/gh-api-json-byte-preservation.md` records the deciding source and verified workaround.
Root adapted the native owner's guarded reader from
`0f8ad795aa92d3beb83e3bfa4e4aa54f1bea06f8`
to trusted native,
desktop Linux,
scanner,
and Scorecard route bindings.
The actual root task at `f0748cf5ad8b0236b5f06696de7a5b323abf1e11`
passed 48 valid and 312 rejected binding controls across 12 routes.
Removing its actual digest guard produced 12 invalid acceptances.
Those fixtures stub only the absent canonical Contents API response;
they do not establish provider acceptance of an actual final common manifest.
The current guard body equals the tested body.
Final Linux workflow bindings at `afbaaaccd6f87ff968419bd518c2511155fcdad8`
passed scoped formatting and before/after gate checks.
Scanner integration at `6889dadf4e` reuses the actual prepared policy revision
`32a9250addccf6b902246d39f4756f5f12ca9603`.
The initially selected `409e6` reference lacked the required manual inputs;
the preparation rejected it before writing.
Scorecard integration at `ccc414df2135290d123e0653edc2d06933274a3a`
reuses the existing conditional live-default-branch adapter and its unchanged quota gate.
Both workflows now require the common inputs and a selected case/source/base,
with paired source/cohort guards.
Root `c51a5c94e8dd92db9dbcec0a12ff3d285daa8217` passes structured workflow controls:
the selected scanner command bodies and Scorecard quota script are byte-identical after YAML parsing,
every job has paired guards and required inputs,
and the Scorecard adapter differs only in its standard task shell and runtime cohort metadata.
Actual adapter invocations reject missing tokens and wrong repositories before network or container work.
Scoped formatting passes.
Dispatch the Scorecard cases to terminal state sequentially so GitHub's pending-run replacement cannot cancel a case.
The retained proof fields are `rootCommonCohortPreparation`
and `rootGitHubWorkflowPreparation`.
The verified local GitHub-expression template collision is added to the existing Scorecard troubleshooting document.

Root `ae3aad924c7b74c83d27ad0f286832008195eebf` integrates the logger candidate from
`673ab92e49885dd61fcf08e168ed9c59bf29c6be`:
97 expanded jobs,
with 57 disjoint remainder partitions covering 155 checks,
plus the existing producers and 32 browser partitions.
Its 155 remainder identities and 564 other active identities exactly cover the 719-check graph.
The root's verified native,
desktop,
and common-identity task bodies remain preserved.
The combined workflow uses its separate logger cohort helper.
The old helper accepts a committed `completeGraph: false` case;
the strengthened root helper rejects it,
accepts valid before/after fixtures,
and rejects 21 other invalid binding/input/harness controls.
Removing the complete-case predicate restores invalid acceptance.
Distinct issue `#1141` is filed and verified closed.
`rootLoggerIntegrationPreparation` retains this fixture proof separately from actual provider acceptance.
Its `desktopSourceCompatibility` field also retains fresh logger-source controls:
the actual selector has seven sessions including Auto-play,
the original controller loop passes 17 continuation/safety controls,
and the archive passes eight controls.
The real selector function rejects the legacy six-session receipt.
The logger controller and selector bytes match the desktop case;
this does not claim that the logger source's application assertions have executed.
The authentic linter pilot `q10910cxfm` completes both formerly unmet Markdown prerequisites;
its six remaining library assertion failures match the native owner's mapping and have separate issues
`#1135` through `#1140`.
The generator pilot `vt8vzj154t` failed before source execution because its development environment used
`DEPOT_LOGGER_EXTERNAL_GROUP` while the adapter reads `DEPOT_LOGGER_EXTERNAL`.
The combined candidate already uses the correct variable.
The next pilot exposed a guard-model defect:
the frozen pnpm workspace enables `dedupeDirectDeps`,
so valid consumer links can resolve from the consumer's ancestor workspace root.
Root `ea0306a772` takes the owner's controlled ancestor-resolution helper from
`8b64f790134bf0cc4485a9842071760cbc355f08`.
The real retry `ctg9148f4t` invokes all three original generators in separate consumers.
The declared-type generator then stops at an unmet rootful container prerequisite before corpus execution.
The owner is adapting the existing validated container forwarder and replaying only that unfinished invocation.
Take only the resulting used-helper changes before the final root freeze.
These are development results,
not final cohort coverage.

The VM artifact-consumer route is prepared at `8719ad3db0167574a5a8324132fa9a0bb74fd098`.
The owner's actual 49 controls pass,
including artifact,
default rootful,
and cohort controls.
The original task still fails `#1076`,
and both lifecycle identities remain unexecuted.
The route is statically ready;
the separate `#1134` cleanup incident is repaired and closed with retained evidence.
No additional guest or emulator has launched.

Root `7d487813bfe559b0fa5299e36aba010d71439b34` prepares `.github/workflows/fs-id.yml`
as the manually bound `logger.hosts` route for the original Linux,
macOS,
and Windows matrix.
It retains the actual fs-id command and Node26.5.0 setup,
adds both original semantic-bridge consumer tasks,
and derives Node26.11.1/pnpm12.11.2 from the immutable source lock.
Root `bbb40c740f` formats the workflow.
The portable source guard derives from `2032975cbad924e63ac98bc69e14a56b20288275`;
the stronger logger cohort helper now admits only each route's registered workflow,
including `logger.hosts`.
The existing combined-route fixture suite still passes two valid and 22 rejection controls,
with the expected legacy and complete-case guard-removal invalid acceptances.
Structured workflow/result controls preserve the original commands and matrix,
accept a complete result,
reject seven failed/skipped/wrong-runtime outcomes,
and detect invalid acceptance after removing the result failure gate.
The initial generator expanded its own literal Mise template before writing the three helper definitions,
embedding a local absolute scratch path in every task directory.
This is a generator defect,
not an inherited-context defect in the native owner's portable guard.
Root `bcc683bd85` first bound source-case reads and harness Git operations to explicit `GITHUB_WORKSPACE/harness` paths.
Root `d05781e816` preserves the portable task directory template in all three definitions;
the preparer now constructs template delimiters at runtime.
Actual Mise probes reach the correct harness directory for every helper,
while the three legacy definitions reach the wrong directory.
The repaired root passes four host/source acceptances and nine rejection controls,
and the workflow/result controls still pass.
The source and lock prerequisite prefix passes without installing dependencies or running the source build.
An isolated development-only host workflow is prepared on `trial/depot-logger-host-development-20261017`.
It has no common-manifest inputs or cohort API steps,
retains exact source checks,
and marks receipts as development-only with no cohort binding.
Its first dispatch at `c6aa8b698c` was rejected before jobs started:
GitHub reported HTTP 422 for `runner.temp` in the job environment.
Root `81d751db414445c281aebbdc2eaf2735434799c6` moves the final route's temporary paths into consuming steps.
The corrected development-only pilot at `a4a8e2501c02d590056a9f53991025f8aa93e5af` is accepted as
GitHub run `39023040772`.
All three original OS jobs are terminal with success.
All original commands passed on Linux,
macOS,
and Windows;
the three artifact archives match GitHub's SHA256 digests.
Exact source identities match before and after the commands.
The source guard records Node `26.11.1` before the original fs-id setup
and Node `26.5.0` after it.
The fs-id command retains its original Node `26.5.0` runtime.
The development receipt is retained in `loggerOriginalHostDevelopmentAcceptance`
and its compressed evidence archive.
See `doc/troubleshooting/github-actions-job-env-runner-context.md` for the existing verified scope rule.
Generator incident `#1144` is closed with the exact repair/control body and state re-fetched.
The separately filed parser incident `#1164` is also closed after exact body and state verification.
No actual final common manifest has been published.

The logger owner's bounded development recovery `lt7qwlz6h1` is terminal with all three artifacts downloaded.
All three original task terminal records are verified:
conf-fork fuzz and TOML mutation pass;
conf mutation fails on one red baseline in `src/conf.cache.unit.test.ts`.
No task remains incomplete.
The historical run retained exact logs but omitted native per-mutant report files;
that loss remains tracked by `#1160` without retrospective per-mutant credit.
Authentic bounded source control `knsghczl71` verifies fresh native JSON retention.
Root `ce052bc29c4df4f801fa2d5bf5016b6614411f01`
integrates the exact controlled collector and native-verification artifact path.
Only reports changed since the worker's before-snapshot receive collection credit.
That bounded control does not establish whole-check mutation coverage.
Root `d30d88df0de27e7af86680dfe220c4ba431b3a02`
also integrates the generated-consumer cleanup verified by provider control `p3jlkgvl99`.
The stronger root cohort guard,
generated identity guard,
and non-generated external branches are preserved.
The long development recovery uses clean pushed harness `f69a78a3b31f81cbfe47496bcdc6e2ac75549777`.
Only conf fuzz and TOML/conf mutation are replayed,
with unchanged source arguments,
two-hour check budgets,
and 150-minute jobs.
Fuzz uses `2x8`;
mutation uses `4x16` for the authored two concurrent `2 CPU/2 GiB` containers.
The prior 35-leaf recovery has 26 passing,
six source-failed,
and three incomplete checks.
The independent browser development union covers all 118 configured cases:
116 passed,
one failed on OPFS availability in WebKit,
and one authored skip.
Its 32 partitions retain zero PID rejection and OOM samples.
The real Git executable pilot `10zmqch10h`
passed its upstream build,
private installation,
actual commit,
and executing-ELF checks in a bounded disposable worker-userland overlay.
That pilot's Markdown tests stop at the separate unset `GIT_POLICY_NATIVE_TEST_LINTER` prerequisite.
The later authentic-linter control `q10910cxfm` reaches both Markdown assertions successfully.
Its Git CLI suite records 137 passes and one ignore;
the library suite records 1,806 passes,
six failures,
and four ignores.
The six source failures remain separately filed as `#1135` through `#1140`.
The logger owner continues prerequisite recovery,
issue reconciliation,
and final registration.
The browser OPFS failure is reconciled with `#1052`;
the distinct missing CI-environment propagation is filed as `#1131` without a source policy change.

The TOML observer incident is separately filed as `#1130`.
The source pilot exits zero with a green baseline and one retained compile-error mutant,
but an eight-second metadata command deadline failed the wrapper.
The bounded observer trial `vlrxjltfkq` at
`316e1a8a309e71e456a741eb63e5ff3b94b75646`
uses a 60-second metadata deadline with forwarding receipts and timeout controls.
The original source consumer helper remains byte-identical to `d098532a`.
That actual bounded control succeeded with all metadata,
correlation,
source outcome,
and cleanup proofs;
`#1130` is closed.
The full unchanged-source TOML mutation retry `23dk3nr90f` is terminal at
`0f1ef041a0de2ff6d2db6dd55210b1160c592648`.
All 3,062 identities match the frozen source spans and 463 green-baseline shard reports.
Native runner outcomes are 921 killed,
256 survived,
three timeout,
1,882 compileError,
and zero runtimeError.
These native labels do not imply that every killed outcome is an authored assertion failure.
The separate actual-source classifier control demonstrates that a missing test-spawn executable also returns killed.
Its caught-spawn diagnostic emitter is verified;
the full campaign contains no matching diagnostic at that specific branch.
The campaign's source arguments remain unchanged,
source identities and cleanup pass,
and four auxiliary inspect failures remain separately retained.
The native owner filed source classifier issue `#1142` and retained its actual-source control.
Harness recovery `#1113` is closed with all 24 historical task terminals reconciled:
20 passed and four failed.
The original canceled run remains incomplete,
and source failures `#1068` and `#1070` remain unresolved.
The prior failed campaigns and separate source-producer issue `#1118` remain retained.

The final logger combined workflow covers both browser external checks through its disjoint 32-cell matrix.
Its 97 expanded jobs include the 84 authored browser cases and 34 e2e cases.
Register `root.browser` and `root.e2e` under `logger.combined`;
the standalone `logger.browser` workflow is an alternative and must not be dispatched redundantly.
Preserve the source's actual zero retries and missing CI forwarding in `#1131`.
The Linux/Windows-only semantic-bridge manifest row was incorrect.
The frozen `.github/workflows/readonly-semantic-bridge.yml` includes `macos-15`
and runs both semantic source tasks there.
Its workflow blob is `a091f76946056c0e0920c98fa657219be7ec3f71`.
The separate fs-id workflow uses `macos-latest`.
The proposal to skip macOS semantic validation is withdrawn.
The final host adapter must preserve the four-label union and each original runtime.
Development run `39023040772` retains its actual successful commands and artifacts;
its `macos-latest` label does not establish the explicit `macos-15` binding.
The isolated correction is integrated through `1042e35209`,
including distinct artifacts for the two macOS labels.
Four valid fixtures,
70 rejections,
and a guard-removal sensitivity control pass under simulated platform metadata.
The artifact-name positive control reproduces the legacy alias and verifies four distinct revised names.
No development redispatch occurred.
Issue `#1168` remains open for actual final `macos-15` execution.

Root `6aefa5d651b8754cbd2d330704e5513cc1157650` adds the exact changed-Cargo-file formatter
to `native.producers` wave `native-core` and `desktop.linux`.
The actual Mise helper passes 20 control rows across native and desktop fixtures.
Both unselected-file positive controls expose failures that the selected-file adapter correctly excludes.
Malformed selected Cargo input is retained and fails terminal verification;
removing the source-revision guard restores invalid acceptance.
Source lockfiles remain unchanged.
These synthetic Cargo fixtures establish adapter behavior,
not either authored Cargo file's formatting outcome or actual final provider acceptance.
The [formatter evidence](../audit/depot-changed-cargo-format-controls-2026-10-17.evidence.json)
retains the helper,
control driver,
receipts,
and verified Mise install isolation.

The native host workflow's complete conditions at `2032975cba` omitted common-before success.
The execution consequence at that revision remains a source/documentation inference.
Held trial repair `8c1eac43273b0b0d468a54affd15dc9612d7fe49` requires it for build,
Clippy,
nextest,
and smoke while preserving independent source-after identity.
Development run `39025557306` actually rejected the provisional native case on macOS Intel:
all four consumers and common-after were skipped,
and ordinary source identity passed before and after with equal recorded lock hashes.
The separate artifact-upload failure remains retained because rejection produced no provenance directory.
[The guard audit](../audit/depot-common-manifest-route-guards-2026-10-17.md)
and [original provider archive](../audit/depot-native-common-before-negative-2026-10-17.evidence.zip)
retain full outcomes,
usage,
and scope.
Issue `#1165` is closed with the exact body and closing comment re-fetched.
This control gives no source-consumer execution or common-bound credit.
The independent native guard control remains development-only;
the actual common-bound replay has now started.

The remaining queue is explicit:
collect all bound routes to terminal state and chain the remaining Scorecard cases;
retain formatter incident closure and the completed terminal accounting;
retain the completed matching Android consumer and incident closures;
and compute allocation cost and overlap-aware critical paths.
The reader report section list requires confirmation under the source-audited-report skill.
The proposed sections are decision and acceptance limits;
immutable workloads and scope;
gate and artifact-consumer results;
reliability and source failures;
provider allocation,
cost,
and critical paths;
and adoption conditions and unresolved evidence.
The question tool has submitted this section list and the answer remains pending.
Continue technical execution independently;
do not draft the reader report before confirmation.

The VM owner's natural-build consumer bridge for source cwd failure `#1076` is statically ready,
while preserving the original authored gate's result and avoiding Windows fixtures.
The held Android join now accepts only `native.producers`,
the final native workflow,
and producer wave `native-android`.
Its 13 actual validator controls include a self-consistent wrong route,
wrong workflow,
and `native-core` rejection.
No emulator was launched;
matching final producer evidence remains required.
No final artifact consumer may use a mismatched development producer.

### Resumed execution on October 17

The user additionally authorized registering the repository's search/fetch MCP globally in Codex,
and suggested a shallow `openai/codex` source clone.
The global `search_fetch` registration is installed in the user's Codex configuration.
Fresh Codex 0.163.0 app-server clients discovered both tools and completed real search and fetch calls,
including a client launched outside this repository.
The registration enables `mcp_2026_07_28` and opts this stdio server into revision `2026-07-28`.
A temporary client override disabling the feature reproduced JSON-RPC `-32601` for `initialize`;
the saved global configuration remains enabled.
The setup is documented in `doc/troubleshooting/codex-search-fetch-mcp.md`;
scoped Markdown lint and actual rendered HTML inspection passed,
with the linter's existing `rust/require-rustdoc` warning on the quoted upstream Rust excerpt.
The user clarified that `.out-of-scope/codex-harness.md` describes no ongoing maintenance guarantee,
not a prohibition on Codex work.
The document now explicitly permits work authorized by the active task without a separate policy exception.
The user chose to keep this clarification only in the Codex scope document;
the proposed general `AUT` clarification was declined,
and `AGENTS.md` remains unchanged.

The existing Depot work remains active:
logger continuation `1wzr86fssk` failed before source checks in all 37 jobs,
at Podman's `cannot unmount /sys/kernel/debug: device or resource busy` user-namespace boundary.
The logger owner retained every failed attempt and crossed that boundary with a private owned tmpfs volume.
Cloud pilot `360d91ht6w` passed the unchanged mutation integration consumer and its build prerequisites.
Actual source-image metadata was measured as root-owned mode `0600`;
the wrapper binds a byte-identical private owned copy read-only without changing the source or image.
All six shard volumes and inspection containers were removed.
The logger owner is continuing repaired or unexecuted leaves and preparing per-file browser controls.
Browser and DOM recovery still require complete per-file and per-engine coverage.
The native owner completed platform route preparation and now owns the two missing TOML mutation gates,
including complete retained mutant identities and acknowledged timeout cleanup;
the VM owner completed source-identity controls and prepared the exact 15-identity Android consumer contract.
No new emulator launch has occurred;
the final matching producer receipt and APKs remain required.

The runtime-only diagnostics have finished and their artifacts are downloaded.
Both show PID-controller rejection counters increasing from zero to four inside the private user hierarchy,
with no memory OOM event.
The measured default service task limit is 76 and the user slice limit is 168,
inside the outer 512-task container budget.
Root retained the raw counter samples and provider metadata in
`doc/audit/depot-final-recovery-2026-10-17.evidence.json`.

Clean pushed harness `4c4063e78bc9aaed8e293c1b478db089d6eef0ce`
now configures private default-service limits of 256 and a user-slice limit of 448,
retaining the outer 512-task budget.
Actual consumer reruns `v0xj2z7q66` and `4xgjgr8gbr` are terminal.
Both crossed the thread-creation boundary with zero retained PID rejections and zero OOM events,
and both verified cleanup.
Each emitted 60 passing,
nine failing,
and one skipped wide-session assertions before an uncaught folder-setup `CheckFailureError`.
No later configured session executed and no summary file was produced.
Root retained all assertion identities,
raw counter samples,
artifact hashes,
and full provider allocation intervals in the `desktopRuntimePidRecovery` evidence field.
Issue `#1110` remains open while independent desktop coverage is recovered.

The prior harness explicitly supplied a source-built compositor to the E2E consumer.
Harness `9f4a01f699f09abb25f10258455ac0faed0ecae8`
installs the immutable lockfile's published compositor and leaves `E2E_COMPOSITOR` unset,
while preserving the same source-built software parent and resource bounds.
It retains the published ELF hash and version output in `native-tools.json`.
Controlled native run `xdlnfpwhzz` and desktop run `7x5lk9dx9q` are terminal and fully retained.
Both actual consumer logs select the published binary,
whose ELF SHA256 is `3ced1113988238dc40b9826265aed4c7f620db3f610971b6e47d9e24953ba6e3`.
Each emitted exactly the same 70 assertion identity/status pairs as its source-built control:
60 passing,
nine failing,
and one skipped.
All retained PID rejection and OOM counters remain zero,
and cleanup is verified.
Provider allocations were 506.565 seconds and 499.738 seconds,
each at 16 vCPUs and 64 GiB;
these are separate development attempts.
The evidence field `desktopRuntimePublishedCompositorControl` preserves their complete records.

The retained wide screenshots are 1920 by 1080,
although authored `WIDE_WINDOW` is 1000 by 800 and pointer targets use those fixed coordinates.
The source compositor's `handler/xdg_shell.rs` deliberately fullscreens children to its screen size
and ignores resize requests.
The additional fixed-size software parent therefore changes the display geometry.
Root's next consumer control will wrap the locked published binary with its supported
`--headless --gpu software` mode so each authored session owns its requested output size.
The wrapper passed real disposable version,
help,
argument-boundary,
child-exit,
missing-binary,
and removed-headless negative controls.
Clean pushed harness `107b968b957dc8cb47540df94edda02e26445749`
is now running native control `rfq31sq4g9` and desktop control `jq7ctm8pb2`.
Each dispatch independently pushed and checked remote equality and clean status.
Both workflows now retain before/after immutable source identity records.
Both controls are terminal and their artifacts are downloaded.
Each emitted 69 passing,
zero failing,
and one skipped assertion;
all nine prior failing identities are restored to passing.
All 20 retained screenshots per case have the authored 1000 by 800 dimensions.
Before/after source identities match,
all observed PID rejection and OOM counters remain zero,
and cleanup is verified.
The unchanged source still throws the folder-setup timeout before later sessions execute.
Existing issue `#1005` already records this exact 69-pass fixed-coordinate fixture boundary.
Root added an explicitly recorded in-memory session-continuation adapter:
retain an ordinary setup failure,
recheck the original preconditions after cleanup,
then run fresh independent sessions.
Safety-stop,
unknown,
and cleanup errors must still stop the run.
The adapter is committed and pushed at `bb475f636f51a00864710454c6499fcd3db6b8a4`.
Native control `s3cmrhf2kp` and desktop control `wxq11xm2hd` are dispatched at that exact SHA;
one terminal waiter owns both.
Each source version passed 17 actual-loop/recorder fixture controls,
including retained ordinary failures,
fatal safety/unknown/cleanup errors,
source-hash and anchor rejection,
unfiltered-only enforcement,
and a removed-type-guard negative control.
These are adapter controls,
not actual all-session coverage.

The native source declares six default session routes.
The desktop source additionally declares `auto-play`,
for seven routes.
Their `runSessions` function and recorder bytes match,
but their complete `run.ts` and `selection.ts` bytes do not.
The unfiltered authored selector remains authoritative and automatically includes `auto-play`.
The trial receipt's hardcoded six-name array is incomplete metadata and must be replaced in the final harness.
Preserve original receipts and reconstruct expected coverage from each actual immutable source.
The corrected fixture consumes each source's actual selector and exercises every returned route.
The initial no-hook fixture's `ReferenceError` and hash-preempted anchor control are retained as rejected controls;
the corrected controls exercise the intended `CheckFailureError` and anchor guards separately.
Existing issue `#1007` already tracks the nested-route pointer/text boundary;
the measured geometry belongs there rather than in a duplicate issue.
The control establishes an observed route workaround without proving a nested-protocol root cause.
The evidence field `desktopRuntimeHeadlessGeometryControl` retains the actual comparison and provider allocations.

Root grafted the truthful executor metadata repair while preserving native nextest continuation
at `100452c7c6b9078d01f20ab5a086f13edeaf49d9`.
The combined executor passed its actual disposable passing,
failure,
deferred-prerequisite,
and running-progress controls;
the legacy executor failed the running-progress negative control as expected.
The evidence field `combinedExecutorControls` retains all five controls and the graft receipt.
The native owner reconciled the timed-out TOML wave and filed its uncovered source and harness failures.
The missing mutation gate wave `p6vm5s00w6` at `274ec49595` is terminal.
File-enforcer retains its source failure.
The TOML native report has 3062 identities,
all `runtimeError`,
from 9037 shard attempts censored by `rsync` ownership rejection before source assertions.
The provider allocation is 2834043 ms across both jobs at 16 CPUs and 64 GiB,
including 45344688 allocated vCPU-ms.
The native owner retained the full native report compressed with its digest in main commit `893b26838e`.
Existing issue `#1047` received the observed rootless `/work` ownership boundary;
it remains distinct from issues `#1116` and `#1117`.

The bounded repaired-adapter pilot `0j8zhbn9n0` at `a6dcb98aaa` crossed that boundary:
the unchanged source command exited zero,
its 37-file baseline was green,
and its sole selected mutant reached `compileError` without infrastructure errors or unrun entries.
This terminal category supplies no mutant assertion credit.
The provider job still failed because a metadata-snapshot operation returned null status,
and its pilot-only terminal-category acceptance was stricter than the authored source CLI.
The owner is recording exact failed-operation diagnostics,
aligning acceptance with source terminal categories,
and rerunning the bounded consumer before the full campaign.
The distinct inherited-object mutation producer defect is filed as `#1118`;
it does not explain the ownership-censored cloud failures.
All final TOML work remains held for the common dispatch interval.

The common cohort contract uses a frozen main-branch file named
`doc/audit/depot-complete-cohort-<cohort_id>.json`.
It binds `cohortId`,
each `cases.<case>.sourceRevision` and `manifestSha256`,
and each `routes.<route>.harnessRevision` and repo-relative workflow path.
The shared request timestamp is created after publication and retained identically before and after each route;
it is not part of the frozen manifest.
Authenticated Contents API object retrieval and base64 decoding preserved all 1,043,537 tested bytes;
the mismatched digest control was rejected.
Raw JSON CLI output changed escaped control characters,
even with `--allow-escape-sequences`,
so it is not the selected byte-preserving transport.
The common manifest remains unfrozen and no final dispatch is authorized by a control fixture.
The native owner has implemented actual-manifest bindings on its prepared routes and is verifying disposable controls.
The VM route is clean and pushed at `b2dcdc40d8ae2cfb1f7e13001db28ad8f17626b3`,
with 28 actual-task fixture controls and the correct logger case digest.
Its actual source/provider/guest execution remains held.
Android's manifest lookup now validates `cases.native` and separate case/cohort digests;
nine offline controls passed and no emulator operation occurred.
The logger cohort helper at `040a4814a63e27e392a68c83bceaaff78441c962`
passed before/after actual-program controls with a 1047160-byte base64 payload,
including a removed-digest-guard negative control.
Its authenticated transport proof is separate from those stubbed response controls.

Logger browser pilot `x0rb16tjzb` passed its actual five selected Playwright cases,
including localStorage,
IndexedDB,
and OPFS property reports.
Its full 84-browser/34-DOM inventories,
exact disjoint selections,
source identities,
resource limits,
and zero PID/OOM events are retained by the logger owner.
Remaining-browser run `jps875l9qs` and 35-leaf recovery `1pgpb8qhn6` are terminal with failed provider status;
their independent collectors are classifying actual source failures and censored work.
The original Podman browser command does not forward `CI`,
so the observed source config selects zero retries;
the owner is documenting that measured policy nuance without silently changing the source.
Final logger remainder registration is being split into isolated leaves so one group timeout cannot hide other checks.

The user requested continuation of the authorized trial queue.
Root is collecting the terminal native,
Linux,
VM,
and TOML attempts before final coverage and cost reconciliation.
No complete-pipeline success claim has been made.

Native recovery collection now confirms that both source builds,
release CLI consumers,
and final source-identity checks passed.
The actual source Clippy and nextest outcomes failed and are being reconciled by test identity.
The separate recovery evidence is
`doc/audit/depot-native-recovery-2026-10-17.evidence.json`.

Linux nextest attempt `pbg4h8ww6k` stopped before tests:
its workflow omitted `DEPOT_CI_WAVE=native-desktop`,
so source tool preparation did not install `slint-viewer`.
Harness `409e6ec106` restores the desktop tool wave.
Root's independent final-recovery branch is `trial/depot-final-recovery-20261017`.
It was pushed and its remote SHA verified before all new dispatches.
Root owns its TOML full replay,
native and desktop runtime log-retrieval replays,
and Linux nextest recovery.
Their current run IDs are `p9r4xl6n12`,
`63r0kzrsmn`,
`km2xkxwpjs`,
and `xf4hhngdqg` respectively.
The terminal collector waits for these runs and retains each final provider status.

The current frozen logger package graph,
Pi,
and WireGuard run is `4x77lb56cs`.
Its separate external-adapter run is `j4m7gw8mwg`.
That external matrix adds browser,
DOM E2E,
Electron,
generated consistency,
full root Oxlint,
and the logger revision's desktop artifact and runtime consumers.
Logger host run `39003180992` supplies the six operating-system host gates.
Scanner runs `39003219313`,
`39003221137`,
`39003222894`,
and `39003225054` cover the TOML,
logger,
native,
and desktop immutable changes respectively.
Current-policy Scorecard gate `39003200588` passed and denied analysis under its rolling budget;
the exact diagnostic was four analyses already executed in the last hour.
It examined six historical runs using one GraphQL request and one point,
with no REST history requests.
The actual scanner path sets,
job metadata,
log hashes,
Scorecard diagnostic,
and failed TOML patch attempt are retained in
`doc/audit/depot-final-recovery-2026-10-17.evidence.json`.
Separate dispatch times must remain visible in whole-case accounting.

After adapter development is reconciled,
the queue still requires a coherent replay from frozen complete manifests.
The separate development requests cannot supply whole-pipeline elapsed time.
The logger owner is preparing a combined package/external workflow and explicit routing for all external checks.
The native owner is preparing the native Windows/macOS,
desktop Windows,
and Alpine routes with source provenance retained.
Root owns shared dispatch timestamps,
the Depot native feature and Android producers,
the headful Android consumer,
and final cost reconciliation.
No final coherent replay has been dispatched yet.

Runtime-log recoveries `km2xkxwpjs` and `63r0kzrsmn` failed in their first wide sessions.
Both logs retain Rust's thread-creation panic with OS error 11,
`Resource temporarily unavailable`.
The declared 512-task container limit is a hypothesis until live cgroup evidence is retained.
Runtime-only diagnostic routes on `trial/depot-final-recovery-20261017` add those counters;
scoped formatting and both JavaScript parsers passed.
Root pushed and verified clean harness `32e1c7290a62c41c314a33894cf8f43f9e0f23b8` before dispatch.
The native runtime diagnostic is `fs5t4ns6j6`;
the desktop runtime diagnostic is `tlc73pbmkk`.
Root must collect their terminal artifacts and counters before changing the PID budget.
Later configured sessions remain unexecuted.
The diagnosis lives in `doc/troubleshooting/depot-historical-checkout-bootstrap.md`.

Corrected Linux replay `xf4hhngdqg` passed all 1,460 eligible tests;
one source-declared ignored test remains skipped.
The native accounting owner is reconciling every emitted identity and ordinal with the final artifact.
Missing desktop prerequisite selection is tracked separately as issue `#1108`.
Logger host verification passed all six jobs in `39003180992`,
including source/lock identities before and after execution.
The frozen logger candidate now has 719 package checks and 19 external checks;
the 708-check candidate count in this handover is its historical development state.
The candidate still declares `completeGraph: false` while terminal reconciliation is unfinished.

TOML attempt `gfklnntq0x` failed in Depot's injected local-patch step in all jobs,
before historical source setup.
It produced no check artifacts.
Its original three attempts remain in accounting.
The retry uses the clean pushed branch and avoids that patch-injection boundary.

The VM diagnostic owner is collecting `22b5683gx8` and its retained daemon logs.
The exact log identifies virtiofsd's failed user-namespace mapping;
the runner's effective GID differs from its passwd primary GID.
Recovery `vq299blkbw` has crossed the affected final Ubuntu preparation boundary,
with successful preparation and destruction consumers.
Its Windows preparation and final evidence collection remain active.
The logger owner is completing the frozen graph's package and external adapters.
The native accounting owner is reconciling Windows/macOS results and distinct failure issues.
Account access remains excluded.

The overall evaluation is not complete.
The user corrected the completion claim:
measuring selected jobs does not measure complete intended CI,
which also includes Oxlint and the other quality gates.
The completion claim in `bae6ef28a1` was incorrect.
Account access remains deliberately excluded and is unrelated to this missing work.

Active work:

- Select checks only for affected files or packages and their affected consumers.
  The user explicitly requires change-aware selection.
  Oxlint must retain each selected package's complete configured project context:
  the custom rules make changed-file-only Oxlint invalid.
  Changed files select packages;
  they are never passed as an Oxlint narrowing filter.
- Enumerate the full intended CI task graph from root and package tasks,
  including quality gates,
  builds,
  unit/native/browser/device checks,
  and required platform boundaries.
  Prove package discovery and report every omitted or inapplicable task explicitly.
- Execute that graph in an isolated trial on one immutable source revision,
  retaining actual check outcomes and setup/service dependencies.
  Early failures must not silently omit independent checks.
- Measure whole-pipeline usage and elapsed time until every required check reaches a terminal state.
  Reconcile those measurements with the cost model and the user's overlap-aware time objective.

The queued source-change cases are the first-parent diffs ending at:

- `525d2e09f8168a3bdd7e513e26d4c80b363123e5`:
  TOML-edit source change for initial pipeline verification.
- `ef01490f5e960f8ca0b424c12242caeb3c5a2b5e`:
  logger source/tests with its actual broader consumer closure.
- `cb95cf6d785c3073c192b95f0632cdae7b910894`:
  native audio feature/source change and affected applications.
- `3434eaa455d569c18b52cf15100f77f9db251d21`:
  desktop source change with the Windows and Alpine workflows present at that exact revision.
  The earlier native case predates those workflows and `Containerfile.alpine`;
  it cannot supply their current platform evidence.

Each execution must use that case's immutable head and dependency graph.
A documentation-only selection control must prove unrelated package checks are absent.
The initial smaller case does not replace the broader logger and native cases.
The desktop case also measures its applicable Linux gates alongside the existing Windows and Alpine jobs.
Preparation run `1v4l9tglbt` failed before checks because shallow checkout omitted the asserted base commit.
Its setup failure and retry remain part of trial accounting;
no check coverage is claimed from that attempt.

The change replay models a push event.
At the TOML case's exact head,
CodeQL runs for pull requests,
scheduled events,
or manual requests;
final-newline runs for pull requests,
merge groups,
or manual requests.
Their event-policy exclusions remain explicit in the manifest.
The expanded package quality gates are still required.
The existing forbidden-strings push gate is selected.
Its isolated harness is commit `0b8cc50c62` on `trial/depot-complete-scanner-20261016`:
it uses the verified current scanner `0.5.0`,
then checks out each immutable source head and scans that case's actual base/head range.
The TOML source revision declares scanner `0.4.1`;
this trial tests the current scanner against historical source,
not a historical scanner performance comparison.
The exact desktop platform ref is `trial/depot-complete-desktop-20261016`,
pointing directly to `3434eaa455d569c18b52cf15100f77f9db251d21`.
Neither trial ref changes production workflows.

The candidate manifests now retain these separate execution queues:

- TOML:
  24 package checks across six owners,
  including a test-only live-source consumer of file-enforcer.
  The initial five-owner manifest omitted that consumer and was corrected before the complete replay.
- Logger:
  708 package checks across 158 affected owners;
  artifact prerequisites and external execution adapters remain under review.
- Native core:
  21 package checks across four owners,
  with native host consumers,
  feature-matrix checks,
  and Android device execution tracked explicitly.
- Desktop:
  six package checks plus Windows,
  Alpine,
  macOS,
  and repository-gate adapters.

Every candidate still declares `completeGraph: false`.
Development waves must be followed by a coherent replay with a frozen complete manifest;
manual adapter-development gaps cannot be presented as steady-state CI latency.
The [accounting audit](../audit/depot-complete-ci-accounting-2026-10-16.md)
retains failed preparation and cancelled attempts.
TOML scanner run `38840751221` passed the exact changed-file gate.
Scorecard run `38841678032` completed the current rate-limit gate and correctly denied analysis:
four analyses had already executed in the preceding hour.
The earlier schedule-only description of Scorecard was incorrect:
its workflow also triggers on main pushes.

Native host run `38842242564` uses harness `703aca7141` to check source `cb95cf6d78`
on isolated Windows and macOS runners.
Native scanner run `38842263437` checks the same source change.
The native Linux checks and APK producers remain queued with the cloud executor.
The private headed Android emulator passed its resource-readiness probe:
API 37,
x86_64,
16 KiB pages,
font scale `1.0`,
and no forced density or display-size settings.
It was shut down gracefully and its unit is inactive.
This proves resource readiness,
not execution of the native case's still-pending APK consumer.

The retained job experiments provide inputs and verified execution routes;
they do not satisfy whole-pipeline completion on their own.

The user's completion question exposed unfinished experiments in the broader brief.
The user then explicitly authorized executing that queue and added Depot Test.
The resumed execution now has results for each accessible experiment:

- [Depot Test](../audit/depot-test-feature-2026-10-16.md):
  real nextest JUnit ingestion,
  controlled failure and skip,
  same-commit retry,
  analytics,
  and timing-based splitting verified.
  The final split returns 375 unique identities,
  with 374 passes and one existing ignored case.
  Repeated report upload duplicates rows and remains an integration risk.
- [Prepared desktop](../audit/depot-prepared-desktop-2026-10-16.md):
  two fresh consumers reuse installed packages and exact built artifacts,
  start real user services,
  and each pass 117 interaction checks with no failures and one uninjectable check.
  Both verify cleanup.
- [Expanded execution](../audit/depot-expanded-execution-2026-10-16.md):
  native feature coverage and Clippy pass;
  276 native executions cover 80 unique names.
  DOM browser checks pass 34 cases.
  Logger/filesystem browser checks pass 67 cases,
  retaining one WebKit failure and one skip.
- [Standalone linker workload](../audit/depot-linker-experiment-2026-10-16.md):
  identical compiled inputs,
  unchanged-input baselines,
  alternating linkers,
  and 160 consumer invocations completed.
  Artifacts were retrieved and the sandbox stopped.
  The difference is smaller than baseline spread;
  no linker winner is established.
- [Broader cost replay](../audit/depot-resumed-cost-2026-10-16.md):
  prepared source rebuilds and consumers,
  native coverage,
  browser checks,
  test reporting,
  and standalone sensitivities are included explicitly.
  Developer measured-workload subtotals are USD 133.33 for every affected revision,
  USD 125.01 with immediate superseded cancellation,
  or USD 20 with daily Depot selection and GitHub push validation retained.
  These are not complete account forecasts.

The Android instrumentation result uses cloud-built APKs on a local headed emulator;
it does not establish Android device execution inside Depot.
Production migration and plan changes remain separate decisions.

The selected experiments in the resumed queue have completed.
The user explicitly excluded further account-access work on 2026-10-15 local time.
Account-wide reconciliation is excluded by that decision,
not a pending completion requirement.
Other packages and platforms remain unpriced limits on a broader rollout forecast.
The complete intended CI measurement remains required;
fixing every discovered application issue is separate from measuring and reporting its check outcome.
Filing an issue does not count as executing an experiment or completing production integration.

The current usage API re-probe still rejects the CLI user token and requires an organization token.
Browser discovery through the available CLI found no running Chrome instance.
The user said they cannot supply an organization-token file or a signed-in browser for this check.
The accessible experiments and their measured usage are now reconciled.
Account-wide billing and spending-control verification remain unverified and intentionally excluded.

The user requested a GitHub issue for every discovered problem on 2026-10-15 local time.
The [issue inventory](depot-trial-issues.md) records each distinct trial finding,
including repaired incidents with their verification.
Search existing issues before creating duplicates;
track upstream-tool findings in this repository unless separate upstream publication is requested.
The issue inventory covers application/harness failures,
cache and service setup,
release transfer and quota behavior,
scheduling,
coverage gaps,
and budget observability.

The user asked whether package installation should also be cached.
The desktop trial repeats apt provisioning on disposable GitHub runners.
Depot CI snapshot run `6w30pv5zds` now proves installed-package reuse in two fresh consumers:
23 exact package versions and ImageMagick execute without apt or downloads.
Both consumers reported `Failed to connect to bus: No medium found` for user services.
That first filesystem-only trial did not establish desktop runtime startup.
The resumed prepared desktop trial subsequently combined both successfully.
The expanded cost report and snapshot troubleshooting record retain this boundary.
The user chose to leave `EVL` unchanged and retain this requirement in the trial documents.

On 2026-10-15 the user requested repair of organization-transfer download failures for all tools,
then reiterated that avoiding the shared 5,000-request REST quota remains a Depot objective.
The supplied failure is run `38828960531`,
job `116742977613`,
in `forbidden-strings`,
which invokes `gh release` rather than `cargo-binstall`.
Probe run `38829393247` reproduced authenticated REST HTTP 403,
public HTTP 200 downloads for all four published tools,
and `gh release view` reporting `release not found`.
The contemporaneous Scorecard log explicitly reports installation quota exhaustion.

The scanner's old provenance returns HTTP 404 through repository endpoints,
but `users/Aquaticat/attestations/<digest>` retains its signed bundles.
Verification using the downloaded bundle and original repository identity passed locally.
The consumer repair passed production run `38830142505`,
including provenance verification and the scan.
Positive and negative controls accept the original archive and reject altered bytes and an unrelated signer identity.
The Cargo metadata generator and its eight managed manifests now use the organization URLs;
already published manifests are immutable and were not republished.
The GraphQL query replacement landed in main commit `f598404b31`,
preserving the existing gate policy.
Production run `38831778986` inspected 972 runs with 10 GraphQL requests and no REST history requests;
the daily budget correctly skipped analysis.
The existing admission-reservation gap is separate and remains a proposed design change.

The Android context experiment completed with 16 passes and no failures or skips,
and its disposable visible emulator was shut down.
Desktop run `38831098294` completed every session independently:
116 checks passed,
one folder-dialog check failed,
and held-modifier repeat remained not injectable.
Keyboard selection recovered the thirty-track fixture;
installing Zenity did not recover the chooser check.
Trial run `38832349057` reproduced the chooser failure while proving the shortcut reached the application.
Run `38832874391` captured a Zenity `SIGSEGV` after 163 ms.
GLib reported a `v` variant expected but a `u` value received.
The private appearance portal returns the same shape from `Read` and `ReadOne`,
although the current XDG contract requires an extra variant layer for deprecated `Read`.
Trial commit `5eb02af048` adds exact wire-value signature assertions;
the unchanged implementation failed four of 270 tests on those assertions.
The wrapper correction in `dcaae7c673` passed all 270 tests without skips.
Full desktop cloud run `38833901553` verified the consumer boundary at `93560fbb09`:
117 checks passed,
none failed,
and one held-modifier check remained uninjectable.
Zenity opened and cancellation restored focus without a crash.
Final trial source `d6deca5fa5` also passes Clippy and scoped lint;
its cleanup only names the error converter and wraps comments.
Issue [#1004](https://github.com/Aquaticats/Monochromatic/issues/1004)
now contains the subprocess evidence.
The successful job took 410 seconds including setup and builds.
Its additional cost replay gives USD 23.38 for daily-latest Depot allocation
or USD 160.28 for every affected revision with separate container validation retained.
These are measured-subset Developer scenarios,
not complete monthly bills or controlled timing improvements.
Startup's every-affected equivalent exceeds its included runner allowance.
The selected execution batch and provisional recommendation are complete.
The remaining evaluation experiments are listed in Completed trial execution and remaining acceptance gaps.
Integration of trial-only corrections remains separate work.
No production migration or plan change was adopted.

### Active replay at 03:01 UTC on October 16

The subsequent native recovery uses harness `32a9250add`:
run `38847322477` checks historical native source `cb95cf6d78` on Windows and macOS,
and run `38847322559` checks desktop source `3434eaa455` on macOS.
The harness explicitly installs Clippy for `nightly-2026-10-13`
and derives the original test task body through mise metadata and configuration reads.
Its only test invocation change is `--no-fail-fast`,
applied in memory without editing historical source.
A disposable control verified the added argument,
package environment,
and Slint debug-info activation before dispatch.
Neither recovery is a result until its provider run and individual checks terminate.

VM retry `8wfm7bfgjv` stopped before VM checks:
the runner had no `XDG_RUNTIME_DIR`,
so the short-socket assertion rejected setup.
It produced no VM artifact and remains a failed preparation attempt.
Harness `295045dd82` creates a missing private runner runtime directory under `/run/user/<uid>`;
its retry is `7gxcgml68r`.
Root retains documentation and issue work for the native Clippy prerequisite,
nextest fail-fast coverage,
VM socket/runtime setup,
and the original vmsync task's relative command path.
The final coherent whole-case replays remain required after adapter development.

The native Clippy setup repair passed both hosts and is closed as issue `#1072`.
The full-suite coverage gap remains issue `#1073`.
Linux desktop test recovery `pbg4h8ww6k` uses additive harness `af3ccd254d`,
retaining complete test logs and source identity while disabling only nextest fail-fast.
The distinct Linux initial-queue and extra-burst-report observations are issues `#1079`,
`#1080`,
and `#1081`.

VM run `7gxcgml68r` verified the missing-runtime repair,
closed as issue `#1075`,
but exposed a long QEMU monitor socket.
Harness `0bcfa40820` shortened private configuration paths as well.
Its run `rv6hfj76w9` progressed beyond socket-length checks,
then failed when QEMU could not connect to its virtiofs socket.
Root's next diagnostic replay retains private daemon logs before cleanup.
The original package task path defect remains issue `#1076`;
socket layout is issue `#1074`.
None of these failures counts as passing VM lifecycle coverage.

The reviewed logger graph is frozen at 719 checks,
with manifest SHA-256 `f4cbd47f210f5b031d76c5fc1c28fa704b349621f91eefa3f4f3ad575acfedc3`.
The independent AST/runtime audit found no additional edges and retained all conservative barriers.
Coherent TOML replay `55gv2bqvll` starts its core,
container/mutation,
and generated-consistency branches together.
The environment owner is assembling the coherent logger dispatch from that frozen graph.

The complete graph remains unfinished.
The TOML core attempt `hgqj2nstwq` recorded 16 passing checks,
5 failed checks,
and 3 deferred checks.
The accounting audit independently reconciled every check record with the frozen manifest.
Its provider allocation was 229 seconds at 16 vCPUs;
its request interval remains censored because required checks are unfinished.
Control run `g57729x66d` observed the intended pass,
intentional task failure,
Oxlint argument rejection,
and file-scoped Oxlint rejection.

Native host attempt `38842242564` failed before package checks on Windows and macOS:
the harness omitted the configured `cargo-binstall` prerequisite.
Trial commit `87f5bf2c1f` restores the documented prerequisite sequence.
Retry `38842787230` uses the same historical native source revision.
Issue `#1059` tracks the setup defect until both hosts cross that boundary.
The source trace and attempted remedy are in
`doc/troubleshooting/mise-cargo-backend-needs-rust-installed.md`.
Native scanner run `38842263437` passed and scanned all eight changed paths.

Depot native core and Android attempt `5tbb7qftkw` stopped during an unrelated inherited Git-wrapper build.
That historical revision already uses the published native wrapper;
it does not use the TOML revision's JavaScript wrapper build and trust command.
The environment owner is replacing those inherited setup steps with the source-declared published tool.
Historical source checks remain unchanged.

The logger manifest now contains 718 local checks and 19 external groups,
including 155 full-project package Oxlint checks,
157 type checks,
and 1057 ordinary unit-test files.
The added artifact graph has 165 build prerequisite edges and no cycles.
Root owns the filesystem identity and semantic bridge host adapter at trial commit `fc236f278b`,
awaiting its source-specific review before dispatch.
It exercises exact source `ef01490f5e960f8ca0b424c12242caeb3c5a2b5e`
on Linux,
Windows,
and macOS.
The environment owner retains Linux runtime groups,
including desktop,
Electron,
terminal,
Pi,
disposable VM,
and network namespace checks.
The accounting owner retains Scorecard and complete attempt accounting.
No account access has been reopened.

Root's Android consumer now has separate artifact-hash verification and full-suite result capture tasks
in the private emulator harness.
Source inspection of the exact native head found 15 instrumented tests across four classes,
rather than the later revision's 16 tests.
The consumer will compare every observed test name and status with that source catalog.
Next actions remain producer APK retrieval,
headful private AVD execution with synthetic fixtures,
and graceful shutdown.
The final coherent whole-case replays remain queued.

Subsequent host observations:
logger scanner `38843463588` verified three paths,
and desktop scanner `38843587917` verified six paths.
Each scanner job took 28 seconds.
Logger host run `38843901438` passed all six jobs against source `ef01490`:
filesystem identity and semantic bridge/external-consumer checks on their original operating-system matrices.
Trial harness `3b06b44af2` explicitly disables mise-action environment export during bootstrap;
exact tool installation and package task activation happen separately.
Alpine run `38843525621` passed both architectures against desktop source `3434eaa455`.

Desktop Windows run `38843524267` reproduced the already documented historical PowerShell assertion mismatch
at that same source revision.
`doc/decision/music-player-windows-gui-subsystem.md` records the original failing run at `3434eaa455`
and the later corrected workflow at `2db98160ed`.
This replay does not establish a new application defect.
The failed PowerShell step skipped nextest,
so trial harness `198a7bacb9` keeps the immutable source checkout and makes nextest independent
of that known failure.
Its replay remains required before accepting the Windows test coverage.
Issue `#1060` was corrected to the historical mismatch and closed;
the missing replay coverage remains part of `#1030`.

The successful Android producer is Depot run `j80v9dqfb4`,
job `sk8lkf9xfp`,
attempt `s57prgfpx8`.
Root started the private headful AVD for its consumer:
unit `depot-trial-fold`,
invocation `c7538398d50c4cb5994b60eaa257c8a9`.
Boot verification returned API 37,
`x86_64`,
16384-byte pages,
font scale `1.0`,
and no forced size or density.
The installed debug and test APK hashes matched the producer's recorded hashes.
The unfiltered run matched every test name from the historical source catalog:
15 executed,
13 passed,
two failed,
and none skipped.
Both `SessionStoreTest` cases reported the previously documented null-context exception.
No application or test source was changed.
Execution took 6.041 seconds from instrumentation launch to command return;
this is a functional result,
not a performance comparison.
The emulator completed sync/wait/sync/emulator-kill shutdown and its unit is inactive.
The [exact-source Android evidence](../audit/depot-complete-android-2026-10-16.evidence.json)
retains the source catalog,
artifact hashes,
per-test statuses,
and complete instrumentation output.

Root next owns the disposable VM adapter for the logger case.
Cloud probe `1dg9t9znbj` found readable and writable `/dev/kvm`,
but no `virsh` or prepared VM image directory.
No VM was created by that probe.
The next step is to prepare isolated libvirt and guest prerequisites,
then execute the unchanged historical `vmsync` lifecycle task.
This remains required work;
the probe does not establish that cloud VM execution is unavailable.

The VM adapter passed independent source/data-boundary review and was dispatched as Depot run `99tl7zjfgp`
from the pushed trial commit `a0785c49e6`.
It uses source `ef01490`,
16 vCPUs and 64 GiB,
a 75-minute job bound,
and a 60-minute task deadline that reserves cleanup time.
Its private home and libvirt runtime start without guest domains.
Guest images remain outside the uploaded log/evidence directory.
The adapter retains the original `vmsync` package task result;
a separate source invocation from the repository root cannot turn that failed gate green.
Cleanup errors do not suppress daemon termination requests or final source-identity checks.
Daemon termination requests alone are not evidence that the processes exited.

Android issue `#1002` now includes the exact-source replay evidence in comment `6207416475`.
Its scope and open status are unchanged.

VM run `99tl7zjfgp` failed after the source bundle built successfully.
The original package task resolved its relative `mvm` entrypoint beneath `package/cli/vmsync`
and failed with `MODULE_NOT_FOUND` before creating a guest.
The separate environment probe installed libvirt,
started its private daemons,
verified an empty domain list,
and ran the source `mvm --help`.
Ubuntu template preparation then failed because the generated Unix socket path exceeded its length limit.
No guest execution is claimed.
Trial commit `906bf436de` moves private runtime sockets beneath the cloud user's runtime directory;
its retry and incident documentation remain queued.

Windows recovery `38844610546` completed all three nextest targets:
each reports 1432 passes and one ignored test,
while preserving the historical PowerShell assertion failure.
Native host `38842787230` and desktop macOS `38843527788` reached package checks.
Their Clippy steps lacked the locked toolchain's Clippy component.
The native Windows source additionally fails test compilation on unresolved `libc`;
that historical source defect was fixed later.
macOS tests failed and nextest's default fail-fast behavior left remaining cases unexecuted.
Root owns the explicit Clippy prerequisite and fail-fast adapter recovery;
the accounting owner retains the separate historical source findings.
Current nextest `0.9.148` source and CLI help confirm `--no-fail-fast` support.
No application or historical test source change is authorized by this trial recovery.

## Purpose and current state

The user subsequently requested immediate execution of the agreed trial.
The interview and runner-sizing follow-up are complete.
The selected broader evaluation experiments and recommendation are recorded with their acceptance gaps.
The [trial report](../audit/depot-trial-2026-10-15.md)
records remaining coverage and account-access limits.
The user explicitly clarified that this is not an apples-to-apples migration.
Controlled build comparisons answer sizing questions;
the migration recommendation must evaluate the expanded coverage,
reusable environments,
agent-requested experiments,
convenience,
and total target-workload cost.

The user is trialing Depot and requested a grilling interview on 2026-10-15.
The user has delegated the technical recommendation after establishing priorities and budget.
The user explicitly requested further questions even when their answer might be uncertain.
They then confirmed Q19 and its corrected ownership clause,
establishing shared understanding of the trial brief.
The [trial brief](../planning/depot-ci-trial.md) records the agreed evidence-gathering scope.
No migration or vendor adoption has been accepted.
The user explicitly requested this handover.

The user is inexperienced with CI/CD and wants concrete explanations and probing questions.
Abstract questions about acceptance thresholds and failure criteria have produced uncertainty.
The user explicitly said looking at Claude transcripts is unnecessary;
transcript investigation was stopped.

## Answers to preserve

- Motivation:
  GitHub Actions is slow.
- Provider-spending ceiling:
  USD $200 per month.
  During execution the user clarified that saving USD $180 monthly matters:
  prefer the USD $20 Developer plan if it fits without excessive inconvenience.
  Compare Developer with GitHub routing and selective overage before recommending Startup.
  No subscription or billing change is authorized by this preference.
  The user then clarified that the current seven-day trial is still on Startup
  and explicitly wants the evaluation to use its available resources freely.
  Do not constrain trial coverage to Developer allowances;
  evaluate future monthly placement separately from current trial execution.
  The time-value objective now guides latency tradeoffs;
  no absolute wait threshold has been chosen.
- The user confirmed that the music-player release taking about 24 minutes is a relevant example.
- The user clarified that it was blocking Claude from working on the music player.
  Waiting to download a personal build was an interviewer hypothesis,
  not a confirmed requirement.
- The current CI setup is insufficient;
  the missing checks are not yet specified.
- The user is interested in avoiding repeated tool installation through Depot container builds.
  The claim that every run reinstalls every tool has not been verified across workflows.
- The user reports being logged in to the Depot CLI and points out that Depot skills are available.
- Asked which mistakes automatic checks should catch,
  the user answered that they are not sure yet.
- The user emphasized twice that current CI is far from sufficient or comprehensive.
  The target includes missing checks,
  not merely faster execution of the existing workflows.
- The user wants sliding-window rate limits,
  or an inexpensive way to express them without GitHub's overhead.
  They rejected hypothetical burst/debounce questions and requested inspection of actual GitHub CI history.
- The user highlighted GitHub's 10 GB cache space as another concern.
- The user accepts agent-requested cloud checks,
  while preserving early commits even when broken.
  Checking uncommitted changes is an optional capability,
  not a requirement to delay commits.
- The user knows GitHub cache capacity can be increased for a charge and considers that prohibitively expensive.
  Paid GitHub cache expansion is not an accepted fallback.
- Whether app/device checks should run on every push or at agent-requested checkpoints remains undecided.
  The user answered that they are not sure;
  no scheduling preference was accepted.
- The user identifies Depot's $200 Startup plan as the most they are willing to spend.
  Current published pricing includes usage allowances and charges overages;
  it is not unlimited usage for $200.
- The interviewer checked current GitHub cache pricing and gave a 50 GB total-cache example costing $2.80 extra monthly.
  The user explicitly said this does not change their view of GitHub's total costs and described GitHub as wasteful.
  The preference concerns total cost and repeated work,
  not just cache storage unit pricing.
- The user values each second while relevant CI remains unfinished at half Ontario's general minimum wage.
  They accepted counting elapsed time even while other useful work continues,
  with overlapping jobs counted once.
- Ontario's current general minimum is CAD $17.95 per hour.
  The resulting time value is CAD $8.975 per hour or approximately CAD $0.002493056 per second.
  The observed 1,448-second release interval represents CAD $3.609944 under this model.
- When asked to choose a comprehensive technical coverage target,
  the user delegated the recommendation to the agent.
  Further questions should seek user preferences or constraints,
  not ask the user to design CI machinery.
- Q13 asked about Claude's working habits while checks run.
  The user rejected it as irrelevant to Depot.
  The interviewer withdrew it;
  keep subsequent questions tied to Depot evaluation and adoption constraints.
- Q14 established an intentional hybrid:
  intelligently route some CI to GitHub and some to Depot.
  Routing should use measured benefit and the provider-spending ceiling from the outset;
  GitHub is not merely a fallback after budget exhaustion.
- Q15 remains unresolved:
  the user does not yet know whether a future organization transfer is acceptable.
  Evaluate currently available routes without assuming transfer permission.
  Superseded by observed state on 2026-10-15:
  GitHub subsequently reported `Aquaticats/Monochromatic` as organization-owned.
  This session did not perform that transfer.
- Q16 accepts prioritizing the newest version and stopping superseded intermediate checks.
  Explicit release builds remain separate from this intermediate-check policy.
- Q17 accepts deeper Depot dependence when measured benefits justify it.
  Portable checks are not a hard requirement;
  the intentional GitHub/Depot split remains a requirement.
- In Q18 the user reported the USD $200/month plan,
  a seven-day trial,
  and no usage so far.
  These are user-reported account facts.
  Exact expiry and configured spending controls remain unverified.
- Q19 and its ownership correction are confirmed.
  Evaluate both Depot CI and Depot GitHub Actions runners where applicable,
  using the intentional hybrid and combined cost objective.
  This confirms the trial brief,
  not vendor adoption or a measured result.

## Measured release example

[Run 38785821860][release-run] was inspected with `gh run list` and `gh run view --json jobs`.
Timestamp differences from this October 15 execution:

- Workflow creation to last update:
  24 minutes 8 seconds.
- Alpine x64 release compilation:
  18 minutes 46 seconds.
- Alpine x64 container image creation:
  28 seconds.
- Windows GNU compilation:
  14 minutes 12 seconds.
- Windows MSVC compilation:
  5 minutes 27 seconds.
- Linux x64 compilation:
  2 minutes 14 seconds.
- First job start minus workflow creation:
  1 minute 36 seconds.
  This is observed delay,
  not proof of runner queue time;
  workflow concurrency also serializes runs.

These measurements describe one execution,
not repeatability or a controlled Depot comparison.
The relationship between this run and Claude's inability to continue remains unverified.

## Evidence affecting the trial

The release workflow is in [music-player-release.yml](../../.github/workflows/music-player-release.yml).
Its Alpine compilation happens in a separate `podman run` after container image creation.
Replacing only the image builder does not move that separate compilation command to Depot.

[CI caching policy](../decision/ci-caching.md) records deliberate uncached Alpine and Windows GNU release builds.
The recorded reason includes GitHub cache capacity pressure.
Those cache sizes are historical readings,
not current measurements.

GitHub's repository API reported `Aquaticat/Monochromatic` as public and personally owned.
On 2026-10-15 during the interview,
GitHub's push response reported its move to `Aquaticats/Monochromatic`.
`gh api repos/Aquaticats/Monochromatic` then confirmed public visibility and owner type `Organization`.
The personal-ownership observation is historical;
the current organization ownership permits evaluation of the runner product under its ownership requirement.
Current [Depot runner documentation][runner-docs] requires organization-owned GitHub repositories.
[Depot CI documentation][ci-docs] supports personal repositories but excludes Windows and macOS sandboxes.
These are distinct products;
a complete unchanged release migration is not established.

[Depot container builds][build-docs] retain image layers for subsequent builds.
Tool installation reuse and reuse of compiled dependencies are separate opportunities to measure.
The trial measured cold and warm Depot executions;
the controlled GitHub comparison and corrected runner-sizing follow-up have completed.

## Rate-limit and cache probes

The existing [Scorecard workflow](../../.github/workflows/scorecard.yml) has rolling budgets of
2 analysis executions per minute,
4 per hour,
and 8 per day.
Its separate gate job reads GitHub run and job history before allowing analysis.
Rejected runs are skipped,
not scheduled for later.
This concrete policy supersedes the interviewer's hypothetical burst scenarios.

In [run 38806855827][gate-run],
the gate's setup step finished at 21:03:09 UTC on October 15.
The history-counting step remained in progress when queried at approximately 21:06:35 UTC.
This sample does not support attributing the delay solely to machine startup.
A query for Scorecard runs created since October 14 at 21:00 UTC returned 926 runs.
No completed gate duration or causal diagnosis has been established.

Depot documents [concurrency waiting inside its orchestration system][depot-orchestration],
but that is not equivalent to the Scorecard rolling execution budgets.
Searches of current compatibility docs,
documentation indexes,
API docs,
and public CLI request definitions
did not establish a native configurable rolling-window admission feature.
A custom controller before dispatch is an untested design possibility,
not a verified capability or accepted proposal.
Copying the current gate to Depot would still count GitHub runs instead of Depot runs.

The GitHub cache usage API returned 10,696,470,622 bytes and 15 entries on October 15.
A subsequent cache listing returned 16 entries,
so these endpoints are not a simultaneous snapshot.
The configured cache cap was not independently queried.
[GitHub documents paid cache expansion][github-cache],
so 10 GB is not a universal hard ceiling.
The user's objection to expansion is cost.

`depot org list --output json` succeeded and returned the user's Depot organization.
CLI access is verified without printing credentials or modifying authentication.

## Cost constraint and published allowances

[Depot pricing][depot-pricing],
checked October 15,
lists Startup at $200 per month with:

- 250 GB shared cache and registry storage.
- 20,000 Depot CI minutes,
  with larger machines consuming allowances faster.
- 20,000 GitHub Actions runner minutes.
- 5,000 container build minutes.
- 1,000,000 passing test results.

Overage charges apply beyond included usage.
The $200 ceiling is a requirement,
not authorization to purchase a subscription or consume additional paid resources.
Public documentation establishes some per-product minute caps,
but not a guaranteed aggregate USD $200 bill.
Already-running jobs,
storage,
and CI-specific cap behavior require account-level verification.
Current account plan,
trial credits,
and cap settings have not been inspected directly.
The user subsequently reported the USD $200/month plan,
a seven-day trial,
and no usage.
The browser skill's account inspection returned `No browser is available`;
browser discovery returned an empty list.
No Depot page opened and no authentication or billing setting changed.

A subsequent read-only probe used Depot CLI 2.102.22 and the documented `UsageService.GetUsage` endpoint.
The request covered October 1 through October 15 at 21:00 UTC.
Depot rejected the existing user token with code `unauthenticated`:
"This endpoint requires an organization token,
but a user token was provided.
Create an organization token in Organization Settings > API Tokens".
No organization token was available in `DEPOT_TOKEN`,
and none was created.
The inspected organization response exposes identity rather than billing configuration.
Current usage and an enforceable aggregate cap remain unknown.
The Q18 answer supplies plan,
trial duration,
and reported usage;
it does not establish an expiry timestamp or spending-limit behavior.

[GitHub billing][github-billing] lists cache overage at $0.07 per GB-month,
where its GB unit is a GiB.
The illustrative calculation was `(50 - 10) * 0.07 = 2.80` for constant full-month occupancy.
This storage-only calculation does not estimate the cost of the desired comprehensive CI.
The user rejected it as a reason to change their preference.

## Recent overlap and branch-rule evidence

A read-only workflow-history probe covered October 15 from 00:00:00 through 19:00:00 UTC.
The API returned 75 runs and matching total counts:
29 release runs,
33 Windows runs,
and 13 Alpine runs.
Ten older runs had executing jobs after a newer descendant commit triggered the same workflow on the same branch.
Commit ancestry and actual job completion timestamps were checked.
Seven finished without cancellation;
three release runs were cancelled.
One successful overlapping release run only performed detection and skipped builds.
The probe does not establish that older results were unnecessary or why cancellations happened.

For example,
newer Windows and Alpine runs were created at 15:59:37 UTC.
The [older Windows run][overlap-windows] completed its last job at 16:09:01;
the [older Alpine run][overlap-alpine] completed its last job at 16:05:42.
These are elapsed overlaps,
not billed runner minutes.
Q16 initially called the newer-run creation time a start time;
the interviewer corrected that distinction explicitly.

At 21:27:05 UTC,
the branch-protection API returned empty required-status-check lists for `main`.
The effective rules API returned only a Copilot review rule.
These responses do not establish any required music-player CI checks.
This is background evidence,
not a proposal to redesign branch governance.

## Evaluation objective

Compare provider charges plus valued elapsed CI time for equivalent coverage and the same stream of changes.
The provider bill alone must remain within USD $200 per month.
Convert provider charges and the Ontario-based time value into the same currency with a dated exchange rate.
An exact CAD/USD rate has not been established in this session.

The time rate comes from [Ontario's general minimum wage][ontario-wage],
effective October 1, 2026.
Intervals with overlapping relevant CI jobs contribute elapsed time once.
Failed setup or skipped required checks are not successful performance results.

## Concrete coverage gaps

The read-only inventory compared workflow declarations,
root tasks,
and selected music-player tasks.
It did not execute tests,
inspect every package or third-party action,
or audit branch protection.

- [Android tasks](../../package/music-player/android-app/mise.toml) include host unit tests,
  device tests,
  Android lint,
  Kotlin checks,
  and native Rust Clippy.
  The inspected workflows do not invoke those tasks or their direct test commands.
  Android release packaging invokes `build:release`;
  ordinary Kotlin edits are outside that release workflow's selected `Cargo.toml` push paths.
- [Desktop tasks](../../package/music-player/desktop-app/mise.toml) include
  [real-keyboard end-to-end tests](../../package/music-player/desktop-e2e/README.md).
  The inspected workflows do not invoke that suite.
  Windows and Alpine workflows do run desktop unit and smoke tests.
- [Shared audio core tasks](../../package/music-player/truepeak-core/mise.toml) include their own all-features test.
  No dedicated invocation appeared in workflows;
  building the core as a dependency is not running its tests.
- [Root tasks](../../mise.toml) expose package builds,
  lint,
  unit tests,
  browser tests,
  and end-to-end tests.
  The inspected workflows run selected package checks rather than a general changed-package check pipeline.

These are missing connections to existing checks,
not proof that the underlying tests cover all desired behavior.
The broader CI plan must also identify behaviors for which tests do not yet exist.

## Open questions and next work

- Execution work items:
  - Container compilation and cache reuse:
    dedicated project created;
    remote marker build and local artifact export succeeded.
    Alpine build `ws40v1c9xt` passed in 468 seconds total;
    Cargo reported 7 minutes 13 seconds.
    The exported executable reported `music-player 0.1.4` inside the production runtime stage.
    Unchanged Cargo-input repetitions `t66qp9xk3l` and `5cfbvsmxz9` passed in 21 and 22 seconds,
    including local artifact export.
    The first warm Cargo invocation itself took 0.31 seconds.
    Intentional failure control `wk649mjc2s` failed at `RUN test "1" = 0` and returned exit status 1.
    Compilation has a 1,800-second limit.
    The dedicated project's requested 20 GB cache policy returned 25 GiB;
    treat the returned value as authoritative and the policy as a soft cache limit.
  - Depot CI and GitHub runner access:
    canonical Git remote updated to `Aquaticats/Monochromatic`;
    the user connected Depot Code Access.
    CLI preflight and the real checkout run `kbz7cjc5vt` succeeded.
    The user subsequently connected `depot-managed-runners`,
    verified in the organization installation listing.
    In run `38813764705`,
    GitHub's build passed with 869.508 seconds cold,
    then 1.975 and 1.716 seconds for same-job warm repeats.
    The Depot runner attempt failed with lost communication and no step results.
    Minimal runner run `38815232329` subsequently executed `uname` successfully.
    The original Depot build job passed on retry as job `116700352199`.
    Its cold build took 757.394 seconds;
    warm repeats took 1.173 and 1.093 seconds.
    This later execution does not establish the first failure's cause.
    See the [reliability record](../troubleshooting/depot-github-runner-communication.md).
  - Broader checks and deliberate failure detection:
    Run `wjr4cmpjvs` passed logger build/types/unit tests and all 80 shared audio-core tests.
    Its creation-to-finish interval was 152 seconds.
    Android host tests and lint passed as `3tvq2wvskh` in 218 seconds.
    The Gradle test task executed;
    run `jp9w8gw62r` inspected 127 cached XML suites containing 1,289 tests from that first execution.
    Branch `trial/depot-20261015` contains the disposable workflows without release credentials.
    A GitHub reference job uses the identical Alpine build definition and unchanged-source repetitions.
    Real-test negative control `2clfb5npj3` failed as intended:
    79 tests passed and `window::tests::window_frames_floors_and_clamps` failed
    with actual `192000` and deliberately incorrect expected `192001`.
    The fixture changed only its disposable remote source and restored it during cleanup.
  - Scheduling and rolling budgets:
    the existing script passed fixture replays for allowed and rejected windows,
    exact expiry,
    cancelled,
    failed,
    queued,
    and missing analysis jobs.
    A 926-run skipped fixture issued 926 sequential job-history calls.
    Concurrent fixture requests with absent analysis jobs both admitted;
    this exposes a missing reservation in that modeled interval,
    not a demonstrated live incident.
  - Runtime and monthly cost:
    explicit scenarios and dated CAD/USD conversion are recorded in the
    [trial report](../audit/depot-trial-2026-10-15.md).
    Developer with intentional GitHub routing ranks ahead of selective overage,
    then Startup for the measured workload.
    The report records excluded costs and unmeasured cross-platform completion effects.
  - Non-CI experiments and parallel throughput:
    the user proposed benchmarking `lld` against `mold` and highlighted unlimited concurrency.
    Evaluate direct remote execution separately from CI dispatch;
    preserve identical inputs and hardware for timing comparisons.
    Depot's container build parallelism documentation says builds in the same project and architecture
    share a builder by default.
    Separate benchmark latency from contention and aggregate throughput.
    Standalone SDK execution passed with an absolute executable path,
    including cleanup in 4.396 seconds.
    Two standalone sandboxes then ran overlapping commands for 9.882 seconds and both stopped.
    These prove the execution route and limited concurrency,
    not linker performance or unlimited capacity.
    No linker benchmark has run yet.
- Source invalidation build `jbld9m545r` rebuilt the application in 3 minutes 17 seconds.
  Its exported executable displayed the deliberately changed CLI name,
  proving the changed source reached the executable.
  The disposable source marker was restored in `d5ab91d795`.
- Prepared-image run `qrjbnf2gbv` passed after a 252.433-second additional mise cache lookup.
  Run `jp9w8gw62r` reused the image and passed with that extra action cache disabled.
  Its complete parallel validation took 118 seconds.
  See the [cache observation](../troubleshooting/depot-snapshot-mise-cache.md)
  and [standalone command workaround](../troubleshooting/depot-sandbox-command-path.md).
- The cloud device prerequisite probe found `/dev/kvm`,
  no `/dev/dri` entries,
  no display or Wayland session,
  an offline systemd user manager,
  and no PipeWire,
  `pactl`,
  or Weston executables.
  No emulator was launched and no graphical interaction coverage is claimed.
- Dependency invalidation build `41xd3lwh8d` passed in 382 seconds with Cargo regenerating a disposable lockfile
  from `log` version `0.4.34` to `0.4.33`.
  The source checkout's lockfile is untouched.
- Toolchain invalidation build `14trfrrxg6` passed in 473 seconds.
  The exported ELF comment identifies `mold 2.39.1` and the runtime prints `music-player 0.1.4`.
  This recompilation does not constitute a controlled linker benchmark.
- The initial measured subset is finished;
  remaining acceptance gaps are graphical/device execution,
  account-wide usage and spending controls,
  complete cross-platform history replay,
  and a validated rolling-budget replacement.
  The environment preflight found missing graphical prerequisites;
  the usage endpoint requires an organization credential unavailable to this trial.
  No production migration or recurring automation is installed.
  Standalone sandboxes are stopped;
  the dedicated cache and reusable CI snapshot remain for trial review.
  The final report,
  machine-readable evidence,
  handover,
  and runner incident record passed scoped `lint:monochromatic-lint`.
  CommonMark rendering was inspected for the report,
  changed handover sections,
  incident record,
  trial README,
  and organization-policy troubleshooting record.
  Trial implementation is retained at branch commit `7f0a8b6449`.
- The trial brief and corrected ownership scope are confirmed.
  No interview question is pending.
- Account-wide spending controls remain unverified;
  the supported usage endpoint rejected the available user-token credential.
  The user authorized immediate execution and reported an unused seven-day trial.
  Execution is bounded to explicit trial runs with time limits;
  no recurring dispatch or billing setting has been enabled.
  The user subsequently pasted the dashboard banner:
  seven days remain and billing details are required to continue builds after the trial.
- GitHub reference run `38812935669` failed before starting a job
  because organization policy allowed local actions only.
  The user approved the pinned action exceptions and then deliberately saved broader organization patterns:
  `actions/*`,
  `jdx/*`,
  `docker/*`,
  GitHub-owned actions,
  and verified creators.
  They explicitly confirmed the broader choice.
  The repository uses `selected` and retains SHA pinning;
  its allowed-action list is inherited from the organization.
  Do not narrow the user's deliberate policy choice.
  The prepared trial workflow still pins every action to an exact revision.
  The failed startup run could not be retried;
  a new trial-branch push launches both runner variants.
- The user pasted Depot's `High resource usage` warning during the cold build:
  CPU 100% peak and 89% average,
  memory 13% peak and 6% average,
  hardware 16 CPUs and 32 GB RAM.
  Preserve the hardware for this comparison;
  the warning did not prevent the build from succeeding.
- Preserve existing explicit-release handling during the proposed trial.
  The current release workflow serializes runs with `cancel-in-progress: false`;
  Q16 changes only the proposed policy for intermediate checks.
- Use the [trial proposal](../planning/depot-ci-trial.md) to collect comparable runtime,
  coverage,
  and cost evidence.
- Establish device-test runtime and cost before recommending its frequency.
- Compare Depot's effect on relevant completion times without reopening Claude's working habits.
- Make the technical coverage recommendation,
  preserving supported platforms and recording missing behavior tests.
- Compare the desired broader CI against the $200 ceiling and repeated work,
  rather than reopening cache unit pricing.
- Keep the agreed trial requirements separate from unproven performance claims and vendor-adoption decisions.

At interview completion no workflow,
Depot resource,
credential,
or billing setting has been changed in this interview.
The worktree initially contained an unrelated `mise.lock` modification.
Commit `86961fee56` created this handover;
its first version passed a CommonMark-to-HTML render check with Pandoc.
Commit `1c9599732d` added the rate-limit evidence and expanded coverage scope;
that version also passed rendering.

## Runner sizing follow-up

The user asked whether a bigger Depot runner would help,
then corrected the agent for stopping at a proposed experiment instead of continuing the authorized trial.
The original sizing experiment executed in
[run 38819692881](https://github.com/Aquaticats/Monochromatic/actions/runs/38819692881),
from trial commit `84a0b9c5ab`.
It compares CPU/job-count pairs `4/4`,
`8/4`,
`8/8`,
`16/4`,
`16/8`,
and `16/16` on separate ephemeral runners.
Each configuration performs two rounds with independent Cargo cache namespaces:
cold build,
two unchanged-source repetitions,
a visible CLI-name source change,
and an unchanged repetition of that changed source.
Every export must print the expected runtime version from the production Alpine runtime stage.
The workflow is bounded to six concurrent runners and 75 minutes per job.
The larger-runner follow-up,
[run 38820672079](https://github.com/Aquaticats/Monochromatic/actions/runs/38820672079)
at commit `7cd7b9e508`,
adds pairs `32/16`,
`32/32`,
`64/16`,
and `64/64` with the same measurement task,
bounded to four concurrent runners and 45 minutes per job.
Scoped configuration lint and resolved Bake-variable inspection passed before result collection.
The measured GitHub runner had four CPUs;
the separate container builder already had sixteen.
The initial provider comparison used `cargo --jobs 4`.
The container dashboard showed 89% average CPU and 13% peak memory utilization.
This suggests investigating CPU scaling,
but does not establish the speedup from additional CPUs or a different Cargo job count.

Current [runner pricing](https://depot.dev/docs/github-actions/runner-types)
is USD $0.012/minute for four CPUs,
$0.024 for eight,
and $0.048 for sixteen.
Doubling size must halve elapsed time to preserve compute charges or included-minute consumption.
Under the user's elapsed-time valuation,
a smaller improvement can still reduce combined cost if this job determines relevant CI completion.

The [container builder documentation](https://depot.dev/docs/container-builds/overview)
lists sixteen CPUs on all plans;
32 and 64 CPUs require Startup or Business and use two or four times the included minutes.
This distinction matters to the Developer-plan preference.
The runner-sizing subtask is complete:
the source-cache investigation,
corrected matrix,
result collection,
cost calculations,
and allocation recommendation are recorded in the
[sizing report](../audit/depot-runner-sizing-2026-10-15.md).
The container-product follow-up also completed two isolated cold builds and unchanged repeats
with 16 Cargo jobs on the existing default builder.
It uses the dedicated trial project,
fresh Cargo cache scopes,
the production runtime version assertion,
and the existing 1,800-second Cargo execution bound.
This measures the no-size-upgrade concurrency option instead of extrapolating GitHub-runner measurements to it.
The independent coverage check completed the existing `test:alpine` task through
`.github/workflows/depot-trial-alpine-checks.yml` on a disposable four-CPU Depot GitHub runner,
in [run 38820372915](https://github.com/Aquaticats/Monochromatic/actions/runs/38820372915)
at commit `1c1334f700`.
It exercises application unit tests and the existing no-audio,
ALSA,
and PipeWire graphical smoke scenarios.
The task bounds build containers to four CPUs and 12 GB memory,
smoke containers to four CPUs and 4 GB,
and the workflow to 60 minutes.
This does not replace the full desktop keyboard suite or Android instrumented tests.
No production size setting changed in response to this question.

The first sizing matrix rejected stale output in the second source-change phase.
The 8-CPU/8-job,
16-CPU/8-job,
and 16-CPU/16-job configurations independently returned `music-player 0.1.4`
when the changed source required `music-player-sizing 0.1.4`.
A disposable two-CPU,
2 GiB container reproduced Cargo 1.96.1's stale result after restoring changed source with old timestamps.
Cleaning the affected package restored the correct result.
Trial commit `4648e5ad3d` hashes copied source content and invalidates `music-player` and `truepeak-core`
when it differs from the target cache's recorded content.
Dependency artifacts survive this invalidation.
The corrected full sizing rerun passed.
Both the cache investigation and completed-job GitHub log retrieval workaround have troubleshooting records.

The corrected full reruns are `38822116503` and `38822116510`,
at trial commit `ef616a2900`.
Intermediate runs `38821765989` and `38821765814` failed during mise setup,
before compilation,
with `GitHub releases API returned 403 Forbidden`.
The trial follows the recorded workaround in `mise-action-version-ahead-of-release.md`:
pin published mise `2026.10.9` with an explanatory comment.
Every main-matrix job passed setup and reached compilation after that change.
The cache and completed-job log investigations now exist in
`doc/troubleshooting/cargo-buildkit-source-timestamps.md`
and `doc/troubleshooting/gh-completed-job-logs.md`;
their rendered reviews passed,
and the final application results passed in both corrected workflows.
The durable minimal reproduction lives on the trial branch under `depot-trial/cache-repro/`.

The Alpine behavior run completed successfully:
1,461 tests passed,
one was skipped,
and all existing no-audio,
ALSA,
and PipeWire graphical smoke scenarios passed.
The report records the scope limitation.

Local collection and documentation checks encountered task-allocation failures;
the enclosing worker slice reported a 256-task ceiling.
These local failures are separate from cloud runner outcomes.
Bounded CPU affinity and sequential commands allowed some checks to resume,
but do not establish a permanent fix.
The committed minimal fixture's first attempt failed before its container started.
Its rerun completed successfully,
including the stale-output negative control and corrected output after package invalidation.
The user subsequently reported the Codex issue corrected;
the worker ceiling now measures 2,048 tasks and local command execution resumed.
No cloud build input or measurement was changed by that local recovery.

The final corrected matrix passed all 100 checked outputs across ten CPU/job configurations.
Cold means were 526.7 seconds at `4/4`,
320.4 at `8/8`,
246.6 at `16/16`,
212.0 at `32/32`,
and 323.4 at `64/64`.
Keeping only four jobs on sixteen CPUs averaged 476.6 seconds.
The default container builder with sixteen jobs passed both cold builds in 268.558 and 265.417 seconds,
plus all four unchanged repetitions.
The [sizing report](../audit/depot-runner-sizing-2026-10-15.md)
contains ranges,
hardware differences,
source-rebuild results,
costs,
and source links.
It recommends `16/16` for cold builds determining completion;
`8/8` and `16/16` are an unresolved economic tie for the measured source rebuild.
These are allocation findings,
not a verdict on the broader migration's value.
Original failed matrices remain separate evidence.
No runner-sizing execution or collection remains queued.
The broader evaluation queue remains active as recorded in the completion audit.

The parallel history check inspected complete October 15 API pages:
13 Alpine and 33 Windows runs.
Four exact-commit completed push pairs exist;
three succeeded on both platforms.
Windows finished last in every successful pair,
by 104,
199,
and 123 seconds.
Linux-only acceleration cannot advance those pairs' final completion while Windows stays unchanged.
The failed Linux pair is excluded from successful-completion comparisons.
Evidence and a positive control for the upper-bound calculation are recorded in the
[trial report](../audit/depot-trial-2026-10-15.md).

## Completion audit

This section preserves the completion audit that reopened execution work.
The current terminal results and unresolved adoption gaps are recorded in
Completed trial execution and remaining acceptance gaps.

The user asked whether everything was done.
Comparing the agreed brief with the results showed that closing the entire trial queue was premature.
The runner measurements and policy edit are complete;
the broader evaluation still requires independently verifiable work:

- Desktop interaction:
  exercise the existing full keyboard suite through an isolated path,
  including provisioning its documented prerequisites before treating their absence as a blocker.
- Android device behavior:
  inspect the available device and cloud constraints,
  then exercise instrumented checks through a headed environment allowed by the emulator skill.
- Coverage and routing inventory:
  map the expanded target checks to triggers,
  execution locations,
  outputs,
  and measured or explicitly missing completion evidence.
- Scheduling:
  finish the evidence-based recommendation for the rolling budgets,
  including the observed missing reservation and history-query cost.
- Monthly cost:
  replay a representative change stream against the expanded target inventory,
  separate measured usage from assumptions,
  and compare hybrid allocations against the ceiling.
- Account visibility:
  recheck available bridges for aggregate usage,
  storage,
  trial expiry,
  and spending controls;
  identify the precise missing authorization if access remains blocked.

The desktop player does not require a physical GPU:
its Slint build includes the software renderer,
and the successful Alpine graphical smoke run used the compositor's `--headless --gpu software` path.
The absence of a cloud GPU does not block desktop interaction testing.
Android emulator graphics requirements are a separate environment constraint,
not an application requirement.

The desktop prerequisite workflow initially failed because its task file was outside the configured discovery roots.
The task was moved into the desktop package's existing `mise.toml`;
the rerun is `38826610100`.
The desktop prerequisite rerun passed:
the Depot GitHub runner has a running systemd user manager and noninteractive sudo.
Full interaction run `38826827023` built the app and compositor,
rendered through llvmpipe,
and passed keyboard/playback/audio-routing checks,
but failed pointer/text checks and aborted during the wide session.
The trial environment and the unexercised scenarios still need investigation.

Android artifact run `38826684750` passed.
Both cloud-built APKs installed on a private headed Android 37 x86_64 emulator with 16 KiB pages.
The unfiltered instrumented invocation reported 16 cases and two `SessionStoreTest` failures:
`SessionStore.prefs` dereferenced a null application context from the test APK's context.
The raw initial result was nine passes,
five skips,
and two failures.
Synthetic audio fixtures and permission were then supplied:
the rerun has fourteen passes,
two failures,
and no skips.
The [device evidence](../audit/depot-android-device-2026-10-15.evidence.json) records individual statuses.
The isolated test-only context experiment from trial commit `bf426f7c13`
passed all 16 cases without skips and its emulator was shut down.
The initial desktop direct-session run `38828362014` failed before E2E execution because the compositor path was absent.
The corrected run `38828801464` exercised the software-rendered app and reached the wide-session folder fixture timeout.
Run `38830049770` collects every session independently.
No passing full-device-suite claim is made for the unchanged tests.

The [coverage inventory](../planning/depot-expanded-coverage.md)
and [scheduling evaluation](../planning/depot-rolling-budget-evaluation.md) are complete.
The scheduling evaluation measured the completed live gate:
331 seconds counting,
922 fetched runs,
and 790 sequential job-history results.
It found no validated equivalent replacement for the rolling budgets.
The subsequent isolated GraphQL query repair preserves the old script's admission decisions
across 13 named comparisons and 32 deterministic variations;
11 incomplete-history and quota cases reject admission.
Its actual YAML script visited 959 live runs with ten GraphQL requests,
ten points,
and zero REST requests,
rejecting the event at the eight-analysis day limit.
The single live sample took 33.2418 seconds and is not a controlled latency comparison.
The query passed with the workflow's own token in run `38830412356`.
The exact candidate then passed `38831327297` and production verification `38831778986`.
The candidate retains the existing creation-time,
latest-attempt,
cancellation,
and absent-analysis accounting boundaries.
This is a narrow history-query repair,
not adoption of a Depot scheduler.
The seven-day target-workload replay subsequently completed,
including the failed and successful full-desktop duration scenarios.

The user supplied account settings after browser discovery again returned no available browser.
The USD $200 plan reports 36 container minutes,
3,804 GitHub Actions minutes,
and 29 Depot CI minutes used.
Selected hard-start limits are 5,000,
20,000,
and 20,000 minutes respectively;
the displayed text says new builds or workflows fail to start when a limit is reached.
The copied storage figures are 25.1 GB for an Intel cache view,
2 GB in current usage,
and 1 GB average in a build-history view.
Their scopes and time aggregation have not been reconciled.
Trial expiry is still displayed as seven days,
without an exact timestamp.
No account settings were changed.
Production deployment remains outside the trial's authorization.

## Interview process correction

### Reopened research execution queue, October 18

The user rejected research readiness.
Do not resume grilling until the expanded repository and vendor coverage has actual evidence.
The parent owns native retry and parallel semantics,
physical test shards,
service and job-container consumers,
the current-source VM controls,
the local Android artifact consumer,
and the integrated decision and cost report.
The active coverage researcher owns the remaining non-module Node profiles
and the corrected Android APK pair producer.
The cache researcher owns the paired cache-off,
warm,
fault,
and invalidation consumers.
The platform researcher owns BuildKit exports,
cache mounts,
and reconciliation of deployed documentation with the first-party source repositories.

The native VM consumer `zg88jz25gb` passed guest execution,
the nonzero-stderr control,
byte transfer,
and explicit domain and daemon cleanup.
GitHub runner VM controls are next after service consumers release a parent cloud slot.
GitHub service run `39109462308` passed host HTTP consumers on its three declared platforms;
job-container results remain pending.
Native secret control `c8lwr60rdm` observed an absent branch-scoped secret
and empty ref context in this manual run.
Run the repository-only variant control before interpreting native container compatibility.

The private Android unit is `mono-depot-ci-android-20261018`.
Its private launch metadata is in the parent's scratch `android-local/launch.json`.
The exact debug APK from `qvsjm5hmdp` is being installed and launched;
the corrected Android producer must supply the instrumentation APK.
Finish with the documented sync,
eight-second wait,
sync,
and private-HOME `adb emu kill` sequence.
Stop only the owned adb server after the emulator exits.

Retain public hash-bound receipts for the VM,
matched process timeout controls,
service containers,
and PR cleanup;
render and review the updated documents.
The corrected retry fixture must record late descendants independently of retry timeout renewal.
Run its actual artifact consumer and the delayed parallel cancellation observer.
Review the broader Rust workspace and current-source linter coverage after the Node inventory wave;
the lock-bound published linter prerequisite does not prove a current-source linter build.
Reconcile the final six approved report sections and measured expanded-workload cost paths.

Remove the exact owned GitHub pull secret and native variants after service consumers finish,
then tell the platform researcher to release the shared image and project.
The temporary PR protection and disposable PR base are already removed;
the private cleanup receipt confirms main protection still matches its baseline.

The Codex maintenance clarification is complete in `.out-of-scope/codex-harness.md` only.
The global search/fetch MCP configuration and fresh-client verification are recorded in
[`codex-search-fetch-mcp.md`](../troubleshooting/codex-search-fetch-mcp.md).
No general `AUT` clarification was accepted for that Codex correction.

The proposed research-gap clarification remains pending:
"Usage/migration advice:
enumerate vendor features;
inspect usage/config,
parallel code,
TODOs/workarounds,
suppressions,
policies;
map desired workloads to source and runtime proof before readiness."
Do not edit `AGENTS.md` without the user's acceptance.

The user clarified that the migration is not apples to apples.
The reports now distinguish controlled allocation measurements from the value and cost of expanded CI capabilities.
The user approved this replacement for existing `AGENTS.md` rule `EVL`:
"Usage/migration advice:
inspect usage/config,
parallel implementations,
TODOs/workarounds,
suppressions,
and policies.
Evaluate desired capabilities and target workload,
not just today's jobs."
The replacement is 192 characters after whitespace normalization and preserves the existing evidence categories.

The agent answered the larger-runner follow-up with an unexecuted measurement proposal,
despite the authorized trial and the user's request to use its resources.
The user explicitly directed execution.
The missing experiment has completed with all corrected configurations passing.
Proposed replacement wording for existing `AGENTS.md` rule `PXQ`,
pending acceptance:
"Complete the tracked queue.
Within an authorized trial,
answer measurement follow-ups by running the missing comparison.
Stop only at completion or a genuine blocker,
never at a proposed next step."

Q13 expanded the interview into agent working habits.
The user rejected that topic as irrelevant to Depot,
and the interviewer withdrew the question.
Proposed clarification to existing `AGENTS.md` rule `AUT`,
pending acceptance:
keep interview questions within the requested decision;
observed symptoms do not authorize adjacent workflow redesign.

The interviewer stopped after drafting the trial proposal despite unresolved decisions.
The user asked for further questions even if they might answer that they do not know.
Delegating technical judgment does not confirm that the interview is complete.
The grilling skill requires an empty decision frontier and confirmation of shared understanding before implementation.
Proposed clarification to existing `AGENTS.md` rule `ASK`,
pending acceptance:
"Non-measurable facts:
ask.
An uncertain answer or delegated technical judgment does not close unresolved preferences;
offer concrete recommendations and confirm shared understanding."

The interviewer asked for hypothetical rate-limit preferences before inspecting the existing policy and run history.
The user corrected this;
subsequent questions should start from measured repository examples.
Proposed clarification to existing `AGENTS.md` rule `QF1`,
pending acceptance:
require relevant configuration and recent execution history to be inspected before asking behavior questions
that those artifacts can answer.

## Independent review correction and remaining execution, 2026-10-18

The independent Rust-lane reviewer verified all nineteen root policy/process/MVM ZIPs and 566 members.
It identified a substantive MVM scope error:
the 51-byte push/pull check reads and writes the host shared directory,
without separately asserting guest payload visibility or guest writes.
The working report,
MVM receipt qualification and KVM troubleshooting note now retract the stronger interpretation.
The root lane must execute an explicit guest read,
distinct output creation and transformation consumer on native Depot,
Depot-managed GitHub and standard GitHub,
with original-source identities and cleanup retained.
This is an active factual task;
it is not a future production acceptance condition.

The coverage lane also owns current-source desktop/logger Mac and Windows controls after desktop and packed consumers.
Historical cross-platform owner/dependency hashes and locked toolchains differ from the frozen expanded source.
Use standard public-GitHub runners sequentially,
with maximum thirty-minute jobs and a retained admission ledger;
record actual runtime versus compile-only boundaries and preserve source failures.
The root lane retains ownership of the coherent Linux graph and repeated whole-graph timing comparison.

## Remaining execution ownership, 2026-10-18 13:55 UTC

The ARM lane completed native `p4xj5dsh3s`:
89 Truepeak passes and two deliberate skips match all 91 AMD64 result identities,
and the actual `p-limit-fork` owner matches all eleven runtime groups.
Its source and archive checks are published in
[`depot-arm64-workload-research.md`](../planning/depot-arm64-workload-research.md).
The platform lane continues substantive applicability/source-ledger reconciliation without new allocations.

The independent reviewer completed the ten service and eleven disposable-PR original archives,
with fresh App identity,
merge-parent,
base-absence and unchanged-main-policy checks.
That lane now owns a separate all-product normalized allocation ledger,
deduplicated by actual provider/product/run/job/attempt or VM/build identity.
Missing billing boundaries and conservative reservations remain explicit.
The new guest-transformation archives remain in its review queue after publication.

Root's guest-file GitHub run `39132577785` did not reach a runner:
the generated label was silently templated into `$ + value +`.
The exact owned-run API showed queued jobs and empty runner assignments.
Root is cancelling that attempt,
retaining it as a generator failure,
and correcting the generator with a final-workflow-field guard before the bounded replay.
The lifecycle's frozen application source remains unchanged.

Root created isolated branch `research/depot-expanded-graph-20261018` at the frozen expanded source.
It owns the shared Node producer,
actual module/browser consumers,
Rust owner and final aggregation graph,
with unchanged repetitions and retained end-to-end completion times.
No graph allocation has been dispatched yet.
The coverage lane retains the corrected desktop,
proper packed-install and current-source Mac/Windows queue.

[release-run]: https://github.com/Aquaticat/Monochromatic/actions/runs/38785821860
[runner-docs]: https://depot.dev/docs/github-actions/overview
[ci-docs]: https://depot.dev/docs/ci/overview
[build-docs]: https://depot.dev/docs/container-builds/overview
[gate-run]: https://github.com/Aquaticat/Monochromatic/actions/runs/38806855827/job/116669815274
[depot-orchestration]: https://depot.dev/blog/building-ci-with-durable-lambda
[github-cache]: https://docs.github.com/en/actions/reference/workflows-and-actions/dependency-caching
[github-billing]: https://docs.github.com/en/billing/concepts/product-billing/github-actions
[depot-pricing]: https://depot.dev/pricing
[ontario-wage]: https://www.ontario.ca/document/your-guide-employment-standards-act-0/minimum-wage
[overlap-windows]: https://github.com/Aquaticat/Monochromatic/actions/runs/38765128213
[overlap-alpine]: https://github.com/Aquaticat/Monochromatic/actions/runs/38765128314
