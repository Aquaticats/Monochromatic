/**
 The step a launcher takes once its unit ended, as it touches the run's one scratch folder: emptied when the run's work is done,
 kept when the run's own work still runs in it.

 The launcher's real emptying command is run here, under a root the test made, so what this reaches is the wiring a run's end
 reaches rather than a stand-in for it.

 @module
 */

import {
  mkdir,
  mkdtemp,
  readdir,
  rm,
  symlink,
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
  type CommandOutput,
  KEEP_FOLDER,
  type RunCommand,
  WAKE_LIMIT,
  finishTheRun,
  makeScratchFolder,
  runCommandWithChildProcess,
} from '../dist/final/node/index.mjs';

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
  const path = await mkdtemp(join(scratchRoot(), 'pi-finish-a-run-',),);
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
 Everything a folder holds, by name, so a test says exactly what is left.

 @param folder - the folder to list

 @returns its entries' names, sorted

 @example
 ```ts
 const names = await namesInFolder('/home/a/temp/agent/a-scratch');
 ```
 */
async function namesInFolder(folder: string,): Promise<readonly string[]> {
  return (await readdir(folder,)).toSorted();
}

/**
 One run's folder, with the summary the driver writes when a run ends its turn.

 @param request - the folder this run's own folder goes in, and the run's name

 @returns the runs folder, the run folder and the scratch root

 @example
 ```ts
 const run = await makeFinishedRun({ base: folder.path, runName: 'a', });
 ```
 */
async function makeFinishedRun({
  base,
  runName,
}: {
  readonly base: string;
  readonly runName: string;
}): Promise<{
  readonly runsFolder: string;
  readonly runFolder: string;
  readonly scratchRoot: string
}> {
  /**
   The folder every run of this test writes into.
   */
  const runsFolder = join(
    base,
    'runs',
  );
  /**
   This run's own folder.
   */
  const runFolder = join(
    runsFolder,
    `${runName}-1`,
  );
  await mkdir(runFolder, { recursive: true, },);
  await writeFile(join(
    runFolder,
    'summary.json',
  ), '{"endReason":"idle"}\n',);
  return {
    runsFolder,
    runFolder,
    scratchRoot: join(
      base,
      'agent',
    ),
  };
}

/**
 A process table in which one process works in one folder, which is how the run's own work is read.

 @param request - the folder this test's process folders live in, and one process's working folder

 @returns the process folder, made of the one process

 @example
 ```ts
 const processFolder = await processTableOf({ base: folder.path, working: runFolder, });
 ```
 */
async function processTableOf({
  base,
  working,
}: {
  readonly base: string;
  readonly working: string;
}): Promise<string> {
  /**
   The folder a Linux machine keeps one entry per process in.
   */
  const processFolder = join(
    base,
    'proc',
  );
  /**
   The one process of this test's table.
   */
  const process = join(
    processFolder,
    '4242',
  );
  await mkdir(process, { recursive: true, },);
  await symlink(
    working,
    join(
      process,
      'cwd',
    ),
  );
  return processFolder;
}

/**
 A process table in which one process works in each folder named, which is how the run's own work is read.

 @param request - the folder this test's process folders live in, and the folder each process works in

 @returns the process folder, made of one process per folder

 @example
 ```ts
 const processFolder = await processTableOfMany({ base: folder.path, working: [runFolder], });
 ```
 */
async function processTableOfMany({
  base,
  working,
}: {
  readonly base: string;
  readonly working: readonly string[];
}): Promise<string> {
  /**
   The folder a Linux machine keeps one entry per process in.
   */
  const processFolder = join(
    base,
    'proc',
  );
  await mkdir(processFolder, { recursive: true, },);
  /* oxlint-disable eslint/no-await-in-loop -- one process entry at a time, each made before the next, so a failure names the
     folder it belongs to; see doc/lint/sequential-awaits.md. */
  for (const [index, folder,] of working.entries()) {
    /**
     This process's entry.
     */
    const process = join(
      processFolder,
      String(5_000 + index,),
    );
    await mkdir(process, { recursive: true, },);
    await symlink(
      folder,
      join(
        process,
        'cwd',
      ),
    );
  }
  /* oxlint-enable eslint/no-await-in-loop */
  return processFolder;
}

/**
 A command runner that answers every question with a status and no output, which is what a git question answers here.

 @param status - the status every answer carries

 @returns the runner

 @example
 ```ts
 const run = answering(1);
 ```
 */
