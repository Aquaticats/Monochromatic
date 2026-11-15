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
  type ProcessFolderReads,
  ProcessFileUnreadableError,
  type RunCommand,
  cgroupOf,
  readsOfTheProcessFolder,
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
 The start time a fixture gives both a recorded process and the process at its number, in clock ticks since boot.

 @returns the start time

 @example
 ```ts
 const birth = aBirth();
 ```
 */
function aBirth(): string {
  return '20004601';
}

/**
 The start time a later process at the same number has, which no record of the first process repeats.

 @returns the start time

 @example
 ```ts
 const birth = aLaterBirth();
 ```
 */
function aLaterBirth(): string {
  return '21000000';
}

/**
 How many fields a line of `/proc/<pid>/stat` holds between the state and the start time, whose values nothing here reads.
 */
const FIELDS_BETWEEN_STATE_AND_BIRTH = 18;

/**
 The line `/proc/<pid>/stat` holds for a process a test wrote, as the kernel writes it.

 @param state - the state the kernel gives it

 @param birth - its start time in clock ticks since boot

 @returns the line

 @example
 ```ts
 const line = aStatLine({ state: 'S', birth: aBirth(), },);
 ```
 */
function aStatLine({
  state,
  birth,
}: {
  readonly state: string;
  readonly birth: string;
},): string {
  /**
   The fields after the program's name: the state first, and the start time twentieth.
   */
  const afterTheName = [
    state,
    ...Array.from(
      { length: FIELDS_BETWEEN_STATE_AND_BIRTH, },
      function aFieldNothingReads() {
        return '0';
      },
    ),
    birth,
  ];
  return `4242 (a-program) ${afterTheName.join(' ',)}\n`;
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

 @param birth - the start time the kernel gave it

 @param state - the state the kernel gives it

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
  birth = aBirth(),
  state = 'S',
}: {
  readonly folder: string;
  readonly pid: number;
  readonly cgroup: string;
  readonly worksIn: string;
  readonly command: string;
  readonly birth?: string;
  readonly state?: string;
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
  await writeFile(
    join(
      folder,
      String(pid,),
      'stat',
    ),
    aStatLine({
      state,
      birth,
    },),
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
 A failure that carries the code the operating system gave a read of a process's own file.
 */
class AReadThatFailedError extends Error {
  /**
   The code the operating system gave, which is what a reader of a process folder decides with.
   */
  readonly code: string;

  /**
   Keeps the code and the file in the shape the operating system gives them.

   @param path - the file the read was about

   @param code - the code the read failed with
   */
  constructor({
    path,
    code,
  }: {
    readonly path: string;
    readonly code: string;
  },) {
    super(`${code}: no such file, read '${path}'`,);
    this.name = 'AReadThatFailedError';
    this.code = code;
  }
}

/**
 Reads a process folder a test wrote, answering the reads of one path with the code a process that has ended gives.

 This is a reader a test supplies in place of the operating system's, so a scan over a process that ends while it is read is
 checked without racing a real process for the moment.

 @param folder - the process folder the test wrote

 @param entry - the entry whose reads fail with that code

 @param code - the code those reads fail with, `ESRCH` for a process that ended after its file was opened

 @param failingFile - the one file of that entry whose reads fail, or nothing for every read of the entry

 @returns how a reader of one process answers for this folder

 @example
 ```ts
 const reads = readsAnsweringFor({ folder: processes, entry: '102', code: 'ESRCH', },);
 ```
 */
function readsAnsweringFor({
  folder,
  entry,
  code,
  failingFile = '',
}: {
  readonly folder: string;
  readonly entry: string;
  readonly code: string;
  readonly failingFile?: string;
},): ProcessFolderReads {
  /**
   What a failing path starts with: the process's own folder and its separator, or one of its files.
   */
  const gone = (failingFile === '')
    ? `${join(
      folder,
      entry,
    )}/`
    : join(
      folder,
      entry,
      failingFile,
    );
  return {
    workingFolderOf: async function workingFolderOf(path,) {
      if (path.startsWith(gone,)) {
        throw new AReadThatFailedError({
          path,
          code,
        },);
      }
      return await readsOfTheProcessFolder.workingFolderOf(path,);
    },
    textOf: async function textOf(path,) {
      if (path.startsWith(gone,)) {
        throw new AReadThatFailedError({
          path,
          code,
        },);
      }
      return await readsOfTheProcessFolder.textOf(path,);
    },
  };
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
          name: 'names the work the process tool reported while its number still holds that very process, and asks no manager about it',
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
              birth: aBirth(),
            },);
            await writeFile(
              join(folder.path, 'feed.jsonl',),
              `${aToolAnswer({
                action: 'list',
                processes: [aProcess({
                  pid: 101,
                  birth: aBirth(),
                },),],
              },)}\n`,
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
          name: 'counts recorded work as gone when a later process holds its number, and asks no manager about it',
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
            // The process at the number the build held, which is a later one: its start time is its own, and it works elsewhere.
            await writeProcess({
              folder: processes,
              pid: 101,
              cgroup: aScopeOfItsOwn(),
              worksIn: '/elsewhere',
              command: 'unrelated',
              birth: aLaterBirth(),
            },);
            await writeFile(
              join(folder.path, 'feed.jsonl',),
              `${aToolAnswer({
                action: 'start',
                process: aProcess({
                  pid: 101,
                  birth: aBirth(),
                },),
              },)}\n`,
            );
            expect(await workOutlivingTheTurnIn({
              runFolder: folder.path,
              workingFolder: aWorktree(),
              processFolder: processes,
              run: async function answered() {
                throw new Error('no manager is asked about work whose process is not the recorded one',);
              },
            },),).toEqual([],);
          },
        },),
        it({
          name: 'counts recorded work as gone when the record kept no start time for it',
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
              cgroup: aScopeOfItsOwn(),
              worksIn: '/elsewhere',
              command: 'bash',
              birth: aBirth(),
            },);
            // A record written before the driver kept a start time, which cannot say which process it named.
            await writeFile(
              join(folder.path, 'feed.jsonl',),
              `${aToolAnswer({
                action: 'start',
                process: aProcess({ pid: 101, },),
              },)}\n`,
            );
            expect(await workOutlivingTheTurnIn({
              runFolder: folder.path,
              workingFolder: aWorktree(),
              processFolder: processes,
              run: async function answered() {
                throw new Error('no manager is asked about work that names no unit',);
              },
            },),).toEqual([],);
          },
        },),
        it({
          name: 'counts recorded work as gone when its process has ended and waits to be reaped',
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
            // The very process the record named, which has ended and whose parent has not reaped it: a zombie does no work.
            await writeProcess({
              folder: processes,
              pid: 101,
              cgroup: aScopeOfItsOwn(),
              worksIn: '/elsewhere',
              command: 'bash',
              birth: aBirth(),
              state: 'Z',
            },);
            await writeFile(
              join(folder.path, 'feed.jsonl',),
              `${aToolAnswer({
                action: 'start',
                process: aProcess({
                  pid: 101,
                  birth: aBirth(),
                },),
              },)}\n`,
            );
            expect(await workOutlivingTheTurnIn({
              runFolder: folder.path,
              workingFolder: aWorktree(),
              processFolder: processes,
              run: async function answered() {
                throw new Error('no manager is asked about work that names no unit',);
              },
            },),).toEqual([],);
          },
        },),
        it({
          name: 'reports a read of the recorded work\'s own line that failed for a reason other than the process having gone',
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
              cgroup: aScopeOfItsOwn(),
              worksIn: '/elsewhere',
              command: 'bash',
              birth: aBirth(),
            },);
            await writeFile(
              join(folder.path, 'feed.jsonl',),
              `${aToolAnswer({
                action: 'start',
                process: aProcess({
                  pid: 101,
                  birth: aBirth(),
                },),
              },)}\n`,
            );
            /**
             The reader of the process folder, whose read of that process's own line fails as a damaged table would.
             */
            const reads = readsAnsweringFor({
              folder: processes,
              entry: '101',
              code: 'EIO',
              failingFile: 'stat',
            },);
            await expect(workOutlivingTheTurnIn({
              runFolder: folder.path,
              workingFolder: aWorktree(),
              processFolder: processes,
              reads,
              run: async function answered() {
                throw new Error('no manager is asked about a read that failed',);
              },
            },),).rejects.toBeInstanceOf(ProcessFileUnreadableError,);
            await expect(workOutlivingTheTurnIn({
              runFolder: folder.path,
              workingFolder: aWorktree(),
              processFolder: processes,
              reads,
              run: async function answered() {
                throw new Error('no manager is asked about a read that failed',);
              },
            },),).rejects.toHaveProperty('code', 'EIO',);
            await expect(workOutlivingTheTurnIn({
              runFolder: folder.path,
              workingFolder: aWorktree(),
              processFolder: processes,
              reads,
              run: async function answered() {
                throw new Error('no manager is asked about a read that failed',);
              },
            },),).rejects.toHaveProperty('path', join(
              processes,
              '101',
              'stat',
            ),);
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
        it({
          name: 'counts a process that ended at any instant of the scan as gone, and goes on with the others',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The process folder a test writes, holding three processes that work in the run's folder.
             */
            const processes = join(folder.path, 'proc',);
            await writeSelf({
              folder: processes,
              cgroup: theRunsUnit(),
            },);
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
              worksIn: aWorktree(),
              command: 'make',
            },);
            await writeProcess({
              folder: processes,
              pid: 103,
              cgroup: aScopeOfItsOwn(),
              worksIn: aWorktree(),
              command: 'sleep',
            },);
            /**
             The reader that answers for a process which ended before its folder was read, which is every read of it.
             */
            const endedBeforeItsFolderWasRead = readsAnsweringFor({
              folder: processes,
              entry: '102',
              code: 'ESRCH',
            },);
            /**
             The reader that answers for a process which ended between the two reads of it: its folder was read and its name file
             was not.
             */
            const endedBetweenItsTwoReads = readsAnsweringFor({
              folder: processes,
              entry: '103',
              code: 'ESRCH',
              failingFile: 'comm',
            },);
            /**
             The reader of the whole scan, whose one process that is still there answers as the operating system does.
             */
            const reads: ProcessFolderReads = {
              workingFolderOf: async function workingFolderOf(path,) {
                return await endedBeforeItsFolderWasRead.workingFolderOf(path,);
              },
              textOf: async function textOf(path,) {
                return await endedBetweenItsTwoReads.textOf(path,);
              },
            };
            expect(await workOutlivingTheTurnIn({
              runFolder: folder.path,
              workingFolder: aWorktree(),
              processFolder: processes,
              reads,
              run: async function answered() {
                return {
                  status: 3,
                  stdout: 'inactive\n',
                  stderr: '',
                };
              },
            },),).toEqual(['101 bash',],);
          },
        },),
        it({
          name: 'counts a process whose control group cannot be read because it has gone as gone, and reports a failure that is not that',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The process folder a test writes, holding one process that ended while the scan read it.
             */
            const processes = join(folder.path, 'proc',);
            await writeProcess({
              folder: processes,
              pid: 102,
              cgroup: aScopeOfItsOwn(),
              worksIn: aWorktree(),
              command: 'sleep',
            },);
            expect(await cgroupOf({
              processFolder: processes,
              entry: '102',
              reads: readsAnsweringFor({
                folder: processes,
                entry: '102',
                code: 'ESRCH',
              },),
            },),).toBe('',);
            expect(await cgroupOf({
              processFolder: processes,
              entry: '102',
              reads: readsAnsweringFor({
                folder: processes,
                entry: '102',
                code: 'ENOENT',
              },),
            },),).toBe('',);
            /**
             A read that failed for a reason which is no process having gone names the path and the code, so a reader of a run's
             summary can see what stopped the scan.
             */
            await expect(cgroupOf({
              processFolder: processes,
              entry: '102',
              reads: readsAnsweringFor({
                folder: processes,
                entry: '102',
                code: 'EIO',
              },),
            },),).rejects.toThrow('EIO',);
            await expect(cgroupOf({
              processFolder: processes,
              entry: '102',
              reads: readsAnsweringFor({
                folder: processes,
                entry: '102',
                code: 'EIO',
              },),
            },),).rejects.toBeInstanceOf(ProcessFileUnreadableError,);
            await expect(cgroupOf({
              processFolder: processes,
              entry: '102',
              reads: readsAnsweringFor({
                folder: processes,
                entry: '102',
                code: 'EIO',
              },),
            },),).rejects.toHaveProperty('path', join(processes, '102', 'cgroup',),);
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
