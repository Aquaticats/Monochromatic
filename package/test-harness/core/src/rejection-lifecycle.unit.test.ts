/**
 Child-process lifecycle regressions for the shared rejection observer.
 @module
 */

import { describe, it, } from '@monochromatic-dev/test-harness-core';
import { runRejectionProcess, } from './rejection-fixture-process.ts';

await describe({
  name: 'rejection observer lifecycle',
  concurrency: 1,
  children: [
    ...(['no-node-global', 'no-node-version',] as const).map(function neutralRuntime(scenario,) {
      return it({ name: scenario, fn: async ({ expect, },): Promise<void> => {
        /** A non-Node runtime executes descriptors without process-global observers. */
        const result = await runRejectionProcess({ scenario, },);
        expect(result.code,).toBe(0,);
        expect(result.stdout,).toContain('LISTENER_DELTA=0',);
      }, },);
    },),
    ...([
      { scenario: 'shared-copies', contains: '[source root] [artifact nested] [async work] [FAIL]', marker: 'LISTENER_DELTA=1', },
      { scenario: 'existing-listener', contains: 'EXISTING_LISTENER_CALLED', marker: 'LISTENERS=2', },
      { scenario: 'concurrent-attribution', contains: '[concurrent] [rejecting owner] [async work] [FAIL]', marker: 'OTHER_OWNER_FINISHED', },
      { scenario: 'expected-failure', contains: '[expected-failure] [async work] [FAIL]', marker: 'EXPECTED_BODY_PASSED', },
      { scenario: 'rejecting-context', contains: '[rejector] [async work] [FAIL]', marker: 'CREATOR=creator', },
      { scenario: 'timeout-tail', contains: '[timed-out owner] [async work] [FAIL]', marker: 'AFTER_TIMEOUT_FINISHED', },
      { scenario: 'suite-context', contains: '[owning suite] [async work] [FAIL]', marker: 'awaited suite execution is completed', },
      { scenario: 'non-error', contains: 'primitive rejection', marker: 'undefined', },
    ] as const).map(function lifecycleCase({ scenario, contains, marker, },) {
      return it({ name: scenario, fn: async ({ expect, },): Promise<void> => {
        /** Captured child output proves both attribution and survival. */
        const result = await runRejectionProcess({ scenario, },);
        /** Diagnostics may span the logger's stdout and stderr sinks. */
        const output = result.stdout + result.stderr;
        expect(result.code,).toBe(1,);
        expect(output,).toContain(contains,);
        expect(output,).toContain(marker,);
        expect(output,).toContain('LIFECYCLE_FINISHED',);
        expect(output,).not.toContain('[other owner] [async work]',);
        expect(output,).not.toContain('[creator] [async work]',);
      }, },);
    },),
    ...([
      { scenario: 'reset-exit', code: 1, },
      { scenario: 'preserve-exit', code: 7, },
    ] as const).map(function exitCase({ scenario, code, },) {
      return it({ name: scenario, fn: async ({ expect, },): Promise<void> => {
        /** Exit hook reasserts failure without replacing another nonzero status. */
        const result = await runRejectionProcess({ scenario, },);
        expect(result.code,).toBe(code,);
        expect(result.stdout,).toContain('LIFECYCLE_FINISHED',);
      }, },);
    },),
    ...(['reporter-throws', 'reporter-detached', 'reporter-flush-rejects', 'reporter-unfinished',] as const)
      .map(function reportingCase(scenario,) {
        return it({ name: scenario, fn: async ({ expect, },): Promise<void> => {
          /** Reporter faults must not escape or start a recursive diagnostic loop. */
          const result = await runRejectionProcess({ scenario, },);
          expect(result.code,).toBe(1,);
          expect(result.stdout,).toContain('REPORTER_FIXTURE_FINISHED',);
          expect(result.stderr,).toContain(scenario === 'reporter-unfinished'
            ? 'Process exited before rejection diagnostics finished.'
            : 'Rejection reporting failed',);
          expect(result.stderr.split(scenario === 'reporter-unfinished'
            ? 'Process exited before rejection diagnostics finished.'
            : 'Rejection reporting failed',),).toHaveLength(2,);
        }, },);
      },),
    it({ name: 'respects explicit strict mode instead of taking over uncaught exceptions', fn: async ({ expect, },): Promise<void> => {
      /** Strict mode escalates before the rejection event by Node's explicit policy. */
      const result = await runRejectionProcess({ scenario: 'active', nodeArgs: ['--unhandled-rejections=strict',], },);
      expect(result.code,).toBe(1,);
      expect(result.stderr,).toContain('fixture escaped rejection',);
      expect(result.stdout,).not.toContain('SIBLING_FINISHED',);
    }, },),
    ...(['warn', 'warn-with-error-code', 'none', 'throw',] as const).map(function runtimeMode(mode,) {
      return it({ name: `fails and continues with explicit ${mode} mode`, fn: async ({ expect, },): Promise<void> => {
        /** Explicit nonfatal policies still require a failing file status from the harness. */
        const result = await runRejectionProcess({ scenario: 'active', nodeArgs: [`--unhandled-rejections=${mode}`,], },);
        expect(result.code,).toBe(1,);
        expect(result.stdout,).toContain('SIBLING_FINISHED',);
        expect(result.stdout + result.stderr,).toContain('[leaking test] [async work] [FAIL]',);
      }, },);
    },),
  ],
},);
