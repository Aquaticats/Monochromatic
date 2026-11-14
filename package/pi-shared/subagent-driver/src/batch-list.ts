/**
 A batch of Pi runs, started one after another so each waits for a slot that frees as the runs before it end, and the line each
 one ends with.

 The batch is a list of what each run is, never a thing the commands invent: every entry is handed to the retrying start exactly
 as it was written, with the defaults a coordinator's list has always had.

 @module
 */

import {
  isAbsolute,
  join,
} from 'node:path';
import {
  ENTRY_REASON_CHARACTERS,
  reasonInFailedStart,
} from './reason-in-a-failed-start.ts';

/**
 Milliseconds between starting one entry and the next, which is what keeps a batch's starts from all asking for a slot at once.
 */
export const BATCH_STAGGER_MILLISECONDS = 20_000;

/**
 Seconds a batch keeps asking for a slot before it gives up, when the caller names none.
 */
export const BATCH_DEFAULT_DEADLINE_SECONDS = 7_200;

/**
 Seconds a batch run may last, when its entry names none.
 */
export const BATCH_DEFAULT_WALL_SECONDS = 7_200;

/**
 The processor share a batch run's unit gets, when its entry names none.
 */
export const BATCH_DEFAULT_CPU_PERCENT = 800;

/**
 The memory a batch run's unit may hold, when its entry names none.
 */
export const BATCH_DEFAULT_MEMORY = '12G';

/**
 The number of tasks a batch run's unit may hold, when its entry names none.
 */
export const BATCH_DEFAULT_TASK_COUNT = 3_000;

/**
 Seconds the retrying start waits between its asks for a slot.
 */
export const BATCH_RETRY_PERIOD_SECONDS = 30;

/**
 How many kibibytes of what a start printed are kept for its line and for the file the line names.
 */
const KEPT_OUTPUT_KIBIBYTES = 64;

/**
 How many characters one kibibyte of what a start printed is.
 */
const CHARACTERS_PER_KIBIBYTE = 1_024;

/**
 How much of what a start printed is kept for its line and for the file the line names. A Node stack with the command that failed
 and the git refusal it printed fits well inside this; a git refusal alone ran to a kilobyte. The reason is always at the end, so
 a longer log loses only what no reader needs.
 */
export const ENTRY_OUTPUT_KEPT_CHARACTERS: number = KEPT_OUTPUT_KIBIBYTES * CHARACTERS_PER_KIBIBYTE;

/**
 How much of an entry's line is printed. One entry's line names one run and says why it failed, and a report of thirty entries stays
 readable.
 */
export const ENTRY_LINE_CHARACTERS = 300;

/**
 One entry of a batch list.
 */
export type BatchEntry = {
  /**
   The run's name.
   */
  readonly name: string;

  /**
   The branch the run's work goes on.
   */
  readonly branch: string;

  /**
   The commit or branch the run's new branch starts at; `main` when absent. An existing branch of the same name keeps its own
   commits, so this is not used then.
   */
  readonly base?: string;

  /**
   The first prompt, as a path or as a name under the runs folder; the run's own prompt file when absent.
   */
  readonly prompt?: string;

  /**
   The processor share the run's unit gets.
   */
  readonly cpu?: number;

  /**
   The memory the run's unit may hold.
   */
  readonly memory?: string;

  /**
   The number of tasks the run's unit may hold.
   */
  readonly tasks?: number;

  /**
   How long the run may last, in seconds.
   */
  readonly wall?: number;

  /**
   The number of the run's folder, or nothing for the next free number.
   */
  readonly run?: number;

  /**
   The finished run whose session this run resumes.
   */
  readonly sessionOf?: string;

  /**
   A session file to resume directly, for a run that ended with no summary to read a session from.
   */
  readonly session?: string;

  /**
   The provider the run asks for.
   */
  readonly provider?: string;

  /**
   The model the run asks for.
   */
  readonly model?: string;
};

