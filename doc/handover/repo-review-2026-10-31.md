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
  the orchestrator spot audit below; machine scans do not close prose areas.
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

Cross-cutting concerns planned, one subagent each: `GRW`, `TWS`, workspace deps + `ST3` imports,
logging adherence, error-handling patterns, async + unbounded input, manifest and mise
conventions, test-convention health, TSDoc quality, CSS quality, security sweep.

## Harness limits measured this session

- Subagent child limit: about 32 admitted children including idle ones; child names are
  permanent. The sweep runs as a worker pool: idle children are re-tasked via
  `tools.agents.send_message` with a fresh standalone brief. Proven repeatedly.
- `run_typescript` 60-second budget; a spawn costs 5 to 8 seconds (at most 2 to 3 per program);
  `send_message` is cheap (7 per program fine).
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
  the property is `CPUQuotaPerSecUSec`.
- Orchestrator spot audit (5 planning files fully read, 2026-10-31): the machine scan's yield is
  roughly an order of magnitude low. Solid findings the scan missed: which-lane-ships.md ALL-CAPS
  emphasis (WR3, six spans at lines 37, 58, 98, 118, 140, 258); tsdown-removal.md two stale
  "Open items" that actually landed (rolldown catalog floor now `>=1.2.15` at
  pnpm-workspace.yaml:185; the "still owed" package-externals and shebang-executable unit tests
  both exist under package/config/rolldown/src/); wall-clock-bound-test-remedies.md:491
  instructs saving probe scripts as `.mjs` against NMJ. wg-allowedips.md and unified-linter.md
  were clean at the in-file level. Deep-read yield: about 0.8 solid findings per file versus the
  scan's 0.07.

## Central runs

- `mise run lint` completed (exit 1); log `lint-full.log`. Triage into a per-package digest is
  still pending; distribute diagnostics to owning children.
- `mise run test:unit` running capped as `rr-central-tests`; log `test-unit-full.log` (23.7k
  lines at last check, all `[PASS]` so far).
- Dedupe index: 1787 tracker issues in `issues-all.jsonl`. Refresh after every filing batch.
- systemd v259 shallow clone at `${HOME}/temp/agent/systemd-v259-2026-10-31` for the
  troubleshooting doc source trace (source trace complete at commit 9ca433482f; permalink verification running).

## Filing pipeline

Scripts in `${HOME}/temp/agent/repo-review-2026-10-31/`:

- `verify-findings.ts <repoRoot> <findings...>`: schema, tracked path:line bounds, evidence
  quote match; writes `<findings>.verdicts.jsonl`. Pass explicit paths, never a glob (outputs
  match `*.jsonl` too).
- `dedupe-findings.ts <issues.jsonl> <findings...>`: rare-token scoring (tokens in at most 40
  issue titles; 5 per title match, 1 per body match; candidates at score 10). The first version
  scored generically and matched everything; this version is the reset. Writes
  `<findings>.candidates.jsonl`.
- `draft-issues.ts <outDir> <findings...>`: builds plan files `<name>.plan.jsonl`; entries
  `action: new` (auto draft, labels needs-triage + category + difficulty by severity:
  blocker hard, warning medium, nit easy) or `adjudicate` (verdict problems or dedupe
  candidates). Body format: `## What happens`, `## Evidence (2026-10-31)` with verbatim quotes,
  "No preferred resolution has been picked" line, sweep provenance footer.
- Pipeline state (2026-10-31, wave 6): 33 findings files, 904 verified findings. Verify and
  dedupe run per wave; plans drafted per file, refreshed as findings grow.
  `classify-adjudications.ts` sorts adjudications by title-token Jaccard against dedupe
  candidates: 445 auto-new, 9 auto-comment (Jaccard at least 0.45), 238 manual (168 ambiguous
  0.2 to 0.45; 70 with verifier problems: quote mismatches or missing path:line refs, repair
  before filing). `file-issues.ts` executes decisions: creates issues with labels (creating
  missing labels first), comments on existing issues with a sweep header, appends to
  `filed-log.jsonl`, idempotent by filed id. None filed yet; first batch next. Stale plans
  pending re-draft: docs-core, w2-throw-async-a, w2-th-diag-expect, w2-dom-a, docs-decisions,
  root-configs, w2-logger-src, w2-logger-tests, docs-ops, w2-th-core.
## Children

All 32 admitted children re-tasked as a worker pool. Current assignments (name -> area,
prior findings count):

- `root-configs` -> file-enforcer/core slice 1 (files 1-80 of about 242). Prior: 47.
- `ci-depot` -> cross-cutting: security and syntax-boundary safety (SYB, STB, PRV). Prior:
  40 (kv-store/memoize: undefined-recomputes claim false; cache-key colon collisions; async
  clear races).
