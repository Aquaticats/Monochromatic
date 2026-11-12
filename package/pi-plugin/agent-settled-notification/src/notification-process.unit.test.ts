/**
 Tests for the bounded desktop-notification subprocess boundary.

 Every case runs a real child process, and none of them can reach a desktop: the
 executable is this Node binary told to write and exit, never `notify-send`.
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  NOTIFICATION_CAPTURE_LIMIT_BYTES,
  NotificationSpawnError,
  runNotificationProcess,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Node binary running the stand-in notifier, so no case depends on the host's PATH.
 */
const NODE_EXECUTABLE = process.execPath;

/**
 Deadline for a stand-in notifier that is expected to answer on its own.
 */
const PROMPT_DEADLINE_MS = 5_000;

/**
 Deadline for a stand-in notifier that is expected to be killed by it.
 */
const KILLING_DEADLINE_MS = 250;

/**
 Marker a killed stand-in notifier never gets to write.
 */
const SURVIVING_MARKER = 'survived the deadline';

/**
 Command that cannot exist, so the process boundary provably fails to start it.
 */
const MISSING_EXECUTABLE = '/nonexistent/notify-send-for-the-settled-notification-tests';

/**
 Bytes of standard error a flooding stand-in notifier writes.
 */
const FLOOD_BYTES = NOTIFICATION_CAPTURE_LIMIT_BYTES * 3;

/**
 Runs this Node binary with one inline program.

 @param program - JavaScript source the stand-in notifier runs

 @returns arguments for the process boundary

 @example
 ```ts
 nodeArguments('process.exit(1)');
 ```
 */
function nodeArguments(program: string,): readonly string[] {
  return ['--eval', program,];
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: runNotificationProcess.name,
      children: [
        it({
          name: 'reports the exit status of a notifier that refused',
          fn: async () => {
            /**
             Test notifications the stand-in notifier refuses with.
             */
            const outcome = await runNotificationProcess({
              command: NODE_EXECUTABLE,
              args: nodeArguments(String.raw`process.stderr.write("Failed to show notification: refused\n"); process.exit(1);`,),
              timeoutMs: PROMPT_DEADLINE_MS,
            },);

            expect(outcome.ending,).toEqual({
              kind: 'exited',
              exitCode: 1,
            },);
          },
        },),

        it({
          name: 'keeps the standard error that names why the desktop refused',
          fn: async () => {
            /**
             Plasma's own refusal text, as the notifier would print it.
             */
            const refusal = 'Failed to show notification: GDBus.Error:org.freedesktop.Notifications.Error.'
              + 'ExcessNotificationGeneration: Created too many similar notifications in quick succession';

            /**
             Test notifications the stand-in notifier refuses with.
             */
            const outcome = await runNotificationProcess({
              command: NODE_EXECUTABLE,
              args: nodeArguments(`process.stderr.write(${JSON.stringify(refusal,)}); process.exit(1);`,),
              timeoutMs: PROMPT_DEADLINE_MS,
            },);

            expect(outcome.stderr,).toBe(refusal,);
          },
        },),

        it({
          name: 'bounds a standard error a notifier floods',
          fn: async () => {
            /**
             Test notifications the stand-in notifier floods its error stream with.
             */
            const outcome = await runNotificationProcess({
              command: NODE_EXECUTABLE,
              args: nodeArguments(
                `process.stderr.write("x".repeat(${String(FLOOD_BYTES,)})); process.exit(1);`,
              ),
              timeoutMs: PROMPT_DEADLINE_MS,
            },);

            expect(outcome.stderr,).toContain('[stderr truncated]',);
            expect(outcome.stderr.length,).toBeLessThan(FLOOD_BYTES,);
          },
        },),

        it({
          name: 'reports a quiet notifier that accepted with no standard error',
          fn: async () => {
            /**
             Test notifications the stand-in notifier accepts.
             */
            const outcome = await runNotificationProcess({
              command: NODE_EXECUTABLE,
              args: nodeArguments('process.exit(0);',),
              timeoutMs: PROMPT_DEADLINE_MS,
            },);

            expect(outcome.ending,).toEqual({
              kind: 'exited',
              exitCode: 0,
            },);
            expect(outcome.stderr,).toBe('',);
          },
        },),

        it({
          name: 'kills a notifier that outlives the deadline',
          fn: async () => {
            /**
             Test notifications the stand-in notifier keeps alive past the deadline.
             */
            const outcome = await runNotificationProcess({
              command: NODE_EXECUTABLE,
              args: nodeArguments(
                `setTimeout(function survive(){ process.stderr.write(${JSON.stringify(SURVIVING_MARKER,)}); }, 60_000);`,
              ),
              timeoutMs: KILLING_DEADLINE_MS,
            },);

            expect(outcome.ending,).toEqual({ kind: 'killed-at-its-deadline', },);
            expect(outcome.stderr,).not.toContain(SURVIVING_MARKER,);
          },
        },),

        it({
          name: 'reports an executable that cannot be started',
          fn: async () => {
            /**
             Start failure the process boundary reported.
             */
            let caught: unknown;
            try {
              await runNotificationProcess({
                command: MISSING_EXECUTABLE,
                args: [],
                timeoutMs: PROMPT_DEADLINE_MS,
              },);
            }
            catch (error) {
              caught = error;
            }

            expect(caught,).toBeInstanceOf(NotificationSpawnError,);
            expect((caught as Error).message,).toContain(MISSING_EXECUTABLE,);
          },
        },),
      ],
    },),
  ],
},);
