/**
 What one answer of the run's own `process` tool says it holds: the tool's name for the process, the number it started, the folder
 it works in and the command it runs.

 Kept apart from the reader that asks the machine what still runs, because two places need the same reading: the feed's writer,
 which keeps the answer whole as one record of its own, and the reader of that record. The `tool_end` line of the feed carries
 only a preview of the tool's answer, and a preview cut inside a JSON string can never be read as one: on 2026-11-13 that cut
 text ended eleven runs of this machine as `driver-failed` after their work was whole and their Pi had exited 0.

 @module
 */

import type { PiRecord, } from './driver-types.ts';
import {
  lifeOfOneProcessNow,
  type LifeOfAProcess,
  startInstantIn,
} from './reads-of-the-process-folder.ts';
import {
  isRecord,
  recordsIn,
} from './read-pi-fields.ts';

/**
 The tool whose answers name the work a run started.
 */
export const PROCESS_TOOL_NAME = 'process';

/**
 The feed record that carries one whole answer of the `process` tool, so a reader needs no preview of it.

 The `tool_end` record of the same answer keeps a preview, cut at the length a feed line may hold, and that is what a reader
 cannot parse.
 */
export const PROCESSES_RECORD_KIND = 'processes';

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

  /**
   The start the kernel gave the process, in clock ticks since boot, kept when it agrees with the tool's own start time.

   Ticks counted from the boot say which process this is, and no later process at the same number shares them; unlike a wall-clock
   instant worked out from the folder's boot time, they do not move when the wall clock does. A record written before the driver kept
   them, one whose line could not be read, and one whose ticks disagree with the tool's start by more than the tolerance carry none,
   and a scan then counts the work as work that may still run.
   */
  readonly birth?: string;
};

/**
 Reads one process the tool's answer names, or no process when the entry names none a reader can follow.

 @param value - one entry of the answer's processes

 @returns the process, as one entry, or nothing when that entry names no process

 @example
 ```ts
 const process = processesInOne({ value: { id: 'proc_7adb', command: 'cargo build', cwd: '/repo' }, });
 ```
 */
function processesInOne({ value, }: { readonly value: unknown },): readonly WorkOfARun[] {
  if ((!isRecord(value,)) || ((typeof value.id) !== 'string')) {
    return [];
  }
  /**
   The tool's own name for the process.
   */
  const { id, } = value;
  return [{
    id,
    name: ((typeof value.name) === 'string') ? value.name : id,
    ...((typeof value.pid) === 'number' ? { pid: value.pid, } : {}),
    command: ((typeof value.command) === 'string') ? value.command : '',
    cwd: ((typeof value.cwd) === 'string') ? value.cwd : '',
    ...((typeof value.status) === 'string' ? { status: value.status, } : {}),
    ...((typeof value.startTime) === 'number' ? { startTime: value.startTime, } : {}),
    ...((typeof value.birth) === 'string' ? { birth: value.birth, } : {}),
  },];
}

/**
 Reads every process an answer of the `process` tool names, whether it started one, updated one, or listed them.

 The answer the tool gives is Pi's own: the processes sit under `details`, as one `process` or as a `processes` list, and a later
 answer about the same process repeats its name.

 @param answer - what the tool answered, of any shape

 @returns one entry per process it named, in the order it named them

 @example
 ```ts
 const processes = processesInToolAnswer({ answer: { details: { process: { id: 'proc_7adb' } } }, });
 ```
 */
export function processesInToolAnswer({ answer, }: { readonly answer: unknown },): readonly WorkOfARun[] {
  /**
   What the answer says about the processes, if anything.
   */
  const { details, } = isRecord(answer,) ? answer : { details: undefined, };
  return processesNamedIn({ value: details, },);
}

/**
 Reads every process a value names, under the one `process` and the `processes` list the tool's answers and the feed use.

 @param value - what names processes, of any shape

 @returns one entry per process it named, in the order it named them

 @example
 ```ts
 const processes = processesNamedIn({ value: { processes: [{ id: 'proc_7adb' }] }, });
 ```
 */
