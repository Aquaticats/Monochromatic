/**
 Two starts of one run name, at the same time or one after the other's failure: each writes into a run folder of its own, and neither
 removes a worktree the other is using.

 The first test starts the launcher twice, as the command a coordinator runs, with stand-ins on a `PATH` of its own, and holds both
 starts at one step: the `mise` stand-in answers the session's `PATH` only once both starts have asked, which is after each has read
 the runs folder and before either has made its folder.

 The second test asks for a run's worktree while the first attempt at making it fails and another start of that name makes the
 worktree in the window that attempt leaves open, which is what a first attempt killed by its own bound leaves behind.

 @module
 */

import {
  execFile,
} from 'node:child_process';
import {
  chmod,
  mkdir,
  mkdtemp,
  readdir,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import {
  basename,
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
  BASE_OF_A_RUN_NAMING_NONE,
  type CommandOutput,
  type GitRunner,
  makeWorktree,
} from '../dist/final/node/index.mjs';

/**
 The launcher's own entry, run as the command a coordinator runs.
 */
const LAUNCHER = new URL('start-pi-run-in-a-worktree.ts', import.meta.url,).pathname;

/**
 The line the policy wrapper's help starts with, which is the one identity signal the launcher's preflight reads.
 */
const WRAPPER_HELP_LINE = 'Usage: git monochromatic-git-policy-cli <command> [options]';

/**
 The run name both starts take, which is what makes them two starts of one run.
 */
const RUN_NAME = 'shared';

/**
 The word the launcher's "run folder" line puts before the folder.
 */
const RUN_FOLDER_LABEL = 'Run folder ';

/**
 The name of the file a driver writes into the folder of the run that ended.
 */
const SUMMARY_FILE_NAME = 'summary.json';

/**
 Seconds a launcher may take before the test calls it a launch that never ended.
 */
const PATIENCE_SECONDS = 60;

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 `execFile` with a promise result, typed through Node's own adapter contract.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;

/**
 Runs a program without a shell and waits for it to end.
 */
const execFileAsync: typeof execFile.__promisify__ = promisifyExecFile(execFile,);

/**
 A folder for one test, removed when the test ends.

 @returns the folder's path, and the disposer that removes it

 @example
 ```ts
 await using folder = await makeFolder();
 ```
 */
async function makeFolder(): Promise<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  const path = await mkdtemp(join(scratchRoot(), 'pi-two-starts-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, {
        recursive: true,
        force: true,
      },);
    },
  };
}

/**
 Writes an executable stand-in program.

 @param request - where it goes, its name, and what it does

 @returns the path of the program written

 @example
 ```ts
 const git = await writeProgram({ folder: bin, name: 'git', body: 'exit 0\n', },);
 ```
 */
async function writeProgram({
  folder,
  name,
  body,
}: {
  readonly folder: string;
  readonly name: string;
  readonly body: string;
}): Promise<string> {
  /**
   The program's file.
   */
  const file = join(
    folder,
    name,
  );
  await mkdir(
    folder,
    { recursive: true, },
  );
  await writeFile(file, `#!/bin/sh\n${body}`,);
  await chmod(file, 0o700,);
  return file;
}

/**
 The folders one test's stand-ins and its checkout live in.
 */
type Fixture = {
  readonly launcherBin: string;
  readonly sessionBin: string;
  readonly checkout: string;
  readonly runs: string;
  readonly home: string;
  readonly prompt: string;
  readonly rendezvous: string;
  readonly units: string;
};

/**
 Lays out one test's fixture: the stand-in programs, the checkout and the runs folder.

 @param folder - the folder everything goes in

 @returns where each part is

 @example
 ```ts
 const fixture = await fixtureIn(folder.path);
 ```
 */
