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

## Purpose and method

The whole tracked tree is reviewed by scanning subagents (exactly one package each; giant packages
split by subdirectory, still one package) plus cross-cutting subagents (exactly one concern each,
spanning any number of packages by design). Briefs demand: read every tracked file in scope, quote
evidence from the current tree with `path:line`, no edits, no tracker writes, no spawning,
prioritize and report uncovered files rather than compress verification quality inside the
~60-minute budget.

Findings land as JSONL under `${HOME}/temp/agent/repo-review-2026-10-31/findings/`. The
orchestrator re-verifies each finding against the tree, deduplicates against the tracker index,
then files: a comment on the owning existing issue for matches, otherwise a new issue with the
labels above, every body stating that no preferred resolution has been picked. Filing is
incremental per wave.

Cross-cutting concerns planned, one subagent each (many packages allowed): `GRW` duplicate shapes
without owners, `TWS` two-sided code without a stated arrangement, workspace dependency structure
and `ST3` `/ts` imports, repo-wide logging-rule adherence, error-handling patterns, async and
unbounded-input patterns, package manifest and mise task conventions, test-convention health,
TSDoc quality, CSS quality, security sweep.

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
  limits correctly (`CPUQuotaPerSecUSec=370ms`, `MemoryMax=134217728` for `CPUQuota=37%`
  `MemoryMax=128M`), matching the kernel files (`cpu.max: 37000 100000`,
  `memory.max: 134217728`). Queried with a bare unit name, `systemctl show` silently mangles the
  name to `.service`, reports `LoadState=not-found` with stub defaults
  (`MemoryMax=infinity`, `CPUQuotaPerSecUSec=infinity`), and exits 0. The first probe's "show
  displays infinity even when applied" reading was this name-mangling footgun, not a systemd
  display bug; the briefs and queued addendum claiming otherwise are corrected in the re-slice
  interrupt messages. Also: `show -p CPUQuota` prints nothing; the property name is
  `CPUQuotaPerSecUSec`.

## Central runs

- `mise run lint` runs capped as `rr-central-lint` (800% / 12G / 1024), process
  `process-5367861df51f-3afa61a56d494ac4efca2f9d`, log at
  `${HOME}/temp/agent/repo-review-2026-10-31/lint-full.log`. Its per-package diagnostics are
  distributed to the owning scanning child for a verdict plus evidence before anything is filed.
- `mise run test:unit` starts under the same sizing after the lint run finishes.
- Dedupe index: all 1787 tracker issues fetched to
  `${HOME}/temp/agent/repo-review-2026-10-31/issues-all.jsonl` (number, title, state, labels,
  body).
- systemd `v259` shallow clone completed at `${HOME}/temp/agent/systemd-v259-2026-10-31`
  (process `process-5367861df51f-c147e175521274fc29f89757`, exit 0) for the troubleshooting doc
  source trace.

## Children

Scanning children running, with the one-package re-slice delivered by interrupt plus narrowing
message (multi-package children keep their first package; orphaned packages get fresh
single-package children, to be spawned):

- `root-configs`: root config and build files (one area, no narrowing).
- `ci-depot`: `.github/` and `.depot/` (one area, no narrowing).
- `docs-core`: narrowed to the rulebook files only (`AGENTS.md`, `CLAUDE.md`, `README.md`,
  `SECURITY.md`).
- `docs-decisions`: `doc/decision/` (one family, no narrowing).
- `docs-ops`: narrowed to `doc/troubleshooting/` only.
- `docs-planning-audit`: narrowed to `doc/planning/` only.
- `w2-throw-async-a`: narrowed to `package/or-throw/core`; orphans: `array-at/core`,
  `async-iter/core`, `async-time/core`, `caught-value/core`, `const/core`.
- `w2-async-b1`: narrowed to `package/function-arity/core`; orphans: `numeric-format/core`,
  `observable/core`.
- `w2-async-b2`: narrowed to `package/pipe/core`; orphans: `throws/core`, `shell-argument/core`.
- `w2-fs-a`: narrowed to `package/fs-path/core`; orphan: `fs-id/core`.
- `w2-fs-b`: narrowed to `package/kv-store/core`; orphans: `kv-store/memoize`, `zip-writer/core`.
- `w2-dom-a`: narrowed to `package/hyperscript/core`; orphan: `dom/core`.
- `w2-typeface-figma`: narrowed to `package/typeface/aquaticat`; orphans: `figma/kiwi`,
  `figma/to-penpot` (never assigned before).
- `w2-th-diag-expect`: narrowed to `package/test-harness/core-diagnostic`; orphan:
  `test-harness/core-expect`.
- `w2-th-matrix-sandbox`: narrowed to `package/test-harness/core-matrix`; orphan:
  `test-harness/core-sandbox`.
- Single-package children already compliant: `w2-i18n` (`i18n-compose/core`), `w2-logger-src`
  and `w2-logger-tests` (`logger/core`, split by files), `w2-logger-fuzz` (`logger/core.fuzz`),
  `w2-th-core` (`test-harness/core`).
- Cross-cutting child: `sysd-source-trace` (systemd v259 source citations for the
  troubleshooting doc; single concern, exempt from the one-package rule).

Batch-one children (`root-configs` through `w2-throw-async-a`) were spawned before the cgroup
rule landed; the rule, the corrected `systemctl show` guidance, and the narrowing were delivered
by the interrupt plus message sent this session. Batch-two children carried the cgroup rule in
their briefs and received the narrowing plus the corrected caveat by interrupt plus message.

## Concurrent work (do not touch)

Another session has uncommitted edits in flight: `package/config/pnpr/config.yaml` (staged),
`package/search-fetch/core/src/migrate.ts`,
`package/search-fetch/core/src/scratch-parent.ts`,
`doc/audit/standards-review-gate-shadow.md`,
`doc/handover/file-enforcer-rust-rewrite.md`. Findings on these files are marked "concurrent
edits in flight".

## Next actions

1. Commit this handover, then keep updating and committing it at every wave transition.
2. Spawn the orphaned single-package scanning children (21: `array-at`, `async-iter`,
   `async-time`, `caught-value`, `const`, `numeric-format`, `observable`, `throws`,
   `shell-argument`, `fs-id`, `kv-store/memoize`, `zip-writer`, `dom`, `figma/kiwi`,
   `figma/to-penpot`, `test-harness/core-expect`, `test-harness/core-sandbox`, plus docs
   children for `doc/agent` plus `doc/philosophy`, the `.agents/` skills plus `.claude-plugin`,
   `doc/runbook`, and `doc/audit`).
3. Spawn the cross-cutting children (one concern each, many packages allowed).
4. When `sysd-source-trace` returns: verify its citations against the clone, write
   `doc/troubleshooting/<topic>.md` for the `systemctl show` bare-name footgun (symptom, source
   trace, verification catalogs, workarounds, 6-constraint upstream-filing audit; check
   `.out-of-scope/` first), and commit it.
5. After the central lint finishes, start `mise run test:unit` capped, then distribute package
   diagnostics.
6. As children report: verify findings, dedupe against the index, comment or open issues; update
   this handover at every wave transition.

## Stop conditions

No code changes anywhere. Tracker writes happen only after orchestrator verification. Every new
issue body states that no preferred resolution has been picked. Subagent scopes stay within one
package (scanning) or one concern (cross-cutting) and within the ~60-minute budget.
