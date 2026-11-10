/**
 A batch of runs: a list of what to start, and one start per entry, each of which waits for its own slot.

 The list is what a coordinator writes before a round of work, and the entries are started a little apart so the queue formed is
 the list's order. An entry may name the attempt and the run whose session it resumes, which is what makes a batch resumable
 after a deadline.

 @module
 */

/**
 An entry of a batch list.
 */
export type BatchEntry = {
  /**
   The run's name.
   */
  readonly name: string;

  /**
   The branch the run works on, or `none` for the repository's own checkout.
   */
  readonly branch: string;

  /**
   The file holding the first prompt; absent for `<runs folder>/<name>-prompt.md`.
   */
  readonly prompt?: string;

  /**
   The ceiling the unit is given; absent for the default below.
   */
  readonly cpu?: number;
  readonly memory?: string;
  readonly tasks?: number;

  /**
   How long the run may last before its pool multiplies it.
   */
  readonly wall?: number;

  /**
   The number of the attempt, which names its run folder; absent for the first.
   */
  readonly run?: number;

  /**
   The run folder of an attempt whose session this run resumes.
   */
  readonly sessionOf?: string;
};

/**
 What one entry's start ended with.
 */
export type StartedEntry = {
  /**
   The run's name, so a line about it needs nothing else.
   */
  readonly name: string;

  /**
   The status its queue exited with.
   */
  readonly status: number;

  /**
   The last line it printed, which says what happened when the status is not zero.
   */
  readonly lastLine: string;
};

/**
 Starts one entry, which is one queue waiting for a slot.
 */
export type StartAnEntry = (entry: BatchEntry) => Promise<StartedEntry>;

/**
 What an entry that names no ceiling is given of the machine's cores, in percent: the ceiling the coordinator's own batches used
 for a coding run.
 */
export const DEFAULT_BATCH_CPU_PERCENT = 800;

/**
 What an entry that names no memory limit is given.
 */
export const DEFAULT_BATCH_MEMORY = '12G';

/**
 How many tasks an entry that names no limit may start.
 */
export const DEFAULT_BATCH_TASKS = 3_000;

/**
 How long an entry that names no wall clock may last.
 */
export const DEFAULT_BATCH_WALL_SECONDS = 7_200;

/**
 How long an entry's queue waits between two attempts at its start.
 */
export const BATCH_PERIOD_SECONDS = 30;

/**
 How far apart two entries are started: far enough that the earlier entry's unit and run folder exist before the next asks for a
 slot, and short enough that a list of a dozen is queued in a couple of minutes.
 */
export const BATCH_STAGGER_MILLISECONDS = 20_000;

/**
 How much of a line an entry's last printed line is cut to.
 */
const LINE_CHARACTERS = 300;

/**
 A batch list that cannot be read as what it claims to be.
 */
export class BatchListError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'BatchListError';
}

/**
 Reads one field of an entry that must be text.

 @param value - the field as the list holds it

 @param where - which entry and which field, for the message

 @returns the text

 @throws BatchListError when the field is not text with something in it

 @example
 ```ts
 const name = textFieldOf({ value: 'a', where: 'entry 1 --name', });
 ```
 */
function textFieldOf({
  value,
  where,
}: {
  readonly value: unknown;
  readonly where: string;
},): string {
  if (((typeof value) !== 'string') || (value === '')) {
    throw new BatchListError(`${where} must be text with something in it`,);
  }
  return value;
}

/**
 A field the list did not carry, as a symbol so no value it does carry can be mistaken for it.
 */
const NOT_GIVEN: unique symbol = Symbol('the list did not carry this field',);

/**
 Reads one field of an entry that may be absent.

 @param value - the field as the list holds it

 @param where - which entry and which field, for the message

 @returns the number when the field holds one, or nothing when it is absent

 @throws BatchListError when the field holds anything but a number above zero

 @example
 ```ts
 const cpu = optionalNumberFieldOf({ value: 800, where: 'entry 1 --cpu', });
 ```
 */
function optionalNumberFieldOf({
  value,
  where,
}: {
  readonly value: unknown;
  readonly where: string;
},): number | typeof NOT_GIVEN {
  if (value === undefined) {
    return NOT_GIVEN;
  }
  if (((typeof value) !== 'number') || (!Number.isFinite(value,))
    || (value <= 0)) {
    throw new BatchListError(`${where} must be a number above zero`,);
  }
  return value;
}

/**
 Reads one field of an entry that may be absent, as text.

 @param value - the field as the list holds it

 @param where - which entry and which field, for the message

 @returns the text when the field holds one, or nothing when it is absent

 @example
 ```ts
 const prompt = optionalTextFieldOf({ value: 'a-prompt.md', where: 'entry 1 --prompt', });
 ```
 */
function optionalTextFieldOf({
  value,
  where,
}: {
  readonly value: unknown;
  readonly where: string;
},): string | typeof NOT_GIVEN {
  return (value === undefined) ? NOT_GIVEN : textFieldOf({
    value,
    where,
  },);
}

/**
 Reads one entry of a batch list.

 @param value - the entry as the list holds it

 @param at - the entry's place in the list, counting from one

 @returns the entry

 @throws BatchListError when the entry is not an object, or a field it must carry is missing or unusable

 @example
 ```ts
 const entry = entryOf({ value: { name: 'a', branch: 'b', }, at: 1, });
 ```
 */
