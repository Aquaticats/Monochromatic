/**
 The one seam that asks the machine questions: a command that will not answer is killed at its bound rather than waited for.

 @module
 */

import {
  spawn,
} from 'node:child_process';
import {
  once,
} from 'node:events';
import {
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  isThere,
  runCommandShowingItsOutput,
  runCommandWithChildProcess,
} from '../dist/final/node/index.mjs';

/**
 A folder a test may write its own programs in, removed when the test ends.

 @returns the folder, and how to remove it

 @example
 ```ts
 await using folder = await makeFolder();
 ```
 */
async function makeFolder(): Promise<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  /**
   The folder.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-run-command-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Milliseconds a stand-in question is given, which is far less than the sleep it is asked to run.
 */
const SHORT_BOUND_MILLISECONDS = 100;

/**
 Milliseconds the bound of the two grace tests allows, which is long enough for the program it starts to install its own handler: a
 Node start under a loaded machine takes longer than a tenth of a second, and a signal that arrives before the handler ends the
 program instead of testing the grace.
 */
const A_BOUND_THAT_LETS_THE_PROGRAM_START_MILLISECONDS = 1_500;

/**
 Milliseconds a program that ignores the polite signal is given after it, which is the grace `run-command.ts` states.
 */
const POLITE_SIGNAL_GRACE_MILLISECONDS = 5_000;

/**
 Milliseconds this test waits for the grace and the kill to have happened, above the grace itself.
 */
const AFTER_THE_GRACE_MILLISECONDS = 1_500;

/**
 Milliseconds after the polite signal at which the program of the grace tests would write that it survived: past the grace, so its
 absence is the kill and its presence is a program the grace never ended.
 */
const AFTER_THE_POLITE_SIGNAL_MILLISECONDS = 400;

/**
 Milliseconds after its own start at which the program of the grace tests would write that it survived, which is the bound plus the
 grace plus that moment: the bound is set when the program is started.
 */
const SURVIVED_AFTER_START_MILLISECONDS = A_BOUND_THAT_LETS_THE_PROGRAM_START_MILLISECONDS
  + POLITE_SIGNAL_GRACE_MILLISECONDS
  + AFTER_THE_POLITE_SIGNAL_MILLISECONDS;

/**
 The program the bound tests start: it says it started, ignores the polite signal, and would say a stated moment after its start that it
 survived. It ends by itself a moment later, so a test leaves no process behind even when the kill never happens.
 */
const A_CHILD_THAT_IGNORES_ITS_SIGNAL = [
  "import { writeFileSync, } from 'node:fs';",
  'const [started, survived, waitMilliseconds, ] = process.argv.slice(1,);',
  String.raw`writeFileSync(started, 'started\n',);`,
  "process.on('SIGTERM', () => {});",
  String.raw`setTimeout(() => { writeFileSync(survived, 'survived\n',); }, Number(waitMilliseconds,),);`,
  'setTimeout(() => { process.exit(0,); }, Number(waitMilliseconds,) + 600,);',
  '',
].join('\n',);

/**
 Waits a stated time, so a test can watch what a bound left behind.

 @param milliseconds - how long to wait

 @example
 ```ts
 await passTime(1_000);
 ```
 */
async function passTime(milliseconds: number,): Promise<void> {
  await new Promise(function waitItOut(resolve,) {
    setTimeout(resolve, milliseconds,);
  },);
}

/**
 Seconds the command asked to sleep, so a question with no bound would still be sleeping when the test ends.
 */
const LONG_SLEEP_SECONDS = 30;

await describe({
  name: runCommandWithChildProcess.name,
  children: [
    it({
      name: 'answers with what a command printed and the status it exited with',
      fn: async () => {
        expect(await runCommandWithChildProcess({
          command: 'sh',
          words: ['-c', 'printf "out"; printf "err" >&2; exit 3',],
        },),).toEqual({
          status: 3,
          stdout: 'out',
          stderr: 'err',
        },);
      },
    },),
    it({
      name: 'kills a command that runs past its bound instead of waiting for it',
      fn: async () => {
        /**
         How long the question took, which must be the bound rather than the sleep.
         */
        const startedAt = Date.now();
        /**
         What the killed command left.
         */
        const output = await runCommandWithChildProcess({
          command: 'sleep',
          words: [String(LONG_SLEEP_SECONDS,),],
          timeoutMilliseconds: SHORT_BOUND_MILLISECONDS,
        },);
        expect(Date.now() - startedAt,).toBeLessThan(LONG_SLEEP_SECONDS * 1_000,);
        expect(output.status,).toBe(-1,);
        expect(output.problem,).toBeDefined();
        // The bound is named as what ended it, so a caller never reads a command its own bound killed as one that failed.
        expect(output.killedByItsOwnBound,).toBe(true,);
        expect(output.signal,).toBe('SIGKILL',);
      },
    },),
    it({
      name: 'answers a signal from outside as the signal that ended the command, never as its own bound',
      fn: async () => {
        /**
         What a command that is signalled by something else leaves.
         */
        const output = await runCommandWithChildProcess({
          command: 'sh',
          words: ['-c', 'kill -TERM $$',],
        },);
        expect(output.status,).toBe(-1,);
        expect(output.signal,).toBe('SIGTERM',);
        expect(output.killedByItsOwnBound,).toBeUndefined();
      },
    },),
    it({
      name: 'answers that a program that is not there never ran, which a caller reports rather than reads as empty output',
      fn: async () => {
        /**
         What asking for a program that does not exist leaves.
         */
        const output = await runCommandWithChildProcess({
          command: 'a-program-that-is-not-here',
          words: [],
        },);
        expect(output.status,).toBe(-1,);
        expect(output.problem,).toContain('ENOENT',);
      },
    },),
  ],
},);

