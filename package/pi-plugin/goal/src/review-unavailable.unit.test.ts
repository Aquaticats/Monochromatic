/**
 Built-artifact tests for settlement-review exhaustion behavior.
 
 @module
 */

import type { ExtensionContext, } from '@earendil-works/pi-coding-agent';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { ReviewUnavailableError, } from '@monochromatic-dev/pi-shared-model-review/ts';

import {
  createGoalReviewerUnavailableHandler,
  reduceGoalEvents,
  type GoalControllerState,
  type GoalEffect,
  type GoalLifecycleHandle,
  type GoalSettlementReviewRequest,
  type ManualGoalReviewDecision,
} from '../dist/final/node/index.mjs';

/** Stable event timestamp. */
const STARTED_AT = '2026-08-26T00:00:00.000Z';

/** Stable fallback timestamp. */
const FALLBACK_AT = '2026-08-26T00:01:00.000Z';

/** Pi modes exercised by fallback tests. */
type GoalTestMode = 'tui' | 'rpc' | 'json' | 'print';

/** Shared exhausted-review error. */
const REVIEW_ERROR = new ReviewUnavailableError({
  attemptedCandidateIdentities: ['review/one', 'review/two',],
  diagnostics: ['review/one: timeout', 'review/two: malformed verdict',],
},);

/**
 Build captured active settlement.
 
 @returns active settlement request
 */
function fallbackRequest(): GoalSettlementReviewRequest {
  /** Reduced active goal fixture. */
  const goal = reduceGoalEvents([{
    kind: 'run_started',
    runId: 'run-1',
    generationId: 'generation-1',
    objective: 'Ship reviewed feature',
    startedAt: STARTED_AT,
    startBoundary: 'leaf-before-start',
    continuationSequence: 0,
    transitionedAt: STARTED_AT,
  },],);
  if (goal.phase !== 'active')
    throw new Error('expected active fallback fixture',);
  return {
    goal,
    runtimeEpoch: 'runtime-1',
    branchLeafId: 'leaf-current',
    settlementSequence: 0,
  };
}

/**
 Build fallback lifecycle harness.
 
 @param mode - Pi mode under test
 
 @returns mutable state, context, lifecycle, and effects
 */
function fallbackHarness(mode: GoalTestMode,): {
  readonly state: { value: GoalControllerState; };
  readonly leaf: { value: string; };
  readonly context: ExtensionContext;
  readonly lifecycle: GoalLifecycleHandle;
  readonly effects: GoalEffect[];
} {
  /** Current controller cursor. */
  const state: { value: GoalControllerState; } = {
    value: {
      goal: fallbackRequest().goal,
      runtimeEpoch: 'runtime-1',
      settlementSequence: 0,
      shutdown: false,
    } satisfies GoalControllerState,
  };
  /** Current selected branch leaf. */
  const leaf = { value: 'leaf-current', };
  /** Applied transition effects. */
  const effects: GoalEffect[] = [];
  /** Fake lifecycle seam. */
  const lifecycle: GoalLifecycleHandle = {
    currentController() {
      return state.value;
    },
    applyTransition({ transition, }) {
      state.value = transition.controller;
      effects.push(...transition.effects,);
    },
    deliverPendingKickoff() {
      return false;
    },
  };
  /** Fake Pi context with selected leaf. */
  const context = {
    mode,
    isIdle() {
      return true;
    },
    hasPendingMessages() {
      return false;
    },
    sessionManager: {
      getLeafId() {
        return leaf.value;
      },
    },
  } as unknown as ExtensionContext;
  return {
    state,
    leaf,
    context,
    lifecycle,
    effects,
  };
}

