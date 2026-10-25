//region Public index
// The names `@monochromatic-dev/translation-repair-markup` gives its siblings and its own tests, each from the module that declares it.
// Production siblings import it through `/ts`; tests and fixtures through the built entry, so a process loads one copy.

export {
  isAsciiAlphanumeric,
  isAsciiDigit,
  isAsciiDigits,
  isAsciiLetter,
  isAsciiLowerLetter,
  isLowerHexDigit,
} from './ascii-letters.ts';
export {
  type ClosedMarkSpan,
  closedMarkSpans,
} from './closed-mark-spans.ts';
export { fencedLineFlags, } from './code-fence-lines.ts';
export {
  codePointAt,
  codePointBefore,
  codePointCount,
  codePointLength,
  compareCodePoints,
  textsInCodePointOrder,
  wholeOpening,
} from './code-points.ts';
export {
  OverlappingEditsError,
  spliceDisjointEdits,
  type SpliceEdit,
} from './disjoint-splice.ts';
export {
  errorName,
  failureName,
} from './error-name.ts';
export {
  carriesAsciiLetter,
  carriesHan,
  isHanCharacter,
  isHanOnly,
} from './han-only-text.ts';
export {
  bracketedTitles,
  LINE_END,
  LINK_MIDDLE,
  LINK_OPEN,
  TITLE_CLOSE,
  TITLE_OPEN,
  titleText,
} from './han-title-read.ts';
export {
  headingAffinity,
  latinTokens,
} from './heading-affinity.ts';
export { isSuccessStatus, } from './http-success.ts';
export {
  inlineContainerTags,
  tagEnd,
} from './inline-container-tags.ts';
export {
  type FoldedText,
  foldInvisibleVariants,
} from './invisible-variants.ts';
export { isIsoStampText, } from './iso-stamp-text.ts';
export {
  continuesLatinWord,
  continuesLatinWordOrCompatibilityForm,
  foldedLatinWords,
  foldLatinWord,
  isCombiningMark,
  isLatinCapital,
  isLatinLetter,
  isLatinLetterOrMark,
  isLatinWordCharacter,
  isLetterOrDigitOfAnyScript,
  isSpacedScriptWordCharacter,
  type LatinWord,
  latinWordSpans,
  lowerCaseLatinWords,
} from './latin-letters.ts';
export {
  contentWords,
  measureSeedRestoration,
  RESTORATION_WORD_THRESHOLD,
  type SeedRestoration,
} from './lexical-restoration.ts';
export {
  LifecycleAttemptError,
  lifecycleClockStamp,
  lifecycleStamp,
  wallClockNow,
} from './lifecycle-stamp.ts';
export {
  type MarkupDelta,
  markupDelta,
  markupSourceKeys,
  settleMarkupMoves,
} from './markup-atom-preservation.ts';
export {
  MARKUP_ATOM_SHEET_NAMES,
  MARKUP_IDENTIFIER_KINDS,
  type MarkupAtom,
  type MarkupAtomKind,
  scanMarkupAtoms,
} from './markup-atom-scan.ts';
export { treeNodes, } from './mdast-tree-nodes.ts';
export {
  isTagWhitespace,
  readAttributeName,
  readTagName,
} from './mdx-tag-name.ts';
export { opensMdxTag, } from './mdx-tag-start.ts';
export { monotonicMs, } from './monotonic-clock.ts';
export {
  firstNestingExcess,
  isStackOverflow,
  nestingAccount,
  NestingBoundError,
  nestingRefusalAccount,
} from './nesting-bound.ts';
export {
  certainFenceOf,
  fenceOf,
  NO_FENCE,
} from './nesting-fence.ts';
export {
  type HtmlBlock,
  htmlBlockAfter,
  htmlBlockAfterBlank,
} from './nesting-html-block.ts';
export {
  scanInline,
  type ScanState,
} from './nesting-inline-count.ts';
export {
  blanksEnd,
  containerPrefixOf,
  indentationOf,
  isRuleLine,
} from './nesting-line-lexing.ts';
export {
  DELIMITER_BOUND,
  NESTING_BOUND,
  type NestingExcess,
  type NestingReading,
  STACK_OVERFLOW_ACCOUNT,
  STACK_OVERFLOW_MESSAGE,
} from './nesting-vocabulary.ts';
export { NoProviderForModelError, } from './no-provider-for-model-error.ts';
export {
  clusterReadings,
  ReadingClusterError,
} from './reading-cluster.ts';
export {
  characterTrigrams,
  CORROBORATION_TRIGRAM_SHARE,
  type CorroborationVerdict,
  readingsCorroborate,
  trigramOverlap,
} from './reading-corroboration.ts';
export {
  latinWords,
  negatesSomething,
  readsAsRefusal,
  refusalReportsAbsence,
} from './reading-refusal.ts';
export { tableCell, } from './table-cell.ts';
export {
  isDecimalText,
  isNegativeWholeNumberText,
  isUnsignedNumberText,
  isWholeNumberText,
  WHOLE_NUMBER_RULE,
} from './whole-number-text.ts';
export {
  carriesWord,
  tokenStarts,
  type WordEnd,
  wordStarts,
} from './word-bounds.ts';

//endregion Public index
