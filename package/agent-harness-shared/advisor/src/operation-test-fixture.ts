/**
 Deterministic test-only provider timelines using a local clock, not shared fake timers. @module
 */
import { setImmediate as nextTurn, } from 'node:timers/promises';
import type {
  AssistantMessage,
  Usage,
} from '@earendil-works/pi-ai';
import {
  ADVISOR_CLOCK_BOUNDARY,
  runAdvisorOperation,
  type AdvisorDispatch,
  type AdvisorOperationOptions,
  type AdvisorOperationSnapshot,
} from '../dist/final/node/index.mjs';

/**
 Unsettled promises must not advance the fixture clock before scheduling boundaries are known.
 */
const NOT_READY: unique symbol = Symbol('fixture/no-ready-outcome');
/**
 One planned provider attempt.
 */
export type AttemptPlan = {
  /**
   Completion delay relative to preparation start.
   */
  readonly after: number;
  /**
   Text, including whitespace-only empty responses.
   */
  readonly text?: string;
  /**
   Terminal provider state.
   */
  readonly stopReason?: AssistantMessage['stopReason'];
  /**
   Failure diagnostic.
   */
  readonly error?: string;
  /**
   Authentication delay before actual dispatch.
   */
  readonly preparationMs?: number;
};
/**
 Shared synthetic usage makes double-counting observable.
 */
export const FIXTURE_USAGE: Usage = {
  input: 7,
  output: 5,
  reasoning: 3,
  cacheRead: 2,
  cacheWrite: 4,
  totalTokens: 18,
  cost: {
    input: 1,
    output: 2,
    cacheRead: 3,
    cacheWrite: 4,
    total: 10,
  },
};
/**
 Timeline event owned by one fixture.
 */
type FixtureEvent = {
  readonly at: number;
  readonly run: () => void
};
/**
 Captured real dispatch after preparation and the final provider gate.
 */
type FixtureDispatch = {
  readonly model: string;
  readonly at: number;
  readonly signal: Parameters<AdvisorDispatch>[0]['signal']
};
/**
 Optional scheduling overrides for a fixture run.
 */
type FixtureRunOptions = Pick<AdvisorOperationOptions, 'hedgeDelayMs' | 'signal'> & { readonly deadlineAtMs?: number; };
/**
 Test-local clock, provider scripts, and captured evidence.
 */
type OperationFixture = {
  /**
   Actual provider invocations.
   */
  readonly dispatched: readonly FixtureDispatch[];
  /**
   Selected candidates, including preparation failures.
   */
  readonly prepared: readonly string[];
  /**
   Detached operation progress.
   */
  readonly progress: readonly AdvisorOperationSnapshot[];
  /**
   Current fixture clock.
   */
  readonly now: () => number;
  /**
   Exercise production scheduling with the fixture timeline.
   */
  readonly run: (options?: FixtureRunOptions) => Promise<AdvisorOperationSnapshot>;
  /**
   Drain remaining simulated events after local cancellation.
   */
  readonly flush: () => Promise<void>;
};

/**
 Build terminal synthetic provider data.
 
 @param plan - chosen attempt script
 
 @returns complete provider message
 */
function response(plan: AttemptPlan,): AssistantMessage {
  return {
    role: 'assistant',
    api: 'faux',
    provider: 'fixture',
    model: 'fixture',
    timestamp: 0,
    stopReason: plan.stopReason ?? 'stop',
    usage: FIXTURE_USAGE,
    content: [{
      type: 'text',
      text: plan.text ?? '',
    },],
    ...(plan.error === undefined ? {} : { errorMessage: plan.error, }),
  };
}

/**
 Create a local provider timeline without real requests or global timer mutation.
 
 @param plans - per-model attempt sequence
 
 @param events - optional caller actions on the local timeline
 
 @returns operations and captured dispatch evidence
 
 @example
 ```ts
 const fixture = operationFixture({ plans: { 'p/a': [{ after: 5, text: 'review' }] } });
 ```
 */
