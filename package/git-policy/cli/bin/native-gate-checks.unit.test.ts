import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  gateCheckDeadlineMilliseconds,
  runGateChecks,
} from './native-gate-checks.ts';
import { NativeVerificationError, } from './native-verification-process.ts';

/**
 A Clippy run that found nothing to say, which is what a clean Clippy prints.
 */
const greenClippy = {
  status: 0,
  stdout: '',
  stderr: '',
};

/**
 A Clippy run that printed a warning, which `-D warnings` turns into a failure.
 */
const warnedClippy = {
  status: 1,
  stdout: '',
  stderr: 'error: unused variable: `unused`\n',
};

/**
 Milliseconds a check that ends by itself is given, far more than a settled promise needs.
 */
const testDeadlineMilliseconds = 10_000;

/**
 Milliseconds a check that never ends is given before the gate stops waiting for it, far below the time a test would
 wait out.
 */
const shortDeadlineMilliseconds = 300;

/**
 Milliseconds the gate waits past that bound before it answers for a check which never settles, kept short for the same
 reason.
 */
const shortStopGraceMilliseconds = 200;

/**
 Milliseconds a check that stops itself takes, which is past its own bound and inside the grace the gate waits after it.
 */
const selfStoppedMilliseconds = shortDeadlineMilliseconds
  + Math.floor(shortStopGraceMilliseconds / 2);

