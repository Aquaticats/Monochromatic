/**
 Task-specific input parsing for the update API handler.
 
 Delegates to generic validation helpers from task-validation.ts,
 adding task-domain field parsing on top.
 */
import {
  TASK_PRIORITIES,
  TASK_STATUSES,
  type TaskComplexity,
  type TaskPriority,
  type TaskStatus,
  type TaskUpdateInput,
} from '../../lib/types.ts';
import {
  INVALID,
  isRecord,
  parseEnumValue,
  parseStringArray,
} from './task-validation.ts';

/**
 Recognized priority/complexity values for input validation.
 */
const priorities = new Set<string>(TASK_PRIORITIES,);

/**
 Recognized status values for input validation.
 */
const statuses = new Set<string>(TASK_STATUSES,);

/**
 Validates a task status value from untrusted input.
 
 @param value - Raw input value
 
 @returns Validated status, or {@link INVALID} when absent or unrecognized
 */
function parseStatus(value: unknown,): TaskStatus | typeof INVALID {
  if ((typeof value) !== 'string')
    return INVALID;
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- validated by Set.has check
  return statuses.has(value as TaskStatus,) ? (value as TaskStatus) : INVALID;
}

/**
 Validates and extracts a {@link TaskUpdateInput} from an untrusted request body.
 
 A field explicitly set to `null` is treated as "not provided" and skipped
 (the merge in `updateTask` then keeps the existing value); a non-string,
 non-null value for a string/enum field aborts the whole parse.
 
 @param value - Raw parsed JSON body
 
 @returns Parsed update payload, or {@link INVALID} when any field fails validation
 
 @example
 ```ts
 const input = parseTaskUpdateInput(await req.json());
 if (input === INVALID) return badRequest();
 ```
 */
export function parseTaskUpdateInput(value: unknown,): TaskUpdateInput | typeof INVALID {
  if (!isRecord(value,))
    return INVALID;
  /**
   Accumulator: each field block validates the input and adds the parsed value here. Mutable mirror of the readonly {@link TaskUpdateInput}, restored to readonly on return.
   */
  const result: { -readonly [K in keyof TaskUpdateInput]: TaskUpdateInput[K]; } = {};

  if ('title' in value) {
    if ((typeof value.title) !== 'string')
      return INVALID;
    result.title = value.title;
  }
  if ('description' in value) {
    if ((typeof value.description) === 'string')
      result.description = value.description;
    else if (value.description
      !== null)
      return INVALID;
  }
  if ('tags' in value) {
    /**
     Parsed `tags` array; {@link INVALID} aborts the parse.
     */
    const v = parseStringArray(value.tags,);
    if (v === INVALID)
      return INVALID;
    result.tags = v;
  }
  if ('locations' in value) {
    /**
     Parsed `locations` array; {@link INVALID} aborts the parse.
     */
    const v = parseStringArray(value.locations,);
    if (v === INVALID)
      return INVALID;
    result.locations = v;
  }
  if ('blockedBy' in value) {
    /**
     Parsed `blockedBy` array; {@link INVALID} aborts the parse.
     */
    const v = parseStringArray(value.blockedBy,);
    if (v === INVALID)
      return INVALID;
    result.blockedBy = v;
  }
  if ('reminders' in value) {
    /**
     Parsed `reminders` array; {@link INVALID} aborts the parse.
     */
    const v = parseStringArray(value.reminders,);
    if (v === INVALID)
      return INVALID;
    result.reminders = v;
  }
  if (('priority' in value) && (value.priority
    !== null)) {
    /**
     Parsed priority enum; {@link INVALID} signals an unrecognised value and aborts the parse.
     */
    const v = parseEnumValue({
      value: value.priority,
      validValues: priorities,
    },);
    if (v === INVALID)
      return INVALID;
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- v is a member of the priorities set, which holds exactly the TaskPriority values
    result.priority = v as TaskPriority;
  }
  if (('complexity' in value) && (value.complexity
    !== null)) {
    /**
     Parsed complexity enum; {@link INVALID} signals an unrecognised value and aborts the parse.
     */
    const v = parseEnumValue({
      value: value.complexity,
      validValues: priorities,
    },);
    if (v === INVALID)
      return INVALID;
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- v is a member of the priorities set, whose low/medium/high values are exactly the TaskComplexity values
    result.complexity = v as TaskComplexity;
  }
  if ('dueDate' in value) {
    if ((typeof value.dueDate) === 'string')
      result.dueDate = value.dueDate;
    else if (value.dueDate
      !== null)
      return INVALID;
  }
  if ('status' in value) {
    /**
     Parsed task status; {@link INVALID} signals an unrecognised value and aborts the parse.
     */
    const v = parseStatus(value.status,);
    if (v === INVALID)
      return INVALID;
    result.status = v;
  }

  return result;
}
