/**
 The library surface of reading a run's own records: the lines of a feed that a writer has finished, and what one answer of the
 run's `process` tool says it holds. Kept in a file of its own so `index.ts` stays inside the line budget it is held to, the way
 the surfaces of starting and reviewing a run are.

 @module
 */

/**
 What one answer of the run's own `process` tool says it holds, and the feed record that keeps it whole. The feed's own `tool_end`
 line carries only a preview of that answer, and text cut inside a JSON string can never be read back.
 */
export {
  BIRTH_TOLERANCE_MILLISECONDS,
  PROCESS_TOOL_NAME,
  PROCESSES_RECORD_KIND,
  type WorkOfARun,
  processesInToolAnswer,
  processesNamedIn,
  withBirthsOfRecordedWork,
} from './processes-a-tool-answer-names.ts';

/**
 What the process folder says about the process a run's tool recorded: whether the process at its number is the one the record named.
 */
export {
  type HowTheRecordedWorkStands,
  recordedWorkStands,
} from './recorded-work-of-a-run.ts';

/**
 Reads of one process's own files from the process folder, the codes a process that has ended answers to them, and the start the
 kernel gave a process, in ticks since the boot, that tells it from a later one at its number.
 */
export {
  CLOCK_TICKS_PER_SECOND,
  GONE_PROCESS_CODES,
  type ALineOfAProcess,
  type LifeOfAProcess,
  type ProcessFolderReads,
  ProcessFileUnreadableError,
  type StartInstantOfAProcess,
  aLineOfAProcess,
  lifeIn,
  lifeOfOneProcessNow,
  processHasEnded,
  processHasGone,
  readsOfTheProcessFolder,
  startInstantIn,
  theNumberGivesNoLine,
} from './reads-of-the-process-folder.ts';

/**
 The lines of a feed a writer has finished, for a reader that opens a file another process is still appending to.
 */
export { wholeLinesOf, } from './whole-lines-of-a-feed.ts';
