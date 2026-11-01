# Translation repair history, first handover document: loss of the run directory, worktree move and promotion of the driver, 2026-07-24

The accidental removal of the temporary directory that held the old driver and pass artifacts,
the recovery into a durable gitignored location,
the move of the worktree out of the repository,
and the promotion of the driver and probe to committed source.

Part of [Translation repair history: the first handover document, to 2026-08-15](translation-repair-history-first-handover-to-2026-08-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Loss of the old run directory and its recovery

INCIDENT AND RECOVERY (2026-07-24):
   the user accidentally ran
`rm -rf ${HOME}/temp`,
   wiping the old out-of-repo run dir
`${HOME}/temp/translation-repair-corpus/` (driver,
   sentinel probe,
all pass-4 artifacts,
   and `pass4-run-001.log`) mid-run.
   Nothing
irreplaceable was lost:
   the pipeline code,
   this handover,
   and every
recorded decision live in git,
   and the corpus was never in `${HOME}/temp`
(it reads live via `git show` at the pinned SHA from
`${HOME}/one-among-us/data`,
   verified readable post-incident).
   Only
regenerable scaffolding and one interrupted pass's artifacts went with it.
Recovery:
   rebuilt the driver and probe grounded in the module's exported
API (`listCorpusPeople`,
   `readCorpusFile`,
   `createSyntheticClient`,
`repairTranslation`),
   not memory;
   relocated them plus artifacts to the
durable gitignored dir `node_modules/.monochromatic/translation-repair-runs/`
(user's suggestion;
   see "Where work lives").
   Verified at zero quota with
`--plan`:
   pending 92 (tdor excluded by the complete-pair filter,
   no
hardcoded exclusion),
   key injected,
   client constructs.
   Added AGENTS.md
rules TMP (`${HOME}/temp` is ephemeral,
   keep only reconstructable
scaffolding) and NMD (durable uncommittable state goes in
`node_modules/.monochromatic/`),
   regenerated CLAUDE.md,
   commit
`1831230e0`.
   Pass 4 run 001 relaunched on that tip;
   the pipeline itself
is unchanged from `63baaa686`,
   so accumulation resumes exactly where it
would have.

## Worktree move and promotion of the driver to source

WORKTREE MOVE AND SOURCE PROMOTION (2026-07-24):
   the user flagged that a
worktree under the repo's `.claude/` risks the same stray-cleanup loss as
`${HOME}/temp`,
   so the worktree moved via `git worktree move` from
`.claude/worktrees/translation-repair` to `${HOME}/worktrees/translation-repair`
(same filesystem,
   a rename;
   HEAD and all run outputs moved with it;
   `mise trust`
re-run at the new path).
   Then,
   per "driver and probe should be source code" and
"put them under src/<category>",
   the driver and probe were promoted from
gitignored `.mjs` scaffolding to committed TypeScript under
`src/corpus-run/` (`run-config.ts`,
   `corpus-pass.ts`,
   `sentinel-probe.ts`),
importing the pipeline from sibling source,
   `import.meta.main`-guarded,
   run via
new package mise tasks `corpus-pass` and `sentinel-probe`.
   Only run OUTPUTS stay
gitignored in `node_modules/.monochromatic/translation-repair-runs/`.
   See "Where
work lives".
   Only run OUTPUTS stay gitignored in
`node_modules/.monochromatic/translation-repair-runs/`.
RESOLVED (2026-07-24,
   commit `92f7b2c55`):
   the new source is green (format,
oxlint 0/0,
   types),
   `--plan` runs through the mise task (pending 92,
   tdor
excluded,
   client constructs,
   zero quota),
   the old `.mjs` copies are deleted,
   and
`buildAndTest` passes so the library is unregressed by the addition.
   Pass 4 run
002 relaunched via `mise run //package/module/translation-repair:corpus-pass` (moved on 2026-10-18 to
`//package/translation-repair/corpus-pass:corpus-pass`)
(log `node_modules/.monochromatic/translation-repair-runs/pass4-run-002.log`) on
tip `92f7b2c55`;
   the pipeline behavior is unchanged from `63baaa686` (the
intervening commits are docs,
   the worktree move,
   and this source promotion,
   none
touching pipeline logic),
   so this continues pass 4 accumulation.
   The persisted
`attempts.json` survived,
   so entries attempted-but-never-settled by the wiped
runs (e.g. Acheron) now sort after the untouched zero-attempt entries.
