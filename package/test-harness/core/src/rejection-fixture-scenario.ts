/**
 Intentional detached rejections, executed only by a child process. @module
 */

import { setTimeout as wait, } from 'node:timers/promises';
import type { TestContext, } from '../dist/final/neutral/index.mjs';

/**
 Listener baseline taken before importing the artifact under test.
 */
const listenersBeforeImport = process.listenerCount('unhandledRejection',);
/**
 Built artifact used by direct Node execution without a special launcher.
 */
const {
  describe,
  it,
} = await import('../dist/final/neutral/index.mjs');
/**
 Fixture branch selected by the parent regression test.
 */
const scenario = process.argv
  .at(2,);
/**
 Delay separates completed-body and background-work lifetimes.
 */
const BACKGROUND_DELAY_MS = 20;
/**
 Sibling remains running while the delayed rejection is delivered.
 */
const SIBLING_DELAY_MS = 60;

/**
 Creates the rejection that must be attributed without terminating siblings.

 @example
 ```ts
 leak();
 ```
 */
function leak(): void {
  void Promise.reject(new Error('fixture escaped rejection',),);
}

/**
 Leaves descriptor construction observable without starting any work.

 @example
 await dormant();
 */
function dormant(): Promise<void> {
  return Promise.resolve();
}

/**
 Confirms that an awaited, caught rejection stays outside the unhandled path.

 @param expect - scoped assertion owner

 @example
 await handledRejection(context);
 */
async function handledRejection({ expect, }: TestContext,): Promise<void> {
  try {
    await Promise.reject(new Error('handled fixture rejection',),);
  }
  catch (error: unknown) {
    expect(error,)
      .toBeInstanceOf(Error,);
  }
}

if (scenario === 'import-only') {
  it({
    name: 'unawaited descriptor',
    fn: dormant,
  },);
  console.log(`LISTENER_DELTA=${String(process.listenerCount('unhandledRejection',) - listenersBeforeImport,)}`,);
}
else if (scenario === 'unattributed') {
  setTimeout(
    leak,
    BACKGROUND_DELAY_MS,
  );
  await describe({
    name: 'completed root',
    children: [],
  },);
  console.log('ROOT_RESOLVED',);
}
else if (scenario === 'handled') {
  await it({
    name: 'handled rejection',
    fn: handledRejection,
  },);
}
else if (scenario === 'after-root') {
  await it({
    name: 'already completed',
    fn: function completedBody(): Promise<void> {
      setTimeout(
        leak,
        BACKGROUND_DELAY_MS,
      );
      return Promise.resolve();
    },
  },);
  console.log('ROOT_RESOLVED',);
}
else if ((scenario === 'late') || (scenario === 'active')) {
  await describe({
    name: 'outer',
    concurrency: 1,
    children: [
      describe({
        name: 'inner',
        children: [
          it({
            name: 'leaking test',
            fn: async function leakingBody(): Promise<void> {
              if (scenario === 'active') {
                leak();
                await wait(BACKGROUND_DELAY_MS,);
              }
              else
                setTimeout(
                  leak,
                  BACKGROUND_DELAY_MS,
                );
            },
          },),
          it({
            name: 'unrelated sibling',
            fn: async function siblingBody(): Promise<void> {
              await wait(SIBLING_DELAY_MS,);
              console.log('SIBLING_FINISHED',);
            },
          },),
        ],
      },),
    ],
  },);
  console.log('ROOT_RESOLVED',);
}
else if (scenario?.startsWith('no-node-',) === true) {
  /**
   Runtime-neutral controls do not alter the parent process.
   */
  const { runBrowserScenario, } = await import('./rejection-fixture-browser.ts');
  await runBrowserScenario(scenario,);
}
else if (scenario?.startsWith('reporter-',) === true) {
  /**
   Deferred fixture imports keep the import-side-effect baseline meaningful.
   */
  const { runReportingScenario, } = await import('./rejection-fixture-reporting.ts');
  await runReportingScenario(scenario,);
}
else if (scenario !== undefined) {
  /**
   Extra fixtures import the artifact only after the baseline was measured.
   */
  const { runLifecycleScenario, } = await import('./rejection-fixture-lifecycle.ts');
  await runLifecycleScenario(scenario,);
}
else
  throw new Error('Rejection fixture requires a scenario',);