await describe({
  name: runGateChecks.name,
  children: [
    it({
      name: 'asks Clippy to start before the test run can finish, so the shorter check waits inside the longer one',
      fn: async () => {
        /**
         Order the two checks reached their steps in.
         */
        const reached: string[] = [];
        await runGateChecks({
          deadlineMilliseconds: testDeadlineMilliseconds,
          // The test run yields once before it finishes, so a gate that awaited it before starting Clippy reaches its
          // own end first and the order below says so.
          testRun: async function runTests() {
            reached.push('the test run started');
            await Promise.resolve();
            reached.push('the test run finished');
            return undefined;
          },
          clippyRun: async function runClippy() {
            reached.push('Clippy started');
            return greenClippy;
          },
        });
        expect(reached).toEqual([
          'the test run started',
          'Clippy started',
          'the test run finished',
        ]);
      },
      timeout: 10_000,
    }),
    it({
      name: 'waits for Clippy to finish after the test run has failed, so no container is left behind',
      fn: async () => {
        /**
         Settles once the test run has failed; Clippy finishes only after it, so a gate that gave up early is caught.
         */
        const testRunFailed = Promise.withResolvers<undefined>();
        /**
         Whether Clippy reached its own end.
         */
        let clippyFinished = false;
        await expect(runGateChecks({
          deadlineMilliseconds: testDeadlineMilliseconds,
          testRun: async function runTests() {
            testRunFailed.resolve(undefined);
            throw new Error('the suite named one failing test');
          },
          clippyRun: async function runClippy() {
            await testRunFailed.promise;
            clippyFinished = true;
            return greenClippy;
          },
        })).rejects.toThrow('the suite named one failing test');
        expect(clippyFinished).toBe(true);
      },
      timeout: 10_000,
    }),
    it({
      name: 'fails the gate on Clippy alone and quotes what Clippy printed',
      fn: async () => {
        /**
         The failure of a gate whose suite passed and whose Clippy warned.
         */
        const failing = runGateChecks({
          deadlineMilliseconds: testDeadlineMilliseconds,
          testRun: async function runTests() {
            return undefined;
          },
          clippyRun: async function runClippy() {
            return warnedClippy;
          },
        });
        await expect(failing).rejects.toThrow(NativeVerificationError);
        await expect(failing).rejects.toThrow('Clippy failed');
        await expect(failing).rejects.toThrow('unused variable: `unused`');
      },
      timeout: 10_000,
    }),
    it({
      name: 'names both failures when the suite and Clippy both fail',
      fn: async () => {
        /**
         The failure of a gate where both checks failed.
         */
        const failing = runGateChecks({
          deadlineMilliseconds: testDeadlineMilliseconds,
          testRun: async function runTests() {
            throw new Error('the suite failed');
          },
          clippyRun: async function runClippy() {
            return warnedClippy;
          },
        });
        await expect(failing).rejects.toThrow('the test run failed');
        await expect(failing).rejects.toThrow('Clippy failed');
      },
      timeout: 10_000,
    }),
    it({
      name: 'answers when both checks passed',
      fn: async () => {
        await expect(runGateChecks({
          deadlineMilliseconds: testDeadlineMilliseconds,
          testRun: async function runTests() {
            return undefined;
          },
          clippyRun: async function runClippy() {
            return greenClippy;
          },
        })).resolves.toBeUndefined();
      },
      timeout: 10_000,
    }),
    it({
      name: 'reports the suite failure when Clippy never settles, so a silent check cannot hide the verdict',
      fn: async () => {
        /**
         When the gate was started, which the bound is measured from.
         */
        const started = Date.now();
        /**
         Whether Clippy was started at all, which the pair starting together needs.
         */
        let clippyStarted = false;
        /**
         The gate's answer to a suite that failed beside a Clippy that never ends.
         */
        let caught: unknown;
        try {
          await runGateChecks({
            deadlineMilliseconds: shortDeadlineMilliseconds,
            stopGraceMilliseconds: shortStopGraceMilliseconds,
            testRun: async function runTests() {
              throw new Error('the suite named one failing test');
            },
            clippyRun: function runClippy() {
              clippyStarted = true;
              return new Promise(() => {});
            },
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(clippyStarted,).toBe(true,);
        expect(caught,).toBeInstanceOf(NativeVerificationError,);
        expect((caught as Error).message,).toContain('the test run failed: the suite named one failing test',);
        expect((caught as Error).message,).toContain('Clippy failed',);
        expect((caught as Error).message,).toContain('had still not answered 200 ms after the 300 ms bound',);
        expect(Date.now() - started,).toBeLessThan(5_000,);
      },
      timeout: 15_000,
    }),
    it({
      name: 'quotes a check that stopped itself past its bound, so the gate answers only for a check which ignores it',
      fn: async () => {
        /**
         The gate's answer to a Clippy that stopped itself after its bound and before the gate gave up.
         */
        let caught: unknown;
        try {
          await runGateChecks({
            deadlineMilliseconds: shortDeadlineMilliseconds,
            stopGraceMilliseconds: shortStopGraceMilliseconds,
            testRun: async function runTests() {
              return undefined;
            },
            clippyRun: async function runClippy() {
              await new Promise<void>((resolve,) => {
                setTimeout(resolve, selfStoppedMilliseconds,);
              },);
              throw new Error('Clippy stopped itself past its bound');
            },
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(NativeVerificationError,);
        expect((caught as Error).message,).toContain('Clippy stopped itself past its bound',);
        expect((caught as Error).message,).not.toContain('had still not answered',);
      },
      timeout: 15_000,
    }),
    it({
      name: 'gives each check the bound it waits under, so a check can stop what it started',
      fn: async () => {
        /**
         The bound each check was given.
         */
        const bounds: number[] = [];
        await runGateChecks({
          deadlineMilliseconds: testDeadlineMilliseconds,
          testRun: async function runTests(deadlineMilliseconds) {
            bounds.push(deadlineMilliseconds);
            return undefined;
          },
          clippyRun: async function runClippy(deadlineMilliseconds) {
            bounds.push(deadlineMilliseconds);
            return greenClippy;
          },
        },);
        expect(bounds,).toEqual([
          testDeadlineMilliseconds,
          testDeadlineMilliseconds,
        ],);
      },
      timeout: 10_000,
    }),
    it({
      name: 'gives a check the gate is not told a bound for the gate-wide bound, so no check waits forever',
      fn: async () => {
        /**
         The bounds two checks were given by a gate that named none.
         */
        const bounds: number[] = [];
        await runGateChecks({
          testRun: async function runTests(deadlineMilliseconds) {
            bounds.push(deadlineMilliseconds);
            return undefined;
          },
          clippyRun: async function runClippy(deadlineMilliseconds) {
            bounds.push(deadlineMilliseconds);
            return greenClippy;
          },
        },);
        expect(bounds,).toEqual([
          gateCheckDeadlineMilliseconds,
          gateCheckDeadlineMilliseconds,
        ],);
      },
      timeout: 10_000,
    }),
  ],
});
