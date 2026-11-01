# Translation repair history, first handover document: round two's band filter rejected, and the hazards to the graded sheet and the runs directory, 2026-07-27

Why a band filter was rejected as redundant with the band ranking,
the graded sheet that a routine draw would have overwritten and the two defences added,
the runs directory inside the reach of a clean reinstall and its backup,
and the final draw made one-shot.

Part of [Translation repair history: the first handover document, to 2026-08-15](translation-repair-history-first-handover-to-2026-08-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Round two's band filter rejected, and the hazards to the graded sheet and the runs directory

- Band FILTERING in `corpus-pass.ts` (skip bands already at the ~10 quota so no
  run spends ~75 min settling a band that is already full):
  REJECTED as
  redundant,
  not as wrong in intent.
  `rankWithinBands` already offsets each
  band's rank by `countSettledPerBand`,
  so a band that is ahead is
  automatically deprioritized by exactly its lead.
  Verified against live
  counts at 7 small / 8 medium / 7 large:
  medium's first pending entry ranks 8
  while small's and large's rank 7,
  so medium cannot be started until the other
  two catch up.
  Confirmed in flight rather than only on paper,
  since run 017
  picked MTF_0615 (5229 B,
  LARGE),
  a band that is behind.
  Adding a filter on
  top would be a second mechanism for one invariant and a place for the two to
  disagree.
  The one path that DOES bypass the offset is resume-first,
  which is
  intended:
  finishing a cached large document beats starting a fresh anything.
- Grading sheet CLOBBER HAZARD found and fixed (2026-07-27,
  commit
  `e26d13ff5`),
  before it fired.
  `draw-sample.ts` wrote the gate sheet to a
  fixed `grading-sheet.md`,
  which is the same file the user graded round one in,
  IN PLACE.
  24 of those 50 items carry free-text rationale
  (`rg --count-matches '^### \d+\. grade: [YN]\S'`),
  and `gate-verdict.md`
  preserves only the Y/N tally,
  not the reasoning that drove fixes A-F.
  So
  drawing round two would have destroyed the evidence base for round one's
  conclusions,
  through a routine command,
  with no prompt.
  Round one's graded sheet is archived at
  `node_modules/.monochromatic/translation-repair-runs/grading-sheet-round-one-graded.md`.
  Still OUTSIDE git and still never committable:
  it quotes UNLICENSED corpus.
  The fix is two independent defenses in `corpus-run/sheet-path.ts`,
  because
  either alone still loses data.
  Sheets are named after the draw seed
  (`grading-sheet-<seed>.md`),
  so two rounds cannot target one path;
  and a
  `--final` draw throws `GradedSheetExistsError` when its target exists,
  so a
  repeated draw inside ONE round cannot clobber grading already done.
  Preliminary sheets are deliberately exempt and stay redrawable as the pool
  grows.
  Round two therefore writes
  `grading-sheet-milestone-three-precision-round-two.md` and can never reach
  round one's file.
  Verified at the CLI boundary on a throwaway runs dir,
  not by reading the code:
  first `--final` draw writes,
  second refuses with the sheet intact,
  two
  preliminary draws both succeed.
  GENERAL LESSON,
  worth applying past this one file:
  an output path that is a
  CONSTANT is a hazard whenever a human writes into the artifact,
  because the
  file silently changes owner from the program to the person.
  Look for the same
  shape anywhere else a runner writes a fixed name a human then edits.
- RUNS DIR SITS INSIDE THE `rm -rf` BLAST RADIUS (found 2026-07-27,
  mitigated,
  root cause still open).
  `resolveRunsDir` defaults under
  `node_modules/.monochromatic/translation-repair-runs/`,
  and the repo ships
  `//:fix:reinstall`,
  whose body is literally
  `rmSync('node_modules', { recursive: true, force: true })` followed by
  `pnpm install`.
  One invocation of a task described only as "Clean reinstall to
  work around registry or resolution issues" destroys round one's graded sheet
  and its free-text rationale,
  `gate-verdict.md`,
  every settled artifact,
  the
  attempts map,
  and the slice cache.
  "Outside git" was the actual requirement;
  "inside node_modules" was never implied by it.
  MITIGATION IN PLACE:
  the whole runs dir (12 MB) is copied to
  `${HOME}/.local/share/monochromatic/translation-repair-runs-backup`,
  mode 700,
  outside the repo entirely.
  Refresh it after any batch of entries settles.
  It
  quotes UNLICENSED corpus,
  so it is never committable,
  wherever it lives.
  ROOT CAUSE DELIBERATELY NOT FIXED YET:
  changing the `resolveRunsDir` default
  mid-accumulation would point the next launch at an empty directory,
  which
  reads as zero settled entries and silently re-runs the entire corpus.
  Relocate
  only between runs,
  by moving the directory AND setting
  `TRANSLATION_REPAIR_RUNS_DIR` together,
  never by editing the default alone.
- `--final` IS NOW ONE-SHOT by design,
  so do not fire it early.
  The first
  `--final` draw freezes round two's sheet,
  and a later draw refuses even if
  more entries have settled since.
  Use the preliminary path for every validation
  draw and run `--final` exactly once,
  after accumulation is done.
  When the
  refusal appears,
  the correct response is to rename the existing sheet
  deliberately,
  never to reach for a force-shaped workaround:
  the refusal is the
  feature.
