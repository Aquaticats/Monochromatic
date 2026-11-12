/**
 The commands that read what is going on: the processes whose command line holds a text, the running units whose worktree is
 gone, and the final report of each agent's newest finished run. Each command is also run as a program, against stand-in `ps` and
 `systemctl` programs on a `PATH` of the test's own, so no test reads this machine's processes or units.

 @module
 */

import { execFile, } from 'node:child_process';
import {
  chmod,
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { delimiter, dirname, join, } from 'node:path';
import { promisify, } from 'node:util';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  NO_FINISHED_RUN_OF_THAT_NAME,
  REPORT_LINES,
  REPORT_WIDTH,
  type ProcessUnderARun,
  countLineOf,
  goneLineOf,
  matchLineOf,
  newestFinishedRunOf,
  noFinishedRunLineOf,
  processesWhoseCommandLineHolds,
  reportLinesIn,
  reportLinesOf,
  serviceUnitsIn,
  summaryIn,
  unitsWhoseWorktreeIsGone,
} from '../dist/final/node/index.mjs';

/**
 Specialized promisified `execFile`, typed through Node's own adapter contract.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;
const execFileAsync = promisifyExecFile(execFile,);

/**
 One process, as `ps` prints it.

 @param pid - the process number

 @param command - its command line

 @param parentPid - the process that started it

 @returns the process

 @example
 ```ts
 const process = aProcess({ pid: 12, command: 'node a.ts', parentPid: 1 });
 ```
 */
function aProcess({
  pid,
  command,
  parentPid = 1,
}: {
  readonly pid: number;
  readonly command: string;
  readonly parentPid?: number;
},): ProcessUnderARun {
  return {
    pid,
    parentPid,
    elapsedSeconds: 120,
    state: 'S',
    waitingIn: 'do_wait',
    command,
  };
}

/**
 A folder a test throws away.

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
  const path = await mkdtemp(join(scratchRoot(), 'pi-reading-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Writes one stand-in program.

 @param folder - the folder the programs live in

 @param name - the program's name

 @param body - what it does, as a bash body

 @example
 ```ts
 await writeProgram({ folder: bins, name: 'ps', body: 'exit 0', });
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
}): Promise<void> {
  /**
   The program's file.
   */
  const file = join(folder, name,);
  await writeFile(file, `#!/usr/bin/env bash\n${body}\n`,);
  await chmod(file, 0o700,);
}

/**
 Runs one of this package's commands as a program, with the test's own programs ahead of node.

 @param request - the command's name, its words, the folder whose programs it uses, and the environment it gets

 @returns the status and both streams

 @example
 ```ts
 const ran = await runTheCommand({ name: 'list-processes-whose-command-line-holds-text', words: ['--text=x'], folder, environment: {} });
 ```
 */
