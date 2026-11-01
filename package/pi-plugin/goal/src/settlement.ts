/**
 Deferred kickoff and shutdown transitions at Pi's settlement seam.
 
 @module
 */

import { buildGoalMessage, } from './message.ts';
import type {
  GoalControllerState,
  GoalControllerTransition,
} from './types.ts';

/**
 Deliver deferred kickoff for exact active generation.
 
 Generic continuation is owned by independent settlement review.
 
 @param controller - controller at settlement time
 
 @returns next controller with at most one kickoff message
 
 @example
 ```ts
 deliverPendingGoalKickoff(controller);
 ```
 */
function deliverPendingGoalKickoff(controller: GoalControllerState,): GoalControllerTransition {
  if (controller.shutdown)
    return {
      controller,
      effects: [],
    };
  if (controller.goal
    .phase
    !== 'active') {
    return {
      controller: {
        goal: controller.goal,
        runtimeEpoch: controller.runtimeEpoch,
        settlementSequence: controller.settlementSequence,
        shutdown: controller.shutdown,
      },
      effects: [],
    };
  }
  /**
   Deferred kickoff captured while Pi was busy.
   */
  const { pendingKickoff, } = controller;
  if (pendingKickoff === undefined)
    return {
      controller,
      effects: [],
    };
  if ((pendingKickoff.runId
    !== controller.goal
      .runId)
    || (pendingKickoff.generationId
      !== controller.goal
        .generationId)
    || (pendingKickoff.runtimeEpoch !== controller.runtimeEpoch)) {
    return {
      controller: {
        goal: controller.goal,
        runtimeEpoch: controller.runtimeEpoch,
        settlementSequence: controller.settlementSequence,
        shutdown: controller.shutdown,
      },
      effects: [],
    };
  }
  /**
   Task-only kickoff rebuilt from current validated generation.
   */
  const kickoff = buildGoalMessage({
    goal: controller.goal,
    kind: 'kickoff',
    continuationSequence: controller.goal
      .continuationSequence,
    marker: pendingKickoff.marker,
  },);
  /**
   Settlement sequence identifying emitted kickoff.
   */
  const settlementSequence = controller.settlementSequence + 1;
  return {
    controller: {
      goal: controller.goal,
      runtimeEpoch: controller.runtimeEpoch,
      settlementSequence,
      lastEmittedSettlementSequence: settlementSequence,
      shutdown: controller.shutdown,
    },
    effects: [{
      type: 'send_message',
      message: kickoff,
      triggerTurn: true,
    },],
  };
}

/**
 Stop delayed goal actions before runtime replacement or quit.
 
 @param controller - current controller
 
 @returns shutdown controller and footer-clear effect
 
 @example
 ```ts
 shutdownGoalController(controller);
 ```
 */
function shutdownGoalController(controller: GoalControllerState,): GoalControllerTransition {
  return {
    controller: {
      goal: controller.goal,
      runtimeEpoch: controller.runtimeEpoch,
      settlementSequence: controller.settlementSequence,
      shutdown: true,
    },
    effects: [{ type: 'clear_footer', },],
  };
}

export {
  deliverPendingGoalKickoff,
  shutdownGoalController,
};
