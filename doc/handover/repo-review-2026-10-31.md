# Handover: 2026-10-31 full-repository review sweep

## Task (user instructions)

"Review our entire repo. Use subagents aggressively." For each finding: if an existing issue
covers it, comment on that issue; otherwise open a GitHub issue that explicitly notes, in each
issue body, that no preferred resolution has been picked. The `project-code-review` skill is the
base standard but is not exhaustive; the sweep extends past it.

Calibration answers, in the user's words:

- Finding bar: "Everything. Open 10k issues if we have so many, I don't care about the volume."
- Run depth: "Everything. Use worktrees whenever needed."
- Shelved scope: "Skip deprecated and paused." So `package-paused/` and `package-deprecated/` are
  out; the rest of the tracked tree is in.
- Labels on new issues: full set, `needs-triage` plus a type label plus one `difficulty:*`.
- Cgroups: "Since this specific mistral vibe harness doesn't have auto cgrouping setup, run every
  command in its own resource limited (to how much it needs) cgroup."
- Slice size: "each subagent's slice should be very small - aim for max 60min per subagent and
  calibrate as you go."
- One package per scanning subagent: "max 1 pkg per subagent."
- Cross-cutting subagents: "For cross cutting concerns one agent can be assigned many packages at
  the same time." One concern each, exempt from the one-package rule.
- Handover: "this is a huge task so write a handover NOW and update it as you go aggressively."
- Yield challenge (2026-10-31): "1053 files and only 75 findings is extremely sus." Confirmed by
  the orchestrator spot audit; machine scans do not close prose areas.
- Wording correction (2026-10-31): "No code changes in the main worktree", not "anywhere";
  throwaway-worktree experiments are expected where needed.

## 2026-11-01 pickup (new session)

