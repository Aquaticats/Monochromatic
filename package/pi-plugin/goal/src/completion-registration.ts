/**
 Harness-owned settlement-review registration.
 
 @module
 */

import type {
  AgentEndEvent,
  AgentSettledEvent,
  ExtensionAPI,
  ExtensionContext,
} from '@earendil-works/pi-coding-agent';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { registerBackgroundProcessMonitor, } from './background-process-monitor.ts';
import { logger, } from './effects.ts';
import {
  createGoalSettlementReviewRequest,
  executeGoalSettlementReview,
  GOAL_SETTLEMENT_NOT_REVIEWABLE,
  type GoalReviewerUnavailableHandler,
} from './completion.ts';
import type { GoalSettlementReviewer, } from './completion-types.ts';
import {
  defaultCreateId,
  defaultNow,
  type GoalLifecycleHandle,
} from './lifecycle-services.ts';
import { reviewGoalSettlement, } from './review-runner.ts';
import { createGoalReviewerUnavailableHandler, } from './review-unavailable.ts';

/**
 Registration dependencies for private settlement review.
 */
type GoalSettlementReviewRegistration = {
  readonly pi: ForeignBorrowed<ExtensionAPI>;
  readonly lifecycle: GoalLifecycleHandle;
  readonly reviewer?: GoalSettlementReviewer;
  readonly handleReviewerUnavailable?: GoalReviewerUnavailableHandler;
  readonly createId?: () => string;
  readonly now?: () => string;
};

/**
 Domain sentinel before any settlement has been reviewed.
 */
const SETTLEMENT_REVIEW_KEY_ABSENT: unique symbol = Symbol('settlement review key absent',);

/**
 Build stable duplicate-review key from captured settlement.
 
 @param request - captured active settlement
 
 @returns runtime-local duplicate guard key
 
 @example
 ```ts
 settlementReviewKey(request);
 ```
 */
function settlementReviewKey(
  request: Parameters<typeof executeGoalSettlementReview>[0]['request'],
): string {
  return JSON.stringify([
    request.runtimeEpoch,
    request.goal
      .runId,
    request.goal
      .generationId,
    request.branchLeafId,
  ],);
}

/**
 Register private review at Pi's final settlement seam.
 
 @param pi - Pi extension API receiving lifecycle handlers
 
 @param lifecycle - shared goal runtime
 
 @param reviewer - injectable independent reviewer
 
 @param handleReviewerUnavailable - injectable mode-specific fallback
 
 @param createId - private continuation identity source
 
 @param now - timestamp source
 
 @mutates pi - registers agent lifecycle handlers
 
 @example
 ```ts
 registerGoalSettlementReview({ pi, lifecycle });
 ```
 */
function registerGoalSettlementReview(
  {
    pi,
    lifecycle,
    reviewer = reviewGoalSettlement,
    handleReviewerUnavailable,
    createId = defaultCreateId,
    now = defaultNow,
  }: GoalSettlementReviewRegistration,
): void {
  /**
   Passive runtime-local view of background work.
   */
  const backgroundProcessMonitor = registerBackgroundProcessMonitor(pi,);
  /**
   Explicit user abort marker consumed by final settlement.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-function-root-let -- Separate agent_end and agent_settled callbacks share one runtime marker.
  let settledRunWasAborted = false;
  /**
   Most recent captured settlement protected from duplicate callbacks.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-function-root-let -- Runtime-local duplicate guard spans agent_settled callbacks.
  let lastReviewedSettlementKey: string | typeof SETTLEMENT_REVIEW_KEY_ABSENT = SETTLEMENT_REVIEW_KEY_ABSENT;
  /**
   Explicit fallback or production mode-specific exhaustion handler.
   */
  const unavailableHandler = handleReviewerUnavailable
    ?? createGoalReviewerUnavailableHandler({
      lifecycle,
      createId,
      now,
    },);

  pi.on(
    'agent_end',
    function recordAbortedRun(
      event: ForeignBorrowed<AgentEndEvent>,
    ) {
      /**
       Latest assistant message determines explicit abort.
       */
      const finalAssistant = event.messages
        .findLast(function isAssistant(message,) {
          return message.role === 'assistant';
        },);
      settledRunWasAborted = finalAssistant?.stopReason === 'aborted';
    },
  );
  pi.on(
    'agent_settled',
    async function reviewSettledGoal(
      _event: ForeignBorrowed<AgentSettledEvent>,
      context: ForeignBorrowed<ExtensionContext>,
    ) {
      if (settledRunWasAborted) {
        settledRunWasAborted = false;
        return;
      }
      if ((!context.isIdle())
        || context.hasPendingMessages()
        || backgroundProcessMonitor.hasLiveBackgroundProcess()) {
        return;
      }
      if (lifecycle.deliverPendingKickoff(context,))
        return;
      /**
       Finalized selected branch leaf.
       */
      const branchLeafId = context.sessionManager
        .getLeafId();
      if (branchLeafId === null)
        return;
      /**
       Captured active settlement or absent marker.
       */
      const request = createGoalSettlementReviewRequest({
        controller: lifecycle.currentController(),
        branchLeafId,
      },);
      if ((typeof request) === 'symbol') {
        if (request !== GOAL_SETTLEMENT_NOT_REVIEWABLE)
          throw new Error('Unknown settlement review sentinel',);
        return;
      }
      /**
       Duplicate guard for exact runtime, generation, and finalized leaf.
       */
      const reviewKey = settlementReviewKey(request,);
      if (reviewKey === lastReviewedSettlementKey)
        return;
      lastReviewedSettlementKey = reviewKey;
      try {
        await executeGoalSettlementReview({
          request,
          context,
          lifecycle,
          reviewer,
          handleReviewerUnavailable: unavailableHandler,
          createId,
          now,
        },);
      }
      catch (error) {
        logger.error(`private settlement review application failed: ${caughtValueText(error,)}`,);
      }
    },
  );
}

export {
  registerGoalSettlementReview,
  settlementReviewKey,
};
export type { GoalSettlementReviewRegistration, };
