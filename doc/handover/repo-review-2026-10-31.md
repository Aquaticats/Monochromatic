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

Excluded from batch-2 auto-filing: the figma-kiwi binary-reader family
(`pkg-figma-kiwi-001/004/016/021`, `w2-typeface-figma-22/24/25/29`, an existing issue covers
the family; manual review pending) and `w2-async-b1-006` (duplicate of `docs-ops-27`).

## Manual queue

518 entries (238 ambiguous Jaccard 0.2 to 0.45; 280 with verifier problems: quote mismatches,
missing path:line refs, out-of-bounds lines; repair evidence before filing). Not started.
Entry shape: `id`, `kind`, `title`, `scored_candidates` (`peek.ts` prints blanks for these;
inspect with `sed` or `head`). Plan: delegate first-pass adjudication to a freed child, the
orchestrator spot-checks, then files.

## Pipeline state (v10)

- 32 findings files, about 2010 rows as last reported by the children (unverified counts for
  the newest slices), plus `orchestrator-spot-audit.jsonl`. All 32 children are active: 8 in
  flight (ci-depot, pkg-figma-kiwi, w2-logger-tests, w2-th-diag-expect, w2-th-matrix-sandbox,
  w2-typeface-figma, pkg-async-time slice 4, w2-logger-fuzz files 9-13) and 24 re-briefed at
  v10 (see Children).
- Schema repair: the post-compaction compact-schema breakage (197 rows) is resolved except
  `root-configs` (13 rows, its re-brief carries the maintenance instruction). Every other
  child rewrote its compact rows in place; `schema-audit.ts` is the counter.
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

- `root-configs` -> file-enforcer/core slice 4 (position 126, `src/pipeline/exec.ts`) plus
  rewriting its 13 compact rows first. Prior: 111.
- `ci-depot` -> ssg/aquati.cat `src/client` (resume `src/client/highlight-groups.ts`).
  Prior: 74.
- `docs-core` -> mpm-shadow `src/toolchain/` remainder: `plan.rs` onward plus test doubles.
  Prior: 61 (decode_postinstall keeps `link` over `run` silently; cargo-git without
  `packageId` fails at install time instead of decode time).
- `docs-decisions` -> git-policy/cli slice 5: `binary_fault_filesystem_tests.rs` and the rest
  of the `binary_fault_*` tree. Prior: 58 (hatch test doc claims forms never tested).
- `docs-ops` -> claude-code-plugin source package (contains `src/inject.ts`, `src/paths.ts`;
  verify the two recorded notes). Prior: 56 (plugin.json describes a different reminder than
  the shipped one; README omits two trigger groups).
- `docs-planning-audit` -> music-player/design slice 4: `open-questions.md`, then
  `candidates.md`. Prior: 155 (phantom bullet continuations split words; D21 reads standing
  though superseded).
- `docs-skills` -> jsonc-edit family remainder: core-rust `value.rs`, `span.rs`,
  `edit_spine.rs`, `edit_apply.rs`, `edit_state.rs`, `edit_comment.rs`, `navigate.rs`,
  `number.rs`, `number_scale.rs`, `path.rs`, `text_units.rs`, `error.rs`, `mutants.toml`, 8
  test files, then the fuzz sidecar remainder. Prior: 74 (both depth seeds sit on the rejected
  side of the 512 bound).
- `docs-philosophy` -> file-enforcer/core-rust (whole package; the concurrent
  `doc/handover/file-enforcer-rust-rewrite.md` is not evidence). Prior: 63 (zero-channel
  `feed` panic unguarded; `evaluate.rs` mis-slicing uncovered).
- `pkg-dom` -> agent-harness-shared/current-time-context. Prior: 48 (sole truncateContext test
  pins marker presence, not the maxChars contract).
- `pkg-async-iter` -> doc/planning music-player tail: 11 files from
  `music-player-light-error-undo.md`. Prior: 45 (lock-format section present-tense for a
  deleted file; two mise.lock readers never repointed).
- `pkg-async-time` -> monochromatic-lint slice 4: position 191,
  `src/rust_explicit_declarations_tests.rs` onward. Prior: 21 (fence-tag and fix-refusal
  retry both uncovered; rows 020/021 rewritten in place).
