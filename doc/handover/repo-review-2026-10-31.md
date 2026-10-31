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
  out, and the rest of the tracked tree is in (root configs, `.github/`, `.depot/`, `.agents/`,
  `doc/` living families, `LICENSES/`, `playwright/`).
- Labels on new issues: full set, `needs-triage` plus a type label plus one `difficulty:*`.
- Cgroups: "Since this specific mistral vibe harness doesn't have auto cgrouping setup, run every
  command in its own resource limited (to how much it needs) cgroup."
- Slice size: "each subagent's slice should be very small - aim for max 60min per subagent and
  calibrate as you go."
- One package per scanning subagent: "max 1 pkg per subagent. You can't give someone 2 things to
  do and except good results in anything."
- Cross-cutting subagents: "For cross cutting concerns one agent can be assigned many packages at
  the same time." They carry exactly one concern each and are exempt from the one-package rule.
- Handover: "this is a huge task so write a handover NOW and update it as you go aggressively."
- Yield challenge (2026-10-31): "1053 files and only 75 findings is extremely sus." The
  orchestrator treats machine-scan-only yields as suspect; deep slices must read prose in full and
  account per file.

## Purpose and method

The whole tracked tree is reviewed by scanning subagents (exactly one package each; giant packages
split by subdirectory, still one package) plus cross-cutting subagents (exactly one concern each,
spanning any number of packages by design). Briefs demand: read every tracked file in scope, quote
evidence from the current tree with `path:line`, no main-worktree edits, no tracker writes, no
spawning, prioritize and report uncovered files rather than compress verification quality inside
the ~60-minute budget. For doc areas, mechanical scans alone are not sufficient: the yield
challenge above requires deep-read slices with per-file accounting.

Findings land as JSONL under `${HOME}/temp/agent/repo-review-2026-10-31/findings/`. The
orchestrator re-verifies each finding against the tree (`verify-findings.ts`), deduplicates
against the tracker index (`dedupe-findings.ts`), adjudicates, then files: a comment on the
owning existing issue for matches, otherwise a new issue with the labels above, every body stating
that no preferred resolution has been picked. Filing is incremental per wave; the index is
refreshed after each filing batch so later children's duplicates of just-filed issues dedupe to
comments.

Cross-cutting concerns planned, one subagent each (many packages allowed): `GRW` duplicate shapes
without owners, `TWS` two-sided code without a stated arrangement, workspace dependency structure
and `ST3` `/ts` imports, repo-wide logging-rule adherence, error-handling patterns, async and
unbounded-input patterns, package manifest and mise task conventions, test-convention health,
TSDoc quality, CSS quality, security sweep.

## Harness limits measured this session

- Subagent child limit: the harness rejects spawns at about 32 admitted children, and the count
  includes idle (completed) children (`tools.agents.list()` showed 32 admitted with 4 idle at the
  moment the limit fired). Child names are permanent. Consequence: the sweep runs as a worker
  pool: at most ~32 children ever; as each child goes idle it is re-tasked, by
  `tools.agents.send_message` with a fresh standalone brief, with the next assignment from the
  queue in this handover. Re-tasking to an idle child is proven to work.
- `run_typescript` programs have a 60-second budget and each spawn takes 5 to 8 seconds: at most
  3 spawns per program, ideally 2. `send_message` calls are cheaper; 7 fit in one program.
- `gh issue list` here has no `--paginate`; use `--limit 3000`.

## Facts measured (re-verify before trusting)

- Host at sweep start (2026-10-31): 32 cores, 61 GiB total with 22 GiB available, load average
  92.95, systemd 259 (259.10-1.fc44), cgroup v2 unified hierarchy.
- Tracked file census: `package/` 16347 files, `doc/` 2582, `package-paused/` 1174 (out of scope),
  `package-deprecated/` 42 (out of scope), `.github/` 25, `.idea/` 29, `.agents/` 12, plus the
  root config files listed in the root-configs child's scope.
