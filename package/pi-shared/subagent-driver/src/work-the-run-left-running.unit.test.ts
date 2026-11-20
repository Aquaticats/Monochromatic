/**
 The work a run left running: what its own `process` tool reported, what the machine's process table shows of the work descended
 from the run in its folders, and the message that sends the agent back to it. The tests that put a real process in the run's folder
 start that process themselves and end it themselves, and no test here asks a service manager for anything: the manager's own side
 is a stand-in `run`.

 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  chmod,
  mkdir,
  mkdtemp,
  readFile,
  readlink,
  rm,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { setTimeout as wait, } from 'node:timers/promises';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  type CommandOutput,
  FEED_PREVIEW_CHARACTERS,
  NO_PARENT_RECORD,
  preview,
  type PiRecord,
  type RunCommand,
  cgroupOf,
  descendsFrom,
  parentOfProcess,
  startRecordOfPiRun,
  unitNamedBy,
  wakeMessageOf,
  workOutlivingTheTurnIn,
  workTheToolReportedIn,
  MASKING_NOTHING,
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
  const record = startRecordOfPiRun({
    feedFile,
    mask: MASKING_NOTHING,
  },);
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
 The folder this machine keeps one entry per process in. A test reads the records of the processes it starts from here, and never
 lists the machine's processes from it.
 */
const REAL_PROCESS_FOLDER = '/proc';

/**
 Where a parent's process number sits in the fields that follow the program's name on a `/proc/<pid>/stat` line.
 */
const PARENT_AFTER_NAME = 1;

/**
 The first process an account's own processes descend from, where a chain of parents ends.
 */
const FIRST_PROCESS = 1;

/**
 How long a process these tests start lasts, which is far longer than a test takes: the process is there when the driver reads the
 process folder, and the test ends it by the number it was given.
 */
const REAL_PROCESS_SECONDS = 60;

/**
 Milliseconds between two looks while a test waits for a process to be handed over, or to be gone.
 */
const LOOK_MILLISECONDS = 50;

/**
 How long a wait for a process to be handed over, or to be gone, may last: a process that never moves fails the test rather than
 holding the suite open.
 */
const HANDOVER_WAIT_MILLISECONDS = 20_000;

/**
 The records the kernel keeps of one process, which a test copies from the real process folder into its fixture.
 */
const RECORDS_OF_A_PROCESS: readonly string[] = [
  'stat',
  'cgroup',
  'comm',
];

/**
 Reads the process number of the process that started one real process, from this machine's own record of it.

 The test builds its fixture from what the kernel wrote rather than with the reader under test, so a fault in that reader cannot
 hide itself in a fixture built by it.

 @param pid - the real process

 @returns the process number of the process that started it

 @example
 ```ts
 const parent = await realParentOf({ pid: 486667, },);
 ```
 */
async function realParentOf({ pid, }: { readonly pid: number },): Promise<number> {
  /**
   The kernel's own record of the process.
   */
  const stat = await readFile(
    join(
      REAL_PROCESS_FOLDER,
      String(pid,),
      'stat',
    ),
    'utf8',
  );
  /**
   The fields that follow the program's name, of which the second is the parent.
   */
  const afterName = stat.slice(stat.lastIndexOf(')',) + 1,)
    .trim();
  return Number(afterName.split(' ',)[PARENT_AFTER_NAME],);
}

/**
 A real process a test starts and ends itself: the `sleep` program, ended by the number the test was given when it started it.

 @param folder - the folder it works in

 @returns its process number, and what ends it

 @example
 ```ts
 await using other = await startARealProcess({ folder: '/home/user/temp/agent/x/worktree', },);
 ```
 */
async function startARealProcess({ folder, }: { readonly folder: string },): Promise<{
  readonly pid: number;
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  /**
   The process, started from this test's own conversation.
   */
  const started = spawn(
    'sleep',
    [String(REAL_PROCESS_SECONDS,),],
    {
      cwd: folder,
      stdio: 'ignore',
    },
  );
  /**
   The number the operating system gave it.
   */
  const { pid, } = started;
  if (pid === undefined) {
    throw new Error('the operating system started no process, so this test has none to read',);
  }
  return {
    pid,
    async [Symbol.asyncDispose]() {
      started.kill('SIGKILL',);
      if ((started.exitCode === null) && (started.signalCode === null)) {
        await once(
          started,
          'exit',
        );
      }
    },
  };
}

/**
 A process a test started that has left the test's own process tree, with what ends it.
 */
type LeftTheRun = {
  /**
   The process number the kernel gave the background job when the test's own shell started it.
   */
  readonly pid: number;

  /**
   The folder it works in, as the kernel reports it.
   */
  readonly folder: string;

  /**
   Ends it by that number, which the test's shell got for the job it started, and waits for it to be gone.
   */
  readonly stop: () => Promise<void>;

  /**
   The same, so a test that fails partway still ends what it started.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Waits, bounded, until the shell that started a process has gone and something else has taken that process in.

 @param pid - the process's number

 @param from - the process number the shell that started it had

 @returns nothing

 @throws when the process is still that shell's child when the bound is spent

 @example
 ```ts
 await waitToLeaveTheRun({ pid: 486667, from: 486601, },);
 ```
 */
async function waitToLeaveTheRun({
  pid,
  from,
}: {
  readonly pid: number;
  readonly from: number;
},): Promise<void> {
  /**
   When this wait must be over.
   */
  const deadline = Date.now() + HANDOVER_WAIT_MILLISECONDS;
  for (;;) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- a look, then a pause, then the next look; see doc/lint/sequential-awaits.md
    if ((await realParentOf({ pid, },)) !== from) {
      return;
    }
    if (Date.now() >= deadline) {
      throw new Error(`${String(pid,)} is still the child of ${String(from,)}`,);
    }
    // oxlint-disable-next-line eslint/no-await-in-loop -- the same wait, one look at a time; see doc/lint/sequential-awaits.md
    await wait(LOOK_MILLISECONDS,);
  }
}

