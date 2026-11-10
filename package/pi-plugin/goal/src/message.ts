/**
 Task-only extension messages for goal kickoff and continuation.
 
 @module
 */

import { GOAL_MESSAGE_TYPE, } from './constants.ts';
import type {
  ActiveGoalState,
  GoalMessage,
  GoalMessageMarker,
} from './types.ts';

//region Task-only message construction

/**
 Build model-visible task content without harness protocol.
 
 @param goal - exact active goal state
 
 @param kind - kickoff or continuation
 
 @param remainingWork - actionable task-only denial guidance
 
 @returns task context safe for primary-model input
 
 @example
 ```ts
 goalMessageContent({ goal, kind: 'kickoff' });
 ```
 */
function goalMessageContent(
  {
    goal,
    kind,
    remainingWork,
  }: {
    readonly goal: ActiveGoalState;
    readonly kind: 'kickoff' | 'continuation';
    readonly remainingWork?: string;
  },
): string {
  if (kind === 'kickoff')
    return `User objective (exact JSON string): ${JSON.stringify(goal.objective,)}`;
  if (remainingWork === undefined)
    throw new Error('Goal continuation requires task-only remaining work',);
  return remainingWork;
}

/**
 Build task-only custom message with private lifecycle details.
 
 @param goal - exact active goal state
 
 @param kind - kickoff or continuation
 
 @param continuationSequence - private message sequence
 
 @param marker - unique generation-scoped marker
 
 @param remainingWork - actionable task-only denial guidance
 
 @returns Pi custom message
 
 @example
 ```ts
 buildGoalMessage({ goal, kind: 'continuation', continuationSequence: 1, marker, remainingWork: 'Run tests.' });
 ```
 */
function buildGoalMessage(
  {
    goal,
    kind,
    continuationSequence,
    marker,
    remainingWork,
  }: {
    readonly goal: ActiveGoalState;
    readonly kind: 'kickoff' | 'continuation';
    readonly continuationSequence: number;
    readonly marker: GoalMessageMarker;
    readonly remainingWork?: string;
  },
): GoalMessage {
  return {
    customType: GOAL_MESSAGE_TYPE,
    content: goalMessageContent({
      goal,
      kind,
      ...(remainingWork === undefined ? {} : { remainingWork, }),
    },),
    display: true,
    details: {
      runId: goal.runId,
      generationId: goal.generationId,
      continuationSequence,
      marker,
      kind,
    },
  };
}

//endregion Task-only message construction

export {
  buildGoalMessage,
  goalMessageContent,
};
