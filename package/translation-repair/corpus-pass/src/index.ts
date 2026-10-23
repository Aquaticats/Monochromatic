//region Public index
// The names `@monochromatic-dev/translation-repair-corpus-pass` gives its siblings and its own tests, each from the module that declares it.
// Production siblings import it through `/ts`; tests and fixtures through the built entry, so a process loads one copy.

export { nodeEntries, } from './build-entries.ts';
export { consolidateDocument, } from './consolidate-driver.ts';
export { assertPassResumable, } from './corpus-run/corpus-pass-guards.ts';
export { type CorpusPassInput, } from './corpus-run/corpus-pass-input.ts';
export {
  HARD_CAP_MINUTES,
  PASS_MS_PER_MINUTE,
  PLAN_PREVIEW_COUNT,
  SOFT_BUDGET_MS,
} from './corpus-run/corpus-pass-limits.ts';
export {
  passDoneLine,
  passIncompleteLine,
  passLaunchLines,
  passOnlyLines,
  passPlanLine,
  passProvidersOffLines,
  passRequiredLines,
  passStartLine,
} from './corpus-run/corpus-pass-lines.ts';
export { orderPendingEntries, } from './corpus-run/corpus-pass-order.ts';
export { runPassQueue, } from './corpus-run/corpus-pass-queue.ts';
export {
  createPassDryStop,
  type PassDryStop,
  recordOnPark,
} from './corpus-run/pass-dry-stop.ts';
export { runCorpusPassOver, } from './corpus-run/corpus-pass-run.ts';
export { selectPendingEntries, } from './corpus-run/corpus-pass-select.ts';
export {
  passArchiveText,
  passArchiveWithOrigins,
} from './corpus-run/pass-archive.ts';
export { attestPassReferences, } from './corpus-run/pass-attest-references.ts';
export { foldPassCarried, } from './corpus-run/pass-carried-fold.ts';
export * from './index-consistency.ts';
export {
  type ConsolidationHooks,
  consolidationHooksFor,
} from './corpus-run/pass-consolidate-reseat.ts';
export { runPassConsolidation, } from './corpus-run/pass-consolidate.ts';
export {
  type ContestHooks,
  contestHooksFor,
} from './corpus-run/pass-contest-reseat.ts';
export { runPassContest, } from './corpus-run/pass-contest.ts';
export { withEntryBound, } from './corpus-run/pass-entry-bound.ts';
export { persistSettledEntry, } from './corpus-run/pass-entry-persist.ts';
export { settleEntry, } from './corpus-run/pass-entry-settle.ts';
export {
  relabelArchiveFootnotes,
  type RelabelledArchive,
} from './corpus-run/pass-footnote-relabel.ts';
export { decidePassInsertionAdmission, } from './corpus-run/pass-insertion-admission.ts';
export {
  admitPassInsertions,
  type InsertionHooks,
  insertionHooksFor,
} from './corpus-run/pass-insertion-reseat.ts';
export {
  type LanesSeating,
  runPassLanes,
} from './corpus-run/pass-lanes.ts';
export {
  outsideReadsFrom,
  type PassCorpusNameReader,
  type PassOutsideReads,
  type PassReferenceReader,
  type PassWorkTitleReader,
} from './corpus-run/pass-outside-reads.ts';
export {
  entryOverlap,
  readPassOverlap,
} from './corpus-run/pass-overlap.ts';
export { settledPageArtifact, } from './corpus-run/pass-page-assembly.ts';
export { assertPageGuards, } from './corpus-run/pass-page-guards.ts';
export {
  openPageTitleCache,
  pageTitleKey,
  type PageTitleLexiconRecord,
  type PassPageTitles,
  passPageTitles,
} from './corpus-run/pass-page-titles.ts';
export {
  type PicturesHooks,
  picturesHooksFor,
} from './corpus-run/pass-pictures-reseat.ts';
export {
  type PreparationHooks,
  preparationHooksFor,
  runPassPreparation,
} from './corpus-run/pass-prepare-reseat.ts';
export { preparePassEntry, } from './corpus-run/pass-prepare.ts';
export {
  type Reseated,
  reseatHookFor,
} from './corpus-run/pass-reseat-hook.ts';
export {
  type LanesHooks,
  lanesHooksFor,
  readLanesSeats,
  translateReseatFor,
} from './corpus-run/pass-reseat.ts';
export { createPassPictureReader, } from './corpus-run/pass-seated-pictures.ts';
export {
  type PassSettlement,
  runPassSettlement,
} from './corpus-run/pass-settlement.ts';
export { type PassPictureSources, } from './corpus-run/pass-visual-evidence.ts';
export { contestDocumentLanes, } from './lane-contest-driver.ts';

//endregion Public index