export function operationFixture({
  plans,
  events = [],
}: {
  readonly plans: Readonly<Record<string, readonly AttemptPlan[]>>;
  readonly events?: readonly FixtureEvent[];
},): OperationFixture {
  /**
   Mutable time is private to this fixture.
   */
  const clock = { value: 0, };
  /**
   Finite scheduled provider and caller actions.
   */
  const queue = [...events,];
  /**
   Selected endpoints, including rejected preparation.
   */
  const prepared: string[] = [];
  /**
   Actual provider requests after the operation's final gate.
   */
  const dispatched: FixtureDispatch[] = [];
  /**
   Immutable ledger-derived updates.
   */
  const progress: AdvisorOperationSnapshot[] = [];

  /**
   Current local time.

   @returns fixture clock reading
   */
  function now(): number {
    return clock.value;
  }

  /**
   Supply a controlled asynchronous provider boundary.
   
   @param input - scheduler-owned dispatch observers and cancellation
   
   @returns scripted terminal response
   */
  async function complete(input: Parameters<AdvisorDispatch>[0],): Promise<AssistantMessage> {
    /**
     Prior attempts establish this endpoint's script index.
     */
    const index = prepared.filter(function sameModel(model: string,): boolean {
      return model
        === input.candidate
          .model;
    },)
      .length;
    prepared.push(input.candidate
      .model,);
    /**
     Script for the selected attempt.
     */
    const plan = plans[input.candidate
      .model]?.[index];
    if (plan === undefined)
      throw new Error(`fixture missing plan for ${input.candidate
        .model} attempt ${index + 1}`,);
    /**
     Local preparation start anchors the fixture's completion.
     */
    const started = clock.value;
    if (plan.preparationMs !== undefined) {
      /**
       Controlled asynchronous authentication.
       */
      const ready = Promise.withResolvers<void>();
      queue.push({
        at: started + plan.preparationMs,
        run: function preparedReady(): void { ready.resolve(); },
      },);
      await ready.promise;
    }
    input.onDispatch({ reasoning: 'high', },);
    dispatched.push({
      model: input.candidate
        .model,
      at: clock.value,
      signal: input.signal,
    },);
    input.onUsage(FIXTURE_USAGE,);
    /**
     Deliberately noncooperative provider permits late-settlement verification.
     */
    const pending = Promise.withResolvers<AssistantMessage>();
    queue.push({
      at: started + plan.after,
      run: function deliverResponse(): void { pending.resolve(response(plan,),); },
    },);
    return await pending.promise;
  }

  /**
   Select a provider event or the scheduler's own boundary without sleeping for model latency.
   
   @param options - pending outcomes and next scheduler cutoff
   
   @returns next observed outcome or cutoff sentinel
   */
  async function wait(options: Parameters<NonNullable<AdvisorOperationOptions['wait']>>[0],): ReturnType<NonNullable<AdvisorOperationOptions['wait']>> {
    await nextTurn();
    /**
     Process notifications already queued by synchronous or microtask work first.
     */
    const settled = await Promise.race([
      ...options.pending,
      Promise.resolve(NOT_READY,),
    ],);
    if (settled !== NOT_READY)
      return settled;
    queue.sort(function eventOrder(
      left: FixtureEvent,
      right: FixtureEvent,
    ): number { return left.at - right.at; },);
    /**
     Next event may follow the operation's wakeup boundary.
     */
    const [event,] = queue;
    if ((event !== undefined) && (event.at <= options.untilMs)) {
      queue.shift();
      clock.value = event.at;
      event.run();
      await nextTurn();
      return await Promise.race([
        ...options.pending,
        Promise.resolve(ADVISOR_CLOCK_BOUNDARY,),
      ],);
    }
    clock.value = options.untilMs;
    return ADVISOR_CLOCK_BOUNDARY;
  }

  /**
   Record progress across fixture runs without allocating a new callback per run.

   @param value - operation snapshot retained for assertions
   */
  function recordProgress(value: AdvisorOperationSnapshot,): void {
    progress.push(value,);
  }

  return {
    dispatched,
    prepared,
    progress,
    now,
    async run(options: FixtureRunOptions = {}): Promise<AdvisorOperationSnapshot> {
      return await runAdvisorOperation({
        candidates: Object.keys(plans,)
          .map(function candidate(model: string,) {
            return {
              model,
              provider: model.split('/',)[0] ?? 'fixture',
              contextChars: 20,
              estimatedInputTokens: 5,
              truncated: false,
            };
          },),
        startedAtMs: 0,
        deadlineAtMs: 200,
        collectionGraceMs: 30,
        complete,
        wait,
        now,
        onUpdate: recordProgress,
        ...options,
      },);
    },
    async flush(): Promise<void> {
      while (queue.length > 0) {
        queue.sort(function eventOrder(
          left: FixtureEvent,
          right: FixtureEvent,
        ): number { return left.at - right.at; },);
        /**
         Finite remaining fixture event.
         */
        const event = queue.shift();
        if (event === undefined)
          throw new Error('fixture event disappeared',);
        clock.value = event.at;
        event.run();
        // oxlint-disable-next-line no-await-in-loop -- Drain each event's microtasks before advancing its dependent fixture clock.
        await nextTurn();
      }
    },
  };
}
