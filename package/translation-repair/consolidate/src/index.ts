//region Public index
// The names `@monochromatic-dev/translation-repair-consolidate` gives its siblings and its own tests, each from the module that declares it.
// Production siblings import it through `/ts`; tests and fixtures through the built entry, so a process loads one copy.

export { renderConsolidationBrief, } from './consolidate-brief.ts';
export {
  CONSOLIDATE_GATE_QUORUM,
  CONSOLIDATE_GATE_RESPONSE_FORMAT,
  type ConsolidateGateOutcome,
  gateConsolidatedSlice,
  type GateShipped,
  settleGateBallots,
} from './consolidate-gate-stage.ts';
export {
  buildConsolidateGateMessages,
  CONSOLIDATE_GATE_NAMES,
  type ConsolidateGateSubject,
  type GateBallot,
  type GateChoice,
  type GateWire,
  isConsolidateGateWire,
  isGateChoice,
  readConsolidateGateBallot,
} from './consolidate-gate-wire.ts';
export {
  describeStandingVerdict,
  FLAWED_STANDING_GATE_SHIPS_PROPOSAL_FINDING,
  GATE_PREFERRED_INELIGIBLE_STANDING_FINDING,
  INELIGIBLE_STANDING_REPLACED_FINDING,
  INELIGIBLE_STANDING_WITHHELD_FINDING,
  keepTheArchive,
  NO_VALID_WORDING_FINDING,
  nothingValidShips,
  slateIncumbentFor,
  UNDECIDED_GATE_SHIPS_PROPOSAL_FINDING,
} from './consolidate-ineligible-standing.ts';
export {
  consolidationWorthResuming,
  persistConsolidationSettlement,
} from './consolidate-persistence.ts';
export {
  produceConsolidations,
  type ProducedConsolidations,
} from './consolidate-produce.ts';
export { type ProposalVerdict, } from './consolidate-settle-context.ts';
export {
  requireWithheldSlateDecided,
  WithheldSlateAbsenceError,
} from './consolidate-settle-judge.ts';
export {
  type ConsolidationSettlement,
  type ConsolidationSubject,
  type ConsolidationTerminal,
  settleConsolidation,
} from './consolidate-settle.ts';
export {
  buyConsolidationSlice,
  standingKeptUnendorsed,
} from './consolidate-slice-buy.ts';
export {
  readStandingVerdict,
  type StandingVerdict,
} from './consolidate-standing-verdict.ts';
export {
  contestStandingMayShip,
  standingTextFor,
} from './consolidate-standing.ts';
export {
  floorConsolidateSlate,
  type ProposalValidity,
  type SlateFloor,
} from './consolidate-validity-floor.ts';
export {
  buildConsolidateMessages,
  type ConsolidateSubject,
} from './consolidate-wire.ts';
export {
  wrapConsolidation,
  wrapConsolidationProposals,
  type WrappedConsolidation,
} from './consolidate-wrap.ts';
export { ConsolidationLedgerGapError, } from './consolidation-ledger-gap.ts';
export { applyFinalPolish, } from './consolidation-polish-apply.ts';
export { configuredConsolidationPolish, } from './consolidation-polish-config.ts';
export {
  CONSOLIDATION_POLISH_GATE_QUORUM,
  CONSOLIDATION_POLISH_GATE_RESPONSE_FORMAT,
  type ConsolidationPolishGateOutcome,
  type ConsolidationPolishSettlement,
  type ConsolidationPolishSettlementRule,
  gateConsolidationPolish,
  readConsolidationPolishSettlement,
} from './consolidation-polish-gate-stage.ts';
export {
  buildConsolidationPolishGateMessages,
  type ConsolidationPolishBallot,
  type ConsolidationPolishGateSubject,
  type ConsolidationPolishGateWire,
  isConsolidationPolishGateWire,
  POLISH_NAMES,
  type PolishChoice,
  readConsolidationPolishBallot,
} from './consolidation-polish-gate-wire.ts';
export {
  type ConsolidationNaturalnessAudit,
  type ConsolidationNaturalnessCorrectionAudit,
  type ConsolidationPolish,
  type ConsolidationPolishConfig,
} from './consolidation-polish-model.ts';
export {
  finalPolishParagraphs,
  reviewParagraphsOf,
} from './consolidation-polish-round.ts';
export { unpolishedBaseline, } from './consolidation-polish-skip.ts';
export { polishConsolidation, } from './consolidation-polish.ts';

//endregion Public index
