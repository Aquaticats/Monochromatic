/**
 What a run's feed says about the work it left running: every process the run's own `process` tool last reported, read out of the
 feed's records as far as each record's copy of the answer goes.

 The feed keeps 500 characters of a tool result and a marker for the rest, so a `process` answer longer than that arrives cut and
 cannot be parsed whole. An answer nothing can be read from never throws: the run goes on, and the process it may have started is
 work that may still run rather than no work at all (issue 6295).

 @module
 */

import { isRecord, } from './read-pi-fields.ts';
import { asMuchJsonAsTheTextHolds, } from './reading-as-much-json-as-the-text-holds.ts';

/**
 The statuses the `process` tool holds a process under while it still runs, which are the ones a run may be waiting for.
 */
const LIVE_STATUSES: ReadonlySet<string> = new Set<string>([
  'running',
  'terminating',
  'terminate_timeout',
]);

/**
 The marker every process the tool names carries before its own name, as `proc_7adb`.
 */
const PROCESS_NAME = 'proc_';

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

  /**
   Whether the feed holds only a cut copy of the answer that last named this process, so whether it still runs is the process
   table's to say rather than the tool's.

   Absent for a process the feed names in an answer it holds whole.
   */
  readonly statusIsUnknown?: true;
};

/**
 What a run's own `process` tool reported, and what the feed could not say.
 */
export type ReportedWorkOfARun = {
  /**
   The processes the tool last named that may still run, which is every process whose last word was `running`, and every process
   whose last word the feed holds only a cut copy of.
   */
  readonly running: readonly WorkOfARun[];

  /**
   Every answer the feed holds only a cut copy of, from which no process could be read at all, with why it could not be parsed: work
   one of them may have started is work this cannot name, so it is treated as work that may still run.
   */
  readonly unreadableAnswers: readonly string[];
};

/**
 Reads every process one answer of the tool names, from the answer's own details.

 The tool answers in three shapes, and all three are read: a `start`, `status` or `stop` answer names one process under `details`;
 a `list` answer names several under `details.processes`; and an `output` answer names the process it read under flat `details.id`
 and `details.processStatus`. A `stop` answer carries the process under `details.result.info`.

 @param details - the answer's details, of any shape

 @returns every process the answer names

 @example
 ```ts
 const named = processesNamedBy({ details: { action: 'list', processes: [], }, },);
 ```
 */
function processesNamedBy({ details, }: { readonly details: unknown },): readonly unknown[] {
  if (!isRecord(details,)) {
    return [];
  }
  /**
   The processes a `list` answer names, which is nothing for every other answer.
   */
  const listed: readonly unknown[] = Array.isArray(details.processes) ? details.processes : [];
  /**
   What a `stop` answer says about the process it named.
   */
  const stopped = isRecord(details.result) ? details.result
    .info : undefined;
  return [
    ...(isRecord(details.process) ? [details.process,] : []),
    ...listed,
    ...(isRecord(stopped) ? [stopped,] : []),
    // An answer that reads a process's output names it, and its status, in the details themselves.
    ...(((typeof details.id) === 'string')
      ? [{
        id: details.id,
        name: details.processName,
        status: details.processStatus,
      },]
      : []),
  ];
}

/**
 Reads the answer one record of the feed holds, whole or cut.

 The feed keeps 500 characters of a result and a marker for the rest, so an answer longer than that cannot be parsed; what the
 feed holds of it is read as far as it goes, and the process it names is marked as one whose status the feed no longer says. An
 answer nothing can be read from is no answer: it is counted, never thrown (issue 6295, where a cut answer ended two finished
 runs as `driver-failed`).

 @param shown - the record's own copy of the answer

 @returns the answer's details as far as they were whole, and whether the copy was cut

 @example
 ```ts
 const read = answerIn({ shown: record.result, },);
 ```
 */
function answerIn({ shown, }: { readonly shown: string },): {
  readonly details: unknown;
  readonly cut: boolean;
  readonly problem?: string
} {
  try {
    /**
     The answer, whole.
     */
    const whole: unknown = JSON.parse(shown,);
    return {
      details: isRecord(whole) ? whole.details : undefined,
      cut: false,
    };
  }
  catch (error) {
    /**
     As much of the answer as the cut copy holds.
     */
    const read = asMuchJsonAsTheTextHolds({ shown, },);
    return {
      details: isRecord(read.value) ? read.value
        .details : undefined,
      cut: true,
      problem: reasonOfFailure({ error, },),
    };
  }
}

