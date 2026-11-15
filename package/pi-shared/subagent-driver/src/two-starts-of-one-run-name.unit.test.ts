/**
 Two starts of one run name, at the same time or one after the other's failure: each writes into a run folder of its own, asks for a
 unit of its own, and neither uses a worktree the other made or a worktree whose making did not finish.

 Each test starts the package's own launcher, as the command a coordinator runs, with stand-ins for the programs it asks and, where
 the worktree is what the test is about, a real Git repository: a worktree is a thing Git makes, so a test of what a launcher does
 with one has to let Git make it. The other start of a name, a making killed at its bound, and the worktree a person finds are placed
 by the filesystem's state, never by a sleep.

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
  WORKTREE_ADD_TIMEOUT_MILLISECONDS,
  nextRunNumber,
  parseWorktreeLauncherOptions,
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
 The number of the run folder each start claims, which names its unit and the proof it writes.
 */
const FIRST_RUN_NUMBER = 1;

/**
 The word the launcher's "run folder" line puts before the folder.
 */
const RUN_FOLDER_LABEL = 'Run folder ';

/**
 The name of the file a driver writes into the folder of the run that ended.
 */
const SUMMARY_FILE_NAME = 'summary.json';

/**
 Reads the file a making writes beside its worktree, which the launcher names after the run and its own making.

 This is written here rather than taken from the library, so that this file's tests run against the code before the proof existed as
 well: what they show is what a launcher does with a path a person or a killed making left, and the file is the state a reader finds
 beside that path.

 @param worktree - the worktree's path

 @returns the proof file's path

 @example
 ```ts
 const proof = proofFileOf('/repo/.claude/worktrees/pi-shared');
 ```
 */
function proofFileOf(worktree: string,): string {
  /**
   The worktree folder's own name, which the proof is named after.
   */
  const name = worktree.split('/').at(-1) ?? '';
  /**
   The folder the worktree is in.
   */
  const folder = worktree.slice(0, worktree.length - name.length,);
  return `${folder}.${name}.worktree.json`;
}

/**
 Seconds a launcher may take before the test calls it a launch that never ended.
 */
const PATIENCE_SECONDS = 60;

/**
 The search path the real Git of a test fixture is found on, which is the system one: the wrapper this repository installs asks a
 user manager on the session bus for its commit service, and a throwaway repository stays off that bus.
 */
const SYSTEM_GIT_PATH = '/usr/bin:/bin';

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
  readonly units: string;
};

/**
 Lays out one test's fixture: the stand-in programs, the checkout and the runs folder.

 The stand-in for the unit manager holds every unit name to one unit, as the real one does, and records the unit each start asked
 for. The question about a run of the same name already going is answered with nothing, so what these tests show is the unit name a
 start takes, not the courtesy check a start arriving later meets.

 @param folder - the folder everything goes in

 @param withARealRepository - whether the checkout is a Git repository with one commit, which the worktree tests need

 @returns where each part is

 @example
 ```ts
 const fixture = await fixtureIn(folder.path, false);
 ```
 */
