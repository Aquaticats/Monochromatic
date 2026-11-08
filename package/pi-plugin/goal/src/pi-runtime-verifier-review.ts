/**
 Built-artifact settlement-review checks with deterministic fake reviewer.
 
 @module
 */

import type { ExtensionContext, } from '@earendil-works/pi-coding-agent';

import {
  createGoalController,
  executeGoalSettlementReview,
  startGoal,
  type GoalControllerState,
  type GoalLifecycleHandle,
  type GoalSettlementReviewRequest,
} from '../dist/final/node/index.mjs';

/**
 Deterministic timestamp for built review verification.
 */
const REVIEW_TIMESTAMP = '2026-08-26T00:00:00.000Z';

/**
 Review outcome fixture returned by deterministic fake transport.
 */
type ReviewFixture = {
  readonly approved: boolean;
  readonly expectedOutcome: 'approved' | 'continued';
  readonly rationale: string;
  readonly remainingWork: string;
};

/**
 Dead-end denial guidance produced by the measured Collatz runs.
 */
const UNREACHABLE_USER_GUIDANCE = 'Explain that a proof is currently unknown and ask the user which alternative task they want.';

/**
 Build active controller and settlement identity.
 
 @param mode - Pi session mode deciding human reachability
 
 @returns controller cell, lifecycle adapter, context, request, and continuation text
 */
function createReviewFixture(mode: 'tui' | 'rpc' = 'rpc',): {
  readonly state: { value: GoalControllerState; };
  readonly lifecycle: GoalLifecycleHandle;
  readonly context: ExtensionContext;
  readonly request: GoalSettlementReviewRequest;
  readonly messages: string[];
} {
  /**
   Pure start transition supplying active state.
   */
  const started = startGoal({
    controller: createGoalController('runtime-review',),
    objective: 'Verify fake reviewer transport',
    runId: 'run-review',
    generationId: 'generation-review',
    startBoundary: 'leaf-before-review',
    marker: 'marker-review',
    timestamp: REVIEW_TIMESTAMP,
    isIdle: true,
    hasPendingMessages: false,
  },);
  /**
   Runtime-owned controller cell.
   */
  const state = { value: started.controller, };
  /**
   Continuation text emitted by denial transitions.
   */
  const messages: string[] = [];
  /**
   Lifecycle capability consumed by settlement implementation.
   */
  const lifecycle: GoalLifecycleHandle = {
    currentController() {
      return state.value;
    },
    applyTransition({ transition, },) {
      state.value = transition.controller;
      for (const effect of transition.effects) {
        if (effect.type !== 'send_message')
          continue;
        /**
         Emitted task message.
         */
        const { message, } = effect;
        messages.push(message.content,);
      }
    },
    deliverPendingKickoff() {
      return false;
    },
  };
  /* oxlint-disable typescript/no-unsafe-type-assertion -- Disposable verifier implements only context members consumed by settlement review. */
  /**
   Focused context for stale-result leaf validation.
   */
  const context = {
    mode,
    hasUI: false,
    isIdle() {
      return true;
    },
    hasPendingMessages() {
      return false;
    },
    sessionManager: {
      getLeafId() {
        return 'leaf-current-review';
      },
    },
  } as unknown as ExtensionContext;
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  /**
   Active started goal after transition validation.
   */
  const { goal, } = started.controller;
  if (goal.phase !== 'active')
    throw new Error('review fixture did not start active goal',);
  /**
   Captured active settlement request.
   */
  const request: GoalSettlementReviewRequest = {
    goal,
    runtimeEpoch: 'runtime-review',
    branchLeafId: 'leaf-current-review',
    settlementSequence: started.controller
      .settlementSequence,
  };
  return {
    state,
    lifecycle,
    context,
    request,
    messages,
  };
}

/**
 Execute one built settlement path through fake reviewer.
 
 @param approved - fake structured reviewer decision
 
 @param expectedOutcome - expected harness disposition
 
 @param rationale - private reviewer rationale
 
 @param remainingWork - task-only denial guidance
 
 @returns verified disposition
 
 @throws when result or state transition differs
 
 @example
 ```ts
 await verifyReviewOutcome({ approved: true, expectedOutcome: 'approved', rationale: 'complete', remainingWork: '' });
 ```
 */
