/**
 Built-artifact tests for private settlement review.
 
 @module
 */

import type {
  ExtensionAPI,
  ExtensionContext,
} from '@earendil-works/pi-coding-agent';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  createGoalController,
  createGoalSettlementReviewRequest,
  executeGoalSettlementReview,
  GOAL_SETTLEMENT_NOT_REVIEWABLE,
  reduceGoalEvents,
  registerGoalSettlementReview,
  type ActiveGoalState,
  type GoalControllerState,
  type GoalLifecycleHandle,
  type GoalSettlementReviewRequest,
} from '../dist/final/node/index.mjs';

/** Stable start timestamp. */
const STARTED_AT = '2026-08-26T00:00:00.000Z';

/** Stable completion timestamp. */
const COMPLETED_AT = '2026-08-26T00:01:00.000Z';

/**
 Build active settlement-test goal.
 
 @returns active goal state
 */
function completionGoal(): ActiveGoalState {
  /** Reduced active fixture. */
  const goal = reduceGoalEvents([{
    kind: 'run_started',
    runId: 'run-1',
    generationId: 'generation-1',
    objective: 'Explain why 67 is prime in five ways',
    startedAt: STARTED_AT,
    startBoundary: 'leaf-before-start',
    continuationSequence: 0,
    transitionedAt: STARTED_AT,
  },],);
  if (goal.phase !== 'active')
    throw new Error('expected active completion fixture',);
  return goal;
}

/**
 Build active controller fixture.
 
 @returns active controller
 */
function completionController(): GoalControllerState {
  return {
    goal: completionGoal(),
    runtimeEpoch: 'runtime-1',
    settlementSequence: 0,
    shutdown: false,
  };
}

/**
 Build captured settlement request.
 
 @returns request bound to active controller identities
 */
function settlementRequest(): GoalSettlementReviewRequest {
  return {
    goal: completionGoal(),
    runtimeEpoch: 'runtime-1',
    branchLeafId: 'leaf-current',
    settlementSequence: 0,
  };
}

/**
 Build mutable fake lifecycle and context.
 
 @returns lifecycle, context, and transition observations
 */
function executionHarness(): {
  readonly lifecycle: GoalLifecycleHandle;
  readonly context: ExtensionContext;
  readonly transitions: GoalControllerState[];
  readonly messages: unknown[];
  readonly leaf: { value: string; };
  readonly pending: { value: boolean; };
} {
  /** Runtime controller cursor. */
  const state = { value: completionController(), };
  /** Selected branch leaf cursor. */
  const leaf = { value: 'leaf-current', };
  /** Whether human input owns next turn. */
  const pending = { value: false, };
  /** Applied controller states. */
  const transitions: GoalControllerState[] = [];
  /** Task messages emitted by transition effects. */
  const messages: unknown[] = [];
  /** Fake lifecycle boundary. */
  const lifecycle: GoalLifecycleHandle = {
    currentController() {
      return state.value;
    },
    applyTransition({ transition, }) {
      state.value = transition.controller;
      transitions.push(transition.controller,);
      for (const effect of transition.effects) {
        if (effect.type === 'send_message')
          messages.push(effect.message,);
      }
    },
    deliverPendingKickoff() {
      return false;
    },
  };
  /** Fake Pi context exposing selected leaf. */
  const context = {
    mode: 'tui',
    isIdle() {
      return true;
    },
    hasPendingMessages() {
      return pending.value;
    },
    sessionManager: {
      getLeafId() {
        return leaf.value;
      },
    },
  } as unknown as ExtensionContext;
  return {
    lifecycle,
    context,
    transitions,
    messages,
    leaf,
    pending,
  };
}