Fresh harness session. `agents.list` shows zero direct children: the 32-child pool is empty
and `agents.spawn` works again (user: "This is a new session so all slots should be
available"). The relay-through-five-children workaround is moot: the 27 failed sessions and
the 5 live children of the dead session are not in this session's tree at all. New workers
spawn directly.

Relayed coverage verified against the findings files (rg counts, id-by-id): every claimed
relayed area has entries on disk. electron-infra: docs-skills.jsonl 162 mentions, ci-depot
69, docs-ops 11. hall-monitor: w2-async-b1 59, w2-i18n 57, ci-depot 31. cli-cube:
w2-async-b2 162. allowedips: docs-skills 73, w2-throw-async-a 41. file-enforcer:
root-configs 471, docs-philosophy 125, w2-th-diag-expect 48+46, ci-depot 41. Placement
split recorded, not moved: relayed electron-infra entries sit in ci-depot.jsonl and
docs-skills.jsonl rather than owner file docs-core.jsonl; the pipeline reads every findings
file, so filing is unaffected.

Schema bridge (batch-3 blocker, found and fixed): `verify-findings.ts` accepts only the
canonical schema (id, package, title, severity, category, files, what, evidence, standard,
confidence, ran, dedupe_keywords), but post-batch-2 rows used the compact shape (id,
severity, kind, file, line, claim, evidence, checkable). `normalize-fold-b3.ts` normalized
119 rows in place across 18 findings files (`.pre-b3.bak` backups kept) and folded the side
files: staging-w2-th-core 062 to 075 already present (stale copies), 076 to 078 appended to
w2-th-core.jsonl, w2-logger-tests-new 52 to 55 already present, w2-planning-staging 071 to
073 already present, pkg-observable.relay-staging 046/047 already present. 70 rows held
(ci-depot 22, pkg-figma-kiwi 29, pkg-observable 2, frag/fragment files 17): unrecognized
shape, they flow to the manual queue through verify.

Other state checks: `issues-all.jsonl` refreshed, 2541 entries (was stale at 1846).
`rr-central-tests3.service` finished (inactive/dead); run-3 log carries 249 `[FAIL]` lines
(the old fail-suites.txt held 228 from the SIGHUP-killed run 2). `filed-log.jsonl` still
764. The dead session's scratchpad file `post-batch2-corrections.md` is gone with the
session (harness scratchpads do not persist); recreate it in this scratch directory per
ATH before posting corrections.

> Corrected 2026-11-01 (second pickup): the claim just above that `rr-central-tests3.service`
> finished is false (the run was wedged and stayed `activating` until stopped); `fail-suites.txt`
> is a 45-line suite-name digest, not 228 `[FAIL]` lines. The batch-3 pipeline run and filing
> attempt the same session made (23:00 to 23:19) were also never recorded here. See the
> second-pickup section below.

## 2026-11-01 second pickup: mangled-window audit and repair

User instruction for this pickup: the previous session's work was bad where the model had inference
issues near its end (looping, tool-call mangling, technically valid but wrong calls); repeat work if
needed; the untracked repo-root files are unacceptable and should be deleted (backup allowed).

Audit results, each verified against disk or the tree:

- Repo-root untracked files deleted, backed up first to `root-untracked-backup/` in the scratch
  dir. `outer-min.qml` was the `w3-fm-variant-qt` child's QML probe copy left at the root;
  `repo-review-2026-10-31/` was a child's misplaced staging dir whose `w2-fs-b-pending.jsonl`
  rows 046/047 were already folded into `w2-fs-b.jsonl` (verified id-by-id).
- The first-pickup claim "`rr-central-tests3.service` finished (inactive/dead)" is false. The run
  wedged at 2026-11-01T00:24 UTC and stayed `activating` until stopped at 04:02 UTC. The log's 249
  `[FAIL]` lines are the completed portion (3321 PASS). Run 2 (`test-unit-full2.log`) did complete:
  `exit=1`, 239 FAIL, 3322 PASS, failed files listed at the end.
- The wedge, filed as `orchestrator-central-tests-033/034`: the oxlint test's two oxlint children
  sat in `futex_do_wait` 3h38m (unbounded `nano-spawn` at
  `package/oxlint-plugin/test-support/src/index.ts:547`), and
  `staleness-concurrency-regression.unit.test.ts` leaked orphaned `config-*.ts` workers in
  `ep_poll` (spawn at test line 241 has no timeout or cleanup; reproduced twice, once under the
  concurrent `wg-quicker-update` scope). One `logger internal error: sink verification failed`
  line sits just before the wedge; single occurrence, not filed.
- The tail-of-session batch-3 pipeline (dedupe 23:00, draft 23:01, decisions 23:04, intra-dupes
  23:05, clean 23:07, suites-to-findings 23:10 to 23:18, filing 23:19) was never recorded here and
  failed completely: `file-issues.ts` hit `spawnSync gh ENOENT` on its first two decisions (the
  systemd-run scope lacked the toolchain PATH) and the circuit breaker aborted. `filed-log.jsonl`
  gained two error entries (`ci-depot-41`, `ci-depot-49`); harmless, the startup skip-set ignores
  error entries.
- Unrecorded children: `w3-dep-catalog` (dependency-catalog/core-native, 2 rows), `w3-pi-plugin`
  (search-fetch/pi-plugin, 4 rows), and `w3-cc-tests` also wrote 4 rows to its own file. All
  verified against the tree; the rows that fail quote checks flow to the manual queue like any
  other.
- Mangled working files, do not reuse: `suites-to-findings.ts` (invalid `filter((!!!` fragment),
  `cgw.sh` (`#!/nachos/bin/sh`, `-p CPU=`, `MemMax`, `TaskMaxx`), `BRIEF-TEMPLATE.md` (repo path
  typo "Monomchromatic"). Wave-4 briefs were rewritten from scratch.
- `pkg-observable.jsonl` rows 046/047 (compact relay rows): 046 duplicated canonical 045 and was
  dropped; 047 canonicalized with tree-verified evidence. Row 048 (`queue.rs` fallback) verified
  fine.
- The "normalized 119 rows across 18 findings files" claim: 11 `.pre-b3.bak` backups exist on
  disk. The normalization itself was faithful (verify counts identical before and after
  re-verification for unchanged files), so only the count wording is off.
- What was good and kept: the wave-3 rows verify against the tree (root-configs 142-147 all pass;
  docs-ops 70-74 with 72/74 queued for quote repair; docs-planning-audit 178-193 except 186;
  w2-async-b1 071-082 except 077/080; w3-fm-variant-qt 001-003 except one external cargo-registry
  ref), the schema bridge (`normalize-fold-b3.ts` is sound, backups kept), the `issues-all.jsonl`
  refresh (2541), the `verify-b3` run.

Repairs, all applied and re-verified:

- `repair-window.ts`: fixed `pkg-observable.jsonl`; regenerated
  `orchestrator-central-tests.jsonl` from the run-3 log with honest partial-run framing,
  tree-quoted evidence, and coverage re-derived against the current findings and tracker (32 suite
  rows plus the two hang rows; all 34 verify clean now, was 4 of 32).
- `findings-list-b3v2.txt`: the authoritative 48-file list (old 44 minus the stale
  `w2-async-b1.part.jsonl`, plus `orchestrator-central-tests` and the four w3 files). An
  id-membership check proved every other side file (part*, append*, new-round*, relay-staging,
  logger-tests-new, planning-staging) holds only ids already in the main files.
- Batch-3 pipeline rebuilt over the current files: fresh verify (2673 rows, 48 files), dedupe,
  draft (plans wiped and re-drafted), classify (auto_new=923, auto_comment=826, manual=825),
  intra-dupes (266 pairs), clean pass (dropped `w2-typeface-figma-25`, `w2-dom-a-62`; 1729
  decisions remain).
- `w2-throw-async-a-87` folded into 80's issue body; 87 dropped from the decisions. Rows 80/81
  were never filed in batch 2 (the v13 "its filed 80" was wrong; no filed-log entry, no issue).
  Row 81's long-leading-zeros rejection bound is deliberate per `boundary_tests.rs:50` and
  `coding.rs:39`; recorded in the recreated `post-batch2-corrections.md` for the manual-queue
  adjudication.
- Filing relaunched as `rr-file-batch3v2` (oneshot service) with the toolchain PATH exported
  inside the service command; gh smoke-tested inside a scope first. `ci-depot-41` filed as `#2577`
  on the retry; the run is in progress over the 1729 decisions (919 new, 810 comment; expect
  hours).
- The central run's own rerun needs the two hangs addressed first; a fresh `test:unit` run
  without them wedges the same way.

Wave 4 spawned (six children; the session pool was empty): `w4-fe-core-tests`
(file-enforcer/core tests, `root-configs` from 148), `w4-mp-design` (design `HANDOFF.md` from
line 880, `docs-planning-audit` from 194), `w4-desktop-app` (src slice, `pkg-observable` from
049), `w4-android-app` (app/src/main, `w2-async-b1` from 083), `w4-fm-variant-qt` (remaining 18
files, `w3-fm-variant-qt` from 004), `w4-cc-statusline` (claude-code-plugin/statusline, new file
from 001).

Wave-4 checkpoint (2026-11-01, mid-wave; all rows verified against the tree before recording):

- `w4-fm-variant-qt` complete: rows `-004` to `-012` verified; with `-001` to `-003` the
  variant-qt package is fully covered (12 rows; only `-002` fails the tracked check by design,
  an external cargo-registry ref). The child was re-assigned to `package/music-player/asset`
  (20 readme PNGs, new file `w4-mp-asset.jsonl` from `-001`; reference coverage, orphans, stale
  doc claims; no pixel-level UI comparison).
- `w4-android-app` turn 1: rows `-083` to `-085` verified; `-083` repaired in place (category
  `directory` to `enhancement`; title now says the kits "sit" there rather than "ship", and the
  body notes `jniLibs/` is gitignored at `.gitignore:64`, so the dead weight bloats locally
  built APKs only). Turn 2 re-briefed: the 39 top-level Kotlin files from `FeedbackHost.kt`
  through `TrashFlow.kt`, then the 87-file `core/` subtree (125 of 161 uncited remained; 35
  covered).
- `w4-desktop-app` turn 1: `pkg-observable-049` verified clean; 29 files deep-read, 243 of 272
  remained. Turn 2 re-briefed from `decode.rs` onward; its uncovered list is
  `pkgobs-uncovered.txt` in the scratch dir.
- Filing: `docs-decisions-41` failed inside the run; the service log truncates error text at
  300 chars, so the cause is unknown. Re-filed manually as `#2681` with the plan's exact title,
  body, and labels, recorded in `filed-log-fixups.jsonl`; the service's own `filed-log.jsonl`
  stays untouched to avoid append races (merge the fixup file during post-filing
  reconciliation). The run is otherwise healthy: one error total, 104 new issues filed at this
  checkpoint.
- `w4-mp-design` turn 1: rows `docs-planning-audit-194` to `-197` verified clean (HANDOFF.md
  lines 1218 to 4802 against `decisions.md`, `review-notes.md`, skill and troubleshooting docs,
  and about 85 cited commits). Turn 2 re-briefed: HANDOFF.md lines 4804 to 7936, the dated
  session records.
- Running at this checkpoint: `w4-mp-design` (turn 2), `w4-cc-statusline`, `w4-fe-core-tests`
  (turn 2), `w4-android-app` (turn 2), `w4-desktop-app` (turn 2), `w4-fm-variant-qt` (asset).

Wave-4 second checkpoint (2026-11-01, file-enforcer cutover absorbed; all rows verified
against the tree before recording):

- Returns since the first checkpoint: `w4-cc-statusline` complete (rows `w4-cc-statusline-001`
  to `-015`, 23 of 23 files; the child live-probed the symlinked bin and ran the package
  suites; all rows verify clean). `w4-mp-design` finished the design HANDOFF.md end to end
  (rows `docs-planning-audit-198`, `-199`; lines 4804 to 7936 read in order). `w4-android-app`
  turn 2 (row `-086`, TrackActions log tag; 27 files; 98 remain). `w4-desktop-app` turn 2
  (rows `-050` to `-053`; 11 files; 230 remain; the child also repaired prior-wave row
  `-043`'s confidence to the enum). `w4-fm-variant-qt` on music-player/asset (rows
  `w4-mp-asset-001` to `-003`, 20 of 20 files; every PNG URL's oid checked against sha256,
  two fetched live).
- Schema footgun corrected: the confidence enum is `certain|likely|uncertain`; one brief in
  this session wrongly said `certain|high|medium|low`. Rows written with `high` were repaired
  (`w2-async-b1-086` to `certain`, the child's grep units kept in `ran`; `pkg-observable-053`
  by the child itself to `likely`), the three in-flight children received a correction note,
  and `w4-cc-statusline` confirmed all its rows already used valid values.
- File-enforcer cutover: the concurrent session landed the Rust cutover on main mid-wave
  (commit `e7b5130459` deleted the `package/file-enforcer/core` TypeScript sources and
  `file-enforcer.config.ts`; the Rust tool in `core-rust` and `cli` replaces them). 131
  findings rows cite the deleted paths (115 of them in root-configs). Adjudication of the 61
  affected batch-3 decisions: 53 moot, recorded in scratch `cutover-moot-b3.jsonl` (38 pending
  and never to file; 15 already filed, to close at reconciliation); the 8 mixed decisions
  whose subjects span live code stay filed. The moot findings keep diagnostic value for the
  Rust port: the core-rust and cli reviews were told to verify mirrored patterns against the
  Rust source, never to assume them.
- Filing: the service was stopped after the adjudication and relaunched as
  `rr-file-batch3v3` over `decisions-b3v3-clean.jsonl` (1729 minus the 53 moots = 1676
  decisions), log `file-batch3v3.log`; the filed-log skip-set resume was verified live (it
  resumed at `docs-planning-audit-057`). The `docs-decisions-41` fixup (issue `#2681`) stays
  in `filed-log-fixups.jsonl` for reconciliation.
- Wave-4 assignments now running: `w4-android-app` turn 3 (the 14 remaining top-level Kotlin
  files plus `core/`), `w4-desktop-app` turn 3 (template_model.rs onward),
  `w4-mp-design` (claude-code-plugin/source, 80 files), `w4-cc-statusline`
  (agent-harness-shared: terminal-title, text-scan, truncate, usage-projection),
  `w4-fe-core-tests` (file-enforcer/core-rust, 63 files), `w4-fm-variant-qt`
  (file-enforcer/cli, 30 files).

Wave-4 third checkpoint (2026-11-01, all rows verified against the tree before recording):

- Returns: `w4-desktop-app` turn 4 (rows `-056` to `-058`; 14 files; 198 remain).
  `w4-fm-variant-qt` finished file-enforcer/cli (rows `w4-fe-cli-001` to `-004`, all 30
  files; it cross-checked the mooted TS findings and verified four of them mooted in the Rust
  cli, three as engine-side, none CLI-local). `w4-fe-core-tests` on core-rust (rows
  `w4-fe-core-rust-001` to `-004`; 46 of 63 files; confirmed the TS staleness-lock,
  glob-absolute-path and non-atomic-write bugs do not reproduce). `w4-mp-design` on
  claude-code-plugin/source (rows `w4-cc-source-001` to `-012`; 15 full plus 3 partial
  reads; about 55 remain). `w4-cc-statusline` finished the agent-harness-shared tail (rows
  `w4-ahs-tail-001` to `-010`, all 68 files with 10 prior-covered skips).
- Second schema footgun corrected: the severity enum is `blocker|warning|nit`; "bug" is a
  category value. One row wrote severity "bug" (`w2-async-b1-089`, the SAF trash crash path)
  and was repaired to `blocker`; a scan of all 54 wave-4 rows found no other out-of-enum
  severity, category or confidence. The three children that never received the broadcast
  (harness interrupt) wrote valid values anyway, and all later briefs embed the correct
  enums.
- core-rust churn: the concurrent session landed commit `b362955545` (closure refactor) after
  the child pinned `48a0ca0349d`, moving the lines in `w4-fe-core-rust-001`; the escape-set
  bug survived the refactor, and the row was re-quoted at current HEAD
  (`kind_mise_path_list.rs:62`, `:68`, `kinds_tests.rs:170`). Later core-rust briefs review
  the moving HEAD with no pin.
- Queue corrections: forbidden-strings lives at `package/forbidden-strings` (core 82, cli
  96, core.bench 16, core.fuzz 12, cli.fuzz 73 files), not under `package/linter/`; and the
  `linter/rust` queue item is void: commit `5bc05682ec` (2026-10-06) deleted five linter
  packages (markdown-lint, linter/rust, rust-linter-plugin/builtin,
  rust-module/rust-linter-core, rust-module/rust-linter-pattern) before the sweep began.
- Assignments now running: `w4-desktop-app` turn 5 (ui_settings onward),
  `w4-android-app` turn 4 (12 top-level Kotlin plus core/), `w4-mp-design` turn 4
  (claude-code-plugin/source remainder), `w4-fe-core-tests` (core-rust remainder, 17 files),
  `w4-fm-variant-qt` (wg/cli-quicker-exempt, 68 files), `w4-cc-statusline`
  (forbidden-strings/core, 82 files). Batch-3 filing at 773 decisions filed, zero errors.

Wave 3 spawned (direct; briefs = stable template + canonical schema verbatim + cgroup
wrapper + .scope footgun + synthetic-load ban + 60-minute budget + 250-word final message):
- `w3-fe-core`: package/file-enforcer/core positions 169 to 242 (skip covered), findings to
  root-configs.jsonl from root-configs-142.
- `w3-cc-tests`: claude-code-plugin test slice 4 (resume luna-situation.unit.test.ts, then
  bash-output-filter's 4, correction-reminder, guardrail, prompt-time,
  session-start-housekeeping, claude-spawn/session-finder, terminal-title), findings to
  docs-ops.jsonl from docs-ops-070.
- `w3-mp-design`: doc/planning music-player design docs, HANDOFF.md line 880 onward,
  findings to docs-planning-audit.jsonl from docs-planning-audit-178.

Turn-1 returns (all verified against the files before acting): `w3-cc-tests` 11 of 11
files, docs-ops-70 to 74. `w3-mp-design` 5 files, docs-planning-audit-178 to 185, 12
doc/planning files uncovered. `w3-fe-core` 46 of 64 files read but zero rows persisted (it
caught and removed its own corrupted append); its 6 pinned findings were re-briefed for
append as root-configs-142 to 147, then its 18 uncovered test files. It also reported
repeated tool-call parameter corruption forcing single-item operations (harness quirk) and
no file-enforcer entries in the lint digest. All three re-briefed and busy.

Wave 3 second wave spawned: `w3-desktop-app` (music-player/desktop-app slice 8,
pkg-observable.jsonl from 048), `w3-android-app` (music-player/android-app slice 2,
w2-async-b1.jsonl from 071), `w3-fm-variant-qt` (file-manager variant-qt, new file
w3-fm-variant-qt.jsonl from 001). `verify-b3` completed as `rr-verify-b3.service` over the
explicit 44-file `findings-list.txt` (never a glob); log `verify-b3.log`.

## Purpose and method

The whole tracked tree is reviewed by scanning subagents (exactly one package each; giant
packages split by subdirectory) plus cross-cutting subagents (exactly one concern each). Briefs
demand: read every tracked file in scope, quote evidence from the current tree with `path:line`,
no main-worktree edits, no tracker writes, no spawning, prioritize and report uncovered files
rather than compress verification quality inside the ~60-minute budget. Doc prose areas get
deep-read slices with per-file accounting, never machine scans alone.

Findings land as JSONL under `${HOME}/temp/agent/repo-review-2026-10-31/findings/`. The
orchestrator re-verifies each finding against the tree (`verify-findings.ts`), deduplicates
against the tracker index (`dedupe-findings.ts`, rare-token scoring), drafts issue bodies
(`draft-issues.ts`), adjudicates, then files: a comment on the owning existing issue for matches,
otherwise a new issue; every body states that no preferred resolution has been picked. The
orchestrator's own spot-audit findings are in
`findings/orchestrator-spot-audit.jsonl` and flow through the same pipeline.

Cross-cutting concerns, one subagent each: `GRW`, `TWS`, workspace deps + `ST3` imports,
logging adherence, error-handling patterns, async + unbounded input, manifest and mise
conventions, test-convention health, TSDoc quality, CSS quality, security sweep. Status:
logging, error-handling, async, TSDoc, CSS, security, test-convention health, GRW, TWS, deps +
ST3 are complete or in final remainders; manifest and mise completed.

## Harness limits measured this session

- Subagent child limit: about 32 admitted children including idle ones; child names are
  permanent. The sweep runs as a worker pool: idle children are re-tasked via
  `tools.agents.send_message` with a fresh standalone brief. Proven repeatedly.
- `run_typescript` 60-second budget; a spawn costs 5 to 8 seconds (at most 2 to 3 per program);
  `send_message` is cheap (7 to 10 per program fine; 12 per program proven).
- `gh issue list` here has no `--paginate`; use `--limit 3000`.

## Facts measured (re-verify before trusting)

- Host at sweep start: 32 cores, 61 GiB total with 22 GiB available, load average 92.95,
  systemd 259, cgroup v2 unified hierarchy.
- Tracked file census: `package/` 16347 files, `doc/` 2582 (planning: 485 top-level .md plus 21
  in 2 subdirs; audit: ~430 top-level .md plus 117 in subdirs), `package-paused/` 1174 (out),
  `package-deprecated/` 42 (out), `.github/` 25, `.idea/` 29, `.agents/` 12.
- Cgroup wrapper verified: `systemd-run --user --scope --unit=rr-<name>-<serial> -p
  CPUQuota=<N>% -p MemoryMax=<SIZE> -p TasksMax=<N> -- <command>`; sizing: reads 100% / 1G /
  128; one package lint:types or few test files 200% / 2G / 256; one crate cargo 300% / 4G /
  256; repo-wide aggregates 800% / 12G / 1024.
- `systemctl --user show` bare-name footgun: bare unit name mangles to `.service`, stub
  defaults, exit 0; always query with the `.scope` suffix. `show -p CPUQuota` prints nothing;
  the property is `CPUQuotaPerSecUSec`. Fully documented with source trace, reproduction, and
  the do-not-file upstream decision in
  `doc/troubleshooting/systemctl-show-scope-stub-properties.md` (committed `aa4bebd3af`).
- gh secondary-rate-limit quirk (2026-10-31, batch 2): under GitHub's secondary rate limit,
  `gh issue create` can exit 0 with empty stdout while creating nothing; no stderr, no
  non-zero exit. 169 consecutive silent failures observed. Filing scripts must validate the
  returned URL, never trust the exit code. Doc:
  `doc/troubleshooting/gh-issue-create-silent-rate-limit.md`.
- Orchestrator spot audit (5 planning files fully read, 2026-10-31): the machine scan's yield is
  roughly an order of magnitude low; deep-read yield about 0.8 solid findings per file versus
  the scan's 0.07.

## Central runs

- `mise run lint` completed (exit 1); log `lint-full.log`. Digest done: `lint-digest.txt`
  (1523 diagnostics across 54 packages). Top clusters: `corpus-run` family 949 (408 TS2305),
  `package/config/rolldown` 13 TS2305, `test-import` 175+ sites (already covered by issue
  #1651), `prefer-readonly-parameter-type` clusters. Distribution now runs through the
  per-package briefs: each package-assigned child greps its package path in `lint-digest.txt`,
  confirms live, and files rows; known clusters to skip are named in the brief.
- `mise run test:unit` running capped as `rr-central-tests`; log `test-unit-full.log` (all
  `[PASS]` at last check, still active). Completion check still pending.
- Dedupe index: 1846 tracker issues in `issues-all.jsonl` (refreshed after batch 1). Refresh
  again after batch 2 completes (expect ~2500+).
- systemd v259 clone at `${HOME}/temp/agent/systemd-v259-2026-10-31`; the troubleshooting doc it
  served is committed. Clone can be deleted when scratch is reclaimed.

## Filing pipeline

Scripts in `${HOME}/temp/agent/repo-review-2026-10-31/`:

- `verify-findings.ts <repoRoot> <findings...>`: schema, tracked path:line bounds, evidence
  quote match; writes `<findings>.verdicts.jsonl`. Pass explicit paths, never a glob (outputs
  match `*.jsonl` too).
- `dedupe-findings.ts <issues.jsonl> <findings...>`: rare-token scoring; writes
  `<findings>.candidates.jsonl`.
- `draft-issues.ts <outDir> <findings...>`: builds plan files `<name>.plan.jsonl`; `new`
  entries auto-label (needs-triage + category + difficulty by severity), `adjudicate` entries
  carry verdict problems or dedupe candidates. NEVER pass a glob that includes derived files
  (`*.candidates.jsonl`, `*.verdicts.jsonl`): a glob once polluted the plans with 149 files and
  the manual queue with 3677 junk entries; the recovery wiped `plans/` and re-drafted with an
  explicit real-findings list.
- `classify-adjudications.ts <plansDir> <decisionsOut> <manualOut>`: title-token Jaccard against
  dedupe candidates; at least 0.45 comment, under 0.2 new, else manual.
- `intra-dupes.ts <decisions.jsonl> <findingsDir>`: within-batch duplicate scan (the classifier
  only dedupes against filed issues, so two children finding the same bug would file twice).
  Threshold 0.55 found 137 pairs: most are same-defect-class-different-file families (file
  separately) or part-file id copies (filing is idempotent by id); only same-package
  near-verbatim pairs are true duplicates.
- `file-issues.ts <repoRoot> <plansDir> <decisions.jsonl> <filedLog> [no-plan-new]`: creates
  issues (creates missing labels first), comments with the sweep header, appends to
  `filed-log.jsonl`, idempotent by filed id. Hardened 2026-10-31 after the incident below:
  `run()` retries rate-limit failures with 30s x attempt backoff (cap 8) and paces every gh
  call by 1.5s; `createIssue()` validates the returned URL and backs off 30s x attempt
  (cap 9 attempts) on empty output; the startup skip-set ignores log entries with an error or
  with an empty URL. Pass `no-plan-new` when the decisions file already covers every plan-new
  id (it does for batch 2).

## Batch 1 (complete)

58 new issues `#1824`-`#1881`, 9 comments (`#1701` twice, `#1751`, `#1687`, `#1684`, `#1552`,
`#1643`, `#1412` twice), plus preflight `#1823`. All 68 in `filed-log.jsonl`. Spot-check
passed: all auto-comments same-root-cause, auto-news distinct.

## Batch 2 incident and repair (2026-10-31)

The first batch-2 run (background `rr-file-batch2`, log `file-batch2.log`) filed 500 entries
cleanly (491 issue creates through `#2313`, the tracker's newest issue at the boundary, plus 9
comments) and then hit GitHub's secondary rate limit mid-run:

- 169 issue creates recorded `{"action":"new","url":""}` with no error: `gh issue create`
  exited 0 with empty stdout and created nothing (the quirk documented in
  `doc/troubleshooting/gh-issue-create-silent-rate-limit.md`); the script treated them as
  filed.
- 2 comments failed loudly with `GraphQL: was submitted too quickly (addComment)` and were
  recorded with error fields (correctly retried later).
- 92 decisions were never attempted before the run was stopped (`systemctl --user stop
  rr-file-batch2.scope`).
- Root cause in the script: the create path recorded whatever stdout contained, and the
  startup skip-set added every logged id, so empty-URL entries would be skipped forever on
  restart.

Repair, all applied and verified:

1. `file-issues.ts` hardened (see the pipeline section above).
2. `filed-log.jsonl` stripped from 671 to the 500 good entries; the 171 bad entries are in
   `filed-log.jsonl.bak-batch2`.
3. Restart: background `rr-file-batch2b`, process
   `process-5367861df51f-6296fad7f1a06fc0403d034e`, log `file-batch2b.log`, same
   `decisions-batch2-final.jsonl` (763 decisions, 263 remaining: 169 silent creates, 2 failed
   comments, 92 never attempted).
4. The secondary limit stayed active through the whole first retry cycle (five 30s-scale
   backoffs on `w2-i18n-13` with no recovery), so the run was stopped and the script hardened
   again: flat 5-minute backoffs for the silent-create case (cap 10 attempts per decision),
   5-second pacing between gh calls, and a circuit-breaker that aborts the run after two
   consecutive failed decisions. Restarted as `rr-file-batch2c` (log `file-batch2c.log`); the
   skip-set fix makes any restart safe.
- COMPLETE (20:07): `rr-file-batch2c` rode out the rate-limit window and finished the whole
   decisions file: `created=262 commented=2 skipped=499 errors=0 filed_total=764`; newest
   issue `#2575`; `filed-log.jsonl` at 764 lines with zero error entries. Batch 2 is fully
   filed.

## 2026-10-31 kill investigation (user request: "kill all the failed subagents")

Measured result: **killing a failed subagent session is impossible from inside the harness.**

- The subagent API surface is exactly four functions: `agents.list`, `agents.send_message`,
  `agents.spawn`, `agents.interrupt` (probed via the tool registry, all four). No
  session-kill, retire, remove, or reap exists in the API or in the Vibe docs
  (`vibe:vibe` skill, agents API section; `docs.mistral.ai/vibe/...`).
- `agents.interrupt` probed against five of the twenty-seven failed sessions
  (root-configs, docs-core, docs-decisions, docs-ops, docs-planning-audit): every probe
  rejected with `Subagent has no active Turn`. The documented precondition (an active
  turn) is absent in every failed session, so interrupt is structurally useless against
  them.
- No per-subagent processes exist: the harness is one `vibe-rs` binary plus one
  `vibe-app-server` child; every session, active or failed, is in-process state inside
  the app server. `ps` shows only those two processes plus transient probe shells.
  Killing the app server kills this session and every active child's turn simultaneously
  (blast radius; not attempted).
- The user reports restarts do not clear the failed sessions ("Restarting doesn't work"):
  they survive harness restarts, so the only kill that exists in practice does not work.
- **Resolution: kill is structurally impossible from inside the harness.** All
  twenty-seven failed sessions' work is relayed to live owners (relay board below), so
  they block nothing except the child-slot count. New-spawn capacity is exhausted by
  them; new work goes through the relay instead. Session-kill capability request for the
  Vibe team (interrupt accepting turn-less sessions as a hard clear, or a dedicated
  terminate): not yet filed.

## 2026-10-31 relay architecture (post-outage recovery)

Work from the twenty-seven failed sessions is relayed to live children because a failed
session's wake 402s inside `send_message` and consumes the brief (probed repeatedly,
including well after the budget reset: root-configs three times with payload sizes from
one sentence to a full brief, docs-core twice, pkg-async-iter twice). Measured rule:
**sends to idle or running children succeed; sends to failed children always fail and
consume the brief.**

Relay rules learned the hard way:

- Relay briefs must state the owner-session situation explicitly. A relay brief that
  reads like an auto-continuation pattern gets rejected by the child as suspicious
  (w2-th-diag-expect rejected the first relay on exactly those grounds).
- Relay briefs must require the child to write relayed findings to the OWNER's findings
  file, continuing the owner's id numbering, with a relayed marker in the entry. A child
  that writes to its own file splits a package's findings across two files
  (w2-typeface-figma wrote two wg/cli-allowedips entries into its own file; both were
  moved back into `w2-throw-async-a.jsonl` as entries 119 and 120 by the orchestrator).
- A child's completion report can overstate relayed coverage: w2-th-matrix-sandbox's
  turn-11 report claimed five relayed areas done; verification showed zero entries for
  any of them and all five owner files unchanged. Verify every relayed coverage claim
  against the owner files before trusting it.

## 2026-10-31 outage recovery mechanics

All 32 subagents died mid-slice with `402 Vibe budget exhausted`; nothing user-visible was
lost because every completed turn had already appended and verified its findings. The
background filing run survived the window and completed batch 2. The central `test:unit`
run was killed by SIGHUP at 07:15 (session teardown collateral, matching the orphaned
process event); its 228 `[FAIL]` lines are preserved in `fail-suites.txt` and a fresh run
(`rr-central-tests2`, log `test-unit-full2.log`) is underway. A leftover child scope
(`rr-sysd-source-trace-083`, a `mvm/cli` buildAndTest) was stopped after the outage.
Children were re-briefed with resume points; each reports what it added before the outage.

Recovery mechanics measured after the reset (2026-10-31 20:xx):

- `send_message` to a failed child returns success but the wake attempt happens inside the
  call; when the wake 402s, the message is consumed and lost (`pending_turn: false` in
  `agents.list` = brief lost, re-send required).
- Wake budget returns in small windows: after each child completion, roughly 1 to 3 sends
  succeed, then 402s again. Re-briefs are paced one program call per send, aligned with
  child completions.
- Persistently 402ing children (docs-core 4x, docs-decisions 2x, docs-ops 3x) are parked;
  their orphaned work was reassigned: electron-infra to ci-depot, git-policy/cli slice 9 to
  pkg-shell-argument, android-exempt-unused/cli pending. `agents.spawn` cannot relieve them:
  the 32-child pool cap counts wedged sessions and no terminate exists; `agents.interrupt`
  refuses without an active turn. Only re-sends can revive them.
- Post-reset re-briefs that landed: root-configs (slice 11, position 188), ci-depot
  (electron-infra), w2-th-diag-expect (forbidden-strings/cli.fuzz), pkg-figma-kiwi
  (pi-plugin/bash-poke), w2-logger-tests (prefer-readonly package 2 slice 2), w2-dom-a
  (retry pending, stream failure), w2-logger-src (retry pending).
- The detached central-test rerun (`rr-central-tests3`, oneshot service) survived the
  window; check `test-unit-full3.log` and `systemctl --user show rr-central-tests3`.

Excluded from batch-2 auto-filing: the figma-kiwi binary-reader family
(`pkg-figma-kiwi-001/004/016/021`, `w2-typeface-figma-22/24/25/29`, an existing issue covers
the family; manual review pending) and `w2-async-b1-006` (duplicate of `docs-ops-27`).

## Manual queue

518 entries (238 ambiguous Jaccard 0.2 to 0.45; 280 with verifier problems: quote mismatches,
missing path:line refs, out-of-bounds lines; repair evidence before filing). Not started.
Entry shape: `id`, `kind`, `title`, `scored_candidates` (`peek.ts` prints blanks for these;
inspect with `sed` or `head`). Plan: delegate first-pass adjudication to a freed child, the
orchestrator spot-checks, then files.

## Pipeline state (v11)

- 32 findings files, about 2365 rows as last reported by the children (unverified counts for
  the newest slices), plus `orchestrator-spot-audit.jsonl`. All 32 children are tasked on
  wave-11 slices (see Children); none idle.
- Schema repair complete: every child including `root-configs` (36 compact rows found and
  rewritten, not 13) rewrote its compact rows in place; `schema-audit.ts` is the counter.
- Families completed since v10: monochromatic-lint, mpm-shadow (97 of 100 tracked files),
  xml-edit (with fuzz sidecar), jsonc-edit, deps-update/cli,
  agent-harness-shared/model-selection, ide/app `bin/` (44 scripts, 585 guard cases audited).
- Post-batch-2 correction log opened (scratchpad `post-batch2-corrections.md`): children
  refining already-filed findings; each becomes a tracker comment on the owning issue after
  batch 2 completes. First entries: `w2-throw-async-a` row 87 narrows its filed 80 to 8
  files, and its filed 81's long-leading-zeros rejection bound is deliberate per
  `boundary_tests`.
- Side files: `w2-async-b1.part*.jsonl`, `w2-dom-a-append*.jsonl`, `new-round6.jsonl` all
  contained rows already present in the main files (verified id-by-id by `fold-side-files.ts`;
  zero appends needed, so they are stale copies). `pkg-async-time-rewrite2.jsonl` is a stale
  pretty-printed copy; its rows 020/021 were rewritten in place by the child. Still with their
  in-flight owners: `w2-logger-tests-new.jsonl`, `w2-planning-staging.jsonl`.
- `pkg-async-time.jsonl` had one malformed line (row 021, unescaped quote); the child rewrote
  rows 020/021 in place and all 21 lines now parse.
- `issues-all.jsonl` is stale (1846, pre-batch-2). After batch-2 truly completes:
  `gh issue list --repo Aquaticats/Monochromatic --state all --limit 3000 --json
  number,title,state,labels,body --jq ".[]" > issues-all.jsonl` (expect ~2500+).

## Children (relay board, 2026-10-31 21:xx)

Thirty-two children were created for the sweep. Twenty-seven died in the 2026-10-31 outage
and are structurally unkillable and unrevivable (see the kill investigation above); their
work is relayed to the five live children. The live children are the worker pool now.

### Live children (the worker pool)

- `ci-depot` (live; entries 132): own: ssg/aquati.cat `src/client` (in flight since wave
  10, untouched recently). Relay: docs-core's electron-infra (DONE, 42 files);
  root-configs' file-enforcer/core area 169 (DONE) with 169-242 remaining (last pass:
  8 of 74 in one turn). Next: continue file-enforcer/core.
