/**
 Pi lifecycle registration for command, restoration, prompting, and continuation.
 
 @module
 */

import type {
  BeforeAgentStartEvent,
  BeforeAgentStartEventResult,
  ExtensionAPI,
  ExtensionCommandContext,
  ExtensionContext,
  SessionCompactEvent,
  SessionShutdownEvent,
  SessionStartEvent,
  SessionTreeEvent,
} from '@earendil-works/pi-coding-agent';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { parseGoalCommand, } from './command.ts';
import {
  clearGoal,
  createGoalController,
  restoreGoalController,
  rotateGoalGeneration,
  startGoal,
} from './controller.ts';
import { applyGoalEffects, } from './effects.ts';
import { goalEventsFromBranch, } from './events.ts';
import {
  defaultCreateId,
  defaultNow,
  type GoalLifecycleHandle,
  type GoalLifecycleServices,
} from './lifecycle-services.ts';
import { buildActiveGoalPrompt, } from './prompt.ts';
import { reduceGoalEvents, } from './reducer.ts';
import {
  deliverPendingGoalKickoff,
  shutdownGoalController,
} from './settlement.ts';
import type {
  GoalControllerState,
  GoalControllerTransition,
  GoalGenerationRotatedEvent,
} from './types.ts';

/**
 Register goal state lifecycle without model-completion review tool.
 
 @param pi - Pi extension API receiving command and lifecycle registrations
 
 @param services - injectable identity and clock boundaries
 
 @returns shared runtime boundary for completion-review registration
 
 @mutates pi - registers handlers and writes effects when handlers execute
 
 @example
 ```ts
 const lifecycle = registerGoalLifecycle({ pi });
 ```
 */