await describe({
  name: executeGoalSettlementReview.name,
  children: [
    it({
      name: 'continues denial with task-only guidance',
      fn: async () => {
        const harness = executionHarness();
        const outcome = await executeGoalSettlementReview({
          request: settlementRequest(),
          context: harness.context,
          lifecycle: harness.lifecycle,
          async reviewer() {
            return {
              verdict: {
                approved: false,
                rationale: 'Integration test absent.',
                remainingWork: 'Add and run the integration test.',
              },
              reviewerIdentity: 'review/model',
              attemptedReviewerIdentities: ['review/model',],
              transcriptTruncated: false,
            };
          },
          async handleReviewerUnavailable() {
            throw new Error('unexpected unavailable fallback',);
          },
          createId() {
            return 'continuation-marker';
          },
          now() {
            return COMPLETED_AT;
          },
        },);
        expect(outcome,).toBe('continued',);
        expect(harness.transitions[0]?.goal.phase,).toBe('active',);
        expect(harness.messages,).toHaveLength(1,);
        expect(harness.messages[0],).toMatchObject({
          content: 'Add and run the integration test.',
        },);
      },
    },),
    it({
      name: 'ignores approval when human input becomes pending during review',
      fn: async () => {
        const harness = executionHarness();
        const outcome = await executeGoalSettlementReview({
          request: settlementRequest(),
          context: harness.context,
          lifecycle: harness.lifecycle,
          reviewer() {
            harness.pending.value = true;
            return Promise.resolve({
              verdict: {
                approved: true,
                rationale: 'Every requirement is supported.',
                remainingWork: '',
              },
              reviewerIdentity: 'review/model',
              attemptedReviewerIdentities: ['review/model',],
              transcriptTruncated: false,
            },);
          },
          handleReviewerUnavailable() {
            return Promise.reject(new Error('unexpected unavailable fallback',),);
          },
          createId() {
            return 'unused-marker';
          },
          now() {
            return COMPLETED_AT;
          },
        },);
        expect(outcome,).toBe('stale',);
        expect(harness.transitions,).toHaveLength(0,);
      },
    },),
    it({
      name: 'approves with durable audit and no primary message',
      fn: async () => {
        const harness = executionHarness();
        const outcome = await executeGoalSettlementReview({
          request: settlementRequest(),
          context: harness.context,
          lifecycle: harness.lifecycle,
          async reviewer() {
            return {
              verdict: {
                approved: true,
                rationale: 'Every requirement is supported.',
                remainingWork: '',
              },
              reviewerIdentity: 'review/model',
              attemptedReviewerIdentities: ['review/model',],
              transcriptTruncated: false,
            };
          },
          async handleReviewerUnavailable() {
            throw new Error('unexpected unavailable fallback',);
          },
          createId() {
            return 'unused-marker';
          },
          now() {
            return COMPLETED_AT;
          },
        },);
        expect(outcome,).toBe('approved',);
        expect(harness.transitions[0]?.goal.phase,).toBe('completed',);
        expect(harness.messages,).toHaveLength(0,);
      },
    },),
  ],
},);

await describe({
  name: createGoalSettlementReviewRequest.name,
  children: [
    it({
      name: 'captures active settlement and refuses absent state',
      fn: async () => {
        expect(createGoalSettlementReviewRequest({
          controller: completionController(),
          branchLeafId: 'leaf-current',
        },),).toEqual(settlementRequest(),);
        expect(createGoalSettlementReviewRequest({
          controller: createGoalController('runtime-1',),
          branchLeafId: 'leaf-current',
        },),).toBe(GOAL_SETTLEMENT_NOT_REVIEWABLE,);
      },
    },),
  ],
},);

await describe({
  name: registerGoalSettlementReview.name,
  children: [
    it({
      name: 'registers no tool, waits for pending human input, and reviews leaf once',
      fn: async () => {
        /** Captured handlers by event name. */
        const handlers = new Map<string, ((event: unknown, context: ExtensionContext) => unknown)[]>();
        /** Unexpected primary tool registration count. */
        const registeredTools = { value: 0, };
        /** Fake Pi registration seam. */
        const pi = {
          on(event: string, handler: (event: unknown, context: ExtensionContext) => unknown,) {
            handlers.set(event, [...(handlers.get(event,) ?? []), handler,],);
          },
          registerTool() {
            registeredTools.value += 1;
          },
        } as unknown as ExtensionAPI;
        /** Mutable controller state observed by lifecycle. */
        const state = { value: completionController(), };
        /** Review call count. */
        const reviews = { value: 0, };
        /** Fake lifecycle seam. */
        const lifecycle: GoalLifecycleHandle = {
          currentController() {
            return state.value;
          },
          applyTransition({ transition, }) {
            state.value = transition.controller;
          },
          deliverPendingKickoff() {
            return false;
          },
        };
        registerGoalSettlementReview({
          pi,
          lifecycle,
          async reviewer() {
            reviews.value += 1;
            return {
              verdict: {
                approved: true,
                rationale: 'Complete.',
                remainingWork: '',
              },
              reviewerIdentity: 'review/model',
              attemptedReviewerIdentities: ['review/model',],
              transcriptTruncated: false,
            };
          },
          async handleReviewerUnavailable() {
            throw new Error('unexpected unavailable fallback',);
          },
          createId() {
            return 'fixture-id';
          },
          now() {
            return COMPLETED_AT;
          },
        },);
        /** Whether human input already owns next turn. */
        const pending = { value: true, };
        /** Focused final-settlement context. */
        const context = {
          isIdle() {
            return true;
          },
          hasPendingMessages() {
            return pending.value;
          },
          sessionManager: {
            getLeafId() {
              return 'leaf-current';
            },
          },
        } as unknown as ExtensionContext;
        const settled = handlers.get('agent_settled',)?.[0];
        if (settled === undefined)
          throw new Error('agent_settled handler missing',);
        await settled({ type: 'agent_settled', }, context,);
        expect(reviews.value,).toBe(0,);
        pending.value = false;
        await settled({ type: 'agent_settled', }, context,);
        await settled({ type: 'agent_settled', }, context,);
        expect(registeredTools.value,).toBe(0,);
        expect(reviews.value,).toBe(1,);
        expect(state.value.goal.phase,).toBe('completed',);
      },
    },),
  ],
},);
