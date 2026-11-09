/**
 Reading Pi's RPC records as they arrive: one JSON record per line, answers handed to the commands waiting for them, and the
 compaction Pi reports kept, because Pi at work is not Pi silent.

 @module
 */

import { NO_RUN_FAILURE, } from './driver-constants.ts';
import type {
  CommandAnswer,
  CompactionOfPi,
  PiRecord,
  RunFailure,
} from './driver-types.ts';
import type { ConnectionListeners, } from './shapes-of-the-connection.ts';
import {
  isRecord,
  textOf,
} from './read-pi-fields.ts';
import { startLineSplitter, } from './split-pi-output.ts';
import type { WaitingForAnAnswer, } from './wait-for-an-answer.ts';

/**
 Stands for a line that held no record.
 */
const UNREADABLE_LINE: PiRecord = Object.freeze({},);

/**
 Reads what a compaction record says of Pi's session.

 @param record - the `compaction_end` record, or any other

 @returns why the compaction failed, or the fact that it did not
 */
function compactionOutcomeOf(record: PiRecord,): RunFailure {
  if (record.aborted === true) {
    return {
      failed: true,
      problem: 'Pi aborted the compaction',
    };
  }
  /**
   What Pi said of the failure, which is empty for a compaction that worked.
   */
  const message = textOf(record.errorMessage,);
  return (message === '')
    ? NO_RUN_FAILURE
    : {
      failed: true,
      problem: message,
    };
}

/**
 What the records of one channel are read from, and who hears what they say.
 */
export type PiRecordsRequest = {
  /**
   Pi's standard output, which carries one record per line.
   */
  readonly output: NodeJS.ReadableStream;

  /**
   Pi's standard input, which carries the commands.
   */
  readonly input: NodeJS.WritableStream;

  /**
   Whether Pi has gone, which is when writing a command has no reader left.
   */
  readonly hasExited: () => boolean;

  /**
   The commands waiting for their answers, by identifier.
   */
  readonly waitingForAnswer: ReadonlyMap<string, WaitingForAnAnswer>;

  /**
   What the run wants to hear about.
   */
  readonly listeners: ConnectionListeners;
};

/**
 The records of one channel.
 */
export type PiRecords = {
  /**
   Writes a command if Pi can still read it.
   */
  readonly write: (command: PiRecord,) => void;

  /**
   What Pi reports of its own session, which moves the bounds of every waiting command.
   */
  readonly compactionOfPi: () => CompactionOfPi;
};

/**
 Starts reading Pi's records from its output.

 @param output - Pi's standard output

 @param input - Pi's standard input

 @param hasExited - whether Pi has gone

 @param waitingForAnswer - the commands waiting for their answers

 @param listeners - what the run wants to hear about

 @returns where commands are written and what Pi reports of its session

 @example
 ```ts
 const records = startReadingPiRecords({ output, input, hasExited, waitingForAnswer, listeners, },);
 ```
 */
export function startReadingPiRecords(
  {
    output,
    input,
    hasExited,
    waitingForAnswer,
    listeners,
  }: PiRecordsRequest,
): PiRecords {
  /**
   What changes while Pi runs: the compaction it reports of its own session, or the fact that it reports none.
   */
  const state: {
    compaction: CompactionOfPi;
  } = {
    compaction: { compacting: false, },
  };

  /**
   Parses a line, reporting it when it holds no record.

   @param line - text of one record

   @returns the record, or the marker for a line that held none
   */
  function parseLine(line: string,): PiRecord {
    try {
      /**
       The line's value, of any shape.
       */
      const parsed: unknown = JSON.parse(line,);
      if (isRecord(parsed,)) {
        return parsed;
      }
      listeners.onUnreadableLine(line,);
      return UNREADABLE_LINE;
    }
    catch (error) {
      listeners.onUnreadableLine(`${line} (${String(error,)})`,);
      return UNREADABLE_LINE;
    }
  }

  /**
   Notes whether Pi reports a compaction beginning or ending, because Pi at work is not Pi silent: a command waiting for its
   answer stops counting silence for as long as the compaction runs, and keeps what Pi said of a compaction that failed.

   @param record - a record Pi wrote that answers no command
   */
  function noteCompaction(record: PiRecord,): void {
    if (record.type === 'compaction_start') {
      /**
       Whether this record begins the compaction rather than reporting one already under way.
       */
      const {
        compaction: { compacting: alreadyCompacting },
      } = state;
      state.compaction = {
        compacting: true,
        reason: textOf(record.reason,),
        since: Date.now(),
      };
      if (alreadyCompacting) {
        // A second report of the same compaction is no new silence to stop counting, so no wait hears of it twice.
        return;
      }
      for (const waiting of waitingForAnswer.values()) {
        waiting.pauseSilence();
      }
      return;
    }
    if (record.type !== 'compaction_end') {
      return;
    }
    /**
     Why the compaction failed, when it did.
     */
    const compaction = compactionOutcomeOf(record,);
    /**
     Whether a compaction was reported under way, whose end the waits are counting from.
     */
    const {
      compaction: { compacting: wasCompacting },
    } = state;
    state.compaction = { compacting: false, };
    if (!wasCompacting) {
      return;
    }
    for (const waiting of waitingForAnswer.values()) {
      waiting.resumeSilence(compaction,);
    }
  }

  /**
   Hands one line to whoever waits for it.

   @param line - text of one record
   */
  function readLine(line: string,): void {
    if (line.trim() === '') {
      return;
    }
    /**
     The record, or the marker for a line that held none.
     */
    const record = parseLine(line,);
    if (record === UNREADABLE_LINE) {
      return;
    }
    if ((record.type === 'response') && ((typeof record.id) === 'string')) {
      /**
       The answer, which is Pi's own record rather than the driver's reading of it.
       */
      const answered: CommandAnswer = {
        ending: 'answered',
        record,
      };
      waitingForAnswer.get(record.id,)
        ?.settle(answered,);
      return;
    }
    noteCompaction(record,);
    listeners.onEvent(record,);
  }

  /**
   Writes a command if Pi can still read it.

   @param command - record to send
   */
  function write(command: PiRecord,): void {
    if (!hasExited()) {
      input.write(`${JSON.stringify(command,)}\n`,);
    }
  }

  /**
   What Pi reports of its own session.

   @returns the compaction under way, or the fact that Pi reports none
   */
  function compactionOfPi(): CompactionOfPi {
    return state.compaction;
  }

  output.on(
    'data',
    startLineSplitter({ onLine: readLine, },),
  );

  return {
    write,
    compactionOfPi,
  };
}