- `pkg-array-at` -> git-executable/core. Prior: 37 (mcp/stdio complete).
- `pkg-figma-kiwi` -> pi-plugin/ask-user-question final 14 files (resume
  `helper-launch.unit.test.ts`). Prior: 44.
- `pkg-figma-to-penpot` -> ide/app slice 4: `inspect-ignored-guards.ts` case lists, then the
  other guard case lists, then `design/*.ts`. Prior: 52 (hard-coded `typescript@7.0.2` store
  paths ENOENT against the pinned 7.1.0-beta).
- `pkg-observable` -> music-player/desktop-app slice 4: `src/analysis_notification.rs`
  onward (~25 src files; ~279 remain). Prior: 41 (notices script breaks on percent-encoded
  paths).
- `pkg-shell-argument` -> dsh-plugin/neuralwatt. Prior: 88 (cost-dismissal's whole
  implementation was untracked, swallowed by `*.js` gitignore).
- `pkg-throws` -> browserslist/core-rust remainder: the ~25 test/src files through `bin/*.ts`;
  completes the package. Prior: 39 (`parse_float` returns NaN where JS trims whitespace).
- `sysd-source-trace` -> nested-wayland-session/cli final remainder: `mise.toml` lines
  1155-2702, README body, Cargo.lock as data. Prior: 19 (`record start` truncates prior
  frames; Containerfile pins floating nextest).
- `w2-async-b1` -> file-manager/variant-gtk-sticky. Prior: 52 (variant-electron complete:
  Left/Right clamp divergence, dedup-ignoring closePane, scroll-listener contradiction).
