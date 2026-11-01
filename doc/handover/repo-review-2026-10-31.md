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

Wave-4 fourth checkpoint (2026-11-01, batch-3 filing complete; all rows verified against
the tree before recording):

- Batch-3 filing finished: `rr-file-batch3v3` completed with `created=743 commented=50
  skipped=883 errors=0` (all 1676 decisions accounted; the skip set was the batch-3v2
  progress). Reconciliation done: the service re-filed `docs-decisions-41` as `#2716` after
  my manual `#2681` (error entries do not join the skip set), so `#2716` was closed as a
  duplicate of `#2681`; all three filed-log error entries now have later successes; the 15
  filed moot tracker actions landed (6 closes `#2605` to `#2610` as not planned, 9 follow-up
  comments `#2203` to `#2211`); `issues-all.jsonl` refreshed to 3000 entries, which is the
  `--limit 3000` cap, so the batch-4 dedupe must paginate past 3000.
- Returns verified: `w4-fe-core-tests` finished file-enforcer/core-rust (all 63 files; rows
  `-001` to `-009`; confirmed the TS staleness-lock, glob-absolute-path and non-atomic-write
  bugs do not reproduce, found the toml scalar-over-table and unre-parsed block-append bugs).
  `w4-mp-design` rows `w4-cc-source-013` to `-024` (cost-dismissal done; the child now
  re-reads every quoted line before appending, which caught two concurrent reflows).
  `w4-cc-statusline` rows `w4-fs-core-001` to `-008` (32 of 82; two rows re-quoted by me
  after concurrent commits moved their lines: the mutants Containerfile pin drift persists,
  the rote doc filler persists). `w4-desktop-app` rows `-059` to `-062` (disk_view.rs read;
  177 files remain). `w4-android-app` rows `-090` to `-093` (RustEngine, PlayerController,
  SessionStore, PlaybackService, LicensesPage, TrackFingerprint, TrackMenuPopup; the
  pending-window position reporting adjudicated as designed optimistic handoff).
  `w4-fm-variant-qt` rows `w4-wg-quicker-exempt-001` to `-004` (53 of 68; `-001` is blocker
  severity: the Rust cutover landed without this package, so its three guard tests fail by
  design and the README still describes the pre-cutover world).
- Concurrent-session churn is now the dominant repair source: commits `b362955545`,
  `68415aeb53`, `999752b420`, `2b1073678c` and follow-ups moved lines under four children
  mid-review. All stale rows were re-quoted at current HEAD with a note in `ran`; every
  finding survived its re-verification.
- Assignments now running: `w4-desktop-app` turn 7 (feedback_tests onward),
  `w4-android-app` turn 7 (the last five large top-level Kotlin files), `w4-mp-design`
  turn 6 (guardrail onward), `w4-cc-statusline` turn 4 (forbidden-strings/core remainder,
  50 files), `w4-fe-core-tests` (wg/cli-quicker, 110 files), `w4-fm-variant-qt` turn 5
  (cli-quicker-exempt test files).
- Unblocked next: the manual queue `manual-b3v2.jsonl` (825 entries) is now actionable;
  evidence repair first (re-quote against the current tree), then ambiguous adjudication.
  `w2-throw-async-a-81` should be refuted as deliberate per the recreated
  `post-batch2-corrections.md`.

Wave-4 fifth checkpoint (2026-11-01, all rows verified against the tree before recording):

- Package completions: `w4-mp-design` finished claude-code-plugin/source end to end (rows
  `w4-cc-source-013` to `-035`; 83 of 83 tracked files; the final turn read all 8 remaining
  test files including the luna-handler the concurrent session extended to 1071 lines
  mid-turn; `-035`: the handler-runtime records() helper discards parse errors; file 35 of
  35 verify-clean). `w4-fm-variant-qt` finished wg/cli-quicker-exempt (rows `-001` to
  `-006`, 68 of 68). `w4-cc-statusline` finished forbidden-strings/core (rows
  `w4-fs-core-009` to `-015`, 82 of 82) and moved to forbidden-strings/cli (new file
  `w4-fs-cli.jsonl`, rows `-001` to `-010` verified; the turn-7 `scan:all` invalid-argv
  blocker recorded for filing).
