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

//region Fixture constants and contracts

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

//endregion Fixture constants and contracts

//region Fixture construction

/**
 Build active controller and settlement identity.
 
 @returns controller cell, lifecycle adapter, context, and request
 
 @example
 ```ts
 const fixture = createReviewFixture();
 ```
 */
function createReviewFixture(): {
  readonly state: { value: GoalControllerState; };
  readonly lifecycle: GoalLifecycleHandle;
  readonly context: ExtensionContext;
  readonly request: GoalSettlementReviewRequest;
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
   Lifecycle capability consumed by settlement implementation.
   */
  const lifecycle: GoalLifecycleHandle = {
    currentController() {
      return state.value;
    },
    applyTransition({ transition, },) {
      state.value = transition.controller;
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
    mode: 'rpc',
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
  };
}

//endregion Fixture construction

//region Outcome verification

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
  return `fake reviewer ${outcomes.join(' and ')}`;
}

//endregion Outcome verification

export { verifyInjectedReviewerOutcomes, };