async function runTheCommand({
  name,
  words,
  folder,
  environment,
}: {
  readonly name: string;
  readonly words: readonly string[];
  readonly folder: {
    readonly path: string;
    readonly bins: string;
  };
  readonly environment: Readonly<Record<string, string>>;
}): Promise<{
  readonly status: number;
  readonly stdout: string;
  readonly stderr: string;
}> {
  /**
   The programs the command may run, then the node that runs it.
   */
  const path = [
    folder.bins,
    dirname(process.execPath,),
    '/usr/bin',
  ].join(delimiter,);
  try {
    /**
     What the command printed.
     */
    const done = await execFileAsync(process.execPath, [
      new URL(`${name}.ts`, import.meta.url,).pathname,
      ...words,
    ], {
      env: {
        PATH: path,
        HOME: folder.path,
        ...environment,
      },
      encoding: 'utf8',
      timeout: 60_000,
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
 Makes a folder with a program folder of its own.

 @returns the folder and where its programs live

 @example
 ```ts
 await using folder = await makeFolderWithPrograms();
 ```
 */
async function makeFolderWithPrograms(): Promise<{
  readonly path: string;
  readonly bins: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  /**
   The folder a test throws away.
   */
  const folder = await makeFolder();
  /**
   Where the stand-in programs live.
   */
  const bins = join(folder.path, 'bin',);
  await mkdir(bins,);
  return {
    path: folder.path,
    bins,
    [Symbol.asyncDispose]: folder[Symbol.asyncDispose],
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: processesWhoseCommandLineHolds.name,
      children: [
        it({
          name: 'finds every process whose command line holds the text, leaves its own process and every other line out, and cuts a long command line',
          fn: async () => {
            /**
             The processes a machine might report.
             */
            const table = [
              aProcess({
                pid: 12,
                command: 'node /runs/start-pi-run-in-a-worktree.ts --name=copy-route',
                parentPid: 3,
              },),
              aProcess({
                pid: 13,
                command: 'node /runs/another.ts',
              },),
              aProcess({
                pid: 14,
                command: `node /runs/reader.ts --text=${'x'.repeat(300,)}`,
              },),
            ];
            /**
             What the reader found.
             */
            const found = processesWhoseCommandLineHolds({
              table,
              text: '--text=',
              excludingPid: 14,
              commandCharacters: 20,
            },);
            expect(found,).toHaveLength(0,);
            /**
             What the reader found for the launcher.
             */
            const launchers = processesWhoseCommandLineHolds({
              table,
              text: 'start-pi-run-in-a-worktree',
              excludingPid: 12,
              commandCharacters: 20,
            },);
            expect(launchers,).toEqual([],);
            /**
             The same question asked by a process that is not the launcher.
             */
            const seen = processesWhoseCommandLineHolds({
              table,
              text: 'start-pi-run-in-a-worktree',
              excludingPid: 1,
              commandCharacters: 20,
            },);
            expect(seen,).toEqual([{
              pid: 12,
              parentPid: 3,
              elapsedSeconds: 120,
              command: 'node /runs/start-pi-...',
            },],);
          },
        },),
        it({
          name: 'says one match as a line naming its process, its parent, its age and its command line',
          fn: async () => {
            expect(matchLineOf({
              pid: 12,
              parentPid: 3,
              elapsedSeconds: 150,
              command: 'node /runs/a.ts',
            },),).toBe('12 parent 3 age 2m node /runs/a.ts',);
          },
        },),
      ],
    },),
    describe({
      name: serviceUnitsIn.name,
      children: [
        it({
          name: 'keeps the whole service units a listing names, and leaves a blank line, a legend and a non-service unit out',
          fn: async () => {
            expect(serviceUnitsIn({
              shown: [
                'pi-copy-route.service loaded active running x',
                'LEGEND',
                '',
                'pi-half.service loaded active running x',
                'session-1.scope loaded active running x',
              ].join('\n',),
            },),).toEqual(['pi-copy-route.service', 'pi-half.service',],);
          },
        },),
      ],
    },),
    describe({
      name: unitsWhoseWorktreeIsGone.name,
      children: [
        it({
          name: 'names every running unit whose worktree folder is gone, asking by the name the launcher would give the folder',
          fn: async () => {
            /**
             The checkout the test's worktrees belong to.
             */
            const checkoutFolder = '/repo';
            /**
             The units that are running now.
             */
            const unitNames = [
              'pi-copy-route.service',
              'pi-second.service',
            ];
            /**
             What the folders answered.
             */
            const there = new Map<string, boolean>([
              [join(checkoutFolder, '.claude', 'worktrees', 'pi-copy-route'), true],
              [join(checkoutFolder, '.claude', 'worktrees', 'pi-second'), false],
            ],);
            expect(await unitsWhoseWorktreeIsGone({
              unitNames,
              checkoutFolder,
              isThere: async function ofFolder(folder,) {
                return there.get(folder,) ?? false;
              },
            },),).toEqual([{
              unitName: 'pi-second.service',
              folder: join(checkoutFolder, '.claude', 'worktrees', 'pi-second'),
            },],);
          },
        },),
        it({
          name: 'says one gone worktree as the line a reader acts on, and says how many running units were listed',
          fn: async () => {
            expect(goneLineOf({
              unitName: 'pi-a.service',
              folder: '/repo/.claude/worktrees/pi-a',
            },),).toBe('GONE /repo/.claude/worktrees/pi-a',);
            expect(countLineOf({
              listed: 3,
              gone: 1,
            },),).toBe('3 running units, 1 without their worktree',);
          },
        },),
      ],
    },),
    describe({
      name: newestFinishedRunOf.name,
      children: [
        it({
          name: 'takes the highest numbered attempt that wrote a summary, leaves one that did not out, and answers that no run of that name finished when none did',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             A finished run of the name.
             */
            await mkdir(join(folder.path, 'copy-route-1',), { recursive: true, },);
            await writeFile(join(folder.path, 'copy-route-1', 'summary.json',), '{}',);
            /**
             A later attempt that has not finished, which is no answer.
             */
            await mkdir(join(folder.path, 'copy-route-2',), { recursive: true, },);
            /**
             A run whose name only starts with the one asked for.
             */
            await mkdir(join(folder.path, 'copy-route-extra-9',), { recursive: true, },);
            await writeFile(join(folder.path, 'copy-route-extra-9', 'summary.json',), '{}',);
            expect(await newestFinishedRunOf({
              runsFolder: folder.path,
              runName: 'copy-route',
            },),).toBe('copy-route-1',);
            expect(await newestFinishedRunOf({
              runsFolder: folder.path,
              runName: 'never-run',
            },),).toBe(NO_FINISHED_RUN_OF_THAT_NAME,);
            expect(await newestFinishedRunOf({
              runsFolder: join(folder.path, 'not-a-folder',),
              runName: 'copy-route',
            },),).toBe(NO_FINISHED_RUN_OF_THAT_NAME,);
          },
        },),
      ],
    },),
    describe({
      name: summaryIn.name,
      children: [
        it({
          name: 'reads the end reason, the turn count and the last message a summary holds, and leaves out a field another tool wrote in another shape',
          fn: async () => {
            expect(summaryIn({ shown: '{"endReason":"idle","turns":40,"lastAssistantText":"done"}', },),).toEqual({
              endReason: 'idle',
              turns: 40,
              lastAssistantText: 'done',
            },);
            expect(summaryIn({ shown: '{"endReason":7,"turns":"many","lastAssistantText":null}', },),).toEqual({},);
            expect(summaryIn({ shown: '[]', },),).toEqual({},);
          },
        },),
      ],
    },),
    describe({
      name: reportLinesOf.name,
      children: [
        it({
          name: 'heads the report with the run, why it ended and its turns, prints the message head, leaves blank lines out, cuts a long line, and says how many more lines there were',
          fn: async () => {
            /**
             A message of more lines than are printed.
             */
            const text = ['first', '', 'second', 'x'.repeat(20,), 'third',].join('\n',);
            expect(reportLinesIn({
              text,
              width: 8,
            },),).toEqual(['first', 'second', 'xxxxxxxx', 'third',],);
            expect(reportLinesOf({
              runFolder: 'copy-route-1',
              summary: {
                endReason: 'idle',
                turns: 40,
                lastAssistantText: text,
              },
              lines: 2,
              width: 8,
            },),).toEqual([
              '=== copy-route-1: idle, 40 turns',
              'first',
              'second',
              '[2 more lines]',
            ],);
            expect(REPORT_LINES,).toBeGreaterThan(0,);
            expect(REPORT_WIDTH,).toBeGreaterThan(0,);
          },
        },),
        it({
          name: 'prints a report with no end reason or turns as the summary left them, and says an agent with no finished run has none',
          fn: async () => {
            expect(reportLinesOf({
              runFolder: 'a-1',
              summary: {},
            },)[0],).toBe('=== a-1: no-end-reason, 0 turns',);
            expect(noFinishedRunLineOf('copy-route',),).toBe('=== copy-route: no finished run',);
          },
        },),
      ],
    },),
    describe({
      name: 'list-processes-whose-command-line-holds-text',
      children: [
        it({
          name: 'prints every process a listing gives, and its own process is never one of them',
          fn: async () => {
            await using folder = await makeFolderWithPrograms();
            await writeProgram({
              folder: folder.bins,
              name: 'ps',
              body: `printf '%s\\n' ' 12 3 120 S - node /runs/launcher.ts --name=copy-route' ' 13 1 60 S - node /runs/other.ts'`,
            },);
            /**
             What the command printed.
             */
            const ran = await runTheCommand({
              name: 'list-processes-whose-command-line-holds-text',
              words: ['--text=launcher.ts',],
              folder,
              environment: {},
            },);
            expect(ran.status,).toBe(0,);
            expect(ran.stdout.trim(),).toBe('12 parent 3 age 2m node /runs/launcher.ts --name=copy-route',);
          },
        },),
        it({
          name: 'says so when no process holds the text, and refuses a line that names no text',
          fn: async () => {
            await using folder = await makeFolderWithPrograms();
            await writeProgram({
              folder: folder.bins,
              name: 'ps',
              body: `printf '%s\\n' ' 13 1 60 S - node /runs/other.ts'`,
            },);
            /**
             What the command printed for a text nothing holds.
             */
            const ran = await runTheCommand({
              name: 'list-processes-whose-command-line-holds-text',
              words: ['--text=nowhere',],
              folder,
              environment: {},
            },);
            expect(ran.status,).toBe(0,);
            expect(ran.stdout.trim(),).toBe('no process whose command line holds nowhere',);
            /**
             What the command answers when it is told no text.
             */
            const refused = await runTheCommand({
              name: 'list-processes-whose-command-line-holds-text',
              words: [],
              folder,
              environment: {},
            },);
            expect(refused.status,).not.toBe(0,);
            expect(refused.stderr,).toContain('--text is required',);
          },
        },),
      ],
    },),
    describe({
      name: 'list-running-units-whose-worktree-is-gone',
      children: [
        it({
          name: 'names the running unit whose worktree folder is gone, and counts the units it listed',
          fn: async () => {
            await using folder = await makeFolderWithPrograms();
            await writeProgram({
              folder: folder.bins,
              name: 'systemctl',
              body: `printf '%s\\n' 'pi-copy-route.service loaded active running x' 'pi-gone.service loaded active running x'`,
            },);
            /**
             The one worktree that is still there.
             */
            await mkdir(join(folder.path, '.claude', 'worktrees', 'pi-copy-route',), { recursive: true, },);
            /**
             What the command printed.
             */
            const ran = await runTheCommand({
              name: 'list-running-units-whose-worktree-is-gone',
              words: [`--checkout-folder=${folder.path}`,],
              folder,
              environment: {},
            },);
            expect(ran.status,).toBe(0,);
            expect(ran.stdout.trim(),).toBe([
              `GONE ${join(folder.path, '.claude', 'worktrees', 'pi-gone',)}`,
              '2 running units, 1 without their worktree',
            ].join('\n',),);
          },
        },),
        it({
          name: 'refuses a listing systemd would not give, naming what it said',
          fn: async () => {
            await using folder = await makeFolderWithPrograms();
            await writeProgram({
              folder: folder.bins,
              name: 'systemctl',
              body: `printf '%s\\n' 'Failed to connect to bus' >&2; exit 1`,
            },);
            /**
             What the command answered.
             */
            const refused = await runTheCommand({
              name: 'list-running-units-whose-worktree-is-gone',
              words: [`--checkout-folder=${folder.path}`,],
              folder,
              environment: {},
            },);
            expect(refused.status,).not.toBe(0,);
            expect(refused.stderr,).toContain('Failed to connect to bus',);
          },
        },),
      ],
    },),
    describe({
      name: 'print-the-final-report-of-the-newest-run-of-each-agent',
      children: [
        it({
          name: 'prints one block per agent, from the newest run that finished, and says which agent has none',
          fn: async () => {
            await using folder = await makeFolderWithPrograms();
            /**
             A finished run of one agent.
             */
            await mkdir(join(folder.path, 'copy-route-1',), { recursive: true, },);
            await writeFile(join(folder.path, 'copy-route-1', 'summary.json',), JSON.stringify({
              endReason: 'idle',
              turns: 40,
              lastAssistantText: 'the work is done\nand pushed',
            },),);
            /**
             What the command printed.
             */
            const ran = await runTheCommand({
              name: 'print-the-final-report-of-the-newest-run-of-each-agent',
              words: [
                '--agent=copy-route',
                '--agent=never-run',
                `--runs-folder=${folder.path}`,
              ],
              folder,
              environment: {},
            },);
            expect(ran.status,).toBe(0,);
            expect(ran.stdout.trim(),).toBe([
              '=== copy-route-1: idle, 40 turns',
              'the work is done',
              'and pushed',
              '=== never-run: no finished run',
            ].join('\n',),);
          },
        },),
        it({
          name: 'refuses a line that names no agent, and one whose width is no number',
          fn: async () => {
            await using folder = await makeFolderWithPrograms();
            /**
             What the command answers with no agent named.
             */
            const refused = await runTheCommand({
              name: 'print-the-final-report-of-the-newest-run-of-each-agent',
              words: [`--runs-folder=${folder.path}`,],
              folder,
              environment: {},
            },);
            expect(refused.status,).not.toBe(0,);
            expect(refused.stderr,).toContain('--agent is required at least once',);
            /**
             What it answers with a width it cannot use.
             */
            const badWidth = await runTheCommand({
              name: 'print-the-final-report-of-the-newest-run-of-each-agent',
              words: ['--agent=a', '--width=wide',],
              folder,
              environment: {},
            },);
            expect(badWidth.status,).not.toBe(0,);
            expect(badWidth.stderr,).toContain('--width must be a number above zero',);
          },
        },),
      ],
    },),
  ],
},);
