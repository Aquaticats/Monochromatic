/**
 The work a run left running: what its own `process` tool reported, what the machine's process table shows in the run's folders
 outside the run's own unit, and the message that sends the agent back to it. Nothing here starts a process or a unit.

 @module
 */

import {
  mkdir,
  mkdtemp,
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
  type RunCommand,
  cgroupOf,
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
 One line of a run's feed, holding what the `process` tool answered.

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
            expect(workTheToolReportedIn({ feedShown: shown, },),).toEqual({
              running: [],
              unreadableAnswers: [],
            },);
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
            expect(workTheToolReportedIn({ feedShown: shown, },),).toEqual({
              running: [{
                id: 'proc_7adb',
                name: 'native-build-2',
                pid: 486_667,
                command: `cd ${aWorktree()} && systemd-run --user --scope -- mise run build`,
                cwd: aWorktree(),
                status: 'running',
              },],
              unreadableAnswers: [],
            },);
          },
        },),
      ],
    },),
    describe({
      name: workTheToolReportedIn.name,
      children: [
        it({
          name: 'reads a process out of an answer the feed holds only a cut copy of, which is the answer that ended two finished runs as driver-failed, and never throws',
          fn: async () => {
            /**
             A feed whose one answer is cut where the feed cuts it: 500 characters of the tool's own answer and the marker for the
             rest, as `~/temp/agent/pi-runs/optique-two-remaining-copies-prototype-1/feed.jsonl` line 289 holds it.
             */
            const shown = JSON.stringify({
              kind: 'tool_end',
              tool: 'process',
              isError: false,
              result: '{"content":[{"type":"text","text":"Started process deno-core-suite (proc_a88e) with pid 284377."}],'
                + '"details":{"action":"start","process":{"id":"proc_a88e","name":"deno-core-suite","pid":284377,'
                + '"command":"W=/home/user/temp/agent/upstream-prototype.3lsGY9v7; podman run --rm --cpus 2 ...[1937]',
            },);
            /**
             What the function answered, which must be an answer rather than an exception.
             */
            const reported = workTheToolReportedIn({ feedShown: `${shown}\n`, },);
            expect(reported.unreadableAnswers,).toEqual([],);
            expect(reported.running,).toEqual([{
              id: 'proc_a88e',
              name: 'deno-core-suite',
              pid: 284_377,
              command: '',
              cwd: '',
              // The cut fell inside the command, so the status is the process table's to say and the tool's no longer.
              statusIsUnknown: true,
            },],);
          },
        },),
        it({
          name: 'reads the process an output answer names out of the flat details a cut copy still holds',
          fn: async () => {
            /**
             A feed whose one answer is an `output` answer, cut inside the file name it ends with.
             */
            const shown = JSON.stringify({
              kind: 'tool_end',
              tool: 'process',
              isError: false,
              result: `${String.raw`{"content":[{"type":"text","text":"\"deno-core-suite\" (proc_a88e) [running]"}],`
                 }"details":{"action":"output","id":"proc_a88e","processName":"deno-core-suite",`
                + `"processStatus":"running","stream":"both","tailLines":40,"pattern":null,"mode":"literal",`
                + `"stdoutFile":"/tmp/pi-proce...[920]`,
            },);
            expect(workTheToolReportedIn({ feedShown: `${shown}\n`, },).running,).toEqual([{
              id: 'proc_a88e',
              name: 'deno-core-suite',
              command: '',
              cwd: '',
              status: 'running',
            },],);
          },
        },),
        it({
          name: 'counts an answer nothing can be read from, so a process it started is work that may still run, and reads no line of the feed it cannot parse',
          fn: async () => {
            /**
             A feed whose answer is cut inside its first string, and one line that is no record at all.
             */
            const shown = [
              JSON.stringify({
                kind: 'tool_end',
                tool: 'process',
                isError: false,
                result: '{"content":[{"type":"text","text\\',
              },),
              'not a record',
              '',
            ].join('\n',);
            /**
             What the function answered about a feed with both kinds of unreadable line.
             */
            const reported = workTheToolReportedIn({ feedShown: shown, },);
            expect(reported.running,).toEqual([],);
            expect(reported.unreadableAnswers,).toHaveLength(2,);
            expect(reported.unreadableAnswers[0],).toContain('a process-tool answer the feed holds only a cut copy of',);
            expect(reported.unreadableAnswers[1],).toContain('a line of the feed that is no record',);
          },
        },),
        it({
          name: 'keeps the status an answer that is whole reports, and the last one a later answer reports',
          fn: async () => {
            /**
             A feed whose tool reported one process running and then reported it exited.
             */
            const shown = [
              aToolAnswer({ action: 'start', process: aProcess({},), },),
              aToolAnswer({ action: 'status', process: aProcess({ status: 'exited', },), },),
              '',
            ].join('\n',);
            expect(workTheToolReportedIn({ feedShown: shown, },),).toEqual({
              running: [],
              unreadableAnswers: [],
            },);
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
          name: 'names a process working in the run one scratch folder, which is the work a run own end step must not empty under',
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
            /**
             The scratch folder the launcher gave the run.
             */
            const scratch = '/home/a/temp/agent/copy-route-scratch';
            // A reproduction working in the scratch folder, in a scope of its own, which is where the briefs tell an agent to run
            // a command whose result a later turn reads.
            await writeProcess({
              folder: processes,
              pid: 103,
              cgroup: aScopeOfItsOwn(),
              worksIn: scratch,
              command: 'bash',
            },);
            expect(await workOutlivingTheTurnIn({
              runFolder: folder.path,
              workingFolder: aWorktree(),
              scratchFolder: scratch,
              processFolder: processes,
              run: async function answered() {
                return {
                  status: 1,
                  stdout: 'inactive\n',
                  stderr: '',
                };
              },
            },),).toEqual(['103 bash',],);
            // The same process with no scratch folder named is no work of the run's, which is the state the run's end was in before
            // the folder counted.
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
            },),).toEqual([],);
          },
        },),
        it({
          name: 'counts a process a cut answer names but no longer numbers as work that may still run, and says an answer it cannot read at all may have started some',
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
            /**
             A feed holding one cut answer that names a process and numbers neither it nor its status, and one answer nothing can be
             read from.
             */
            const feed = [
              JSON.stringify({
                kind: 'tool_end',
                tool: 'process',
                isError: false,
                result: '{"details":{"action":"start","process":{"id":"proc_7adb","name":"native-build-2","command":"bui',
              },),
              JSON.stringify({
                kind: 'tool_end',
                tool: 'process',
                isError: false,
                result: '{"content":[{"type":"text","text\\',
              },),
              '',
            ].join('\n',);
            await writeFile(join(
              folder.path,
              'feed.jsonl',
            ), feed,);
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
            },),).toEqual([
              'proc_7adb (native-build-2)',
              '1 process-tool answer the feed holds only a cut copy of, so a process one of them started may still run',
            ],);
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
