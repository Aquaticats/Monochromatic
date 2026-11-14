/**
 The work a run left running: what its own `process` tool reported, what the machine's process table shows in the run's folders
 outside the run's own unit, and the message that sends the agent back to it. Nothing here starts a process or a unit.

 @module
 */

import {
  mkdir,
  mkdtemp,
  readFile,
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
  FEED_PREVIEW_CHARACTERS,
  preview,
  type PiRecord,
  type RunCommand,
  cgroupOf,
  startRecordOfPiRun,
  unitNamedBy,
  wakeMessageOf,
  workOutlivingTheTurnIn,
  workTheToolReportedIn,
} from '../dist/final/node/index.mjs';

/**
 The folder the run works in, which its processes work in too.

 @returns the folder's path

 @example
 ```ts
 const folder = aWorktree();
 ```
 */
function aWorktree(): string {
  return '/repo/.claude/worktrees/pi-copy-route';
}

/**
 The run's own folder, where its feed is.

 @returns the folder's path

 @example
 ```ts
 const folder = aRunFolder();
 ```
 */
function aRunFolder(): string {
  return '/runs/copy-route-1';
}

/**
 The control group of the run's own unit, which the driver and Pi are in.

 @returns the control group's text, as the kernel writes it

 @example
 ```ts
 const cgroup = theRunsUnit();
 ```
 */
function theRunsUnit(): string {
  return '0::/user.slice/user-1000.slice/user@1000.service/app.slice/pi-copy-route.service';
}

/**
 The control group of a scope the agent made, which is not the run's own unit.

 @returns the control group's text, as the kernel writes it

 @example
 ```ts
 const cgroup = aScopeOfItsOwn();
 ```
 */
function aScopeOfItsOwn(): string {
  return '0::/user.slice/user-1000.slice/user@1000.service/app.slice/run-p486667-i392.scope';
}

/**
 One line of a feed written before the driver kept a tool's answer whole, holding the answer itself in `result`.

 A feed written now keeps only a preview of the answer there, which is why the driver writes a `processes` record beside it: the
 preview of an answer longer than a feed line is cut inside a JSON string and cannot be read back.

 @param details - what the tool's answer holds

 @param isError - whether the tool answered with an error

 @returns the feed line

 @example
 ```ts
 const line = aToolAnswer({ action: 'list', processes: [], },);
 ```
 */
function aToolAnswer(details: unknown, isError = false,): string {
  return JSON.stringify({
    kind: 'tool_end',
    tool: 'process',
    isError,
    result: JSON.stringify({ details, },),
  },);
}

/**
 What one answer of the `process` tool holds when the agent read a build's log: text longer than a feed line keeps, beside the
 process it is about.

 @param process - the process the answer is about

 @returns the answer

 @example
 ```ts
 const answer = aLongAnswerAbout(aProcess({},),);
 ```
 */
function aLongAnswerAbout(process: Readonly<Record<string, unknown>>,): PiRecord {
  return {
    content: [{
      type: 'text',
      // A real answer of this kind carries the tail of a build's log, which is what makes it longer than a feed line holds: the
      // preview of it ends inside a JSON string and cannot be read back.
      text: 'x'.repeat(FEED_PREVIEW_CHARACTERS.toolResult * 2,),
    },],
    details: {
      action: 'output',
      process,
    },
  };
}

/**
 Writes a feed through the driver's own record, from the events Pi emits, so a test reads what a real run left on disk rather
 than a shape a test made up.

 @param folder - the run folder a test owns

 @param answers - one `process` tool answer per event, in the order the run produced them

 @returns the feed's text, as a reader of it sees it

 @example
 ```ts
 const shown = await aFeedTheDriverWrote({ folder: '/runs/a-1', answers: [aLongAnswerAbout(aProcess({},),)], },);
 ```
 */
async function aFeedTheDriverWrote({
  folder,
  answers,
}: {
  readonly folder: string;
  readonly answers: readonly PiRecord[];
},): Promise<string> {
  /**
   The feed the driver writes.
   */
  const feedFile = join(folder, 'feed.jsonl',);
  /**
   The record of that run.
   */
  const record = startRecordOfPiRun({ feedFile, },);
  for (const result of answers) {
    record.observe({
      event: {
        type: 'tool_execution_end',
        toolName: 'process',
        isError: false,
        result,
      },
      declineDialog: function declined() {
        // No dialog is asked in an event of this kind.
      },
    },);
  }
  await record.close();
  return await readFile(feedFile, 'utf8',);
}