- Queue correction measured: wg/cli-quicker is TypeScript with 138 tracked files (the
  queue's "110 files" Rust label was wrong). Rows `w4-wg-quicker-001` to `-011` verify 11
  of 11; 70 of 138 files covered; the turn-2 cross-check target
  `doc/troubleshooting/node-runtime-watcher-upgrade.md` no longer exists, that check is
  moot; rows 004 and 005 still lack covering tests.
- Manual queue slice 1 complete and orchestrator-verified: `w4-fm-variant-qt` processed the
  first 60 `kind:"problems"` entries (outcome log `manual-repair-slice1.jsonl`): 59
  repaired in place (ci-depot 34, docs-core 16, docs-decisions 5, docs-ops 5; the 22-row
  ci-depot 75-96 block was a different schema, restored mechanically), 1 needs-refutation
  (`docs-core-56`: `reject_unknown_keys` was deleted from model_decode.rs by the
  concurrent rewrite; surviving refs re-quoted, confidence uncertain). My own verify run
  over the four touched files confirms none of the 60 ids are flagged and the remaining
  BAD rows (ci-depot 21, docs-core 4, docs-decisions 1, docs-ops 6) match the child's
  stated pre-existing lists exactly. Slice 2 (next 60) in flight.
- Churn and incident repairs: `w4-cc-source-014`, `-016`, `-025`, `w4-fs-core-002`, `-004`
  re-quoted by exact-text relocation after concurrent commits; every finding survived.
  `pkg-observable-077`'s evidence quote 2 was an off-by-one (cited 471, actual 472) left by
  the child's post-incident unicode-escape rebuild; relocated and re-verified. Footgun
  confirmed and now canonical: `systemd-run` expands dollar-brace sequences in command
  arguments before the wrapped command parses them, so wrapper-arg payloads containing
  such text arrive emptied; keep it out of wrapper args or rebuild from unicode escapes.
- Returns verified and re-briefed: `w4-android-app` rows `-094` to `-097` (FolderRail cites
  deleted design docs, DisplayPath empty-segment guard, TrackListPane menu-key consumption,
  folderScrollIndexFor KDoc stranded; the carried lead closed) with 41 core files left;
  `w4-desktop-app` rows `-073` to `-077` (measure_tests temp_cache header, No-droppings
  cleanup missing wal/shm sidecars, non_spacing_mark flag/skin-tone split, oklch mix
  pseudocode achromatic takeover, pagination pseudocode ordering) with 141 files left
  from output_cpal.rs.
- Assignments now running: `w4-mp-design` turn 10 (claude-code-plugin subpackages beyond
  source and statusline; 70-file zero-coverage set: hook-type, prompt-time, research-agent,
  session-start-housekeeping, stop-reminder, terminal-title, verbose-tool-output, root
  README; new file `w4-cc-plugins.jsonl` from `-001`), `w4-android-app` turn 11
  (Pagination onward), `w4-desktop-app` turn 13 (output_cpal.rs onward), `w4-cc-statusline`
  turn 9 (fs-cli test sidecars and bin scripts), `w4-fe-core-tests` turn 9
  (opensnitch-rules-command.ts onward, 68 files), `w4-fm-variant-qt` slice 2.
- claude-code-plugin coverage accounting (measured): 237 tracked files; source 83 complete,
  statusline 23 complete; the wave-3 test slice's 11 files are the only proven coverage
  among the remaining 131 (that relay is blocked, see the relay board); subpackage
  mention counts in findings files (bash-output-filter, claude-spawn, correction-reminder,
  cost-dismissal, guardrail) are unproven per-file coverage.

Wave-4 sixth checkpoint (2026-11-01, all rows verified against the tree before recording):

- Package completions: claude-code-plugin family complete at 237 of 237 (`w4-mp-design`
  turns 10 to 12; rows `w4-cc-plugins-001` to `-015` verified 15 of 15; the zero-coverage
  70-file slice plus the final 61-file five-subpackage slice; the wave-3 test slice's rows
  all cite `source/` paths so nothing was double-covered; headline: all nine
  marketplace.json source paths say the plural `packages/` where the directory is
  `package/`, the SGD rename commit 413020169e missed them). forbidden-strings/cli
  complete at 97 of 97 (`w4-cc-statusline`; rows `w4-fs-cli-001` to `-013` verified 13 of
  13; headline: the `scan:all` mise task passes `scan .` to a CLI whose only subcommand is
  `compile-rules`, and PERF.md's undated living sections still teach the pre-2026-07-17
  architecture). android-app core/ complete at 84 of 84; all 159 Kotlin files are now
  covered (`w4-android-app` turns 1 to 14; rows through `w2-async-b1-104` verified; the
  package tail, rust/ JNI bridge plus resources and build files, is in flight).
- Manual queue slice 2 orchestrator-verified: 58 unique ids repaired (60 entries;
  `docs-philosophy-019`/`-021` appeared twice), zero refutations; the 17 batch-3 stubs
  `pkg-figma-kiwi-051` to `-067` restored to canonical schema (run 1 had written
  `path:NaN`); docs-ops BAD count dropped from 6 to 4 exactly as reported; my own verify
  run over the five touched files confirms no slice id flagged and every remaining BAD is
  a pre-existing or later-slice stub (the pkg-figma-kiwi 045-050 and 068-080 blocks).
  Slice 3 (from `pkg-array-at-002`; 296 problems entries remain) in flight.
- Repairs since the fifth checkpoint: `w2-async-b1-028` relocated (a concurrent
  crate-release/cli change shifted release-run.ts by +73 lines; quotes re-verified at the
  new lines). `pkg-observable-083`'s pre-compaction append was confirmed single by the
  child before continuing. `pkg-observable-089`'s evidence initially paraphrased to dodge
  an apostrophe that broke the child's shell wrapper; the child rewrote it with a unicode
  escape and recorded the repair in `ran`; verified verbatim.
- Returns verified: `w4-fe-core-tests` rows `w4-wg-quicker-014` to `-023` (23 of 23
  lines; headline row 020: update-hosts.unit.test.ts still scans the
  cutover-deleted `package/file-enforcer/core/src/toolchain`, so both host-coverage tests
  fail at HEAD and the suite is red; also the default-route duplicate-skip miss, the
  XDG_CACHE_HOME empty-string defeat with two cross-package twins, the teardown
  exemption-watcher leak). 120 of 138 files covered; the final tunnel cluster is in flight.
  `w4-desktop-app` rows `pkg-observable-076` to `-089` (oklch mix pseudocode, pagination
  pseudocode ordering, peak_swap tutorial comments, the five-instance scratch_root
  temp-dir comment class, player_content ninth-field omission); 116 files remain from
  relpath.rs. `w4-mp-design` ide/app slice 1 (root files plus six small src modules) and
  `w4-cc-statusline` on the core.bench/core.fuzz sidecars (28 files; cli.fuzz's 73 files
  follow) both in flight.
- Queue state: claude-code-plugin done; forbidden-strings cli done, sidecars in flight
  (core.bench 16, core.fuzz 12, then cli.fuzz 73); android-app tail in flight; wg/cli-quicker
  final turn in flight.

Wave-4 seventh checkpoint (2026-11-01, all rows verified against the tree before recording):

- Package completions: wg/cli-quicker complete at 138 of 138 (`w4-fe-core-tests` turns 3 to
  14; rows `w4-wg-quicker-001` to `-026` verified 26 of 26; the mid-review
  tunnel-bypass.integration-fixture.ts deletion was reverted by the concurrent session, so
  nothing is moot; headline: update-hosts.unit.test.ts still scans the cutover-deleted
  file-enforcer toolchain dir, so the suite is red at HEAD). android-app complete at 354 of
  354 (rows `w2-async-b1-083` to `-107`; the JNI boundary verified 28 external fun
  declarations against 28 Rust exports with matching load codes and panic boundary;
  headline: rust/Cargo.toml claims API 35 while build.gradle.kts sets targetSdk 36).
  forbidden-strings sidecars core.bench (16 of 16) and core.fuzz (12 of 12) complete (rows
  `w4-fs-sidecars-001` to `-009` verified; headline: the raced-pattern README claim is
  false, BUILTIN_RULES carries four `&~` rules; zero seed files are tracked despite the
  committed-seeds claim). cli.fuzz (73 files) in flight.
- Retraction (orchestrator, per the correction protocol): `w2-async-b1-094` claimed
  FolderRail.kt cites `design/candidates/buckets.js` and `artists.js` "which exist nowhere";
  both exist under `package/music-player/design/candidates/` (55 and 82 lines; 96 tracked
  .dc.html mockups beside them). The child's repo-wide find silently failed (QRY class;
  mp-design reported the same refusal). Row deleted from the findings file before batch-4
  filing (now 106 rows, baseline BAD set unchanged); correction recorded in the scratch
  `post-batch2-corrections.md`. Briefs now require a positive control before trusting a
  "no matches" search result.
- Orchestrator relocations since the sixth checkpoint: `w4-ide-app-001` (Cargo.toml 33 was
  a digit transposition; the comment lives at 63), `w4-wg-quicker-025` (quote cited the
  TSDoc delimiter at 153; the sentence lives at 154). Both findings stand.
- In flight: `w4-mp-design` on the ide/app README (lines 1510 to 3120 verified clean;
  3120 to 6801 plus mise.toml and task.ts remain), `w4-android-app` on the music-player
  design remainder (README, candidates.md, Decisions.dc.html, the two JS helpers, then
  the 96 .dc.html mockups; rows to docs-planning-audit.jsonl from -200),
  `w4-desktop-app` rows through `pkg-observable-100` (99 rows verified; 81 files remain
  from startup_failure_tests.rs), `w4-cc-statusline` on cli.fuzz, `w4-fe-core-tests` on
  dependency-catalog (core-rust 22 files fresh, then core-native 54; new file
  `w4-depcat.jsonl`), `w4-fm-variant-qt` manual slice 3 (from `pkg-array-at-002`).

Wave-4 eighth checkpoint (2026-11-01, all rows verified against the tree before recording):

- Package completions: forbidden-strings cli.fuzz complete at 73 of 73, closing the whole
  family (core, cli, core.bench, core.fuzz, cli.fuzz; sidecar rows `w4-fs-sidecars-001` to
  `-013` verified; row 009 self-corrected by the child before return, the seed-path claim
  reads `seeds/` not `seed/`). dependency-catalog core-rust complete at 22 of 22 (the queue
  label said TypeScript; the slice is Rust) and core-native complete at 54 of 54;
  cli-rust-update complete at 36 of 36 and cli-tighten complete at 24 of 24 (`w4-fe-core-tests`;
  rows `w4-depcat-001` to `-018` verified, my own run over `w4-depcat.jsonl` is 18 of 18
  clean; headline rows: declared.ts catch-all leaves the never-accuse contract open, the
  yaml and TOML writers both carry the silent-no-op rewrite hazard, splitSemver Number
  precision). pi-plugin spawn complete at 24 of 24 and guardrail complete at 21 of 21
  (`w4-cc-statusline`; rows `w4-pi-plugin-001` to `-016` verified, 16 of 16 clean;
  headline row 013: the model-retirement README headline counts 1604/551/29 while the
  committed fixture pins 1714/596/32). ide/app slice 1 complete: all 12 root files, the six
  small src modules, README 1 to 6801, mise.toml 1 to 1040 and task.ts 1 to 1983
  (`w4-mp-design`; rows `w4-ide-app-001` to `-011` verified 11 of 11 clean; headline row
  010: mise.toml and task.ts disagree on scratch roots in four inspect tasks and accept a
  different IDE_NATIVE_PROJECT root per runner while README documents only the mise side).
- Manual queue slices 3 and 4 orchestrator-verified: slice 3 42 repaired plus
  `pkg-figma-kiwi-040` needs-refutation (the Aquicalat rename); slice 4 35 repaired, 8
  external-by-design, 17 needs-refutation, and my own verify run confirms 44 of 60 slice
  ids clean with the 16 flagged rows exactly the deleted-subject set (`root-configs-3`,
  `-6`, `-52`, `-076` to `-088`, all citing files deleted by the file-enforcer TS
  cutover). Slice 5 in flight from `root-configs-089` (60 entries, problems-subset index
  240; 116 problems entries remain after it, then the 409 ambiguous).
- Orphan repair pass (rows BAD in the findings files but absent from every queue):
  `pkg-observable-31` (toolchain.lock.json nightly pin moved to line 467 and rolled to
  2026-11-01; `mise.lock` still absent, claim stands), `pkg-throws-18` (Object.assign
  lines 99 to 104 and 107 to 112), `pkg-throws-26` (route 69 to 74, servedMany 114 to
  119), `docs-planning-audit-050` (504 to 511), `-068` (255 to 262), `-069` (311 to 318).
  `pkg-shell-argument-83` retracted: commit e3724a44c9 takes zod from the pnpm catalog,
  so the DM1 finding no longer holds; recorded in `post-batch2-corrections.md`. The two
  remaining BAD rows (`pkg-shell-argument-100`, `pkg-throws-25`) are queued as ambiguous
  and stay for adjudication. `pkg-observable`, `pkg-throws`, `docs-planning-audit` and
  `pkg-shell-argument` now verify with zero unbridged BAD rows.