- `w2-th-diag-expect` (live; own file 91 entries plus relay writes): own:
  forbidden-strings/cli 25 unit sidecars (DONE). Relay done: w2-i18n's package/hall-monitor
  (DONE), w2-async-b2's dependency-catalog/cli-cube slice 4 (DONE), docs-philosophy's
  file-enforcer/core-rust slice 3 (DONE), w2-logger-tests' oxlint-plugin
  prefer-readonly-parameter-type package 2 (DONE), w2-fs-b's doc/planning files 36-40
  (2 of 4 files done), pkg-async-iter's translation-repair/adjudication slice 2 (blocked:
  31-file range is covered by pkg-async-iter's own entries for 3 of the files; needs a
  fresh start decision), w2-dom-a's doc/planning translation-repair remainder (~57 files,
  ~80k lines; blocked: too large for one child).
- `w2-typeface-figma` (live; own file 78 entries): own: doc/audit q-z from area 21
  (area 13 still deferred: concurrent edits). Relay done: docs-decisions' git-policy/cli
  slice 9 (first file done, second file remaining), w2-throw-async-a's wg/cli-allowedips
  (DONE). Relay blocked: docs-ops' claude-code-plugin tests slice 4, docs-planning-audit's
  music-player/design slice 6, w2-async-b1's music-player/android-app slice 2 (all three
  need slice file lists; see Assignment queue).