async function fixtureIn(folder: string,): Promise<Fixture> {
  /**
   Every part of the fixture, each under the folder of this test.
   */
  const fixture: Fixture = {
    launcherBin: join(
      folder,
      'launcher-bin',
    ),
    sessionBin: join(
      folder,
      'session-bin',
    ),
    checkout: join(
      folder,
      'checkout',
    ),
    runs: join(
      folder,
      'runs',
    ),
    home: join(
      folder,
      'home',
    ),
    prompt: join(
      folder,
      'prompt.md',
    ),
    rendezvous: join(
      folder,
      'rendezvous.log',
    ),
    units: join(
      folder,
      'units.log',
    ),
  };
  await mkdir(fixture.launcherBin, { recursive: true, },);
  await mkdir(fixture.sessionBin, { recursive: true, },);
  // A folder a checkout marks, which is what says where a run's worktree belongs when the launcher is started inside it.
  await mkdir(join(
    fixture.checkout,
    '.git',
  ), { recursive: true, },);
  await mkdir(fixture.runs, { recursive: true, },);
  await mkdir(fixture.home, { recursive: true, },);
  await writeFile(fixture.prompt, 'do the work\n',);
  // The launcher's own git: records the question and answers the checkout's base.
  await writeProgram({
    folder: fixture.launcherBin,
    name: 'git',
    body: `case "$*" in\n  'rev-parse --show-toplevel')\n    printf '%s\\n' '${fixture.checkout}'\n    exit 0 ;;\n  *'rev-parse --verify'*)\n    printf '%s\\n' 'a-commit'\n    exit 0 ;;\nesac\nexit 1\n`,
  },);
  // The launcher's own mise: the step both starts are held at, and the node the unit runs. It answers the session's PATH only once
  // both starts have asked, so the folder a start reads for its number cannot have been made by the other start yet.
  await writeProgram({
    folder: fixture.launcherBin,
    name: 'mise',
    body: `case "$*" in\n  'exec -- printenv PATH')\n    printf '%s\\n' "$$" >> '${fixture.rendezvous}'\n    waited=0\n    while [ "$(wc -l < '${fixture.rendezvous}')" -lt 2 ] && [ "$waited" -lt 400 ]; do\n      sleep 0.05\n      waited=$((waited + 1))\n    done\n    printf '%s\\n' '${fixture.sessionBin}${delimiter}/usr/bin'\n    exit 0 ;;\n  *'command -v node')\n    printf '%s\\n' '${process.execPath}'\n    exit 0 ;;\nesac\nexit 1\n`,
  },);
  // The manager the launcher asks about a unit's name and state, which holds no unit and answers before the wait looks again.
  await writeProgram({
    folder: fixture.launcherBin,
    name: 'systemctl',
    body: `sleep 0.1\nprintf '%s\\n' 'not-found'\nexit 0\n`,
  },);
  // The unit manager: writes the summary a driver writes into the folder it is told to write into, saying which start wrote it, so
  // the folder each start's report landed in is visible afterwards.
  await writeProgram({
    folder: fixture.launcherBin,
    name: 'systemd-run',
    body: `for word in "$@"; do\n  case "$word" in\n    --run-folder=*)\n      runFolder="\${word#--run-folder=}"\n      ;;\n  esac\ndone\nprintf '%s\\t%s\\n' "$PI_START_ID" "$runFolder" >> '${fixture.units}'\nprintf '%s\\n' '{"endReason":"pi-exited","startId":"'"$PI_START_ID"'"}' > "$runFolder/${SUMMARY_FILE_NAME}"\nexit 0\n`,
  },);
  // The session's own git, which answers the policy namespace help as the wrapper does.
  await writeProgram({
    folder: fixture.sessionBin,
    name: 'git',
    body: `printf '%s\\n' '${WRAPPER_HELP_LINE}'\nexit 0\n`,
  },);
  // The session's own Pi, which loads its extensions and prints its help.
  await writeProgram({
    folder: fixture.sessionBin,
    name: 'pi',
    body: `printf '%s\\n' 'Usage: pi [options]'\nexit 0\n`,
  },);
  return fixture;
}

/**
 What one start of the run name ended with.

 @param startId - which start of the name this was

 @param status - the status it exited with

 @param output - what it printed on both streams
 */