- Cgroup wrapper verified while a scope was alive: for `CPUQuota=50%` and `MemoryMax=256M` the
  kernel files showed `cpu.max: 50000 100000` and `memory.max: 268435456`. Verified wrapper:
  `systemd-run --user --scope --unit=rr-<name>-<serial> -p CPUQuota=<N>% -p MemoryMax=<SIZE> -p
  TasksMax=<N> -- <command>`. Sizing in force: reads and queries 100% / 1G / 128 tasks; one
  package's `lint:types` or a few unit-test files 200% / 2G / 256; one crate's `cargo nextest`
  or `cargo build` 300% / 4G / 256; repo-wide aggregates 800% / 12G / 1024.
- `systemctl --user show` behavior, settled by the differential probe
  (`probe2-cgroup-show.ts`): with the full `<unit>.scope` name, `show` displays the applied
  limits correctly. Queried with a bare unit name, `systemctl show` silently mangles the name to
  `.service`, reports `LoadState=not-found` with stub defaults, and exits 0. Also: `show -p
  CPUQuota` prints nothing; the property name is `CPUQuotaPerSecUSec`.

## Central runs

- `mise run lint` completed (exit 1), log at
  `${HOME}/temp/agent/repo-review-2026-10-31/lint-full.log`. Still to triage into a per-package
  digest, then distribute package diagnostics to owning children for a verdict plus evidence.
- `mise run test:unit` started capped as `rr-central-tests` (800% / 12G / 1024), log at
  `${HOME}/temp/agent/repo-review-2026-10-31/test-unit-full.log` (process
  `process-5367861df51f-5da96774b169cb0e8ea33d48`).
- Dedupe index: all 1787 tracker issues fetched to
  `${HOME}/temp/agent/repo-review-2026-10-31/issues-all.jsonl`.
- systemd `v259` shallow clone completed at `${HOME}/temp/agent/systemd-v259-2026-10-31` for the
  troubleshooting doc source trace.

## Filing pipeline

- `${HOME}/temp/agent/repo-review-2026-10-31/verify-findings.ts <repoRoot> <findings.jsonl> [...]`:
  schema check, cited `path:line` tracked and in bounds, evidence quote match against the tree;
  writes `<findings>.verdicts.jsonl` next to the input. Note: script outputs match `*.jsonl`, so
  pass explicit findings paths, never a glob.
- `${HOME}/temp/agent/repo-review-2026-10-31/dedupe-findings.ts <issues.jsonl> <findings.jsonl> [...]`:
  token-overlap candidates against the issue index; writes `<findings>.candidates.jsonl`.
- Remaining: adjudicate candidates (comment versus new issue), draft bodies in the house format
  (title `<area>: <problem>`, `## What happens`, `## Evidence (2026-10-31)` with `path:line` and
  verbatim quotes, explicit "no preferred resolution has been picked" line), then file via
  `gh issue create` / `gh issue comment` after orchestrator verification. First batch: 210
  findings across 9 files (see Children). Refresh the issue index after each batch.

## Children

Completed turn 1 or 2, findings accepted (idle; most re-tasked already):

- `docs-planning-audit`: 75 findings (13 warning, 62 nit) over all 1053 `doc/planning` plus
  `doc/audit` files, machine-scanned. Yield challenged by the user; deep re-slices queued.
  Re-tasked to `package/fs-id/core`.
- `docs-ops`: 19 findings (3 warning, 16 nit), all 819 `doc/troubleshooting` files
  machine-scanned. Remainder queued: 79 fixture and code files plus per-command flag
  verification. Re-tasked to `package/numeric-format/core`.
- `docs-core`: 15 findings (1 blocker, 7 warning, 7 nit), four rulebook files read in full.
  Blocker: `.claude-plugin/marketplace.json` sources use `./packages/...` but the repo directory
  is `package/`, so every `claude plugin install` fails. Re-tasked to `package/caught-value/core`.