export function processesNamedIn({ value, }: { readonly value: unknown },): readonly WorkOfARun[] {
  if (!isRecord(value,)) {
    return [];
  }
  return [
    ...(isRecord(value.process) ? [value.process,] : []),
    ...recordsIn(value.processes,),
  ].flatMap(function processNamed(named,) {
    return processesInOne({ value: named, },);
  },);
}

/**
 How far the start a process's own line gives may be from the start time the tool gave before that line is taken to be another
 process's.

 The tool stamps the process it started, and the kernel's ticks say when that process started; two readings of one start differ by
 the tick's hundredth of a second and by how long the tool took to stamp. The folder's own boot time is whole seconds, which is why
 the tolerance covers a second as well. Measured here on 2026-11-15, the boot time and `/proc/uptime` placed one start 678, 682 and
 685 ms apart. The tolerance has two sides: a line whose start is clearly earlier than the tool's is another process's as surely as
 one whose start is clearly later.
 */
export const BIRTH_TOLERANCE_MILLISECONDS = 2_000;

/**
 A feed line with the start the kernel gave each process a `processes` record names.

 The record keeps what the tool said, and adds this one field per process, which the tool did not say. It is kept only when the line
 it was read from agrees with the start time the tool itself gave, within `BIRTH_TOLERANCE_MILLISECONDS`: a number the tool held can
 have been given to a later process before this read, and that process's start is not this answer's. A line this instant cannot read,
 a record the tool gave no start time for, and a disagreement all leave the field out, and a scan reading such a record treats the
 work as work that may still run.

 @param entry - a feed line, of any kind

 @param processFolder - the folder holding one entry per process, `/proc` here

 @param now - where the driver reads the time, which a test replaces

 @param lifeOf - how one process's line is read now, which a test replaces

 @returns the line, with a start under every process of a `processes` record whose line agrees with the tool's start time

 @example
 ```ts
 const line = withBirthsOfRecordedWork({ entry: { kind: 'processes', processes: [{ id: 'proc_7adb', pid: 486667, startTime }] }, processFolder: '/proc', },);
 ```
 */
export function withBirthsOfRecordedWork({
  entry,
  processFolder,
  now = Date.now,
  lifeOf = lifeOfOneProcessNow,
}: {
  readonly entry: PiRecord;
  readonly processFolder: string;
  readonly now?: () => number;
  readonly lifeOf?: (request: {
    readonly processFolder: string;
    readonly pid: number;
  }) => LifeOfAProcess;
},): PiRecord {
  if (entry.kind !== PROCESSES_RECORD_KIND) {
    return entry;
  }
  /**
   The processes the record names, each with the start the kernel gave it where that start is this answer's.
   */
  const processes = recordsIn(entry.processes,)
    .map(function withItsBirth(process,) {
      /**
       The fields of the process the tool named, of which the number and the start time are read here.
       */
      const {
        pid,
        startTime,
      } = process;
      if (((typeof pid) !== 'number') || ((typeof startTime) !== 'number')) {
        return process;
      }
      /**
       What the line of that process's own record says now.
       */
      const life = lifeOf({
        processFolder,
        pid,
      },);
      if (!life.found) {
        return process;
      }
      /**
       Where the kernel says that process started, at this instant.
       */
      const started = startInstantIn({
        birth: life.birth,
        now: now(),
      },);
      // Only a start that agrees with the tool's own start names this answer's process: a later process at that number, and a
      // tool whose clock differs from this machine's by more than the tolerance, leave the record without one.
      if ((!started.found) || (Math.abs(started.instant - startTime) > BIRTH_TOLERANCE_MILLISECONDS)) {
        return process;
      }
      return {
        ...process,
        birth: life.birth,
      };
    },);
  return {
    ...entry,
    processes,
  };
}