/**
 Reads what the process folder holds of a process's control group, or nothing when it holds no such process.

 @param pid - the process number

 @returns the control group's text, or empty text when the process is gone

 @example
 ```ts
 const cgroup = await cgroupShownByMachine({ pid: 486667, },);
 ```
 */
async function cgroupShownByMachine({ pid, }: { readonly pid: number },): Promise<string> {
  return await cgroupOf({
    processFolder: REAL_PROCESS_FOLDER,
    entry: String(pid,),
  },);
}

/**
 Starts a real process that a shell of the test's puts in the background and leaves, so it is handed to whatever took it in when
 that shell exited and no chain of parents reaches the test.

 This is the shape an agent leaves behind with a background job in a `bash` call, a `setsid`, or a double fork, and it needs no
 service manager: the work is a `sleep` this test ends itself.

 @param folder - the folder the work runs in, where the shell writes the number it was given

 @returns the work's process number and folder, and what ends it

 @example
 ```ts
 await using left = await startAProcessThatLeavesTheRun({ folder: '/x/worktree', },);
 ```
 */
async function startAProcessThatLeavesTheRun({ folder, }: { readonly folder: string },): Promise<LeftTheRun> {
  /**
   The file the shell writes the background job's process number into, which the kernel gave it when it started that job.
   */
  const jobFile = join(folder, 'left-running-job-pid.txt',);
  /**
   The shell that starts the work and goes.
   */
  const caller = spawn(
    'sh',
    [
      '-c',
      `sleep ${String(REAL_PROCESS_SECONDS,)} > /dev/null 2>&1 & echo $! > ${jobFile}`,
    ],
    {
      cwd: folder,
      stdio: 'ignore',
    },
  );
  /**
   The number the kernel gave that shell.
   */
  const { pid: callerPid, } = caller;
  if (callerPid === undefined) {
    throw new Error('the operating system started no shell, so no work was started',);
  }
  await once(
    caller,
    'exit',
  );
  /**
   The work's process number, which the shell got for the job it started.
   */
  const pid = Number((await readFile(jobFile, 'utf8',)).trim(),);
  if ((!Number.isSafeInteger(pid,)) || (pid <= 0)) {
    throw new Error(`the shell wrote no process number into ${jobFile}`,);
  }
  /**
   Ends the work, by the number that shell got for it, and only that one.
   */
  async function stop(): Promise<void> {
    if ((await cgroupShownByMachine({ pid, },)) !== '') {
      process.kill(pid, 'SIGKILL',);
    }
    /**
     When this wait must be over.
     */
    const deadline = Date.now() + HANDOVER_WAIT_MILLISECONDS;
    for (;;) {
      // oxlint-disable-next-line eslint/no-await-in-loop -- a look, then a pause, then the next look; see doc/lint/sequential-awaits.md
      if ((await cgroupShownByMachine({ pid, },)) === '') {
        return;
      }
      if (Date.now() >= deadline) {
        throw new Error(`${String(pid,)} is still there after the test ended it`,);
      }
      // oxlint-disable-next-line eslint/no-await-in-loop -- the same wait, one look at a time; see doc/lint/sequential-awaits.md
      await wait(LOOK_MILLISECONDS,);
    }
  }
  // The work is the test's own however this wait ends, so a shell that never went still leaves no `sleep` behind: the number the
  // shell got is held here, and `await using` can register nothing while the helper has not returned.
  try {
    await waitToLeaveTheRun({
      pid,
      from: callerPid,
    },);
  }
  catch (error) {
    await stop();
    throw error;
  }
  return {
    pid,
    folder: await readlink(join(
      REAL_PROCESS_FOLDER,
      String(pid,),
      'cwd',
    ),),
    stop,
    [Symbol.asyncDispose]: stop,
  };
}

/**
 A shell started the way the `process` tool starts one, and the child it runs: both real, and both ended by the test that started
 them.

 The tool runs a command as a child of a shell it started, and that child keeps the shell's number and the shell as its parent; a
 real `systemd-run --user --scope` keeps both as well, as the first review's probe A measured (the payload kept the number
 `systemd-run` was given and the parent that started it, while its control group became the scope's), but it needs this machine's
 user manager, which no test here asks for. So this fixture keeps the tree and leaves the scope's control group out of it.
 */