function entryOf({
  value,
  at,
}: {
  readonly value: unknown;
  readonly at: number;
},): BatchEntry {
  if (((typeof value) !== 'object') || (value === null)) {
    throw new BatchListError(`entry ${String(at,)} must be an object`,);
  }
  /**
   The entry's fields, by name.
   */
  const fields = value as Readonly<Record<string, unknown>>;
  /**
   The entry's place, for a message about one of its fields.
   */
  const where = `entry ${String(at,)}`;
  /**
   The fields whose presence is a decision, each kept only when the list carried it.
   */
  /**
   How much of the machine's cores the entry's unit may use, when it names that.
   */
  const cpu = optionalNumberFieldOf({
    value: fields.cpu,
    where: `${where} --cpu`,
  },);
  /**
   How many tasks the entry's unit may start, when it names that.
   */
  const tasks = optionalNumberFieldOf({
    value: fields.tasks,
    where: `${where} --tasks`,
  },);
  /**
   How long the entry's run may last, when it names that.
   */
  const wall = optionalNumberFieldOf({
    value: fields.wall,
    where: `${where} --wall`,
  },);
  /**
   The number of the attempt, when the entry names one.
   */
  const run = optionalNumberFieldOf({
    value: fields.run,
    where: `${where} --run`,
  },);
  /**
   The file holding the entry's first prompt, when it names one.
   */
  const prompt = optionalTextFieldOf({
    value: fields.prompt,
    where: `${where} --prompt`,
  },);
  /**
   The memory limit of the entry's unit, when the entry names one.
   */
  const memory = optionalTextFieldOf({
    value: fields.memory,
    where: `${where} --memory`,
  },);
  /**
   The run folder of the attempt whose session this entry continues, when it names one.
   */
  const sessionOf = optionalTextFieldOf({
    value: fields.sessionOf,
    where: `${where} --session-of`,
  },);
  return {
    name: textFieldOf({
      value: fields.name,
      where: `${where} --name`,
    },),
    branch: textFieldOf({
      value: fields.branch,
      where: `${where} --branch`,
    },),
    ...((prompt === NOT_GIVEN) ? {} : { prompt, }),
    ...((cpu === NOT_GIVEN) ? {} : { cpu, }),
    ...((memory === NOT_GIVEN) ? {} : { memory, }),
    ...((tasks === NOT_GIVEN) ? {} : { tasks, }),
    ...((wall === NOT_GIVEN) ? {} : { wall, }),
    ...((run === NOT_GIVEN) ? {} : { run, }),
    ...((sessionOf === NOT_GIVEN) ? {} : { sessionOf, }),
  };
}

/**
 Reads the list a batch file holds, or says why it is no JSON.

 @param text - the list as the file holds it

 @returns what the list holds

 @throws BatchListError when the text is not JSON

 @example
 ```ts
 const list = parsedListOf({ text: '[]', });
 ```
 */
function parsedListOf({
  text,
}: {
  readonly text: string;
},): unknown {
  try {
    return JSON.parse(text,);
  }
  catch (error) {
    throw new BatchListError(
      `a batch list must be JSON: ${String(error,)}`,
      { cause: error, },
    );
  }
}

/**
 Reads a batch list.

 @param text - the list as the file holds it

 @returns the entries, in the order the list gives them

 @throws BatchListError when the text is no list of entries

 @example
 ```ts
 const entries = parseBatchList({ text: '[{"name":"a","branch":"b"}]', });
 ```
 */
export function parseBatchList({
  text,
}: {
  readonly text: string;
},): readonly BatchEntry[] {
  /**
   What the list holds.
   */
  const list = parsedListOf({ text, },);
  if (!Array.isArray(list,)) {
    throw new BatchListError('a batch list must be an array of entries',);
  }
  return list.map(function entryAt(
    value,
    index,
  ) {
    return entryOf({
      value,
      at: index + 1,
    },);
  },);
}

/**
 Starts every entry of a batch, a little apart, and answers with one line per entry.

 @param entries - the entries to start

 @param startEntry - one entry's queue

 @param wait - how time passes between two entries, which a test replaces

 @param staggerMilliseconds - how far apart two entries are started

 @returns one line per entry, in the list's order

 @example
 ```ts
 const lines = await startABatch({ entries, startEntry, wait, staggerMilliseconds: 0, });
 ```
 */
export async function startABatch({
  entries,
  startEntry,
  wait,
  staggerMilliseconds,
}: {
  readonly entries: readonly BatchEntry[];
  readonly startEntry: StartAnEntry;
  readonly wait: (milliseconds: number) => Promise<void>;
  readonly staggerMilliseconds: number;
},): Promise<readonly string[]> {
  /**
   One promise per entry, each settling with what its queue answered.
   */
  const started: Promise<StartedEntry>[] = [];
  for (const entry of entries) {
    started.push(startEntry(entry,),);
    // oxlint-disable-next-line eslint/no-await-in-loop -- the entries are started a little apart, so the queue's order is the list's; see doc/lint/sequential-awaits.md
    await wait(staggerMilliseconds,);
  }
  return (await Promise.all(started,)).map(function asLine(entry,) {
    return `${entry.name}: status ${String(entry.status,)}${(entry.status === 0) ? '' : ` ${entry.lastLine}`}`
      .slice(
      0,
      LINE_CHARACTERS,
      );
  },);
}