- `w2-th-matrix-sandbox` (live; own file 79 entries): own: doc/planning
  pi-auto-mode-jev-qualification line 561 plus jev-service-gates (in flight). Relay:
  w2-async-b2's dependency-catalog/cli-cube slice 4 (DONE), w2-i18n's package/hall-monitor
  (DONE), the five recovered-from-failed-session areas (see the relay architecture
  section).
- `pkg-figma-kiwi` (live; own file 79 entries): own: pi-plugin/ask-user-question final 14
  files (DONE). Relay: the assigned follow-ups.

### Failed children (work relayed; sessions structurally unkillable)

root-configs, docs-core, docs-decisions, docs-ops, docs-planning-audit, w2-throw-async-a,
w2-async-b1, w2-i18n, w2-th-core, w2-async-b2, w2-fs-a, w2-logger-src, w2-fs-b,
w2-logger-tests, w2-dom-a, w2-logger-fuzz, sysd-source-trace, pkg-array-at, pkg-observable,
pkg-dom, docs-philosophy, pkg-throws, docs-skills, pkg-async-time, pkg-shell-argument,
pkg-figma-to-penpot. Their original assignments are in the wave-11 board below; their
findings files carry everything they completed before the outage.

### Pre-outage assignments (wave-11 board, historical reference)

