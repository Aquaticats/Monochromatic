//region Public index
// The names `@monochromatic-dev/translation-repair-stage` gives its siblings and its own tests, each from the module that declares it.
// Production siblings import it through `/ts`; tests and fixtures through the built entry, so a process loads one copy.

export { downgradeCount, } from './downgrade-count.ts';
export { isIndexPairList, } from './index-pair-list.ts';
export {
  assertOverlap,
  failureWritten,
  mapOverlapped,
  markFailureWritten,
  type OverlappedRow,
  OverlapRefusedError,
} from './overlapped-map.ts';
export {
  agreePairs,
  type IndexPair,
  type PairAgreement,
  type PairingShape,
} from './pair-agreement.ts';
export {
  type BlockPairCounts,
  countPairedBlocks,
} from './pair-block-counts.ts';
export { blockPairingToSteps, } from './pair-blocks-steps.ts';
export {
  type BlockPair,
  BlockPairingError,
  type BlockPairingWire,
  buildBlockPairingMessages,
  type FreeOrderBlocks,
  isBlockPairingWire,
  type NumberedBlock,
  readBlockPairing,
  requireBlockPairingRefusal,
} from './pair-blocks-wire.ts';
export {
  crossingFinding,
  definitionIndexes,
  type DefinitionLabelPair,
  definitionLabelsOf,
  type SplitDefinitionPairs,
  splitDefinitionPairs,
} from './pair-definition-order.ts';
export {
  claimMediaAdjacentTargets,
  type MediaAdjacentClaim,
} from './pair-media-adjacency.ts';
export {
  isSectionPairingWire,
  readSectionPairing,
} from './pair-sections-read.ts';
export {
  type PairedDocumentRecord,
  pairSectionsWithRoster,
  SECTION_PAIRING_RESPONSE_FORMAT,
  type SectionPairingOutcome,
} from './pair-sections-stage.ts';
export { sectionPairingToSteps, } from './pair-sections-steps.ts';
export {
  buildSectionPairingMessages,
  type NumberedSection,
  requireSectionPairingRefusal,
  type SectionPair,
  SectionPairingError,
  type SectionPairingWire,
} from './pair-sections-wire.ts';
export {
  FENCE_CHARACTER,
  longestFenceRun,
  PROMPT_FENCE_MIN,
  selectFence,
} from './prompt-fence.ts';
export {
  type PromptPayloadStore,
  promptPayloadStore,
  PromptPayloadStoreError,
} from './prompt-payload-store.ts';
export {
  modelPromptDigest,
  promptUniqueClient,
} from './prompt-uniqueness-client.ts';
export {
  CUT_SHORT_RECOVERY_NUDGE,
  OFF_SHAPE_RECOVERY_NUDGE,
  RECOVERY_NUDGES,
  UNREADABLE_CAUSES,
  type UnreadableCause,
  unreadableCauseOf,
} from './recovery-nudge.ts';
export {
  type BlocklistVerdict,
  blocklistVerdictFor,
  ROSTER_BLOCKLIST,
  type RosterBlocklistEntry,
} from './roster-blocklist.ts';
export {
  assertRostersConfigured,
  RosterConfigurationError,
} from './roster-configuration.ts';
export { rosterQuorumSize, } from './roster-quorum-size.ts';
export {
  createSeatTally,
  RUN_SEATS,
  type SeatCount,
  type SeatOutcome,
  seatReportLines,
  type SeatTally,
  seatTallyClient,
} from './seat-tally.ts';
export {
  attemptStageCall,
  isRequestFault,
  type StageVoice,
} from './stage-call.ts';
export {
  attemptDecisionCall,
  type StageDecision,
} from './stage-decision-call.ts';
export {
  assertBenchStart,
  StageBenchStartError,
} from './stage-bench-start.ts';
export {
  askingWindow,
  benchRotation,
  FANOUT_SPARE,
  type FanOutMode,
  firstRoundWindow,
  rotatedBench,
} from './stage-fanout-window.ts';
export {
  type HeardVoice,
  type StageGather,
} from './stage-gather.ts';
export {
  gatherStageVoices,
  SEAT_WITHHOLD_ASKS,
  STAGE_RETRY_ROUNDS,
} from './stage-quorum.ts';
export {
  MIN_STAGE_VOICES,
  type ReachableQuorum,
  reachableQuorum,
  shortBenchStageFinding,
} from './stage-reachable-quorum.ts';
export {
  runRecoveryRound,
  type SharedRoundRequest,
} from './stage-recovery-round.ts';
export {
  repeatedRosterIds,
  StageRosterRepeatError,
} from './stage-roster-repeat.ts';
export {
  type StageRosterCoverage,
  summarizeStageRoster,
} from './stage-roster.ts';
export {
  roundLine,
  type RoundLineQuorum,
} from './stage-round-line.ts';
export { StragglerCutError, } from './straggler-cut.ts';
export {
  type RoundOutcome,
  runGatherRound,
  STRAGGLER_GRACE_MS,
} from './stage-round.ts';
export {
  readStageSettings,
  type StageSettings,
} from './stage-settings.ts';
export {
  refinementIsResumable,
  silentStageLabel,
  silentStagesOf,
  silentStagesOfRecord,
  silentStagesOfSettlement,
  STAGE_QUORUM_UNMET_PREFIX,
  stageQuorumUnmetFinding,
} from './stage-silence.ts';
export {
  endsTheRun,
  parksTheRun,
  isDryRefusal,
} from './stage-run-end.ts';
export {
  lostVoiceFindings,
  rosterShortfallFindings,
} from './stage-voice-lost.ts';
export { runWindowedRounds, } from './stage-windowed-rounds.ts';
export {
  reuseTwinOrBuy,
  type TwinMemo,
  type TwinOrBought,
  type TwinStored,
} from './twin-memo.ts';
export { UnpreparedSliceError, } from './unprepared-slice.ts';

//endregion Public index