type AShellAndItsChild = {
  /**
   The shell's process number, as the kernel gave it to the test.
   */
  readonly shellPid: number;

  /**
   The child's process number, which the kernel gave the shell when it started it.
   */
  readonly childPid: number;

  /**
   The program the kernel names for the child.
   */
  readonly childProgram: string;

  /**
   Ends both, through the process group the shell leads, and waits for them to be gone.
   */
  readonly stop: () => Promise<void>;

  /**
   The same, so a test that fails partway still ends what it started.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Waits, bounded, until a process number is gone from the process folder.

 @param pid - the process number

 @returns nothing

 @throws when the process is still there when the bound is spent

 @example
 ```ts
 await waitUntilGone({ pid: 486667, },);
 ```
 */
async function waitUntilGone({ pid, }: { readonly pid: number },): Promise<void> {
  /**
   When this wait must be over.
   */
  const deadline = Date.now() + HANDOVER_WAIT_MILLISECONDS;
  for (;;) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- a look, then a pause, then the next look; see doc/lint/sequential-awaits.md
    if ((await cgroupShownByMachine({ pid, },)) === '') {
      return;
    }
    if (Date.now() >= deadline) {
      throw new Error(`${String(pid,)} is still there after the test ended it`,);
    }
    // oxlint-disable-next-line eslint/no-await-in-loop -- the same wait, one look at a time; see doc/lint/sequential-awaits.md
    await wait(LOOK_MILLISECONDS,);
  }
}

/**
 Waits, bounded, until a process has a child, which it reads from the kernel's own record of that process's children.

 The record is the one the kernel keeps per thread (`/proc/<pid>/task/<pid>/children`), and a shell forks the child this reads from
 its own thread.

 @param pid - the process whose child is waited for

 @returns the child's process number

 @throws when the process has no child when the bound is spent

 @example
 ```ts
 const child = await waitForAChildOf({ pid: 486667, },);
 ```
 */
async function waitForAChildOf({ pid, }: { readonly pid: number },): Promise<number> {
  /**
   The file the kernel writes that process's children into.
   */
  const childrenFile = join(
    REAL_PROCESS_FOLDER,
    String(pid,),
    'task',
    String(pid,),
    'children',
  );
  /**
   When this wait must be over.
   */
  const deadline = Date.now() + HANDOVER_WAIT_MILLISECONDS;
  for (;;) {
    /**
     The process numbers the kernel lists for it.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- a look, then a pause, then the next look; see doc/lint/sequential-awaits.md
    const shown = await readFile(childrenFile, 'utf8',);
    /**
     The first of them.
     */
    const [first = '',] = shown.trim()
      .split(' ',);
    if (first !== '') {
      return Number(first,);
    }
    if (Date.now() >= deadline) {
      throw new Error(`${String(pid,)} started no child`,);
    }
    // oxlint-disable-next-line eslint/no-await-in-loop -- the same wait, one look at a time; see doc/lint/sequential-awaits.md
    await wait(LOOK_MILLISECONDS,);
  }
}

/**
 Starts a shell the way the `process` tool starts one, which runs a child that keeps working in the folder given.

 @param folder - the folder the shell and its child work in

 @returns the shell's number, the child's number and program, and what ends both

 @example
 ```ts
 await using work = await startAShellWhoseChildWorks({ folder: '/x/worktree', },);
 ```
 */
async function startAShellWhoseChildWorks({ folder, }: { readonly folder: string },): Promise<AShellAndItsChild> {
  // The shell is started as a process group of its own, which is what the tool does with the shell it runs a command in, so the
  // test can end the shell and its child together.
  const shell = spawn(
    'sh',
    [
      '-lc',
      `sleep ${String(REAL_PROCESS_SECONDS,)}; :`,
    ],
    {
      cwd: folder,
      detached: true,
      stdio: 'ignore',
    },
  );
  /**
   The number the kernel gave that shell.
   */
  const { pid: startedPid, } = shell;
  if (startedPid === undefined) {
    throw new Error('the operating system started no shell, so no work was started',);
  }
  /**
   The shell's number, which the check above leaves a number for everything below it, a closure included.
   */
  const shellPid: number = startedPid;
  /**
   The child the shell runs, which keeps the folder and the parent it was given.
   */
  const childPid = await waitForAChildOf({ pid: shellPid, },);
  /**
   Ends both, through the process group the shell leads: the group is the number the kernel gave the shell, so killing it ends
   the child with it, and nothing outside the group is signalled.
   */
  async function stop(): Promise<void> {
    if ((await cgroupShownByMachine({ pid: childPid, },)) !== '') {
      process.kill(-shellPid, 'SIGKILL',);
    }
    await waitUntilGone({ pid: childPid, },);
    await waitUntilGone({ pid: shellPid, },);
  }
  return {
    shellPid,
    childPid,
    childProgram: (await readFile(
      join(
        REAL_PROCESS_FOLDER,
        String(childPid,),
        'comm',
      ),
      'utf8',
    )).trim(),
    stop,
    [Symbol.asyncDispose]: stop,
  };
}

/**
 Writes what the kernel records for one real process into a process folder a test owns.

 The process is real and each record is the one the kernel wrote for it; only the folder they are read from is the test's, which is
 the seam the driver has for every read of a process folder. A working folder is the fixture's, so a process working where the run
 does not is written the same way.

 @param folder - the process folder a test owns

 @param pid - the real process

 @param worksIn - the folder it works in, when the test's listing is to hold it

 @returns nothing

 @example
 ```ts
 await writeRecordsOfRealProcess({ folder: '/x/proc', pid: 486667, worksIn: '/x/worktree', },);
 ```
 */
