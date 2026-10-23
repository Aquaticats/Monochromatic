//region Public index
// The names `@monochromatic-dev/translation-repair-translate-stage` gives its siblings and its own tests, each from the module that declares it.
// Production siblings import it through `/ts`; tests and fixtures through the built entry, so a process loads one copy.

export { selectionFanOut, } from './candidate-select-fanout.ts';
export {
  assertDistinctJudges,
  stakesByIndexOf,
} from './candidate-select-roster.ts';
export {
  CONDEMNED_FALLBACK_UNDER_MINIMUM_FINDING,
  condemnedFallbackLoses,
  MIN_SELECTION_BALLOTS,
  type SelectionMinimum,
  selectionMinimum,
  shortBenchFinding,
} from './candidate-select-minimum.ts';
export { selectBestCandidate, } from './candidate-select-record.ts';
export { decideBestCandidate, } from './candidate-select.ts';
export {
  type PresentTenseCandidate,
  presentTenseCandidates,
  type PresentTenseRule,
} from './present-tense-candidates.ts';
export {
  maskQuotedSpeech,
  type SentenceSpan,
  sentenceSpans,
  visibleWords,
} from './present-tense-text.ts';
export {
  isBlankSpan,
  isSEndingWord,
  isWhitespace,
  namesAt,
  wordAfterSpace,
  wordsOf,
  type WordToken,
} from './present-tense-words.ts';
export {
  type TenseCandidateGroup,
  tenseCandidateGroups,
  tenseCandidateLines,
  type TenseCandidateSource,
} from './tense-candidate-block.ts';
export {
  ARCHIVE_TENSE_LABEL,
  CONSOLIDATED_TENSE_LABEL,
  CONTEST_TENSE_LABELS,
  REPAIR_TENSE_LABEL,
  shippedTenseLabelOf,
  STANDING_TENSE_LABEL,
  TRANSLATE_TENSE_LABEL,
} from './tense-candidate-labels.ts';
export { floorTranslateVoices, } from './translate-floor.ts';
export { judgeTranslateSlate, } from './translate-judge.ts';
export {
  type ProducedSlate,
  produceTranslateSlate,
} from './translate-produce.ts';
export {
  judgeSlateWithRetry,
  runoffTiedFinalists,
} from './translate-retry.ts';
export {
  type Runoff,
  runoffFinalists,
} from './translate-runoff.ts';
export { translateSlateEvidence, } from './translate-slate-evidence.ts';
export {
  type TranslateDecision,
  type TranslateStageResult,
} from './translate-stage-result.ts';
export { runTranslateStage, } from './translate-stage.ts';
export {
  settleUnflooredSlice,
  unflooredFinding,
} from './translate-unfloored.ts';
export {
  translatedSlateBreakEvidence,
  translatedSlateCriteria,
} from './translated-slate-criteria.ts';
export { TranslationRepairInterruptedError, } from './translation-repair-interrupted-error.ts';

//endregion Public index
