/**
 Immutable goal controller transitions and semantic effects.
 
 @module
 */

import { formatGoalFooter, } from './footer.ts';
import { buildGoalMessage, } from './message.ts';
import {
  ABSENT_GOAL_STATE,
  reduceGoalEvent,
} from './reducer.ts';
import type {
  ActiveGoalState,
  GoalControllerState,
  GoalControllerTransition,
  GoalGenerationId,
  GoalGenerationRotatedEvent,
  GoalMessageMarker,
  GoalRunId,
  GoalRuntimeEpoch,
  GoalStartBoundary,
  GoalState,
} from './types.ts';

//region Construction

/**
 Create fresh controller for one extension runtime epoch.
 
 @param runtimeEpoch - runtime-instance identity
 
 @returns absent live controller
 
 @example
 ```ts
 const controller = createGoalController('runtime-1');
 ```
 */
function createGoalController(runtimeEpoch: GoalRuntimeEpoch,): GoalControllerState {
  return {
    goal: ABSENT_GOAL_STATE,
    runtimeEpoch,
    settlementSequence: 0,
    shutdown: false,
  };
}

/**
 Restore exact selected-branch state without triggering a model turn.
 
 @param controller - current runtime controller
 
 @param goal - reduced selected-branch state
 
 @returns restored controller and footer effects
 
 @example
 ```ts
 restoreGoalController({ controller, goal });
 ```
 */
function restoreGoalController(
  {
    controller,
    goal,
  }: {
    readonly controller: GoalControllerState;
    readonly goal: GoalState;
  },
): GoalControllerTransition {
  return {
    controller: {
      goal,
      runtimeEpoch: controller.runtimeEpoch,
      settlementSequence: controller.settlementSequence,
      shutdown: false,
    },
    effects: goal.phase === 'active'
      ? [{
        type: 'set_footer',
        text: formatGoalFooter(goal.objective,),
      },]
      : [{ type: 'clear_footer', },],
  };
}

//endregion Construction

//region Start

/**
 Start new run or atomically supersede active or terminal record.
 
 @param controller - current controller
 
 @param objective - exact normalized objective
 
 @param runId - fresh run identity
 
 @param generationId - fresh generation identity
 
 @param startBoundary - stable reviewer start marker
 
 @param marker - unique kickoff message marker
 
 @param timestamp - ISO transition timestamp
 
 @param isIdle - whether Pi can start turn now
 
 @param hasPendingMessages - whether human input already owns next turn
 
 @returns next controller with persist, footer, and kickoff effects
 
 @example
 ```ts
 startGoal({ controller, objective, runId, generationId, startBoundary, marker, timestamp, isIdle: true, hasPendingMessages: false });
 ```
 */
function startGoal(
  {
    controller,
    objective,
    runId,
    generationId,
    startBoundary,
    marker,
    timestamp,
    isIdle,
    hasPendingMessages,
  }: {
    readonly controller: GoalControllerState;
    readonly objective: string;
    readonly runId: GoalRunId;
    readonly generationId: GoalGenerationId;
    readonly startBoundary: GoalStartBoundary;
    readonly marker: GoalMessageMarker;
    readonly timestamp: string;
    readonly isIdle: boolean;
    readonly hasPendingMessages: boolean;
  },
): GoalControllerTransition {
  /**
   Existing run superseded atomically by start event.
   */
  const supersededRunId = controller.goal
    .phase
    === 'absent'
    ? undefined
    : controller.goal
      .runId;
  /**
   Persisted atomic start event.
   */
  const event = {
    kind: 'run_started',
    runId,
    generationId,
    objective,
    startedAt: timestamp,
    startBoundary,
    continuationSequence: 0,
    transitionedAt: timestamp,
    ...(supersededRunId === undefined ? {} : { supersededRunId, }),
  } as const;
  /**
   Active state derived through same reducer used during reconstruction.
   */
  const goal = reduceGoalEvent({
    state: controller.goal,
    event,
  },);
  if (goal.phase !== 'active')
    throw new Error('Goal start event did not reduce to active state',);
  /**
   Visible kickoff custom message.
   */
  const message = buildGoalMessage({
    goal,
    kind: 'kickoff',
    continuationSequence: 0,
    marker,
  },);
  /**
   Whether kickoff may enter Pi immediately.
   */
  const sendImmediately = isIdle && (!hasPendingMessages);
  return {
    controller: {
      goal,
      runtimeEpoch: controller.runtimeEpoch,
      settlementSequence: controller.settlementSequence,
      shutdown: false,
      ...(sendImmediately
        ? {}
        : {
          pendingKickoff: {
            runId,
            generationId,
            runtimeEpoch: controller.runtimeEpoch,
            marker,
          },
        }),
    },
    effects: [
      ...(controller.goal
        .phase
        === 'absent'
        ? []
        : [{ type: 'clear_footer' as const, },]),
      {
        type: 'persist',
        event,
      },
      {
        type: 'set_footer',
        text: formatGoalFooter(objective,),
      },
      ...(sendImmediately
        ? [{
          type: 'send_message' as const,
          message,
          triggerTurn: true as const,
        },]
        : []),
      {
        type: 'log',
        level: 'debug',
        message: supersededRunId === undefined
          ? `started goal run ${runId}`
          : `started goal run ${runId}, superseding ${supersededRunId}`,
      },
    ],
  };
}