- `docs-core` -> doc/audit/resharp-fuzz-2026-06-11 continuation (13 zero-coverage files).
  Prior: 29.
- `docs-decisions` -> redo ST3 import-side verification with both scan bugs fixed. Prior: 30
  (208 package.json stale org; zod literal dep; peerDependencies "*" vs catalog: split).
- `docs-ops` -> error-signaling remainders (fixed @throws detector, builtin-vs-custom,
  upstream-fork). Prior: 31.
- `docs-planning-audit` -> cross-cutting: CSS quality (CXD, HDM, css skill). Prior: 89
  (fs-id: parseDiskutilVolumeUuid binds a later key).
- `docs-skills` -> env-local-json/core, the red workflow root cause. Prior: 23
  (marketplace.json ./packages/ blocker; poll-issue ][ merge corruption).
- `docs-philosophy` -> cross-cutting: TSDoc quality (TD1 through TD7). Prior: 17
  (tool-choices stale claims; mid-token hard wraps).
- `pkg-dom` -> cross-cutting: logging discipline (TLG, LG1, LG2). Prior: 23 (dom/core: e2e
  reads unbuilt dist; root export maps dist).
- `pkg-async-iter` -> doc/planning i-m continuation from issue-659. Prior: 12.
- `pkg-async-time` -> monochromatic-lint slice 1 (files 1-80 of 243). Prior: 14.
- `pkg-figma-kiwi` -> cross-cutting TWS continuation (Rust twins, encode/decode pairs,
  export mirrors, task.ts-only packages). Prior: 29 (7 mise/task.ts twin drift; kiwi ZIP
  constants re-derived).
- `pkg-figma-to-penpot` -> cross-cutting: test-suite health (TCV, TRV, GFP, CXL). Prior: 34
  (multi-page root-frame loss; phantom parent; output overwrites input).
- `pkg-observable` -> doc/audit regex-necessity remainder plus tech-monorepo-manager-vet.
  Prior: 22 (build-mirror literal `${AUDIT}` directory, blocker).
- `pkg-shell-argument` -> doc/audit resharp-fuzz-06-19 plus bad-glm plus tech-meow
  (working).
- `pkg-throws` -> cross-cutting: manifest and mise conventions (PKG, AP2, AP4, SGD, SBS,
  CM2). Prior: 5.
- `sysd-source-trace` -> systemd permalink verification for the troubleshooting doc. Source
  trace complete at commit 9ca433482f (tag v259).
- `w2-async-b1` -> cross-cutting: async and unbounded input (PP1, PP2, RG2). Prior: 24
  (doc/agent: doc/adr denial contradicts the tree; regression case cites a removed path).
- `w2-async-b2` -> linter closure package. Prior: 13 (stale sinon shim premise).
- `w2-dom-a` -> doc/planning t-z continuation. Prior: 30.
- `w2-fs-a` -> doc/audit d-h slice (working, no report yet).
- `w2-fs-b` -> doc/planning a-d continuation. Prior: 29.
- `w2-i18n` -> kotlin linter package. Prior: 28 (doc/runbook: roster-fixture.ts deleted,
  blocker).
- `w2-logger-fuzz` -> doc/audit a-c slice (working, no report yet).
- `w2-logger-src` -> doc/troubleshooting prose slice (first 25 sorted). Prior: 36.
- `w2-logger-tests` -> first oxlint-plugin package. Prior: 34 (zip-writer CRC by reference
  blocker; dos-time year wrap).
- `w2-th-core` -> doc/audit i-p continuation (4 files left in window). Prior: 35.
- `w2-th-diag-expect` -> first forbidden-strings package. Prior: 47 (planning e-h slice
  complete: stale Status blocks throughout).
- `w2-th-matrix-sandbox` -> doc/planning n-s continuation. Prior: 34.
- `w2-throw-async-a` -> first monorepo-task package. Prior: 31 (core-expect:
  toHaveProperty undefined false pass; calledWith loose arity).
- `w2-typeface-figma` -> doc/audit q-z slice (working, no report yet).

Brief template v4 is in the transcript (reviewer-only, main-worktree protection, cgroup wrapper,
JSONL schema, 250-word final message; deep-doc variant adds per-file accounting and the
25-file target).

## Assignment queue (re-task idle children in this order)

1. doc/audit deep slices: top-level a-c, d-h, i-p, q-z are running; next: doc/audit subdirs
   (resharp-fuzz-2026-06-04 29 files, -06-11 24, regex-necessity 22, tech-monorepo-manager-vet
   17, resharp-fuzz-2026-06-19 13, bad-glm 10, tech-meow 2; code and fixture review) and any
   top-level remainders the slices report.
2. Cross-cutting concerns, one each: `GRW`, `TWS`, workspace deps + `ST3` imports, logging
   adherence, error-handling patterns, async + unbounded input, manifest and mise conventions,
   test-convention health, TSDoc quality, CSS quality, security sweep.
