/**
 The work a run left running: what its own `process` tool reported, what the machine's process table shows of the work descended
 from the run in its folders, and the message that sends the agent back to it. The tests that put another session's process in the
 run's folder start real processes and end them themselves.

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
  runCommandWithChildProcess,
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
 Milliseconds between two looks while a test waits for this machine's user manager to answer, or for a process to be handed over.
 */
const LOOK_MILLISECONDS = 50;

/**
 How long a wait for this machine's user manager may last: a manager that never answers fails the test rather than holding the suite
 open.
 */
const MANAGER_WAIT_MILLISECONDS = 20_000;

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
 Work a test started as a scope or as a unit of its own, with what ends it.
 */
type StartedWork = {
  /**
   The unit this machine's user manager knows the work by.
   */
  readonly unit: string;

  /**
   The process number of the work itself.
   */
  readonly pid: number;

  /**
   The folder it works in, as the kernel reports it.
   */
  readonly folder: string;

  /**
   Asks the manager to stop the unit, which is the only way a test ends work: it never signals a process the kernel handed no
   number to.
   */
  readonly stop: () => Promise<void>;

  /**
   The same, so a test that fails partway still stops what it started.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Asks this machine's user manager one question, through the runner the driver asks it with.

 @param command - the program to run

 @param words - what to pass it

 @returns what it printed and the status it ended with

 @example
 ```ts
 const answer = await askTheManager({ command: 'systemctl', words: ['--user', 'is-active', 'a.service',], },);
 ```
 */
async function askTheManager({
  command,
  words,
}: {
  readonly command: string;
  readonly words: readonly string[];
},): Promise<CommandOutput> {
  return await runCommandWithChildProcess({
    command,
    words,
  },);
}

/**
 Says whether this machine's user manager holds a unit now.

 @param unit - the unit's name

 @returns whether it is active

 @example
 ```ts
 const holds = await theManagerHolds({ unit: 'pi-build-2.service', },);
 ```
 */
async function theManagerHolds({ unit, }: { readonly unit: string },): Promise<boolean> {
  /**
   What the manager answered about it.
   */
  const answer = await askTheManager({
    command: 'systemctl',
    words: [
      '--user',
      'is-active',
      unit,
    ],
  },);
  return answer.stdout.trim() === 'active';
}

/**
 Waits, bounded, until this machine's user manager holds a unit.

 @param unit - the unit's name

 @returns nothing

 @throws when the manager still holds no such unit when the bound is spent

 @example
 ```ts
 await waitUntilTheManagerHolds({ unit: 'pi-build-2.service', },);
 ```
 */
async function waitUntilTheManagerHolds({ unit, }: { readonly unit: string },): Promise<void> {
  /**
   When this wait must be over.
   */
  const deadline = Date.now() + MANAGER_WAIT_MILLISECONDS;
  for (;;) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- a look, then a pause, then the next look; see doc/lint/sequential-awaits.md
    if (await theManagerHolds({ unit, },)) {
      return;
    }
    if (Date.now() >= deadline) {
      throw new Error(`this machine's user manager never held ${unit}`,);
    }
    // oxlint-disable-next-line eslint/no-await-in-loop -- the same wait, one look at a time; see doc/lint/sequential-awaits.md
    await wait(LOOK_MILLISECONDS,);
  }
}

/**
 Stops a unit a test started and clears what the manager remembers of it, so the machine is left as it was found.

 @param unit - the unit's name

 @returns nothing

 @example
 ```ts
 await stopUnit({ unit: 'pi-build-2.service', },);
 ```
 */
async function stopUnit({ unit, }: { readonly unit: string },): Promise<void> {
  await askTheManager({
    command: 'systemctl',
    words: [
      '--user',
      'stop',
      unit,
    ],
  },);
  await askTheManager({
    command: 'systemctl',
    words: [
      '--user',
      'reset-failed',
      unit,
    ],
  },);
}