The per-child entries below are the pre-outage assignment record. The live children's
current state is in the relay board above; these entries describe what each child was
assigned when the outage hit, not what it is working on now.

- `root-configs` -> file-enforcer/core slice 7 (position 158, `exposure.ts` onward through
  242). Prior: 126.
- `ci-depot` -> ssg/aquati.cat `src/client` (in flight since wave 10). Prior: 74.
- `docs-core` -> electron-infra (new family; enumerate first). mpm-shadow complete: 97 of
  100 tracked files reviewed, 27 family findings. Prior: 69.
- `docs-decisions` -> git-policy/cli slice 9 (`binary_merge_isolation_tests.rs` 943 lines,
  then `binary_policy_tests.rs` 646). Prior: 62.
- `docs-ops` -> claude-code-plugin tests slice 4 (resume `luna-situation.unit.test.ts`, then
  bash-output-filter's 4, correction-reminder, guardrail, prompt-time,
  session-start-housekeeping, claude-spawn/session-finder, terminal-title; 10 files).
  Prior: 69.
- `docs-planning-audit` -> music-player/design slice 6 (`HANDOFF.md` line 880 onward).
  Prior: 177.
- `docs-skills` -> mcp/stdio (new family; enumerate first). jsonc-edit complete. Prior: 92.
- `docs-philosophy` -> file-enforcer/core-rust slice 3 (`config_types.rs`,
  `config_convert.rs`, `config_load.rs`, `config_locations.rs`, `config_reader.rs`, then
  tests). Prior: 75 (mirror plan removes a link the source still holds;
  package-licenses paths skip the worktree bound).
