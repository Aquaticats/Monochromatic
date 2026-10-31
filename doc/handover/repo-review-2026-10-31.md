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
  `send_message` is cheap (7 to 10 per program fine).
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
- Orchestrator spot audit (5 planning files fully read, 2026-10-31): the machine scan's yield is
  roughly an order of magnitude low; deep-read yield about 0.8 solid findings per file versus
  the scan's 0.07.

## Central runs

- `mise run lint` completed (exit 1); log `lint-full.log`. Digest done: `lint-digest.txt`
  (1523 diagnostics across 54 packages). Top clusters: `corpus-run` family 949 (408 TS2305),
  `package/config/rolldown` 13 TS2305, `test-import` 175+ sites (already covered by issue
  #1651), `prefer-readonly-parameter-type` clusters. Distribution to owning children still
  pending; mass TS2305/TS7006 clusters are design signals, not auto-filed findings.
- `mise run test:unit` running capped as `rr-central-tests`; log `test-unit-full.log` (all
  `[PASS]` at last check). Completion check still pending.
- Dedupe index: 1846 tracker issues in `issues-all.jsonl` (refreshed after batch 1). Refresh
  again after batch 2 completes.
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
  `filed-log.jsonl`, idempotent by filed id; errors do NOT mark filed. Pass `no-plan-new` when
  the decisions file already covers every plan-new id (it does for batch 2).
- Batch 1 (filed): 58 new issues `#1824`-`#1881`, 9 comments (`#1701` twice, `#1751`, `#1687`,
  `#1684`, `#1552`, `#1643`, `#1412` twice), plus preflight `#1823`. All 68 in `filed-log.jsonl`.
  Spot-check passed: all auto-comments same-root-cause, auto-news distinct.
- Batch 2 (running, background `rr-file-batch2`, log `file-batch2.log`): progress 215 of 763
  filings in `filed-log.jsonl` at last check (through issue #2028), 0 errors, ~2.7 filings per
  minute; several hours remain. `decisions-batch2-final.jsonl` = 763 decisions (703 auto-new + 69
  auto-comment from the classifier, minus 8 excluded + 1 deduped). Excluded from auto:
  the figma-kiwi binary-reader family (`pkg-figma-kiwi-001/004/016/021`,
  `w2-typeface-figma-22/24/25/29`) because an existing issue covers the family (manual review
  pending), and `w2-async-b1-006` (duplicate of `docs-ops-27`, same package and finding).
  Filing runs as background process `rr-file-batch2`, log `file-batch2.log`.
- Manual queue: 518 entries (238 ambiguous Jaccard 0.2 to 0.45; 280 with verifier problems:
  quote mismatches, missing path:line refs, out-of-bounds lines; repair evidence before
  filing). Not started. Entry shape confirmed by raw read: `id`, `kind` (`ambiguous`), `title`,
  `scored_candidates` with number/state/jaccard; `peek.ts` prints blank lines for these entries
  (they lack the `claim` field `peek.ts` prints), so inspect with `sed` or `head` on the raw
  file. Plan: review in batches; consider delegating first-pass adjudication to a freed child,
  orchestrator spot-checks then files.
- Pipeline state: 51 real findings files, about 1290 findings, all verified through wave 12
  except the newest slices; plans and decisions current as of batch 2 launch.

## Children

All 32 admitted children re-tasked as a worker pool. Current assignments (name -> area; counts
are findings so far in that child's file):

- `root-configs` -> file-enforcer/core slice 1 finish (positions 36-80); slice 2 starts at
  position 81, `src/io/staleness-types.ts`. Prior: 57.
- `ci-depot` -> ssg/aquati.cat. Prior: 48 (wc/app done: README Lines claim contradicts code).
- `docs-core` -> mpm-shadow/cli. Prior: 42 (resharp 06-04 + 06-11 docs fully read).
- `docs-decisions` -> git-policy/cli slice 1 (metadata plus src in sorted path order, ~60%).
  Prior: 52 (rss/app and codex-plugin/bash-limit done; bash-limit: `verify` reads only the
  parent cgroup's controls, child limits unchecked).
- `docs-ops` -> claude-code-plugin family, first package. Prior: 35 (error-handling concern
  done: 509 throw sites without @throws; mixed-signaling and no-class clusters).
- `docs-planning-audit` -> done/app: the 12 `*-styles.ts` files line-by-line (four long docs
  done: FTS5 virtual-table claims vs Turso `USING fts` reality; settings status contradiction;
  429 claim vs HTTP 200). Prior: 133.
- `docs-skills` -> electron-infra/core (app-counter done: logger-core root import instead of
  `/ts`; duplicated state-path literal; title literal at three sites). Prior: 45.
- `docs-philosophy` -> claude-cpu-cap family, first package. Prior: 24 (TSDoc concern done:
  629 delegation wrappers, 966 @param restatements).
- `pkg-dom` -> agent-harness-shared family, first package. Prior: 34 (logging concern done;
  translation-repair data-id tags finding).
- `pkg-async-iter` -> doc/planning module-logger-log-accumulation onward (5 files). Prior: 25.
- `pkg-async-time` -> monochromatic-lint slice 2 (positions 81-140). Prior: 18 (slice 1 done;
  disposable-Fixture scaffolding duplication warning).
- `pkg-figma-kiwi` -> pi-plugin family, first package. Prior: 37 (overlap classification done;
  #1334/#1253 verdict: same category, distinct clusters).
- `pkg-figma-to-penpot` -> CXL spawn-test remainders (advisor, openai-fast, remaining spawn
  files). Prior: 42 (mvm-mcp stderr discard; testing-practices buildless omission).
- `pkg-observable` -> tech-monorepo-manager-vet discovery-github.md lines 215-2113 (398 entry
  bodies, the 343-entry excluded-but-close section, taxonomy terms). Prior: 28 (chunk files
  done; `${AUDIT}` unexpanded-literal blocker in build-mirror.mjs).
- `pkg-shell-argument` -> done/variant-postcss remainders (component, docs, styles). Prior: 53
  (timer overwrite, prompt injection, blockedBy self-cycle).
- `pkg-throws` -> cloudflare-worker/rand. Prior: 23 (open-code-review-issue/cli complete).
- `sysd-source-trace` -> terminal/app. Prior: 0 for this file (systemd work done and committed).
- `w2-async-b1` -> hall-monitor family, first package. Prior: 33 (async concern done: css
  parser and toml-edit walkers unbounded recursion; scanner v3 covered all blind spots).
- `w2-async-b2` -> mutation-test/cli-cargo-mutants (cli unit tests done: exported `selectTests`
  holds the 030/031 defects with zero unit tests; ground truth was 7 files, not the briefed 9).
  Prior: 40.
- `w2-dom-a` -> doc/planning t-z continuation from the-third-rendering-runs-6-and-7-read.md.
  Prior: 44.
- `w2-fs-a` -> doc/planning em-dash.md (6965 lines, alone). Prior: 27.
- `w2-fs-b` -> doc/planning a-d: cli-git-rust-behavior-ledger.md lines 3101-4653, then files
  23-76. Prior: 43 (starvation default contradiction, 2 vs 1; root changeset task runs
  deleted bump:dependents).
- `w2-i18n` -> doodle-widget/app. Prior: 41 (kotlin package done: publish coordinates 404).
- `w2-logger-fuzz` -> doc/troubleshooting 2026-05-14-repo-assessment.md plus README.md, then
  its slice is complete. Prior: 27.
- `w2-logger-src` -> doc/troubleshooting next 8 from android-trash-request-outcome-boundary.md.
  Prior: 40 (676 remain in worklist).
- `w2-logger-tests` -> oxlint-plugin rule modules (arrow-function-params through
  no-low-information-symbol-description and siblings); prefer-readonly-parameter-type package 2
  after. Prior: 46 (ClassBody walk skips member decorators, so the autofix rewrites behavior;
  spread-bridge `.then(success, failure)` helper; no-useless-spread double-space fix pinned).
- `w2-th-core` -> syllable-break-demo/app. Prior: 43 (doc/audit i-p done).
- `w2-th-diag-expect` -> forbidden-strings/cli finish, then cli.fuzz, core, core.bench,
  core.fuzz. Prior: 57.
- `w2-th-matrix-sandbox` -> doc/planning pi-auto-mode: governing-domain-closure plus next 3
  files. Prior: 56 (effect contract cites ten extension sources deleted by a3955898f1; three-call
  ceiling contradicts the axioms' 2026-10-15 replacement; deadline raised to 15s, doc says 5s).
- `w2-throw-async-a` -> monorepo-task/core-exec. Prior: 47 (cli done: dead typed helpers
  mirrored by untyped prelude twins; fanout unbounded).
- `w2-typeface-figma` -> doc/audit q-z positions 14+ (deferred
  standards-review-gate-shadow.md, concurrent edits). Prior: 65.

Brief template (reviewer-only, main-worktree protection, cgroup wrapper with the suffixed-name
footgun note, JSONL schema, 250-word final message) is stable; reuse verbatim from the
transcript.

## Assignment queue (re-task idle children in this order)

1. Finish in-flight slices as children return (see Children).
2. Apps: `ide/app` (1134, split ~3), `file-manager/*` (4), music-player `design` (1269, split
   ~3), `desktop-app` (361, split 2), `android-app` (354, split 2), `asset`, `truepeak-core`
   plus bench, `desktop-e2e`.
3. Plugins: remaining `agent-harness-shared/*`, `claude-code-plugin/*`, `pi-plugin/*`,
   `dsh-plugin/*`, `search-fetch/*` (core has concurrent edits),
   remaining `electron-infra/*`, `claude-cpu-cap` remainder, remaining `hall-monitor`,
   remaining `mcp/stdio` slices if the budget cuts it. Done this wave: `codex-plugin/bash-limit`.
4. Linter tooling: `monochromatic-lint` slices 3-4, `oxlint-plugin` package 2,
   `forbidden-strings` (4 remaining packages), `monorepo-task` remainder, `file-enforcer/core`
   slices 2-3 plus `cli`, `mutation-test` (3 remaining packages).
5. Rust: `git-policy/cli` (1030, split 3-4; slice 1 in flight with `docs-decisions`),
   plus fuzz, `file-enforcer/core-rust`,
   `browserslist/core-rust`, `jsonc-edit/core-rust` plus fuzz, `xml-edit/core-rust` plus fuzz,
   `dependency-catalog/*`, `wg/core-allowedips`, `linter/rust`, `learning/rust`,
   `ownership-marker/foreign-borrowed`, `search-shadow/cli`, `crate-release/cli`, `mvm/cli`,
   `nested-wayland-session/cli`, `vm-builder/cli`, `vmsync/cli`.
6. `translation-repair/*` (~35, one per assignment); `upstream-fork/*` (~12 plus fuzz sidecars);
   `config/*` (9; pnpr has concurrent edits); misc: `backup-path/cli`,
   `android-exempt-unused/cli`, `deps-update/cli`, `fy/cli`, `git-clone-size/cli`,
   `git-executable/core`, `rgffplay/cli`, `page-weight/cli`, `terminal-exec/cli`,
   `watch-restart/cli`, `kwin/key-helper`, `intellij-plugin/islands-black`,
   `rolldown-plugin/import-attributes`, `pnpm-override/*` (7), `runtime-error/bun`,
   hyperscript-class remainders, `test-fixture/*` source code only.
7. Doc remainders: doc/audit subdirs beyond the finished campaigns; doc/planning t-z tail
   (~212 files) and n-s and a-d tails; doc/troubleshooting (~676); doc/agent and doc/decision
   spot coverage.

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

1. Watch `rr-file-batch2` (background filing of 763 decisions; log `file-batch2.log`; 215
   filed, 0 errors at last check). On completion: read the tail counts, refresh
   `issues-all.jsonl` (expect ~2500), spot-check a sample of new issue numbers, commit this
   handover (v8 committed at the 215-filing mark; the next commit is v9).
2. Re-run the pipeline for findings grown during batch 2 (verify, dedupe, draft, classify,
   intra-dupes), then file batch 3 the same way.
3. Work the 518-entry manual queue (evidence repair first, then ambiguous adjudication).
4. Triage `lint-full.log` into a per-package digest; distribute to owning children.
5. Check `rr-central-tests` completion; capture failures.
6. Keep re-tasking returning children from the queue; update and commit this handover at every
   wave transition.

## Stop conditions

No code changes in the main worktree; experiments run only in throwaway worktrees. Tracker
writes happen only after orchestrator verification. Every new issue body states that no
preferred resolution has been picked. Subagent scopes stay within one package (scanning) or one
concern (cross-cutting) and within the ~60-minute budget. Machine scans alone do not close a
prose area: deep reads with per-file accounting do.
