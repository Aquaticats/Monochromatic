//region Page guard reports
// THE TWO PAGE GUARDS AND THE REFUSALS THEY RAISE, as one export of the package (B1181). Each guard is a step of
// `publishFixedPage` (`publish-page-guard-steps.ts`) that reports and never stops the entry; a caller that weighs a
// page itself asks the guard and catches its refusal.
export {
  assertCarriedInsertionsRemain,
  CarriedEvidenceLostError,
} from './carried-insertion-completeness.ts';
export {
  assertPageFootnotesIntact,
  PageFootnoteIntegrityError,
} from './page-footnote-integrity.ts';
export { pageGuardSteps, } from './publish-page-guard-steps.ts';
// THE TWO STOPS THAT NOW SHIP (B1207): a page the grammar refuses keeps the archive at the slices that break it, and a
// body without naturalness approval is a defect.
export { finalNaturalnessDefects, } from './final-naturalness-defects.ts';
// A REPUBLISH OVER A PAGE THAT STANDS refuses the fallback page instead of shipping it (B1260).
export {
  FallbackPageRefusedError,
  type FallbackPolicy,
  pageThatParses,
  type ParsingPage,
  refuseFallbackPage,
} from './page-grammar-fallback.ts';

//endregion Page guard reports
