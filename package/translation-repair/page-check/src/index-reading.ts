//region Public index of the reading comparison and the fresh tails
// The names the comparison of what a reader of the page sees with what a reader of the source sees, and the reading of the
// text after a footnote on its own, give their siblings and their own tests, split out of `index.ts` to keep that file
// under the line bound.

export {
  blockOpenerStart,
  openerStarts,
  rangesWithoutOpeners,
} from './corpus-run/source-ships-open-comment.ts';
export { rootWithFreshTails, } from './corpus-run/source-ships-fresh-tails.ts';
export { differenceDefectsOf, } from './corpus-run/source-ships-differences.ts';
export {
  PageSyntaxError,
  readerOfPage,
  readerOfSource,
  withoutLinkTargets,
} from './corpus-run/source-reading-account.ts';
export {
  type ReadingDifference,
  readingDifferenceOf,
} from './corpus-run/source-reading-difference.ts';

//endregion Public index of the reading comparison and the fresh tails