- Returns verified since the seventh checkpoint: design mockups `docs-planning-audit-200`
  to `-211` (row 207 drift attribution re-adjudicated and rejected after first-run-a's
  preview convention; 208 first-run-a per-arrival caption, 209 SUPERSEDED fold-b/fold-c
  bullets, 210 fold-d/e static demos, 211 fold-f/g missing captions); desktop rows
  `pkg-observable-103` to `-111` (111: the ui_menu_model test name undercounts shortcut
  exceptions); pi-plugin rows `-011` to `-016` (model-retirement complete at 27 of 27 and
  thinking-default complete at 15 of 15); depcat rows `-011` to `-018`; ide/app rows
  `-009` to `-011` (the child caught its own argv-slicing append drop via the row count
  and re-appended 009).
- Orchestrator relocations and retractions since the seventh checkpoint: `pkg-observable-110`,
  `w4-pi-plugin-009` (files 189 to 29, quote 193 to 194), `w4-pi-plugin-013` (fixture
  entryCount line 3 to 4), `w4-depcat-014` (declared.ts 7 to 10).
  `pkg-observable-108`'s probeEstimate half was self-retracted by the child; the filter
  half stands.
- In flight: `w4-mp-design` ide/app slice 2 (src/native, 153 files), `w4-android-app` on
  the design mockups from light-a (row 212 next), `w4-desktop-app` on the remaining 26
  ui_* test files (row 112 next), `w4-cc-statusline` on statusline, terminal-title and
  agent-settled-notification (row 017 next), `w4-fe-core-tests` on cli-tighten.matrix (the
  last dependency-catalog slice; row 019 next), `w4-fm-variant-qt` manual slice 5.

Wave-4 ninth checkpoint (2026-11-01, all rows verified against the tree before recording):

- dependency-catalog family complete: cli-tighten.matrix 12 of 12 (rows `w4-depcat-019`
  to `-023` verified, my own run over `w4-depcat.jsonl` is 23 of 23 clean; headline row
  019, blocker: `IN_CONTAINER_ENTRY` still names the pre-rename
  `/repo/packages/dev-script/...` path, so all 20 scenarios fail with cannot-find-module
  and `test:matrix` is red at HEAD; row 020: runTool's catch returns an empty output and
  discards the child transcript). Family total: 148 files, 23 rows across the five
  packages. `w4-fe-core-tests` reassigned to `learning/rust` (10 files, prose-heavy; new
  findings file `w4-learning-rust.jsonl`); `linter/rust` is skipped as void (the 5bc05682ec
  deletion recorded at the fourth checkpoint).
- Manual queue slice 5 verified and accepted: 30 repaired, 2 external-by-design
  (`w2-async-b2-011`, `-081`, node_modules quotes prose-cited from disk-verified copies),
  28 needs-refutation (`root-configs-089` to `-140`, every cited file under the deleted
  `package/file-enforcer/core/`; rows kept as historical records with confidence lowered
  to uncertain). My own run confirms 32 of 60 slice ids clean with the 28 flagged rows
  exactly the deleted-subject set; the parenthetical-confidence normalization also
  cleared the last schema_bad rows in root-configs. Slice 6 assigned: problems-subset
  index 300, first id `w2-dom-a-74`, 60 entries (56 problems entries remain after it,
  then the 409 ambiguous), plus 12 orphan rows folded into the same brief
  (`sysd-source-trace-16`, `w2-async-b1-002`/`-027`/`-070`/`-083`,
  `w2-async-b2-004`/`-014`/`-017`/`-043`/`-046`/`-052`/`-054`; BAD but absent from every
  queue; `-027` and `-083` cite not-tracked files so the deleted-subject path is
  expected). `w2-dom-a-74`/`-81`/`-85`/`-95` are slice 6's first queue entries and stay
  bridged.
- `w4-mp-design` src/native progress: the first nine files plus annotation_tests.rs
  lines 1 to 120 verified clean with zero rows; every cross-file claim resolved live
  (the i-slint-backend-testing 1.18.1 pin, the ui/search.slint:93 placeholder,
  HIGHLIGHT_PATIENCE against tests/language/support.rs:41, the FIXTURE position claims).
  Resume at annotation_tests.rs line 121; 143 files remain.
- Queue text corrections: the item-5 phrases "file-enforcer/core-rust slices 3+ (in
  flight)" and the dependency-catalog in-flight parenthetical are retired (core-rust
  completed at the fourth checkpoint, all 63 files; file-enforcer/cli at the third, all
  30). git-policy/cli slices 9+ have no wave-4 child holding them; re-assign from this
  queue when a child frees.

Wave-4 tenth checkpoint (2026-11-01, all rows verified against the tree before recording):

- pi-plugin seven-subpackage slice complete: statusline 17 of 17, terminal-title 11 of
  11, agent-settled-notification 9 of 9 (rows `w4-pi-plugin-017` to `-021` verified, 21
  of 21 clean; headline row 017: statusline projectUsagePercent accepts nowMs and
  silently drops it; row 019: terminal-title README and index.ts install examples point
  at nonexistent ./packages/ paths, family-wide, agent-settled-notification's instance is
  issue 1494). `w4-cc-statusline` reassigned to pi-plugin auto-mode (148 files, the
  first of the unproven-coverage subpackages; openai-fast 53, goal 51, bash-poke 48,
  advisor 23, current-time-context 12, cost-dismissal 8 follow).

Wave-4 eleventh checkpoint (2026-11-01, all rows verified against the tree before recording):