/**
 Waits, bounded, until the shell that started a scope has gone and the manager has taken the work in.

 @param pid - the work's process number

 @param from - the process number the shell that started it had

 @returns nothing

 @throws when the work is still that shell's child when the bound is spent

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
  const deadline = Date.now() + MANAGER_WAIT_MILLISECONDS;
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
 Waits, bounded, until `systemd-run` has printed the name of the scope it made.

 The file is written empty before the shell starts, so a look never fails on a path that is not there yet: the message whose name
 this reads is `Running as unit: <name>; invocation ID: <id>`, as `systemd-run` prints it.

 @param file - the file that run's output goes to

 @returns the unit's name

 @throws when no name is printed before the bound is spent

 @example
 ```ts
 const unit = await waitForTheScopeName({ file: '/x/scope-output.txt', },);
 ```
 */
async function waitForTheScopeName({ file, }: { readonly file: string },): Promise<string> {
  /**
   The text `systemd-run` puts before the unit's name.
   */
  const marker = 'Running as unit: ';
  /**
   When this wait must be over.
   */
  const deadline = Date.now() + MANAGER_WAIT_MILLISECONDS;
  for (;;) {
    /**
     What the run has printed so far.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- a look, then a pause, then the next look; see doc/lint/sequential-awaits.md
    const shown = await readFile(file, 'utf8',);
    /**
     What follows the marker, which holds the name up to the semicolon after it, and nothing while no name is printed yet.
     */
    const after = shown.split(marker,)[1] ?? '';
    /**
     Where the name ends.
     */
    const endsAt = after.indexOf(';',);
    if (endsAt > 0) {
      return after.slice(
        0,
        endsAt,
      )
        .trim();
    }
    if (Date.now() >= deadline) {
      throw new Error(`systemd-run printed no unit name into ${file}`,);
    }
    // oxlint-disable-next-line eslint/no-await-in-loop -- the same wait, one look at a time; see doc/lint/sequential-awaits.md
    await wait(LOOK_MILLISECONDS,);
  }
}

/**
 Starts a scope the way a bash tool call leaves one behind: a shell puts it in the background and exits at once, so the work it
 started belongs to whatever took it in when that shell went.

 The scope names no unit of its own, which is the shape the driver can only find through the process table.

 @param folder - the folder the work runs in, and where the run's output and the job's number are written

 @returns the unit, the work's process number and folder, and what stops it

 @example
 ```ts
 await using backgrounded = await startAScopeInTheBackground({ folder: '/x/worktree', },);
 ```
 */