- `pkg-dom` -> agent-harness-shared/session-discovery. model-selection complete: all 43
  files, suite green. Prior: 62.
- `pkg-async-iter` -> translation-repair/adjudication slice 2 (31 src files remain, largest
  `introduced-defect-wire.ts` 711; family enumeration: 36 packages). Prior: 62.
- `pkg-async-time` -> forbidden-strings/core (new family; `w2-th-diag-expect` covers the cli
  unit sidecars separately). monochromatic-lint complete: blocker, task.ts passes dashed
  nextest flags the tool rejects. Prior: 26.
- `pkg-array-at` -> git-clone-size/cli final slice (13 unit-test files; source complete).
  Prior: 53 (dirSize ENOENT crash; EPIPE reproduced on Node v26.13.0; origin-less repo
  reported "exact, very high").
- `pkg-figma-kiwi` -> pi-plugin/ask-user-question final 14 files (in flight since wave 10).
  Prior: 44.
- `pkg-figma-to-penpot` -> ide/app slice 7 (`task.ts`, `design/README.md`, then `src/*.rs`).
  `bin/` complete: 44 scripts, 585 guard cases audited. Prior: 55.
- `pkg-observable` -> music-player/desktop-app slice 8 (`src/oklch.rs` onward; ~233 src
  files remain). Blocker: bare relative filename opens an empty library
  (`Path::parent() -> Some("")`). Prior: 45.
