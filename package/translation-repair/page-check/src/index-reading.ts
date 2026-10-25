//region Public index of the reading comparison
// The names the comparison of what a reader of the page sees with what a reader of the source sees gives its siblings and
// its own tests, split out of `index.ts` to keep that file under the line bound.

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

//endregion Public index of the reading comparison
