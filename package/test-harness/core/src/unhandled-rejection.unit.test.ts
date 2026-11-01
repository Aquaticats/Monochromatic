/** Regression coverage for escaped promises at the test-file boundary. @module */

import { describe, it, } from '@monochromatic-dev/test-harness-core';
import { runRejectionProcess, } from './rejection-fixture-process.ts';

await describe({
  name: 'unhandled test rejections',
  concurrency: 1,
  children: [
    it({
      name: 'does not install a listener on import or descriptor construction',
      fn: async ({ expect, },): Promise<void> => {
        /** A successful import remains free of rejection-listener side effects. */
        const result = await runRejectionProcess({ scenario: 'import-only', },);
        expect(result.code,).toBe(0,);
        expect(result.stdout,).toContain('LISTENER_DELTA=0',);
      },
    },),
    it({
      name: 'does not fail the file for a handled rejection',
      fn: async ({ expect, },): Promise<void> => {
        /** Positive control keeps ordinary awaited error assertions successful. */
        const result = await runRejectionProcess({ scenario: 'handled', },);
        expect(result.code,).toBe(0,);
        expect(result.stdout + result.stderr,).not.toContain('[async work]',);
      },
    },),
    ...(['active', 'late',] as const).map(function rejectionTiming(scenario,) {
      return it({
        name: `reports ${scenario} rejection separately without changing body verdicts`,
        fn: async ({ expect, },): Promise<void> => {
          /** Real child process prevents a runtime crash from bypassing assertions. */
          const result = await runRejectionProcess({ scenario, },);
          /** Both output streams participate in the harness diagnostic contract. */
          const output = result.stdout + result.stderr;
          expect(result.code,).toBe(1,);
          expect(result.stdout,).toContain('SIBLING_FINISHED',);
          expect(result.stdout,).toContain('ROOT_RESOLVED',);
          expect(output,).toContain('[outer] [inner] [leaking test] [async work] [FAIL]',);
          expect(output,).toContain('fixture escaped rejection',);
          expect(output,).toContain('async assertions',);
          expect(output,).toContain('cleanup',);
          expect(output,).toContain('timed-out',);
          expect(output,).toContain('dependency',);
          expect(output,).not.toContain('[leaking test] [FAIL]',);
          expect(output,).not.toContain('UnhandledPromiseRejectionWarning',);
        },
      },);
    },),
    it({
      name: 'keeps observing after the root await has resolved',
      fn: async ({ expect, },): Promise<void> => {
        /** A referenced timer outlives the completed root descriptor. */
        const result = await runRejectionProcess({ scenario: 'after-root', },);
        expect(result.code,).toBe(1,);
        expect(result.stdout,).toContain('ROOT_RESOLVED',);
        expect(result.stdout + result.stderr,).toContain('[already completed] [async work] [FAIL]',);
        expect(result.stdout + result.stderr,).toContain('completed',);
      },
    },),
    it({
      name: 'fails unattributed work without guessing a completed test name',
      fn: async ({ expect, },): Promise<void> => {
        /** Timer created outside any test lacks execution-context provenance. */
        const result = await runRejectionProcess({ scenario: 'unattributed', },);
        /** Diagnostic must explicitly identify the missing attribution. */
        const output = result.stdout + result.stderr;
        expect(result.code,).toBe(1,);
        expect(output,).toContain('[module-test] [unattributed async work] [FAIL]',);
        expect(output,).not.toContain('[completed root] [async work]',);
      },
    },),
  ],
},);
