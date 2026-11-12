/**
 The work one run left running: what the run's own `process` tool still reports as going, and what the machine's process table
 shows working in the run's folders outside the run's own unit.

 A run that ends its turn while a build it started is still compiling is not a run that finished: the driver that ended one as
 `idle` on 2026-11-12 closed a Pi whose session was waiting for that build, and the agent's work was never read. Briefs tell
 agents that ending a turn while work runs is right, and that the launcher resumes them, so the driver has to keep the session
 open until the work is over.

 Three things are asked, and only what they show is used:

 - the run's own `process` tool, whose every start, update and list is a record of the run's feed, and whose records carry the
   process number, the folder it was started in and the command it runs;
 - the machine's process table, for a process working in the run's worktree or run folder whose control group is not the run's
   own unit: a `systemd-run --scope` the agent made leaves the unit's control group, so it is visible there and the unit itself
   is not;
 - the unit a command names with `--unit=`, asked of the manager, for work whose process the tool recorded but whose number has
   gone: a service of its own starts from the manager rather than from the run, so its number is not the tool's to keep.

 What none of them can show is not guessed at: a process whose folder cannot be read counts for nothing, and a run whose work
 nothing names ends as it ended before this existed.

 @module
 */

import {
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';
import { isRecord, } from './read-pi-fields.ts';
import {
  type RunCommand,
  runCommandWithChildProcess,
} from './run-command.ts';
import { processesAtTheirFolders, } from './wake-a-run-whose-work-outlives-its-turn.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The statuses the `process` tool holds a process under while it still runs, which are the ones a run may be waiting for.
 */
const LIVE_STATUSES: ReadonlySet<string> = new Set<string>([
  'running',
  'terminating',
  'terminate_timeout',
]);

/**
 The number of characters of a command a line carries, so a line stays a line.
 */
const COMMAND_CHARACTERS = 160;

/**
 One process a run started, as the run's own `process` tool recorded it.
 */
export type WorkOfARun = {
  /**
   The tool's name for the process, such as `proc_7adb`.
   */
  readonly id: string;

  /**
   The name the agent gave it, which is the tool's own name when it gave none.
   */
  readonly name: string;

  /**
   The process number the tool holds, which is no number when the tool recorded none.
   */
  readonly pid?: number;

  /**
   The command the tool started.
   */
  readonly command: string;

  /**
   The folder the tool started it in.
   */
  readonly cwd: string;

  /**
   The status the tool last reported, when a record named one.
   */
  readonly status?: string;
};

/**
 Reads the processes a run's own `process` tool last reported as running.

 The records are the feed's own: every start, update and list the tool answered is a `tool_end` whose result carries the process
 or the processes it answered with, so what the tool holds is read without asking Pi, whose session may already be over.

 @param feedShown - the run's feed, one JSON record per line

 @returns the processes the tool reported and did not later report as ended

 @example
 ```ts
 const running = workTheToolReportedIn({ feedShown: await readFile(feedFile, 'utf8',), },);
 ```
 */
export function workTheToolReportedIn({ feedShown, }: { readonly feedShown: string },): readonly WorkOfARun[] {
  /**
   The last status the tool reported for each process, by the tool's name for it.
   */
  const reported = new Map<string, WorkOfARun>();
  for (const line of feedShown.split('\n',)) {
    if (line === '') {
      continue;
    }
    /**
     One record of the feed, of any shape.
     */
    const record: unknown = JSON.parse(line,);
    if ((!isRecord(record,)) || (record.kind !== 'tool_end')
      || (record.tool !== 'process')
      || (record.isError === true)) {
      continue;
    }
    if ((typeof record.result) !== 'string') {
      continue;
    }
    /**
     What the tool answered, of any shape.
     */
    const result: unknown = JSON.parse(record.result,);
    /**
     What that answer says about processes, if anything.
     */
    const { details, } = isRecord(result,) ? result : { details: undefined, };
    if (!isRecord(details,)) {
      continue;
    }
    /**
     Every process this answer names, one or many, as the answer holds them.
     */
    const listed: readonly unknown[] = Array.isArray(details.processes) ? details.processes : [];
    /**
     What this answer says about the processes, one of them or a list.
     */
    const named: readonly unknown[] = [
      ...(isRecord(details.process,) ? [details.process,] : []),
      ...listed,
    ];
    for (const process of named) {
      if (!isRecord(process,)) {
        continue;
      }
      /**
       The tool's name for it, which is what a later record repeats.
       */
      const { id, } = process;
      if ((typeof id) !== 'string') {
        continue;
      }
      reported.set(
        id,
        {
          id,
          name: ((typeof process.name) === 'string') ? process.name : id,
          ...((typeof process.pid) === 'number' ? { pid: process.pid, } : {}),
          command: ((typeof process.command) === 'string') ? process.command : '',
          cwd: ((typeof process.cwd) === 'string') ? process.cwd : '',
          ...((typeof process.status) === 'string' ? { status: process.status, } : {}),
        },
      );
    }
  }
  return [...reported.values()].filter(function stillRuns(work,) {
    return work.status === undefined ? false : LIVE_STATUSES.has(work.status,);
  },);
}

/**
 Reads the control group a process is in, from the process folder's own record of it.

 @param processFolder - the folder holding one entry per process, `/proc` here

 @param entry - the process's entry in that folder, or `self`

 @returns the control group, or empty text when it cannot be read

 @example
 ```ts
 const cgroup = await cgroupOf({ processFolder: '/proc', entry: 'self', },);
 ```
 */
export async function cgroupOf({
  processFolder,
  entry,
}: {
  readonly processFolder: string;
  readonly entry: string;
},): Promise<string> {
  try {
    return (await readFile(
      join(
        processFolder,
        entry,
        'cgroup',
      ),
      'utf8',
    )).trim();
  }
  catch (error) {
    // A process that ended, or one whose folder this account may not read, is in no control group this run can name.
    if ((errorCodeOf(error,) === MISSING_PATH_CODE) || (errorCodeOf(error,) === 'EACCES')
      || (errorCodeOf(error,) === 'EPERM')) {
      return '';
    }
    throw error;
  }
}

/**
 Says whether a process number is one the process folder still holds.

 @param processFolder - the folder holding one entry per process

 @param pid - the process number

 @returns whether the process is there

 @example
 ```ts
 const alive = processIsThere({ processFolder: '/proc', pid: 486667, },);
 ```
 */
async function processIsThere({
  processFolder,
  pid,
}: {
  readonly processFolder: string;
  readonly pid: number;
},): Promise<boolean> {
  /**
   The control group of a process that is there, which is never empty.
   */
  const cgroup = await cgroupOf({
    processFolder,
    entry: String(pid,),
  },);
  return cgroup !== '';
}

/**
 Reads the unit a command starts, when it starts one of its own.

 A command started with `systemd-run --user --unit=<name>` runs in a unit of its own, which outlives the command that asked for
 it, so the number the `process` tool holds stops naming the work while the unit still runs it.

 @param command - the command the tool started

 @returns the unit's name, or nothing when the command names none

 @example
 ```ts
 const unit = unitNamedBy({ command: 'systemd-run --user --unit=pi-build true', },);
 ```
 */
export function unitNamedBy({ command, }: { readonly command: string },): { readonly unit?: string } {
  /**
   The words the command was written with, which whitespace of any width separates.
   */
  const words = command.split(' ',)
    .filter(function isAWord(word,) {
      return word !== '';
    },);
  for (const [at, word,] of words.entries()) {
    if (word.startsWith('--unit=',)) {
      return { unit: word.slice('--unit='.length,), };
    }
    /**
     The word after `--unit`, which names the unit when it stands apart.
     */
    const named = words[at + 1];
    if ((word === '--unit') && (named !== undefined)) {
      return { unit: named, };
    }
  }
  return {};
}

/**
 Reads what a process works on, as a line a reader needs.

 @param work - the process, and the name the tool gave it

 @returns the line

 @example
 ```ts
 const line = lineOf({ work, },);
 ```
 */
function lineOf({ work, }: { readonly work: WorkOfARun },): string {
  /**
   How the tool named it, which is what a reader matches against the agent's own words.
   */
  const named = (work.name === work.id) ? work.id : `${work.id} (${work.name})`;
  /**
   The command, cut so the line stays one line.
   */
  const command = work.command
    .slice(
    0,
    COMMAND_CHARACTERS,
    );
  return `${named}${((work.pid === undefined) ? '' : ` ${String(work.pid,)}`)}${((command === '') ? '' : ` ${command}`)}`;
}

/**
 Reads the work one run left running, as the lines that name it.

 @param runFolder - the run's own folder, whose feed names what its tool started

 @param workingFolder - the folder the run works in

 @param processFolder - the folder holding one entry per process, `/proc` here

 @param run - how a command is run, which a test replaces

 @returns one line per process the run still has going, empty when it has none the driver can see

 @example
 ```ts
 const work = await workOutlivingTheTurnIn({ runFolder: '/runs/a-1', workingFolder: '/repo/.claude/worktrees/pi-a', processFolder: '/proc', },);
 ```
 */
export async function workOutlivingTheTurnIn({
  runFolder,
  workingFolder,
  processFolder,
  run = runCommandWithChildProcess,
}: {
  readonly runFolder: string;
  readonly workingFolder: string;
  readonly processFolder: string;
  readonly run?: RunCommand;
},): Promise<readonly string[]> {
  /**
   The feed the run's own tool wrote its answers into.
   */
  const feedShown = await readFileOrNothing(join(
    runFolder,
    'feed.jsonl',
  ),);
  /**
   What the tool reported and did not report as ended.
   */
  const reported = workTheToolReportedIn({ feedShown, },);
  /**
   The control group of the run's own unit, which is where the driver and Pi run and no work of the agent's does.
   */
  const ownCgroup = await cgroupOf({
    processFolder,
    entry: 'self',
  },);
  /**
   The lines that name the work still going.
   */
  const lines: string[] = [];
  /**
   The process numbers already named, so a process is not named twice.
   */
  const named = new Set<number>();
  for (const work of reported) {
    /**
     Whether the number the tool holds is still a process.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- one process after another, each read before the next is asked about; a handful of processes at most; see doc/lint/sequential-awaits.md
    const alive = (work.pid === undefined) ? false : await processIsThere({
      processFolder,
      pid: work.pid,
    },);
    /**
     Whether the unit the command names still runs it.
     */
    const { unit, } = alive ? {} : unitNamedBy({ command: work.command, },);
    /**
     What the manager says about that unit.
     */
    const unitRuns = (unit === undefined)
      ? false
      // oxlint-disable-next-line eslint/no-await-in-loop -- the same handful of processes, each asked once; see doc/lint/sequential-awaits.md
      : (await run({
        command: 'systemctl',
        words: [
          '--user',
          'is-active',
          unit,
        ],
      },)).stdout
        .trim()
        === 'active';
    if ((!alive) && (!unitRuns)) {
      continue;
    }
    lines.push(lineOf({ work, },),);
    if (work.pid !== undefined) {
      named.add(work.pid,);
    }
  }
  /**
   What the process table shows working in the run's own folders.
   */
  const processes = await processesAtTheirFolders({ processFolder, },);
  for (const process of processes) {
    if (named.has(process.pid,)) {
      continue;
    }
    /**
     Whether it works in one of the run's own folders, which is the run's work however it was started.
     */
    const worksInOwnFolder = [
      workingFolder,
      runFolder,
    ].some(function isTheFolder(own,) {
      return (own !== '') && ((process.folder === own)
        || process.folder
          .startsWith(`${own}/`,));
    },);
    if (!worksInOwnFolder) {
      continue;
    }
    /**
     The control group it is in, which tells the run's own unit from a scope the agent made.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- one process after another, each read once; see doc/lint/sequential-awaits.md
    const cgroup = await cgroupOf({
      processFolder,
      entry: String(process.pid,),
    },);
    // A process in the run's own unit is the driver or the Pi of this run, which is no work the run left: a scope the agent
    // made leaves that unit, and a service of its own never joins it. A control group nobody can read is no evidence either
    // way, so it is not counted.
    if ((cgroup === '') || (cgroup === ownCgroup)) {
      continue;
    }
    lines.push(`${String(process.pid,)} ${process.command}`,);
  }
  return lines;
}

/**
 Reads a file, or answers nothing when it is not there.

 @param file - the file to read

 @returns its text, or empty text when it is missing

 @example
 ```ts
 const shown = await readFileOrNothing('/runs/a-1/feed.jsonl',);
 ```
 */
async function readFileOrNothing(file: string,): Promise<string> {
  try {
    return await readFile(
      file,
      'utf8',
    );
  }
  catch (error) {
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return '';
    }
    throw error;
  }
}

/**
 Reads the line the driver sends an agent whose work has ended, so it goes on.

 @param work - the lines that named the work when the driver began to wait

 @returns the message

 @example
 ```ts
 const said = wakeMessageOf({ work: ['proc_7adb (native-build-2) 486667 systemd-run'], },);
 ```
 */
export function wakeMessageOf({ work, }: { readonly work: readonly string[] },): string {
  return [
    'The work you left running when your turn ended has ended:',
    ...work.map(function bulleted(line,) {
      return `- ${line}`;
    },),
    'The driver watched the process table rather than your process tool, so read how each one ended with the `process` tool '
      + 'and continue your task from where you stopped. Ending your turn again while something runs is fine; you are resumed '
      + 'when it ends.',
  ].join('\n',);
}