3. Linter tooling: `monochromatic-lint` (243 files, split), its closure, `kotlin`,
   `oxlint-plugin/*` (split), `monorepo-task/*`, `file-enforcer/core` (242, split) plus `cli`,
   `forbidden-strings/*` (split), `mutation-test/*`, `open-code-review-issue/cli`,
   `env-local-json/core` (owns the ci-depot blocker root cause).
4. Apps: `done/app`, `done/variant-postcss`, `rss/app`, `wc/app`, `doodle-widget/app`,
   `syllable-break-demo/app`, `terminal/app`, `ssg/aquati.cat` (split), `ide/app` (1134, split
   ~3), `file-manager/*` (4), music-player `design` (1269, split ~3), `desktop-app` (361, split
   2), `android-app` (354, split 2), `asset`, `truepeak-core` plus bench, `desktop-e2e`.
5. Plugins: `agent-harness-shared/*` (9), `claude-code-plugin/*` (~14), `pi-plugin/*` (~18),
   `dsh-plugin/*`, `codex-plugin/bash-limit`, `search-fetch/*` (core has concurrent edits),
   `electron-infra/*`, `cloudflare-worker/rand`, `claude-cpu-cap/daemon`, `hall-monitor/daemon`,
   `mcp/*`.
6. Rust: `git-policy/cli` (1030, split 3-4) plus fuzz, `file-enforcer/core-rust`,
   `browserslist/core-rust`, `jsonc-edit/core-rust` plus fuzz, `xml-edit/core-rust` plus fuzz,
   `dependency-catalog/*`, `wg/core-allowedips`, `linter/rust`, `learning/rust`,
   `ownership-marker/foreign-borrowed`, `mpm-shadow/cli`, `search-shadow/cli`,
   `crate-release/cli`, `mvm/cli`, `nested-wayland-session/cli`, `vm-builder/cli`, `vmsync/cli`.
7. `translation-repair/*` (~35, one per assignment); `upstream-fork/*` (~12 plus fuzz sidecars);
   `config/*` (9; pnpr has concurrent edits); misc: `backup-path/cli`,
   `android-exempt-unused/cli`, `deps-update/cli`, `fy/cli`, `git-clone-size/cli`,
   `git-executable/core`, `rgffplay/cli`, `page-weight/cli`, `terminal-exec/cli`,
   `watch-restart/cli`, `kwin/key-helper`, `intellij-plugin/islands-black`,
   `rolldown-plugin/import-attributes`, `pnpm-override/*` (7), `runtime-error/bun`,
   hyperscript-class remainders, `test-fixture/*` source code only.

Calibration: single-package deep reviews fit one turn each at 8 to 26 findings per package
(async-iter 12 files/8, matrix 21/24, hyperscript 19/21, logger-src 40/26, typeface 20/37,
th-core 52/18). Doc deep slices target 25 to 40 files per turn; planning has 485 top-level
files across six running slices, audit about 430 across four.

## Concurrent work (do not touch)

Another session has uncommitted edits in flight: `package/config/pnpr/config.yaml` (staged),
`package/search-fetch/core/src/migrate.ts`,
`package/search-fetch/core/src/scratch-parent.ts`,
`doc/audit/standards-review-gate-shadow.md`,
`doc/handover/file-enforcer-rust-rewrite.md`. Findings on these files are marked "concurrent
edits in flight".

## Next actions

1. Commit this handover update (v6).
2. Write the systemd troubleshooting doc: harness verified (both failure variants plus the
   loud-failure contrast), permalink verification running in `sysd-source-trace`;
   `.out-of-scope/` checked with no systemd exemption; run the tracker duplicate search,
   the 6-constraint audit, then write
   `doc/troubleshooting/systemctl-show-scope-stub-properties.md` and commit.
3. Re-draft the stale plans (10 files listed under Filing pipeline), re-run the classifier,
   spot-check the auto decisions (all 9 auto-comments plus a sample of auto-new), file batch 1
   via `file-issues.ts`, refresh `issues-all.jsonl`.
4. Work the 238-entry manual queue in batches (ambiguous Jaccard 0.2 to 0.45 need judgment;
   problem entries need evidence repair first).
5. Triage `lint-full.log` into a per-package digest; distribute to owning children. Check
   `rr-central-tests` completion; capture failures.
6. Keep re-tasking idle children from the queue; update and commit this handover at every
   wave transition.

## Stop conditions

No code changes in the main worktree; experiments run only in throwaway worktrees. Tracker
writes happen only after orchestrator verification. Every new issue body states that no
preferred resolution has been picked. Subagent scopes stay within one package (scanning) or one
concern (cross-cutting) and within the ~60-minute budget. Machine scans alone do not close a
prose area: deep reads with per-file accounting do.
