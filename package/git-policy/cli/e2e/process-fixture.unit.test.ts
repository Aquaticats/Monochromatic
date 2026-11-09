import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  FixtureCommandError,
  startProcess,
} from './process-fixture.ts';

/**
 Milliseconds the event loop runs after a failed start, long enough for the child's `error` event to be delivered.
 */
const ERROR_EVENT_GRACE_MS = 100;

/**
 Records the uncaught exceptions that arrive while the returned binding lives, and stops recording when it is disposed.

 @returns the exceptions recorded so far, read through the binding

 @example
 ```ts
 using capture = captureUncaughtExceptions();
 await new Promise((resolve) => setTimeout(resolve, ERROR_EVENT_GRACE_MS));
 expect(capture.seen).toHaveLength(0);
 ```
 */
function captureUncaughtExceptions(): Readonly<{ seen: unknown[]; }> & Disposable {
  /** Exceptions the process reported while the binding lived. */
  const seen: unknown[] = [];
  /** Records one exception instead of letting it end the process. */
  const record = (error: unknown,): void => {
    seen.push(error,);
  };
  process.on('uncaughtException', record,);
  return {
    seen,
    [Symbol.dispose](): void {
      process.off('uncaughtException', record,);
    },
  };
}

await describe({
  name: startProcess.name,
  children: [
    it({
      name: 'reports a command that cannot start as a catchable error and keeps the process alive',
      fn: async () => {
        expect(() => startProcess({
          command: '/nonexistent/monochromatic-no-such-command',
          args: [],
          cwd: process.cwd(),
          env: {},
        },),).toThrow(FixtureCommandError,);
        // An unhandled `error` event on the child arrives on a later tick as an uncaught exception,
        // so this window is the failure signal: the capture records one rather than letting it end the process.
        using capture = captureUncaughtExceptions();
        await new Promise<void>((resolve,) => {
          setTimeout(
            resolve,
            ERROR_EVENT_GRACE_MS,
          );
        },);
        expect(capture.seen,).toHaveLength(0);
      },
    },),
  ],
},);
