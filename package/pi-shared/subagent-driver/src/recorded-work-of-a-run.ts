/**
 What the process folder says about the process a run's tool recorded, which is the work the run left running only while the process
 at that number is the one the tool named.

 A number alone names no process: the kernel gives a freed number to a later process, which may work in another folder and another
 control group, and a scan that counted the number counted that process as the build. So the record keeps the start the kernel gave
 the process, in clock ticks since the boot, together with the start time the tool itself gave it, and this compares the ticks.

 What this cannot tell: a record whose ticks were never kept, because the line could not be read or disagreed with the tool's start,
 is no evidence either way, so its work counts as work that may still run. A record whose number is held by a process in another PID
 namespace would be compared with that namespace's process, whose ticks are its own; the tool names a number in the namespace the
 driver and Pi share, so a record cannot name one elsewhere.

 @module
 */

import { join, } from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';
import type { WorkOfARun, } from './processes-a-tool-answer-names.ts';
import {
  type ProcessFolderReads,
  ProcessFileUnreadableError,
  aLineOfAProcess,
  lifeIn,
  processHasEnded,
  processHasGone,
  theNumberGivesNoLine,
} from './reads-of-the-process-folder.ts';

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

 A number that holds nothing holds nothing, whatever the record kept: that is `ended`. A number whose process has ended, and one whose
 ticks differ from the record's, are `ended` too. Everything else is no evidence that the work ended, so it counts as work that may
 still run: a record with no number, a record whose ticks were never kept, and a line this account may not read.

 @param processFolder - the folder holding one entry per process, `/proc` here

 @param work - the process the run's tool recorded, with the start the kernel gave it

 @param reads - the reads of one process, which a test replaces

 @returns how the recorded work stands

 @throws ProcessFileUnreadableError when the read failed for a reason that is neither the process having gone nor this account
 being refused

 @example
 ```ts
 const stands = await recordedWorkStands({ processFolder: '/proc', work, reads, },);
 ```
 */
export async function recordedWorkStands({
  processFolder,
  work,
  reads,
}: {
  readonly processFolder: string;
  readonly work: WorkOfARun;
  readonly reads: ProcessFolderReads;
},): Promise<HowTheRecordedWorkStands> {
  if (work.pid === undefined) {
    // A record with no number names no process this can follow: nothing read can say which process it meant, and nothing read says
    // the work ended.
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
    // The number holds nothing, or the folder is this account's to read no more. A number that holds nothing is a process that has
    // gone, whatever the record kept: no start time is needed to know that.
    if (theNumberGivesNoLine({ code: line.code, },)) {
      return (work.birth === undefined) ? 'unknown' : 'ended';
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
   The start the record kept, in clock ticks since the boot.
   */
  const recorded = work.birth;
  if ((recorded === undefined) || (!life.found)) {
    // Nothing here says which process the record named, and nothing says the work ended.
    return 'unknown';
  }
  // Ticks counted from the boot name one process for as long as it lives and never name another: a later process at that number
  // began at another tick. Neither figure moves when the wall clock does.
  return (life.birth === recorded) ? 'runs' : 'ended';
}
