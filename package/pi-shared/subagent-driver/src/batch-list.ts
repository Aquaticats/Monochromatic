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
 How much of what the retrying start printed is kept for the entry's line. A run that failed prints why at the end, and a
 status 1 with no reason left the round 164 failures of 2026-11-11 unexplained.
 */
export const ENTRY_OUTPUT_TAIL_CHARACTERS = 400;

/**
 How much of an entry's line is printed. One entry's line names one run, and a report of thirty entries stays readable.
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
   The provider the run asks for.
   */
  readonly provider?: string;

  /**
   The model the run asks for.
   */
  readonly model?: string;
};

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
  ];
}

/**
 Reads the line one entry ends with.

 @param name - the run's name

 @param started - what starting it answered

 @returns the line to print

 @example
 ```ts
 const line = entryLineOf({ name: 'a', started: { status: 0, output: '', }, });
 ```
 */
export function entryLineOf({
  name,
  started,
}: {
  readonly name: string;
  readonly started: StartedEntry;
}): string {
  if ((started.startProblem !== undefined) || (started.status === undefined)) {
    return `${name}: could not start the retry (${started.startProblem ?? 'it answered no status'})`;
  }
  /**
   What the entry printed, of which the end is what says why it ended so.
   */
  const tail = started.output
    .trim()
    .slice(-ENTRY_OUTPUT_TAIL_CHARACTERS,);
  return `${name}: status ${String(started.status,)}${(started.status === 0) ? '' : ` ${tail.split('\n',)
    .at(-1)
    ?? ''}`}`
    .slice(
    0,
    ENTRY_LINE_CHARACTERS,
    );
}

/**
 Starts every entry of a batch, one after another, and answers with the line each one ended with.

 @param entries - the runs to start

 @param runsFolder - folder the runs live in

 @param deadlineSeconds - how long each retrying start keeps asking for a slot

 @param start - one entry's start, which a test replaces

 @param wait - how the batch passes time between starts

 @returns one line per entry, in the list's order

 @example
 ```ts
 const lines = await startBatch({ entries, runsFolder: '/runs', deadlineSeconds: 7_200, start, wait, },);
 ```
 */
export async function startBatch({
  entries,
  runsFolder,
  deadlineSeconds,
  start,
  wait,
}: {
  readonly entries: readonly BatchEntry[];
  readonly runsFolder: string;
  readonly deadlineSeconds: number;
  readonly start: (request: {
    readonly name: string;
    readonly words: readonly string[];
  }) => Promise<StartedEntry>;
  readonly wait: (milliseconds: number) => Promise<void>;
},): Promise<readonly string[]> {
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
  return (await Promise.all(started,)).map(function lineFor(
    startedEntry,
    index,
  ) {
    return entryLineOf({
      name: entries[index]
        ?.name
        ?? '',
      started: startedEntry,
    },);
  },);
}
