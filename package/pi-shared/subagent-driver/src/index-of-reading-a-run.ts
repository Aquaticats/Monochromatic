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
  PROCESS_TOOL_NAME,
  PROCESSES_RECORD_KIND,
  type WorkOfARun,
  processesInToolAnswer,
  processesNamedIn,
} from './processes-a-tool-answer-names.ts';

/**
 The lines of a feed a writer has finished, for a reader that opens a file another process is still appending to.
 */
export { wholeLinesOf, } from './whole-lines-of-a-feed.ts';

/**
 The reads the driver makes inside one process's folder, which a test replaces so that a process ending between the folder being
 listed and a record being read can be told from a read that truly failed.
 */
export {
  READS_OF_A_PROCESS_FOLDER,
  type ReadsAProcessRecord,
  type ReadsOfAProcessFolder,
} from './reads-of-a-process-folder.ts';