/**
 One process, as the `process` tool records it.

 @param fields - what to override

 @returns the process

 @example
 ```ts
 const process = aProcess({},);
 ```
 */
function aProcess(fields: Readonly<Record<string, unknown>> = {},): Readonly<Record<string, unknown>> {
  return {
    id: 'proc_7adb',
    name: 'native-build-2',
    pid: 486_667,
    command: `cd ${aWorktree()} && systemd-run --user --scope -- mise run build`,
    cwd: aWorktree(),
    status: 'running',
    ...fields,
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
  const path = await mkdtemp(join(scratchRoot(), 'pi-work-left-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Writes one process into a process folder a test owns.

 @param folder - the process folder

 @param pid - the process's number

 @param cgroup - the control group it is in

 @param worksIn - the folder it works in

 @param command - the program the kernel names for it


 @returns nothing

 @example
 ```ts
 await writeProcess({ folder: '/proc', pid: 12, cgroup: '0::/a.scope', worksIn: '/repo', command: 'bash', },);
 ```
 */
async function writeProcess({
  folder,
  pid,
  cgroup,
  worksIn,
  command,
}: {
  readonly folder: string;
  readonly pid: number;
  readonly cgroup: string;
  readonly worksIn: string;
  readonly command: string;
},): Promise<void> {
  await mkdir(join(
    folder,
    String(pid,),
  ), { recursive: true, },);
  await writeFile(
    join(
      folder,
      String(pid,),
      'cgroup',
    ),
    `${cgroup}\n`,
  );
  await writeFile(
    join(
      folder,
      String(pid,),
      'comm',
    ),
    `${command}\n`,
  );
  await symlink(
    worksIn,
    join(
      folder,
      String(pid,),
      'cwd',
    ),
    'dir',
  );
}

/**
 Writes the driver's own entry into a process folder a test owns, so the run's own unit is known.

 @param folder - the process folder

 @param cgroup - the control group of the run's own unit

 @returns nothing

 @example
 ```ts
 await writeSelf({ folder: '/proc', cgroup: theRunsUnit(), },);
 ```
 */
async function writeSelf({
  folder,
  cgroup,
}: {
  readonly folder: string;
  readonly cgroup: string;
},): Promise<void> {
  await mkdir(join(folder, 'self',), { recursive: true, },);
  await writeFile(
    join(folder, 'self', 'cgroup',),
    `${cgroup}\n`,
  );
}


/**
 A manager that answers what it says about every unit it is asked about.

 @param request - what it answers, and where to note what it was asked

 @returns how a command is run for this read

 @example
 ```ts
 const run = aManagerAnswering({ status: 0, stdout: 'active\n', asked, },);
 ```
 */
function aManagerAnswering({
  status,
  stdout,
  asked = [],
}: {
  readonly status: number;
  readonly stdout: string;
  readonly asked?: string[][];
},): RunCommand {
  return async function answered(request,) {
    asked.push([...request.words,],);
    return {
      status,
      stdout,
      stderr: '',
    } satisfies CommandOutput;
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: workTheToolReportedIn.name,
      children: [
        it({
          name: 'reads the process one tool answer names, and drops it once a later answer reports it ended',
          fn: async () => {
            /**
             A feed whose tool started a process and later reported it gone.
             */
            const shown = [
              aToolAnswer({
                action: 'start',
                process: aProcess({},),
              },),
              aToolAnswer({
                action: 'update',
                process: aProcess({},),
              },),
              aToolAnswer({
                action: 'list',
                processes: [aProcess({ status: 'exited', exitCode: 0, },),],
              },),
              '',
            ].join('\n',);
            expect(workTheToolReportedIn({ feedShown: shown, },),).toEqual([],);
          },
        },),
        it({
          name: 'reads one process out of a list, with the folder and command the tool recorded, and leaves a failed answer out',
          fn: async () => {
            /**
             A feed with a list of two processes, one of them running, and an answer the tool failed.
             */
            const shown = [
              aToolAnswer({
                action: 'list',
                processes: [
                  aProcess({},),
                  aProcess({
                    id: 'proc_8bce',
                    name: 'clippy',
                    pid: 486_668,
                    status: 'exited',
                  },),
                ],
              },),
              JSON.stringify({
                kind: 'tool_end',
                tool: 'process',
                isError: true,
                result: JSON.stringify({ details: { action: 'start', process: aProcess({ id: 'proc_bad', },), }, },),
              },),
              '',
            ].join('\n',);
            expect(workTheToolReportedIn({ feedShown: shown, },),).toEqual([{
              id: 'proc_7adb',
              name: 'native-build-2',
              pid: 486_667,
              command: `cd ${aWorktree()} && systemd-run --user --scope -- mise run build`,
              cwd: aWorktree(),
              status: 'running',
            },],);
          },
        },),
        it({
          name: 'reads the process a tool answered with from the feed the driver writes, where the answer is longer than a line keeps',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The feed a run of this driver leaves behind.
             */
            const shown = await aFeedTheDriverWrote({
              folder: folder.path,
              answers: [aLongAnswerAbout(aProcess({},),),],
            },);
            /**
             The tool_end line the feed kept, whose result is only a preview of that answer.
             */
            const toolEnd = shown.split('\n',)
              .filter((line,) => line !== '')
              .map((line,) => JSON.parse(line,) as PiRecord,)
              .find((entry,) => entry.kind === 'tool_end',);
            // The fixture is the defect: the line the feed keeps holds text that is no JSON document, so a reader parsing it
            // throws `Unterminated string in JSON`. The record beside it holds the answer whole.
            expect(function parsedTheLine() {
              return JSON.parse(String(toolEnd?.result,),) as unknown;
            },).toThrow(SyntaxError,);
            expect(workTheToolReportedIn({ feedShown: shown, },),).toEqual([{
              id: 'proc_7adb',
              name: 'native-build-2',
              pid: 486_667,
              command: `cd ${aWorktree()} && systemd-run --user --scope -- mise run build`,
              cwd: aWorktree(),
              status: 'running',
            },],);
          },
        },),
        it({
          name: 'reads no work from a feed that kept only a cut preview of the tool answer',
          fn: async () => {
            /**
             A feed written before the record of a whole tool answer existed, whose one answer is longer than a line keeps.
             */
            const shown = `${JSON.stringify({
              kind: 'tool_end',
              tool: 'process',
              isError: false,
              result: preview({
                value: aLongAnswerAbout(aProcess({},),),
                length: FEED_PREVIEW_CHARACTERS.toolResult,
              },),
            },)}\n`;
            expect(workTheToolReportedIn({ feedShown: shown, },),).toEqual([],);
          },
        },),
        it({
          name: 'reads the work a feed\'s whole records name while the driver is still writing the line after them',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             A feed the driver wrote.
             */
            const written = await aFeedTheDriverWrote({
              folder: folder.path,
              answers: [aLongAnswerAbout(aProcess({},),),],
            },);
            /**
             The same feed as a reader sees it while the driver is still writing its next record.
             */
            const beingWritten = `${written}{"kind":"processes","processes":[{"id":"proc_9`;
            expect(workTheToolReportedIn({ feedShown: beingWritten, },),).toEqual([{
              id: 'proc_7adb',
              name: 'native-build-2',
              pid: 486_667,
              command: `cd ${aWorktree()} && systemd-run --user --scope -- mise run build`,
              cwd: aWorktree(),
              status: 'running',
            },],);
          },
        },),
        it({
          name: 'names a whole line that is no record rather than reading the run\'s work as none',
          fn: async () => {
            expect(
              () =>
                workTheToolReportedIn({
                  feedShown: `${aToolAnswer({ action: 'list', processes: [aProcess({},),], },)}\nnot a record at all\n`,
                },),
            ).toThrow(SyntaxError,);
          },
        },),
      ],
    },),
    describe({
      name: unitNamedBy.name,
      children: [
        it({
          name: 'reads the unit a command starts of its own, in either spelling, and answers nothing when it names none',
          fn: async () => {
            expect(unitNamedBy({ command: 'systemd-run --user --unit=pi-build -- true', },),).toEqual({ unit: 'pi-build', },);
            expect(unitNamedBy({ command: 'systemd-run --user --unit pi-build -- true', },),).toEqual({ unit: 'pi-build', },);
            expect(unitNamedBy({ command: 'systemd-run --user --scope -- true', },),).toEqual({},);
            expect(unitNamedBy({ command: 'mise run build', },),).toEqual({},);
          },
        },),
      ],
    },),
    describe({
      name: workOutlivingTheTurnIn.name,
      children: [
        it({
          name: 'names a scope the agent made that works in the run folders, and counts neither the run own unit nor another folder',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The process folder a test writes, whose own entry is the run's unit.
             */
            const processes = join(folder.path, 'proc',);
            await writeSelf({
              folder: processes,
              cgroup: theRunsUnit(),
            },);
            await writeProcess({
              folder: processes,
              pid: 100,
              cgroup: theRunsUnit(),
              worksIn: aWorktree(),
              command: 'node',
            },);
            // The build the agent started in a scope of its own, which left the run's unit and kept the run's folder.
            await writeProcess({
              folder: processes,
              pid: 101,
              cgroup: aScopeOfItsOwn(),
              worksIn: aWorktree(),
              command: 'bash',
            },);
            await writeProcess({
              folder: processes,
              pid: 102,
              cgroup: aScopeOfItsOwn(),
              worksIn: '/elsewhere',
              command: 'sleep',
            },);
            expect(await workOutlivingTheTurnIn({
              runFolder: folder.path,
              workingFolder: aWorktree(),
              processFolder: processes,
              run: async function answered() {
                return {
                  status: 1,
                  stdout: 'inactive\n',
                  stderr: '',
                };
              },
            },),).toEqual(['101 bash',],);
            expect(await cgroupOf({
              processFolder: processes,
              entry: '100',
            },),).toBe(theRunsUnit(),);
            expect(await cgroupOf({
              processFolder: processes,
              entry: 'nobody',
            },),).toBe('',);
          },
        },),
        it({
          name: 'names the work the process tool reported while its number is still there, and asks no manager about it',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The process folder a test writes, whose own entry is the run's unit.
             */
            const processes = join(folder.path, 'proc',);
            await writeSelf({
              folder: processes,
              cgroup: theRunsUnit(),
            },);
            await writeProcess({
              folder: processes,
              pid: 101,
              cgroup: theRunsUnit(),
              worksIn: aWorktree(),
              command: 'bash',
            },);
            await writeFile(
              join(folder.path, 'feed.jsonl',),
              `${aToolAnswer({ action: 'list', processes: [aProcess({ pid: 101, },),], },)}\n`,
            );
            expect(await workOutlivingTheTurnIn({
              runFolder: folder.path,
              workingFolder: aWorktree(),
              processFolder: processes,
              run: async function answered() {
                return {
                  status: 1,
                  stdout: 'inactive\n',
                  stderr: '',
                };
              },
            },),).toEqual(['proc_7adb (native-build-2) 101 cd /repo/.claude/worktrees/pi-copy-route && systemd-run --user --scope -- mise run build',],);
          },
        },),
        it({
          name: 'asks the manager about a unit the tool named when the number it held has gone, and answers nothing when that unit is not active',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The process folder a test writes, which holds no process of the run.
             */
            const processes = join(folder.path, 'proc',);
            await mkdir(processes, { recursive: true, },);
            await writeFile(
              join(folder.path, 'feed.jsonl',),
              `${aToolAnswer({
                action: 'start',
                process: aProcess({
                  command: 'systemd-run --user --unit=pi-build-2 -- mise run build',
                  pid: 999_999,
                },),
              },)}\n`,
            );
            /**
             What the manager was asked, and what it answers.
             */
            const asked: string[][] = [];
            /**
             A manager that holds the unit.
             */
            const holding = aManagerAnswering({
              status: 0,
              stdout: 'active\n',
              asked,
            },);
            expect(await workOutlivingTheTurnIn({
              runFolder: folder.path,
              workingFolder: aWorktree(),
              processFolder: processes,
              run: holding,
            },),).toEqual(['proc_7adb (native-build-2) 999999 systemd-run --user --unit=pi-build-2 -- mise run build',],);
            expect(asked,).toEqual([[
              '--user',
              'is-active',
              'pi-build-2',
            ],],);
            /**
             A manager that holds nothing.
             */
            const holdingNothing = aManagerAnswering({
              status: 3,
              stdout: 'inactive\n',
            },);
            expect(await workOutlivingTheTurnIn({
              runFolder: folder.path,
              workingFolder: aWorktree(),
              processFolder: processes,
              run: holdingNothing,
            },),).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: wakeMessageOf.name,
      children: [
        it({
          name: 'names every process that ended, says the driver watched the process table, and asks for the agent to go on',
          fn: async () => {
            /**
             The lines that named the work when the driver began to wait.
             */
            const work = ['proc_7adb (native-build-2) 486667 systemd-run --user --scope', '101 bash',];
            expect(wakeMessageOf({ work, },),).toContain('proc_7adb (native-build-2) 486667 systemd-run --user --scope',);
            expect(wakeMessageOf({ work, },),).toContain('- 101 bash',);
            expect(wakeMessageOf({ work, },),).toContain('`process` tool',);
            expect(wakeMessageOf({ work, },),).toContain('continue your task',);
          },
        },),
      ],
    },),
  ],
},);