/**
 The fields one entry of a batch list may name, which are the launcher's own options. A field outside this list is refused, because
 dropping it would start a run other than the one the list describes.
 */
export const BATCH_ENTRY_FIELDS: readonly string[] = [
  'name',
  'branch',
  'base',
  'prompt',
  'cpu',
  'memory',
  'tasks',
  'wall',
  'run',
  'sessionOf',
  'session',
  'provider',
  'model',
];

/**
 A batch list this command cannot read as the list it was given.
 */
export class BatchListError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'BatchListError';
}

/**
 What starting one entry answered.
 */
export type StartedEntry = {
  /**
   The status the retrying start ended with, or nothing when it could not be started at all.
   */
  readonly status?: number;

  /**
   What it printed on both streams, of which the end is what a reader needs.
   */
  readonly output: string;

  /**
   Why it could not be started, when it could not be.
   */
  readonly startProblem?: string;
};

/**
 Reads a batch list.

 @param shown - the list as it was written

 @param listFile - the file the list came from, for the message

 @returns the entries

 @throws Error when the list is not a JSON array, because a batch that starts nothing is a defect worth seeing

 @example
 ```ts
 const entries = batchEntriesIn({ shown: '[{"name":"a","branch":"feat/a"}]', listFile: 'batch.json', },);
 ```
 */
export function batchEntriesIn({
  shown,
  listFile,
}: {
  readonly shown: string;
  readonly listFile: string;
}): readonly BatchEntry[] {
  /**
   What the list holds, of any shape.
   */
  const listed: unknown = JSON.parse(shown,);
  if (!Array.isArray(listed,)) {
    throw new Error(`${listFile} holds no JSON array of entries, so nothing could be started from it.`,);
  }
  listed.forEach(function checkEntry(
    entry: unknown,
    index: number,
  ) {
    if (((typeof entry) !== 'object') || (entry === null)) {
      throw new BatchListError(`${listFile}: entry ${String(index + 1,)} is no JSON object, so it cannot be a run.`,);
    }
    /**
     The fields this entry names that the command does not read.
     */
    const unread = Object.keys(entry,)
      .filter(function isUnread(field,) {
        return !BATCH_ENTRY_FIELDS.includes(field,);
      },);
    if (unread.length === 0) {
      return;
    }
    /**
     The run the entry names, which tells a reader which entry to fix.
     */
    const named = ('name' in entry) ? entry.name : undefined;
    throw new BatchListError(
      `${listFile}: entry ${String(index + 1,)}${((typeof named) === 'string') ? ` (${named})` : ''} names a field this command does not read: ${unread.join(', ')}. It reads ${BATCH_ENTRY_FIELDS.join(', ')}, so a field that would be dropped is refused rather than ignored.`,
    );
  },);
  return listed as readonly BatchEntry[];
}

/**
 Reads the prompt file one entry names, which is its own file under the runs folder unless it is a path.

 @param entry - the entry

 @param runsFolder - folder the runs live in

 @returns the prompt file's path

 @example
 ```ts
 const prompt = promptFileOf({ entry: { name: 'a', branch: 'feat/a', }, runsFolder: '/runs', },);
 ```
 */
export function promptFileOf({
  entry,
  runsFolder,
}: {
  readonly entry: BatchEntry;
  readonly runsFolder: string;
}): string {
  /**
   The prompt the entry names, or the one its name gives.
   */
  const named = entry.prompt ?? `${entry.name}-prompt.md`;
  return isAbsolute(named,) ? named : join(
    runsFolder,
    named,
  );
}

/**
 Reads the words one entry is started with: the retrying start's own options, then the pool gate's.

 @param entry - the entry

 @param runsFolder - folder the runs live in

 @param deadlineSeconds - how long the retrying start keeps asking for a slot

 @returns the words, each kept whole

 @example
 ```ts
 const words = retryWordsOf({ entry: { name: 'a', branch: 'feat/a', }, runsFolder: '/runs', deadlineSeconds: 7_200, },);
 ```
 */
