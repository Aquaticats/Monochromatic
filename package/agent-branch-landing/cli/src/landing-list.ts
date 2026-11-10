/**
 A list file: the branches one run lands, each with the message that names what it carries and the check it gets first.

 @module
 */

import { readFile, } from 'node:fs/promises';
import type { CheckKind, } from './check-a-branch.ts';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { LandingConfigurationError, } from './errors.ts';
import {
  found,
  type Looked,
  missing,
} from './lookups.ts';

/**
 One branch to land.
 */
export type LandingEntry = {
  /**
   The branch to land, or empty text when the worktree's own branch is the one to land.
   */
  readonly branch: string;

  /**
   The worktree folder's name under `.claude/worktrees`, or empty text for a branch named outright.
   */
  readonly worktree: string;

  /**
   What checks it on the merged tree before it lands.
   */
  readonly check: CheckKind;

  /**
   Tasks the entry names, which a `package` check runs beside the cheapest declared ones and a `tasks` check runs alone.
   */
  readonly tasks: readonly string[];

  /**
   The message its merge commit carries.
   */
  readonly message: string;

  /**
   Why this check is enough, printed with the entry so the reason lands in the run's log. Owner, 2026-11-10: "I don't think
   you'd need to rerun the same lints or tests another agent did, do you?"
   */
  readonly note: string;

  /**
   A name `--only` may select this entry by, or empty text when it has none.
   */
  readonly key: string;
};

/**
 The checks a list entry may ask for, as the file spells them.
 */
const CHECK_KINDS: readonly CheckKind[] = [
  'wrapper',
  'package',
  'tasks',
  'none',
];

/**
 Reads one field a record may hold under any of several names, which is how the list files the scratch rounds wrote keep
 working: they named a worktree `agent` or `name`, and an extra task list `consumerTasks`.

 @param record - the entry as the file holds it

 @param names - the field names, the current one first

 @returns the value, or nothing when the record holds none of them

 @example
 ```ts
 const asked = fieldOf({ record, names: ['worktree', 'agent', 'name',], },);
 ```
 */
function fieldOf({
  record,
  names,
}: {
  readonly record: Readonly<Record<string, unknown>>;
  readonly names: readonly string[];
},): unknown {
  for (const name of names) {
    if (Object.hasOwn(
      record,
      name,
    )) return record[name];
  }
  return undefined;
}

/**
 Reads a field that must be text, refusing anything else.

 @param value - the field's value

 @param field - the field's name

 @param where - the file it was read from

 @returns the text, empty when the field is absent

 @throws LandingConfigurationError when the field holds something that is not text

 @example
 ```ts
 const message = textOf({ value: 'merge(a): b', field: 'message', where, },);
 ```
 */
function textOf({
  value,
  field,
  where,
}: {
  readonly value: unknown;
  readonly field: string;
  readonly where: string;
},): string {
  if (value === undefined) return '';
  if ((typeof value) !== 'string') {
    throw new LandingConfigurationError(`${where}: ${field} must be text`,);
  }
  return value;
}

/**
 Reads a list of task names, refusing anything else.

 @param value - the field's value

 @param field - the field's name

 @param where - the file it was read from

 @returns the task names, empty when the field is absent

 @throws LandingConfigurationError when the field is not a list of repository-qualified task names

 @example
 ```ts
 const tasks = tasksOf({ value: ['//package/a/b:lint',], field: 'tasks', where, },);
 ```
 */
function tasksOf({
  value,
  field,
  where,
}: {
  readonly value: unknown;
  readonly field: string;
  readonly where: string;
},): readonly string[] {
  if (value === undefined) return [];
  if ((!(Array.isArray(value,))) || (!value.every(function isTaskName(task: unknown,) {
    return ((typeof task) === 'string') && task.startsWith('//',);
  },))) {
    throw new LandingConfigurationError(`${where}: ${field} must be a list of task names such as //package/a/b:lint`,);
  }
  return value as readonly string[];
}

/**
 Reads the check an entry asks for, deriving it from the entry's own fields when the file does not state one.

 @param record - the entry as the file holds it

 @param where - the file it was read from

 @returns the check to run

 @throws LandingConfigurationError when the file names a check that is not one of the four

 @example
 ```ts
 const check = checkOf({ record, where, },);
 ```
 */
