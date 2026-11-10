/**
 Readers that turn one plan field into a checked value.

 Every reader throws a {@link PlanError} naming the field's path,
 so the first typo stops the run with its own address.

 @module
 */

import { PlanError, } from './errors.ts';
import { ECHO_LIMIT, } from './plan-schema.ts';
import { looksLikeCommit, } from './plan-shape.ts';

/**
 A parsed JSON object, read-only so passing one on cannot mutate it.
 */
export type JsonObject = Readonly<Record<string, unknown>>;

/**
 Narrows an unknown value to a keyed record.
 
 @param value - Value read from the plan.
 
 @returns True for a non-array object.
 
 @example
 ```ts
 if (!isRecord(value)) throw new PlanError('the plan must be an object');
 ```
 */
export function isRecord(value: unknown,): value is JsonObject {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value,));
}

/**
 Describes a refused value briefly.
 
 @param value - Value that broke the schema.
 
 @returns Text naming the kind of value found.
 
 @example
 ```ts
 const found = describeValue([]);
 ```
 */
export function describeValue(value: unknown,): string {
  if (value === undefined) {
    return 'missing';
  }
  if ((typeof value) === 'string') {
    return `the string ${JSON.stringify(value.slice(
      0,
      ECHO_LIMIT,
    ),)}`;
  }
  if (Array.isArray(value,)) {
    return `an array of ${String(value.length,)}`;
  }
  if (((typeof value) === 'number') || ((typeof value) === 'boolean')) {
    return `${typeof value} ${String(value,)}`;
  }
  if (isRecord(value,)) {
    return `an object with keys ${Object.keys(value,)
      .join(', ',)}`;
  }
  return typeof value;
}

/**
 Throws the plan failure for one field.
 
 @param at - Path of the field inside the plan.
 
 @param expects - What the field must hold.
 
 @param found - Description of what it held instead.
 
 @throws {@link PlanError} naming the field and the expectation.
 
 @example
 ```ts
 refuse({ at: 'issues', expects: 'an array', found: describeValue({}) });
 ```
 */
export function refuse({
  at,
  expects,
  found,
}: {
  readonly at: string;
  readonly expects: string;
  readonly found: string;
},): never {
  throw new PlanError(`The plan's ${at} must be ${expects}, but it is ${found}.`,);
}

/**
 Reads one field as a keyed record.
 
 @param value - Value read from the plan.
 
 @param at - Path of the field inside the plan.
 
 @returns The field as a record.
 
 @throws {@link PlanError} when the field is not a keyed record.
 
 @example
 ```ts
 const record = asRecord({ value: entry, at: 'issues[0]' });
 ```
 */
export function asRecord({
  value,
  at,
}: {
  readonly value: unknown;
  readonly at: string;
},): JsonObject {
  if (!isRecord(value,)) {
    refuse({
      at,
      expects: 'a JSON object',
      found: describeValue(value,),
    },);
  }
  return value;
}

/**
 Reads one field as a non-empty string.
 
 @param value - Value read from the plan.
 
 @param at - Path of the field inside the plan.
 
 @returns The field as a string.
 
 @throws {@link PlanError} when the field is not a non-empty string.
 
 @example
 ```ts
 const branch = asString({ value: 'main', at: 'branch' });
 ```
 */
export function asString({
  value,
  at,
}: {
  readonly value: unknown;
  readonly at: string;
},): string {
  if (((typeof value) !== 'string') || (value.trim() === '')) {
    refuse({
      at,
      expects: 'a non-empty string',
      found: describeValue(value,),
    },);
  }
  return value;
}

/**
 Reads one field as an integer at or above a floor.
 
 @param value - Value read from the plan.
 
 @param at - Path of the field inside the plan.
 
 @param floor - Smallest accepted value.
 
 @returns The field as a number.
 
 @throws {@link PlanError} when the field is not such an integer.
 
 @example
 ```ts
 const count = asInteger({ value: 2, at: 'issues[0].checks[0].count', floor: 0 });
 ```
 */
