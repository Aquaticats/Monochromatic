/**
 The queue's commands as a coordinator runs them: `fill-free-places-from-a-queue` with a real queue file, a real checkout whose range
 changes a test file alone, and stand-in programs on a `PATH` of the test's own, so no unit, no batch and no Pi is started.

 The control this file carries: a queue whose first entry cannot start and whose second can, one pass, and the second is started while
 the first is set aside with its reason and the time, readable by `list-the-entries-a-queue-set-aside` and put back by
 `put-a-set-aside-entry-back-into-a-queue`.

 @module
 */

import {
  execFile,
} from 'node:child_process';
import {
  chmod,
  mkdir,
  mkdtemp,
  readFile,
  readdir,
  rm,
  writeFile,
} from 'node:fs/promises';
import {
  delimiter,
  dirname,
  join,
} from 'node:path';
import { promisify, } from 'node:util';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  NO_SET_ASIDE_RECORD,
  readQueueOfRuns,
  setAsideRecordOf,
} from '../dist/final/node/index.mjs';

/**
 Specialized promisified `execFile`, typed through Node's own adapter contract.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;

/**
 Runs a program without a shell and waits for it to end.
 */
const execFileAsync: typeof execFile.__promisify__ = promisifyExecFile(execFile,);

/**
 The filling command's own entry, as the coordinator runs it.
 */
const FILLING = new URL('fill-free-places-from-a-queue.ts', import.meta.url,).pathname;

/**
 The command that names the entries a queue set aside.
 */
const LIST_SET_ASIDE = new URL('list-the-entries-a-queue-set-aside.ts', import.meta.url,).pathname;

/**
 The command that puts a set-aside entry back.
 */
const PUT_BACK = new URL('put-a-set-aside-entry-back-into-a-queue.ts', import.meta.url,).pathname;

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 Seconds a command of this file may take, which only keeps one that hangs from holding the test.
 */
const PATIENCE_SECONDS = 60;

/**
 Room the build volume is said to have, in bytes, so no test depends on how full this machine's disk is.
 */
const AMPLE_FREE_SPACE = String(1_000 * 1_024 * 1_024 * 1_024,);

/**
 A folder a test may write in, with its queue, its runs, its programs and a checkout whose range changes a test file alone.
 */
