/**
 Manual approval and reviewer-unavailable terminal transitions.
 
 @module
 */

import type { GoalSettlementReviewRequest, } from './completion-types.ts';
import { reduceGoalEvent, } from './reducer.ts';
import type {
  GoalControllerState,
  GoalControllerTransition,
} from './types.ts';

//region Manual completion

/**
 Persist manual approval after every model reviewer attempt failed.
 
 @param controller - current revalidated controller
 
 @param request - captured settlement identity
 
 @param attemptedReviewerIdentities - exhausted model transports
 
 @param diagnostic - normalized model failure diagnostic
 
 @param timestamp - ISO completion timestamp
 
 @returns terminal manually approved transition
 
 @example
 ```ts
 manuallyApproveGoalCompletion({ controller, request, attemptedReviewerIdentities: [], diagnostic, timestamp });
 ```
 */
function manuallyApproveGoalCompletion(
  {
    controller,
    request,
    attemptedReviewerIdentities,
    diagnostic,
    timestamp,
  }: {
    readonly controller: GoalControllerState;
    readonly request: GoalSettlementReviewRequest;
    readonly attemptedReviewerIdentities: readonly string[];
    readonly diagnostic: string;
    readonly timestamp: string;
  },
): GoalControllerTransition {
  if (controller.goal
    .phase
    !== 'active')
    throw new Error('Cannot manually approve completion for non-active goal',);
  /**
   Persisted manually approved terminal event.
   */
  const event = {
    kind: 'run_completed_manual',
    runId: request.goal
      .runId,
    generationId: request.goal
      .generationId,
    reviewerRationale: diagnostic,
    attemptedReviewerIdentities,
    completedAt: timestamp,
  } as const;
  /**
   Terminal state derived through branch reducer.
   */
  const goal = reduceGoalEvent({
    state: controller.goal,
    event,
  },);
  return {
    controller: {
      goal,
      runtimeEpoch: controller.runtimeEpoch,
      settlementSequence: controller.settlementSequence,
      shutdown: controller.shutdown,
    },
    effects: [
      {
        type: 'persist',
        event,
      },
      {
        type: 'persist_completion_diagnostic',
        diagnostic: {
          runId: request.goal
            .runId,
          generationId: request.goal
            .generationId,
          approvalSource: 'manual',
          reviewerIdentity: 'manual',
          reviewerRationale: diagnostic,
          attemptedReviewerIdentities,
          transcriptTruncated: false,
          completedAt: timestamp,
        },
      },
      { type: 'clear_footer', },
      {
        type: 'log',
        level: 'warn',
        message: `goal completion manually approved for ${request.goal
          .runId}`,
      },
    ],
  };
}

//endregion Manual completion

//region Review unavailable

/**
 Persist non-interactive reviewer exhaustion as terminal goal state.
 
 @param controller - current revalidated controller
 
 @param request - captured settlement identity
 
 @param attemptedReviewerIdentities - model transports that started
 
 @param diagnostic - normalized model failure diagnostic
 
 @param timestamp - ISO terminal timestamp
 
 @returns terminal reviewer-unavailable transition and renderable diagnostic
 
 @example
 ```ts
 markGoalReviewUnavailable({ controller, request, attemptedReviewerIdentities, diagnostic, timestamp });
 ```
 */
function markGoalReviewUnavailable(
  {
    controller,
    request,
    attemptedReviewerIdentities,
    diagnostic,
    timestamp,
  }: {
    readonly controller: GoalControllerState;
    readonly request: GoalSettlementReviewRequest;
    readonly attemptedReviewerIdentities: readonly string[];
    readonly diagnostic: string;
    readonly timestamp: string;
  },
): GoalControllerTransition {
  if (controller.goal
    .phase
    !== 'active')
    throw new Error('Cannot mark reviewer unavailable for non-active goal',);
  /**
   Persisted non-interactive terminal event.
   */
  const event = {
    kind: 'review_unavailable',
    runId: request.goal
      .runId,
    generationId: request.goal
      .generationId,
    attemptedReviewerIdentities,
    diagnostic,
    terminalAt: timestamp,
  } as const;
  /**
   Terminal state derived through branch reducer.
   */
  const goal = reduceGoalEvent({
    state: controller.goal,
    event,
  },);
  return {
    controller: {
      goal,
      runtimeEpoch: controller.runtimeEpoch,
      settlementSequence: controller.settlementSequence,
      shutdown: controller.shutdown,
    },
    effects: [
      {
        type: 'persist',
        event,
      },
      {
        type: 'persist_review_unavailable_diagnostic',
        diagnostic: {
          runId: request.goal
            .runId,
          generationId: request.goal
            .generationId,
          attemptedReviewerIdentities,
          diagnostic,
          terminalAt: timestamp,
        },
      },
      { type: 'clear_footer', },
      {
        type: 'log',
        level: 'error',
        message: `goal reviewer unavailable for ${request.goal
          .runId}: ${diagnostic}`,
      },
    ],
  };
}

//endregion Review unavailable

export {
  manuallyApproveGoalCompletion,
  markGoalReviewUnavailable,
};
