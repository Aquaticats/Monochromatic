//region Public index
// The names `@monochromatic-dev/translation-repair-repair` gives its siblings and its own tests, each from the module that declares it.
// Production siblings import it through `/ts`; tests and fixtures through the built entry, so a process loads one copy.

export {
  type ChunkCriticPhase,
  runChunkCriticPhase,
} from './chunk-critic-phase.ts';
export {
  candidateConfirmedIssueIds,
  measurePatchedCandidate,
  selectCreditableIssues,
} from './chunk-measure.ts';
export {
  attachClaimFilers,
  type ClaimFilers,
  claimFilersOf,
  describeClaimFiling,
  describeIssueFiling,
  recordIssuesWithFilers,
} from './claim-filers.ts';
export { panelClause, } from './claim-panel-voters.ts';
export {
  runDerivabilityProbe,
  type SeedDerivability,
} from './derivability-probe.ts';
export {
  buildDerivabilityMessages,
  DERIVABILITY_RESPONSE_FORMAT,
  DERIVABILITY_VERDICTS,
  type DerivabilityPlan,
  type DerivabilityVerdict,
  isDerivabilityVerdict,
  resolveDerivabilityJudgment,
} from './derivability-wire.ts';
export {
  deriveOmissionSeeds,
  SENTENCE_TERMINATORS,
  splitSentences,
} from './derive-seeds.ts';
export {
  type SentenceRegion,
  sentenceRegionAround,
} from './sentence-region.ts';
export {
  buildEditorMessages,
  type EditorPromptPlan,
} from './edit-prompt.ts';
export {
  EDITOR_RESPONSE_FORMAT,
  type EditorEditResolution,
  type EditorEditWire,
  type EditorReportWire,
  isEditorReportWire,
  resolveEditorEdits,
} from './edit-wire.ts';
export {
  buildChunkCandidates,
  buildEditorCandidates,
  chunkCandidateOf,
  type ChunkCandidateSet,
  type EditorCandidateSet,
  pickFallbackCandidate,
} from './editor-candidates.ts';
export {
  type EditorProposals,
  gatherEditorProposals,
} from './editor-gather.ts';
export {
  applyCandidate,
  selectChunkPatch,
  selectPerEnvelope,
} from './editor-ensemble.ts';
export { collectEnvelopeProposals, } from './editor-proposals.ts';
export {
  type ChunkPatchSelection,
  type EditorCandidate,
  type EnvelopeSelection,
  NOBODY_WROTE_IT,
  type ShippedProducer,
} from './editor-selection-result.ts';
export {
  CHUNK_SELECTION_CRITERIA,
  CHUNK_SELECTION_TASK,
  ENVELOPE_SELECTION_CRITERIA,
  ENVELOPE_SELECTION_TASK,
} from './editor-selection-sheet.ts';
export {
  digitRuns,
  sharedNumber,
  unsupportedVariant,
} from './fidelity-alteration.ts';
export {
  alterSharedNumber,
  type DamageAttempt,
  deleteOneSentence,
  FIDELITY_DAMAGE_KINDS,
  type FidelityDamageKind,
  insertBorrowedSentence,
} from './fidelity-damage.ts';
export { spliceOutSentence, } from './fidelity-splice.ts';
export {
  neighbouringIncumbent,
  neighbouringSource,
  type SliceNeighbourContext,
  sliceNeighbourContexts,
} from './fidelity-window.ts';
export { frontMatterRepairOutcome, } from './front-matter-repair.ts';
export {
  type AtomGateVerdict,
  gateParagraphRewrite,
  type InspectionRejection,
  inspectParagraph,
  type ParagraphInspection,
} from './inspect-paragraph.ts';
export {
  appliedIssuesByEnvelope,
  collectIssueAuthors,
  collectRefinedAuthors,
} from './issue-authors.ts';
export { persistRefinePhaseSlice, } from './refine-phase-slice.ts';
export {
  type RefinePhaseResult,
  runRefinePhase,
} from './refine-phase.ts';
export { refineProbeVerdict, } from './refine-probe-verdict.ts';
export {
  buildRefineMessages,
  type RefinePromptPlan,
} from './refine-prompt.ts';
export { retainsResolvedIssues, } from './refine-recheck.ts';
export {
  buildRefineSelectionContext,
  objectingJudgesOf,
  type ObjectionGroup,
  objectionsHeading,
  type RefineSelectionContext,
  type RefineStageMode,
} from './refine-selection-context.ts';
export {
  REFINE_CACHE_VERSION,
  refineRunShape,
  refineSliceKey,
} from './refine-slice-key.ts';
export {
  type RefinedSliceOutcome,
  type RefinedSliceSettlement,
  settleRefinedSlice,
} from './refine-slice-settle.ts';
export {
  applyReply,
  inRosterOrder,
  resolveReply,
} from './refine-stage-replies.ts';
export {
  type RefineStageResult,
  runRefineStage,
} from './refine-stage.ts';
export {
  isRefineReportWire,
  REFINE_RESPONSE_FORMAT,
  type RefineReportWire,
  type RefineResolution,
  type RefineRewriteWire,
  resolveRefineRewrites,
} from './refine-wire.ts';
export {
  MIN_REPAIR_DISPATCH_BUDGET_MS,
  type RepairAttemptRecord,
  type RepairBenchmarkResult,
  runRepairBenchmark,
} from './repair-benchmark.ts';
export {
  cacheRefusalLine,
  cacheRefusalsOf,
} from './repair-cache-gate.ts';
export {
  assertCheckerBench,
  checkerBenchAtStage,
  standingSeating,
} from './repair-checker-reseat.ts';
export { chunkEvidence, } from './repair-chunk-evidence.ts';
export { proveRepairedChunk, } from './repair-chunk-proof.ts';
export { settleShippedPatch, } from './repair-chunk-settle.ts';
export {
  type ChunkVerdict,
  describeChunkSettlement,
  settleChunkFromChecks,
  settleChunkVerdict,
} from './repair-chunk-verdict.ts';
export { repairChunk, } from './repair-chunk.ts';
export {
  damageClaimLinesBySlice,
  type ProbedChunk,
} from './repair-damage-evidence.ts';
export {
  type CheckerStageResult,
  runCheckerStage,
} from './repair-edit-stages.ts';
export {
  type EditorStageResult,
  runEditorStage,
} from './repair-editor-stage.ts';
export { repairTranslation, } from './repair-entry.ts';
export {
  notApplicableFinding,
  notApplicableRepair,
} from './repair-not-applicable.ts';
export { refineSettledSlices, } from './repair-refine-step.ts';
export {
  computeRepairScorecard,
  type RepairScorecard,
} from './repair-scorecard.ts';
export { buyRepairSlice, } from './repair-slice-buy.ts';
export {
  repairRunShape,
  repairSliceKey,
  SLICE_CACHE_VERSION,
} from './repair-slice-key.ts';
export { foldStageFindings, } from './repair-stage-findings.ts';
export {
  type CriticStageResult,
  runCriticStage,
} from './repair-stages.ts';
export { repairPreparedDocument, } from './repair-translation.ts';
export { unchangedChunkOutcome, } from './repair-unchanged-outcome.ts';
export {
  stripWorseVotedEdits,
  type WorseStrip,
} from './repair-worse-strip.ts';
export {
  buildRestorationJudgeMessages,
  isRestorationJudgeWire,
  isRestorationVerdict,
  type JudgeReference,
  resolveRestorationJudgment,
  RESTORATION_JUDGE_RESPONSE_FORMAT,
  RESTORATION_JUDGE_VERDICTS,
  type RestorationJudgePlan,
  type RestorationJudgeWire,
  type RestorationJudgmentWire,
  type RestorationVerdict,
} from './restoration-judge-wire.ts';
export {
  runRestorationJudge,
  type SeedJudgment,
} from './restoration-judge.ts';

//endregion Public index