/**
 Reads the names of processes an answer mentions anywhere in its text.

 A cut copy of a `list` answer may hold none of the answer's details and all of the processes it named, in the text the tool shows
 the agent (`proc_b6c6` followed by the status, the name and the command). Only names the caller already knows are of use: they are
 processes whose number the caller holds, so whether they still run is the process table's to say rather than this answer's.

 @param shown - the answer as the feed holds it

 @returns every process name the answer's text mentions, in the order it mentions them

 @example
 ```ts
 const ids = processNamesIn({ shown: 'proc_b6c6\trunning\tmatrix\t...', },);
 ```
 */
function processNamesIn({ shown, }: { readonly shown: string },): readonly string[] {
  /**
   The names mentioned, without a repeat.
   */
  const names: string[] = [];
  /**
   Where this scan looks for the next name.
   */
  let at = 0;
  while (at < shown.length) {
    /**
     Where the next name's marker is, which is nothing when the text holds no more.
     */
    const found = shown.indexOf(
      PROCESS_NAME,
      at,
    );
    if (found === (-1)) {
      return names;
    }
    /**
     Where the name's own characters end.
     */
    let end = found + PROCESS_NAME.length;
    while ((end < shown.length) && isNameCharacter({ character: shown.charAt(end,), },)) {
      end += 1;
    }
    if (end > (found + PROCESS_NAME.length)) {
      /**
       The name, which is the marker and the characters after it.
       */
      const name = shown.slice(
        found,
        end,
      );
      if (!names.includes(name,)) {
        names.push(name,);
      }
    }
    at = end;
  }
  return names;
}

/**
 Says whether one character may be part of a process's own name.

 @param character - the character

 @returns whether it is a lower-case letter or a digit

 @example
 ```ts
 const isPart = isNameCharacter({ character: 'a', },);
 ```
 */
function isNameCharacter({ character, }: { readonly character: string },): boolean {
  return ((character >= 'a') && (character <= 'z')) || ((character >= '0') && (character <= '9'));
}

/**
 Marks every process a cut answer mentions, whose number this already holds, as one whose status the feed no longer says.

 @param answer - the answer as the feed holds it

 @param reported - what the feed has said about each process so far, by the tool's name for it

 @returns whether the answer mentioned a process this already held

 @example
 ```ts
 const mentioned = markMentionedAsUnknown({ answer: record.result, reported, },);
 ```
 */
function markMentionedAsUnknown({
  answer,
  reported,
}: {
  readonly answer: string;
  readonly reported: Map<string, WorkOfARun>;
},): boolean {
  /**
   Whether the answer mentioned a process this already held.
   */
  let mentioned = false;
  for (const id of processNamesIn({ shown: answer, },)) {
    /**
     What the feed has said about that process so far.
     */
    const before = reported.get(id,);
    if (before === undefined) {
      continue;
    }
    reported.set(
      id,
      {
        ...before,
        statusIsUnknown: true,
      },
    );
    mentioned = true;
  }
  return mentioned;
}

/**
 Says why one read failed, in the words the failure gave.

 @param error - what was caught

 @returns the reason

 @example
 ```ts
 const reason = reasonOfFailure({ error, },);
 ```
 */
function reasonOfFailure({ error, }: { readonly error: unknown },): string {
  if (Error.isError(error,) && (error.message !== '')) {
    return error.message;
  }
  return String(error,);
}

/**
 Reads the processes a run's own `process` tool last reported as running.

 The records are the feed's own: every start, update and list the tool answered is a `tool_end` whose result carries the process
 or the processes it answered with, so what the tool holds is read without asking Pi, whose session may already be over.

 An answer the feed holds only a cut copy of is read as far as it goes, and a process it names is reported as one whose status the
 feed no longer says: the process table answers that, not the tool. An answer nothing can be read from never ends a run as failed.

 @param feedShown - the run's feed, one JSON record per line

 @returns the processes that may still run, and how many answers the feed could say nothing from

 @example
 ```ts
 const reported = workTheToolReportedIn({ feedShown: await readFile(feedFile, 'utf8',), },);
 ```
 */
