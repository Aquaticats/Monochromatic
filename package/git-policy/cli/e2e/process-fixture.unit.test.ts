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
        // so this window is the failure signal; the listener records one instead of letting it end the process.
        const uncaught: unknown[] = [];
        const recordUncaught = (error: unknown,): void => {
          uncaught.push(error,);
        };
        process.on('uncaughtException', recordUncaught,);
        try {
          await new Promise<void>((resolve,) => {
            setTimeout(
              resolve,
              ERROR_EVENT_GRACE_MS,
            );
          },);
        }
        finally {
          process.off('uncaughtException', recordUncaught,);
        }
        expect(uncaught,).toHaveLength(0);
      },
    },),
  ],
},);