function checkOf({
  record,
  where,
}: {
  readonly record: Readonly<Record<string, unknown>>;
  readonly where: string;
},): CheckKind {
  /**
   The check the file states, when it states one.
   */
  const stated = textOf({
    value: record.check,
    field: 'check',
    where,
  },);
  if (stated !== '') {
    /**
     The check the file names, when it names one of the four.
     */
    const named = CHECK_KINDS.find(function isNamed(kind: CheckKind,) {
      return kind === stated;
    },);
    if (named === undefined) {
      throw new LandingConfigurationError(`${where}: check must be one of ${CHECK_KINDS.join(', ')}, not ${stated}`,);
    }
    return named;
  }
  // A file written before this command existed states none, and its own fields say which check it meant: `consumerTasks` was
  // always an addition to the cheapest declared task, `tasks` was always the whole set, `agent` always named a wrapper branch,
  // and `name` always named a package branch.
  if (Object.hasOwn(
    record,
    'consumerTasks',
  )) return 'package';
  if (Object.hasOwn(
    record,
    'tasks',
  )) return 'tasks';
  if (Object.hasOwn(
    record,
    'agent',
  )) return 'wrapper';
  if (Object.hasOwn(
    record,
    'name',
  ) || Object.hasOwn(
    record,
    'worktree',
  )) return 'package';
  return 'none';
}

/**
 Tells whether a value read from JSON is an object with named fields.

 @param value - what the file held

 @returns true when it is an object

 @example
 ```ts
 if (isRecord(entry,)) return;
 ```
 */
function isRecord(value: unknown,): value is Readonly<Record<string, unknown>> {
  return ((typeof value) === 'object') && (value !== null);
}

/**
 Reads a list file's text as JSON.

 @param text - the file's text

 @returns what the text holds, or why it is not JSON

 @example
 ```ts
 const parsed = parsedText({ text: '[]', },);
 ```
 */
function parsedText({ text, }: {
  readonly text: string;
},): Looked<unknown> {
  try {
    return found(JSON.parse(text,),);
  }
  catch (caught) {
    return missing(`this is not JSON: ${caughtValueText(caught,)}`,);
  }
}

/**
 Reads every entry of a list file's text.

 @param text - the text

 @param where - the file it was read from

 @returns the entries, in the order the file holds them

 @throws LandingConfigurationError when the text is not a list of entries, or an entry is incomplete

 @example
 ```ts
 const entries = entriesIn({ text: '[{"branch":"fix/a","message":"merge(a): b"}]', where: '/list.json', },);
 ```
 */
export function entriesIn({
  text,
  where,
}: {
  readonly text: string;
  readonly where: string;
},): readonly LandingEntry[] {
  /**
   What the file holds.
   */
  const parsed = parsedText({ text, },);
  if (!parsed.found) throw new LandingConfigurationError(`${where}: ${parsed.why}`,);
  if (!Array.isArray(parsed.value,)) {
    throw new LandingConfigurationError(`${where}: a list file holds an array of entries`,);
  }
  return parsed.value
    .map(function entryOf(
      record: unknown,
      index: number,
    ) {
    /**
     Where a refusal names this entry.
     */
      const at = `${where}: entry ${String(index + 1,)}`;
      if (!isRecord(record,)) throw new LandingConfigurationError(`${at} is not an object`,);
    /**
     The entry itself.
     */
      const entry = record;
    /**
     The message its merge commit carries, which every entry needs.
     */
      const message = textOf({
        value: entry.message,
        field: 'message',
        where: at,
      },);
      if (message === '') throw new LandingConfigurationError(`${at} names no message, and a merge commit needs one`,);
      return {
        branch: textOf({
          value: entry.branch,
          field: 'branch',
          where: at,
        },),
        worktree: textOf({
          value: fieldOf({
            record: entry,
            names: [
              'worktree',
              'agent',
              'name',
            ],
          },),
          field: 'worktree',
          where: at,
        },),
        check: checkOf({
          record: entry,
          where: at,
        },),
        tasks: [
          ...tasksOf({
            value: entry.tasks,
            field: 'tasks',
            where: at,
          },),
          ...tasksOf({
            value: entry.consumerTasks,
            field: 'consumerTasks',
            where: at,
          },),
        ],
        message,
        note: textOf({
          value: fieldOf({
            record: entry,
            names: [
              'note',
              'lintEqualToMain',
            ],
          },),
          field: 'note',
          where: at,
        },),
        key: textOf({
          value: entry.key,
          field: 'key',
          where: at,
        },),
      };
    },);
}

/**
 Reads a list file from disk.

 @param path - the path

 @returns the entries

 @throws LandingConfigurationError when the file cannot be read

 @example
 ```ts
 const entries = await entriesOfFile({ path: '/home/user/temp/agent/round.json', },);
 ```
 */
export async function entriesOfFile({ path, }: {
  readonly path: string;
},): Promise<readonly LandingEntry[]> {
  try {
    return entriesIn({
      text: await readFile(
        path,
        'utf8',
      ),
      where: path,
    },);
  }
  catch (caught) {
    if (caught instanceof LandingConfigurationError) throw caught;
    throw new LandingConfigurationError(`${path}: this file cannot be read`,);
  }
}
