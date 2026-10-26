//region Public index of the reading comparison and the fresh tails
// The names the comparison of what a reader of the page sees with what a reader of the source sees, and the reading of the
// text after a footnote on its own, give their siblings and their own tests, split out of `index.ts` to keep that file
// under the line bound.

export { blockOpenerStarts, } from './corpus-run/source-ships-open-block.ts';
export {
  openerStarts,
  rangesWithoutOpeners,
} from './corpus-run/source-ships-open-comment.ts';
export {
  isIndentedFourOrMore,
  PARAGRAPH_LEAD,
  spansWithoutLead,
} from './corpus-run/source-ships-paragraph-lead.ts';
export { rootWithFreshTails, } from './corpus-run/source-ships-fresh-tails.ts';
export { differenceDefectsOf, } from './corpus-run/source-ships-differences.ts';
export {
  blocksOfPage,
  blocksOfSource,
  PageSyntaxError,
  readerOfPage,
  readerOfSource,
  type SourceBlock,
  withoutLinkTargets,
} from './corpus-run/source-reading-account.ts';
export {
  type BlockDifference,
  blockDifferenceOf,
} from './corpus-run/source-reading-blocks.ts';
export {
  indexOfWords,
  matchesAt,
  placedAfter,
} from './corpus-run/source-reading-places.ts';
export { decodedTarget, } from './corpus-run/source-reading-target.ts';
export { footnoteOpensRawHtml, } from './corpus-run/source-ships-footnote-html.ts';
export {
  footnoteContainerPrefixOf,
  continuesAsParagraph,
  continuesTheFootnote,
  goesOnAsLazyText,
  isLazyText,
  opensFootnoteBody,
  withoutContainerAndIndentation,
} from './corpus-run/source-ships-footnote-container.ts';
export {
  footnoteReadingsOf,
  NO_PAGE_ONLY_DEFINITION,
  pageOnlyDefinitionWritten,
} from './corpus-run/source-ships-footnote-ranges.ts';
export { splitFlow, } from './corpus-run/source-ships-opener-flow.ts';
export { openersInFootnoteFlowsOf, } from './corpus-run/source-ships-opener-footnote.ts';
export { htmlInterruptionEdits, } from './corpus-run/source-ships-html-interrupt.ts';
export {
  endingAt,
  writtenAsText,
} from './corpus-run/source-ships-opener-text.ts';
export {
  type ReadingComparison,
  type ReadingDifference,
  readingDifferenceOf,
} from './corpus-run/source-reading-difference.ts';

//endregion Public index of the reading comparison and the fresh tails