export function retryWordsOf({
  entry,
  runsFolder,
  deadlineSeconds,
}: {
  readonly entry: BatchEntry;
  readonly runsFolder: string;
  readonly deadlineSeconds: number;
}): readonly string[] {
  return [
    `--deadline-seconds=${String(deadlineSeconds,)}`,
    `--period-seconds=${String(BATCH_RETRY_PERIOD_SECONDS,)}`,
    '--',
    `--runs-folder=${runsFolder}`,
    `--name=${entry.name}`,
    `--branch=${entry.branch}`,
    `--prompt=${promptFileOf({
      entry,
      runsFolder,
    },)}`,
    `--cpu=${String(entry.cpu ?? BATCH_DEFAULT_CPU_PERCENT,)}`,
    `--memory=${entry.memory ?? BATCH_DEFAULT_MEMORY}`,
    `--tasks=${String(entry.tasks ?? BATCH_DEFAULT_TASK_COUNT,)}`,
    `--wall=${String(entry.wall ?? BATCH_DEFAULT_WALL_SECONDS,)}`,
    ...((entry.run === undefined) ? [] : [`--run=${String(entry.run,)}`,]),
    ...((entry.base === undefined) ? [] : [`--base=${entry.base}`,]),
    ...((entry.provider === undefined) ? [] : [`--provider=${entry.provider}`,]),
    ...((entry.model === undefined) ? [] : [`--model=${entry.model}`,]),
    ...((entry.sessionOf === undefined) ? [] : [`--session-of=${entry.sessionOf}`,]),
    ...((entry.session === undefined) ? [] : [`--session=${entry.session}`,]),
  ];
}

/**
 Reads what one line of a failed entry can say about why it failed: the reason it printed, cut to the room the line has left, or
 the fact that it printed none.

 @param reason - what the start printed as its reason, or nothing when it printed none

 @param characters - how much of the reason the line has room for

 @returns the words the line carries

 @example
 ```ts
 const said = reasonCutTo({ reason: 'Error: no prompt', characters: 200 });
 ```
 */
function reasonCutTo({
  reason,
  characters,
}: {
  readonly reason?: string;
  readonly characters: number;
}): string {
  if (reason === undefined) {
    return 'it printed no reason';
  }
  return reason.slice(
    0,
    characters,
  );
}

/**
 Reads the line one entry ends with: the reason it failed by, never the last line of a stack, and where the whole of what it
 printed was kept.

 @param name - the run's name

 @param started - what starting it answered

 @param keptAt - where the whole of what a failed start printed was kept, when it was kept

 @returns the line to print

 @example
 ```ts
 const line = entryLineOf({ name: 'a', started: { status: 0, output: '', }, });
 ```
 */
export function entryLineOf({
  name,
  started,
  keptAt,
}: {
  readonly name: string;
  readonly started: StartedEntry;
  readonly keptAt?: string;
},): string {
  if ((started.startProblem !== undefined) || (started.status === undefined)) {
    return `${name}: could not start the retry (${started.startProblem ?? 'it answered no status'})`;
  }
  /**
   What the line says before the reason, which is what the reason has to fit beside.
   */
  const head = `${name}: status ${String(started.status,)}`;
  if (started.status === 0) {
    return head;
  }
  /**
   Where the whole of what it printed is kept, for a reader who needs more than the reason.
   */
  const kept = (keptAt === undefined) ? '' : ` (all it printed: ${keptAt})`;
  /**
   How much of the line the run's name and the kept file already take.
   */
  const saidSoFar = head.length + kept.length;
  /**
   How much of the reason is left once they have their room.
   */
  const roomForReason = Math.max(
    ENTRY_LINE_CHARACTERS
      - saidSoFar
      - 1,
    ENTRY_REASON_CHARACTERS,
  );
  /**
   Why the start failed, which is what a reader scans for.
   */
  const reason = reasonCutTo({
    ...reasonInFailedStart({ output: started.output, },),
    characters: roomForReason,
  },);
  return `${head} ${reason}${kept}`;
}