async function fixtureIn(folder: string, withARealRepository: boolean,): Promise<Fixture> {
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
    units: join(
      folder,
      'unit-requests.log',
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
  if (withARealRepository) {
    await mkdir(join(fixture.checkout, '.git',), { recursive: true, },);
    await runGit(fixture.checkout, ['init', '--initial-branch=main',],);
    await writeFile(join(fixture.checkout, 'tracked.txt',), 'base\n',);
    await runGit(fixture.checkout, ['add', 'tracked.txt',],);
    await runGit(fixture.checkout, ['-c', 'user.name=Disposable Test', '-c', 'user.email=test@example.invalid', 'commit', '--message=base',],);
  }
  if (!withARealRepository) {
    // A checkout that is no repository: the launcher's own git is a stand-in that answers the base and nothing else, because the
    // test is about the run folder and the unit name rather than about a worktree.
    await writeProgram({
      folder: fixture.launcherBin,
      name: 'git',
      body: `case "$*" in\n  'rev-parse --show-toplevel')\n    printf '%s\\n' '${fixture.checkout}'\n    exit 0 ;;\n  *'rev-parse --verify'*)\n    printf '%s\\n' 'a-commit'\n    exit 0 ;;\nesac\nexit 1\n`,
    },);
  }
  // The launcher's own git, in the other tests, is the real repository's own git on the search path this test gives it.
  // The launcher's own mise: answers the session's PATH, the node the unit runs, and the dependency link.
  await writeProgram({
    folder: fixture.launcherBin,
    name: 'mise',
    body: `case "$*" in\n  'exec -- printenv PATH')\n    printf '%s\\n' '${fixture.sessionBin}${delimiter}/usr/bin'\n    exit 0 ;;\n  *'command -v node')\n    printf '%s\\n' '${process.execPath}'\n    exit 0 ;;\nesac\nrmdir '${folder}/unit-names/'"$unit" 2>/dev/null || true\nexit 0\n`,
  },);
  // The manager the launcher asks about a unit's state, and about the units of a run's name. It holds no unit, because a unit of a
  // real name cannot be made from a test without a manager; the unit stand-in below is what holds a name to one start.
  await writeProgram({
    folder: fixture.launcherBin,
    name: 'systemctl',
    body: `sleep 0.1\nprintf '%s\\n' 'not-found'\nexit 0\n`,
  },);
  // The unit manager: one unit per name, refused the second time as the real manager refuses, and the summary a driver writes into
  // the folder the unit is told to write into, saying which start wrote it and which branch its worktree was on.
  await writeProgram({
    folder: fixture.launcherBin,
    name: 'systemd-run',
    body: `for word in "$@"; do\n  case "$word" in\n    --unit=*) unit="\${word#--unit=}" ;;\n    --run-folder=*) runFolder="\${word#--run-folder=}" ;;\n    --working-folder=*) workingFolder="\${word#--working-folder=}" ;;\n  esac\ndone\nprintf '%s\\t%s\\t%s\\n' "$PI_START_ID" "$unit" "$runFolder" >> '${fixture.units}'\nmkdir -p '${folder}/unit-names'\nif ! mkdir '${folder}/unit-names/'"$unit" 2>/dev/null; then\n  printf 'Unit %s was already loaded or has a fragment file.\\n' "$unit" >&2\n  exit 1\nfi\nwaited=0\nwhile [ "$(wc -l < '${fixture.units}')" -lt 2 ] && [ "$waited" -lt 100 ]; do\n  sleep 0.02\n  waited=$((waited + 1))\ndone\nbranch=$(git -C "$workingFolder" branch --show-current 2>/dev/null || true)\nstatus=$(git -C "$workingFolder" status --porcelain 2>/dev/null || true)\nprintf '%s\\n' '{"endReason":"pi-exited","startId":"'"$PI_START_ID"'","branch":"'"$branch"'","statusLines":"'"$(printf '%s' "$status" | tr '\\n' ';')"'"}' > "$runFolder/${SUMMARY_FILE_NAME}"\nexit 0\n`,
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
 Runs one Git command in a disposable repository, with the system git from a search path of its own.

 @param folder - the repository's folder

 @param words - the command

 @returns what Git printed

 @example
 ```ts
 await runGit(checkout, ['init', '--initial-branch=main']);
 ```
 */
async function runGit(folder: string, words: readonly string[],): Promise<string> {
  /**
   What git printed.
   */
  const answered = await execFileAsync('git', [...words,], {
    cwd: folder,
    env: {
      PATH: SYSTEM_GIT_PATH,
      HOME: folder,
      GIT_TERMINAL_PROMPT: '0',
    },
    encoding: 'utf8',
    timeout: PATIENCE_SECONDS * 1_000,
  },);
  return answered.stdout;
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

 @param request - the fixture, which start of the name this is, and the branch it asks for

 @returns the status it exited with, and what it printed

 @example
 ```ts
 const started = await startOf({ fixture, startId: 'a', branch: 'feat/first' });
 ```
 */
async function startOf({
  fixture,
  startId,
  branch,
}: {
  readonly fixture: Fixture;
  readonly startId: string;
  readonly branch: string;
}): Promise<Started> {
  try {
    /**
     What a start that answered printed.
     */
    const answered = await execFileAsync(process.execPath, [
      LAUNCHER,
      `--name=${RUN_NAME}`,
      `--branch=${branch}`,
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

 @param request - the runs folder the folder is in, and the folder's name

 @returns the summary as it was written, or empty text when the run wrote none

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
    if ((error as NodeJS.ErrnoException).code === 'ENOENT') {
      return '';
    }
    throw error;
  }
}

/**
 Reads the units both starts asked for, one line per request.

 @param file - the log the unit stand-in writes

 @returns one entry per request: the start, the unit and the run folder

 @example
 ```ts
 const requests = await unitRequestsIn(fixture.units);
 ```
 */
async function unitRequestsIn(file: string,): Promise<readonly { readonly startId: string; readonly unitName: string }[]> {
  return (await readFile(
    file,
    'utf8',
  )).trim()
    .split('\n',)
    .filter(function isALine(line,) {
      return line !== '';
    },)
    .map(function fieldsOf(line,) {
      /**
       The three fields of one request.
       */
      const [startId, unitName,] = line.split('\t',);
      return {
        startId: startId ?? '',
        unitName: unitName ?? '',
      };
    },);
}

await describe({
  name: 'two starts of one run name',
  children: [
    it({
      name: 'refuses a run folder number past the largest a run can be counted to, and a number after it, where each is read',
      fn: async () => {
        // A number past what can be counted: 9007199254740992 is 2^53, and adding one to it changes nothing, so a folder claimed
        // there could never be followed by another.
        expect(() => parseWorktreeLauncherOptions({
          commandLine: [
            '--name=copy-route',
            '--branch=feat/copy-route',
            '--prompt=p.md',
            '--cpu=8',
            '--memory=1G',
            '--tasks=10',
            '--wall=60',
            '--run=9007199254740992',
          ],
          checkoutFolder: '/repo',
        },),).toThrow('whole number a run can count to',);
        // A folder at the largest number a run can be counted to leaves no number after it, which the reader of the runs folder
        // says rather than naming one that cannot rise.
        expect(() => nextRunNumber({
          runName: 'copy-route',
          entries: [`copy-route-${String(Number.MAX_SAFE_INTEGER,)}`,],
        },),).toThrow('largest number a run can be counted to',);
      },
    },),
    it({
      name: 'gives each of two starts held at the same step a run folder, a unit and a report of its own',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         The stand-ins, the checkout and the runs folder both starts share.
         */
        const fixture = await fixtureIn(folder.path, false,);
        /**
         What both starts printed, each its own process, started at once.
         */
        const started = await Promise.all([
          startOf({
            fixture,
            startId: 'a',
            branch: 'none',
          },),
          startOf({
            fixture,
            startId: 'b',
            branch: 'none',
          },),
        ],);
        for (const one of started) {
          // The same instant, a day after one unit name was the only one a name could take, ended one of the two: each start asks
          // for the unit its own claimed number names.
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
          `${RUN_NAME}-${String(FIRST_RUN_NUMBER,)}`,
          `${RUN_NAME}-${String(FIRST_RUN_NUMBER + 1,)}`,
        ],);
        /**
         The folder each start's own line named, and what that folder's summary says.
         */
        const namedAndWritten = await Promise.all(started.map(async function namedByItsLine(one,) {
          /**
           The folder this start's own line named.
           */
          const named = runFolderNamedIn(one.output,);
          return {
            startId: one.startId,
            named,
            summary: await summaryIn({
              runsFolder: fixture.runs,
              folder: named.split('/').at(-1) ?? '',
            },),
          };
        },),);
        for (const one of namedAndWritten) {
          expect(one.summary,).toContain(`"startId":"${one.startId}"`,);
        }
        expect(namedAndWritten[0]?.named,).not.toBe(namedAndWritten[1]?.named,);
        /**
         The units the two starts asked for, which are two units of one run name and two numbers.
         */
        const requests = await unitRequestsIn(fixture.units,);
        expect(requests.map(function unitOf(request,) {
          return request.unitName;
        },)
          .toSorted(),).toEqual([
            `pi-${RUN_NAME}@${String(FIRST_RUN_NUMBER,)}.service`,
            `pi-${RUN_NAME}@${String(FIRST_RUN_NUMBER + 1,)}.service`,
          ],);
      },
    },),
    it({
      name: 'refuses a worktree another start of the name made on another branch, and leaves it exactly as it is',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         The stand-ins, the real repository and the runs folder both starts share.
         */
        const fixture = await fixtureIn(folder.path, true,);
        /**
         The worktree path both starts of this name use.
         */
        const worktree = join(
          fixture.checkout,
          '.claude',
          'worktrees',
          `pi-${RUN_NAME}`,
        );
        /**
         What the first start left in the worktree it made.
         */
        const firstStartsFile = join(
          worktree,
          'uncommitted-by-the-first-start.txt',
        );
        /**
         What the first start ended with.
         */
        const first = await startOf({
          fixture,
          startId: 'first',
          branch: 'feat/first',
        },);
        expect(first.status,).toBe(0,);
        await writeFile(firstStartsFile, 'the first start wrote this and did not commit it\n',);
        /**
         What the second start ended with, asking for another branch.
         */
        const second = await startOf({
          fixture,
          startId: 'second',
          branch: 'feat/second',
        },);
        // Nothing here hands a start asking for feat/second a worktree made on feat/first: a commit written there would land on the
        // other branch.
        expect(second.status,).not.toBe(0,);
        expect(second.output,).toContain(worktree,);
        expect(second.output,).toContain('feat/first',);
        expect(second.output,).toContain('feat/second',);
        // The first start's worktree and its uncommitted work are exactly as that start left them.
        expect(await readFile(firstStartsFile, 'utf8',),).toContain('did not commit it',);
        expect((await runGit(worktree, ['branch', '--show-current',],)).trim(),).toBe('feat/first',);
        // The second start wrote no report, because it never worked in a worktree.
        const secondFolder = runFolderNamedIn(second.output,).split('/').at(-1) ?? '';
        expect(await summaryIn({
          runsFolder: fixture.runs,
          folder: secondFolder,
        },),).toBe('',);
      },
    },),
    it({
      name: 'never uses a worktree whose making did not finish, and clears it only once the making own bound has passed',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         The stand-ins, the real repository and the runs folder this start shares with the making it finds.
         */
        const fixture = await fixtureIn(folder.path, true,);
        /**
         The worktree path this name uses.
         */
        const worktree = join(
          fixture.checkout,
          '.claude',
          'worktrees',
          `pi-${RUN_NAME}`,
        );
        // A making killed while Git was populating the worktree: the worktree is there, its `.git` is there, and the files Git had
        // not written yet read as deletions. Placed by Git and the filesystem, not by a sleep.
        await runGit(fixture.checkout, ['worktree', 'add', '-b', 'feat/shared', worktree, 'main',],);
        await rm(join(worktree, 'tracked.txt',), { force: true, },);
        /**
         The proof that making left, which says its making has not finished and was written a moment ago.
         */
        const proofFile = proofFileOf(worktree,);
        function proofOfTheMaking(when: number,): string {
          return JSON.stringify({
            path: worktree,
            runName: RUN_NAME,
            runFolder: `${RUN_NAME}-1`,
            branch: 'feat/shared',
            state: 'making',
            writtenAt: new Date(when,).toISOString(),
          },);
        }
        await writeFile(proofFile, proofOfTheMaking(Date.now(),),);
        /**
         What one start that finds that worktree ended with.
         */
        const whileItMayStillBeMade = await startOf({
          fixture,
          startId: 'while-making',
          branch: 'feat/shared',
        },);
        expect(whileItMayStillBeMade.status,).not.toBe(0,);
        expect(whileItMayStillBeMade.output,).toContain('making has not finished',);
        expect(whileItMayStillBeMade.output,).toContain(worktree,);
        // The half-written worktree is left exactly as it is, so a person can look at what that making left.
        expect((await runGit(worktree, ['status', '--porcelain',],)).trim(),).toContain('D tracked.txt',);
        // The same proof, older than the making's own bound, names an add killed at that bound: its partial checkout is cleared and
        // the worktree is made again, so the run name is not blocked by a making that can never finish.
        await writeFile(proofFile, proofOfTheMaking((Date.now() - WORKTREE_ADD_TIMEOUT_MILLISECONDS) - 120_000,),);
        /**
         What one start that finds a making given up ended with.
         */
        const onceItsBoundPassed = await startOf({
          fixture,
          startId: 'after-the-bound',
          branch: 'feat/shared',
        },);
        expect(onceItsBoundPassed.status,).toBe(0,);
        expect((await runGit(worktree, ['status', '--porcelain',],)).trim(),).toBe('',);
        expect((await runGit(worktree, ['branch', '--show-current',],)).trim(),).toBe('feat/shared',);
      },
    },),
  ],
},);