async function writeRecordsOfRealProcess({
  folder,
  pid,
  worksIn,
}: {
  readonly folder: string;
  readonly pid: number;
  readonly worksIn?: string;
},): Promise<void> {
  /**
   The entry one process has in that folder.
   */
  const entry = join(
    folder,
    String(pid,),
  );
  await mkdir(entry, { recursive: true, },);
  await Promise.all(RECORDS_OF_A_PROCESS.map(async function copied(record,) {
    await writeFile(
      join(entry, record,),
      await readFile(
        join(
          REAL_PROCESS_FOLDER,
          String(pid,),
          record,
        ),
        'utf8',
      ),
    );
  },),);
  if (worksIn !== undefined) {
    await symlink(
      worksIn,
      join(entry, 'cwd',),
      'dir',
    );
  }
}

/**
 Reads the whole chain of processes that started one real process, from this machine's own records of them.

 @param pid - the real process

 @returns every process that started it, the one nearest it first, and never the first process itself

 @example
 ```ts
 const ancestors = await ancestorsOf({ pid: 486667, },);
 ```
 */
async function ancestorsOf({ pid, }: { readonly pid: number },): Promise<readonly number[]> {
  /**
   The chain read so far, whose last entry is the process whose parent is read next.
   */
  const chain = [pid,];
  for (;;) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- each link names the process whose parent is read next
    const parent = await realParentOf({ pid: chain.at(-1,) ?? pid, },);
    if (parent <= FIRST_PROCESS) {
      return chain.slice(1,);
    }
    chain.push(parent,);
  }
}

/**
 Writes the record of every process that started the one given, so a fixture holds the whole chain a reader follows.

 An ancestor carries no working folder, so the listing of a fixture never counts one as a process working somewhere: only the chain
 of parents a reader walks is complete.

 @param folder - the process folder a test owns

 @param pid - the real process whose ancestors are written

 @returns nothing

 @example
 ```ts
 await writeParentChainOf({ folder: '/x/proc', pid: 486667, },);
 ```
 */
async function writeParentChainOf({
  folder,
  pid,
}: {
  readonly folder: string;
  readonly pid: number;
},): Promise<void> {
  await Promise.all((await ancestorsOf({ pid, },)).map(async function wrote(ancestor,) {
    /**
     The entry that ancestor has in the fixture.
     */
    const entry = join(
      folder,
      String(ancestor,),
    );
    await mkdir(entry, { recursive: true, },);
    await writeFile(
      join(entry, 'stat',),
      await readFile(
        join(
          REAL_PROCESS_FOLDER,
          String(ancestor,),
          'stat',
        ),
        'utf8',
      ),
    );
  },),);
}

/**
 Writes one process into a process folder a test owns.

 @param folder - the process folder

 @param pid - the process's number

 @param cgroup - the control group it is in

 @param worksIn - the folder it works in

 @param command - the program the kernel names for it

 @param parentPid - the process that started it, which a chain of parents follows

 @returns nothing

 @example
 ```ts
 await writeProcess({ folder: '/x/proc', pid: 12, cgroup: '0::/a.scope', worksIn: '/repo', command: 'bash', parentPid: 11, },);
 ```
 */
