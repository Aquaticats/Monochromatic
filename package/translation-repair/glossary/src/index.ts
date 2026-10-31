//region Public index
// The names `@monochromatic-dev/translation-repair-glossary` gives its siblings and its own tests, each from the module that declares it.
// Production siblings import it through `/ts`; tests and fixtures through the built entry, so a process loads one copy.

export {
  ACCURACY_CATEGORY_SCOPE,
  ADJUDICATION_CATEGORY_CHECK,
} from './accuracy-category-policy.ts';
export {
  COMMUNITY_GLOSSARY,
  communityRenderingDepartures,
  communityRenderingsBlock,
  type CommunityTerm,
  communityTermLines,
  communityTermsIn,
  type RenderingCandidate,
} from './community-glossary.ts';
export { readAskedCount, } from './corpus-run/asked-count.ts';
export { writeFileAtomic, } from './corpus-run/atomic-write.ts';
export {
  AttemptCountError,
  type AttemptMap,
  attemptsOf,
  countAttempt,
  readAttemptMap,
  writeAttemptMap,
} from './corpus-run/attempt-store.ts';
export {
  DECLINED_DIR,
  DeclinedEntriesUnreadableError,
  declinedEntryIds,
  type DeclinedEntryRecord,
  type DeclineReason,
  writeDeclinedEntry,
} from './corpus-run/declined-entries.ts';
export {
  type DigestGroup,
  groupByDigest,
} from './corpus-run/digest-group.ts';
export {
  INTERIM_DIR,
  type InterimEntryRecord,
  interimPageDigest,
  type InterimReason,
  readInterimEntries,
  removeInterimEntry,
  writeInterimEntry,
} from './corpus-run/interim-entries.ts';
export {
  directoryIdNameStands,
  namesDirectoryId,
} from './corpus-run/directory-id-name.ts';
export {
  type DirectoryReading,
  type EntryKind,
  filesystemReason,
  namesIn,
  namesOfKind,
  presentNamesOfKind,
  readingOf,
} from './corpus-run/directory-listing.ts';
export { textSettingOf, } from './corpus-run/env-text-setting.ts';
export {
  parseRunLogRecord,
  readRunLogLines,
  readRunLogText,
  renderRunLogLine,
  type RunLogRecord,
  runLogFilesUnder,
} from './corpus-run/run-log-read.ts';
export { resolveGit, } from './corpus-run/git-command.ts';
export {
  assertPipelineDigest,
  digestPipeline,
  isDigestShaped,
  PIPELINE_DIGEST_SHAPE,
  type PipelineDigest,
  PipelineDigestError,
  type PipelineStamp,
} from './corpus-run/pipeline-digest.ts';
export {
  inProse,
  proseBodyTree,
  type ProtectedRange,
  protectedRanges,
} from './corpus-run/prose-ranges.ts';
export {
  GradedSheetExistsError,
  resolveSheetPath,
  type SheetKind,
  UnsafeSeedError,
} from './corpus-run/sheet-path.ts';
export {
  assertSheetPairFree,
  writeSheetPair,
} from './corpus-run/sheet-write.ts';
export { sharesMedia, } from './corpus-run/transcription-suspect.ts';
export { FOREIGN_PHRASE_NAME_TITLE_SCOPE, } from './foreign-phrase-scope.ts';
export {
  foldForGlossary,
  formStarts,
  renderingSpans,
  textCarriesForm,
} from './glossary-match.ts';
export {
  adoptCalibrationGrace,
  CALIBRATION_STRAGGLER_GRACE_MS,
  type CalibrationGrace,
  graceOverrideNote,
  isTimerWindow,
  MAX_TIMER_DELAY_MS,
  readWindowDial,
  resolveStragglerGraceMs,
  STRAGGLER_GRACE_VAR,
} from './grace-override.ts';
export {
  carriesHandleToken,
  isHandleCharacter,
  standsAsHandle,
} from './handle-token.ts';
export {
  buildLicensedQuotes,
  buildRemovableQuotes,
} from './licensed-quotes.ts';
export { OBLIGATORY_DIFFERENCES, } from './obligatory-differences.ts';
export { type PictureReaderSeating, } from './picture-reader-seating.ts';
export {
  sourceOnlyBreakFindings,
  substituteBreakFindings,
} from './source-only-breaks.ts';
export { StatedRefusalError, } from './stated-refusal.ts';
export {
  blocksOrWholeText,
  droppedAddressFindings,
  withoutHtmlComments,
} from './translate-address-drop.ts';
export { latinTitleFindings, } from './translate-latin-title.ts';
export { strayClosingQuoteFindings, } from './translate-quote-balance.ts';
export {
  type PageGrammar,
  readPageSkeleton,
} from './translate-skeleton-page.ts';
export {
  blockDetail,
  type BlockShape,
  readSliceSkeleton,
  type SkeletonRead,
  type SliceSkeleton,
  walkAtoms,
} from './translate-skeleton.ts';
export { droppedSuicideFindings, } from './translate-suicide-drop.ts';
export {
  compareBlocks,
  describeBlocks,
  quotedPassageFloorFindings,
  sameShape,
} from './translate-validate-blocks.ts';
export {
  readWriterGrace,
  resolveWriterGraceMs,
  WRITER_GRACE_MS,
  WRITER_GRACE_VAR,
  WRITER_STAGE_LABELS,
  type WriterGrace,
  writerGraceOverrideNote,
} from './writer-grace-override.ts';

//endregion Public index
