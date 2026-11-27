/**
 What one add writes about itself, read from the timing line the wrapper appends.

 The line is written by the wrapper (`package/git-policy/cli/src/native/worktree/add_timing_record.rs`,
 `Recorder::line`), one line per add, and every measurement script of 2026-11-21 reads it to say where an add's time went.
 This module turns one such line into numbers, so a measurement can be judged without a repository: a line of a shape the
 wrapper never writes stops the script that read it instead of becoming a silently missing figure.
 */

/**
 The word the line's fields are named by, written beside each guard so a malformed line names the field it failed on.
 */
const millisecondsField = 'ms';

/**
 The word a step's call count is named by.
 */
const callsField = 'calls';

/**
 What one step or one piece of worker-thread work spent.
 */
export type AddTimingSpent = {
  /**
   The time itself, in milliseconds, with the steps entered inside it taken out.
   */
  readonly ms: number;
  /**
   How many times the name was entered.
   */
  readonly calls: number;
};

/**
 One add's own account of itself, as the wrapper wrote it.
 */
export type AddTimingRecord = {
  /**
   The add's whole time in milliseconds.
   */
  readonly totalMs: number;
  /**
   The status the add ended with.
   */
  readonly exitCode: number;
  /**
   The steps of the add by name, each with its own time.
   */
  readonly steps: Readonly<Record<string, AddTimingSpent>>;
  /**
   The work of other threads by name, which overlaps and is no part of the steps' sum.
   */
  readonly work: Readonly<Record<string, AddTimingSpent>>;
  /**
   What the add noted about itself that is a word, such as the route it took.
   */
  readonly notes: Readonly<Record<string, string>>;
};

/**
 Whether a parsed JSON value is an object whose fields can be read by name.

 The value is taken as one argument rather than a destructured object because a type predicate cannot name a binding
 pattern's element, and the narrowing is the whole point of this guard.

 @param value - the parsed value to judge

 @returns true for an object that is not an array and not null
 */
function isRecord(value: unknown): value is Readonly<Record<string, unknown>> {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value,));
}

/**
 The number one field of the line holds.

 @param fields - the parsed object the field belongs to

 @param name - the field's name

 @param line - the whole line, so a refusal says which line it could not read

 @returns the field's value

 @throws Error when the field is missing or is not a number
 */
function numberField({
  fields,
  name,
  line,
}: {
  readonly fields: Readonly<Record<string, unknown>>;
  readonly name: string;
  readonly line: string;
}): number {
  /**
   What the field holds.
   */
  const value = fields[name];
  if ((typeof value) !== 'number') throw new Error(`the add's timing line holds no number for ${name}: ${line}`);
  return value;
}

/**
 The steps or the worker-thread work of one line, by name.

 @param value - the parsed field, which the wrapper leaves out when there is nothing to report

 @param line - the whole line, so a refusal says which line it could not read

 @returns every entry by name, empty when the field is absent

 @throws Error when the field or one of its entries is neither absent nor an object of numbers
 */
function spentOf({
  value,
  line,
}: {
  readonly value: unknown;
  readonly line: string;
}): Readonly<Record<string, AddTimingSpent>> {
  if (value === undefined) return {};
  if (!isRecord(value)) throw new Error(`the add's timing line holds no object for a list of steps: ${line}`);
  /**
   What each name spent, in the order the line lists them.
   */
  const spent: Record<string, AddTimingSpent> = {};
  for (const [name, entry,] of Object.entries(value,)) {
    if (!isRecord(entry)) throw new Error(`the add's timing line holds no object for ${name}: ${line}`);
    /**
     One entry's own fields.
     */
    const entryFields = entry;
    spent[name] = {
      ms: numberField({
        fields: entryFields,
        name: millisecondsField,
        line,
      },),
      calls: numberField({
        fields: entryFields,
        name: callsField,
        line,
      },),
    };
  }
  return spent;
}

/**
 The notes of one line, by name.

 @param value - the parsed field, which the wrapper leaves out when it noted nothing

 @param line - the whole line, so a refusal says which line it could not read

 @returns every note by name, empty when the field is absent

 @throws Error when a note is not a word
 */
function notesOf({
  value,
  line,
}: {
  readonly value: unknown;
  readonly line: string;
}): Readonly<Record<string, string>> {
  if (value === undefined) return {};
  if (!isRecord(value)) throw new Error(`the add's timing line holds no object for its notes: ${line}`);
  /**
   What the add noted, in the order the line lists it.
   */
  const notes: Record<string, string> = {};
  for (const [name, note,] of Object.entries(value,)) {
    if ((typeof note) !== 'string') throw new Error(`the add noted no word for ${name}: ${line}`);
    notes[name] = note;
  }
  return notes;
}

/**
 Reads one timing line into the add's own account of itself.

 @param line - one line of `monochromatic-git-policy-cli-worktree-add-timing.jsonl`

 @returns the add's total time, status, steps, worker-thread work and notes

 @throws Error when the line is not the object of fields the wrapper writes

 @example
 ```ts
 addTimingRecordOf({ line: '{"totalMs":412,"exitCode":0,"steps":{},"notes":{"route":"clone"}}' });
 ```
 */
export function addTimingRecordOf({ line, }: { readonly line: string; }): AddTimingRecord {
  /**
   The line as a JSON value, which every field below is read out of.
   */
  const parsed: unknown = JSON.parse(line,);
  if (!isRecord(parsed)) throw new Error(`the add's timing line is not an object: ${line}`);
  return {
    totalMs: numberField({
      fields: parsed,
      name: 'totalMs',
      line,
    },),
    exitCode: numberField({
      fields: parsed,
      name: 'exitCode',
      line,
    },),
    steps: spentOf({
      value: parsed.steps,
      line,
    },),
    work: spentOf({
      value: parsed.work,
      line,
    },),
    notes: notesOf({
      value: parsed.notes,
      line,
    },),
  };
}

/**
 Reads the last add's account from the whole of a timing file.

 The wrapper appends one line per add, so the last line is the add that just ended.

 @param text - the whole file

 @returns the last add's account

 @throws Error when the file holds no line, or when its last line is not one the wrapper wrote

 @example
 ```ts
 lastAddTimingRecordOf({ text: await readFile(record, 'utf8') });
 ```
 */
export function lastAddTimingRecordOf({ text, }: { readonly text: string; }): AddTimingRecord {
  /**
   The file's lines that hold something, in file order.
   */
  const lines = text
    .split('\n',)
    .filter(function holdsSomething(line) {
      return line.trim() !== '';
    },);
  /**
   The last add's line, which is the one that just ended.
   */
  const line = lines.at(-1,);
  if (line === undefined) throw new Error('the timing file holds no line');
  return addTimingRecordOf({ line, },);
}
