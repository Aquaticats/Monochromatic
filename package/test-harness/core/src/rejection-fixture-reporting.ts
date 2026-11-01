/**
 Reporter fault injection isolated from the parent test process.
 @module
 */

import { setTimeout as wait, } from 'node:timers/promises';
import { logger, } from '@monochromatic-dev/logger-core/ts';
import { it, } from '../dist/final/neutral/index.mjs';

/**
 Timer delivers the fault after the root body and its normal flush finish.
 */
const REJECTION_DELAY_MS = 20;

/**
 Exercises throwing, detached, and unfinished reporter work without recursion.

 @param scenario - reporter fault to activate only after body completion

 @example
 await runReportingScenario('reporter-detached');
 */
export async function runReportingScenario(scenario: string,): Promise<void> {
  /**
   Normal runner flushes must complete before injecting the reporter-only fault.
   */
  const phase = { reporting: false, };
  await it({
    name: 'reporting owner',
    l: {
      ...logger,
      warn: function warning(message: string,): void {
        phase.reporting = true;
        logger.warn(message,);
        if (scenario === 'reporter-throws')
          throw new Error('injected warning failure',);
        if (scenario === 'reporter-detached')
          void Promise.reject(new Error('detached reporter failure',),);
      },
      flush: function flushReporting(): Promise<void> {
        if (phase.reporting && (scenario === 'reporter-unfinished')) {
          process.stderr
            .cork();
          return Promise.withResolvers<void>()
            .promise;
        }
        if (phase.reporting && (scenario === 'reporter-flush-rejects'))
          return Promise.reject(new Error('reporter flush rejected',),);
        return logger.flush();
      },
    },
    fn: function delayedRejection(): Promise<void> {
      setTimeout(
        function rejectAfterRoot(): void {
          void Promise.reject(new Error('original reported failure',),);
        },
        REJECTION_DELAY_MS,
      );
      return Promise.resolve();
    },
  },);
  await wait(REJECTION_DELAY_MS * 2,);
  console.log('REPORTER_FIXTURE_FINISHED',);
}
