/**
 Lifecycle cases executed in child processes, never in the parent regression suite.
 @module
 */

import { setTimeout as wait, } from 'node:timers/promises';
import { logger, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  it,
} from '../dist/final/neutral/index.mjs';

/**
 Referenced delay allows Node to deliver detached rejection events.
 */
const DELIVERY_MS = 20;

/**
 Nonzero status set by another owner must survive harness failure handling.
 */
const EXISTING_EXIT_STATUS = 7;

/**
 Returns without async work for listener-sharing and expected-failure controls.

 @example
 await pass();
 */
function pass(): Promise<void> {
  return Promise.resolve();
}

/**
 Leaks one rejection and gives the observer time to finish before the body settles.

 @example
 await rejectWhileRunning();
 */
async function rejectWhileRunning(): Promise<void> {
  void Promise.reject(new Error('lifecycle escaped rejection',),);
  await wait(DELIVERY_MS,);
}

/**
 Selects independently reproducible lifecycle branches.

 @param scenario - child-only case requested by the parent test

 @throws Error when the requested case is absent

 @example
 await runLifecycleScenario('shared-copies');
 */
export async function runLifecycleScenario(scenario: string,): Promise<void> {
  if (scenario === 'shared-copies') {
    /**
     Distinct source and artifact modules must share both listener and storage.
     */
    const source = await import('./index.ts');
    /**
     Listener count before any descriptor executes in this child.
     */
    const before = process.listenerCount('unhandledRejection',);
    await Promise.all([
      source.it({
        name: 'source root',
        fn: async function sourceBody(): Promise<void> {
          await it({
            name: 'artifact nested',
            fn: rejectWhileRunning,
          },);
        },
        repeats: 1,
      },),
      it({
        name: 'independent root',
        fn: pass,
      },),
    ],);
    console.log(`LISTENER_DELTA=${String(process.listenerCount('unhandledRejection',) - before,)}`,);
  }
  else if (scenario === 'existing-listener') {
    process.on(
      'unhandledRejection',
      function existingListener(): void {
        console.log('EXISTING_LISTENER_CALLED',);
      },
    );
    await it({
      name: scenario,
      fn: rejectWhileRunning,
    },);
    console.log(`LISTENERS=${String(process.listenerCount('unhandledRejection',),)}`,);
  }
  else if ((scenario === 'reset-exit') || (scenario === 'preserve-exit')) {
    await it({
      name: scenario,
      fn: rejectWhileRunning,
    },);
    process.exitCode = scenario === 'reset-exit' ? 0 : EXISTING_EXIT_STATUS;
  }
  else if (scenario === 'expected-failure') {
    await it({
      name: scenario,
      fails: true,
      fn: async function expectedBodyFailure(): Promise<void> {
        await rejectWhileRunning();
        throw new Error('expected returned rejection',);
      },
    },);
    console.log('EXPECTED_BODY_PASSED',);
  }
  else if (scenario === 'concurrent-attribution') {
    await describe({
      name: 'concurrent',
      children: [
        it({
          name: 'rejecting owner',
          fn: rejectWhileRunning,
        },),
        it({
          name: 'other owner',
          fn: async function otherBody(): Promise<void> {
            await wait(DELIVERY_MS * 2,);
            console.log('OTHER_OWNER_FINISHED',);
          },
        },),
      ],
    },);
  }
  else if (scenario === 'rejecting-context') {
    /**
     Creation ownership differs deliberately from the context calling reject.
     */
    const pending = { current: Promise.withResolvers<void>(), };
    /**
     Creator installs a fresh deferred promise inside its execution context.
     */
    const deferred = await it({
      name: 'creator',
      fn: function createInTest(): Promise<void> {
        pending.current = Promise.withResolvers<void>();
        return Promise.resolve();
      },
    },);
    // Rejection from another test must follow the rejecting context, not creator.
    console.log(`CREATOR=${deferred.name}`,);
    await it({
      name: 'rejector',
      fn: async function rejectFromAnotherBody(): Promise<void> {
        pending.current
          .reject(new Error('rejected from another execution',),);
        await wait(DELIVERY_MS,);
      },
    },);
  }
  else if (scenario === 'timeout-tail') {
    try {
      await it({
        name: 'timed-out owner',
        timeout: 1,
        fn: async function bodyOutlivingTimeout(): Promise<void> {
          await wait(DELIVERY_MS,);
          void Promise.reject(new Error('detached timeout tail',),);
        },
      },);
    }
    catch (error: unknown) {
      console.log(`EXPECTED_TIMEOUT=${String(error,)}`,);
    }
    await it({
      name: 'after timeout',
      fn: async function waitForTail(): Promise<void> {
        await wait(DELIVERY_MS * 2,);
        console.log('AFTER_TIMEOUT_FINISHED',);
      },
    },);
  }
  else if (scenario === 'suite-context') {
    /**
     Logger entry runs inside the suite's execution, outside a child test.
     */
    let leaked = false;
    await describe({
      name: 'owning suite',
      children: [],
      l: {
        ...logger,
        debug: function suiteLog(message: string,): void {
          logger.debug(message,);
          if (!leaked) {
            leaked = true;
            void Promise.reject(new Error('suite infrastructure rejection',),);
          }
        },
      },
    },);
    await wait(DELIVERY_MS,);
  }
  else if (scenario === 'non-error') {
    await it({
      name: scenario,
      fn: async function rejectPrimitives(): Promise<void> {
        // oxlint-disable-next-line eslint/prefer-promise-reject-errors, typescript/prefer-promise-reject-errors -- Explicit primitive fault injection covers Node's unknown reason contract; option limits documented in doc/troubleshooting/module-test-unhandled-rejection.md.
        void Promise.reject('primitive rejection',);
        // oxlint-disable-next-line eslint/prefer-promise-reject-errors, typescript/prefer-promise-reject-errors -- Explicit undefined reason is not an omitted argument; see the primitive fixture rationale in doc/troubleshooting/module-test-unhandled-rejection.md.
        void Promise.reject(undefined,);
        await wait(DELIVERY_MS,);
      },
    },);
  }
  else
    throw new Error(`Unknown lifecycle scenario: ${scenario}`,);
  console.log('LIFECYCLE_FINISHED',);
}
