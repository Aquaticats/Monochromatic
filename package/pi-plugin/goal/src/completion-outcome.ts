/**
 Settlement-review revalidation and persisted outcome transitions.
 
 @module
 */

import type {
  GoalSettlementReview,
  GoalSettlementReviewRequest,
} from './completion-types.ts';
import { buildGoalMessage, } from './message.ts';
import { reduceGoalEvent, } from './reducer.ts';
import type {
  GoalControllerState,
  GoalControllerTransition,
  GoalMessageMarker,
} from './types.ts';

/**
 Revalidate controller identities before touching captured session context.
 
 @param controller - current live controller
 
 @param request - settlement capture before asynchronous work
 
 @returns whether review still belongs to current runtime generation
 
 @example
 ```ts
 settlementReviewControllerStillCurrent({ controller, request });
 ```
 */
function settlementReviewControllerStillCurrent(
  {
    controller,
    request,
  }: {
    readonly controller: GoalControllerState;
    readonly request: GoalSettlementReviewRequest;
  },
): boolean {
  if (controller.shutdown)
    return false;
  if (controller.runtimeEpoch !== request.runtimeEpoch)
    return false;
  if (controller.settlementSequence !== request.settlementSequence)
    return false;
  if (controller.goal
    .phase
    !== 'active')
    return false;
  if (controller.goal
    .runId
    !== request.goal
      .runId)
    return false;
  return controller.goal
    .generationId
    === request.goal
      .generationId;
}

/**
 Revalidate controller and selected branch after asynchronous work.
 
 @param controller - current live controller
 
 @param request - settlement capture before asynchronous work
 
 @param branchLeafId - current selected branch leaf
 
 @returns whether review still belongs to current selected settlement
 
 @example
 ```ts
 settlementReviewStillCurrent({ controller, request, branchLeafId });
 ```
 */
function settlementReviewStillCurrent(
  {
    controller,
    request,
    branchLeafId,
  }: {
    readonly controller: GoalControllerState;
    readonly request: GoalSettlementReviewRequest;
    readonly branchLeafId: string;
  },
): boolean {
  return settlementReviewControllerStillCurrent({
    controller,
    request,
  },) && (branchLeafId === request.branchLeafId);
}

/**
 Persist valid denial and emit one task-only continuation.
 
 @param controller - current revalidated controller
 
 @param request - captured settlement identity
 
 @param review - valid private reviewer result
 
 @param marker - private continuation identity
 
 @param timestamp - ISO denial timestamp
 
 @returns active denial transition with one primary turn
 
 @example
 ```ts
 continueGoalAfterDenial({ controller, request, review, marker, timestamp });
 ```
 */
function continueGoalAfterDenial(
  {
    controller,
    request,
    review,
    marker,
    timestamp,
  }: {
    readonly controller: GoalControllerState;
    readonly request: GoalSettlementReviewRequest;
    readonly review: GoalSettlementReview;
    readonly marker: GoalMessageMarker;
    readonly timestamp: string;
  },
): GoalControllerTransition {
  if (controller.goal
    .phase
    !== 'active')
    throw new Error('Cannot continue denied non-active goal',);
  if (review.verdict
    .approved)
    throw new Error('Cannot continue approved goal as denial',);
  /**
   Persisted private reviewer audit.
   */
  const denialEvent = {
    kind: 'review_denied',
    runId: request.goal
      .runId,
    generationId: request.goal
      .generationId,
    remainingWork: review.verdict
      .remainingWork,
    reviewerIdentity: review.reviewerIdentity,
    reviewerRationale: review.verdict
      .rationale,
    attemptedReviewerIdentities: review.attemptedReviewerIdentities,
    transcriptTruncated: review.transcriptTruncated,
    continuationSequence: controller.goal
      .continuationSequence,
    transitionedAt: timestamp,
  } as const;
  /**
   Active state retaining task-only remaining work.
   */
  const deniedGoal = reduceGoalEvent({
    state: controller.goal,
    event: denialEvent,
  },);
  if (deniedGoal.phase !== 'active')
    throw new Error('Goal denial did not retain active state',);
  /**
   Next private continuation sequence.
   */
  const continuationSequence = deniedGoal.continuationSequence + 1;
  /**
   Persisted continuation issuance.
   */
  const continuationEvent = {
    kind: 'continuation_issued',
    runId: deniedGoal.runId,
    generationId: deniedGoal.generationId,
    continuationSequence,
    transitionedAt: timestamp,
  } as const;
  /**
   Active state advanced through branch reducer.
   */
  const goal = reduceGoalEvent({
    state: deniedGoal,
    event: continuationEvent,
  },);
  if (goal.phase !== 'active')
    throw new Error('Goal continuation did not retain active state',);
  /**
   Task-only continuation visible to primary model.
   */
  const message = buildGoalMessage({
    goal,
    kind: 'continuation',
    continuationSequence,
    marker,
    remainingWork: review.verdict
      .remainingWork,
  },);
  /**
   Runtime settlement sequence after continuation.
   */
  const settlementSequence = controller.settlementSequence + 1;
  return {
    controller: {
      goal,
      runtimeEpoch: controller.runtimeEpoch,
      settlementSequence,
      lastEmittedSettlementSequence: settlementSequence,
      shutdown: controller.shutdown,
    },
    effects: [
      {
        type: 'persist',
        event: denialEvent,
      },
      {
        type: 'persist',
        event: continuationEvent,
      },
      {
        type: 'send_message',
        message,
        triggerTurn: true,
      },
      {
        type: 'log',
        level: 'debug',
        message: `continued goal run ${goal.runId} after private denial`,
      },
    ],
  };
}

/**
 Persist valid model approval as terminal completion.
 
 @param controller - current revalidated controller
 
 @param request - captured settlement identity
 
 @param review - valid approving reviewer result
 
 @param timestamp - ISO completion timestamp
 
 @returns terminal model-approved transition
 
 @example
 ```ts
 approveGoalCompletion({ controller, request, review, timestamp });
 ```
 */
function approveGoalCompletion(
  {
    controller,
    request,
    review,
    timestamp,
  }: {
    readonly controller: GoalControllerState;
    readonly request: GoalSettlementReviewRequest;
    readonly review: GoalSettlementReview;
    readonly timestamp: string;
  },
): GoalControllerTransition {
  if (controller.goal
    .phase
    !== 'active')
    throw new Error('Cannot approve completion for non-active goal',);
  if (!review.verdict
    .approved)
    throw new Error('Cannot approve denied goal review',);
  /**
   Persisted model-approved terminal event.
   */
  const event = {
    kind: 'run_completed_model',
    runId: request.goal
      .runId,
    generationId: request.goal
      .generationId,
    reviewerIdentity: review.reviewerIdentity,
    reviewerRationale: review.verdict
      .rationale,
    attemptedReviewerIdentities: review.attemptedReviewerIdentities,
    transcriptTruncated: review.transcriptTruncated,
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
          approvalSource: 'model',
          reviewerIdentity: review.reviewerIdentity,
          reviewerRationale: review.verdict
            .rationale,
          attemptedReviewerIdentities: review.attemptedReviewerIdentities,
          transcriptTruncated: review.transcriptTruncated,
          completedAt: timestamp,
        },
      },
      { type: 'clear_footer', },
      {
        type: 'log',
        level: 'debug',
        message: `goal completion approved privately for ${request.goal
          .runId}`,
      },
    ],
  };
}

export {
  approveGoalCompletion,
  continueGoalAfterDenial,
  settlementReviewControllerStillCurrent,
  settlementReviewStillCurrent,
};