- `docs-decisions`: 23 findings in `findings/docs-decisions.jsonl` (report text never reached the
  orchestrator; file verified present). Re-tasked to `package/const/core`.
- `ci-depot`: 32 findings (1 blocker, 19 warning, 12 nit), all 27 in-scope files. Blocker:
  `env-local-json.yml` red on main, six consecutive failed runs, `spawnSync /root/.local/bin/mise
  ENOENT` inside `ubuntu:latest`; root cause lives in `package/toolchain/env-local-json` code (a
  scanning child must cover it). Re-tasked to `package/kv-store/memoize`.
- `w2-logger-tests`: 12 findings (5 warning, 7 nit), logger/core task and CI wiring slice.
  Re-tasked to `package/zip-writer/core`.
- `w2-throw-async-a`: 19 findings (9 warning, 10 nit) for `or-throw/core` full plus verified
  findings on its five dropped packages, marked with scope notes (dedupe against the fresh
  pkg-* children when they report). Re-tasked to `package/test-harness/core-expect`.
- Partial findings files exist already (children still running): `w2-async-b2.jsonl` (9),
  `w2-fs-a.jsonl` (6).

Running (with current assignment): `root-configs` (root configs), `sysd-source-trace` (systemd
v259 source citations), `w2-async-b1` (function-arity/core), `w2-async-b2` (pipe/core),
`w2-fs-a` (fs-path/core), `w2-fs-b` (kv-store/core), `w2-dom-a` (hyperscript/core),
`w2-typeface-figma` (typeface/aquaticat), `w2-th-diag-expect` (test-harness/core-diagnostic),
`w2-th-matrix-sandbox` (test-harness/core-matrix), `w2-i18n` (i18n-compose/core),
`w2-logger-src`, `w2-logger-fuzz` (logger/core), `w2-th-core` (test-harness/core),
`pkg-array-at`, `pkg-async-iter`, `pkg-async-time`, `pkg-observable`, `pkg-throws`,
`pkg-shell-argument`, `pkg-dom`, `pkg-figma-kiwi`, `pkg-figma-to-penpot`, `docs-philosophy`,
`docs-skills`, plus the seven just re-tasked (docs-core, docs-decisions, docs-ops,
docs-planning-audit, ci-depot, w2-logger-tests, w2-throw-async-a).

Brief template v4 (in the transcript): reviewer-only, one area, main-worktree protection with
throwaway-worktree experiments allowed, cgroup wrapper with corrected `.scope`-suffix caveat,
JSONL schema, final message under 250 words. For doc deep slices add: every file read in full
gets an explicit verdict, and the final message reports files reviewed versus findings per file.

## Assignment queue (re-task idle children in this order)

1. `package/test-harness/core-sandbox`
2. `doc/agent/`
3. `doc/runbook/`
4. `doc/troubleshooting/` code and fixture files (79) plus per-command flag verification
5. `doc/audit/` and `doc/planning/` deep passes, re-sliced by subdirectory with per-file
   accounting (see the yield challenge; slice sizes from the filelist subdir stats)
6. Cross-cutting concerns, one each: `GRW`, `TWS`, workspace deps + `ST3` imports, logging
   adherence, error-handling patterns, async + unbounded input, manifest and mise conventions,
   test-convention health, TSDoc quality, CSS quality, security sweep
7. Linter tooling: `monochromatic-lint` (243 files, split), its closure, `kotlin`,
   `oxlint-plugin/*` (split), `monorepo-task/*`, `file-enforcer/core` (242, split) plus `cli`,
   `forbidden-strings/*` (split), `mutation-test/*`, `open-code-review-issue/cli`,
   `env-local-json/core` (owns the ci-depot blocker root cause)
8. Apps: `done/app`, `done/variant-postcss`, `rss/app`, `wc/app`, `doodle-widget/app`,
   `syllable-break-demo/app`, `terminal/app`, `ssg/aquati.cat` (split), `ide/app` (1134, split
   ~3), `file-manager/*` (4), music-player `design` (1269, split ~3), `desktop-app` (361, split
   2), `android-app` (354, split 2), `asset`, `truepeak-core` plus bench, `desktop-e2e`
