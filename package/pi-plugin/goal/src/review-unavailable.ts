/**
 TUI manual and non-interactive fallbacks after reviewer exhaustion.
 
 @module
 */

import type { ExtensionContext, } from '@earendil-works/pi-coding-agent';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { ReviewUnavailableError, } from '@monochromatic-dev/pi-shared-model-review/ts';

import {
  revalidateSettlementReview,
  type GoalReviewerUnavailableHandler,
} from './completion.ts';
import { continueGoalAfterDenial, } from './completion-outcome.ts';
import type { GoalSettlementDisposition, } from './completion-types.ts';
import {
  manuallyApproveGoalCompletion,
  markGoalReviewUnavailable,
} from './completion-terminal.ts';
import {
  defaultCreateId,
  defaultNow,
  type GoalLifecycleHandle,
} from './lifecycle-services.ts';
import {
  type ManualGoalReviewPrompt,
  promptManualGoalReview,
} from './manual-review-dialog.ts';

/**
 Task-only fallback guidance when manual rejection has no reason.
 */
const DEFAULT_MANUAL_REJECTION_REMAINING_WORK = 'Continue working on the current user objective.';

/**
 Normalized model-review exhaustion audit.
 */
type GoalReviewFailureAudit = {
  readonly attemptedReviewerIdentities: readonly string[];
  readonly diagnostics: readonly string[];
  readonly diagnostic: string;
};

/**
 Convert arbitrary exhausted-review error to stable audit fields.
 
 @param error - model selection, auth, transport, timeout, or parser failure
 
 @returns attempted identities and normalized diagnostics
 
 @example
 ```ts
 normalizeGoalReviewFailure(error);
 ```
 */
function normalizeGoalReviewFailure(error: unknown,): GoalReviewFailureAudit {
  if (error instanceof ReviewUnavailableError) {
    /**
     Shared normalized diagnostics with non-empty fallback.
     */
    const diagnostics = error.diagnostics
      .length
      === 0
      ? [error.message,]
      : error.diagnostics;
    return {
      attemptedReviewerIdentities: error.attemptedCandidateIdentities,
      diagnostics,
      diagnostic: diagnostics.join('; ',),
    };
  }
  /**
   Normalized unexpected reviewer orchestration failure.
   */
  const diagnostic = caughtValueText(error,);
  return {
    attemptedReviewerIdentities: [],
    diagnostics: [diagnostic,],
    diagnostic,
  };
}

/**
 Return explicit stale fallback disposition.
 
 @returns stale harness outcome
 
 @example
 ```ts
 staleFallbackDisposition();
 ```
 */
function staleFallbackDisposition(): GoalSettlementDisposition {
  return 'stale';
}

/**
 Create mode-specific reviewer exhaustion handler.
 
 @param lifecycle - live runtime used for stale revalidation
 
 @param promptManualReview - injectable mandatory TUI decision dialog
 
 @param createId - private continuation identity source
 
 @param now - timestamp source
 
 @returns reviewer-unavailable handler
 
 @mutates promptManualReview - dialog capability may update TUI state
 
 @example
 ```ts
 const handler = createGoalReviewerUnavailableHandler({ lifecycle });
 ```
 */
function createGoalReviewerUnavailableHandler(
  {
    lifecycle,
    promptManualReview = promptManualGoalReview,
    createId = defaultCreateId,
    now = defaultNow,
  }: {
    readonly lifecycle: GoalLifecycleHandle;
    readonly promptManualReview?: ForeignBorrowed<ManualGoalReviewPrompt>;
    readonly createId?: () => string;
    readonly now?: () => string;
  },
): GoalReviewerUnavailableHandler {
  return async function handleGoalReviewerUnavailable(
    {
      error,
      request,
      context,
    },
  ) {
    /**
     Normalized failed-model audit.
     */
    const audit = normalizeGoalReviewFailure(error,);
    /**
     Stale check before mode-specific UI or transition.
     */
    const initialRevalidation = revalidateSettlementReview({
      lifecycle,
      request,
      context,
    },);
    if (!initialRevalidation.current)
      return staleFallbackDisposition();
    if (context.mode !== 'tui') {
      lifecycle.applyTransition({
        transition: markGoalReviewUnavailable({
          controller: initialRevalidation.controller,
          request,
          attemptedReviewerIdentities: audit.attemptedReviewerIdentities,
          diagnostic: audit.diagnostic,
          timestamp: now(),
        },),
        context,
      },);
      return 'review_unavailable';
    }
    /**
     Mandatory human decision after model exhaustion.
     */
    const decision = await promptManualReview({
      context,
      diagnostic: audit.diagnostic,
    },);
    /**
     Post-dialog stale check before state mutation.
     */
    const finalRevalidation = revalidateSettlementReview({
      lifecycle,
      request,
      context,
    },);
    if (!finalRevalidation.current)
      return staleFallbackDisposition();
    if (decision.action === 'accept') {
      lifecycle.applyTransition({
        transition: manuallyApproveGoalCompletion({
          controller: finalRevalidation.controller,
          request,
          attemptedReviewerIdentities: audit.attemptedReviewerIdentities,
          diagnostic: audit.diagnostic,
          timestamp: now(),
        },),
        context,
      },);
      return 'approved';
    }
    /**
     Optional human reason normalized to task-only fallback guidance.
     */
    const reason = decision.reason
      .trim();
    /**
     Task-only guidance from human reason or stable fallback.
     */
    const remainingWork = reason === ''
      ? DEFAULT_MANUAL_REJECTION_REMAINING_WORK
      : reason;
    lifecycle.applyTransition({
      transition: continueGoalAfterDenial({
        controller: finalRevalidation.controller,
        request,
        review: {
          verdict: {
            approved: false,
            rationale: `Manual rejection after reviewer exhaustion: ${audit.diagnostic}`,
            remainingWork,
          },
          reviewerIdentity: 'manual',
          attemptedReviewerIdentities: audit.attemptedReviewerIdentities,
          transcriptTruncated: false,
        },
        marker: createId(),
        timestamp: now(),
      },),
      context,
    },);
    return 'continued';
  };
}

export {
  createGoalReviewerUnavailableHandler,
  normalizeGoalReviewFailure,
  staleFallbackDisposition,
};
export type { GoalReviewFailureAudit, };