- `w2-async-b2` -> mutation-test/cli-unused-export (13 files; completes the family).
  Prior: 58 (fixture's all-killed reference case ships a survivor).
- `w2-dom-a` -> doc/planning: the three translation-repair-archive files. Prior: 74 (doc
  attributes exemptions to a dropped path list; bare issue refs resolve to unrelated issues).
- `w2-fs-a` -> doc/audit resharp-fuzz-2026-06-04 remainder (26 files; README done). Prior: 56
  (Lean pair arithmetic short by 17; findings decomposition cannot sum).
- `w2-fs-b` -> doc/planning file 29 onward: `closure-lint-everywhere.md`. Prior: 48 (Done
  record cites a module deleted the same morning).
- `w2-i18n` -> desktop-e2e: `wide-seek.ts`, `wide-select.ts` 1-329, `wide-session.ts`,
  `wide-settings.ts`, `wide-track-list.ts`. Prior: 86 (settledState leaks Ctrl across awaited
  presses; restores only on success paths).
- `w2-logger-fuzz` -> doc/decision files 9-13 (`ci-provider.md`,
  `cli-arg-validation.md`, `cli-git-concurrent-commits.md`,
  `cli-git-policies-platform.md`, `cli-git-rust-rewrite.md`). Prior: 68 (18 rows rewritten;
  decision doc teaches a relocated build dir).
- `w2-logger-src` -> doc/troubleshooting worklist line 52 onward
  (`btrfs-scrub-start-limit-reset.md`). Prior: 58 (biome doc asserts no release after
  2024-12-18; 0.7.0 shipped 2026-09-19).
- `w2-logger-tests` -> oxlint-plugin rule modules (`no-class` onward, then
  prefer-readonly-parameter-type package 2). Prior: 51.
- `w2-th-core` -> crate-release/cli final two files
  (`service-arguments.unit.test.ts`, `validate.unit.test.ts`). Prior: 61 (curl discard
  unguarded; ENOENT crash masked by a forgiving double).
- `w2-th-diag-expect` -> forbidden-strings/cli 25 unit sidecars. Prior: 87.
- `w2-th-matrix-sandbox` -> doc/planning pi-auto-mode: `pi-auto-mode-jev-qualification.md`
  line 561 onward, then `jev-service-gates`. Prior: 70.
- `w2-throw-async-a` -> xml-edit/core-rust plus its fuzz sidecar. Prior: 79 (monorepo-task
  family complete: spawn-failure steps hang the CLI forever; no cycle detection in
  `depends`).
- `w2-typeface-figma` -> doc/audit q-z position 20 onward (position 13 still deferred,
  concurrent edits). Prior: 76.

Brief template (reviewer-only, main-worktree protection, cgroup wrapper with the suffixed-name
footgun note, 250-word final message) is stable; every brief now embeds the full findings
schema verbatim plus, where relevant, a maintenance instruction (only `root-configs` still
has compact rows) and the bounded lint-digest grep for package-assigned children.

## Assignment queue (re-task idle children in this order)

1. Finish in-flight slices as children return (see Children).
2. Apps: `ide/app` remainder, `file-manager/*` remainder (variant-qt),
   music-player `design` remainder, `desktop-app` slices 5+, `android-app` (354, split 2),
   `asset`, `desktop-e2e` remainder, `truepeak-core` bench if anything remains.
3. Plugins: remaining `agent-harness-shared/*` (7 after current-time-context),
   `claude-code-plugin/*` (10 after the source package), `pi-plugin/*` (12 after
   ask-user-question), `dsh-plugin/search-fetch`, `search-fetch/*` (core has concurrent
   edits), remaining `electron-infra/*`, remaining `hall-monitor`, remaining `mcp/stdio`
   slices if the budget cut any.
4. Linter tooling: `monochromatic-lint` slices 5+, `oxlint-plugin` package 2,
   `forbidden-strings` remaining (core, core.bench, core.fuzz, cli.fuzz), `file-enforcer/core`
   slices 5+ then `cli`, `monorepo-task` remainder if any, `mutation-test` remainder if any.
5. Rust: `git-policy/cli` remaining slices, `file-enforcer/core-rust` (in flight),
   `dependency-catalog/*`, `wg/core-allowedips` remainder, `linter/rust`, `learning/rust`,
   `ownership-marker/foreign-borrowed`, `search-shadow/cli`, `mvm/cli`, `vm-builder/cli`,
   `vmsync/cli`.
6. `translation-repair/*` (~35, one per assignment); `upstream-fork/*` (~12 plus fuzz
   sidecars); `config/*` (9; pnpr has concurrent edits); misc: `backup-path/cli`,
   `android-exempt-unused/cli`, `deps-update/cli`, `fy/cli`, `git-clone-size/cli`,
   `rgffplay/cli`, `page-weight/cli`, `terminal-exec/cli`, `watch-restart/cli`,
   `kwin/key-helper`, `intellij-plugin/islands-black`, `rolldown-plugin/import-attributes`,
   `pnpm-override/*` (7), `runtime-error/bun`, hyperscript-class remainders,
   `test-fixture/*` source code only.
7. Doc remainders: doc/audit subdirs beyond the finished campaigns; doc/planning tails;
   doc/troubleshooting (~665 remaining); doc/agent and doc/decision spot coverage
   (`doc/decision` ~195 remaining after files 9-13).

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

1. Watch `rr-file-batch2c` (background filing of the remaining 263 decisions; log
   `file-batch2c.log`; 5-minute backoffs, circuit-breaker after two consecutive failed
   decisions). If it aborts, restart it later the same way; the skip-set makes restarts
   idempotent. On completion: read the tail counts, refresh `issues-all.jsonl`
   (expect ~2500+), spot-check a sample of new issue numbers, commit this handover (v10 is
   the incident-plus-wave commit; the next is v11).
2. Process child returns continuously; re-brief from the queue; keep the Children section
   current.
3. After batch 2 truly completes: re-run the pipeline for findings grown during it (verify,
   dedupe, draft, classify, intra-dupes) over every findings file including the schema-repaired
   rows, then file batch 3 the same way.
4. Work the 518-entry manual queue (evidence repair first, then ambiguous adjudication).
5. Check `rr-central-tests` completion; capture failures.
6. Keep updating and committing this handover at every wave transition.

## Stop conditions

No code changes in the main worktree; experiments run only in throwaway worktrees. Tracker
writes happen only after orchestrator verification. Every new issue body states that no
preferred resolution has been picked. Subagent scopes stay within one package (scanning) or one
concern (cross-cutting) and within the ~60-minute budget. Machine scans alone do not close a
prose area: deep reads with per-file accounting do.
