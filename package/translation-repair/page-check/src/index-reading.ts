//region Public index of the reading comparison and the fresh tails
// The names the comparison of what a reader of the page sees with what a reader of the source sees, and the reading of the
// text after a footnote on its own, give their siblings and their own tests, split out of `index.ts` to keep that file
// under the line bound. The report of a page with no body (`pageWithoutBodyDefects`) sits here for the same reason.

export { pageWithoutBodyDefects, } from './corpus-run/publish-defects.ts';
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
export { boundOf, } from './corpus-run/source-reading-bound.ts';
export { cheapestPlacing, } from './corpus-run/source-reading-cheapest.ts';
export {
  type Lookahead,
  lookaheadsOf,
} from './corpus-run/source-reading-lookahead.ts';
export {
  type OverlapIndex,
  overlapIndexOf,
  reachesAny,
} from './corpus-run/source-reading-overlap.ts';
export {
  indexOfWords,
  matchesAt,
  placedAfter,
} from './corpus-run/source-reading-places.ts';
export { firstAtLeast, } from './corpus-run/source-reading-words.ts';
export { autolinkReading, } from './corpus-run/source-ships-autolink.ts';
export { joinBeforeFenceMark, } from './corpus-run/source-ships-fence-join.ts';
export {
  bodyStartsOnLabelLine,
  withoutLabelLineTable,
} from './corpus-run/source-ships-label-line-table.ts';
export {
  isIndentedCodeBlock,
  markerEscapeEdits,
} from './corpus-run/source-ships-list-marker.ts';
export { literalBlockReading, } from './corpus-run/source-ships-literal-block.ts';
export {
  charactersAt,
  endOfEdit,
} from './corpus-run/source-ships-nested-text.ts';
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
