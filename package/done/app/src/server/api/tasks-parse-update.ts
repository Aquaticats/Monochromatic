/**
 Task update input parsing and validation.
 
 Separated from `tasks-parse.ts` because `parseTaskUpdateInput` is the
 largest single validation function and exceeds the line budget when
 combined with the primitive helpers.
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
} from './tasks-parse.ts';

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
 Validates and extracts a {@link TaskUpdateInput} from an untrusted request
 body, using {@link isRecord}, {@link parseStringArray}, {@link parseEnumValue},
 and {@link parseStatus} for field-level validation.
 
 A field explicitly set to `null` is treated as "not provided" and skipped
 (the merge in {@link updateTask} then keeps the existing value); a non-string,
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
   Accumulator: each field block validates the input and adds the parsed value here. Mutable mirror of the readonly `TaskUpdateInput`, restored to readonly on return.
   */
  const taskUpdateInput: { -readonly [K in keyof TaskUpdateInput]: TaskUpdateInput[K]; } = {};

  if ('title' in value) {
    if ((typeof value.title) !== 'string')
      return INVALID;
    taskUpdateInput.title = value.title;
  }

  if ('description' in value) {
    if ((typeof value.description) === 'string')
      taskUpdateInput.description = value.description;
    else if (value.description
      !== null)
      return INVALID;
  }

  if ('tags' in value) {
    /**
     Parsed `tags` array; `INVALID` aborts the parse.
     */
    const tags = parseStringArray(value.tags,);
    if (tags === INVALID)
      return INVALID;
    taskUpdateInput.tags = tags;
  }

  if ('locations' in value) {
    /**
     Parsed `locations` array; `INVALID` aborts the parse.
     */
    const locations = parseStringArray(value.locations,);
    if (locations === INVALID)
      return INVALID;
    taskUpdateInput.locations = locations;
  }

  if ('blockedBy' in value) {
    /**
     Parsed `blockedBy` array; `INVALID` aborts the parse.
     */
    const blockedBy = parseStringArray(value.blockedBy,);
    if (blockedBy === INVALID)
      return INVALID;
    taskUpdateInput.blockedBy = blockedBy;
  }

  if ('reminders' in value) {
    /**
     Parsed `reminders` array; `INVALID` aborts the parse.
     */
    const reminders = parseStringArray(value.reminders,);
    if (reminders === INVALID)
      return INVALID;
    taskUpdateInput.reminders = reminders;
  }

  if (('priority' in value) && (value.priority
    !== null)) {
    /**
     Enum-validated priority; `INVALID` signals an unrecognised value and aborts the parse.
     */
    const priority = parseEnumValue({
      value: value.priority,
      validValues: priorities,
    },);
    if (priority === INVALID)
      return INVALID;
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- member of the priorities set, which holds exactly the TaskPriority values
    taskUpdateInput.priority = priority as TaskPriority;
  }

  if (('complexity' in value) && (value.complexity
    !== null)) {
    /**
     Enum-validated complexity; `INVALID` signals an unrecognised value and aborts the parse.
     */
    const complexity = parseEnumValue({
      value: value.complexity,
      validValues: priorities,
    },);
    if (complexity === INVALID)
      return INVALID;
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- member of the priorities set, whose low/medium/high values are exactly the TaskComplexity values
    taskUpdateInput.complexity = complexity as TaskComplexity;
  }

  if ('dueDate' in value) {
    if ((typeof value.dueDate) === 'string')
      taskUpdateInput.dueDate = value.dueDate;
    else if (value.dueDate
      !== null)
      return INVALID;
  }

  if ('status' in value) {
    /**
     Validated status string; `INVALID` aborts the parse.
     */
    const status = parseStatus(value.status,);
    if (status === INVALID)
      return INVALID;
    taskUpdateInput.status = status;
  }

  return taskUpdateInput;
}