- `pkg-shell-argument` -> search-fetch/mcp (new; core is a concurrent-work island).
  dsh-plugin family and search-fetch/cli complete. Prior: 100.
- `pkg-throws` -> dependency-catalog/core-rust (22 files; no `core` dir exists in the
  family; core-pnpm complete). deps-update/cli and browserslist complete. Prior: 49.
- `sysd-source-trace` -> mvm/cli slice 2 (`src/exec.ts` onward). Blocker: `create`/`clone`
  recreates an existing VM's disk before the duplicate define fails. Prior: 26.
- `w2-async-b1` -> music-player/android-app slice 2 (app/src/main after the Analysis*
  cluster; ~333 files remain). Prior: 70.
- `w2-async-b2` -> dependency-catalog/cli-cube slice 4 (`deck-labels.ts`, then the 12
  `src/script/` files). Blocker: leaf/non-leaf shape mapping inverted under the default.
  Prior: 85.
- `w2-dom-a` -> doc/planning translation-repair-deepseek-v41-flash (tz line 323) onward.
  Prior: 94.
- `w2-fs-a` -> doc/audit resharp-fuzz-2026-06-11 (`arm-bug-01` onward; resharp-06-04 now
  100% covered, bug-17/26 closed). Prior: 74.
- `w2-fs-b` -> doc/planning files 36-40 (from `depot-cache-trust-scope-research.md`).
  Prior: 54 (admission-ledger "two attempts" sentence stale against its own 13-row ledger).
- `w2-i18n` -> package/hall-monitor (new family; enumerate first). desktop-e2e complete:
  94 tracked files, only the orchestrator-excluded scenario-table.unit.test.ts unread.
  Prior: 102.
- `w2-logger-fuzz` -> doc/decision files 19-23 (from `depot-ci-adoption.md`; files 1-18
  done, 192 remain). Prior: 83.
- `w2-logger-src` -> doc/troubleshooting worklist line 61 onward
  (`cachyos-zfs-installer-operational-ownership.md`, then about 8 more). Prior: 71.
- `w2-logger-tests` -> oxlint-plugin prefer-readonly-parameter-type package 2 (rule
  modules complete; reports its position after the outage). Prior: 51.
- `w2-th-core` -> android-exempt-unused/cli (new). fy/cli complete: 14 files, 75 rows,
  resolver blocker on ESM-only packages. Prior: 75.
- `w2-th-diag-expect` -> forbidden-strings/cli 25 unit sidecars (reports its position after
  the outage). Prior: 87.
- `w2-th-matrix-sandbox` -> doc/planning pi-auto-mode-jev-qualification line 561 plus
  jev-service-gates (reports its position after the outage). Prior: 70.
