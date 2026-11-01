//region Public index
// The names `@monochromatic-dev/translation-repair-house-rule` gives its siblings and its own tests, each from the module that declares it.
// Production siblings import it through `/ts`; tests and fixtures through the built entry, so a process loads one copy.

export { BILINGUAL_LINE_CLAUSE, } from './bilingual-line-clause.ts';
export {
  CANDIDATE_APPARATUS_RULE,
  CANDIDATE_DECLARED_NAMES_RULES,
  JUDGE_LINE_STRUCTURE_CLAUSE,
} from './candidate-judge-rules.ts';
export {
  type ContestLedger,
  isLedgerFileName,
  LAUNCH_STAMP,
  LEDGER_DIR,
  type LedgerCandidate,
  ledgerFileName,
  type LedgerRound,
  readContestLedger,
  recordContest,
} from './candidate-ledger.ts';
export { mergeIdenticalCandidates, } from './candidate-merge.ts';
export {
  type Candidate,
  type CandidateProducer,
  type CandidateWeight,
  describeProducer,
  FULL_VOTE_WEIGHT,
  mergeProducers,
  MIN_SELECTION_BALLOTS,
  producerModelIds,
  type SelectionBallot,
  type SelectionDisposition,
  type SelectionOutcome,
  type SelectionTally,
  SELF_VOTE_WEIGHT,
} from './candidate-select-model.ts';
export {
  archiveContributorNameForms,
  type ContributorDeclarationLine,
  contributorDeclarationLines,
  droppedContributorNameForms,
} from './contributor-name-authority.ts';
export {
  CONTRIBUTOR_AUTHORITY_FINDING,
  contributorAuthorityFindings,
} from './contributor-translation-guard.ts';
export {
  declaredNameForms,
  declaredNameRefusalFinding,
  type DeclaredNameRefusalReport,
  declaredNameRefusalReport,
  findDroppedDeclaredNames,
} from './declared-name-survival.ts';
export { formatGradingSheet, } from './grading-sheet.ts';
export {
  HOUSE_FORM_CORRECTION_KEEPS_MEANING,
  HOUSE_FORM_CORRECTIONS,
} from './house-form-corrections.ts';
export {
  HOUSE_POLICY_BLOCK,
  JUDGE_POLICY_BLOCK,
  MEASUREMENT_POLICY_BLOCK,
} from './house-policy.ts';
export {
  collectIdentityLines,
  type DeclaredIdentity,
  extractDeclaredIdentity,
  isDeclared,
  lifeEndedOf,
  sourcePronounLines,
  unreadableDeathKindOf,
} from './identity-context.ts';
export {
  declaredNameNote,
  type DeclaredNamePair,
  declaredNamePairs,
  formsOf,
} from './linked-title-declared-name.ts';
export {
  DECLARED_NAME_REFERENCE_EXEMPTION,
  NAME_FORM_SCOPE_RULE,
} from './name-form-policy.ts';
export {
  carriesName,
  type NameOccurrence,
  nameOccurrences,
  type NameProjection,
  nameProjection,
  projectName,
} from './name-projection.ts';
export { ORDINARY_CAPITALISED_WORDS, } from './ordinary-capitalised-words.ts';
export {
  arrivesAsWholeName,
  type BesideWords,
  besideWordsOf,
  declaredWordsOf,
  type ExcusedNeighbours,
  excusedNeighboursOf,
} from './whole-name-arrival.ts';
export {
  HOUSE_CORRECTION_IS_AN_IMPROVEMENT,
  POLISH_GATE_HOUSE_RULES,
} from './polish-gate-house-rules.ts';
export {
  detectRefusalShape,
  REFUSAL_MARKERS,
  REFUSAL_SCAN_WINDOW,
  type RefusalScan,
} from './refusal.ts';
export {
  type IssueAuthorship,
  UNATTRIBUTED_TEXT,
  wroteTextForIssue,
} from './resolution-authorship.ts';
export {
  CHECKER_IDENTITY_RULE,
  CHECKER_QUOTE_RULE,
  CHECKER_REFERENCE_RULE,
  jsonLine,
  resolutionIssueBlock,
} from './resolution-sheet-evidence.ts';
export {
  allocateBandQuota,
  drawStratifiedSample,
} from './sample-draw.ts';
export {
  assertRepairMeasurable,
  assertSourceBytes,
  type BandQuota,
  classifyBand,
  classifySourceAnchor,
  countUnrecordedRepairs,
  DEFAULT_PRECISION_BAR,
  DEFAULT_SAMPLE_SEED,
  DEFAULT_SAMPLE_SIZE,
  extractGradingCandidate,
  type GradableClaim,
  type GradableIssue,
  type GradableRepair,
  type GradableRepairRegion,
  type GradableSpan,
  type GradingCandidate,
  MEDIUM_BAND_MAX_BYTES,
  SIZE_BANDS,
  type SizeBand,
  SMALL_BAND_MAX_BYTES,
  type SourceAnchorKind,
  type SourceBytes,
  sourceBytesOf,
  UnmeasurableRepairError,
} from './sample-grading.ts';
export {
  flattenSpace,
  indentContinuation,
} from './sheet-line-text.ts';

//endregion Public index