- music-player/desktop-app src complete: all 285 src/*.rs files covered or cited (rows
  `pkg-observable-112` and `-113` verified, my own run over `pkg-observable.jsonl` is
  112 of 112 clean; headline row 113: unplayable_notification deliver never drops a
  failed cached connection, contradicting its drop-and-retry docs; row 112: the ui_search
  ingest Trk helper's documented plain track is never built when best_in is out of the
  batch range. The admit-before-deliver ledger behavior is pinned as intended by its
  named test, so no row). `w4-desktop-app` reassigned to music-player/asset (20 files,
  rows continue in `pkg-observable.jsonl` from id 114 with package field
  package/music-player/asset; desktop-e2e 94 and the truepeak-core bench follow).
- Design mockups verified: `docs-planning-audit-213` to `-215` (215 of 215 clean;
  213: candidates.md repeats the D34 white center spacer claim that light-c contradicts,
  214: rows-c's 48px caption under the family arithmetic, 215: scan-b/d catalog cells
  claim 860x~540 geometry the files do not carry). 20 mockups plus the empty-a/b
  residual cells remain (scan-ef, scroll-a, settings-a/b, sub-a/b, tabletop-a/b/c,
  toast-a, unf-a to unf-j).

Wave-4 twelfth checkpoint (2026-11-01, all rows verified against the tree before recording):

- learning/rust complete: 10 of 10 files (rows `w4-learning-rust-001` to `-003` verified,
  3 of 3 clean; headline: RESOURCES.md links to the removed `package/rust-module/` and
  `package/desktop-app/` dirs, and only aquascope-decoder.html carries the NOTES-mandated
  color-scheme meta). ownership-marker/foreign-borrowed complete and clean: 7 of 7 files,
  zero rows, the empty `w4-ownership-marker.jsonl` accounts for the slice (zero-runtime
  claim verified; the no-root-import rule honored tree-wide with a 76-reference control).
  `w4-fe-core-tests` now on search-shadow/cli (31 files; then vm-builder/cli, vmsync/cli,
  mvm/cli slice 2, git-policy/cli slices 9+).
- pi-plugin auto-mode progress: rows `w4-pi-plugin-022` to `-028` verified (28 of 28
  clean after my 027 evidence-format fix: the child omitted the space after the
  line-number colon, which made the verifier misparse the path; briefs now carry the
  format note). 28 of 148 files read; the package-wide link sweep is complete (126
  links, 4 dead, all already covered by earlier rows). Headline row 027:
  architecture.md still describes the removed native implementation as current.
  Remaining: historical-extension-and-migration.md, the five planning docs, the review
  HTML, and the ~120 PNGs for density coverage.
- ide/app src/native progress: row `w4-ide-app-012` verified (12 of 12 clean;
  frame_trace.rs:97's garbled comment obscures that wheel::note_event rides the
  window's single winit event hook). The find family is complete and clean; 133 files
  remain from ignored_row_tests.rs.
- Manual slice 6 (plus the 12 orphan rows) still in flight with `w4-fm-variant-qt`.

Wave-4 thirteenth checkpoint (2026-11-01, all rows verified against the tree before recording):

- music-player design sweep complete: all 96 .dc.html mockups deep-read across
  `w4-android-app` turns 18 to 24, with README.md, candidates.md, Decisions.dc.html,
  buckets.js and artists.js and the empty-a/b residual cells closed (rows
  `docs-planning-audit-213` to `-222` verified, 222 of 222 clean; headline rows 221: the
  ROUND 4 unf-f row describes optional bucket chips where the file renders a two-level
  picker with no sub-buckets, 217: settings-a renders switches D84/D85/D86 record as
  removed with no supersession note, 220: artists.js generates 1,250 artists against
  its ~1000 comment and the catalog's 1,218). `w4-android-app` reassigned to
  rgffplay/cli (9 files; then page-weight 17, terminal-exec 27, watch-restart 43,
  kwin/key-helper 25).
- music-player asset complete and clean: 20 of 20 files, zero rows (row
  `pkg-observable-114` verified, 113 of 113 clean; the byte-level chunk scan, OCR name
  sweep, brightness and dimension cross-checks, and the sha256 gallery verification with
  the retake-history closure). `w4-desktop-app` on desktop-e2e (4 of 94 covered; row 114
  is the actions.ts anyState doc-example nit; 85 src files plus 5 package-level files
  remain, tracked in the child's pkgobs-e2e-uncovered.txt).
- ide/app src/native progress: nine more files clean in turn 20 (through
  language/hint_skip_tests.rs); 124 files remain from language/hover_tests.rs; row 012
  stands as the slice's only row so far.

Wave-4 fourteenth checkpoint (2026-11-01, all rows verified against the tree before recording):

- search-shadow/cli complete: 31 of 31 files (rows `w4-search-shadow-001` to `-004`
  verified, 4 of 4 clean; headline row 001: link removes any entry named like a shadowed
  tool, not just its own symlinks; two hypotheses refuted by live controls: piped-stdin
  grep walks the CWD, and the /dev/null refusal matches real ripgrep). Coordinator note
  recorded for the config-package review: search-shadow's plural `Aquaticats` URLs match
  the live git remote while the singular `Aquicalat/` spelling in about 209 files under
  package/config and dependency-catalog does not. `w4-fe-core-tests` now on
  vm-builder/cli (16 files; then vmsync/cli, mvm/cli slice 2, git-policy/cli 9+).
- pi-plugin auto-mode complete: 148 of 148 files (rows `w4-pi-plugin-029` and `-030`
  verified, 30 of 30 clean; headline row 030: 27 of 114 tracked PNGs are referenced by
  no tracked document, 18 more only via blanket phrases; row 029: the Q14 option label
  divergence in ground-up-rebuild-review.html). `w4-cc-statusline` now on openai-fast
  (53 files; then goal 51, bash-poke 48, advisor 23, current-time-context 12,
  cost-dismissal 8).

Wave-4 fifteenth checkpoint (2026-11-01, all rows verified against the tree before recording):

- Manual queue slice 6 verified and accepted: 67 repaired, 3 external-by-design
  (`w2-fs-a-41` relocation sheets, `w2-fs-b-038` toolchain plan, `w2-async-b1-083`),
  2 needs-refutation (`w2-async-b1-027` deleted-subject flagged as expected;
  `w2-i18n-22`'s staleness claim needs re-adjudication because the concurrent cutover
  rewrote its runbook, FORBIDDEN_STRINGS_RULES now via tracked
  `.config/mise/conf.d/20-env.toml`). My own run over the nine touched files confirms
  71 of 72 ids clean with only `-027` flagged; the queue cross-check found 32 further
  BAD rows absent from every queue (w2-fs-a-36, the w2-fs-b and w2-i18n blocks, the
  w2-logger-fuzz tail, w2-logger-src-67); all 32 are folded into slice 7 as orphans.
  Slice 7, the FINAL problems slice, is in flight: problems-subset indexes 360 to 415
  (56 entries, first id w2-logger-src-9) plus those 32 orphans, 88 ids total. After it
  the manual queue holds only the 409 ambiguous entries for adjudication.
- rgffplay/cli complete: 9 of 9 files (rows `w4-rgffplay-001` to `-003` verified, 3 of 3
  clean; headline row 003: unbalanced query brackets make rg abort with an unclosed
  character class that only the exit-1 path maps to a friendly error; row 001: the
  README attributes the missing progress line to Bun's child_process.spawn while the
  CLI runs on node via nano-spawn). `w4-android-app` now on page-weight/cli (17 files;
  then terminal-exec 27, watch-restart 43, kwin/key-helper 25).

Wave-4 sixteenth checkpoint (2026-11-01, all rows verified against the tree before recording):

- vm-builder/cli complete: all 17 tracked files (rows `w4-vm-builder-001` to `-008`
  verified, 8 of 8 clean after my 005 relocation, Flatpak line 372 to 373; headline row
  001, blocker: `.containerignore` un-ignores the plural `packages/` where the
  directory is `package/`, a live FROM-scratch podman repro fails at the Containerfile
  COPY and the singular variant builds, so the primary run task is red at HEAD; row 008
  joins the Aquaticat/Aquaticats spelling family). `w4-fe-core-tests` now on vmsync/cli
  (28 files; then mvm/cli slice 2, git-policy/cli slices 9+).
- pi-plugin openai-fast complete: 53 of 53 files, one finding (row `w4-pi-plugin-031`
  verified, 31 of 31 clean; the README's present-tense pi 1.0.0 claim against the
  installed 1.4.0 host). `w4-cc-statusline` now on goal (51 files; then bash-poke 48,
  advisor 23, current-time-context 12, cost-dismissal 8).
- desktop-e2e progress: rows `pkg-observable-115` to `-119` verified (118 of 118
  clean; the loadRunConfig example missing defaultCompositorPath, the OMEGA_TONE_FILE
  folder-name mismatch pinned by its test, the narrow-session module-doc enumeration
  gap). 54 files remain from dialog-session.ts; the ansi.ts gap was found and closed.

Wave-4 seventeenth checkpoint (2026-11-01, all rows verified against the tree before recording):

- page-weight/cli complete: 17 of 17 files (rows `w4-page-weight-001` to `-012`
  verified, 12 of 12 clean; headline row 001, blocker: video and audio src never
  counted because ownAssetUrl's media branches are dead code behind the MEDIA_PARENTS
  short-circuit; row 002: xlink:href read under the wrong hast key, legacy SVG use refs
  dropped). `w4-android-app` now on terminal-exec/cli (27 files; then watch-restart 43,
  kwin/key-helper 25).
- desktop-e2e progress: rows `pkg-observable-120` to `-123` verified (122 of 122
  clean after my 123 relocation, the SessionName type moved from line 17 to 22;
  headline row 120: the dialog coverage poll accepts >= while the assert demands >, so
  exact equality passes the poll and fails the check). 39 files remain from
  ring-session.ts.
- ide/app src/native progress: row `w4-ide-app-013` verified (13 of 13 clean;
  surface.rs popup and dismiss pseudocode list signatures the functions no longer
  take); turns 22 and 23 covered the language tail and seven minimap files clean; 97
  files remain from minimap.rs.
- pi-plugin goal progress: rows `w4-pi-plugin-032` verified (32 of 32 clean; the
  goal:review-unavailable renderer has zero coverage across tests and verifiers); 43 of
  51 files read, the 8-file tail in flight.

Wave-4 eighteenth checkpoint (2026-11-01, all rows verified against the tree before recording):

- vmsync/cli complete: 28 of 28 files (rows `w4-vmsync-001` to `-013` verified, 13 of
  13 clean; headline row 001: transferRegion checks 512-byte alignment but computes dd
  skip/seek in 1 MiB units, so fractional skip aborts sync for typical 64 KiB-cluster
  regions, live dd control; row 002: createOverlay overwrites existing overlays,
  destroying unsynced guest writes on retried boots; row 004: the e2e MVM_ARGS path
  points at the pre-move mvm location so the expensive suite cannot start).
  `w4-fe-core-tests` now on mvm/cli (94 files, fresh full review; the queue's "slice 2
  in flight" was stale, no findings existed).
- pi-plugin goal complete: 51 of 51 files across three turns, one finding total
  (row 032 stands). `w4-cc-statusline` now on bash-poke (48 files; then advisor 23,
  current-time-context 12, cost-dismissal 8).
- ide/app src/native progress: the minimap family is fully read and clean (minimap.rs
  through minimap_worker/serve.rs, including the 1457-line minimap_tests.rs with every
  plan() assertion traced against pace.rs); 90 files remain from
  navigation/actions.rs; row 013 stands as the slice's only row since 012.
- Manual queue slice 7 verified and accepted: 77 repaired, 2 needs-refutation
  (`w2-logger-tests-52`: the cited unix run command was fixed on the live tree;
  `w2-i18n-41`: the cited block was removed by the task-catalog restructure), 4
  external-by-design; my own run over the fourteen touched files confirms zero of the
  83 ids flagged. THE PROBLEMS QUEUE IS FULLY WORKED: all 416 problems entries are
  processed across slices 1 to 7, plus every orphan found along the way. The cross-check
  surfaced 20 further orphans (w2-logger-tests-5/-27/-34, the w2-th-core 004/005/055/056/
  058-to-061 set, the w2-th-diag-expect 027/028/048/053/055/064/104/105 set,
  w3-fm-variant-qt-008); `w4-fm-variant-qt` is repairing them and then starting the
  ambiguous adjudication: 409 entries, slice 1 is the first 60 (first id ci-depot-1),
  decisions logged to manual-adjud-slice1.jsonl as duplicate (with owning issue
  number) or distinct, cross-checked against post-batch2-corrections.md and
  cutover-moot-b3.jsonl.

Wave-4 nineteenth checkpoint (2026-11-01, all rows verified against the tree before recording):

- pi-plugin bash-poke complete: 48 of 48 files, zero new rows (every defect already
  covered by existing rows; the four compaction-cut reads were recovered from saved
  results). `w4-cc-statusline` now on advisor (23 files; then current-time-context 12,
  cost-dismissal 8).
- terminal-exec/cli complete: 27 of 27 files (rows `w4-terminal-exec-001` to `-015`
  verified, 15 of 15 clean; headline row 001, blocker: the KDE XDG fallback uses
  config without the dot against xdg-paths's .config, so the kdeglobals fallback never
  works on stock systems; row 002: lower-priority config files override higher ones).
  `w4-android-app` now on watch-restart/cli (43 files; then kwin/key-helper 25).
- mvm/cli turn 1: 27 of 94 files (rows `w4-mvm-001` to `-007` verified, 7 of 7 clean;
  headline row 001: ephemeral-run re-raises SIGINT/SIGTERM while its own listener is
  still registered, wedging Ctrl+C during mvm run, live repro re-entered the handler
  five times; row 002 is another Aquicat/Aquaticats spelling instance). 67 files
  remain from src/backend/hetzner/server-types.unit.test.ts.

Wave-4 twentieth checkpoint (2026-11-01, all rows verified against the tree before recording):

- pi-plugin family complete, all 15 subpackages: advisor 23 of 23 (row `w4-pi-plugin-033`
  verified, 33 of 33 clean; headline: the extension factory's `@example` configures
  `"./packages/pi-plugin/advisor"` against the tracked singular `package/` tree, the one
  occurrence confirmed by scoped git grep). current-time-context 12 of 12 and
  cost-dismissal 8 of 8 closed with zero new rows: every defect pre-covered by the kiwi
  block (`kiwi-060` to `-071`) and `docs-decisions-30`. With spawn 24 of 24 and guardrail
  21 of 21 already done, `package/pi-plugin/*` is fully swept.
- mvm/cli turns 2 and 3: 53 more files, 62 of 94 (rows `w4-mvm-008` to `-015` verified,
  15 of 15 clean; headline row 008, warning: create.ts and clone.ts never check whether
  the destination VM already exists, so `mvm create` into an existing name silently
  overwrites disk.qcow2; my scoped control confirms create.ts has no access()/stat()
  at all and clone.ts checks only the source at clone.ts:122; row 012, warning: a
  failed download pipeline skips controller.abort(), leaking the 500 ms poller timer
  so the CLI never exits; row 015: destroyAll feeds prefix-filtered listVms names
  into a recursive rm without the validateName gate; the LC_ALL=C claim verified true,
  virsh.ts:47 to :116). Backend cluster and the 1112-line cli-file-transfer integration
  test closed clean. `w4-fe-core-tests` resumes at guest-exec.unit.test.ts.
- ide/app src/native: navigation family plus the picture-frame, pointer, query-input,
  quiet-read, refresh-latency and refused-tile test families complete, 26 more files
  over two turns, zero new rows (file stands at 13 clean; every cross-file claim
  resolved live, including the RowMap and TreeEntry splits and the slint 1.18.1
  menu-activation claim checked against the registry source). 64 files remain;
  `w4-mp-design` resumes at render/paint.rs.
- `w4-fm-variant-qt`: orphan repair and ambiguous-adjudication slice 1 both complete
  and verified. All 20 assigned orphans resolved (17 repaired, 3 needs-refutation:
  `w2-logger-tests-27` now uses LFH_FIXED_SIZE at index.unit.test.ts:311,
  `w2-th-core-005`'s Symbol.for key extracted to async-failure-runtime-key.ts:13,
  `w2-th-core-059`'s Aquicalat reference gone from package/crate-release); my verify
  run over the four touched findings files is 254 of 254 rows clean. Adjudication
  slice 1 (ambiguous entries ci-depot-1 to docs-ops-8): 23 duplicate with owner
  issues, 37 distinct, zero undecidable; my control confirms all 23 owner numbers
  exist in issues-all and the sampled pairs match on defect. Caveat for the batch-4
  filer: 18 touched rows had stale evidence in concurrently rewritten files and were
  prose-repaired without re-substantiation, so re-verify the distinct ones live
  before filing (duplicates are moot). Slice 2 starts at ambiguous index 60, first
  id docs-ops-9.
- `w4-cc-statusline` reassigned to the git-policy/cli remainder campaign (queue item 5,
  "slices 9+"): slices 1-8 covered the files before `binary_merge_isolation_tests.rs`
  (docs-decisions rows 53 to 62) and the relay read `binary_merge_isolation_tests.rs`
  clean, so the fresh work list starts at `binary_policy_tests.rs`: 585 subject files,
  153,642 lines (scratch `cc-git-policy-brief-list.txt`), 18 in-range files already
  cited and skipped as subjects. New findings file `w4-git-policy.jsonl` from id 001,
  serial prefix rr-gp, progress ledger `w4-git-policy-progress.md`.

Wave-4 twenty-first checkpoint (2026-11-01, all rows verified against the tree before recording):

- mvm/cli complete: 94 of 94 tracked files, rows `w4-mvm-001` to `-024` verified 24 of 24
  clean by my run. Headlines: 001 the ephemeral-run SIGINT re-raise loop (live repro);
  008 create/clone silently overwriting an existing destination VM (my control: no
  access()/stat() in create.ts, clone.ts checks only the source); 012 the failed-download
  abort leak (no try/finally around the pipeline); 015 destroyAll's ungated recursive rm;
  017 the image cache with no temp-plus-rename (my control: zero rename/mkstemp hits,
  pathExists early-exits at image.ts:75, :197, :320); 019 XDG_CACHE_HOME set-but-empty;
  023 resolveImage accepting inherited Object.prototype keys; 024 update deleting every
  cached template before any replacement is built. Adjudications closed: validateName's
  charset makes the create/clone/destroy interpolations safe, the LC_ALL=C claim is true
  (virsh.ts:47 to :116), the TODO.md exec-args quirk is mise-run's own parsing (no
  finding), autounattend inputs are constants. Progress file closed out.
- `w4-fe-core-tests` reassigned to `package/translation-repair/archive` (92 tracked
  files), first package of the queue's translation-repair one-per-assignment campaign;
  new findings file `w4-tr-archive.jsonl` from id 001, serial prefix rr-tra, ledger
  `w4-tr-archive-progress.md`. My pre-assignment control: the family has 36 dirs plus
  the special adjudication area; all have prior citations (2 to 80 per dir), so prior
  rows matter for dedupe.

Wave-4 twenty-second checkpoint (2026-11-01, all rows verified against the tree before recording):

- watch-restart/cli complete: 43 of 43 tracked files, rows `w4-watch-restart-001` to
  `-012` verified 12 of 12 clean. Headline blocker 001: the README-documented library
  exports gitignoreFilter, hiddenFilter, regexFilter exist in src/filter/ but are never
  re-exported from index.ts (my control: index.ts exports only the four other filter
  helpers). Warnings include 004 (Ctrl+C inside a restart's stopTimeout grace abandons
  the SIGKILL escalation and can orphan the detached child) and 009 (publish-ready with
  a TypeScript-source bin and no engines floor); the cli.ts:165 @mutates claim was
  disproven against @optique/run 1.3.6 source and filed as row 010.
- kwin/key-helper complete: 25 of 25 tracked files, rows `w4-kwin-key-helper-001` to
  `-004` verified 4 of 4 clean (001 package-wide raw console logging against TLG; 002
  async-time-core imported without the /ts subpath, the package's only bare workspace
  import; 003 the long-lived Ghostty instance started through buffered execFile, whose
  default 1 MiB maxBuffer terminates the child, my control confirms execFile at
  launch.ts:211; 004 the autorepeat no-state-change branch untested with a 3-match
  positive control). Issue 1375 not re-filed.
- git-policy/cli campaign split for pace: `w4-android-app` joins `w4-cc-statusline`,
  taking the tail of cc-git-policy-brief-list.txt from `maintenance_hk_config.rs`
  (position 300) through `wrapper_invocation_tests.rs` (585 files, 286 subjects);
  `w4-cc-statusline` continues forward from command_stash_table.rs and stops at
  `maintenance_hk_config.rs`. New findings file `w4-git-policy-b.jsonl` (ids
  `w4-git-policy-b-001` onward), serial prefix rr-gpb; both sides check the other's
  findings file plus docs-decisions rows 53 to 62 for dedupe.

Wave-4 twenty-third checkpoint (2026-11-01, all rows verified against the tree before recording):

- Adjudication slices 1 to 3 complete: 178 of 409 ambiguous entries decided
  (slice 1: 23 duplicate / 37 distinct; slice 2: 9 / 50; slice 3: 20 / 39; running
  total 52 duplicate with verified owners, 126 distinct, zero undecidable). Slice 3
  introduced the census-subsumption check with git-age probes (rows predating the
  censuses) and a legacy-format schema conversion (pkg-dom-040). Slice 4 in flight
  from index 180.
- Corpus BAD enumeration supersedes the stale count: a fresh verify over all 100
  findings files finds 259 BAD rows across 45 files (root-configs 129, mostly the
  slice-4/5 deleted-subject set; ci-depot 18; docs families 28; w2 part files 24;
  figma-kiwi frag 15; remainder scattered). Full id list: scratch
  `mq10-bad-enumeration.txt`. Repair of this set is `w4-fm-variant-qt`'s next campaign
  after the adjudication slices; until then the batch-4 verify pass will funnel them
  to the manual queue.
- git-policy/cli split executing: `w4-cc-statusline` (head, positions 1 to 299) rows
  `w4-git-policy-010` to `-014` verified 14 of 14 clean; the two warnings are the
  campaign's strongest bug rows so far (013: normalise_inline_short splits OPTARG
  attached values, so -S6ABCDEF rewrites to -S -6 -A -B -C DEF and reaches the real
  commit; 014: to_string_lossy corrupts byte-preserving non-UTF-8 commit argv).
  `w4-android-app` (tail, positions 300 to 585) rows `w4-git-policy-b-001` to `-004`
  verified 4 of 4 clean (landing lock held for check against its fix-only docs; two
  stale citations; pushed_scope quadratic dedup, family of w4-git-policy-002).
- ide/app src/native: rows `w4-ide-app-015` and `-016` verified 16 of 16 clean
  (TileWorker's Drop detaches the thread while the field doc promises join-on-shutdown;
  my control: no .join( in tile_build.rs, the "source tile worker detached" log at
  :334). Position 133, 21 files remain.
- translation-repair/archive at 69 of 92 files (rows 001 to 004; the coverage-candidates
  mirror against slice-reading verified branch-by-branch, one roster-pairing residual
  logged for a future caller).

Wave-4 twenty-fourth checkpoint (2026-11-01, all rows verified against the tree before
recording):

- Package completion: `w4-fe-core-tests` finished `package/translation-repair/archive`,
  92 of 92 tracked files, ledger closed (rows `w4-tr-archive-005` to `-008` verified 8 of
  8 clean; the package totals 8 rows with 001 to 004). Headline `-005` (nit,
  documentation): the two-votes test cites the root
  `doc/design-commitments-votes-repairs-and-run-offs.md`, a path never tracked; the real
  rulings live at `package/translation-repair/doc/...` lines 205 to 212. My control:
  `git log --all` is empty for the root path, the tracked `package/translation-repair/doc/*`
  listing is the positive control, and the stale citation sits at
  `archive/src/non-translation-two-votes.unit.test.ts:5`. `-006`/`-007`: two
  `prepare*Pairing` TSDoc blocks omit one param each (`pictureReadings`,
  `sourceEntryInfo`); `-008`: `pictureReadings` and `fanOut` have zero unit coverage in
  this package. The root-path citation family is routed to the other translation-repair
  reviews via `-005`'s dedupe keywords. The child was re-assigned to
  `package/translation-repair/artifact` (178 files, new file `w4-tr-artifact.jsonl` from
  `-001`).
- `w4-mp-design` turn 32 on ide/app src/native: row `w4-ide-app-017` verified, findings
  file 17 of 17 clean. The frame-trace target `monochromatic_ide::native::frame` is
  hardcoded at `frame_trace.rs:4,32,41,65,71`, `viewport.rs:484` and `gpu_tile.rs:504`
  while the crate is `ide_app` (`Cargo.toml:10`), completing the stale-crate-name log
  target cluster with `-014` (`rewrap.rs:230`); my control re-grepped all sites live.
  Positions 133 to 139 covered (tile_slice_frame through viewport); next position 140
  (`waiting_tests.rs`), 14 files remain.
- In flight, unchanged: `w4-android-app` (git-policy/cli tail positions 300 to 585),
  `w4-desktop-app` (desktop-e2e), `w4-cc-statusline` (git-policy/cli head positions 1 to
  299), `w4-fm-variant-qt` (adjudication slice 4 from index 180, then the 259-BAD corpus
  repair).

Wave-4 twenty-fifth checkpoint (2026-11-01, all rows verified against the tree before
recording):

- Adjudication slice 4 complete and orchestrator-verified: 60 ambiguous entries decided
  (indexes 180 to 239), 8 duplicate with owners (pkg-throws-24 #1256, pkg-throws-25
  #1253, root-configs-7 #1640, root-configs-10 #1661, root-configs-47 #1638,
  w2-async-b1-017 #1707, w2-async-b1-018 #1712, w2-async-b2-018 #1142; four of the eight
  owners re-checked present by me), 52 distinct. Running total: 238 of 409 decided, 60
  duplicate, 178 distinct. 12 rows tagged stale-evidence (root-configs-9, -10, -17, -19,
  -63, -68, -117, -119, -121, -134, -145, -147; deleted mise.no-env.toml, Dockerfile and
  file-enforcer TS subjects); root-configs-9 was reclassified from planned repair to
  stale-evidence because the concurrent env move rewrote the tracked
  `.config/mise/conf.d/20-env.toml` comment with correct numbering, so the defect no
  longer holds. 2 in-place repairs (pkg-shell-argument-100 re-quoted after the
  depends-based task.ts rewrite; pkg-throws-25 after TSDoc line shifts). My verify runs:
  pkg-shell-argument 99 of 99, pkg-throws 49 of 49, and the root-configs BAD list
  contains exactly the 12 classified slice rows. Filer note: #333 and #427 are absent
  from issues-all.jsonl, so rows owning them cross-check live at filing. Slice 5
  (indexes 240 to 299, first id w2-async-b2-041) in flight.
- `w4-desktop-app` was not wedged: its turn had closed cleanly at `pkg-observable-123`
  (my verify: 122 of 122 clean; 122 rows is correct because relay duplicate 046 was
  dropped, so ids run to 123), all one-shot scopes dead. Re-briefed from `ring-session.ts`
  (39 files; the deferred context.ts startSession @throws check rides along until
  session.ts comes up).
- Returns verified since the twenty-fourth checkpoint: `w4-android-app` rows
  `w4-git-policy-b-005` to `-013`, 13 of 13 clean. Headline `-012` (warning, bug):
  `run_git_bytes` writes all stdin at `manual_push_range.rs:92` before
  `wait_with_output` at `:94` with stdout piped, and `peel_commits` feeds every unique
  oid to cat-file --batch-check, so roughly 1400 distinct prior values deadlock the push
  guard with no timeout; my controls read both functions live, and the bounded markdown
  child's scratch-file drain confirms the bug is specific to manual_push_range's own
  runner. Also 6 stale-citation nits in both family forms (the `-005` control:
  citation at manual_push_probe.rs:6, zero tracked src/policy-engine files, cutover
  delete commit 642b6d72cb). Then a clean markdown_linter cluster turn (7 files, no
  rows). `w4-cc-statusline` rows `w4-git-policy-015` to `-023`, 23 of 23 clean, nine
  stale-citation nits in both forms (controls: commit_preparation.rs:11 dual citation
  including the shadow-repository variant; commit_index.rs:13 bare-filename
  index-file-timestamps.ts with zero tracked matches and a positive control).
  `w4-mp-design` positions 140 to 144 adjudicated clean (no rows; 9 files remain).
  `w4-fe-core-tests` opened translation-repair/artifact: rows `w4-tr-artifact-001`
  (repository.url names the Aquaticat org while the live remote is Aquaticats; family
  instance of w4-tr-archive-003; the child's chat report misspelled it Aquicat but the
  row text is correct) and `-002` (keyVocabularyOf's totality over
  KNOWN_ARTIFACT_SCHEMA_VERSIONS is pinned only through generation 9), 19 of 178 files
  covered, prior-row do-not-re-file ledger built.

Wave-4 twenty-sixth checkpoint (2026-11-01, all rows verified against the tree before
recording):

- Package slice completion: `w4-mp-design` finished the ide/app src/native slice, 153 of
  153 tracked files, zero remaining by the coverage formula. Findings
  `w4-ide-app.jsonl` stands at 18 rows, 18 of 18 clean by my own run. Headline rows:
  the hardcoded `monochromatic_ide` log-target cluster (013/014/017 with rewrap.rs:230)
  against the crate name `ide_app`, the TileWorker Drop-detach vs documented
  join-on-shutdown pair (015/016), and the wrap_resize_tests.rs:920 comment arithmetic
  (018: the comment's own numbers give 24 MB where it claims about 70 MB; the real tile
  is 61.6 MB so the test expectations hold; comment-only). The
  tall_window_tests.rs:325 issue-1785 note stays ledgered for the filer
  (network-dependent). The child moves to the ide/app tests slice (about 83 files,
  re-measured live).
- Returns verified since the twenty-fifth checkpoint: `w4-android-app` rows
  `w4-git-policy-b-014` to `-017`, 17 of 17 clean. b-014: the quadratic staged lookup at
  merge_isolation_recovery.rs:227 (my control read the loop). b-015 (warning, bug):
  is_ignorable swallows `--no-summary`, which this host's `git merge -h` documents as a
  synonym to `--stat`, so native Git's diffstat suppression is lost (my live probe
  confirmed). b-016/b-017: owner-lock stale citations, zero-tracked matches proven.
  The markdown_linter and merge_isolation clusters adjudicated clean; open leads carried:
  parse_merge_intent's skipped emptiness checks (confirm at transaction_owner.rs) and
  apply()'s RecoveryPlace common_dir vs the identity's common dir (adjudicate at
  recovery_landing.rs and transaction_git.rs).
- `w4-cc-statusline` rows `w4-git-policy-024` to `-026`, 26 of 26 clean (three
  stale-citation nits in both family forms, controls run), then a clean 11-file config
  parse/validate/snapshot chain turn (2^53-1 ceilings, traversal protection, 60-case
  differential harness against real Git).
- `w4-desktop-app` rows `pkg-observable-124` to `-127`, 126 of 126 clean: the ring check
  counts the Play circle's own fill, the same-filesystem guard is vacuous by
  construction, the scenario-table ids are hand-mirrored, and the created
  `<label>-home` directory has no consumer (my control: session.ts:277 is the only
  creation site and every HOME-shaped variable in desktop-e2e is XDG-prefixed). The
  deferred context.ts startSession @throws lead closed clean. 31 files remain from
  states-conditions.ts.
- `w4-fe-core-tests` on artifact: 33 of 178 files covered, rows 001 and 002 stand (2 of
  2 clean), the coverage ledger was repaired to a clean 145-file list; next file
  consolidate-archive-flawed.ts.
- In flight: `w4-android-app` (git-policy tail, owner_lock.rs line 231; 246 remain),
  `w4-desktop-app` (desktop-e2e, states-conditions.ts; 31 remain), `w4-cc-statusline`
  (git-policy head, config_values.rs; 407 remain), `w4-fe-core-tests` (artifact; 145
  remain), `w4-fm-variant-qt` (adjudication slice 5, indexes 240 to 299).

Wave-4 twenty-seventh checkpoint (2026-11-01, all rows verified against the tree before
recording):

- Package completion: `w4-desktop-app` finished music-player/desktop-e2e; the uncovered
  list is empty. Findings `pkg-observable.jsonl` 135 rows, 135 of 135 clean by my own
  run (ids run to 136 because relay duplicate 046 was dropped). Closing rows 132 to
  136: four doc-drift nits on the wide-* module docs, and one warning (136):
  wide-seek's `movedForward` demands `position > base + BAR_STEP_S` (BAR_STEP_S = 1 at
  wide-seek.ts:107, comparison at :412 to :414), unreachable by the documented
  one-second smallest paused step, so the wait hangs or the constant's doc understates
  the app's step (likely; no seek-step constant found). My controls: the module doc at
  wide-search.ts:2 names double Shift while the file never imports checkDoubleShift
  (grep zero; its records function is checkSearchAndTooltip, whose own doc line :126
  repeats the drift), and the wide-double-shift.ts:7 import line is a TSDoc example,
  not a self-import. wide-browse.ts correctly skipped as row 114's subject. Open lead
  recorded: wide-breakpoint's "the runner's start-up check" is not locatable in
  desktop-e2e and likely points into package/nested-wayland-session/cli; resolve the
  reference target only, do not review that package. The child moves to the
  truepeak-core bench sidecar (queue item 2's tail).
- Returns verified since the twenty-sixth checkpoint: `w4-android-app` row
  `w4-git-policy-b-018` (policy_added_path.rs:6 removed-wrapper citation, zero tracked
  matches, positive control run; the worktree_completion.rs:13 twin ledgered for its
  own row) plus clean owner_lock, panic_notice, path_bytes, pid_namespace,
  policy_added_path, policy_checks and partial policy_content turns; 232 tail files
  remain, RecoveryPlace common_dir and transaction_owner leads still open.
  `w4-cc-statusline` rows `w4-git-policy-027` to `-044`: 027 the config_values.rs:212
  quadratic contains scan (mirrors 002; my control read the loop), 028 to 044
  stale-citation nits across the dependent_version cluster, all with tree-wide
  zero-tracked-match proofs (my control on 037: dependent_version_paths.rs lines 5 to
  7 cite three removed files; only unrelated .md docs share the source-imports name);
  379 head files remain, the pending citation rows ride with states, text,
  policy_dependent_version and policy_registry_tests. `w4-fe-core-tests` rows
  `w4-tr-artifact-003` (dispute-note append unpinned present-vs-absent; my grep shows
  the note enters the test only through the one label-distinctness call at :558) and
  `-004` (three-way producer docs vs four CandidateProducer kinds; the doc line at
  :191 confirmed); artifact at 48 of 178, rendered-sheets cross-check still open.
  `w4-mp-design` opened the ide/app tests slice: 83 tracked entries measured live (82
  Rust files plus tests/fixture/browser-find.json; the 52 src/*_tests.rs unit files
  outside native are a separate queue set), positions 1 to 5 clean; open lead:
  annotation_placement.rs:10 claims the README cites the probe's measurements, no such
  citation found, positive control pending. `w4-fm-variant-qt` finished adjudication
  slice 5: 60 entries, 11 duplicate (owners verified; four re-checked by me: 1651,
  1846, 1896, 2227), 49 distinct, zero stale-evidence tags, 3 in-place repairs
  (w2-fs-b 56 of 56 and w2-i18n 113 of 113 clean by my runs; slice log 60 rows).
  Running adjudication total: 298 of 409 (71 duplicate, 227 distinct). Slice 6
  (indexes 300 to 359) in flight.
- In flight: `w4-android-app` (git-policy tail, policy_content.rs line 481; 232
  remain), `w4-desktop-app` (truepeak-core bench sidecar), `w4-cc-statusline`
  (git-policy head, dependent_version_release.rs; 379 remain), `w4-fe-core-tests`
  (artifact, corpus-run cluster; 130 remain), `w4-fm-variant-qt` (adjudication slice 6),
  `w4-mp-design` (ide/app tests slice, position 6 change_watch.rs; 78 remain).

Wave-4 twenty-eighth checkpoint (2026-11-01, all rows verified against the tree before
recording):

- Campaign milestone: the ambiguous adjudication of manual-b3v2.jsonl is complete.
  Slice 7 closed the queue (49 log rows: 7 second-occurrence skips pointing at slice
  6's decisions, 9 duplicates with owners all verified present, four re-checked by me:
  1318, 1431, 1589, 2093; 33 distinct; zero stale-evidence, zero repairs needed). My
  verify runs over the campaign's slice-owned files, all clean: w2-logger-fuzz 83,
  w2-logger-src 71, w2-logger-tests 57, w2-th-core 78, w2-th-diag-expect 107,
  w2-throw-async-a 97, w2-typeface-figma 76, w2-th-matrix-sandbox 73. Notable slice 7
  adjudications: #1237 covers only the em-dash character; #1877 is a different
  mechanism in the same parse-svg (figma-07 keeps a cross-reference note); a-08's #2142
  flags the opposite deviation (filer note: the testing-practices skill prescribes the
  empty-string name, reconcile before filing); matrix-044 joins the #2570/#2573/#2568
  cluster. The child moves to the 259-BAD corpus repair (mq10-bad-enumeration.txt).
- Returns verified since the twenty-seventh checkpoint: `w4-cc-statusline` rows
  `w4-git-policy-045` to `-057`, 57 of 57 clean by my run. The dependent_version family
  is fully covered, every stale citation filed; 048 is duplicate_name's quadratic
  states[..index].find loop (my control read it); 051 to 055 are the
  anonymous-functions family per-site rows cross-filed with the tail child (my
  control: the claim is verbatim at candidate_stage.rs:140, candidate_tree.rs:60 and
  config_rules_file.rs:97); 056/057 direct-fix citations with zero tracked matches.
  `w4-android-app` rows `w4-git-policy-b-020` to `-024`, 24 of 24 clean by my run:
  b-021 the family origin (the closure-lint decision doc at :75-76 records the
  repo-wide removal on 2026-10-18; policy_events.rs:427 and panic_notice_tests.rs:132
  still claim the ban), b-022/b-023 the tail instances, b-024 the removed markdown-lint
  package (my control: zero git ls-files matches). `w4-desktop-app` rows
  `pkg-observable-137` to `-139`, 138 of 138 clean: collect.rs:109 nulls stderr and
  :150 drops the exit status (my control), invalid bin_seconds/workers silently become
  defaults, classify.rs:16 says below-a-cutoff where :75 routes at >= cutoff_db (my
  control); the wide-breakpoint lead closed clean (the reference is desktop-e2e's own
  compositor-version.ts called by run.ts:126); the truepeak-core.bench sidecar is at 8
  of 34 files with a 9-file prior-row skip ledger, 16 analysis .mjs remain.
  `w4-fe-core-tests` rows `w4-tr-artifact-005` (the generation-seventeen header is
  stale; contract.ts:146-147 control) and `-006` (verifyArtifactMeasurements compares
  only targetChars, never the recorded archiveText; corpus-verify.ts:123 vs
  build.ts:284/:300 controls); artifact at 63 of 178, the two-lane contract file
  complete. `w4-mp-design` tests-slice positions 6 to 27 clean, no rows; the
  annotation_placement lead closed (README:3381 does cite the probe; the prior miss was
  a head-capped grep).
- In flight: `w4-android-app` (tail, policy_markdown_tests.rs; 217 remain),
  `w4-desktop-app` (sidecar analysis scripts; 16 remain), `w4-cc-statusline` (head,
  direct_fix_tests.rs; 365 remain), `w4-fe-core-tests` (artifact read-consistency
  cluster; 115 remain), `w4-mp-design` (tests slice, font_features.rs; 56 remain),
  `w4-fm-variant-qt` (259-BAD corpus repair, opening slice).

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
   the source package), `search-fetch/mcp`
   (in flight; core has concurrent edits), `hall-monitor` (in flight), `electron-infra`
   (in flight). Done: dsh-plugin family, search-fetch/cli, pi-plugin family (all 15
   subpackages); mcp/stdio has only its final test slice in flight.
4. Linter tooling: `oxlint-plugin` package 2 (in flight), `forbidden-strings` done
   2026-11-01 (core, cli, core.bench, core.fuzz, cli.fuzz all complete; real root:
   `package/forbidden-strings`). Superseded 2026-11-01: the
   TypeScript `file-enforcer/core` slices 8+ are moot, the cutover deleted the package; the
   Rust replacements `file-enforcer/core-rust` (63 files) and `file-enforcer/cli` (30 files,
   now Rust) are under review by `w4-fe-core-tests` and `w4-fm-variant-qt`. Done:
   monochromatic-lint, monorepo-task, mutation-test.
5. Rust: `dependency-catalog/*` complete (core-pnpm, core-rust 22, core-native 54,
   cli-rust-update 36, cli-tighten 24, cli-tighten.matrix 12; cli-cube in flight), `wg/*`
   done through cli-quicker 138 of 138, `git-policy/cli` remainder campaign in flight by
   `w4-cc-statusline` (585 subject files from `binary_policy_tests.rs`, ledger
   `w4-git-policy-progress.md`), `mvm/cli` done (24 rows), `learning/rust`
   done (3 rows), `ownership-marker/foreign-borrowed` done clean (zero rows),
   `search-shadow/cli`, `vm-builder/cli`, `vmsync/cli` done by `w4-fe-core-tests`
   (4, 8, 13 rows).
6. `translation-repair/*` (35 after adjudication, one per assignment; `archive` done 92 of
   92 by `w4-fe-core-tests`, now on `artifact`, 178 files); `upstream-fork/*`
   (about 12 plus fuzz sidecars); `config/*` (9; pnpr has concurrent edits); misc:
   `intellij-plugin/islands-black`, `rolldown-plugin/import-attributes`,
   `pnpm-override/*` (7), `runtime-error/bun`, hyperscript-class remainders,
   `test-fixture/*` source code only. Done or in flight: `backup-path/cli`, `fy/cli`,
   `git-clone-size/cli`, `deps-update/cli`, `android-exempt-unused/cli`,
   `rgffplay/cli`, `page-weight/cli`, `terminal-exec/cli`, `watch-restart/cli` (43 of
   43, 12 rows), `kwin/key-helper` (25 of 25, 4 rows).
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
4. Done 2026-11-01: batch-3 filing completed as `rr-file-batch3v3` (1676 decisions, zero
   errors) and the reconciliation landed: `#2716` closed as duplicate of the manually filed
   `#2681`, the 15 filed moot actions handled (6 closes, 9 follow-up comments),
   `issues-all.jsonl` refreshed to the 3000-entry cap (batch-4 dedupe must paginate past
   it). Remaining from the plan: work the manual queue `manual-b3v2.jsonl`.
5. Done differently 2026-11-01: rows 80/81 were never filed in batch 2, so there were no
   owning issues to comment on. 87's correction is folded into 80's batch-3 issue body, and 81's
   deliberate-bound refutation is recorded in the recreated `post-batch2-corrections.md` for the
   manual-queue adjudication.
6. Done for the completed portion 2026-11-01: the regenerated `orchestrator-central-tests.jsonl`
   (34 rows, all verify-clean) carries the uncovered failing suites from run 3's completed
   portion plus the two hang findings. A fresh complete central run should wait until the two
   hangs are fixed, then refresh the triage.
7. Work the manual queue: `manual-b3v2.jsonl` (825 entries, ambiguous plus 416 with evidence
   problems) replaces the old 518-entry queue for batch 3. Superseded 2026-11-01: the
   remaining evidence repair is the 259-BAD corpus campaign (scratch
   `mq10-bad-enumeration.txt`), queued for `w4-fm-variant-qt` after the adjudication
   slices. Ambiguous adjudication: slices 1 to 4 done (238 of 409; 60 duplicate, 178
   distinct; twelfth to twenty-fifth checkpoints), slice 5 (indexes 240 to 299) in
   flight; `docs-core-56` needs re-verification of the deleted `reject_unknown_keys`
   mechanism. The old `manual-queue.jsonl` (batch 1/2 era) still holds
   its own entries.
8. Keep updating and committing this handover at every wave transition.

## Stop conditions

No code changes in the main worktree; experiments run only in throwaway worktrees. Tracker
writes happen only after orchestrator verification. Every new issue body states that no
preferred resolution has been picked. Subagent scopes stay within one package (scanning) or one
concern (cross-cutting) and within the ~60-minute budget. Machine scans alone do not close a
prose area: deep reads with per-file accounting do.
