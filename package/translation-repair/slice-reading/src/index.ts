//region Public index
// The names `@monochromatic-dev/translation-repair-slice-reading` gives its siblings and its own tests, each from the module that declares it.
// Production siblings import it through `/ts`; tests and fixtures through the built entry, so a process loads one copy.

export {
  type BlockPairingProtocol,
  blockPairingProtocol,
} from './block-pairing-protocol.ts';
export { blockPairingQuestionKey, } from './block-pairing-question-key.ts';
export {
  type BlockPairingQuestion,
  blockPairingQuestion,
} from './block-pairing-question.ts';
export {
  alignDocumentSections,
  type AlignmentAttachment,
  type AlignmentFinding,
  chunkByHeadings,
  chunkLabel,
  type ChunkPair,
  describeAlignmentAttachment,
  type SectionAlignment,
  type SliceSyntax,
} from './chunk-document.ts';
export {
  CitedReferenceFetchError,
  EXA_CONTENTS_URL,
  fetchCitedReference,
  fetchedOf,
  type FetchedReference,
  REFERENCE_TEXT_CHARACTERS,
} from './cited-reference-fetch.ts';
export {
  CITED_REFERENCE_AUDIT_RULE,
  CITED_REFERENCE_CANDIDATE_RULE,
  CITED_REFERENCE_RULE,
  citedReferenceBlockText,
  citedReferenceCandidateLines,
  citedReferenceEvidence,
} from './cited-reference-rule.ts';
export { placeClosingPunctuation, } from './closing-punctuation.ts';
export {
  bothHalvesInserted,
  type ContainerHalf,
  type ContainerHalfPair,
  containerHalfPairs,
} from './container-half-pairs.ts';
export {
  assertContainerIntegrity,
  ContainerIntegrityError,
} from './container-integrity.ts';
export {
  declinedTargetBlocks,
  declinedTargetIds,
  declinedTargetIdsOfPairing,
} from './declined-target-runs.ts';
export { frontMatterCommentAuthorityFindings, } from './front-matter-comment-authority.ts';
export {
  frontMatterSlice,
  frontMatterSliceIndexes,
  type FrontMatterSliceResult,
} from './front-matter-slice.ts';
export {
  type AgreementTally,
  parsePreGrades,
  type PrecisionTally,
  scoreGradeAgreement,
  scoreGradedPrecision,
} from './grade-agreement.ts';
export {
  type GradedItem,
  type GradeVerdict,
  parseGradedSheet,
} from './grade-sheet-read.ts';
export {
  EmptyImageAssetError,
  type EncodedAsset,
  encodeImageAsset,
  extensionOf,
} from './image-asset.ts';
export {
  ocrReaderOver,
  type OcrReading,
  type ProgramRunner,
  readImageWithOcr,
  runInstalledProgram,
} from './image-ocr.ts';
export {
  containFailedReaders,
  type LabelledReading,
} from './image-reading-contain.ts';
export { imageReadingKey, } from './image-reading-key.ts';
export {
  isResumableReading,
  type ModelReading,
  type OcrReader,
  type PairedReading,
  readImagePair,
} from './image-reading-pair.ts';
export {
  readPastRefusal,
  REFUSAL_ASK_LIMIT,
} from './image-reading-past-refusal.ts';
export {
  MIN_READING_CHARS,
  readingMakesSense,
  type ReadingVerdict,
  solidCharacters,
} from './image-reading-sense.ts';
export {
  IMAGE_READING_PERSPECTIVES,
  type ImageReading,
  type ImageReadingPerspective,
  imageReadingText,
  isTransientReadingReason,
  readImageAsset,
  READING_INSTRUCTION,
} from './image-reading-stage.ts';
export { foldCarriageReturns, } from './line-endings.ts';
export { lineStartsOf, } from './line-starts.ts';
export { buildEditorAddendum, } from './line-structure-addendum.ts';
export {
  type ChunkGovernance,
  type ChunkSlice,
  governedSliceIndices,
} from './line-structure-inherit.ts';
export { isLineStructured, } from './line-structure.ts';
export {
  LocalProgramFailedError,
  localProgramFailureOf,
  readerFailureText,
} from './local-program-failure.ts';
export { mostCarriedReading, } from './most-carried-reading.ts';
export {
  PairingEvidenceError,
  type PairingEvidenceFailure,
} from './pair-blocks-evidence-identity.ts';
export { readBlockPairingOutcomes, } from './pair-blocks-read-outcomes.ts';
export {
  type BlockPairingOutcome,
  pairBlocksWithRoster,
  type PairedSectionRecord,
} from './pair-blocks-stage.ts';
export { pairingPictureContext, } from './pairing-pictures.ts';
export {
  type PairingQuestion,
  pairingQuestionKey,
} from './pairing-question-key.ts';
export { queriedBlockPairingDetails, } from './queried-block-pairing-details.ts';
export {
  type SectionBlockPairing,
  sectionPairingsOf,
} from './section-pairing.ts';
export {
  assertSliceCoverage,
  type BlockPlacementFault,
  blockPlacementSentence,
  coverageSentence,
  SliceCoverageError,
  type SliceCoverageFault,
} from './slice-coverage.ts';
export {
  assertSliceIndexing,
  indexingSentence,
  reindexSlicePair,
  SliceIndexingError,
  type SliceIndexingFault,
} from './slice-indexing.ts';
export {
  SIDE_LEGEND,
  slicePictureContexts,
  slicePictureNames,
  type SlicePictures,
  slicePictures,
} from './slice-pictures.ts';
export {
  assertSpanContiguity,
  SpanContiguityError,
} from './span-contiguity.ts';
export {
  leadingLineEndings,
  matchSpanEdges,
} from './span-edge-match.ts';
export {
  type StrictParseReading,
  strictRefusalOffset,
} from './strict-refusal-offset.ts';
export {
  opensWithVerdict,
  trimLeadingDelimiters,
  VERDICT_DELIMITERS,
} from './verdict-letter.ts';

//endregion Public index