async function startAScopeInTheBackground({ folder, }: { readonly folder: string },): Promise<StartedWork> {
  /**
   The file `systemd-run` prints the name it made into, written empty before the shell goes so a look never fails on it.
   */
  const shownFile = join(folder, 'scope-output.txt',);
  await writeFile(shownFile, '',);
  /**
   The file the shell writes the background job's process number into, which the kernel gave it when it started that job.
   */
  const jobFile = join(folder, 'scope-job-pid.txt',);
  /**
   The shell that starts the scope and goes, as a bash tool call does.
   */
  const caller = spawn(
    'sh',
    [
      '-c',
      `systemd-run --user --scope -- sleep ${String(REAL_PROCESS_SECONDS,)} > ${shownFile} 2>&1 & echo $! > ${jobFile}`,
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
    throw new Error('the operating system started no shell, so no scope was asked for',);
  }
  await once(
    caller,
    'exit',
  );
  /**
   The unit the manager made for it.
   */
  const unit = await waitForTheScopeName({ file: shownFile, },);
  await waitUntilTheManagerHolds({ unit, },);
  /**
   The work's process number, which the shell got when it started the job and `systemd-run` kept by running the payload in its
   own place.
   */
  const pid = Number((await readFile(jobFile, 'utf8',)).trim(),);
  await waitToLeaveTheRun({
    pid,
    from: callerPid,
  },);
  /**
   Ends the work, which is the manager's to do.
   */
  async function stop(): Promise<void> {
    await stopUnit({ unit, },);
  }
  return {
    unit,
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
 Starts a service unit of the run's own by name, which this machine's manager runs, so its process belongs to the manager rather
 than to the run.

 The unit is collected, so the manager unloads it once it ends: a suite killed partway through this test leaves at most the `sleep`
 it started, and no unit of its own.

 @param name - the unit's name, which the test makes unique to itself

 @returns the unit, the work's process number and folder, and what stops it

 @example
 ```ts
 await using named = await startANamedUnit({ name: 'pi-work-left-unit-486601', },);
 ```
 */
async function startANamedUnit({ name, }: { readonly name: string },): Promise<StartedWork> {
  /**
   What `systemd-run` answered when it asked the manager for the unit.
   */
  const started = await askTheManager({
    command: 'systemd-run',
    words: [
      '--user',
      '--collect',
      `--unit=${name}`,
      '--',
      'sleep',
      String(REAL_PROCESS_SECONDS,),
    ],
  },);
  if (started.status !== 0) {
    throw new Error(`systemd-run made no unit ${name}: ${started.stderr.trim()}`,);
  }
  await waitUntilTheManagerHolds({ unit: name, },);
  /**
   What the manager says about the process it runs for that unit.
   */
  const shown = await askTheManager({
    command: 'systemctl',
    words: [
      '--user',
      'show',
      name,
      '--property',
      'MainPID',
      '--value',
    ],
  },);
  /**
   The process number of the work the unit runs.
   */
  const pid = Number(shown.stdout.trim(),);
  if ((!Number.isSafeInteger(pid,)) || (pid <= 0)) {
    throw new Error(`this machine's user manager names no process for ${name}`,);
  }
  /**
   Ends the work, which is the manager's to do.
   */
  async function stop(): Promise<void> {
    await stopUnit({ unit: name, },);
  }
  return {
    unit: name,
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
          name: 'never names a scope the agent put in the background whose caller exited, though it works in the run folder',
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
            // The work: a real scope of the kind a bash call leaves behind when it puts `systemd-run --user --scope` in the
            // background and returns, so the manager has taken the payload in and no chain reaches this test.
            await using backgrounded = await startAScopeInTheBackground({ folder: workingFolder, },);
            // The premise: it really works in the run's own folder, in a control group that is not the run's unit, and it
            // descends from nothing here.
            expect(backgrounded.folder,).toBe(workingFolder,);
            expect(await descendsFrom({
              processFolder: REAL_PROCESS_FOLDER,
              pid: backgrounded.pid,
              rootPid: process.pid,
            },),).toBe(false,);
            await writeRecordsOfRealProcess({
              folder: processes,
              pid: backgrounded.pid,
              worksIn: workingFolder,
            },);
            expect(await workOutlivingTheTurnIn({
              runFolder: folder.path,
              workingFolder,
              processFolder: processes,
              ownProcess: process.pid,
            },),).toEqual([],);
            // The unit this test started is stopped, and the manager says so, rather than being left on the machine.
            await backgrounded.stop();
            expect(await theManagerHolds({ unit: backgrounded.unit, },),).toBe(false,);
          },
        },),
        it({
          name: 'keeps the work a unit of the run own does, whose recorded number has gone, because the manager holds that unit',
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
            // The name a test keeps to itself, so a run of this suite beside it never shares a unit with it.
            const name = `pi-work-left-unit-${String(process.pid,)}`;
            // The work: a real service of the run's own, which the manager runs, so no chain reaches this test either.
            await using named = await startANamedUnit({ name, },);
            expect(await theManagerHolds({ unit: named.unit, },),).toBe(true,);
            expect(await descendsFrom({
              processFolder: REAL_PROCESS_FOLDER,
              pid: named.pid,
              rootPid: process.pid,
            },),).toBe(false,);
            /**
             The command the run's own tool recorded, which names the unit, with the number the tool held now gone.
             */
            const command = `systemd-run --user --collect --unit=${named.unit} -- sleep ${String(REAL_PROCESS_SECONDS,)}`;
            await writeFile(
              join(folder.path, 'feed.jsonl',),
              `${aToolAnswer({
                action: 'start',
                process: aProcess({
                  command,
                  pid: 999_999,
                },),
              },)}\n`,
            );
            // The manager is asked for real here, which is what names the work once the number the tool held has gone.
            expect(await workOutlivingTheTurnIn({
              runFolder: folder.path,
              workingFolder,
              processFolder: processes,
              ownProcess: process.pid,
            },),).toEqual([`proc_7adb (native-build-2) 999999 ${command}`,],);
            await named.stop();
            expect(await theManagerHolds({ unit: named.unit, },),).toBe(false,);
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
