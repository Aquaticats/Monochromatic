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
  birthOfProcessNow,
  type BirthOfAProcessNow,
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
   The start time the kernel gave the process, in clock ticks since boot, when the record was written.

   The tool says nothing of it: the driver reads it once, from `/proc/<pid>/stat`, as it writes the record of an answer that names
   the process, and a scan reads it again later. A number the tool held can be a later process's by then, and this is what tells the
   two apart (`reads-of-the-process-folder.ts`). A record written before this existed, and one whose read failed, carries none.
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
 A feed line with the birth of every process a `processes` record names, so a scan can tell the process it recorded from a later one
 at the same number.

 The record keeps the tool's answer whole but for this one field per process, which the tool did not say. It is read as the line is
 written, because a number alone names no process once the kernel gives it to the next one: the review of 2026-11-14 found a build
 that had ended reported as still running from the number a later process held.

 @param entry - a feed line, of any kind

 @param processFolder - the folder holding one entry per process, `/proc` here

 @param birthOf - how a birth is read now, which a test replaces

 @returns the line, with a birth under every process of a `processes` record whose number could be read

 @example
 ```ts
 const line = withBirthsOfRecordedWork({ entry: { kind: 'processes', processes: [{ id: 'proc_7adb', pid: 486667 }] }, processFolder: '/proc', },);
 ```
 */
export function withBirthsOfRecordedWork({
  entry,
  processFolder,
  birthOf = birthOfProcessNow,
}: {
  readonly entry: PiRecord;
  readonly processFolder: string;
  readonly birthOf?: (request: {
    readonly processFolder: string;
    readonly pid: number;
  }) => BirthOfAProcessNow;
},): PiRecord {
  if (entry.kind !== PROCESSES_RECORD_KIND) {
    return entry;
  }
  /**
   The processes the record names, each with the birth of the process at its number as this moment reads it.
   */
  const processes = recordsIn(entry.processes,)
    .map(function withItsBirth(process,) {
      /**
       The fields of the process the tool named, of which only the number is read here.
       */
      const { pid, } = process;
      if ((typeof pid) !== 'number') {
        return process;
      }
      /**
       What reading the birth now said.
       */
      const read = birthOf({
        processFolder,
        pid,
      },);
      // A birth this moment cannot read is left out, and the line is written all the same: the run's record of what its own tool said
      // must not fail over one process, and a scan reading a line with no birth says the work it named is not provably still there.
      return (read.read === 'birth') ? {
        ...process,
        birth: read.birth,
      } : process;
    },);
  return {
    ...entry,
    processes,
  };
}