type AThrowawayFolder = {
  /**
   The folder's path.
   */
  readonly path: string;

  /**
   The queue's path.
   */
  readonly queueFile: string;

  /**
   The folder the runs live in.
   */
  readonly runs: string;

  /**
   The folder one folder per review goes in.
   */
  readonly reviews: string;

  /**
   The checkout the review reads, whose newest commit changes a test file alone.
   */
  readonly checkout: string;

  /**
   The newest commit of that checkout.
   */
  readonly commit: string;

  /**
   Where the stand-in programs record what they were asked.
   */
  readonly log: string;

  /**
   Removes the folder and everything under it.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Writes an executable stand-in program.

 @param file - the program's path

 @param body - what it does, as a shell body
 */
async function writeProgram({
  file,
  body,
}: {
  readonly file: string;
  readonly body: string;
},): Promise<void> {
  await writeFile(file, `#!/usr/bin/env bash\n${body}\n`,);
  await chmod(file, 0o700,);
}

/**
 Runs git in a folder, as plain `git` with an explicit pathspec, and waits for it to end.

 @param request - the folder and git's words

 @returns what git printed
 */
async function gitIn({
  folder,
  words,
}: {
  readonly folder: string;
  readonly words: readonly string[];
},): Promise<string> {
  /**
   What git printed.
   */
  const done = await execFileAsync('git', words, {
    cwd: folder,
    encoding: 'utf8',
    env: {
      // A PATH of the test's own, so the fixture's commits are made by a plain git and never by this machine's policy wrapper: the
      // wrapper asks the user manager to start a commit service, which is this machine's and not a test's to start.
      PATH: '/usr/bin:/bin',
      HOME: folder,
      GIT_AUTHOR_NAME: 'a test',
      GIT_AUTHOR_EMAIL: 'a-test@example.invalid',
      GIT_COMMITTER_NAME: 'a test',
      GIT_COMMITTER_EMAIL: 'a-test@example.invalid',
    },
    timeout: PATIENCE_SECONDS * 1_000,
  },);
  return done.stdout;
}

/**
 A folder a test may write in, with its queue, its runs, its programs and a checkout whose newest commit changes a test file alone.

 @param queueEntries - what the queue holds, which is the review of the test-only range and a run behind it when none is given

 @returns the folder
 */
async function makeFolder(queueEntries?: readonly Record<string, unknown>[],): Promise<AThrowawayFolder> {
  /**
   The folder's path.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-queue-commands-',),);
  /**
   The folder the runs live in.
   */
  const runs = join(path, 'runs',);
  /**
   The folder one folder per review goes in.
   */
  const reviews = join(path, 'reviews',);
  /**
   The checkout the review reads.
   */
  const checkout = join(path, 'checkout',);
  /**
   Where the stand-in programs live.
   */
  const bins = join(path, 'bin',);
  await mkdir(runs, { recursive: true, },);
  await mkdir(reviews, { recursive: true, },);
  await mkdir(join(checkout, 'src',), { recursive: true, },);
  await mkdir(bins, { recursive: true, },);
  await writeProgram({
    file: join(bins, 'systemd-run',),
    body: `printf '%s\\n' "systemd-run $*" >> "$FAKE_LOG"`,
  },);
  await writeProgram({
    file: join(bins, 'mise',),
    body: `printf '%s\\n' '/bin:/usr/bin'`,
  },);
  await writeProgram({
    file: join(bins, 'systemctl',),
    body: 'exit 0',
  },);
  await writeProgram({
    file: join(bins, 'notify-send',),
    body: `printf '%s\\n' "notify-send $*" >> "$FAKE_LOG"`,
  },);
  /**
   Where the stand-ins record what they were asked.
   */
  const log = join(path, 'asked.log',);
  // The checkout's newest commit changes a test file alone, which is the range the incident's first entry reviewed.
  await gitIn({
    folder: checkout,
    words: ['init', '--initial-branch=main',],
  },);
  await writeFile(join(checkout, 'src', 'a.ts',), 'export const a = 1;\n',);
  await writeFile(join(checkout, 'src', 'a.test.ts',), 'test one\n',);
  await gitIn({
    folder: checkout,
    words: ['add', '--', 'src/a.ts', 'src/a.test.ts',],
  },);
  await gitIn({
    folder: checkout,
    words: ['commit', '--message', 'first', '--', 'src/a.ts', 'src/a.test.ts',],
  },);
  await writeFile(join(checkout, 'src', 'a.test.ts',), 'test one\ntest two\n',);
  await gitIn({
    folder: checkout,
    words: ['commit', '--message', 'second', '--', 'src/a.test.ts',],
  },);
  /**
   The newest commit of the checkout.
   */
  const commit = (await gitIn({
    folder: checkout,
    words: ['rev-parse', 'main',],
  },)).trim();
  /**
   The queue's path.
   */
  const queueFile = join(path, 'queue.json',);
  await writeFile(
    queueFile,
    `[\n${(queueEntries ?? [
      {
        name: 'review-tests-alone',
        reviewOfRange: 'main~1...main',
        commit,
        source: 'src',
      },
      {
        name: 'b',
        branch: 'feat/b',
        prompt: join(runs, 'b-prompt.md',),
        base: commit,
        wall: 28_800,
      },
    ]).map(function asALine(entry,) {
      return `  ${JSON.stringify(entry,)}`;
    },).join(',\n',)}\n]\n`,
  );
  return {
    path,
    queueFile,
    runs,
    reviews,
    checkout,
    commit,
    log,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Runs one command of the package with the test's own programs ahead of node, and answers what it printed.

 @param request - the command's entry, the folder whose programs it uses, and its words

 @returns its status and both streams
 */
async function runCommand({
  entry,
  folder,
  words,
}: {
  readonly entry: string;
  readonly folder: AThrowawayFolder;
  readonly words: readonly string[];
},): Promise<{
  readonly status: number;
  readonly stdout: string;
  readonly stderr: string;
}> {
  /**
   The programs the command may run, then the node that runs it.
   */
  const path = [
    join(folder.path, 'bin',),
    dirname(process.execPath,),
    '/usr/bin',
  ].join(delimiter,);
  /**
   The folder the process table is read from, which holds no process, so no launcher of this machine is counted.
   */
  const processFolder = join(folder.path, 'processes',);
  await mkdir(processFolder, { recursive: true, },);
  try {
    /**
     What the command printed.
     */
    const done = await execFileAsync(process.execPath, [
      entry,
      ...words,
    ], {
      cwd: folder.path,
      env: {
        PATH: path,
        HOME: folder.path,
        FAKE_LOG: folder.log,
        PI_RUNS_FREE_SPACE_BYTES: AMPLE_FREE_SPACE,
        PI_RUNS_PROCESS_FOLDER: processFolder,
      },
      encoding: 'utf8',
      timeout: PATIENCE_SECONDS * 1_000,
      killSignal: 'SIGKILL',
    },);
    return {
      status: 0,
      stdout: done.stdout,
      stderr: done.stderr,
    };
  }
  catch (caught) {
    /**
     What the failure carries.
     */
    const failure = caught as {
      readonly code?: unknown;
      readonly stdout?: unknown;
      readonly stderr?: unknown;
    };
    return {
      status: ((typeof failure.code) === 'number') ? failure.code : 1,
      stdout: ((typeof failure.stdout) === 'string') ? failure.stdout : '',
      stderr: ((typeof failure.stderr) === 'string') ? failure.stderr : '',
    };
  }
}

/**
 Reads what the stand-in programs were asked, which is empty when they were asked nothing.

 @param log - the stand-ins' log

 @returns their lines
 */
async function askedIn(log: string,): Promise<readonly string[]> {
  try {
    return (await readFile(log, 'utf8',)).split('\n',)
      .filter(function isALine(line,) {
        return line !== '';
      },);
  }
  catch (error) {
    // A stand-in that was asked nothing writes no log at all, which is the empty answer.
    if ((error as NodeJS.ErrnoException).code !== MISSING_PATH_CODE) {
      throw error;
    }
    return [];
  }
}

await describe({
  name: 'fill-free-places-from-a-queue',
  children: [
    it({
      name: 'sets aside the first entry whose own start fails and starts the second in the same pass, and the entry can be read and put back',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         What the one pass answered.
         */
        const filled = await runCommand({
          entry: FILLING,
          folder,
          words: [
            `--queue=${folder.queueFile}`,
            '--level=1',
            `--runs-folder=${folder.runs}`,
            `--reviews-folder=${folder.reviews}`,
            `--checkout-folder=${folder.checkout}`,
            '--once',
          ],
        },);
        expect(filled.stderr,).toBe('',);
        expect(filled.status,).toBe(0,);
        expect(filled.stdout,).toContain('nothing to review under src in main~1...main',);
        expect(filled.stdout,).toContain('set aside review-tests-alone (the-entry)',);
        expect(filled.stdout,).toContain('started b in launch-batch-',);
        // The entry that could not start was written down beside the runs, with its reason, before it left the queue.
        expect(await setAsideRecordOf({
          runsFolder: folder.runs,
          name: 'review-tests-alone',
        },),).toMatchObject({
          name: 'review-tests-alone',
          reason: 'nothing to review under src in main~1...main',
        },);
        // The run behind it was handed to its own unit, whose batch list names it, and no run's start was asked for the review.
        /**
         What the stand-in manager was asked.
         */
        const asked = await askedIn(folder.log,);
        expect(asked,).toHaveLength(1,);
        expect(asked[0],).toContain('--unit=launch-batch-');
        expect(asked[0],).toContain('-queue-entry-b',);
        expect(asked[0],).not.toContain('review-tests-alone',);
        // A review whose change has nothing to read writes no brief, so no reviewer is ever given an empty task.
        expect(await readdir(folder.reviews,),).toEqual([],);
        // Both entries are out of the queue: one started, one set aside where a person and a session can read it.
        expect(await readQueueOfRuns({ queueFile: folder.queueFile, },),).toEqual([],);
        /**
         What the command that lists the entries set aside answered.
         */
        const listed = await runCommand({
          entry: LIST_SET_ASIDE,
          folder,
          words: [`--runs-folder=${folder.runs}`,],
        },);
        expect(listed.stdout,).toContain('review-tests-alone',);
        expect(listed.stdout,).toContain('nothing to review under src in main~1...main',);
        expect(listed.stdout,).toContain('1 entry is set aside',);
        /**
         What putting the entry back answered.
         */
        const putBack = await runCommand({
          entry: PUT_BACK,
          folder,
          words: [
            `--queue=${folder.queueFile}`,
            '--name=review-tests-alone',
            `--runs-folder=${folder.runs}`,
          ],
        },);
        expect(putBack.status,).toBe(0,);
        expect(putBack.stdout,).toContain('put review-tests-alone back at position 0 of 1',);
        expect(await readQueueOfRuns({ queueFile: folder.queueFile, },),).toEqual([{
          name: 'review-tests-alone',
          reviewOfRange: 'main~1...main',
          commit: folder.commit,
          source: 'src',
        },],);
      },
    },),
    it({
      name: 'leaves the entry in place and starts nothing behind it when the machine refuses its unit, setting nothing aside',
      fn: async () => {
        await using folder = await makeFolder([
          {
            name: 'a',
            branch: 'feat/a',
            prompt: join('runs', 'a-prompt.md',),
            wall: 28_800,
          },
          {
            name: 'b',
            branch: 'feat/b',
            prompt: join('runs', 'b-prompt.md',),
            wall: 28_800,
          },
        ],);
        // The manager refuses the unit, which is the machine's failure and not the entry's: the entry keeps its place and the
        // filling stops rather than taking the place the earlier entry is waiting for.
        await writeProgram({
          file: join(folder.path, 'bin', 'systemd-run',),
          body: `printf '%s\\n' "systemd-run $*" >> "$FAKE_LOG"\n`
            + `printf '%s\\n' 'Failed to start transient service unit: the manager is not there' >&2\n`
            + 'exit 1',
        },);
        /**
         What the one pass answered.
         */
        const filled = await runCommand({
          entry: FILLING,
          folder,
          words: [
            `--queue=${folder.queueFile}`,
            '--level=2',
            `--runs-folder=${folder.runs}`,
            `--reviews-folder=${folder.reviews}`,
            `--checkout-folder=${folder.checkout}`,
            '--once',
          ],
        },);
        expect(filled.status,).toBe(0,);
        expect(filled.stdout,).toContain('left a in place at position 0 of 2',);
        expect(filled.stdout,).toContain('the manager is not there',);
        expect(await setAsideRecordOf({
          runsFolder: folder.runs,
          name: 'a',
        },),).toBe(NO_SET_ASIDE_RECORD,);
        expect(await readQueueOfRuns({ queueFile: folder.queueFile, },),).toHaveLength(2,);
        // One start was asked for, and the entry behind it was never handed to a unit.
        expect(await askedIn(folder.log,),).toHaveLength(1,);
      },
    },),
  ],
},);