export function asInteger({
  value,
  at,
  floor,
}: {
  readonly value: unknown;
  readonly at: string;
  readonly floor: number;
},): number {
  if (((typeof value) !== 'number') || (!Number.isSafeInteger(value,))
    || (value < floor)) {
    refuse({
      at,
      expects: `an integer of at least ${String(floor,)}`,
      found: describeValue(value,),
    },);
  }
  return value;
}

/**
 Reads one field as an array.
 
 @param value - Value read from the plan.
 
 @param at - Path of the field inside the plan.
 
 @returns The field as an array.
 
 @throws {@link PlanError} when the field is not an array.
 
 @example
 ```ts
 const entries = asArray({ value: [], at: 'issues' });
 ```
 */
export function asArray({
  value,
  at,
}: {
  readonly value: unknown;
  readonly at: string;
},): readonly unknown[] {
  if (!Array.isArray(value,)) {
    refuse({
      at,
      expects: 'an array',
      found: describeValue(value,),
    },);
  }
  return value;
}

/**
 Reads one field as a non-empty array of strings.
 
 @param value - Value read from the plan.
 
 @param at - Path of the field inside the plan.
 
 @returns The field as a list of strings.
 
 @throws {@link PlanError} when the list is empty or any item is not a non-empty string.
 
 @example
 ```ts
 const paths = asStringArray({ value: ['package/a'], at: 'issues[0].checks[0].paths' });
 ```
 */
export function asStringArray({
  value,
  at,
}: {
  readonly value: unknown;
  readonly at: string;
},): readonly string[] {
  /**
   Items as read, before each is required to be a path.
   */
  const items = asArray({
    value,
    at,
  },);
  if (items.length === 0) {
    refuse({
      at,
      expects: 'a non-empty array of paths',
      found: 'an empty array',
    },);
  }
  return items.map(function toPath(
    item,
    index,
  ): string {
    return asString({
      value: item,
      at: `${at}[${String(index,)}]`,
    },);
  },);
}

/**
 Reads one field as a commit id.
 
 @param value - Value read from the plan.
 
 @param at - Path of the field inside the plan.
 
 @returns The commit id.
 
 @throws {@link PlanError} when the field is not a commit id.
 
 @example
 ```ts
 const commit = asCommit({ value: '2b309f8f25', at: 'issues[0].checks[0].commit' });
 ```
 */
export function asCommit({
  value,
  at,
}: {
  readonly value: unknown;
  readonly at: string;
},): string {
  /**
   Value as a string, before it is required to be hexadecimal.
   */
  const commit = asString({
    value,
    at,
  },);
  if (!looksLikeCommit(commit,)) {
    refuse({
      at,
      expects: 'a lowercase hexadecimal commit id of seven to forty characters',
      found: JSON.stringify(commit.slice(
        0,
        ECHO_LIMIT,
      ),),
    },);
  }
  return commit;
}

/**
 Refuses a field this schema does not know,
 so a misspelled key cannot be silently ignored.
 
 @param record - Field that was read.
 
 @param allowed - Keys the field may hold.
 
 @param at - Path of the field inside the plan.
 
 @throws {@link PlanError} naming the first unknown key.
 
 @example
 ```ts
 refuseUnknownKeys({ record: {}, allowed: ISSUE_KEYS, at: 'issues[0]' });
 ```
 */
export function refuseUnknownKeys({
  record,
  allowed,
  at,
}: {
  readonly record: JsonObject;
  readonly allowed: readonly string[];
  readonly at: string;
}): void {
  for (const key of Object.keys(record,)) {
    if (!allowed.includes(key,)) {
      refuse({
        at: `${at}.${key}`,
        expects: `absent, or one of ${allowed.join(', ')}`,
        found: 'a key this plan does not define',
      },);
    }
  }
}