function answering(status: number,): RunCommand {
  return async function everyQuestion(): Promise<CommandOutput> {
    return {
      status,
      stdout: '',
      stderr: '',
    };
  };
}

await describe({
  name: finishTheRun.name,
  children: [
    it({
      name: 'keeps the scratch folder while the run own work still runs, whatever reason the run ended with, and empties it when nothing of the run is left',
      fn: async () => {
        /**
         One case: the reason the run ended, the folder its own work works in, and whether the emptying may run.
         */
        const cases = [
          // A reproduction the agent started in a scope of its own keeps running though the run met its wall clock.
          { endReason: 'wall-clock', worksIn: 'worktree', emptied: false, },
          // The same work started from the scratch folder, which is where the briefs tell an agent to run a command.
          { endReason: 'wall-clock', worksIn: 'scratch', emptied: false, },
          // And a run whose work is over is emptied as it always was.
          { endReason: 'wall-clock', worksIn: 'none', emptied: true, },
        ] as const;
        /* oxlint-disable eslint/no-await-in-loop -- one run at a time, each made, ended, emptied and read before the next one, so a
           failure names the case it belongs to; see doc/lint/sequential-awaits.md. */
        for (const one of cases) {
          await using folder = await makeFolder();
          /**
           The run that ended.
           */
          const run = await makeFinishedRun({
            base: folder.path,
            runName: 'copy-route',
          },);
          /**
           The one scratch folder this run got.
           */
          const scratch = await makeScratchFolder({
            scratchRoot: run.scratchRoot,
            runName: 'copy-route',
          },);
          /**
           What a reproduction left in the scratch folder.
           */
          const sentinel = join(
            scratch.folder,
            'repro-output.txt',
          );
          await writeFile(sentinel, 'the reproduction is still writing this\n',);
          /**
           The folder the run's own process works in, which is its worktree, its scratch folder, or nothing.
           */
          const workingFolder = (one.worksIn === 'worktree')
            ? run.runFolder
            : ((one.worksIn === 'scratch') ? scratch.folder : join(folder.path, 'elsewhere',));
          /**
           Every command the end step asked for, so a deletion that did not run is a deletion that was never asked for.
           */
          const asked: string[] = [];
          /**
           Every line the launcher reported.
           */
          const reported: string[] = [];
          await finishTheRun({
            runName: 'copy-route',
            runFolder: run.runFolder,
            runsFolder: run.runsFolder,
            unitStatus: 0,
            wallSeconds: 60,
            readOnlyInMain: true,
            worktree: run.runFolder,
            checkoutFolder: folder.path,
            processFolder: await processTableOfMany({
              base: folder.path,
              working: [workingFolder,],
            },),
            commandLine: [],
            launcherEntry: process.execPath,
            wakesSoFar: 0,
            environment: process.env,
            scratchRoot: run.scratchRoot,
            git: async function noGitQuestion() {
              throw new Error('a run in the checkout itself asks git nothing about a worktree',);
            },
            // The case with nothing left of the run runs the emptying for real, so the deletion is a deletion; the two that keep the
            // folder record what they were asked and never reach a command.
            run: (one.emptied)
              ? runCommandWithChildProcess
              : async function recorded(request,) {
                asked.push(`${request.command} ${request.words.join(' ',)}`,);
                return {
                  status: 1,
                  stdout: '',
                  stderr: '',
                };
              },
            report: function keep(line,) {
              reported.push(line,);
            },
            wait: async function noWait() {
              // The run's work is read once, and nothing here waits for it.
            },
            now: Date.now,
          },);
          if (one.emptied) {
            // Nothing of the run is left, so the folder is emptied and the file the reproduction wrote is gone.
            expect(reported.join('\n',),).toContain(`scratch: copy-route: emptied ${scratch.folder} except keep/:`,);
            expect(await namesInFolder(scratch.folder,),).not.toContain('repro-output.txt',);
            continue;
          }
          /**
           Whether the end step asked for the emptying at all.
           */
          const emptied = asked.some(function namesTheEmptying(command,) {
            return command.includes('empty-the-scratch-folder-of-a-run',);
          },);
          expect(emptied,).toBe(false,);
          expect(reported.join('\n',),).toContain('scratch: copy-route: the folder was kept, the run\'s own work still runs in it',);
          expect(await namesInFolder(scratch.folder,),).toContain('repro-output.txt',);
        }
        /* oxlint-enable eslint/no-await-in-loop */
      },
      timeout: 60_000,
    },),
    it({
      name: 'empties the run one scratch folder when its work is done, and reports what was removed and what keep holds',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         The run that ended its turn.
         */
        const run = await makeFinishedRun({
          base: folder.path,
          runName: 'copy-route',
        },);
        /**
         The one scratch folder this run got.
         */
        const scratch = await makeScratchFolder({
          scratchRoot: run.scratchRoot,
          runName: 'copy-route',
        },);
        await writeFile(join(
          scratch.folder,
          'a-build-log.txt',
        ), 'a build\n',);
        await writeFile(join(
          scratch.keepFolder,
          'report.md',
        ), 'the report\n',);
        /**
         Every line the launcher reported.
         */
        const reported: string[] = [];
        /**
         What finishing the run answered.
         */
        const finished = await finishTheRun({
          runName: 'copy-route',
          runFolder: run.runFolder,
          runsFolder: run.runsFolder,
          unitStatus: 0,
          wallSeconds: 60,
          readOnlyInMain: true,
          worktree: run.runFolder,
          checkoutFolder: folder.path,
          processFolder: await processTableOf({
            base: folder.path,
            working: folder.path,
          },),
          commandLine: [],
          launcherEntry: process.execPath,
          wakesSoFar: 0,
          environment: process.env,
          scratchRoot: run.scratchRoot,
          git: async function noGitQuestion() {
            throw new Error('a run in the checkout itself asks git nothing about a worktree',);
          },
          run: runCommandWithChildProcess,
          report: function keep(line,) {
            reported.push(line,);
          },
          wait: async function noWait() {
            // The run's work is over before the first look, so no wait is ever taken.
          },
          now: Date.now,
        },);
        expect(finished.status,).toBe(0,);
        expect(reported.join('\n',),).toContain(`scratch: copy-route: emptied ${scratch.folder} except keep/:`,);
        expect(reported.join('\n',),).toContain('whose sizes read');
        expect(reported.join('\n',),).toContain('keep/ holds 11 B in 1 file',);
        expect(await namesInFolder(scratch.folder,),).toEqual([KEEP_FOLDER,],);
        expect(await namesInFolder(scratch.keepFolder,),).toEqual(['report.md',],);
      },
    },),
    it({
      name: 'keeps the scratch folder when the run own work still runs in it, so a build still writing there is not deleted under it',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         The run that ended its turn.
         */
        const run = await makeFinishedRun({
          base: folder.path,
          runName: 'copy-route',
        },);
        /**
         The one scratch folder this run got.
         */
        const scratch = await makeScratchFolder({
          scratchRoot: run.scratchRoot,
          runName: 'copy-route',
        },);
        await writeFile(join(
          scratch.folder,
          'a-build-log.txt',
        ), 'a build\n',);
        /**
         Every line the launcher reported.
         */
        const reported: string[] = [];
        await finishTheRun({
          runName: 'copy-route',
          runFolder: run.runFolder,
          runsFolder: run.runsFolder,
          unitStatus: 0,
          wallSeconds: 60,
          readOnlyInMain: false,
          // The run's own work is read as a process working in the folder its worktree stands for, which is what keeps both.
          worktree: run.runFolder,
          checkoutFolder: folder.path,
          processFolder: await processTableOf({
            base: folder.path,
            working: run.runFolder,
          },),
          commandLine: [],
          launcherEntry: process.execPath,
          // A run woken as often as it may be is one whose work is kept rather than waited out again.
          wakesSoFar: WAKE_LIMIT,
          environment: process.env,
          scratchRoot: run.scratchRoot,
          git: async function noGitQuestion() {
            throw new Error('a worktree whose own work still runs is not asked for its branch',);
          },
          run: answering(1,),
          report: function keep(line,) {
            reported.push(line,);
          },
          wait: async function noWait() {
            // A run woken as often as it may be is not waited for again.
          },
          now: Date.now,
        },);
        expect(reported.join('\n',),).toContain('the folder was kept, the run\'s own work still runs in it',);
        expect(await namesInFolder(scratch.folder,),).toEqual([KEEP_FOLDER, 'a-build-log.txt',].toSorted(),);
      },
    },),
  ],
},);
