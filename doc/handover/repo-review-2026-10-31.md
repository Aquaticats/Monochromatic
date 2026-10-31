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
- Batch 2 (running, background `rr-file-batch2`, log `file-batch2.log`): progress 483 of 763
  filings in `filed-log.jsonl` at last check (through issue #2296), 0 errors, ~2.7 filings per
  minute; the tail is in sight. `decisions-batch2-final.jsonl` = 763 decisions (703 auto-new + 69
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

- `root-configs` -> file-enforcer/core: the 20 test files inside positions 81-125; slice 3
  resumes at position 126, `src/pipeline/exec.ts`. Prior: 88 (eager overwriteEach
  double-records destinations; post-write stat window certifies edits fresh forever).
- `ci-depot` -> ssg/aquati.cat `src/client` (resume `src/client/highlight-groups.ts`). Prior:
  74 (`src/build/` done: missing dropTables STAT logs two console errors per load; question-radio
  color-mix doc drift; `ul > post-card > li` invalid content model).
- `docs-core` -> mpm-shadow/cli `src/toolchain/` at `load.rs` (7 files). Prior: 55
  (install_android_sdk invokes an `android` binary Google's zip never ships, blocker).
- `docs-decisions` -> git-policy/cli slice 4, `src/native/binary_budget_trace.rs` onward. Prior:
  56 (`report_outcome` uncovered-commit note untested; double `--quiet` in traced; slices 1-3
  clean: fail-closed paths verified, finding 53 merge-message gap stands, ancestry-only test).
- `docs-ops` -> claude-code-plugin family, second package claude-spawn. Prior: 42
  (bash-output-filter done: README teaches retired PIPESTATUS wrapper; no tests exercise the
  shipped bundle).
- `docs-planning-audit` -> music-player/design slice 2: material-3-compliance.md then
  decisions.md. Prior: 146 (done/app complete; raw HTML hardware table; underivable 20.41dp
  inset, correct 20.10dp; toggle-switch state by thumb position alone).
- `docs-skills` -> wg/core-allowedips. Prior: 59 (electron-infra family done:
  defaultNestedWaylandBinary path points at a nonexistent package/cli layout, blocker;
  assertPathAccessible EACCES crash verified empirically).
- `docs-philosophy` -> truepeak-core remainder (`meter.rs` onward, then core tests). Prior: 54
  (claude-cpu-cap done; evaluate.rs ignores Track.bin_seconds, silently mis-slicing fine corpora
  10x; bench README presents retired Stage-two policy as shipped).
- `pkg-dom` -> advisor final slice: operation-test-fixture.ts, transcript-test-fixture.ts, 22
  unit tests, package-wide coverage pass. Prior: 42 (throw before its own declaration; README
  lists deleted output-eligibility file; truncateContext marker overflow reachable).
- `pkg-async-iter` -> doc/planning monorepo-manager-* six files. Prior: 25 (module-logger and
  module-test five-file slice all clean).
- `pkg-async-time` -> monochromatic-lint slice 3 (positions 141-190, `src/processors_model.rs`
  onward). Prior: 20 (slice 2 complete; rsx/rusty fence boundary has no regression guard, so the
  019 fix can land with zero test changes).
- `pkg-figma-kiwi` -> pi-plugin/ask-user-question final 14 files (resume
  helper-launch.unit.test.ts). Prior: 44 (agent-settled-notification done; 37 of 51 files read:
  e2e checklist waits on Helix in a Nano procedure; PRIVATE_FILE_MODE declared twice).
- `pkg-figma-to-penpot` -> ide/app slice 2: remaining `bin/*.ts` scripts, then `design/*.ts`.
  Prior: 45 (slice 1 done: mise.toml documents the retired 75 percent bound, raised to 90).
- `pkg-observable` -> music-player/desktop-app slice 2 (build.rs, mise.toml, task.ts, handover
  docs, then `src/` from accent_override.rs). Prior: 31 (Containerfiles pin nightly-2026-10-29
  against toolchain.lock.json's 10-31; README cites a nonexistent repository mise.lock).
- `pkg-shell-argument` -> dsh-plugin family, second package cost-dismissal (then neuralwatt).
  Prior: 76 (advisor done: `/advisor off` leaves the tool-restriction mask while README claims
  it clears; index exports have zero test hits).
- `pkg-throws` -> android-exempt-unused/cli slice 2 (`src/devices.ts` onward, 7 files). Prior:
  33 (cloudflare-worker/rand done; runAdb and applyChanges exported with zero tests).
- `sysd-source-trace` -> nested-wayland-session/cli. Prior: 8 (terminal/app done: git-policy
  hook rejects vendored Ghostty, breaking lint and test; PageList resize panic).
- `w2-async-b1` -> file-manager second package variant-electron. Prior: 45 (app done: PIDL
  leak on every win_drag error path; README documents nonexistent open-with-default; sync
  directory read on the GTK main thread).
- `w2-async-b2` -> mutation-test/cli-cargo-mutants final 5 unit tests. Prior: 50 (12-file
  slice done: bare JSON.parse SyntaxError names neither task nor file; unvalidated
  option.slice(2) evidence names).
- `w2-dom-a` -> doc/planning t-z at trailing-whitespace-enforcement-layer.md (5 files). Prior:
  64 (translate-unvalidated verdict string has zero source matches; awaiting-decision framing
  though variant A landed).
- `w2-fs-a` -> doc/audit d-h final two files: git-usage.2026-06-05.md, then
  glm-4-7-flash-across-every-stage.md. Prior: 46 (issue #107 collision; ssg i18n migration
  landed but framed pending; findMiseMonorepoRootCached evidence gone).
- `w2-fs-b` -> doc/planning a-d file 25, cli-git-rust-rewrite.md (then file 26 if it fits).
  Prior: 45 (behavior ledger fully verified; stale Next-actions list; private-ref gc claim
  superseded by shadow store).
- `w2-i18n` -> desktop-e2e continuation (ui-state, ui-state-log, log-scan, library, press,
  image, then wide-* scenarios). Prior: 77 (lifecycle core done: start failure leaks the active
  unit; pw-dump proof passes vacuously on malformed JSON; sendCommand has no deadline).
- `w2-logger-fuzz` -> doc/decision family. Prior: 42 (doc/agent done, every prescribed gh
  command verified live; triage-labels raw HTML table; troubleshooting family index links 69 of
  740 docs).
- `w2-logger-src` -> doc/troubleshooting unread.txt lines 47-51 (5 biome/browserslist files).
  Prior: 53 (aws-cloudfront-mirror and bazzite done with live probes: RFC 2181 10.5.1 does not
  exist; snapshot-964 contradiction).
- `w2-logger-tests` -> oxlint-plugin rule modules (no-class onward). Prior: 51 (angle scan
  throws on string literals, losing all rule output for the file; member-receiver iterator
  calls never inspected; receiver-ignore branches untested).
- `w2-th-core` -> crate-release/cli. Prior: 54 (asset areas done: dead tracked PNGs referenced
  nowhere; font provenance checksums unguarded; syllable-break-demo done).
- `w2-th-diag-expect` -> forbidden-strings/cli 25 unit sidecars. Prior: 87 (8 bins + PERF.md +
  7 tests/*.rs done: PERF.md cites the retired pre-0.2.0 tree; BUG 5 doc block stranded).
- `w2-th-matrix-sandbox` -> doc/planning pi-auto-mode: pi-auto-mode-jev-qualification.md line
  561 onward, then jev-service-gates. Prior: 70 (0.99 called the adopted cutoff while its own
  link target records 0.60; code cites deleted by the ground-up rebuild).
- `w2-throw-async-a` -> monorepo-task family, next package after core-exec. Prior: 74
  (core-exec tests done: signal-termination test has zero assertions and orphans the sleep
  child; 84 teardown sites outside try/finally).
- `w2-typeface-figma` -> doc/audit q-z position 20 onward (i-third-family-verifier vet;
  position 13 still deferred, concurrent edits). Prior: 76 (roster-expansion vet closes
  Candidate E citing unrelated issue #50, which is a module/es numeric-constants refactor).

Brief template (reviewer-only, main-worktree protection, cgroup wrapper with the suffixed-name
footgun note, 250-word final message) is stable; reuse verbatim from the transcript. The brief
now embeds the full findings schema verbatim (package, title, category, files array, what,
evidence as repo-relative path:line: quote with a space after the second colon, standard,
confidence, ran, dedupe_keywords) plus a maintenance instruction to rewrite compact-schema
rows: post-compaction briefs briefly specified a compact schema (id, severity, kind, file,
line, claim, evidence, checkable) that the pipeline verifier rejects; `schema-audit.ts` counts
197 compact rows across 31 findings files (it prints per-file counts; also found child-created
side files `new-round6.jsonl`, `w2-async-b1.part6.jsonl`, `w2-logger-tests-new.jsonl`,
`w2-planning-staging.jsonl`, all compact, to merge or fold into their owning files before batch
3). Repair tooling: `repair-evidence.ts` prefixes package-relative evidence refs (needs
`maxBuffer` 128MB for `git ls-files`); `repair-async-time.ts` fixed the root-colliding
`README.md` basenames in `pkg-async-time-001/002/013` (the repair script's tracked check passes
on the repo-root README, so root-colliding basenames need quote-aware handling, and the
verifier's evidence parser needs `path:line: quote` with the space). `pkg-async-time.jsonl` now
verifies 19 of 20 rows clean; the one remaining row (020) is compact, pending its child's
rewrite.

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

1. Watch `rr-file-batch2` (background filing of 763 decisions; log `file-batch2.log`; 483
   filed, 0 errors at last check, through issue #2296). On completion: read the tail counts,
   refresh `issues-all.jsonl` (expect ~2500), spot-check a sample of new issue numbers, commit
   this handover (v9 committed at the 483-filing mark; the next commit is v10).
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
