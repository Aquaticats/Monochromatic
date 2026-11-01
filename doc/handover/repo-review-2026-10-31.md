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

## 2026-10-31 outage window

All 32 subagents died mid-slice with `402 Vibe budget exhausted`; nothing user-visible was
lost because every completed turn had already appended and verified its findings. The
background filing run survived the window and completed batch 2. The central `test:unit`
run was killed by SIGHUP at 07:15 (session teardown collateral, matching the orphaned
process event); its 228 `[FAIL]` lines are preserved in `fail-suites.txt` and a fresh run
(`rr-central-tests2`, log `test-unit-full2.log`) is underway. A leftover child scope
(`rr-sysd-source-trace-083`, a `mvm/cli` buildAndTest) was stopped after the outage.
Children were re-briefed with resume points; each reports what it added before the outage.

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

## Children

All 32 admitted children re-tasked as a worker pool. Current assignments (name -> area;
counts are findings so far in that child's file, as last reported):

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
   (core in flight, core.bench, core.fuzz, cli.fuzz), `file-enforcer/core` slices 8+ then
   `cli`. Done: monochromatic-lint, monorepo-task, mutation-test.
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

1. Batch 2 is complete (764 filed, zero errors). The `issues-all.jsonl` refresh ran in the
   background (`rr-issues-refresh`, expect about 2575 issues); spot-check a sample of new
   issues (#2400, #2500, #2575) before batch-3 dedupe trusts it.
2. Process child returns continuously; re-brief from the queue; keep the Children section
   current. All 32 children were killed mid-slice by the 2026-10-31 budget outage (402s)
   and were re-briefed with their resume points at v11.
3. Re-run the pipeline for all findings grown since batch 2 (verify, dedupe, draft,
   classify, intra-dupes) over every findings file, then file batch 3 the same way; the
   filing script now paces and backs off on its own.
4. Post the batch-2 correction comments on already-filed issues (scratchpad
   `post-batch2-corrections.md` is the log; first entries: `w2-throw-async-a` filed 80
   scope-narrowing, filed 81 deliberate-bound note).
5. Triage the central-test failure inventory (`fail-suites.txt`, 228 `[FAIL]` lines from the
   run killed by SIGHUP at 07:15): cross-check each suite against findings and tracker, file
   the uncovered ones. Known already: the wg/core-allowedips red suite (#599, #600). The
   re-run (`rr-central-tests2`, log `test-unit-full2.log`) is running.
6. Work the 518-entry manual queue (evidence repair first, then ambiguous adjudication).
7. Keep updating and committing this handover at every wave transition.

## Stop conditions

No code changes in the main worktree; experiments run only in throwaway worktrees. Tracker
writes happen only after orchestrator verification. Every new issue body states that no
preferred resolution has been picked. Subagent scopes stay within one package (scanning) or one
concern (cross-cutting) and within the ~60-minute budget. Machine scans alone do not close a
prose area: deep reads with per-file accounting do.
