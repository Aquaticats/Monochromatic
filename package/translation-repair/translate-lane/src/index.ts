//region Public index
// The names `@monochromatic-dev/translation-repair-translate-lane` gives its siblings and its own tests, each from the module that declares it.
// Production siblings import it through `/ts`; tests and fixtures through the built entry, so a process loads one copy.

export { logAlignmentFindings, } from './alignment-findings-log.ts';
export {
  type DocumentLanesResult,
  runDocumentLanes,
} from './document-lanes.ts';
export { readDocumentPictures, } from './document-readings.ts';
export {
  buildSliceSelections,
  type SliceSelection,
} from './slice-selection.ts';
export {
  restoreTargetOnlyRun,
  splitTargetOnlyRun,
  type TargetOnlySplit,
} from './target-only-run.ts';
export { alignmentRefusals, } from './translate-alignment-refusals.ts';
export {
  alignmentRefusalFinding,
  assessSliceAlignment,
  MAX_INCUMBENT_TO_SOURCE_RATIO,
  MIN_PROTECTED_INCUMBENT,
  type SliceAlignmentAssessment,
} from './translate-alignment.ts';
export { assembleTranslation, } from './translate-assemble.ts';
export {
  TRANSLATE_SLICE_CACHE_VERSION,
  type TranslateDisposition,
  type TranslateDocumentResult,
  type TranslateModels,
  type TranslateSliceRecord,
  type TranslateSliceSeating,
  type UnfilledSlice,
} from './translate-document-contract.ts';
export { translateDocument, } from './translate-document.ts';
export { translateLaneWordings, } from './translate-lane-wordings.ts';
export {
  carriedElsewhere,
  notCorroborated,
} from './translate-slice-unbought.ts';
export {
  attemptTranslateSlice,
  type SliceAttempt,
} from './translate-slice-attempt.ts';
export {
  type TranslateSliceInput,
  translateSliceInput,
  type TranslateSliceStageInput,
} from './translate-slice-input.ts';
export {
  translateRunShape,
  translateSliceKey,
} from './translate-slice-key.ts';
export { settleTranslateSlice, } from './translate-slice.ts';
export {
  assertUnheardKeptIncumbent,
  heardNobody,
  TranslateUnheardError,
  unheardCacheDiscardFinding,
} from './translate-unheard.ts';
export { wrapTranslateRecords, } from './translate-wrap.ts';

//endregion Public index