await describe({
  name: runCommandShowingItsOutput.name,
  children: [
    it({
      name: 'answers at its own bound, and kills the program a stated grace after the polite signal so nothing it started outlives the caller',
      timeout: 40_000,
      fn: async () => {
        await using folder = await makeFolder();
        /**
         Where the program says it started, and where it would say it survived the grace.
         */
        const startedFile = join(
          folder.path,
          'started.txt',
        );
        const survivedFile = join(
          folder.path,
          'survived.txt',
        );
        /**
         When the call was made.
         */
        const before = Date.now();
        /**
         What the call answered for a program that ignores the polite signal.
         */
        const ending = await runCommandShowingItsOutput({
          command: process.execPath,
          words: [
            '--input-type=module',
            '--eval',
            A_CHILD_THAT_IGNORES_ITS_SIGNAL,
            startedFile,
            survivedFile,
            String(SURVIVED_AFTER_START_MILLISECONDS,),
          ],
          timeoutMilliseconds: A_BOUND_THAT_LETS_THE_PROGRAM_START_MILLISECONDS,
        },);
        /**
         How long the call took to answer.
         */
        const tookMilliseconds = Date.now() - before;
        expect(ending,).toEqual({ endedByItsOwnBound: true, },);
        expect(tookMilliseconds,).toBeLessThan(4_000,);
        // The program started, and a grace after the polite signal it was killed before it could write that it survived.
        expect(await isThere(startedFile,),).toBe(true,);
        await passTime(SURVIVED_AFTER_START_MILLISECONDS + AFTER_THE_GRACE_MILLISECONDS,);
        expect(await isThere(survivedFile,),).toBe(false,);
      },
    },),
    it({
      name: 'kills the program at its grace even when the caller that set the bound has exited',
      timeout: 40_000,
      fn: async () => {
        await using folder = await makeFolder();
        /**
         Where the program says it started, and where it would say it survived the grace.
         */
        const startedFile = join(
          folder.path,
          'started.txt',
        );
        const survivedFile = join(
          folder.path,
          'survived.txt',
        );
        /**
         The program that sets the bound and exits at once, which is what the launcher does.
         */
        const caller = join(
          folder.path,
          'a-caller-that-exits.ts',
        );
        await writeFile(
          caller,
          [
            `import { runCommandShowingItsOutput, } from ${JSON.stringify(new URL('../dist/final/node/index.mjs', import.meta.url,).href,)};`,
            'const [started, survived, waitMilliseconds, ] = process.argv.slice(2,);',
            'const ending = await runCommandShowingItsOutput({',
            '  command: process.execPath,',
            '  words: [',
            "    '--input-type=module',",
            "    '--eval',",
            `    ${JSON.stringify(A_CHILD_THAT_IGNORES_ITS_SIGNAL,)},`,
            '    started,',
            '    survived,',
            '    waitMilliseconds,',
            '  ],',
            `  timeoutMilliseconds: ${String(A_BOUND_THAT_LETS_THE_PROGRAM_START_MILLISECONDS,)},`,
            '},);',
            String.raw`process.stdout.write(JSON.stringify({ ending, }) + "\n",);`,
            '',
          ].join('\n',),
        );
        /**
         The caller's process, which this test started and whose number it was given back.
         */
        const spawned = spawn(
          process.execPath,
          [caller, startedFile, survivedFile, String(SURVIVED_AFTER_START_MILLISECONDS,),],
          { stdio: 'ignore', },
        );
        /**
         How the caller ended, as the runtime hands it back.
         */
        const closing: readonly unknown[] = await once(
          spawned,
          'close',
        );
        /**
         The status the caller exited with, which is its first closing word.
         */
        const [status,] = closing;
        // The caller answered at its bound and exited; the program is not its to leave behind.
        expect(status,).toBe(0,);
        expect(await isThere(startedFile,),).toBe(true,);
        await passTime(SURVIVED_AFTER_START_MILLISECONDS + AFTER_THE_GRACE_MILLISECONDS,);
        expect(await isThere(survivedFile,),).toBe(false,);
      },
    },),
  ],
},);