//endregion Start

//region Rotation

/**
 Rotate active generation during runtime restoration or tree navigation.
 
 @param controller - controller restored from selected branch
 
 @param generationId - fresh generation identity
 
 @param timestamp - ISO rotation timestamp
 
 @param cause - lifecycle reason requiring stale-callback invalidation
 
 @returns rotation persistence and footer effects, or terminal no-op
 
 @example
 ```ts
 rotateGoalGeneration({ controller, generationId, timestamp, cause: 'runtime_restore' });
 ```
 */
function rotateGoalGeneration(
  {
    controller,
    generationId,
    timestamp,
    cause,
  }: {
    readonly controller: GoalControllerState;
    readonly generationId: GoalGenerationId;
    readonly timestamp: string;
    readonly cause: GoalGenerationRotatedEvent['cause'];
  },
): GoalControllerTransition {
  if (controller.goal
    .phase
    !== 'active') {
    return {
      controller,
      effects: [],
    };
  }
  /**
   Current active state before rotation.
   */
  const previous = controller.goal;
  /**
   Persisted generation rotation.
   */
  const event: GoalGenerationRotatedEvent = {
    kind: 'generation_rotated',
    runId: previous.runId,
    previousGenerationId: previous.generationId,
    generationId,
    continuationSequence: previous.continuationSequence,
    transitionedAt: timestamp,
    cause,
  };
  /**
   Active state with fresh generation.
   */
  const goal = reduceGoalEvent({
    state: previous,
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
        type: 'set_footer',
        text: formatGoalFooter(previous.objective,),
      },
      {
        type: 'log',
        level: 'debug',
        message: `rotated goal generation for ${previous.runId} after ${cause}`,
      },
    ],
  };
}

//endregion Rotation

//region Clear

/**
 Clear current active or terminal record without aborting Pi turn.
 
 @param controller - current controller
 
 @param timestamp - ISO clear timestamp
 
 @returns clear tombstone or idempotent informational no-op
 
 @example
 ```ts
 clearGoal({ controller, timestamp });
 ```
 */
function clearGoal(
  {
    controller,
    timestamp,
  }: {
    readonly controller: GoalControllerState;
    readonly timestamp: string;
  },
): GoalControllerTransition {
  if (controller.goal
    .phase
    === 'absent') {
    return {
      controller: {
        goal: controller.goal,
        runtimeEpoch: controller.runtimeEpoch,
        settlementSequence: controller.settlementSequence,
        shutdown: controller.shutdown,
      },
      effects: [{
        type: 'notify',
        level: 'info',
        message: 'No goal is active.',
      },],
    };
  }
  /**
   Current record cleared by matching tombstone.
   */
  const current = controller.goal;
  /**
   Persisted clear tombstone.
   */
  const event = {
    kind: 'run_cleared',
    runId: current.runId,
    generationId: current.generationId,
    clearedAt: timestamp,
  } as const;
  return {
    controller: {
      goal: ABSENT_GOAL_STATE,
      runtimeEpoch: controller.runtimeEpoch,
      settlementSequence: controller.settlementSequence,
      shutdown: controller.shutdown,
    },
    effects: [
      {
        type: 'persist',
        event,
      },
      { type: 'clear_footer', },
      {
        type: 'notify',
        level: 'info',
        message: `Goal cleared: ${current.objective}`,
      },
      {
        type: 'log',
        level: 'debug',
        message: `cleared goal run ${current.runId}`,
      },
    ],
  };
}

//endregion Clear

export {
  clearGoal,
  createGoalController,
  restoreGoalController,
  rotateGoalGeneration,
  startGoal,
};