type Started = {
  readonly startId: string;
  readonly status: number;
  readonly output: string;
};

/**
 Runs one start of the run name with the fixture's stand-ins on its own search path.

 @param request - the fixture, and which start of the name this is

 @returns the status it exited with, and what it printed

 @example
 ```ts
 const started = await startOf({ fixture, startId: 'a' });
 ```
 */
async function startOf({
  fixture,
  startId,
}: {
  readonly fixture: Fixture;
  readonly startId: string;
}): Promise<Started> {
  try {
    /**
     What a start that answered printed.
     */
    const answered = await execFileAsync(process.execPath, [
      LAUNCHER,
      `--name=${RUN_NAME}`,
      '--branch=none',
      `--prompt=${fixture.prompt}`,
      '--cpu=100',
      '--memory=1G',
      '--tasks=100',
      '--wall=60',
      '--turn-cap=10',
      `--runs-folder=${fixture.runs}`,
      `--checkout-folder=${fixture.checkout}`,
    ], {
      cwd: fixture.checkout,
      env: {
        PATH: [
          fixture.launcherBin,
          dirname(process.execPath,),
          '/usr/bin',
        ].join(delimiter,),
        HOME: fixture.home,
        PI_START_ID: startId,
      },
      encoding: 'utf8',
      timeout: PATIENCE_SECONDS * 1_000,
      killSignal: 'SIGKILL',
    },);
    return {
      startId,
      status: 0,
      output: `${answered.stdout}${answered.stderr}`,
    };
  }
  catch (error) {
    /**
     What a start that failed reported, read field by field because a failure carries the status it exited with.
     */
    const failure = error as {
      readonly code?: unknown;
      readonly stdout?: unknown;
      readonly stderr?: unknown;
    };
    return {
      startId,
      status: ((typeof failure.code) === 'number') ? failure.code : (-1),
      output: `${((typeof failure.stdout) === 'string') ? failure.stdout : ''}${((typeof failure.stderr) === 'string') ? failure.stderr : ''}`,
    };
  }
}

/**
 Reads the run folder a start's own line names.

 @param output - what the start printed

 @returns the folder, or empty text when its line named none

 @example
 ```ts
 const folder = runFolderNamedIn(start.output);
 ```
 */
function runFolderNamedIn(output: string,): string {
  /**
   Where the line's label starts, or nothing when the start printed no such line.
   */
  const label = output.indexOf(RUN_FOLDER_LABEL,);
  if (label === (-1)) {
    return '';
  }
  /**
   What follows the label, up to the end of that line.
   */
  const after = output.slice(label + RUN_FOLDER_LABEL.length,);
  /**
   That line.
   */
  const line = after.split(
    '\n',
    1,
  )[0] ?? '';
  /**
   Where the line stops naming the folder, which is the semicolon before the worktree it also names.
   */
  const end = line.indexOf(';',);
  return ((end === (-1)) ? line : line.slice(
    0,
    end,
  )).trim();
}

/**
 Reads what a run folder's summary says.

 @param runsFolder - the runs folder the folder is in

 @param folder - the run folder's name

 @returns the text of its summary, or empty text when it wrote none

 @example
 ```ts
 const shown = await summaryIn({ runsFolder: fixture.runs, folder: 'shared-1' });
 ```
 */
async function summaryIn({
  runsFolder,
  folder,
}: {
  readonly runsFolder: string;
  readonly folder: string;
}): Promise<string> {
  try {
    return await readFile(join(
      runsFolder,
      folder,
      SUMMARY_FILE_NAME,
    ), 'utf8',);
  }
  catch (error) {
    // A folder whose run wrote no summary answers with empty text, which no summary of a run that wrote one can be.
    if ((error as NodeJS.ErrnoException).code === MISSING_PATH_CODE) {
      return '';
    }
    throw error;
  }
}