async function verifyReviewOutcome(
  {
    approved,
    expectedOutcome,
    rationale,
    remainingWork,
  }: ReviewFixture,
): Promise<string> {
  /**
   Active deterministic review fixture.
   */
  const {
    state,
    lifecycle,
    context,
    request,
  } = createReviewFixture();
  /**
   Built settlement result after deterministic fake reviewer.
   */
  const result = await executeGoalSettlementReview({
    request,
    context,
    lifecycle,
    reviewer() {
      return Promise.resolve({
        verdict: {
          approved,
          rationale,
          remainingWork,
        },
        reviewerIdentity: 'fake-reviewer/distinct-model',
        attemptedReviewerIdentities: ['fake-reviewer/distinct-model',],
        transcriptTruncated: false,
      },);
    },
    handleReviewerUnavailable() {
      return Promise.reject(new Error('fake reviewer unexpectedly became unavailable',),);
    },
    createId() {
      return 'fake-continuation-marker';
    },
    now() {
      return REVIEW_TIMESTAMP;
    },
  },);
  if (result !== expectedOutcome)
    throw new Error(`expected ${expectedOutcome}, received ${result}`,);
  /**
   Expected state phase after reviewer decision.
   */
  const expectedPhase = approved ? 'completed' : 'active';
  if (state.value
    .goal
    .phase
    !== expectedPhase) {
    throw new Error(`expected goal phase ${expectedPhase}, received ${state.value
      .goal
      .phase}`,);
  }
  return expectedOutcome;
}

/**
 Verify that a noninteractive denial keeps the primary model at work.
 
 @returns unreachable-user scenario summary
 
 @throws when dead-end denial guidance reaches the primary model
 
 @example
 ```ts
 await verifyUnreachableUserGuidance();
 ```
 */
async function verifyUnreachableUserGuidance(): Promise<string> {
  /**
   RPC settlement fixture without a reachable person.
   */
  const fixture = createReviewFixture('rpc',);
  /**
   Built settlement result after dead-end denial guidance.
   */
  const result = await executeGoalSettlementReview({
    request: fixture.request,
    context: fixture.context,
    lifecycle: fixture.lifecycle,
    reviewer() {
      return Promise.resolve({
        verdict: {
          approved: false,
          rationale: 'No proof is present.',
          remainingWork: UNREACHABLE_USER_GUIDANCE,
        },
        reviewerIdentity: 'fake-reviewer/distinct-model',
        attemptedReviewerIdentities: ['fake-reviewer/distinct-model',],
        transcriptTruncated: false,
      },);
    },
    handleReviewerUnavailable() {
      return Promise.reject(new Error('fake reviewer unexpectedly became unavailable',),);
    },
    createId() {
      return 'fake-dead-end-marker';
    },
    now() {
      return REVIEW_TIMESTAMP;
    },
  },);
  if (result !== 'continued')
    throw new Error(`expected continued RPC denial, received ${result}`,);
  /**
   Continuation text handed to the primary model.
   */
  const [continuation,] = fixture.messages;
  if (continuation === undefined)
    throw new Error('RPC denial emitted no continuation',);
  if (!continuation.includes('concrete next step',)) {
    throw new Error(`RPC continuation names no concrete next step: ${continuation}`,);
  }
  if (continuation.includes('ask the user',)) {
    throw new Error(`RPC continuation asks an unreachable user: ${continuation}`,);
  }
  return 'rpc denial kept work going';
}

/**
 Verify fake denial and approval through built artifact.
 
 @returns review scenario summary
 
 @example
 ```ts
 await verifyInjectedReviewerOutcomes();
 ```
 */
async function verifyInjectedReviewerOutcomes(): Promise<string> {
  /**
   Parallel independent denial and approval outcomes.
   */
  const outcomes = await Promise.all([
    verifyReviewOutcome({
      approved: false,
      expectedOutcome: 'continued',
      rationale: 'Verification absent.',
      remainingWork: 'Add missing verification evidence.',
    },),
    verifyReviewOutcome({
      approved: true,
      expectedOutcome: 'approved',
      rationale: 'Objective independently verified.',
      remainingWork: '',
    },),
  ],);
  /**
   Noninteractive dead-end denial scenario.
   */
  const unreachable = await verifyUnreachableUserGuidance();
  return `fake reviewer ${outcomes.join(' and ')}; ${unreachable}`;
}

export { verifyInjectedReviewerOutcomes, };
