import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  mkdtemp,
  readFile,
  rm,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { watchForInterruption, } from '../dist/final/node/interrupt-signal.mjs';

/**
 Module logger for this file's cleanup.
 */
const l = tagged({ tag: 'interrupt-signal-test', },);

/**
 Text of the child program,
 which runs the real `bringUp` over recording steps and runs a command in one of them,
 the way every real step does.

 The interrupt is sent to the child's whole process group,
 as a terminal sends it:
 the command in the step dies with the signal, the step fails,
 and the up must roll back instead of dying itself.

 The two modules it imports are filled in with the build output's own paths,
 so the child is the same code the real `up` runs.
 */
const CHILD_SOURCE = `
import { appendFileSync } from 'node:fs';
import { watchForInterruption } from 'INTERRUPT_MODULE';
import { run } from 'RUNNER_MODULE';
import { bringUp } from 'TUNNEL_UP_MODULE';

const record = process.argv[1];
const note = (name) => appendFileSync(record, name + '\\n');
/* A long command in the step, so the interrupt arrives while the step runs.
   It names itself in the record first, so the interrupt is sent only once it runs:
   a signal that arrives before the command starts reaches none of it,
   and the step then waits the command out instead of being interrupted. */
const blocked = { wait: () => run({ command: 'sh', args: ['-c', 'printf "command-running\\n" >> "$0"; sleep 30', record] }) };
using interruption = watchForInterruption();
try {
  await bringUp({
    exemptMark: 100,
    steps: {
      addLink: async () => { note('addLink'); },
      runPreUp: async () => { note('runPreUp'); },
      applyPeerConfig: async () => { note('applyPeerConfig'); },
      allowEndpoints: async () => { note('allowEndpoints'); },
      addAddressesAndBringUp: async () => { note('addAddressesAndBringUp'); },
      setDns: async () => { note('setDns'); },
      setupRoutes: async () => { note('setupRoutes'); await blocked.wait(); },
      startExemptions: async () => { note('startExemptions'); },
      runPostUp: async () => { note('runPostUp'); },
    },
    rollback: async () => {
      /* A real rollback runs commands, so it cannot finish in the same tick the step failed in;
         yielding here also gives the signal handler its turn before the up says why it stopped. */
      await new Promise((resolve) => setTimeout(resolve, 0));
      note('rollback');
    },
    interruption,
  });
}
catch (error) {
  note('threw ' + error.name + ': ' + error.message);
  process.exitCode = 1;
}
`;

/**
 Waits until the recording file holds `text`,
 and fails when it never does.

 @param record - File the child appends to.

 @param text - Line to wait for.

 @example
 ```ts
 await waitFor({ record, text: 'command-running' });
 ```
 */
async function waitFor(
  {
    record,
    text,
  }: {
    readonly record: string;
    readonly text: string;
  },
): Promise<void> {
  /**
   Attempts made so far.
   */
  const attempts = { count: 0, };
  /* oxlint-disable eslint/no-await-in-loop -- Each attempt must follow the one before it. */
  while (attempts.count < 300) {
    try {
      if ((await readFile(record, 'utf8',)).includes(text,))
        return;
    }
    catch (error: unknown) {
      if ((error as NodeJS.ErrnoException).code !== 'ENOENT')
        throw error;
    }
    attempts.count += 1;
    await new Promise(function later(resolve,) {
      setTimeout(resolve, 20,);
    },);
  }
  /* oxlint-enable eslint/no-await-in-loop */
  throw new Error(`the child never recorded ${text}`,);
}

await describe({
  name: watchForInterruption.name,
  children: [
    it({
      name: 'lets every step run while no signal arrives',
      fn: async () => {
        using interruption = watchForInterruption();
        expect(() => {
          interruption.beforeStep('add the link',);
        },).not.toThrow();
      },
    },),
    it({
      name: 'refuses the next step once a signal arrived, naming the signal and the step that did not run',
      fn: async () => {
        using interruption = watchForInterruption();
        process.emit('SIGINT', 'SIGINT',);
        /**
         The refusal the next step boundary must raise.
         */
        const refusal = ((): unknown => {
          try {
            interruption.beforeStep('install the routes and rules',);
          }
          catch (error: unknown) {
            return error;
          }
          throw new Error('the interrupted up did not refuse its next step',);
        })();
        expect((refusal as Error).name,).toBe('UpInterruptedError',);
        expect((refusal as Error).message,).toContain('SIGINT',);
        expect((refusal as Error).message,).toContain('install the routes and rules',);
      },
    },),
    it({
      name: 'removes its signal handlers when the up ends, so a later stop in the same process sees the signal again',
      fn: async () => {
        /**
         Signals the process had before the watch was made.
         */
        const before = process.listenerCount('SIGTERM',);
        {
          using interruption = watchForInterruption();
          expect(process.listenerCount('SIGTERM',),).toBe(before + 1,);
        }
        expect(process.listenerCount('SIGTERM',),).toBe(before,);
      },
    },),
    it({
      name: 'runs the rollback of an interrupted up in a real child process: the signal arrives while a step waits, and what the up created is removed',
      fn: async () => {
        /**
         Scratch runtime directory standing in for the one a real up uses.
         */
        const directory = await mkdtemp(join(scratchRoot(), 'wg-quicker-interrupt-',),);
        /**
         File the child records its steps in.
         */
        const record = join(directory, 'events.txt',);
        /**
         The built modules the child imports, beside each other in the build output.
         */
        const modules = new URL('../dist/final/node/', import.meta.url,).pathname;
        /**
         Child running the real up sequence with the real signal watch,
         killed and removed however this test ends.
         */
        const child = spawn(
          process.execPath,
          [
            '--input-type=module',
            '--eval',
            CHILD_SOURCE
              .replace('INTERRUPT_MODULE', `${modules}interrupt-signal.mjs`,)
              .replace('RUNNER_MODULE', `${modules}runner.mjs`,)
              .replace('TUNNEL_UP_MODULE', `${modules}tunnel-up.mjs`,),
            record,
          ],
          // Its own process group, so the interrupt reaches the command inside the step as a terminal's does.
          { stdio: 'ignore', detached: true, },
        );
        await using cleanup = {
          async [Symbol.asyncDispose](): Promise<void> {
            if (child.pid !== undefined) {
              try {
                process.kill(-child.pid, 'SIGKILL',);
              }
              catch (error: unknown) {
                // The child is gone already, which is what this cleanup is for.
                l.debug(`the child was gone before the cleanup: ${String(error,)}`,);
              }
            }
            await rm(directory, { recursive: true, force: true, },);
          },
        };
        await waitFor({ record, text: 'command-running', },);
        // The whole group, as a terminal's interrupt reaches the foreground job and its children.
        if (child.pid !== undefined)
          process.kill(-child.pid, 'SIGINT',);
        /**
         How the child ended:
         an interrupt that the watch turned into a rollback leaves a non-zero status.
         */
        const closed: readonly unknown[] = await once(child, 'close',);
        /**
         Status the interrupt left the child with.
         */
        const code = closed.at(0,);
        /**
         Recorded steps of the run.
         */
        const recorded = (await readFile(record, 'utf8',)).trim()
          .split('\n',);
        expect(code,).not.toBe(0,);
        expect(recorded.includes('rollback',),).toBe(true,);
        expect(recorded.at(-1),).toContain('UpInterruptedError',);
        expect(recorded.at(-1),).toContain('SIGINT',);
        expect(recorded.at(-1),).toContain('install the routes and rules',);
      },
    },),
  ],
},);
