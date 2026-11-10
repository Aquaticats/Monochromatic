/**
 Active-branch goal event reducer.
 
 @module
 */

import type {
  AbsentGoalState,
  GoalEvent,
  GoalState,
} from './types.ts';

//region Canonical absent state

/**
 Canonical absent goal state.
 */
const ABSENT_GOAL_STATE: AbsentGoalState = { phase: 'absent', };

//endregion Canonical absent state

//region Event reduction

/**
 Reduce one immutable event over current branch state.
 
 Stale generation events are ignored.
 A new run always supersedes current active or terminal record atomically.
 
 @param state - current branch state
 
 @param event - next ordered event
 
 @returns next branch state
 
 @example
 ```ts
 const next = reduceGoalEvent({ state, event });
 ```
 */
function reduceGoalEvent(
  {
    state,
    event,
  }: {
    readonly state: GoalState;
    readonly event: GoalEvent;
  },
): GoalState {
  if (event.kind === 'run_started') {
    return {
      phase: 'active',
      runId: event.runId,
      generationId: event.generationId,
      objective: event.objective,
      startedAt: event.startedAt,
      startBoundary: event.startBoundary,
      continuationSequence: event.continuationSequence,
      transitionedAt: event.transitionedAt,
    };
  }
  if (event.kind === 'generation_rotated') {
    if ((state.phase !== 'active')
      || (state.runId !== event.runId)
      || (state.generationId !== event.previousGenerationId)) {
      return state;
    }
    return {
      ...state,
      generationId: event.generationId,
      continuationSequence: event.continuationSequence,
      transitionedAt: event.transitionedAt,
    };
  }
  if (event.kind === 'review_denied') {
    if ((state.phase !== 'active')
      || (state.runId !== event.runId)
      || (state.generationId !== event.generationId)) {
      return state;
    }
    return {
      ...state,
      continuationSequence: event.continuationSequence,
      transitionedAt: event.transitionedAt,
      remainingWork: 'remainingWork' in event
        ? event.remainingWork
        : event.feedback,
    };
  }
  if (event.kind === 'continuation_issued') {
    if ((state.phase !== 'active')
      || (state.runId !== event.runId)
      || (state.generationId !== event.generationId)) {
      return state;
    }
    return {
      ...state,
      continuationSequence: event.continuationSequence,
      transitionedAt: event.transitionedAt,
    };
  }
  if (event.kind === 'run_completed_model') {
    if ((state.phase !== 'active')
      || (state.runId !== event.runId)
      || (state.generationId !== event.generationId)) {
      return state;
    }
    return {
      phase: 'completed',
      runId: state.runId,
      generationId: state.generationId,
      objective: state.objective,
      approvalSource: 'model',
      reviewerIdentity: event.reviewerIdentity,
      reviewerRationale: event.reviewerRationale
        ?? event.reviewerFeedback
        ?? 'Approved by legacy reviewer.',
      attemptedReviewerIdentities: event.attemptedReviewerIdentities
        ?? [event.reviewerIdentity,],
      transcriptTruncated: event.transcriptTruncated ?? false,
      completedAt: event.completedAt,
      ...(event.summary === undefined ? {} : { legacySummary: event.summary, }),
    };
  }
  if (event.kind === 'run_completed_manual') {
    if ((state.phase !== 'active')
      || (state.runId !== event.runId)
      || (state.generationId !== event.generationId)) {
      return state;
    }
    return {
      phase: 'completed',
      runId: state.runId,
      generationId: state.generationId,
      objective: state.objective,
      approvalSource: 'manual',
      reviewerRationale: event.reviewerRationale
        ?? event.reviewerFeedback
        ?? 'Approved manually after reviewer exhaustion.',
      attemptedReviewerIdentities: event.attemptedReviewerIdentities ?? [],
      completedAt: event.completedAt,
      ...(event.summary === undefined ? {} : { legacySummary: event.summary, }),
    };
  }
  if (event.kind === 'review_unavailable') {
    if ((state.phase !== 'active')
      || (state.runId !== event.runId)
      || (state.generationId !== event.generationId)) {
      return state;
    }
    return {
      phase: 'review_unavailable',
      runId: state.runId,
      generationId: state.generationId,
      objective: state.objective,
      attemptedReviewerIdentities: [...event.attemptedReviewerIdentities,],
      diagnostic: event.diagnostic,
      terminalAt: event.terminalAt,
      ...(event.summary === undefined ? {} : { legacySummary: event.summary, }),
    };
  }
  if (event.kind === 'run_cleared') {
    if (state.phase === 'absent')
      return state;
    if ((state.runId !== event.runId)
      || (state.generationId !== event.generationId)) {
      return state;
    }
    return ABSENT_GOAL_STATE;
  }
  return state;
}

/**
 Reduce ordered active-branch events to current state.
 
 @param events - validated events from selected branch
 
 @returns current goal state
 
 @example
 ```ts
 const state = reduceGoalEvents(events);
 ```
 */
function reduceGoalEvents(events: readonly GoalEvent[],): GoalState {
  return events.reduce<GoalState>(
    function reduceEvent(
      state,
      event,
    ) {
      return reduceGoalEvent({
        state,
        event,
      },);
    },
    ABSENT_GOAL_STATE,
  );
}

//endregion Event reduction

export {
  ABSENT_GOAL_STATE,
  reduceGoalEvent,
  reduceGoalEvents,
};