await describe({
  name: 'two starts of one run name',
  children: [
    it({
      name: 'gives each of two starts held at the same step a run folder of its own, holding its own report and not the other start\'s',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         The stand-ins, the checkout and the runs folder both starts share.
         */
        const fixture = await fixtureIn(folder.path,);
        /**
         What both starts printed, each its own process, started at once.
         */
        const started = await Promise.all([
          startOf({
            fixture,
            startId: 'a',
          },),
          startOf({
            fixture,
            startId: 'b',
          },),
        ],);
        for (const one of started) {
          expect(one.status,).toBe(0,);
        }
        /**
         The run folders the runs folder holds, which is every folder of this name and nothing else.
         */
        const runFolders = (await readdir(fixture.runs,))
          .filter(function isARunFolder(entry,) {
            return entry.startsWith(`${RUN_NAME}-`,);
          },)
          .toSorted();
        expect(runFolders,).toEqual([
          `${RUN_NAME}-1`,
          `${RUN_NAME}-2`,
        ],);
        /**
         The folder each start's own line named, and what that folder's summary says.
         */
        const namedAndWritten = await Promise.all(started.map(async function namedByItsLine(one,) {
          /**
           The folder this start's own line named.
           */
          const named = basename(runFolderNamedIn(one.output,),);
          return {
            startId: one.startId,
            named,
            summary: await summaryIn({
              runsFolder: fixture.runs,
              folder: named,
            },),
          };
        },),);
        for (const one of namedAndWritten) {
          // A start writes into the folder its own line names, which is one of the two folders of this name.
          expect(runFolders,).toContain(one.named,);
          // The summary in that folder is this start's own: a folder both starts wrote into would hold one of the two at most.
          expect(one.summary,).toContain(`"startId":"${one.startId}"`,);
        }
        expect(runFolderNamedIn(started[0]?.output ?? '',),).not.toBe(runFolderNamedIn(started[1]?.output ?? '',),);
        /**
         Every summary both units wrote, each line naming the start and the folder it wrote into.
         */
        const written = (await readFile(fixture.units, 'utf8',)).trim()
          .split('\n',);
        expect(written,).toHaveLength(2,);
        expect(new Set(written.map(function folderOf(line,) {
          return line.split('\t',)[1];
        },)).size,).toBe(2,);
      },
    },),
    it({
      name: 'leaves a worktree another start made exactly as it is when that worktree holds work this start did not make, and fails saying so',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         The worktree path both starts of this name would use.
         */
        const worktree = join(
          folder.path,
          '.claude',
          'worktrees',
          `pi-${RUN_NAME}`,
        );
        /**
         What the other start wrote into the worktree it made, and never committed.
         */
        const otherStartsWork = join(
          worktree,
          'uncommitted-by-the-other-start.txt',
        );
        /**
         What git was asked, each question as it was asked.
         */
        const asked: string[][] = [];
        /**
         Whether the attempt that ended has been seen yet.
         */
        const seen: { firstAdd: boolean } = { firstAdd: false, };
        /**
         A git whose first add ends without making a worktree, and which answers the branch as a branch a killed attempt made.

         @param request - the question git is asked

         @returns what git answers it
         */
        async function gitIn({ words, }: Parameters<GitRunner>[0],): Promise<CommandOutput> {
          asked.push([...words,],);
          const isAdd = (words[0] === 'worktree') && (words[1] === 'add');
          if (isAdd && (!seen.firstAdd)) {
            seen.firstAdd = true;
            // The window a first attempt killed by its own bound leaves open: another start of this name makes the worktree here,
            // and writes what it has not committed into it.
            await mkdir(join(
              worktree,
              '.git',
            ), { recursive: true, },);
            await writeFile(otherStartsWork, 'work the other start has not committed\n',);
            return {
              status: 1,
              stdout: '',
              stderr: 'this attempt ended before it made a worktree',
            };
          }
          /**
           What the code this control was written against asked for between the two attempts: a forced removal of the path, which
           takes the folder and every file the other start wrote in it.
           */
          if ((words[0] === 'worktree') && (words[1] === 'remove')) {
            await rm(worktree, {
              recursive: true,
              force: true,
            },);
            return {
              status: 0,
              stdout: '',
              stderr: '',
            };
          }
          /**
           The branch, which the other start made when it made the worktree, so a second add can take it.
           */
          if (words[0] === 'rev-parse') {
            return {
              status: seen.firstAdd ? 0 : 1,
              stdout: seen.firstAdd ? 'refs/heads/feat/shared\n' : '',
              stderr: '',
            };
          }
          /**
           What the worktree holds, which is the other start's uncommitted file and nothing else.
           */
          if ((words[0] === 'status') && (words[1] === '--porcelain')) {
            return {
              status: 0,
              stdout: `?? ${basename(otherStartsWork,)}\n`,
              stderr: '',
            };
          }
          /**
           A pruning of the registration a killed add may have left, and every later add, which makes the worktree.
           */
          return {
            status: 0,
            stdout: '',
            stderr: '',
          };
        }
        /**
         What making the worktree answered, which is a refusal because the worktree holds work this start did not make.
         */
        const refused: { caught?: unknown } = {};
        try {
          await makeWorktree({
            git: gitIn,
            checkoutFolder: folder.path,
            worktree,
            branch: 'feat/shared',
            base: BASE_OF_A_RUN_NAMING_NONE,
          },);
        }
        catch (error) {
          refused.caught = error;
        }
        // Nothing this start asks git for is a removal: the path holds the worktree another start of this name made.
        expect(asked.filter(function isARemoval(words,) {
          return words.includes('remove',);
        },),).toEqual([],);
        expect(await readFile(otherStartsWork, 'utf8',),).toContain('has not committed',);
        // The refusal is read by the name its error carries and by its words, so this control needs nothing the fix adds to the
        // library: the code without the fix reaches these assertions with nothing caught at all.
        expect((refused.caught as Error | undefined)?.name,).toBe('WorktreeTakenError',);
        expect(String(refused.caught,),).toContain('left exactly as it is',);
        expect(String(refused.caught,),).toContain('nothing was removed',);
      },
    },),
    it({
      name: 'uses a worktree that holds nothing uncommitted as it is, which is what its own killed attempt leaves',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         The worktree path both starts of this name would use.
         */
        const worktree = join(
          folder.path,
          '.claude',
          'worktrees',
          `pi-${RUN_NAME}`,
        );
        /**
         What git was asked, each question as it was asked.
         */
        const asked: string[][] = [];
        /**
         Whether the attempt that ended has been seen yet.
         */
        const seen: { firstAdd: boolean } = { firstAdd: false, };
        /**
         A git whose first add ends without making a worktree while a worktree with nothing uncommitted stands there.

         @param request - the question git is asked

         @returns what git answers it
         */
        async function gitIn({ words, }: Parameters<GitRunner>[0],): Promise<CommandOutput> {
          asked.push([...words,],);
          if ((words[0] === 'worktree') && (words[1] === 'add') && (!seen.firstAdd)) {
            seen.firstAdd = true;
            await mkdir(join(
              worktree,
              '.git',
            ), { recursive: true, },);
            return {
              status: 1,
              stdout: '',
              stderr: 'this attempt ended before it made a worktree',
            };
          }
          return {
            status: 0,
            stdout: '',
            stderr: '',
          };
        }
        expect(await makeWorktree({
          git: gitIn,
          checkoutFolder: folder.path,
          worktree,
          branch: 'feat/shared',
          base: BASE_OF_A_RUN_NAMING_NONE,
        },),).toContain('already there and holding nothing uncommitted',);
        // Nothing is removed and nothing else is asked for: the worktree is there and holds nothing anybody wrote.
        expect(asked.filter(function isARemoval(words,) {
          return words.includes('remove',);
        },),).toEqual([],);
        expect(asked.at(-1),).toEqual(['status', '--porcelain',],);
      },
    },),
  ],
},);
