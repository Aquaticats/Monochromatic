import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { runGateChecks, } from './native-gate-checks.ts';
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
  ],
});