async function writeProcess({
  folder,
  pid,
  cgroup,
  worksIn,
  command,
  parentPid = FIRST_PROCESS,
}: {
  readonly folder: string;
  readonly pid: number;
  readonly cgroup: string;
  readonly worksIn: string;
  readonly command: string;
  readonly parentPid?: number;
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
  // The record of who started it, as the kernel writes its first four fields: the number, the program's name in parentheses, the
  // state, and the process that started it. A real one is `/proc/<pid>/stat`.
  await writeFile(
    join(
      folder,
      String(pid,),
      'stat',
    ),
    `${String(pid,)} (${command}) S ${String(parentPid,)}\n`,
  );
}

/**
 Writes the driver's own entry into a process folder a test owns, so the run's own unit is known.

 @param folder - the process folder

 @param cgroup - the control group of the run's own unit

 @returns nothing

 @example
 ```ts
 await writeSelf({ folder: '/x/proc', cgroup: theRunsUnit(), },);
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

/**
 Builds an error shaped like the one Node raises for a failed system call.

 @param code - the name the operating system gave the failure, such as `ESRCH`

 @param syscall - the call the message names

 @returns the error, carrying its code as a property of its own, as Node's does

 @example
 ```ts
 const refused = systemCallError({ code: 'ESRCH', syscall: 'read', },);
 ```
 */
function systemCallError({
  code,
  syscall,
}: {
  readonly code: string;
  readonly syscall: string;
},): NodeJS.ErrnoException {
  const error: NodeJS.ErrnoException = new Error(`${code}: no such process, ${syscall}`,);
  error.code = code;
  error.syscall = syscall;
  return error;
}

/**
 Reads of a process folder a test hands over, where one read answers an error instead of the record the folder holds.

 The folder is this machine's own reading apart from that one path, so a case says which record the kernel answers an error for
 and every other record is read as it really is.

 @param read - which read answers the error, a record's text or the folder a `cwd` link names

 @param suffix - the end of the path that read answers the error for

 @param thrown - what that read throws

 @returns the reads, for the folder a case wrote

 @example
 ```ts
 const reads = readsRefusing({ read: 'text', suffix: '/101/cgroup', thrown: refused, },);
 ```
 */
function readsRefusing({
  read,
  suffix,
  thrown,
}: {
  readonly read: 'text' | 'link';
  readonly suffix: string;
  readonly thrown: NodeJS.ErrnoException;
},): {
  readonly text: (path: string) => Promise<string>;
  readonly link: (path: string) => Promise<string>;
} {
  return {
    text: async function textOf(path: string,) {
      if ((read === 'text') && path.endsWith(suffix,)) {
        throw thrown;
      }
      return await readFile(path, 'utf8',);
    },
    link: async function linkAt(path: string,) {
      if ((read === 'link') && path.endsWith(suffix,)) {
        throw thrown;
      }
      return await readlink(path,);
    },
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
      name: cgroupOf.name,
      children: [
        it({
          name: 'answers no control group for a process that ended while its record was read, and hands back a read that failed for another reason',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The process folder a test writes.
             */
            const processes = join(folder.path, 'proc',);
            await writeProcess({
              folder: processes,
              pid: 101,
              cgroup: aScopeOfItsOwn(),
              worksIn: aWorktree(),
              command: 'bash',
            },);
            // The record of a process the kernel has released: the folder is still listed while the read answers `ESRCH`, so the
            // process is gone rather than the read being one that failed.
            expect(await cgroupOf({
              processFolder: processes,
              entry: '101',
              reads: readsRefusing({
                read: 'text',
                suffix: '/101/cgroup',
                thrown: systemCallError({ code: 'ESRCH', syscall: 'read', },),
              },),
            },),).toBe('',);
            /**
             A read that failed for a reason that names no ended process.
             */
            const refused = systemCallError({ code: 'EIO', syscall: 'read', },);
            /**
             What the read threw, when it threw.
             */
            let thrown: unknown;
            try {
              await cgroupOf({
                processFolder: processes,
                entry: '101',
                reads: readsRefusing({
                  read: 'text',
                  suffix: '/101/cgroup',
                  thrown: refused,
                },),
              },);
            }
            catch (error) {
              thrown = error;
            }
            expect(thrown,).toBe(refused,);
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
            // The run's own Pi, which the run's unit holds and which started the work below it.
            await writeProcess({
              folder: processes,
              pid: 100,
              cgroup: theRunsUnit(),
              worksIn: aWorktree(),
              command: 'node',
            },);
            // The build the agent started in a scope of its own, which left the run's unit and kept the run's folder and the
            // parent that started it.
            await writeProcess({
              folder: processes,
              pid: 101,
              cgroup: aScopeOfItsOwn(),
              worksIn: aWorktree(),
              command: 'bash',
              parentPid: 100,
            },);
            await writeProcess({
              folder: processes,
              pid: 102,
              cgroup: aScopeOfItsOwn(),
              worksIn: '/elsewhere',
              command: 'sleep',
              parentPid: 100,
            },);
            expect(await workOutlivingTheTurnIn({
              runFolder: folder.path,
              workingFolder: aWorktree(),
              processFolder: processes,
              ownProcess: 100,
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
              ownProcess: 100,
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
              ownProcess: 100,
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
              ownProcess: 100,
              run: holdingNothing,
            },),).toEqual([],);
          },
        },),
        it({
          name: 'keeps a real process descended from the run that works in the run folder in a control group other than the run own unit',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The folder the run works in.
             */
            const workingFolder = join(folder.path, 'worktree',);
            await mkdir(workingFolder, { recursive: true, },);
            /**
             The process folder a test writes, whose own entry is the run's unit.
             */
            const processes = join(folder.path, 'proc',);
            await writeSelf({
              folder: processes,
              cgroup: theRunsUnit(),
            },);
            // The work the run started: a real process this test starts, which descends from this test's own process. The test's
            // own number plays the run's own, as the driver's number does for a run. The run's own unit is a stand-in name, because
            // a real unit is the manager's to make and a unit test asks no manager for one, and the control group is the one the
            // kernel reports for that process.
            await using build = await startARealProcess({ folder: workingFolder, },);
            await writeRecordsOfRealProcess({
              folder: processes,
              pid: build.pid,
              worksIn: workingFolder,
            },);
            /**
             What the fixture's own record says the kernel names that process, which the driver's line carries: the name of a process
             that has not run its program yet is the one of the process that started it, and the line is about descent either way.
             */
            const named = (await readFile(
              join(
                processes,
                String(build.pid,),
                'comm',
              ),
              'utf8',
            )).trim();
            expect(await workOutlivingTheTurnIn({
              runFolder: folder.path,
              workingFolder,
              processFolder: processes,
              ownProcess: process.pid,
              run: async function answered() {
                return {
                  status: 1,
                  stdout: 'inactive\n',
                  stderr: '',
                };
              },
            },),).toEqual([`${String(build.pid,)} ${named}`,],);
          },
        },),
        it({
          name: 'never counts a real process another session started that works in the run folder, which held two runs of 2026-11-19 on their wall clocks',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The folder the run works in.
             */
            const workingFolder = join(folder.path, 'worktree',);
            await mkdir(workingFolder, { recursive: true, },);
            /**
             The process folder a test writes, whose own entry is the run's unit.
             */
            const processes = join(folder.path, 'proc',);
            await writeSelf({
              folder: processes,
              cgroup: theRunsUnit(),
            },);
            // The run's own process: a real process of this test's that stands beside the other one rather than above it, as the
            // driver of a run stands beside another session's.
            await using theRun = await startARealProcess({ folder: folder.path, },);
            // The process another session started: a real process of this test's, working in the run's own folder, which descends
            // from this test and never from the run.
            await using anotherSession = await startARealProcess({ folder: workingFolder, },);
            await writeRecordsOfRealProcess({
              folder: processes,
              pid: anotherSession.pid,
              worksIn: workingFolder,
            },);
            // The whole chain above it, so the reader that follows parents reads a chain that ends at the first process rather than
            // one the fixture left short.
            await writeParentChainOf({
              folder: processes,
              pid: anotherSession.pid,
            },);
            expect(await workOutlivingTheTurnIn({
              runFolder: folder.path,
              workingFolder,
              processFolder: processes,
              ownProcess: theRun.pid,
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
          name: 'never names a real process the run left behind that has left its own process tree, though it works in the run folder',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The folder the run works in.
             */
            const workingFolder = join(folder.path, 'worktree',);
            await mkdir(workingFolder, { recursive: true, },);
            /**
             The process folder a test writes, whose own entry is the run's unit.
             */
            const processes = join(folder.path, 'proc',);
            await writeSelf({
              folder: processes,
              cgroup: theRunsUnit(),
            },);
            // The work: a real `sleep` a shell of this test's puts in the background and leaves, as a bash call does with a
            // background job, a `setsid` or a double fork, so whatever took it in when that shell exited is its parent.
            await using left = await startAProcessThatLeavesTheRun({ folder: workingFolder, },);
            // The premises: it is alive, it works in the run's own folder, and no chain of parents reaches this test. The run's own
            // unit is a stand-in name, so the control group these records hold is the one the kernel wrote for a process outside it.
            expect(await cgroupShownByMachine({ pid: left.pid, },),).not.toBe('',);
            expect(left.folder,).toBe(workingFolder,);
            expect(await descendsFrom({
              processFolder: REAL_PROCESS_FOLDER,
              pid: left.pid,
              rootPid: process.pid,
            },),).toBe(false,);
            // The rule before descent counted it, because its folder and its control group matched: that is the shape this test
            // pins, and the driver names nothing for it now.
            await writeRecordsOfRealProcess({
              folder: processes,
              pid: left.pid,
              worksIn: workingFolder,
            },);
            /**
             What a manager would be asked, which is nothing: the feed is empty, so no record names a unit, and the process table's
             answer never asks one.
             */
            const askedOfAManager: string[][] = [];
            expect(await workOutlivingTheTurnIn({
              runFolder: folder.path,
              workingFolder,
              processFolder: processes,
              ownProcess: process.pid,
              run: aManagerAnswering({
                status: 3,
                stdout: 'inactive\n',
                asked: askedOfAManager,
              },),
            },),).toEqual([],);
            expect(askedOfAManager,).toEqual([],);
            // The test ends its own `sleep` by the number its shell got, and nothing else.
            await left.stop();
            expect(await cgroupShownByMachine({ pid: left.pid, },),).toBe('',);
          },
        },),
        it({
          name: 'names a real child a shell started the way the process tool starts one, through the run own process tree',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The folder the run works in.
             */
            const workingFolder = join(folder.path, 'worktree',);
            await mkdir(workingFolder, { recursive: true, },);
            /**
             The process folder a test writes, whose own entry is the run's unit.
             */
            const processes = join(folder.path, 'proc',);
            await writeSelf({
              folder: processes,
              cgroup: theRunsUnit(),
            },);
            // The route the README recommends: work started as a child of a shell keeps the parent it was given, so the run's own
            // process tree holds it. A `systemd-run --user --scope` keeps the same tree (the first review's probe A measured the
            // payload keeping the number and the parent while its control group becomes the scope's), and a plain child is what a
            // test can start without this machine's user manager.
            await using work = await startAShellWhoseChildWorks({ folder: workingFolder, },);
            // The shell is the test's own child, as the tool's shell is the Pi's, so the test's number plays the run's own.
            expect(await realParentOf({ pid: work.shellPid, },),).toBe(process.pid,);
            expect(await realParentOf({ pid: work.childPid, },),).toBe(work.shellPid,);
            expect(await cgroupShownByMachine({ pid: work.childPid, },),).not.toBe('',);
            // Both are written into the fixture: the shell so the chain of parents has its link, and the child as the work itself,
            // which works in the run's folder.
            await writeRecordsOfRealProcess({
              folder: processes,
              pid: work.shellPid,
            },);
            await writeRecordsOfRealProcess({
              folder: processes,
              pid: work.childPid,
              worksIn: workingFolder,
            },);
            /**
             What a manager would be asked, which is nothing: the feed is empty, so no record names a unit.
             */
            const askedOfAManager: string[][] = [];
            expect(await workOutlivingTheTurnIn({
              runFolder: folder.path,
              workingFolder,
              processFolder: processes,
              ownProcess: process.pid,
              run: aManagerAnswering({
                status: 3,
                stdout: 'inactive\n',
                asked: askedOfAManager,
              },),
            },),).toEqual([`${String(work.childPid,)} ${work.childProgram}`,],);
            expect(askedOfAManager,).toEqual([],);
            // The test ends the shell and the child it started, and nothing else.
            await work.stop();
            expect(await cgroupShownByMachine({ pid: work.childPid, },),).toBe('',);
          },
        },),
        it({
          name: 'treats ESRCH from any record of a process as the record being gone, keeping its program unknown but naming no work for its control group or its parent',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The folder the run works in.
             */
            const workingFolder = join(folder.path, 'worktree',);
            await mkdir(workingFolder, { recursive: true, },);
            /**
             The process folder a test writes, whose own entry is the run's unit.
             */
            const processes = join(folder.path, 'proc',);
            await writeSelf({
              folder: processes,
              cgroup: theRunsUnit(),
            },);
            // The work the run started: a process in a scope of its own that works in the run's folder and descends from the run.
            // The run's own Pi is written beside it, so the chain of parents reaches the process that plays the run's own.
            await writeProcess({
              folder: processes,
              pid: 100,
              cgroup: theRunsUnit(),
              worksIn: aWorktree(),
              command: 'node',
              parentPid: process.pid,
            },);
            await writeProcess({
              folder: processes,
              pid: 101,
              cgroup: aScopeOfItsOwn(),
              worksIn: workingFolder,
              command: 'bash',
              parentPid: 100,
            },);
            /**
             What the manager answers, which no case here asks it about a unit.
             */
            const run = aManagerAnswering({
              status: 3,
              stdout: 'inactive\n',
            },);
            /**
             Each record the driver reads of that process, with what the run is left waiting for when the kernel answers `ESRCH`
             for it: the program is unknown while the entry is there, and no control group or parent means no work of the run.
             */
            const records: readonly (readonly [string, readonly string[]])[] = [
              ['/101/comm', ['101 ?',],],
              ['/101/cgroup', [],],
              ['/101/stat', [],],
            ];
            for (const [record, leftRunning,] of records) {
              // A process that ended while one of its records was read is gone exactly as one whose record is already missing is:
              // the scan answers, rather than ending the run on a read that failed.
              // oxlint-disable-next-line eslint/no-await-in-loop -- one record after another, each a whole scan of the same folder; see doc/lint/sequential-awaits.md
              expect(await workOutlivingTheTurnIn({
                runFolder: folder.path,
                workingFolder,
                processFolder: processes,
                ownProcess: process.pid,
                run,
                reads: readsRefusing({
                  read: 'text',
                  suffix: record,
                  thrown: systemCallError({ code: 'ESRCH', syscall: 'read', },),
                },),
              },),).toEqual(leftRunning,);
            }
          },
        },),
      ],
    },),
    describe({
      name: descendsFrom.name,
      children: [
        it({
          name: 'reaches the run own process along a chain of parents, and answers nothing for a root the chain never names',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The process folder a test writes.
             */
            const processes = join(folder.path, 'proc',);
            await writeProcess({
              folder: processes,
              pid: 300,
              cgroup: aScopeOfItsOwn(),
              worksIn: aWorktree(),
              command: 'bash',
              parentPid: 200,
            },);
            await writeProcess({
              folder: processes,
              pid: 200,
              cgroup: aScopeOfItsOwn(),
              worksIn: aWorktree(),
              command: 'systemd-run',
              parentPid: 100,
            },);
            await writeProcess({
              folder: processes,
              pid: 100,
              cgroup: theRunsUnit(),
              worksIn: aWorktree(),
              command: 'node',
              parentPid: 99,
            },);
            expect(await descendsFrom({
              processFolder: processes,
              pid: 300,
              rootPid: 100,
            },),).toBe(true,);
            expect(await descendsFrom({
              processFolder: processes,
              pid: 300,
              rootPid: 99,
            },),).toBe(true,);
            expect(await descendsFrom({
              processFolder: processes,
              pid: 300,
              rootPid: 777,
            },),).toBe(false,);
            // A process is no descendant of itself, which is what keeps the run's own process out of its own work.
            expect(await descendsFrom({
              processFolder: processes,
              pid: 100,
              rootPid: 100,
            },),).toBe(false,);
          },
        },),
        it({
          name: 'ends at a record it cannot read rather than naming a process the run never started',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The process folder a test writes.
             */
            const processes = join(folder.path, 'proc',);
            await writeProcess({
              folder: processes,
              pid: 300,
              cgroup: aScopeOfItsOwn(),
              worksIn: aWorktree(),
              command: 'bash',
              parentPid: 200,
            },);
            // The record of the process that started it is not there: one that ended while the read was under way leaves the same
            // hole in a process folder, and no chain then says where the process came from.
            expect(await descendsFrom({
              processFolder: processes,
              pid: 300,
              rootPid: 100,
            },),).toBe(false,);
            expect(await descendsFrom({
              processFolder: processes,
              pid: 404,
              rootPid: 100,
            },),).toBe(false,);
            await writeProcess({
              folder: processes,
              pid: 200,
              cgroup: aScopeOfItsOwn(),
              worksIn: aWorktree(),
              command: 'systemd-run',
              parentPid: 100,
            },);
            // Another account's process, whose folder this one may not read: the folder a link is in decides whether the link may
            // be read, so an unsearchable one is what such a process looks like from here. It is searchable again when this test
            // ends, whether it ends well or not, so the folder it made can be removed.
            await chmod(join(
              processes,
              '200',
            ), 0o000,);
            await using madeReadableAgain = {
              async [Symbol.asyncDispose]() {
                await chmod(join(
                  processes,
                  '200',
                ), 0o700,);
              },
            };
            expect(await descendsFrom({
              processFolder: processes,
              pid: 300,
              rootPid: 100,
            },),).toBe(false,);
          },
        },),
        it({
          name: 'ends rather than following a record that names a process as its own ancestor, or two as each other',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The process folder a test writes.
             */
            const processes = join(folder.path, 'proc',);
            await writeProcess({
              folder: processes,
              pid: 300,
              cgroup: aScopeOfItsOwn(),
              worksIn: aWorktree(),
              command: 'bash',
              parentPid: 300,
            },);
            await writeProcess({
              folder: processes,
              pid: 301,
              cgroup: aScopeOfItsOwn(),
              worksIn: aWorktree(),
              command: 'bash',
              parentPid: 302,
            },);
            await writeProcess({
              folder: processes,
              pid: 302,
              cgroup: aScopeOfItsOwn(),
              worksIn: aWorktree(),
              command: 'bash',
              parentPid: 301,
            },);
            // A process that names itself as the one that started it, and is the run's own process besides: no descendant of
            // itself either way.
            await writeProcess({
              folder: processes,
              pid: 500,
              cgroup: aScopeOfItsOwn(),
              worksIn: aWorktree(),
              command: 'bash',
              parentPid: 500,
            },);
            expect(await descendsFrom({
              processFolder: processes,
              pid: 300,
              rootPid: 100,
            },),).toBe(false,);
            expect(await descendsFrom({
              processFolder: processes,
              pid: 301,
              rootPid: 100,
            },),).toBe(false,);
            expect(await descendsFrom({
              processFolder: processes,
              pid: 500,
              rootPid: 500,
            },),).toBe(false,);
            expect(await descendsFrom({
              processFolder: processes,
              pid: 500,
              rootPid: 100,
            },),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: parentOfProcess.name,
      children: [
        it({
          name: 'reads the parent from the fields after the program name, which may hold a space and a parenthesis of its own',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The process folder a test writes.
             */
            const processes = join(folder.path, 'proc',);
            await writeProcess({
              folder: processes,
              pid: 300,
              cgroup: aScopeOfItsOwn(),
              worksIn: aWorktree(),
              command: 'node',
              parentPid: 200,
            },);
            expect(await parentOfProcess({
              processFolder: processes,
              pid: 300,
            },),).toBe(200,);
            await writeFile(
              join(
                processes,
                '300',
                'stat',
              ),
              '300 (a name with a space and a ) in it) S 201\n',
            );
            expect(await parentOfProcess({
              processFolder: processes,
              pid: 300,
            },),).toBe(201,);
          },
        },),
        it({
          name: 'names no parent for a record that holds none, for the first process, and for a process the folder does not hold',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The process folder a test writes.
             */
            const processes = join(folder.path, 'proc',);
            await mkdir(join(processes, '300',), { recursive: true, },);
            await writeFile(join(
              processes,
              '300',
              'stat',
            ), 'not a record the kernel writes\n',);
            await mkdir(join(processes, '301',), { recursive: true, },);
            // The first process, which the kernel names no parent for.
            await writeFile(join(
              processes,
              '301',
              'stat',
            ), '301 (init) S 0\n',);
            expect(await parentOfProcess({
              processFolder: processes,
              pid: 300,
            },),).toBe(NO_PARENT_RECORD,);
            expect(await parentOfProcess({
              processFolder: processes,
              pid: 301,
            },),).toBe(NO_PARENT_RECORD,);
            expect(await parentOfProcess({
              processFolder: processes,
              pid: 404,
            },),).toBe(NO_PARENT_RECORD,);
          },
        },),
        it({
          name: 'names no parent for a record whose read answered ESRCH, and hands back a read that failed for another reason',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The process folder a test writes.
             */
            const processes = join(folder.path, 'proc',);
            await writeProcess({
              folder: processes,
              pid: 300,
              cgroup: aScopeOfItsOwn(),
              worksIn: aWorktree(),
              command: 'node',
              parentPid: 200,
            },);
            // The record of a process the kernel released between the folder being listed and the record being read.
            expect(await parentOfProcess({
              processFolder: processes,
              pid: 300,
              reads: readsRefusing({
                read: 'text',
                suffix: '/300/stat',
                thrown: systemCallError({ code: 'ESRCH', syscall: 'read', },),
              },),
            },),).toBe(NO_PARENT_RECORD,);
            /**
             A read that failed for a reason that names no ended process.
             */
            const refused = systemCallError({ code: 'EIO', syscall: 'read', },);
            /**
             What the read threw, when it threw.
             */
            let thrown: unknown;
            try {
              await parentOfProcess({
                processFolder: processes,
                pid: 300,
                reads: readsRefusing({
                  read: 'text',
                  suffix: '/300/stat',
                  thrown: refused,
                },),
              },);
            }
            catch (error) {
              thrown = error;
            }
            expect(thrown,).toBe(refused,);
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
