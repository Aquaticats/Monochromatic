import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { NativeGateError } from './native-gate-error.ts';
import {
  runGateCommand,
  runGateCommandOrThrow,
  stepNameOf,
} from './native-gate-run.ts';

/**
 Milliseconds a test's own deadline allows, which is far more than a `node -e` needs.
 */
const testDeadlineMilliseconds = 60_000;

/**
 Milliseconds a hung command is given before it is stopped, which the test then waits out.
 */
const shortDeadlineMilliseconds = 300;

await describe({
  name: 'one command a gate command runs',
  children: [
    it({
      name: 'reports what a command printed and that it ended by itself',
      fn: async () => {
        /**
         How the answer ended.
         */
        const outcome = await runGateCommand({
          program: process.execPath,
          args: [
            '-e',
            'process.stdout.write("answered")',
          ],
          deadlineMilliseconds: testDeadlineMilliseconds,
        });
        expect(outcome,).toEqual({
          status: 0,
          stdout: 'answered',
          stderr: '',
          timedOut: false,
        },);
      },
    }),
    it({
      name: 'reports a nonzero status and the output, so a refusal can quote it',
      fn: async () => {
        /**
         How the failure ended.
         */
        const outcome = await runGateCommand({
          program: process.execPath,
          args: [
            '-e',
            String.raw`process.stderr.write("refused\n"); process.exit(2)`,
          ],
          deadlineMilliseconds: testDeadlineMilliseconds,
        });
        expect(outcome.status,).toBe(2,);
        expect(outcome.stderr,).toBe('refused\n',);
        expect(outcome.timedOut,).toBe(false,);
      },
    }),
    it({
      name: 'stops a command that runs past its deadline and says the deadline stopped it',
      fn: async () => {
        /**
         When the stop was noticed.
         */
        const started = Date.now();
        /**
         How the hung command ended.
         */
        const outcome = await runGateCommand({
          program: process.execPath,
          args: [
            '-e',
            'setTimeout(() => {}, 60000)',
          ],
          deadlineMilliseconds: shortDeadlineMilliseconds,
        });
        expect(outcome.timedOut,).toBe(true,);
        expect(Date.now() - started,).toBeLessThan(30_000,);
      },
    }),
    it({
      name: 'reports a program that is not there as a command that could not be run, not as a status',
      fn: async () => {
        /**
         How the refused start ended.
         */
        const outcome = await runGateCommand({
          program: '/nonexistent/gate-command-that-is-not-there',
          args: [],
          deadlineMilliseconds: testDeadlineMilliseconds,
        });
        expect(outcome.status,).toBe(-1,);
        expect(outcome.problem,).toContain('ENOENT',);
      },
    }),
    it({
      name: 'names the step by its program and first argument, which is enough to tell one step from another',
      fn: async () => {
        expect(stepNameOf({
          program: 'git',
          args: [
            '-C',
            '/w',
            'checkout',
            '--quiet',
            '--detach',
            'abc',
          ],
        }),).toBe('git -C',);
        expect(stepNameOf({
          program: 'git',
          args: [],
        }),).toBe('git',);
      },
    }),
    it({
      name: 'refuses a failed command with its status and last lines, so a coordinator reads one sentence',
      fn: async () => {
        await expect(runGateCommandOrThrow({
          program: process.execPath,
          args: [
            '-e',
            String.raw`const lines = Array.from({ length: 15 }, (_, index) => "line " + String(index + 1)); process.stderr.write(lines.join("\n") + "\n"); process.exit(3)`,
          ],
          deadlineMilliseconds: testDeadlineMilliseconds,
        }),)
          .rejects
          .toThrow('node -e failed with status 3 and said: line 6',);
      },
    }),
    it({
      name: 'refuses a command its deadline stopped, naming the bound and never the command\'s silence',
      fn: async () => {
        await expect(runGateCommandOrThrow({
          program: process.execPath,
          args: [
            '-e',
            'setTimeout(() => {}, 60000)',
          ],
          deadlineMilliseconds: shortDeadlineMilliseconds,
        }),)
          .rejects
          .toThrow('still running after 300 ms',);
      },
    }),
    it({
      name: 'refuses a command that could not start, quoting why',
      fn: async () => {
        await expect(runGateCommandOrThrow({
          program: '/nonexistent/gate-command-that-is-not-there',
          args: [],
          deadlineMilliseconds: testDeadlineMilliseconds,
        }),)
          .rejects
          .toThrow(NativeGateError,);
      },
    }),
    it({
      name: 'answers a command that succeeded with its output',
      fn: async () => {
        /**
         What the succeeding command answered.
         */
        const answered = await runGateCommandOrThrow({
          program: process.execPath,
          args: [
            '-e',
            'process.stdout.write("ok")',
          ],
          deadlineMilliseconds: testDeadlineMilliseconds,
        });
        expect(answered.stdout,).toBe('ok',);
      },
    }),
  ],
});
