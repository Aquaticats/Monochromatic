/**
 Separate file-failure diagnostics for promises outside awaited test bodies. @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ObservedExecution, } from './execution-types.ts';
import { formatFailure, } from '@monochromatic-dev/test-harness-core-diagnostic/ts';

/**
 Emits an attributed async-work failure and explains the unchanged body verdict.

 @param reason - rejected value retained for the existing error formatter

 @param execution - rejecting async context, absent when attribution is unavailable

 @throws formatter or logger error for the caller's non-recursive fallback

 @example
 ```ts
 await reportUnhandledRejection({ reason: new Error('disk full'), execution, });
 ```
 */
export async function reportUnhandledRejection({
  reason,
  execution,
}: {
  readonly execution?: ObservedExecution;
  readonly reason: unknown;
},): Promise<void> {
  /**
   Unattributed work has an explicit file-level owner instead of a guessed test.
   */
  const baseLogger = execution?.logger ?? tagged({ tag: 'module-test', },);
  /**
   Async failures cannot be confused with an ordinary test-body verdict.
   */
  const asyncLogger = tagged({
    tag: execution === undefined ? 'unattributed async work' : 'async work',
    l: baseLogger,
  },);
  /**
   File failure record retains the complete test/suite chain when known.
   */
  const failLogger = tagged({
    tag: 'FAIL',
    l: asyncLogger,
  },);
  /**
   Timing describes returned-promise settlement, not background-work completion.
   */
  const timing = execution === undefined
    ? 'No test or suite execution context was available; attribution is unknown.'
    : `The awaited ${execution.kind} execution is ${execution.phase}; its result is unchanged.`;

  asyncLogger.warn(
    `Unhandled promise rejection. ${timing} The test file is failing. `
      + 'A PASS record describes only the awaited body, not detached async work. '
      + 'This usually indicates an unawaited operation, unawaited async assertions, '
      + 'or background work left running after cleanup. A timed-out operation or a dependency can also leak a rejection. '
      + 'Await returned operations and async assertions; stop and await background work during cleanup. '
      + 'For timed-out work, arrange cancellation and wait for termination. '
      + 'Fix a leaking dependency at its owning boundary. '
      + 'For deliberate fault injection, run the rejection scenario in a disposable child process and assert its diagnostics and exit status.',
  );
  failLogger.error(await formatFailure({
    summary: 'unhandled promise rejection; test file failed',
    value: reason,
  },),);
  await asyncLogger.flush();
}
