//region Public index
// The names `@monochromatic-dev/translation-repair-corpus-source` gives its siblings and its own tests, each from the module that declares it.
// Production siblings import it through `/ts`; tests and fixtures through the built entry, so a process loads one copy.

export { widenFootnoteRelabel, } from './archive-footnote-relabel-widen.ts';
export {
  type FootnoteRelabel,
  footnoteRelabelOf,
  footnoteRelabelOfDefinitions,
  type FootnoteRelabelReading,
} from './archive-footnote-relabel.ts';
export {
  archiveNoteReadingsOf,
  type ArchiveOriginalReading,
  archiveOriginalReadingOf,
  type ArchiveOriginalSpan,
  type NoteReading,
  type NoteWithReading,
  readNote,
} from './archive-original-note.ts';
export {
  corpusGitEnvironment,
  gitSearchCeiling,
} from './corpus-git-context.ts';
export {
  type CorpusName,
  corpusNameLines,
  corpusNamesOf,
  readCorpusNames,
} from './corpus-name-index.ts';
export {
  CORPUS_COMMIT_SHA,
  type CorpusPin,
  CorpusReadError,
  type CorpusReadFailure,
  isMissingCorpusObject,
  listCorpusPeople,
  readCorpusBytes,
  readCorpusFile,
} from './corpus-source.ts';
export {
  CORPUS_REFERENCE_EXPANSION,
  documentBaseline,
  MIN_RATIO_SOURCE_CHARS,
  PLAUSIBLE_BASELINE_MAX,
  PLAUSIBLE_BASELINE_MIN,
  SliceBlockCountRefusalError,
  type SliceRatio,
  sliceRatios,
  type SliceSize,
  sliceSizeOf,
  sliceSizesOf,
} from './displacement-ratio.ts';
export {
  admitInsertions,
  type CarriedInsertion,
  type FoldedInsertion,
  type InsertionAdmission,
} from './insertion-admission.ts';
export {
  composedInsertion,
  documentLineEnding,
  fragmentBody,
  fragmentCut,
  type FragmentPlace,
} from './insertion-separator.ts';
export {
  assertPlacementLayout,
  type PlacementFault,
  PlacementLayoutError,
} from './placement-layout.ts';
export {
  assertPreparationIdentity,
  isPreparationIdentityShaped,
  PREPARATION_IDENTITY_SHAPE,
  type PreparationIdentity,
  preparationIdentity,
  PreparationIdentityError,
} from './preparation-identity.ts';
export {
  archiveOriginalSpansOf,
  chunkSealedTargets,
  declinedLessSealed,
  sealedFinding,
} from './preparation-seal.ts';
export {
  unclaimedFinding,
  unclaimedInAlignedPair,
  unclaimedOutsideAlignment,
} from './preparation-unclaimed.ts';
export {
  type FrontMatterAuthority,
  type PreparedDocumentPair,
  type UnclaimedTargetBlock,
} from './prepared-document-pair.ts';
export {
  type PreparedSheetFacts,
  preparedSheetFactsOf,
} from './prepared-sheet-facts.ts';
export {
  type BreakRendering,
  renderedBreakContract,
  renderedBreakFacts,
} from './rendered-break-prompt.ts';
export {
  BREAK,
  CLAUSE_BOUNDARY,
  MAXIMAL_REPEAT_MINIMUM,
  QUOTE_CLOSERS,
  QUOTE_OPENERS,
  REFRAIN_CLAUSE_MINIMUM,
  REFRAIN_PREFIX_MINIMUM,
  REFRAIN_SLOT_MAXIMUM,
  REFRAIN_SLOT_MINIMUM,
  REFRAIN_SUFFIX_MINIMUM,
  STRAIGHT_QUOTE,
  TAG_LETTERS,
  URL_SCHEMES,
} from './repeat-groups-bounds.ts';
export { compareFoundRepeats, } from './repeat-groups-order.ts';
export { disjointRepeatsOf, } from './repeat-groups-overlap.ts';
export {
  quotedCountsOf,
  touchesQuotation,
} from './repeat-groups-quotation.ts';
export {
  clausesOf,
  refrainFramesOf,
} from './repeat-groups-refrain.ts';
export { maximalRunsOf, } from './repeat-groups-runs.ts';
export {
  type MaximalRepeatGroup,
  type RefrainGroup,
  type RefrainOccurrence,
  type RepeatGroup,
  type RepeatOccurrence,
  type UnplacedOccurrence,
  type UnplacedReason,
} from './repeat-groups-types.ts';
export {
  blankedAt,
  cutEndAt,
  cutSetOf,
  hasMarkupOpener,
  hiddenCoverOf,
  hiddenRangesOf,
  wordingEndAt,
  wordingStreamOf,
} from './repeat-groups-wording.ts';
export { repeatGroupsOf, } from './repeat-groups.ts';

//endregion Public index