export function workTheToolReportedIn({ feedShown, }: { readonly feedShown: string },): ReportedWorkOfARun {
  /**
   The last status the tool reported for each process, by the tool's name for it.
   */
  const reported = new Map<string, WorkOfARun>();
  /**
   Every answer the feed holds only a cut copy of, from which no process could be read, and why it could not be parsed.
   */
  const unreadableAnswers: string[] = [];
  for (const line of feedShown.split('\n',)) {
    if (line === '') {
      continue;
    }
    /**
     One record of the feed, of any shape.
     */
    const read = recordIn({ shown: line, },);
    if (read.problem !== undefined) {
      unreadableAnswers.push(`a line of the feed that is no record (${read.problem})`,);
      continue;
    }
    /**
     The record itself.
     */
    const { record, } = read;
    if ((!isRecord(record,)) || (record.kind !== 'tool_end')
      || (record.tool !== 'process')
      || (record.isError === true)) {
      continue;
    }
    if ((typeof record.result) !== 'string') {
      continue;
    }
    /**
     What the record's own copy of the answer holds, and whether that copy was cut.
     */
    const answer = answerIn({ shown: record.result, },);
    /**
     Every process this answer names, one or many, as the answer holds it.
     */
    const named = processesNamedBy({ details: answer.details, },);
    if ((answer.cut) && (named.length === 0)
      && (!markMentionedAsUnknown({
        answer: record.result,
        reported,
      },))) {
      // Nothing of this answer could be read and it mentions no process this knows: work it may have started is work this cannot
      // name, so it counts as work that may still run rather than as no work at all.
      unreadableAnswers.push(
        `a process-tool answer the feed holds only a cut copy of (${answer.problem ?? 'no reason given'})`,
      );
    }
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
      /**
       The status this answer reports for it, which a cut copy may no longer hold.
       */
      const status = ((typeof process.status) === 'string')
        ? process.status
        : (((typeof process.processStatus) === 'string') ? process.processStatus : undefined);
      /**
       What an earlier answer said about it, which a later answer without a status leaves standing.
       */
      const before = reported.get(id,);
      reported.set(
        id,
        {
          id,
          name: ((typeof process.name) === 'string')
            ? process.name
            : (((typeof process.processName) === 'string') ? process.processName : (before?.name ?? id)),
          ...((typeof process.pid) === 'number'
            ? { pid: process.pid, }
            : ((before?.pid === undefined) ? {} : { pid: before.pid, })),
          command: ((typeof process.command) === 'string') ? process.command : (before?.command ?? ''),
          cwd: ((typeof process.cwd) === 'string') ? process.cwd : (before?.cwd ?? ''),
          ...((status === undefined) ? {} : { status, }),
          // A cut copy that named this process and held no status leaves whether it still runs to the process table.
          ...(((answer.cut) && (status === undefined)) ? { statusIsUnknown: true as const, } : {}),
        },
      );
    }
  }
  return {
    running: [...reported.values(),].filter(function stillRunsOrCannotSay(work,) {
      return (work.statusIsUnknown === true)
        || ((work.status !== undefined) && LIVE_STATUSES.has(work.status,));
    },),
    unreadableAnswers,
  };
}

/**
 Reads one line of the feed as a record, or says why it is no record.

 A line the driver cannot read is no record of the tool's, and never a reason to end a run as failed.

 @param shown - the line

 @returns the record, or why the line is no record

 @example
 ```ts
 const read = recordIn({ shown: line, },);
 ```
 */
function recordIn({ shown, }: { readonly shown: string },): {
  readonly record?: unknown;
  readonly problem?: string
} {
  try {
    return { record: JSON.parse(shown,), };
  }
  catch (error) {
    return { problem: reasonOfFailure({ error, },), };
  }
}

/**
 Reads the processes a run's own `process` tool last reported as running.

 The records are the feed's own: every start, update and list the tool answered is a `tool_end` whose result carries the process
 or the processes it answered with, so what the tool holds is read without asking Pi, whose session may already be over.

 An answer the feed holds only a cut copy of is read as far as it goes, and a process it names is reported as one whose status the
 feed no longer says: the process table answers that, not the tool. An answer nothing can be read from never ends a run as failed.

 @param feedShown - the run's feed, one JSON record per line

 @returns the processes that may still run, and how many answers the feed could say nothing from

 @example
 ```ts
 const reported = workTheToolReportedIn({ feedShown: await readFile(feedFile, 'utf8',), },);
 ```
 */
