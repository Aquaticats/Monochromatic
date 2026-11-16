/**
 What the process folder says about the process a run's tool recorded, which is the work the run left running only while the process
 at that number is the one the tool named.

 A number alone names no process: the kernel gives a freed number to a later process, which may work in another folder and another
 control group, and a scan that counted the number counted that process as the build. So the record keeps the start time the tool
 itself gave the process, and this compares it with the instant the kernel says the process at that number started.

 @module
 */

import { join, } from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';
import type { WorkOfARun, } from './processes-a-tool-answer-names.ts';
import {
  type ALineOfAProcess,
  type BootTimeInAProcessFolder,
  type ProcessFolderReads,
  ProcessFileUnreadableError,
  aLineOfAProcess,
  lifeIn,
  processHasEnded,
  startInstantIn,
  theNumberGivesNoLine,
} from './reads-of-the-process-folder.ts';

/**
 How much later than the tool's own start time a process may be before it is taken for a later one at the same number.

 The kernel's boot time is whole seconds and its start times count hundredths, so placing a process's start on the wall clock lands
 up to a second early: measured here on 2026-11-15, the boot time and `/proc/uptime` placed the same start 678, 682 and 685 ms
 apart. Two seconds covers that and the latency of the spawn itself. A later process at the same number would have to start inside
 it, and a freed number comes back only after the whole space of numbers has been handed out again.
 */
const PID_REUSE_TOLERANCE_MILLISECONDS = 2_000;

/**
 What the process folder says about the process a record names.
 */
export type HowTheRecordedWorkStands =
  /**
   The process the record named is at that number, and has not ended.
   */
  | 'runs'
  /**
   The process at that number has ended, or is a later one at the same number.
   */
    | 'ended'
  /**
   Nothing read can say which process the record named, and nothing read says it ended.
   */
    | 'unknown';

/**
 Says whether the process the run's tool recorded is still the process at that number.

 The number alone is no evidence: the kernel gives a freed number to a later process, which may work in another folder and another
 control group. So the record keeps the start time the tool itself gave the process, and this compares it with the instant the kernel
 says the process at that number started. A process that started clearly later than the tool's own start is a later one at the same
 number. Anything else is no evidence of that: a record with no start time, a boot time that cannot be read, and a start time that is
 within the tolerance or earlier all leave the work as work that may still run.

 @param processFolder - the folder holding one entry per process, `/proc` here

 @param work - the process the run's tool recorded, with the start time the tool gave

 @param booted - what the process folder says about when the machine booted

 @param reads - the reads of one process, which a test replaces

 @returns how the recorded work stands

 @throws ProcessFileUnreadableError when the read failed for a reason that is neither the process having gone nor this account
 being refused

 @example
 ```ts
 const stands = await recordedWorkStands({ processFolder: '/proc', work, booted, reads, },);
 ```
 */
export async function recordedWorkStands({
  processFolder,
  work,
  booted,
  reads,
}: {
  readonly processFolder: string;
  readonly work: WorkOfARun;
  readonly booted: BootTimeInAProcessFolder;
  readonly reads: ProcessFolderReads;
},): Promise<HowTheRecordedWorkStands> {
  if (work.pid === undefined) {
    // A record with no number names no process this can follow, and nothing read says it ended.
    return 'unknown';
  }
  /**
   The file holding the state and the start time of the process at that number.
   */
  const path = join(
    processFolder,
    String(work.pid,),
    'stat',
  );
  /**
   What that line says, or the failure that stopped it.
   */
  const line = await aLineOfAProcess({
    path,
    reads,
  },);
  if (!line.read) {
    // A process that ended at any instant of this scan, or one whose folder this account may not read, is at no number this run can
    // name. When the record kept the tool's own start time, that is a process that has ended; when it kept none, nothing read says the
    // work ended.
    if (theNumberGivesNoLine({ code: line.code, },)) {
      return (work.startTime === undefined) ? 'unknown' : 'ended';
    }
    throw (line.error instanceof ProcessFileUnreadableError) ? line.error : new ProcessFileUnreadableError({
      path,
      error: line.error,
    },);
  }
  /**
   What the state and the start time of that process are.
   */
  const life = lifeIn({ statText: line.text, },);
  if (life.found && processHasEnded({ state: life.state, },)) {
    return 'ended';
  }
  /**
   The instant the tool said the process started, when its answer kept one.
   */
  const recorded = work.startTime;
  if (recorded === undefined) {
    // Without the tool's own start time, nothing but the number ties the record to a process, and a number is no name.
    return 'unknown';
  }
  /**
   The instant the kernel says the process at that number started.
   */
  const started = startInstantIn({
    life,
    booted,
  },);
  if (!started.found) {
    // The kernel's start time cannot be placed on the wall clock, so there is nothing to compare the tool's with.
    return 'unknown';
  }
  // Only a process that started clearly later than the tool's own start is a later one: a freed number comes back only after the
  // whole space of numbers has been handed out again, so nothing else can put a later process within the tolerance.
  return (started.instant > (recorded + PID_REUSE_TOLERANCE_MILLISECONDS)) ? 'ended' : 'runs';
}
