/**
 Harness-owned settlement review and stale-result handling.
 
 @module
 */

import type { ExtensionContext, } from '@earendil-works/pi-coding-agent';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  approveGoalCompletion,
  continueGoalAfterDenial,
  settlementReviewControllerStillCurrent,
  settlementReviewStillCurrent,
} from './completion-outcome.ts';
import type {
  GoalSettlementDisposition,
  GoalSettlementReviewer,
  GoalSettlementReviewRequest,
} from './completion-types.ts';
import type { GoalLifecycleHandle, } from './lifecycle-services.ts';
import type { GoalControllerState, } from './types.ts';

/**
 Domain sentinel for settlement without active reviewable goal.
 */
const GOAL_SETTLEMENT_NOT_REVIEWABLE: unique symbol = Symbol('goal settlement is not reviewable',);

/**
 Mode-specific reviewer-exhaustion capability.
 */
type GoalReviewerUnavailableHandler = (
  input: {
    readonly error: unknown;
    readonly request: GoalSettlementReviewRequest;
    readonly context: ForeignBorrowed<ExtensionContext>;
  },
) => Promise<GoalSettlementDisposition>;

/**
 Capture active settlement identity before reviewer spending.
 
 @param controller - current immutable runtime controller
 
 @param branchLeafId - selected finalized session leaf
 
 @returns review request or absent marker
 
 @example
 ```ts
 createGoalSettlementReviewRequest({ controller, branchLeafId: 'leaf-1' });
 ```
 */
function createGoalSettlementReviewRequest(
  {
    controller,
    branchLeafId,
  }: {
    readonly controller: GoalControllerState;
    readonly branchLeafId: string;
  },
): GoalSettlementReviewRequest | typeof GOAL_SETTLEMENT_NOT_REVIEWABLE {
  if (controller.shutdown || (controller.goal
    .phase
    !== 'active'))
    return GOAL_SETTLEMENT_NOT_REVIEWABLE;
  return {
    goal: controller.goal,
    runtimeEpoch: controller.runtimeEpoch,
    branchLeafId,
    settlementSequence: controller.settlementSequence,
  };
}

/**
 Revalidate async review without touching replaced session context first.
 
 @param lifecycle - current runtime controller seam
 
 @param request - captured settlement identity
 
 @param context - original context used only after controller match
 
 @returns current controller or stale marker
 
 @mutates context - context.sessionManager.getLeafId may change Pi-owned state
 
 @example
 ```ts
 revalidateSettlementReview({ lifecycle, request, context });
 ```
 */
function revalidateSettlementReview(
  {
    lifecycle,
    request,
    context,
  }: {
    readonly lifecycle: GoalLifecycleHandle;
    readonly request: GoalSettlementReviewRequest;
    readonly context: ForeignBorrowed<ExtensionContext>;
  },):
    | {
      readonly current: true;
      readonly controller: GoalControllerState;
    }
      | { readonly current: false; } {
  /**
   Current controller read before captured session-handle use.
   */
  const controller = lifecycle.currentController();
  if (!settlementReviewControllerStillCurrent({
    controller,
    request,
  }))
    return { current: false, };
  if ((!context.isIdle()) || context.hasPendingMessages())
    return { current: false, };
  /**
   Current branch leaf read only while runtime and generation match.
   */
  const branchLeafId = context.sessionManager
    .getLeafId();
  if (branchLeafId === null)
    return { current: false, };
  if (!settlementReviewStillCurrent({
    controller,
    request,
    branchLeafId,
  },)) {
    return { current: false, };
  }
  return {
    current: true,
    controller,
  };
}

/**
 Acquire reviewer result or mode-specific fallback disposition.
 
 @param request - captured active settlement
 
 @param context - current Pi context
 
 @param reviewer - independent model reviewer
 
 @param handleReviewerUnavailable - mode-specific exhaustion behavior
 
 @param signal - optional cancellation signal
 
 @returns available review or settled fallback disposition
 
 @mutates context - reviewer or fallback may update Pi state and UI
 
 @mutates reviewer - reviewer may update transport state
 
 @mutates handleReviewerUnavailable - fallback may update UI state
 
 @mutates signal - reviewer transport may retain cancellation signal
 
 @example
 ```ts
 await acquireGoalSettlementReview({ request, context, reviewer, handleReviewerUnavailable });
 ```
 */
