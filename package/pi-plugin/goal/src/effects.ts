/**
 Pi runtime adapter for semantic goal transition effects.
 
 @module
 */

import type {
  ExtensionAPI,
  ExtensionContext,
} from '@earendil-works/pi-coding-agent';
import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  GOAL_COMPLETION_ENTRY_TYPE,
  GOAL_REVIEW_UNAVAILABLE_ENTRY_TYPE,
  GOAL_STATE_ENTRY_TYPE,
  GOAL_STATUS_KEY,
} from './constants.ts';
import type { GoalEffect, } from './types.ts';

/**
 Root goal extension logger.
 */
const logger: Logger = tagged({ tag: 'pi-goal', },);

/**
 Apply ordered goal effects at Pi's mutation boundary.
 
 @param effects - semantic effects from pure controller transition
 
 @param pi - Pi extension API owning session writes and message sends
 
 @param context - current lifecycle context owning UI footer and notifications
 
 @mutates pi - persistence and custom-message effects append session records
 
 @mutates context - footer and notification effects update Pi UI state
 
 @example
 ```ts
 applyGoalEffects({ effects, pi, context });
 ```
 */
function applyGoalEffects(
  {
    effects,
    pi,
    context,
  }: {
    readonly effects: readonly GoalEffect[];
    readonly pi: ForeignBorrowed<ExtensionAPI>;
    readonly context: ForeignBorrowed<ExtensionContext>;
  },
): void {
  for (const effect of effects) {
    if (effect.type === 'persist') {
      pi.appendEntry(
        GOAL_STATE_ENTRY_TYPE,
        effect.event,
      );
      continue;
    }
    if (effect.type === 'set_footer') {
      context.ui
        .setStatus(
        GOAL_STATUS_KEY,
        effect.text,
        );
      continue;
    }
    if (effect.type === 'clear_footer') {
      context.ui
        .setStatus(
        GOAL_STATUS_KEY,
        undefined,
        );
      continue;
    }
    if (effect.type === 'send_message') {
      pi.sendMessage(
        effect.message,
        { triggerTurn: effect.triggerTurn, },
      );
      continue;
    }
    if (effect.type === 'persist_review_unavailable_diagnostic') {
      pi.appendEntry(
        GOAL_REVIEW_UNAVAILABLE_ENTRY_TYPE,
        effect.diagnostic,
      );
      continue;
    }
    if (effect.type === 'persist_completion_diagnostic') {
      pi.appendEntry(
        GOAL_COMPLETION_ENTRY_TYPE,
        effect.diagnostic,
      );
      continue;
    }
    if (effect.type === 'notify') {
      context.ui
        .notify(
        effect.message,
        effect.level,
        );
      continue;
    }
    if (effect.level === 'debug') {
      logger.debug(effect.message,);
      continue;
    }
    if (effect.level === 'warn') {
      logger.warn(effect.message,);
      continue;
    }
    logger.error(effect.message,);
  }
}

export {
  applyGoalEffects,
  logger,
};