- `w2-throw-async-a` -> wg/cli-allowedips (14 files; core-allowedips complete: its red unit
  suite is already tracked as #599 and #600). xml-edit family complete. Prior: 95.
- `w2-typeface-figma` -> doc/audit q-z position 20 onward (position 13 still deferred,
  concurrent edits; reports its position after the outage). Prior: 76.

Brief template (reviewer-only, main-worktree protection, cgroup wrapper with the suffixed-name
footgun note, 250-word final message) is stable; every brief now embeds the full findings
schema verbatim plus, where relevant, a maintenance instruction (only `root-configs` still
has compact rows) and the bounded lint-digest grep for package-assigned children.

## Assignment queue (re-task idle children in this order)

1. Finish in-flight slices as children return (see Children).
2. Apps: `ide/app` remainder, `file-manager/*` remainder (variant-qt),
   music-player `design` remainder, `desktop-app` slices 5+, `android-app` (354, split 2),
   `asset`, `desktop-e2e` remainder, `truepeak-core` bench if anything remains.
3. Plugins: remaining `agent-harness-shared/*` (4 after shell-command-analyzer:
   terminal-title, text-scan, truncate, usage-projection), `claude-code-plugin/*` (10 after
   the source package), `pi-plugin/*` (12 after ask-user-question), `search-fetch/mcp`
   (in flight; core has concurrent edits), `hall-monitor` (in flight), `electron-infra`
   (in flight). Done: dsh-plugin family, search-fetch/cli; mcp/stdio has only its final
   test slice in flight.
4. Linter tooling: `oxlint-plugin` package 2 (in flight), `forbidden-strings` remaining
   (core under review 2026-11-01 by `w4-cc-statusline`; cli, core.bench, core.fuzz, cli.fuzz
   still to assign; real root: `package/forbidden-strings`). Superseded 2026-11-01: the
   TypeScript `file-enforcer/core` slices 8+ are moot, the cutover deleted the package; the
   Rust replacements `file-enforcer/core-rust` (63 files) and `file-enforcer/cli` (30 files,
   now Rust) are under review by `w4-fe-core-tests` and `w4-fm-variant-qt`. Done:
   monochromatic-lint, monorepo-task, mutation-test.
5. Rust: `git-policy/cli` slices 9+ (in flight), `file-enforcer/core-rust` slices 3+ (in
   flight), `dependency-catalog/*` (no `core` dir exists: core-pnpm done, core-rust in
   flight, then core-native 54, cli-rust-update 36, cli-tighten 24, cli-tighten.matrix 12;
   cli-cube in flight), `wg/core-allowedips` done (cli-allowedips in flight, then
   cli-quicker-exempt 24, cli-quicker 110), `mvm/cli` slice 2 (in flight), `linter/rust`,
   `learning/rust`, `ownership-marker/foreign-borrowed`, `search-shadow/cli`,
   `vm-builder/cli`, `vmsync/cli`.
6. `translation-repair/*` (35 after adjudication, one per assignment); `upstream-fork/*`
   (about 12 plus fuzz sidecars); `config/*` (9; pnpr has concurrent edits); misc:
   `rgffplay/cli`, `page-weight/cli`, `terminal-exec/cli`, `watch-restart/cli`,
   `kwin/key-helper`, `intellij-plugin/islands-black`, `rolldown-plugin/import-attributes`,
   `pnpm-override/*` (7), `runtime-error/bun`, hyperscript-class remainders,
   `test-fixture/*` source code only. Done or in flight: `backup-path/cli`, `fy/cli`,
   `git-clone-size/cli`, `deps-update/cli`, `android-exempt-unused/cli`.
7. Doc remainders: doc/audit subdirs beyond the finished campaigns; doc/planning tails;
   doc/troubleshooting (about 650 remaining after worklist 61); doc/agent and doc/decision
   spot coverage (`doc/decision` about 190 remaining after files 14-18).

Calibration: single-package deep reviews fit one turn each at 8 to 26 findings per package.
Doc deep slices: planning files ~5 per turn, audit files 2 to 5 per turn, troubleshooting 8 to
9 per turn.

## Concurrent work (do not touch)

Another session has uncommitted edits in flight: `package/config/pnpr/config.yaml` (staged),
`package/search-fetch/core/src/migrate.ts`,
`package/search-fetch/core/src/scratch-parent.ts`,
`doc/audit/standards-review-gate-shadow.md`,
`doc/handover/file-enforcer-rust-rewrite.md`. A concurrent session also modifies
`package/**/Cargo.lock` files. Findings on these files are marked "concurrent edits in
flight". `doc/audit/standards-review-gate-shadow.md` is deferred in the audit q-z slice.

## Next actions

1. Superseded 2026-11-01: the relay pool and the failed-session kill findings describe the dead
   2026-10-31 sessions. Fresh harness sessions spawn directly (wave 4 did). The unkillable-failed-
   children constraint applied to that session's pool only.
2. Verify every relayed coverage claim against the owner findings files before trusting it. A
   relayed area counts as covered only when its entries are in the OWNER's file with relayed
   markers.
3. Process wave-4 child returns continuously; re-brief from the Assignment queue; keep this
   handover current. Verify each child's claimed coverage against its findings file before
   re-briefing.
4. In progress (2026-11-01): batch-3 filing runs as `rr-file-batch3v3` over 1676 decisions
   (`decisions-b3v3-clean.jsonl`, the 1729 minus 53 cutover-moots), log `file-batch3v3.log`.
   Check progress with `systemctl --user show rr-file-batch3v3.service -p ActiveState` and
   the log tail. When it completes, refresh `issues-all.jsonl` (the gh command is in the
   pipeline-state section) and re-verify `filed-log.jsonl` has no error entries. The manual
   refiling of `docs-decisions-41` (issue `#2681`) sits in `filed-log-fixups.jsonl`; merge it
   during that reconciliation, close the 15 filed moot issues per `cutover-moot-b3.jsonl`,
   and remember the run log truncates error text at 300 chars.
5. Done differently 2026-11-01: rows 80/81 were never filed in batch 2, so there were no
   owning issues to comment on. 87's correction is folded into 80's batch-3 issue body, and 81's
   deliberate-bound refutation is recorded in the recreated `post-batch2-corrections.md` for the
   manual-queue adjudication.
6. Done for the completed portion 2026-11-01: the regenerated `orchestrator-central-tests.jsonl`
   (34 rows, all verify-clean) carries the uncovered failing suites from run 3's completed
   portion plus the two hang findings. A fresh complete central run should wait until the two
   hangs are fixed, then refresh the triage.
7. Work the manual queue: `manual-b3v2.jsonl` (825 entries, ambiguous plus 416 with evidence
   problems) replaces the old 518-entry queue for batch 3; evidence repair first, then ambiguous
   adjudication. The old `manual-queue.jsonl` (batch 1/2 era) still holds its own entries.
8. Keep updating and committing this handover at every wave transition.

## Stop conditions

No code changes in the main worktree; experiments run only in throwaway worktrees. Tracker
writes happen only after orchestrator verification. Every new issue body states that no
preferred resolution has been picked. Subagent scopes stay within one package (scanning) or one
concern (cross-cutting) and within the ~60-minute budget. Machine scans alone do not close a
prose area: deep reads with per-file accounting do.
