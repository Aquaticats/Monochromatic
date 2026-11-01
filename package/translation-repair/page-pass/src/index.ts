//region Public index
// The names `@monochromatic-dev/translation-repair-page-pass` gives its siblings and its own tests, each from the module that declares it.
// Production siblings import it through `/ts`; tests and fixtures through the built entry, so a process loads one copy.

export { applyFootnoteRelabel, } from './apply-footnote-relabel.ts';
export {
  applyPatchOperations,
  EnvelopeOverlapError,
  type PatchOperation,
  type PatchOutcome,
  type PatchRejection,
  type PreservationMode,
} from './apply-patch.ts';
export {
  pageTextBySlice,
  pageTextOf,
  rowsChangedBy,
  SliceNotOnPageError,
  slicesInOrder,
  withRewrittenText,
} from './corpus-run/assembly-page-text.ts';
export {
  assertContributorNamesComplete,
  ContributorCompletenessError,
} from './corpus-run/contributor-completeness.ts';
export { nameAuthorities, } from './corpus-run/contributor-name-authorities.ts';
export { restoreContributorNames, } from './corpus-run/contributor-name-restore.ts';
export { placeHandleGlosses, } from './corpus-run/handle-gloss-place.ts';
export {
  carriesRendering,
  handleReading,
  withoutGloss,
} from './corpus-run/handle-reading.ts';
export { restoreCollidingHeadings, } from './corpus-run/heading-collision-restore.ts';
export {
  assertHeadingsStayDistinct,
  CollapsedHeadingError,
} from './corpus-run/heading-distinctness.ts';
export { unifyHeadingSeries, } from './corpus-run/heading-series-unify.ts';
export { headingTitles, } from './corpus-run/heading-title-lines.ts';
export {
  NO_NUMBER,
  type OrdinalStyle,
  readHanNumeral,
  readOrdinalStyle,
  renderOrdinal,
  styleKey,
} from './corpus-run/ordinal-style.ts';
export { unifyTitleReferences, } from './corpus-run/title-reference-unify.ts';
export { readFootnoteClosureInput, } from './footnote-closure-input.ts';
export { type RelabelClosure, } from './footnote-closure-model.ts';
export {
  type FootnoteLabelRewrite,
  type RetainedArchiveFootnoteLabel,
} from './footnote-label-rewrite.ts';
export {
  type FootnoteProtectedRange,
  footnoteProtectedRanges,
  footnoteRenameTouchesOriginal,
  overlapsFootnoteProtection,
} from './footnote-protected-ranges.ts';
export { retainedFootnoteLabels, } from './footnote-retained-labels.ts';
export {
  checkPreservation,
  type PreservationVerdict,
} from './preservation-check.ts';
export {
  contentTokens,
  isIdeograph,
  properNouns,
} from './preservation-tokens.ts';
export { restoreEllipsis, } from './restore-ellipsis.ts';
export { restoreTypography, } from './restore-typography.ts';
export {
  contentWrite,
  type PlacedSlice,
  placementsOf,
  type Write,
  type Written,
} from './splice-slice-places.ts';
export {
  placeSlices,
  type SliceReplacement,
  SliceSpliceError,
  spliceSlices,
} from './splice-slices.ts';

//endregion Public index
