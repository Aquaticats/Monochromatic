/**
 Settlement-review domain contracts.
 
 @module
 */

import type {
  Api,
  Model,
} from '@earendil-works/pi-ai';
import type { ExtensionContext, } from '@earendil-works/pi-coding-agent';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type { StructuredReviewAuth, } from '@monochromatic-dev/pi-shared-model-review/ts';

import type {
  ActiveGoalState,
  GoalRuntimeEpoch,
} from './types.ts';

/**
 Strict private secondary-review decision.
 */
type GoalReviewVerdict = {
  readonly approved: boolean;
  readonly rationale: string;
  readonly remainingWork: string;
};

/**
 Settlement identity captured before asynchronous review.
 */
type GoalSettlementReviewRequest = {
  readonly goal: ActiveGoalState;
  readonly runtimeEpoch: GoalRuntimeEpoch;
  readonly branchLeafId: string;
  readonly settlementSequence: number;
};

/**
 Serialized post-start evidence independent of reviewer model budget.
 */
type GoalReviewEvidence = {
  readonly objective: string;
  readonly transcriptChunks: readonly string[];
};

/**
 Authenticated distinct reviewer and candidate-specific prompt.
 */
type GoalReviewerCandidate = {
  readonly model: ForeignBorrowed<Model<Api>>;
  readonly auth: ForeignBorrowed<StructuredReviewAuth>;
  readonly systemPrompt: string;
  readonly userContent: string;
  readonly transcriptTruncated: boolean;
};

/**
 Successful independent review with transport audit.
 */
type GoalSettlementReview = {
  readonly verdict: GoalReviewVerdict;
  readonly reviewerIdentity: string;
  readonly attemptedReviewerIdentities: readonly string[];
  readonly transcriptTruncated: boolean;
};

/**
 Production settlement reviewer capability.
 */
type GoalSettlementReviewer = (
  input: {
    readonly request: GoalSettlementReviewRequest;
    readonly context: ForeignBorrowed<ExtensionContext>;
    readonly signal?: AbortSignal;
  },
) => Promise<GoalSettlementReview>;

/**
 Harness-internal outcome of one settlement review.
 */
type GoalSettlementDisposition =
  | 'approved'
    | 'continued'
    | 'review_unavailable'
    | 'stale';

export type {
  GoalReviewerCandidate,
  GoalReviewEvidence,
  GoalReviewVerdict,
  GoalSettlementDisposition,
  GoalSettlementReview,
  GoalSettlementReviewer,
  GoalSettlementReviewRequest,
};