function registerGoalLifecycle(
  {
    pi,
    services = {
      createId: defaultCreateId,
      now: defaultNow,
    },
  }: {
    readonly pi: ForeignBorrowed<ExtensionAPI>;
    readonly services?: GoalLifecycleServices;
  },
): GoalLifecycleHandle {
  /**
   Runtime identity invalidating callbacks from prior extension instances.
   */
  const runtimeEpoch = services.createId();
  /**
   Controller ownership belongs to this single extension runtime closure.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-function-root-let -- Pi lifecycle handlers share one runtime-owned immutable-state cursor.
  let controller = createGoalController(runtimeEpoch,);
  /**
   Store next immutable controller and execute its ordered effects.
   
   @param transition - pure controller transition
   
   @param context - current Pi lifecycle context
   
   @mutates context - delegates footer and notification effects through Pi UI
   */
  function applyTransition(
    {
      transition,
      context,
    }: {
      readonly transition: GoalControllerTransition;
      readonly context: ForeignBorrowed<ExtensionContext>;
    },
  ): void {
    /**
     Next immutable controller cursor and ordered effects.
     */
    const {
      controller: nextController,
      effects,
    } = transition;
    controller = nextController;
    applyGoalEffects({
      effects,
      pi,
      context,
    },);
  }

  /**
   Read current immutable controller snapshot.
   
   @returns current controller
   */
  function currentController(): GoalControllerState {
    return controller;
  }

  /**
   Restore only current branch's persisted goal events.
   
   @param context - lifecycle context exposing selected session branch
   
   @mutates context - restores footer state without starting model turn
   */
  function restoreBranch(context: ForeignBorrowed<ExtensionContext>,): void {
    /**
     Selected-branch entries from Pi session manager.
     */
    const branch = context.sessionManager
      .getBranch();
    /**
     Valid goal events from current active branch only.
     */
    const events = goalEventsFromBranch(branch,);
    /**
     Branch-reduced goal state.
     */
    const goal = reduceGoalEvents(events,);
    applyTransition({
      transition: restoreGoalController({
        controller,
        goal,
      },),
      context,
    },);
  }

  /**
   Rotate restored active generation without triggering model turn.
   
   @param context - lifecycle context receiving persistence and footer effects
   
   @param cause - lifecycle boundary invalidating delayed callbacks
   
   @mutates context - refreshes active footer
   */
  function rotateRestoredGeneration(
    {
      context,
      cause,
    }: {
      readonly context: ForeignBorrowed<ExtensionContext>;
      readonly cause: GoalGenerationRotatedEvent['cause'];
    },
  ): void {
    applyTransition({
      transition: rotateGoalGeneration({
        controller,
        generationId: services.createId(),
        timestamp: services.now(),
        cause,
      },),
      context,
    },);
  }

  /**
   Handle strict public goal command.
   
   @param args - command text after `/goal`
   
   @param context - command context owning UI and session state
   
   @mutates context - context.ui.notify, context.sessionManager.getLeafId, context.isIdle, and context.hasPendingMessages may change context-owned Pi state
   */
  function handleGoalCommand(
    {
      args,
      context,
    }: {
      readonly args: string;
      readonly context: ForeignBorrowed<ExtensionCommandContext>;
    },
  ): Promise<void> {
    /**
     Parsed strict public command.
     */
    const command = parseGoalCommand(args,);
    if (command.kind === 'rejected') {
      context.ui
        .notify(
        command.diagnostic,
        'error',
        );
      return Promise.resolve();
    }
    if (command.kind === 'clear') {
      applyTransition({
        transition: clearGoal({
          controller,
          timestamp: services.now(),
        },),
        context,
      },);
      return Promise.resolve();
    }
    /**
     Fresh public run identity.
     */
    const runId = services.createId();
    /**
     Current leaf identifying reviewer transcript start.
     */
    const leafId = context.sessionManager
      .getLeafId();
    /**
     Stable start boundary, including new empty sessions.
     */
    const startBoundary = leafId ?? `new-session:${runId}`;
    applyTransition({
      transition: startGoal({
        controller,
        objective: command.objective,
        runId,
        generationId: services.createId(),
        startBoundary,
        marker: services.createId(),
        timestamp: services.now(),
        isIdle: context.isIdle(),
        hasPendingMessages: context.hasPendingMessages(),
      },),
      context,
    },);
    return Promise.resolve();
  }

  /**
   Deliver deferred kickoff before settlement review.
   
   @param context - active session context
   
   @returns whether matching deferred kickoff owned this settlement
   
   @mutates context - emitted effects may append task context and update Pi UI
   */
  function deliverPendingKickoff(context: ForeignBorrowed<ExtensionContext>,): boolean {
    if (controller.pendingKickoff === undefined)
      return false;
    /**
     Pending-kickoff transition and delivery evidence.
     */
    const transition = deliverPendingGoalKickoff(controller,);
    /**
     Whether transition emitted task kickoff.
     */
    const delivered = transition.effects
      .some(function sendsKickoff(effect,) {
        return effect.type === 'send_message';
      },);
    applyTransition({
      transition,
      context,
    },);
    return delivered;
  }

  pi.registerCommand(
    'goal',
    {
      description: 'Start one persistent objective or clear current goal',
      handler: function handleRegisteredGoalCommand(
        args,
        context,
      ) {
        return handleGoalCommand({
          args,
          context,
        },);
      },
    },
  );
  pi.on(
    'session_start',
    function restoreStartedSession(
      _event: ForeignBorrowed<SessionStartEvent>,
      context: ForeignBorrowed<ExtensionContext>,
    ) {
      restoreBranch(context,);
      rotateRestoredGeneration({
        context,
        cause: 'runtime_restore',
      },);
    },
  );
  pi.on(
    'session_tree',
    function restoreNavigatedBranch(
      _event: ForeignBorrowed<SessionTreeEvent>,
      context: ForeignBorrowed<ExtensionContext>,
    ) {
      restoreBranch(context,);
      rotateRestoredGeneration({
        context,
        cause: 'tree_navigation',
      },);
    },
  );
  pi.on(
    'session_compact',
    function restoreCompactedBranch(
      _event: ForeignBorrowed<SessionCompactEvent>,
      context: ForeignBorrowed<ExtensionContext>,
    ) {
      restoreBranch(context,);
    },
  );
  pi.on(
    'session_shutdown',
    function stopGoalRuntime(
      _event: ForeignBorrowed<SessionShutdownEvent>,
      context: ForeignBorrowed<ExtensionContext>,
    ) {
      applyTransition({
        transition: shutdownGoalController(controller,),
        context,
      },);
    },
  );
  pi.on(
    'before_agent_start',
    function injectActiveGoal(
      event: ForeignBorrowed<BeforeAgentStartEvent>,
    ): BeforeAgentStartEventResult {
      /**
       Goal state captured for this exact prompt turn.
       */
      const { goal, } = controller;
      if (goal.phase !== 'active')
        return { systemPrompt: event.systemPrompt, };
      return {
        systemPrompt: `${event.systemPrompt}\n\n${buildActiveGoalPrompt(goal,)}`,
      };
    },
  );
  return {
    currentController,
    applyTransition,
    deliverPendingKickoff,
  };
}

export { registerGoalLifecycle, };