async function acquireGoalSettlementReview(
  {
    request,
    context,
    reviewer,
    handleReviewerUnavailable,
    signal,
  }: {
    readonly request: GoalSettlementReviewRequest;
    readonly context: ForeignBorrowed<ExtensionContext>;
    readonly reviewer: ForeignBorrowed<GoalSettlementReviewer>;
    readonly handleReviewerUnavailable: ForeignBorrowed<GoalReviewerUnavailableHandler>;
    readonly signal?: AbortSignal;
  },
): Promise<
  | {
    readonly available: true;
    readonly review: Awaited<ReturnType<GoalSettlementReviewer>>;
  }
    | {
      readonly available: false;
      readonly disposition: GoalSettlementDisposition;
    }
  > {
  try {
    return {
      available: true,
      review: await reviewer({
        request,
        context,
        ...(signal === undefined ? {} : { signal, }),
      },),
    };
  }
  catch (error) {
    if (signal?.aborted === true) {
      return {
        available: false,
        disposition: 'stale',
      };
    }
    return {
      available: false,
      disposition: await handleReviewerUnavailable({
        error,
        request,
        context,
      },),
    };
  }
}

/**
 Execute one finalized settlement review.
 
 @param request - captured active settlement
 
 @param context - current Pi extension context
 
 @param lifecycle - shared controller seam
 
 @param reviewer - independent model reviewer
 
 @param handleReviewerUnavailable - mode-specific exhaustion behavior
 
 @param createId - private continuation identity source
 
 @param now - ISO timestamp source
 
 @param signal - optional cancellation signal
 
 @returns harness-internal settlement disposition
 
 @mutates context - reviewer and lifecycle transitions can update Pi state and UI
 
 @mutates reviewer - reviewer may update transport state
 
 @mutates handleReviewerUnavailable - fallback may update UI state
 
 @mutates signal - reviewer transport may retain cancellation signal
 
 @example
 ```ts
 await executeGoalSettlementReview({ request, context, lifecycle, reviewer, handleReviewerUnavailable, createId, now });
 ```
 */
async function executeGoalSettlementReview(
  {
    request,
    context,
    lifecycle,
    reviewer,
    handleReviewerUnavailable,
    createId,
    now,
    signal,
  }: {
    readonly request: GoalSettlementReviewRequest;
    readonly context: ForeignBorrowed<ExtensionContext>;
    readonly lifecycle: GoalLifecycleHandle;
    readonly reviewer: ForeignBorrowed<GoalSettlementReviewer>;
    readonly handleReviewerUnavailable: ForeignBorrowed<GoalReviewerUnavailableHandler>;
    readonly createId: () => string;
    readonly now: () => string;
    readonly signal?: AbortSignal;
  },
): Promise<GoalSettlementDisposition> {
  /**
   Independent review or mode-specific exhaustion result.
   */
  const acquisition = await acquireGoalSettlementReview({
    request,
    context,
    reviewer,
    handleReviewerUnavailable,
    ...(signal === undefined ? {} : { signal, }),
  },);
  if (!acquisition.available)
    return acquisition.disposition;
  /**
   Available independent review.
   */
  const { review, } = acquisition;
  /**
   Post-review stale-result validation.
   */
  const revalidation = revalidateSettlementReview({
    lifecycle,
    request,
    context,
  },);
  if (!revalidation.current)
    return 'stale';
  if (!review.verdict
    .approved) {
    lifecycle.applyTransition({
      transition: continueGoalAfterDenial({
        controller: revalidation.controller,
        request,
        review,
        marker: createId(),
        timestamp: now(),
      },),
      context,
    },);
    return 'continued';
  }
  lifecycle.applyTransition({
    transition: approveGoalCompletion({
      controller: revalidation.controller,
      request,
      review,
      timestamp: now(),
    },),
    context,
  },);
  return 'approved';
}

export {
  acquireGoalSettlementReview,
  createGoalSettlementReviewRequest,
  GOAL_SETTLEMENT_NOT_REVIEWABLE,
  executeGoalSettlementReview,
  revalidateSettlementReview,
};
export type { GoalReviewerUnavailableHandler, };
