//region Public index
// The names `@monochromatic-dev/translation-repair-typography` gives its siblings and its own tests, each from the module that declares it.
// Production siblings import it through `/ts`; tests and fixtures through the built entry, so a process loads one copy.

export {
  isCapitalLetter,
  isCasedLetter,
  isSmallLetter,
} from './cased-letters.ts';
export {
  childEnvironment,
  isCredentialName,
} from './child-process-environment.ts';
export {
  howOften,
  wordForCount,
} from './count-word.ts';
export {
  assertLandingInside,
  FragmentLandingError,
} from './fragment-landing-error.ts';
export {
  refuseFailedRun,
  startProgram,
} from './lint-program.ts';
export {
  locateQuote,
  type QuoteLocation,
} from './locate-quote.ts';
export {
  contextRoot,
  currentLogContext,
  inEntryLogContext,
  inSliceLogContext,
  type LogContext,
  sliceTagged,
} from './log-context.ts';
export { loneSurrogateIndex, } from './lone-surrogate.ts';
export {
  isMissingPathError,
  rethrowUnlessMissingPath,
} from './missing-path-error.ts';
export {
  bestPairsOf,
  distinctDepths,
  EMPTY_LINE,
  withMark,
  withQuotedNumber,
} from './nested-single-line-readings.ts';
export {
  nestedSinglePairs,
  opensBeforeMarkup,
} from './nested-single-quotes.ts';
export {
  bindsWord,
  closesSpan,
  isElisionPairEnd,
  isLeadingElision,
  precedesClosingMark,
  sequenceBefore,
} from './quote-neighbours.ts';
export {
  collapseLineBreaks,
  collapseSoftLineBreaks,
  normalizePunctuation,
  straightenProseQuotes,
  straightenQuotes,
} from './quote-normalize.ts';
export { readTextOrEmptyIfMissing, } from './read-text-if-present.ts';
export {
  isDefaultIgnorable,
  rendersAsNothing,
} from './renders-as-nothing.ts';
export {
  LINT_BINARY_NAME,
  PINNED_LINT_BINARY_VERSION,
} from './semantic-wrap-binary-constants.ts';
export {
  LintBinaryMissingError,
  LintBinaryVersionError,
  UNREADABLE_ANSWER,
  versionClause,
} from './semantic-wrap-binary-error.ts';
export { wrapWithLintBinary, } from './semantic-wrap-binary.ts';
export {
  countedCharacters,
  LintBinaryRunError,
  LintBinaryTextError,
} from './semantic-wrap-run-error.ts';
export { wrapReplacementText, } from './semantic-wrap.ts';
export { type TreeNode, } from './typography-link-text.ts';
export {
  noteRefusedMask,
  textNamed,
} from './typography-mask-refusal.ts';
export {
  fragmentProseMask,
  proseMaskBuilds,
  type ProseMaskReading,
  proseMaskReading,
} from './typography-prose-mask.ts';
export {
  shownViewOf,
  writtenBack,
} from './typography-shown-view.ts';

//endregion Public index