await describe({
  name: createGoalReviewerUnavailableHandler.name,
  children: [
    ...(['rpc', 'json', 'print',] as const).map(function nonInteractiveMode(mode,) {
      return it({
        name: `terminates ${mode} mode as review unavailable`,
        fn: async () => {
          const harness = fallbackHarness(mode,);
          const handler = createGoalReviewerUnavailableHandler({
            lifecycle: harness.lifecycle,
            now() {
              return FALLBACK_AT;
            },
          },);
          const result = await handler({
            error: REVIEW_ERROR,
            request: fallbackRequest(),
            context: harness.context,
          },);
          expect(result,).toBe('review_unavailable',);
          expect(harness.state.value.goal.phase,).toBe('review_unavailable',);
          expect(harness.effects.some(function isUnavailableDiagnostic(effect,) {
            return effect.type === 'persist_review_unavailable_diagnostic';
          },),).toBe(true,);
        },
      },);
    },),
    it({
      name: 'persists manual approval and human-only completion audit',
      fn: async () => {
        const harness = fallbackHarness('tui',);
        const handler = createGoalReviewerUnavailableHandler({
          lifecycle: harness.lifecycle,
          async promptManualReview(): Promise<ManualGoalReviewDecision> {
            return { action: 'accept', };
          },
          now() {
            return FALLBACK_AT;
          },
        },);
        const result = await handler({
          error: REVIEW_ERROR,
          request: fallbackRequest(),
          context: harness.context,
        },);
        expect(result,).toBe('approved',);
        expect(harness.state.value.goal.phase,).toBe('completed',);
        expect(harness.effects.some(function isCompletionDiagnostic(effect,) {
          return effect.type === 'persist_completion_diagnostic';
        },),).toBe(true,);
      },
    },),
    it({
      name: 'continues manual rejection with task-only reason or fallback',
      fn: async () => {
        /** Reasoned rejection harness. */
        const reasoned = fallbackHarness('tui',);
        const reasonedHandler = createGoalReviewerUnavailableHandler({
          lifecycle: reasoned.lifecycle,
          async promptManualReview(): Promise<ManualGoalReviewDecision> {
            return { action: 'reject', reason: 'Run the integration test.', };
          },
          createId() {
            return 'reasoned-marker';
          },
          now() {
            return FALLBACK_AT;
          },
        },);
        expect(await reasonedHandler({
          error: REVIEW_ERROR,
          request: fallbackRequest(),
          context: reasoned.context,
        },),).toBe('continued',);
        const reasonedMessage = reasoned.effects.find(function isMessageEffect(effect,) {
          return effect.type === 'send_message';
        },);
        if (reasonedMessage?.type !== 'send_message')
          throw new Error('reasoned manual rejection omitted task message',);
        expect(reasonedMessage.message.content,).toBe('Run the integration test.',);
        /** Empty rejection fallback harness. */
        const empty = fallbackHarness('tui',);
        const emptyHandler = createGoalReviewerUnavailableHandler({
          lifecycle: empty.lifecycle,
          async promptManualReview(): Promise<ManualGoalReviewDecision> {
            return { action: 'reject', reason: ' ', };
          },
          createId() {
            return 'empty-marker';
          },
          now() {
            return FALLBACK_AT;
          },
        },);
        await emptyHandler({
          error: REVIEW_ERROR,
          request: fallbackRequest(),
          context: empty.context,
        },);
        const emptyMessage = empty.effects.find(function isMessageEffect(effect,) {
          return effect.type === 'send_message';
        },);
        if (emptyMessage?.type !== 'send_message')
          throw new Error('empty manual rejection omitted fallback task message',);
        expect(emptyMessage.message.content,).toBe(
          'Continue working on the current user objective.',
        );
      },
    },),
    it({
      name: 'ignores stale result after manual dialog changes branch',
      fn: async () => {
        const harness = fallbackHarness('tui',);
        const handler = createGoalReviewerUnavailableHandler({
          lifecycle: harness.lifecycle,
          async promptManualReview(): Promise<ManualGoalReviewDecision> {
            harness.leaf.value = 'changed-leaf';
            return { action: 'accept', };
          },
        },);
        const result = await handler({
          error: REVIEW_ERROR,
          request: fallbackRequest(),
          context: harness.context,
        },);
        expect(result,).toBe('stale',);
        expect(harness.state.value.goal.phase,).toBe('active',);
      },
    },),
  ],
},);
