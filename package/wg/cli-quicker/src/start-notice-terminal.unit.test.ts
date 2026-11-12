import { spawn, } from 'node:child_process';
import { once, } from 'node:events';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { printStartNotice, } from '../dist/final/node/start-notice-terminal.mjs';

/**
 Built module the start path prints through.
 */
const PRINTER_URL: string = new URL('../dist/final/node/start-notice-terminal.mjs', import.meta.url,).href;

/**
 Logger the rest of the program records through, which the two switches are for.
 */
const LOGGER_URL: string = new URL('../../../logger/core/dist/final/node/index.mjs', import.meta.url,).href;

/**
 Text of the child program.
 
 It prints one start notice through the printer under test and one ordinary
 record through a logger, so one run shows both channels: the notice is what the
 decision protects, and the ordinary record is what the switches are for.
 */
const CHILD_SOURCE = `
import { printStartNotice } from ${JSON.stringify(PRINTER_URL,)};
import { tagged } from ${JSON.stringify(LOGGER_URL,)};

const l = tagged({ tag: 'start-notice-terminal-test' });
l.warn('ORDINARY WARNING RECORD');
printStartNotice('START NOTICE LINE');
await l.flush();
`;

/**
 What one run of the child program wrote to its standard error.
 */
type ChildRun = {
  /**
   Everything the child wrote to standard error.
   */
  readonly stderr: string;

  /**
   Everything the child wrote to standard output.
   */
  readonly stdout: string;
};

/**
 Runs the child program with the given environment additions.

 The logging switches are removed from the inherited environment first, so a
 run sees only what this test sets: a switch inherited from the caller would
 make the control case test the caller's shell instead of the code.

 @param environment - Additions to the child's environment.

 @returns What the child wrote to each stream.

 @example
 ```ts
 await runChild({ MONOCHROMATIC_WARN: 'false' });
 ```
 */
async function runChild(
  { environment, }: { readonly environment: Readonly<Record<string, string>>; },
): Promise<ChildRun> {
  /**
   Environment without the logging switches this test owns.
   */
  const inherited: Record<string, string> = {};
  for (const [name, value,] of Object.entries(process.env,)) {
    if ((value !== undefined)
      && (name !== 'MONOCHROMATIC_WARN')
      && (name !== 'MONOCHROMATIC_LOG_CONSOLE'))
      inherited[name] = value;
  }
  /**
   Child running the printer and one ordinary record.
   */
  const child = spawn(
    process.execPath,
    [
      '--input-type=module',
      '--eval',
      CHILD_SOURCE,
    ],
    {
      env: { ...inherited, ...environment, },
      stdio: ['ignore', 'pipe', 'pipe',],
    },
  );
  /**
   Collected text of each stream.
   */
  const collected = { stdout: '', stderr: '', };
  child.stdout.on('data', (chunk,) => {
    collected.stdout += String(chunk,);
  });
  child.stderr.on('data', (chunk,) => {
    collected.stderr += String(chunk,);
  });
  await once(child, 'close',);
  return collected;
}

await describe({
  name: printStartNotice.name,
  children: [
    it({
      name: 'reaches the terminal while the warning switch and the console switch are both off, where a logger record does not',
      fn: async () => {
        /**
         What the child wrote with both switches off.
         */
        const run = await runChild({
          environment: {
            MONOCHROMATIC_WARN: 'false',
            MONOCHROMATIC_LOG_CONSOLE: 'off',
          },
        },);
        expect(run.stderr,).toContain('START NOTICE LINE',);
        expect(run.stderr,).not.toContain('ORDINARY WARNING RECORD',);
      },
    },),
    it({
      name: 'is one record on standard error and none on standard output, so a pipe of the program output stays clean',
      fn: async () => {
        /**
         What the child wrote with no switch set.
         */
        const run = await runChild({ environment: {}, },);
        expect(run.stderr.split('START NOTICE LINE',).length - 1,).toBe(1,);
        expect(run.stdout,).toBe('',);
      },
    },),
    it({
      name: 'leaves the warning switch working for the records the program logs, so a hook keeps its streams quiet with it',
      fn: async () => {
        /**
         What the child wrote with only the warning switch off.
         */
        const run = await runChild({ environment: { MONOCHROMATIC_WARN: 'false', }, },);
        expect(run.stderr,).toContain('START NOTICE LINE',);
        expect(run.stderr,).not.toContain('ORDINARY WARNING RECORD',);
      },
    },),
  ],
},);
