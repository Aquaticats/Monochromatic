/**
 Repository-owned Pi goal extension and public modules.
 
 @module
 */

import type { ExtensionAPI, } from '@earendil-works/pi-coding-agent';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { registerGoalSettlementReview, } from './completion-registration.ts';
import { registerGoalLifecycle, } from './lifecycle.ts';
import { registerGoalTerminalRenderer, } from './terminal-renderer.ts';

/**
 Register repository-owned goal lifecycle, completion review, and terminal renderer.
 
 @param pi - Pi extension API receiving goal registrations
 
 @mutates pi - lifecycle, settlement review, and terminal renderer retain Pi callbacks that may update session, UI, and registration state
 
 @example
 ```ts
 pi -e ./package/pi-plugin/goal/dist/final/node/index.mjs
 ```
 */
export default function piGoal(pi: ForeignBorrowed<ExtensionAPI>,): void {
  /**
   Shared runtime controller boundary for lifecycle and completion review.
   */
  const lifecycle = registerGoalLifecycle({ pi, },);
  registerGoalSettlementReview({
    pi,
    lifecycle,
  },);
  registerGoalTerminalRenderer(pi,);
}

export * from './command.ts';
export * from './completion-outcome.ts';
export * from './completion-registration.ts';
export * from './completion.ts';
export * from './completion-terminal.ts';
export type * from './completion-types.ts';
export * from './constants.ts';
export * from './controller.ts';
export * from './effects.ts';
export * from './events.ts';
export * from './footer.ts';
export * from './lifecycle.ts';
export * from './lifecycle-services.ts';
export * from './manual-review-dialog.ts';
export * from './message.ts';
export * from './prompt.ts';
export * from './reducer.ts';
export * from './remaining-work-guidance.ts';
export * from './review-context.ts';
export * from './review-contract.ts';
export * from './review-runner.ts';
export * from './review-selection.ts';
export * from './review-unavailable.ts';
export * from './settlement.ts';
export * from './terminal-renderer.ts';
export type * from './types.ts';
