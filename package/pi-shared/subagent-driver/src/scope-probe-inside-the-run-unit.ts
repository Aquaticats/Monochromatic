/**
 The program a run's unit runs in a test that asks whether a scope can be started from inside it.

 Why a probe instead of the driver: the driver starts Pi, which spends a provider slot and takes minutes; what this test asks is
 whether the unit this package builds can start a `systemd-run --user --scope` at all, and whether the checkout stays read-only
 around it.

 What it does, in the unit the launcher builds: writes a file into the checkout, which must be refused; writes one into the run's
 worktree and reads it back, which must work; starts a scope that writes one file into the worktree, and reads that back; and writes
 what each answered into the run's own folder, where the test reads it.

 @module
 */

import { execFile, } from 'node:child_process';
import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import { promisify, } from 'node:util';

/**
 `execFile` with a promise result, typed through Node's own adapter contract.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;

/**
 Runs a program without a shell and waits for it to end.
 */
const execFileAsync: typeof execFile.__promisify__ = promisifyExecFile(execFile,);

/**
 What one attempt answered.
 */
type Attempt = {
  /**
   The status it ended with, which is one only for an attempt that failed without a status of its own.
   */
  readonly status: number;

  /**
   What it printed.
   */
  readonly output: string;
};

/**
 A text a failed command reported, which is only a string when the failure carried one.

 @param value - what the failure held where a text would be

 @returns that text, or nothing when it was not one

 @example
 ```ts
 const printed = textOf(failure.stdout);
 ```
 */
function textOf(value: unknown,): string {
  return ((typeof value) === 'string') ? value : '';
}

/**
 The status a failed command reported.

 @param failure - what the failure held where a status would be

 @returns that status, or one when it carried none

 @example
 ```ts
 const status = statusOf(failure.code);
 ```
 */
function statusOf(failure: unknown,): number {
  /**
   What the failure carried where a status would be, which is only a number when the command ended with one.
   */
  const code: unknown = (((typeof failure) === 'object') && (failure !== null)) ? Reflect.get(
    failure,
    'code',
  ) : undefined;
  return ((typeof code) === 'number') ? code : (-1);
}

/**
 The value of one option the unit's words carry, which must be there.

 @param words - the unit's words

 @param option - the option, with its leading dashes

 @returns its value

 @throws Error when the words do not carry it, naming what the probe needs

 @example
 ```ts
 const runFolder = valueOf({ words: process.argv.slice(2,), option: '--run-folder', });
 ```
 */
function valueOf({
  words,
  option
}: {
  readonly words: readonly string[];
  readonly option: string
},): string {
  /**
   The word that carries the option, when one does.
   */
  const carrying = words.find(function namesIt(word,) {
    return word.startsWith(`${option}=`,);
  },);
  if (carrying === undefined) {
    throw new Error(`the probe needs ${option}: ${words.join(' ',)}`,);
  }
  return carrying.slice(`${option}=`.length,);
}

/**
 Tries one command inside this unit, and reports what it answered.

 @param command - the program

 @param words - its words

 @param folder - where it runs

 @returns its status and what it printed

 @example
 ```ts
 const attempt = await attemptOf({ command: 'systemd-run', words: ['--user', '--scope', '--quiet'], folder: '/w', });
 ```
 */
async function attemptOf({
  command,
  words,
  folder,
}: {
  readonly command: string;
  readonly words: readonly string[];
  readonly folder: string;
},): Promise<Attempt> {
  try {
    /**
     What it answered.
     */
    const answered = await execFileAsync(
      command,
      [...words,],
      {
        cwd: folder,
        encoding: 'utf8',
        timeout: 30_000,
        killSignal: 'SIGKILL',
      },
    );
    return {
      status: 0,
      output: `${answered.stdout}${answered.stderr}`,
    };
  }
  catch (error) {
    return {
      status: statusOf(error,),
      output: `${textOf(readField({
        failure: error,
        name: 'stdout',
      },),)}${textOf(readField({
        failure: error,
        name: 'stderr',
      },),)}${textOf(readField({
        failure: error,
        name: 'message',
      },),)}`,
    };
  }
}

/**
 One field a failed command reported, as it was.

 @param failure - what the command threw

 @param name - the field's name

 @returns that field, or nothing when the failure had none

 @example
 ```ts
 const printed = readField({ failure: error, name: 'stderr', });
 ```
 */
function readField({
  failure,
  name
}: {
  readonly failure: unknown;
  readonly name: string
},): unknown {
  return (((typeof failure) === 'object') && (failure !== null)) ? Reflect.get(
    failure,
    name,
  ) : undefined;
}

/**
 The words this probe was given, which are the unit's own.
 */
const words = process.argv
  .slice(2,);
/**
 Where the driver would write this run, which this probe writes its report into.
 */
const runFolder = valueOf({
  words,
  option: '--run-folder',
},);
/**
 The worktree the run works in.
 */
const worktree = valueOf({
  words,
  option: '--working-folder',
},);
/**
 The checkout the run must not change, which this test names after the unit's own words.
 */
const checkout = valueOf({
  words,
  option: '--checkout',
},);

/**
 What a direct write into the checkout answered, which must be the operating system's own refusal.
 */
const checkoutWrite = await attemptOf({
  command: process.execPath,
  words: [
    '--eval',
    `require('node:fs').writeFileSync(${JSON.stringify(join(
      checkout,
      'written-by-the-run.txt',
    ),)}, 'written\\n');`,
  ],
  folder: checkout,
},);

/**
 Where this probe writes in the run's own worktree, which must be allowed.
 */
const worktreeFile = join(
  worktree,
  'written-in-the-worktree.txt',
);
/**
 What a write into the run's own worktree answered, which must work, and what reading it back gave.
 */
const worktreeWrite = await attemptOf({
  command: process.execPath,
  words: [
    '--eval',
    `const fs = require('node:fs'); fs.writeFileSync(${JSON.stringify(worktreeFile,)}, 'written in the worktree\\n'); process.stdout.write(fs.readFileSync(${JSON.stringify(worktreeFile,)}, 'utf8').trim());`,
  ],
  folder: worktree,
},);

/**
 What the scope's own unit was called, named so that a leftover can be found and stopped.
 */
const scopeUnit = `probe-scope-inside-the-run-unit-${String(process.pid,)}`;
/**
 What the scope wrote, which is one file in the run's own worktree.
 */
const scopedFile = join(
  worktree,
  'written-by-the-scope.txt',
);
/**
 What starting the scope answered: the shape every brief on this machine tells a run to use.
 */
const scope = await attemptOf({
  command: 'systemd-run',
  words: [
    '--user',
    '--scope',
    '--quiet',
    `--unit=${scopeUnit}`,
    process.execPath,
    '--eval',
    `require('node:fs').writeFileSync(${JSON.stringify(scopedFile,)}, 'written by a scope\\n');`,
  ],
  folder: worktree,
},);
/**
 What the scope wrote, read back after it ended.
 */
const scopedRead = await attemptOf({
  command: process.execPath,
  words: [
    '--eval',
    `process.stdout.write(require('node:fs').readFileSync(${JSON.stringify(scopedFile,)}, 'utf8').trim());`,
  ],
  folder: worktree,
},);

await writeFile(
  join(
    runFolder,
    'scope-report.json',
  ),
  `${JSON.stringify(
    {
    checkoutWrite,
    worktreeWrite,
    scope,
    scopedRead,
  },
    undefined,
    2,
  )}\n`,
);