9. Plugins: `agent-harness-shared/*` (9), `claude-code-plugin/*` (~14), `pi-plugin/*` (~18),
   `dsh-plugin/*`, `codex-plugin/bash-limit`, `search-fetch/*` (core has concurrent edits),
   `electron-infra/*`, `cloudflare-worker/rand`, `claude-cpu-cap/daemon`, `hall-monitor/daemon`,
   `mcp/*`
10. Rust: `git-policy/cli` (1030, split 3-4) plus fuzz, `file-enforcer/core-rust`,
    `browserslist/core-rust`, `jsonc-edit/core-rust` plus fuzz, `xml-edit/core-rust` plus fuzz,
    `dependency-catalog/*`, `wg/core-allowedips`, `linter/rust`, `learning/rust`,
    `ownership-marker/foreign-borrowed` (ci-depot found the workflow filter misspells this),
    `mpm-shadow/cli`, `search-shadow/cli`, `crate-release/cli`, `mvm/cli`,
    `nested-wayland-session/cli`, `vm-builder/cli`, `vmsync/cli`
11. `translation-repair/*` (~35, one per assignment); `upstream-fork/*` (~12 plus fuzz sidecars);
    `config/*` (9; `pnpr` has concurrent edits); misc: `backup-path/cli`,
    `android-exempt-unused/cli`, `deps-update/cli`, `fy/cli`, `git-clone-size/cli`,
    `git-executable/core`, `rgffplay/cli`, `page-weight/cli`, `terminal-exec/cli`,
    `watch-restart/cli`, `kwin/key-helper`, `intellij-plugin/islands-black`,
    `rolldown-plugin/import-attributes`, `pnpm-override/*` (7), `runtime-error/bun`,
    hyperscript-class remainders, `test-fixture/*` source code only

Calibration data: machine-scan strategies covered ~1000 files per turn but with low yield (75
findings per 1053 files, 19 per 819), which the user has challenged; deep full reads fit ~4
dense files per turn. Treat prose areas as deep-read slices of 30 to 80 smaller files per
child-turn with per-file accounting, and re-slice by subdirectory.

## Concurrent work (do not touch)

Another session has uncommitted edits in flight: `package/config/pnpr/config.yaml` (staged),
`package/search-fetch/core/src/migrate.ts`,
`package/search-fetch/core/src/scratch-parent.ts`,
`doc/audit/standards-review-gate-shadow.md`,
`doc/handover/file-enforcer-rust-rewrite.md`. Findings on these files are marked "concurrent
edits in flight".

## Next actions

1. Adjudicate the verified 210-finding batch (verdicts and candidates files are being written by
   the pipeline run), draft bodies, file after verification, refresh the issue index.
2. Yield challenge: spot-audit a sample of `doc/planning` and `doc/audit` files the scan marked
   clean, count what the scan missed, then queue the deep re-slices by subdirectory with
   per-file accounting. Extend `.agents/skills/project-code-review/SKILL.md` with the
   exhaustive-review mandates once `docs-skills` (currently reviewing `.agents/`) completes, to
   avoid concurrent edits.
3. Triage `lint-full.log` into a per-package digest; distribute diagnostics to owning children.
4. When `sysd-source-trace` returns: verify citations against the clone, write
   `doc/troubleshooting/<topic>.md` (6-constraint upstream audit; check `.out-of-scope/` first),
   and commit.
5. Keep re-tasking idle children from the queue; update and commit this handover at every wave
   transition.

## Stop conditions

No code changes in the main worktree; experiments run only in throwaway worktrees. Tracker
writes happen only after orchestrator verification. Every new issue body states that no
preferred resolution has been picked. Subagent scopes stay within one package (scanning) or one
concern (cross-cutting) and within the ~60-minute budget. Machine scans alone do not close a
prose area: deep reads with per-file accounting do.
