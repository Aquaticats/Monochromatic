//region Public index
// The names `@monochromatic-dev/translation-repair-page-check` gives its siblings and its own tests, each from the module that declares it.
// Production siblings import it through `/ts`; tests and fixtures through the built entry, so a process loads one copy.

export {
  ArchiveBlockAlignmentMissingError,
  ArchiveBlockContextMissingError,
} from './corpus-run/archive-block-context-errors.ts';
export {
  archiveBlockIdentity,
  repairArchiveBlocks,
} from './corpus-run/archive-block-repair.ts';
export {
  keptNotesFinding,
  withBlockNotes,
} from './corpus-run/archive-block-notes.ts';
export { archiveBlockSourceContexts, } from './corpus-run/archive-block-source-context.ts';
export { restoreArchiveCasing, } from './corpus-run/archive-casing-restore.ts';
export {
  replacedDestinations,
  restoreArchiveDestinations,
} from './corpus-run/archive-destination-restore.ts';
export {
  archiveFrontMatterStands,
  frontMatterAuthorityOf,
} from './corpus-run/archive-front-matter.ts';
export { restoreArchiveItalicTitles, } from './corpus-run/archive-italic-title-restore.ts';
export { restoreArchiveNameCasing, } from './corpus-run/archive-name-casing.ts';
export {
  ArchiveOriginalCompletenessError,
  assertArchiveOriginalComplete,
} from './corpus-run/archive-original-completeness.ts';
export {
  type ArchiveRetainedLine,
  isStatusNoticeParagraph,
  isStubMarkerParagraph,
  type StrippedStubMarker,
  stripStubMarkersWithOrigins,
  STUB_MARKER_TOKENS,
} from './corpus-run/archive-stub.ts';
export { unwrapBlockquoteQuotes, } from './corpus-run/blockquote-quote-unify.ts';
export {
  canadianizePage,
  canadianizeText,
} from './corpus-run/canadian-forms.ts';
export { assertCarriedInsertionsRemain, } from './corpus-run/carried-insertion-completeness.ts';
export {
  assertDestinationsComplete,
  DroppedDestinationError,
  traceDroppedDestinations,
} from './corpus-run/destination-completeness.ts';
export {
  ARCHIVE_RENDERING_FINDING,
  BOTH_RENDERINGS_FINDING,
  type DestinationRenderingVerdict,
  judgeDestinationRenderings,
  sameAddress,
} from './corpus-run/destination-renderings.ts';
export { destinationsLine, } from './corpus-run/destinations-line.ts';
export {
  collectDestinations,
  type DestinationCheck,
  droppedDestinations,
  markdownDestinations,
  scanUrlRuns,
} from './corpus-run/dropped-destinations.ts';
export {
  type EmphasisSpan,
  emphasisSpans,
  oneLine,
} from './corpus-run/emphasis-spans.ts';
export { assertFinalNaturalnessComplete, } from './corpus-run/final-naturalness-completeness.ts';
export { finalSelectionFindings, } from './corpus-run/final-selection-completeness.ts';
export {
  assertFrontMatterComplete,
  FrontMatterCompletenessError,
} from './corpus-run/front-matter-completeness.ts';
export { restoreJsxAttributes, } from './corpus-run/jsx-attribute-restore.ts';
export {
  listItemExtent,
  restoreListSpread,
} from './corpus-run/list-spread-restore.ts';
export { restoreNameGlossLines, } from './corpus-run/name-gloss-restore.ts';
export {
  type AgreementSource,
  type EntryAgreement,
  type PageAgreement,
  pageAgreement,
} from './corpus-run/page-agreement.ts';
export { appendedNotes, } from './corpus-run/archive-note-carry.ts';
export { guardPageAssembly, } from './corpus-run/page-assembly-guard.ts';
export { publishInterimPage, } from './corpus-run/interim-page.ts';
export {
  freshlyTakenBack,
  type RoundTakeBacks,
} from './corpus-run/page-assembly-rounds.ts';
export { assertPageFootnotesIntact, } from './corpus-run/page-footnote-integrity.ts';
export {
  assertPageFootnotesResolve,
  PageFootnoteResolutionError,
} from './corpus-run/page-footnote-resolution.ts';
export {
  assertPageParses,
  type PageGrammarReading,
  readPageGrammar,
  UnparseablePageError,
} from './corpus-run/page-grammar.ts';
export {
  type CorpusPairReader,
  type CorpusPairText,
  type RepublishOutcome,
  type RepublishRow,
  republishSettledPages,
} from './corpus-run/page-republish.ts';
export {
  correctPinyinPage,
  correctPinyinTones,
} from './corpus-run/pinyin-tone.ts';
export { unfilledPageFindings, } from './corpus-run/publish-completeness.ts';
export {
  archiveKeptDefects,
  defectsLine,
  type PublishCheck,
  type PublishCheckStep,
  type PublishDefect,
  publishDefects,
} from './corpus-run/publish-defects.ts';
export {
  ENGLISH_PAGE_FILE,
  FIXED_TREE_DIR,
  fixedPagePath,
  PEOPLE_DIR,
  publishFixedPage,
} from './corpus-run/publish-fixed.ts';
export { shippableReplacements, } from './corpus-run/shippable-replacements.ts';
export {
  type MissingWording,
  pageCarriesEveryWording,
  type PageLengthCheck,
  pageWeighsWhatItShould,
  pageWeightRefutes,
  type PageWordingCheck,
  pairPublishedPages,
  type PublishedPairing,
  refusePageThatDisagrees,
} from './corpus-run/published-page-check.ts';
export {
  type PageDisagreement,
  PublishedPageDisagreesError,
} from './corpus-run/published-page-disagreement.ts';
export {
  publishedEntryIds,
  settledEntryIds,
  type VerifiableRun,
  whatThereIsToVerify,
} from './corpus-run/published-tree-listing.ts';
export {
  carriesText,
  carryingSource,
  markRuns,
  type TextCarryingReading,
  type SourceEncoding,
  textOnPage,
  UNTRANSLATED_PASSAGE_NOTE,
  withoutSourceRecordAt,
} from './corpus-run/source-ships.ts';
export {
  encodeAsLiteralMdx,
  leavesFlowOpen,
} from './corpus-run/source-ships-encoding.ts';
export {
  closedWith,
  closerOfWritten,
  NO_MARK,
} from './corpus-run/source-ships-open-flow.ts';
export {
  asTheParseReadsIt,
  blanking,
  foldLoneCarriageReturns,
} from './corpus-run/source-ships-readable.ts';
export {
  spansOf,
  type SourceSpans,
  type TextEdit,
  type TextRange,
} from './corpus-run/source-ships-spans.ts';
export {
  addressAsLink,
  type BareAddress,
  bareAddresses,
} from './corpus-run/source-ships-addresses.ts';
export {
  continuesDefinition,
  endLineOf,
  footnoteBodyStart,
  footnoteStartInside,
  hasFootnoteLabel,
  isFootnoteLabel,
  isIndentedCode,
  isLazyDefinition,
  NO_FOOTNOTE_INSIDE,
  opensFootnote,
  withoutQuoteMarks,
} from './corpus-run/source-ships-footnote.ts';
export {
  footnoteRead,
  type FootnoteRead,
} from './corpus-run/source-ships-footnote-spans.ts';
export { footnoteInContainerSpans, } from './corpus-run/source-ships-container.ts';
export {
  afterLiteralBlockEdits,
  separatingEdits,
} from './corpus-run/source-ships-separation.ts';
export {
  startsOffTables,
  withPageOnlyStarts,
} from './corpus-run/source-ships-footnote-ranges.ts';
export { runsUnfitForCompany, } from './corpus-run/source-ships-company.ts';
export { declaredNamesOf, } from './corpus-run/source-ships-declared.ts';
export { lazyUnderlinesBlanked, } from './corpus-run/source-ships-lazy.ts';
export { endsInLiteral, } from './corpus-run/source-ships-literal-end.ts';
export {
  footnoteStateOf,
  stateAfter,
  takeContinuation,
  takeFootnote,
  topLevelPending,
} from './corpus-run/source-ships-pending.ts';
export { readsAsDisplayMath, } from './corpus-run/source-ships-display-math.ts';
export {
  rawHtmlBlockReading,
  rawHtmlRegionsOf,
} from './corpus-run/source-ships-footnote-html-region.ts';
export { fenceLengthEdits, } from './corpus-run/source-ships-fenced.ts';
export {
  holdsIndentedFenceMark,
  indentedFenceMarkEnd,
} from './corpus-run/source-ships-fence.ts';
export {
  contentStretches,
  joinedLines,
  lineJoinsOf,
} from './corpus-run/source-ships-lines.ts';
export {
  autolinkAsText,
  autolinkWritten,
  escapedLabel,
  cutByTable,
  explicitLink,
  linkInsteadOfAutolink,
} from './corpus-run/source-ships-links.ts';
export {
  footnoteBodySpans,
  footnoteInParagraphSpans,
  nestedSpans,
  type SpansReader,
} from './corpus-run/source-ships-nested.ts';
export { withModuleLinesWritten, } from './corpus-run/source-ships-module-line.ts';
export {
  closingBracket,
  imageTextRanges,
  indicesOf,
  originalStretches,
  rangeOf,
  secondLabel,
  startsAny,
} from './corpus-run/source-ships-ranges.ts';
export {
  type ComposedPage,
  composeSourcePage,
  SourcePageCompositionError,
  type SourcePageStep,
} from './corpus-run/source-ships-compose.ts';
export {
  composeOverTiling,
  tilingOf,
} from './corpus-run/source-ships-tiling.ts';
export {
  droppedRunSets,
  runBodyOf,
  type GrammarReader,
  holdsOpenComment,
  namedSlices,
  runTextOf,
  slicesRefusedAsLiteral,
  sourceRunsOf,
} from './corpus-run/source-ships-runs.ts';
export { unifyQuoteStyle, } from './corpus-run/quote-style-unify.ts';
export {
  applySpanRewrites,
  type SpanRewrite,
} from './corpus-run/span-rewrites.ts';
export { readTags, } from './corpus-run/tag-attributes.ts';
export {
  UnansweredContestSliceError,
  type WouldShipDecider,
  type WouldShipReading,
  type WouldShipSilence,
  type WouldShipSlice,
  type WouldShipSource,
  wouldShipTextFor,
  wouldShipTextPerSlice,
} from './corpus-run/would-ship-text.ts';

export * from './index-reading.ts';

//endregion Public index
