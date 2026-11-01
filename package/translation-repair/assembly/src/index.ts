//region Public index
// The names `@monochromatic-dev/translation-repair-assembly` gives its siblings and its own tests, each from the module that declares it.
// Production siblings import it through `/ts`; tests and fixtures through the built entry, so a process loads one copy.

export {
  type AdjacentRepetition,
  adjacentRepetitionFindings,
  type AdjacentSliceText,
  findAdjacentRepetitions,
} from './assembly-adjacent-repetition.ts';
export {
  CONTAINER_HALF_WITHHELD_FINDING,
  withholdLoneContainerHalves,
} from './assembly-container-halves.ts';
export {
  type ContentSurvival,
  contentSurvivalFindings,
  distinctiveWords,
  measureContentSurvival,
} from './assembly-content-survival.ts';
export {
  type AssemblyContractFault,
  assemblySentence,
  type ChangeSetName,
} from './assembly-contract-fault.ts';
export { repetitionLogLines, } from './assembly-damage-log.ts';
export {
  type GuardedAssembly,
  guardFootnoteAssembly,
} from './assembly-integrity.ts';
export {
  AssemblyContractError,
  assertReplacementsChange,
  checkedChangeSets,
  deriveShippedIndices,
  type OrderedChangeSets,
  orderedChangeSets,
  shippedSliceTexts,
} from './assembly-invariant.ts';
export {
  cutDefinitionBlocks,
  definitionBlockCount,
  isDefinitionTrim,
  type TrimmedReplacements,
  trimOrphanDefinitions,
} from './assembly-orphan-trim.ts';
export {
  introducedFootnoteFindings,
  introducedStructuralRegressions,
} from './assembly-regressions.ts';
export {
  countSpan,
  type GrownSpan,
  grownSpans,
  indexWindows,
  type WindowIndex,
} from './assembly-repetition-span.ts';
export {
  countPhrases,
  findIntroducedRepetitions,
  type RepetitionFinding,
  repetitionFindings,
  whitespaceTokensOf,
} from './assembly-repetition.ts';
export {
  advancingStructuralWithdrawal,
  type AdvancingWithdrawal,
  sharedPrefixLength,
  sharedSuffixLength,
  singleStructuralWithdrawal,
} from './assembly-structural-withdrawal.ts';
export { assembleRepair, } from './repair-assemble.ts';
export {
  assertCheckerIndependence,
  assertCheckerQuorumReachable,
  assertJudgeableEditorRoster,
  assertJudgeableProducerRoster,
  CheckerIndependenceError,
  CheckerQuorumError,
  type ChunkRepairOutcome,
  ProducerRosterError,
  type RepairModels,
  type RepairSliceSeating,
} from './repair-contract.ts';
export { repairLaneWordings, } from './repair-lane-wordings.ts';
export {
  buildIssueRecords,
  type IssueProbeReading,
  REPAIR_DISPOSITIONS,
  type RepairDisposition,
  type RepairIssueRecord,
  SHIPPED_DISPOSITION,
} from './repair-record.ts';
export { repairReplacements, } from './repair-replacements.ts';
export {
  type RepairStatus,
  type RepairTranslationResult,
} from './repair-result.ts';
export {
  CHUNK_SCOPE_ENVELOPE,
  describeAdoptedRound,
  describeJudgedRound,
  describeRepairSlate,
  REPAIR_ROUND_STAGES,
  type RepairJudgedRound,
  type RepairRoundStage,
  type RepairSlateEntry,
} from './repair-round-record.ts';
export {
  EDITOR_ROUND_STAGES,
  REFINER_ROUND_STAGES,
  selectionRoundOf,
  selectionRoundsFor,
  SlatePositionsError,
} from './repair-selection-rounds.ts';
export { formatRepairSheet, } from './repair-sheet.ts';
export {
  assertUnheardKeptArchive,
  heardNobodyAbout,
  RepairUnheardError,
  type RepairVoiceRecord,
  type UnheardClaim,
} from './repair-unheard.ts';
export { wrapRepairOutcomes, } from './repair-wrap.ts';

//endregion Public index