/**
 What starting a batch answered: the line each entry ended with, in the list's order, and the entries that did not start.

 The entries that failed travel beside the lines because the command that started the batch has to answer with a status, and a
 batch that printed a failure and exited 0 told its caller nothing: a unit that ran one, or a coordinator reading the unit's
 status, could not tell a batch of ten runs that all started from one where every entry was refused.
 */
export type StartedBatch = {
  /**
   One line per entry, in the list's order.
   */
  readonly lines: readonly string[];

  /**
   The names of the entries whose start did not end with status 0, including one that never started at all.
   */
  readonly failed: readonly string[];
};

/**
 Starts every entry of a batch, one after another, and answers with the line each one ended with.

 @param entries - the runs to start

 @param runsFolder - folder the runs live in

 @param deadlineSeconds - how long each retrying start keeps asking for a slot

 @param start - one entry's start, which a test replaces

 @param wait - how the batch passes time between starts

 @param keepFailure - where the whole of what a failed start printed is kept, or nothing when it is not kept anywhere

 @returns one line per entry in the list's order, and the entries that did not start

 @example
 ```ts
 const { lines, failed } = await startBatch({ entries, runsFolder: '/runs', deadlineSeconds: 7_200, start, wait, keepFailure, },);
 ```
 */
export async function startBatch({
  entries,
  runsFolder,
  deadlineSeconds,
  start,
  wait,
  keepFailure,
}: {
  readonly entries: readonly BatchEntry[];
  readonly runsFolder: string;
  readonly deadlineSeconds: number;
  readonly start: (request: {
    readonly name: string;
    readonly words: readonly string[];
  }) => Promise<StartedEntry>;
  readonly wait: (milliseconds: number) => Promise<void>;
  readonly keepFailure?: (request: {
    readonly name: string;
    readonly output: string;
  }) => Promise<{ readonly keptAt?: string }>;
},): Promise<StartedBatch> {
  /**
   What each entry ended with, in the list's order.
   */
  const started: Promise<StartedEntry>[] = [];
  /* oxlint-disable eslint/no-await-in-loop -- one entry after another, because the stagger is what keeps the batch's slots in the
     order its list gives them. See doc/lint/sequential-awaits.md. */
  for (const entry of entries) {
    started.push(start({
      name: entry.name,
      words: retryWordsOf({
        entry,
        runsFolder,
        deadlineSeconds,
      },),
    },),);
    await wait(BATCH_STAGGER_MILLISECONDS,);
  }
  /* oxlint-enable eslint/no-await-in-loop */
  /**
   What each entry answered, in the list's order.
   */
  const answered = await Promise.all(started,);
  /**
   The lines it ended with, in the same order.
   */
  const lines = await Promise.all(answered.map(async function lineFor(
    startedEntry,
    index,
  ) {
    /**
     The run this line is about.
     */
    const name = entries[index]
      ?.name
      ?? '';
    /**
     Where the whole of a failed start's output was kept, which only a failure that printed something has.
     */
    const keptAt = (((startedEntry.status === 0) || (startedEntry.status === undefined)) || (keepFailure === undefined))
      ? undefined
      : (await keepFailure({
        name,
        output: startedEntry.output,
      },)).keptAt;
    return entryLineOf({
      name,
      started: startedEntry,
      ...(keptAt === undefined ? {} : { keptAt, }),
    },);
  },),);
  /**
   The runs that did not start, which are why the batch answers with a failing status.
   */
  const failed = answered.flatMap(function nameOfFailed(
    startedEntry,
    index,
  ) {
    /**
     The name the list gave this entry, which the line printed beside it already carries.
     */
    const name = entries[index]
      ?.name
      ?? '';
    return (startedEntry.status === 0) ? [] : [name,];
  },);
  return {
    lines,
    failed,
  };
}
