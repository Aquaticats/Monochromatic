//region Public index
// The names `@monochromatic-dev/translation-repair-slice-cache` gives its siblings and its own tests, each from the module that declares it.
// Production siblings import it through `/ts`; tests and fixtures through the built entry, so a process loads one copy.

export { nodeEntries, } from './build-entries.ts';
export {
  CacheAccountLogError,
  type SettingCandidate,
  settingCommit,
  type SettingReading,
  type SourceCommit,
  sourceCommitOf,
  type UnaccountedCommit,
  unaccountedCommits,
  utcMinutes,
  type VersionSetting,
} from './corpus-run/cache-account-commits.ts';
export {
  cacheAccountCommitsOf,
  cacheAccountGitOutput,
} from './corpus-run/cache-account-git.ts';
export { printCacheAudit, } from './corpus-run/cache-account-print.ts';
export {
  CacheAccountReadError,
  type CacheVersion,
  cacheVersionsIn,
  CITED_HASH_LENGTH,
  citedHash,
  declarationLineCounts,
} from './corpus-run/cache-account-read.ts';
export { auditCacheAccounts, } from './corpus-run/cache-account-run.ts';
export { cacheVersionSetting, } from './corpus-run/cache-account-setting.ts';
export {
  type AuditedSources,
  auditedSourcesOf,
  preSplitSourcesOf,
  preSplitWithoutTestsOf,
  repositoryTopOf,
  sourcesOfPrefix,
  sourcesPathspecOf,
  withoutTestsPathspecOf,
} from './corpus-run/cache-account-sources.ts';
export { reportSliceCaches, } from './corpus-run/cache-account-slice-report.ts';
export {
  type NewestSliceRecord,
  type RunsDirSearch,
  runsDirsIn,
  runsDirsUnder,
  type SliceCacheAccount,
  sliceCacheAccount,
  type SliceRecord,
  type UnlistedDir,
} from './corpus-run/cache-account-slices.ts';
export {
  describeSpread,
  percentileOf,
  REPORTED_PERCENTILES,
} from './corpus-run/census-spread.ts';
export { openConsolidateCache, } from './corpus-run/consolidate-cache-store.ts';
export { runAttemptQueue, } from './corpus-run/entry-attempt-queue.ts';
export {
  type EntryErrorOutcome,
  entryErrorOutcome,
  ReturnTimeNotUtcError,
  tallyCaughtEntry,
} from './corpus-run/entry-error-outcome.ts';
export { gatherEntryPictures, } from './corpus-run/entry-pictures.ts';
export {
  countCachedSlices,
  readAttemptOutcome,
  type ReattemptVerdict,
} from './corpus-run/entry-reattempt.ts';
export { openLaneContestCache, } from './corpus-run/lane-contest-cache-store.ts';
export {
  entryArchiveOriginalOf,
  recordEntryDecline,
  removeDeclinedPages,
} from './corpus-run/pass-decline.ts';
export {
  collectEligiblePairs,
  type IncompleteEntry,
  type PassEligibility,
} from './corpus-run/pass-eligibility.ts';
export {
  type CorpusPair,
  type EntryOutcome,
} from './corpus-run/pass-entry-contract.ts';
export {
  entriesFinishedThisRun,
  finishedEntryIds,
} from './corpus-run/pass-finished.ts';
export {
  assertArtifactsPlaceable,
  assertBuildGenerationResumable,
  GenerationDriftError,
  LegacyPipelineError,
  readDriftOptIn,
  UnplaceableArtifactError,
} from './corpus-run/pass-generation-guard.ts';
export { republishRunPages, } from './corpus-run/pass-republish.ts';
export {
  censusBySchema,
  type SchemaCensusRow,
  type SchemaClassification,
} from './corpus-run/pass-schema-census.ts';
export {
  assertResumableSchemaGeneration,
  MislabelledArtifactError,
  SchemaGenerationError,
} from './corpus-run/pass-schema-guard.ts';
export { artifactBackedIds, } from './corpus-run/pass-settled.ts';
export { stopBeforeNextEntry, } from './corpus-run/pass-stop-before-next.ts';
export { openPictureReadingCache, } from './corpus-run/reading-cache-store.ts';
export {
  carveSettled,
  listSettledEntryIds,
  readSettledRecipe,
  recipeLabel,
  type SettledCarve,
  type SettledRecipe,
} from './corpus-run/settled-carve.ts';
export { settledTallyLine, } from './corpus-run/settled-tally.ts';
export {
  CONSOLIDATE_NAMESPACE,
  EVERY_SLICE_NAMESPACE,
  PAGE_CONSISTENCY_NAMESPACE,
  PAGE_TITLES_NAMESPACE,
  PICTURE_READING_NAMESPACE,
  REPAIR_SLICE_NAMESPACE,
  type SliceNamespace,
  TRANSLATE_SLICE_NAMESPACE,
} from './corpus-run/slice-cache-claims.ts';
export {
  belongsToNamespace,
  discardNamespace,
  isSliceFileName,
  loadNamespacedSlices,
  openNamespacedCache,
  sliceFileName,
} from './corpus-run/slice-cache-namespace.ts';
export {
  discardSliceCache,
  listResumableEntries,
  openPairingCache,
  openRefineSliceCache,
  openSectionPairingCache,
  openSliceCache,
  openTranslateSliceCache,
} from './corpus-run/slice-cache-store.ts';
export { sliceCensusCarveLine, } from './corpus-run/slice-census-carve.ts';
export {
  censusEntry,
  type EntryCensus,
} from './corpus-run/slice-census-entry.ts';
export {
  gatherSliceCensus,
  type SliceCensusGathered,
  type SliceCensusRecipeReading,
} from './corpus-run/slice-census-gather.ts';
export { reportSliceCensus, } from './corpus-run/slice-census-report.ts';
export { sliceCensusTargetOnlyLines, } from './corpus-run/slice-census-target-only.ts';
export { sliceCensusTotal, } from './corpus-run/slice-census-total.ts';
export { sliceCensusUnpairedLines, } from './corpus-run/slice-census-unpaired.ts';
export { sliceCensusWidestLines, } from './corpus-run/slice-census-widest.ts';
export {
  bucketSliceCostsBySize,
  type CostBucket,
  printSliceCostBucket,
} from './corpus-run/slice-cost-bands.ts';
export { printSliceCostLanes, } from './corpus-run/slice-cost-lanes.ts';
export { reportSliceCost, } from './corpus-run/slice-cost-report-run.ts';
export { printSliceCostSpread, } from './corpus-run/slice-cost-spread.ts';
export {
  CALIBRATION_OVERLAP,
  type OverlapSetting,
  type OverlapSettingSource,
  readOverlap,
  readOverlapSetting,
} from './corpus-run/slice-overlap.ts';
export {
  cappedFailureText,
  TALLY_ERROR_CAP,
  tallyErrorText,
} from './corpus-run/tally-error-text.ts';
export {
  assertVisualEvidenceComplete,
  VisualEvidenceInterruptedError,
} from './corpus-run/visual-evidence-completeness.ts';

//endregion Public index
