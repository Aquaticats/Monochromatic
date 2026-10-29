# Translation repair history, first handover document: the goal of the package and where the work lived, with the commit list

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/forbidden-strings` is now `package/forbidden-strings/cli`.

The goal of the package as the first handover document stated it:
a pure function from original and translated text to validated issues and a conservative repaired candidate,
built for individually unreliable models,
and the places the work lived:
the worktree and the branch it moved to,
the committed driver and probe and how to run them,
where run outputs stay out of git,
and the commits on the branch.

Part of [Translation repair history: the first handover document, to 2026-08-15](translation-repair-history-first-handover-to-2026-08-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Goal

`@monochromatic-dev/module-translation-repair`:
a pure fn taking (original zh text,
translated en text)
and returning validated issues plus a conservative repaired candidate.
Built for individually unreliable flat-rate models;
no single model output is ever a decision point.

Everything starts as a pure fn (user requirement);
consumers and deployment are deliberately out of scope for now.

## Where work lives

- Worktree `${HOME}/worktrees/translation-repair`,
  branch
  `translation-repair-rebased` since 2026-08-06.
  The work MOVED off
  `translation-repair`:
  that branch is protected against force-push,
  so the
  rebase onto main could never land on it,
  and the remote copy is frozen at the
  pre-rebase history.
  Work on `translation-repair-rebased` from now on,
  and
  treat `origin/translation-repair` as stale rather than as the branch to push
  to.
  The pre-rebase tip is also kept locally as
  `translation-repair-prerebase-backup`.
  Moved 2026-07-24 (`git worktree move`) out of the old in-repo
  `.claude/worktrees/translation-repair`,
  which risked the same stray-cleanup loss
  as `${HOME}/temp`;
  it now sits alongside the repo's other worktrees under
  `${HOME}/worktrees/`.
  After the move,
  run `mise trust` at the new path.
- Use `/usr/bin/git` for commits in this worktree for this session (user authorization):
  the policy shim fails because the `forbidden-strings` scanner is a gitignored Rust build artifact
  (`package/cli/forbidden-strings/target/release/`) absent from fresh worktrees.
- `.env.local.json` copied from main worktree;
  `TRANSLATION_REPAIR_SYNTHETIC_API_KEY` resolves through mise sops (verified by name,
  never print values).
- Corpus-pass driver and sentinel probe are COMMITTED SOURCE (2026-07-24,
  user
  directive "driver and probe should be source code") under
  `package/module/translation-repair/src/corpus-run/` (moved on 2026-10-18 to
  `package/translation-repair/*/src/corpus-run/`):
  `run-config.ts` (shared roster,
  budgets,
  corpus pin,
  worktree/runs-dir
  resolvers,
  client factory),
  `corpus-pass.ts` (full-corpus accumulation pass),
  `sentinel-probe.ts` (named-entry validation).
  They import the pipeline from
  SIBLING SOURCE and are `import.meta.main`-guarded,
  following the repo's
  executable-in-src pattern (e.g. `package/dev-script/watch-restart/src/cli.ts`);
  no hardcoded dist path.
  Run via mise tasks (package `mise.toml`):
  `mise run //package/module/translation-repair:corpus-pass` (moved on 2026-10-18 to
  `//package/translation-repair/corpus-pass:corpus-pass`) (append `-- --plan`
  for the zero-quota setup check that verifies imports,
  corpus reads,
  filtering,
  ordering,
  key injection,
  and client construction),
  and
  `mise run //package/module/translation-repair:sentinel-probe -- <id>...` (moved on 2026-10-18 to
  `//package/translation-repair/bench:sentinel-probe`).
  Only the RUN OUTPUTS stay gitignored and out of git:
  per-entry artifacts,
  `attempts.json`,
  and run logs live in
  `node_modules/.monochromatic/translation-repair-runs/` (gitignored so
  UNLICENSED-corpus-derived artifacts can never be committed,
  and outside
  `${HOME}/temp` so cleanup cannot wipe them;
  AGENTS.md rules TMP and NMD).
  Override that dir with `TRANSLATION_REPAIR_RUNS_DIR`.
- Commits on branch:
  `16864f509` scaffold,
  `70aaaf557` catalog remark/MDX parser stack,
  `da689f628` document model and segmentation core,
  `401aa7db8` this handover,
  `411931d21` issue model and deterministic anchor validation,
  `c38ffd823` injected-transport Synthetic model client,
  `96eb1f68a` thinking-dominated output and API refusal field handling,
  `7347a73f7` pinned local corpus reads (plus `resolveGit` barrel export),
  `4cd25ae95` seeded-error benchmark harness and scorecard,
  `8f209692a` streamed chat completions,
  `2c90224ba` per-call deadlines (first,
  broken attempt via `AbortSignal.any`),
  `18a8e95ca` pack-scaled concurrency plus working timer-driven deadlines,
  `735e1b34e` seed derivation skips MDX/JSX delimiter-bearing sentences,
  plus docs commits after each task.
