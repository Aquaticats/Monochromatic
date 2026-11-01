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

//endregion Page guard reports
