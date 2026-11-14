/**
 The three shapes of entry a queue of runs holds: a fresh run, a run that resumes a finished one, and a review of a range of a
 change. Each entry is read into the shape it names, and an entry that names neither is refused with the reason rather than
 dropped, because a field silently ignored starts a run other than the one the entry describes.

 The two run shapes are one shape, which is the batch list's own entry: what tells a resume apart is `sessionOf`. A review of a
 range is the shape the queue's writer for a second review makes, with the review's provider and model taken from
 `reviewer-of-a-change.ts` so no queue file carries a model's name.

 @module
 */

import {
  type BatchEntry,
  BATCH_ENTRY_FIELDS,
} from './batch-list.ts';
import {
  MODEL_OF_A_RUN_NAMING_NONE,
  PROVIDER_OF_A_RUN_NAMING_NONE,
  type PoolMembership,
  membershipOf,
  poolNamed,
} from './pool-limits.ts';
import type { QueueRecord, } from './queue-of-runs.ts';
import {
  REVIEWING_MODEL,
  REVIEWING_PROVIDER,
  REVIEW_RUN_PREFIX,
} from './reviewer-of-a-change.ts';

/**
 The fields an entry of a review of a range may name, and no other: the change it reviews is named by its range, the commit the
 review's worktree starts at, and the folder whose change is read.
 */
export const REVIEW_OF_A_RANGE_ENTRY_FIELDS: readonly string[] = [
  'name',
  'reviewOfRange',
  'commit',
  'source',
];

/**
 The run fields whose value is text.
 */
const TEXT_RUN_FIELDS: readonly string[] = [
  'branch',
  'base',
  'prompt',
  'memory',
  'sessionOf',
  'session',
  'provider',
  'model',
];

/**
 The run fields whose value is a whole number.
 */
const NUMBER_RUN_FIELDS: readonly string[] = [
  'cpu',
  'tasks',
  'wall',
  'run',
];

/**
 A review of a range of a change, as a queue holds it.
 */
export type ReviewOfARangeEntry = {
  /**
   The review's name, which is `review-` and the change's name.
   */
  readonly name: string;

  /**
   What changed, as `git diff` takes a range.
   */
  readonly reviewOfRange: string;

  /**
   The commit the review's worktree starts at.
   */
  readonly commit: string;

  /**
   The folder whose change is reviewed, from the top of the checkout.
   */
  readonly source: string;
};

/**
 What one entry of a queue is.
 */
export type ShapeOfAQueueEntry =
  | {
    readonly kind: 'a-run';
    readonly entry: BatchEntry;
  }
    | {
      readonly kind: 'a-review-of-a-range';
      readonly entry: ReviewOfARangeEntry;
    }
    | {
      readonly kind: 'unreadable';
      readonly reason: string;
    };

/**
 Says whether one field of a record holds the type its name gives it.

 @param record - the entry

 @param field - the field's name

 @param wanted - `text` or `number`

 @returns whether the field is absent, or holds that type

 @example
 ```ts
 const fine = fieldHasItsType({ record: { name: 'a' }, field: 'wall', wanted: 'number' });
 ```
 */
function fieldHasItsType({
  record,
  field,
  wanted,
}: {
  readonly record: QueueRecord;
  readonly field: string;
  readonly wanted: 'text' | 'number';
},): boolean {
  /**
   The field's value, of any type.
   */
  const value: unknown = record[field];
  if (value === undefined) {
    return true;
  }
  return (wanted === 'number') ? (typeof value) === 'number' : (typeof value) === 'string';
}

/**
 Reads the fields an entry names that the shape it takes does not, which is why it is refused.

 @param record - the entry

 @param known - the fields the shape holds

 @returns the names of the fields it names and the shape does not, in the order the entry names them

 @example
 ```ts
 const unread = fieldsTheShapeDoesNotHold({ record: { name: 'a', reviewer: 'b' }, known: ['name'] });
 ```
 */
function fieldsTheShapeDoesNotHold({
  record,
  known,
}: {
  readonly record: QueueRecord;
  readonly known: readonly string[];
},): readonly string[] {
  return Object.keys(record,)
    .filter(function isUnread(field,) {
      return !known.includes(field,);
    },);
}

/**
 Reads an entry as a run of a batch: a fresh run, or one that resumes a finished run.

 @param record - the entry

 @returns the run, or why the entry is no run

 @example
 ```ts
 const run = runInAQueueEntry({ record: { name: 'a', branch: 'feat/a' } });
 ```
 */
function runInAQueueEntry({ record, }: { readonly record: QueueRecord },): {
  readonly entry?: BatchEntry;
  readonly reason?: string;
} {
  /**
   The fields this entry names that a run does not hold.
   */
  const unread = fieldsTheShapeDoesNotHold({
    record,
    known: BATCH_ENTRY_FIELDS,
  },);
  if (unread.length > 0) {
    return {
      reason: `it names a field a run does not read: ${unread.join(', ')}. A run reads ${BATCH_ENTRY_FIELDS.join(', ')}, and a field that would be dropped is refused rather than ignored`,
    };
  }
  /**
   The field whose value is not the type its name gives it, if any.
   */
  const mistyped = [
    ...TEXT_RUN_FIELDS.filter(function isText(field,) {
      return !fieldHasItsType({
        record,
        field,
        wanted: 'text',
      },);
    },),
    ...NUMBER_RUN_FIELDS.filter(function isANumber(field,) {
      return !fieldHasItsType({
        record,
        field,
        wanted: 'number',
      },);
    },),
  ];
  if (mistyped.length > 0) {
    return { reason: `it holds a value of the wrong type for ${mistyped.join(', ')}`, };
  }
  if ((typeof record.branch) !== 'string') {
    return { reason: 'it names no branch, which every run of a queue needs', };
  }
  return {
    entry: {
      name: record.name,
      branch: record.branch,
      ...((typeof record.base) === 'string' ? { base: record.base, } : {}),
      ...((typeof record.prompt) === 'string' ? { prompt: record.prompt, } : {}),
      ...((typeof record.cpu) === 'number' ? { cpu: record.cpu, } : {}),
      ...((typeof record.memory) === 'string' ? { memory: record.memory, } : {}),
      ...((typeof record.tasks) === 'number' ? { tasks: record.tasks, } : {}),
      ...((typeof record.wall) === 'number' ? { wall: record.wall, } : {}),
      ...((typeof record.run) === 'number' ? { run: record.run, } : {}),
      ...((typeof record.sessionOf) === 'string' ? { sessionOf: record.sessionOf, } : {}),
      ...((typeof record.session) === 'string' ? { session: record.session, } : {}),
      ...((typeof record.provider) === 'string' ? { provider: record.provider, } : {}),
      ...((typeof record.model) === 'string' ? { model: record.model, } : {}),
    },
  };
}

/**
 Reads an entry as a review of a range.

 @param record - the entry

 @returns the review, or why the entry is no review of a range

 @example
 ```ts
 const review = reviewOfARangeInAQueueEntry({ record: { name: 'review-a', reviewOfRange: 'main...a', commit: 'a', source: 'src' } });
 ```
 */
function reviewOfARangeInAQueueEntry({ record, }: { readonly record: QueueRecord },): {
  readonly entry?: ReviewOfARangeEntry;
  readonly reason?: string;
} {
  /**
   The fields this entry names that a review of a range does not hold.
   */
  const unread = fieldsTheShapeDoesNotHold({
    record,
    known: REVIEW_OF_A_RANGE_ENTRY_FIELDS,
  },);
  if (unread.length > 0) {
    return {
      reason: `it names a field this review shape does not read: ${unread.join(', ')}. It reads ${REVIEW_OF_A_RANGE_ENTRY_FIELDS.join(', ')}, and a field that would be dropped is refused rather than ignored`,
    };
  }
  /**
   The range, the commit and the source folder the entry names.
   */
  const {
    reviewOfRange,
    commit,
    source,
  } = record;
  /**
   The fields it names as no text, if any.
   */
  const missing = [
    ...((typeof reviewOfRange) === 'string' ? [] : ['reviewOfRange',]),
    ...((typeof commit) === 'string' ? [] : ['commit',]),
    ...((typeof source) === 'string' ? [] : ['source',]),
  ];
  if (
    ((typeof reviewOfRange) !== 'string')
      || ((typeof commit) !== 'string')
      || ((typeof source) !== 'string')
  ) {
    return { reason: `it holds no text for ${missing.join(', ')}, which a review of a range needs`, };
  }
  return {
    entry: {
      name: record.name,
      reviewOfRange,
      commit,
      source,
    },
  };
}

/**
 Reads what one entry of a queue is.

 @param record - the entry

 @returns the run it names, the review of a range it names, or why it is neither

 @example
 ```ts
 const shape = shapeOfAQueueEntry({ record: { name: 'a', branch: 'feat/a' } });
 ```
 */
export function shapeOfAQueueEntry({ record, }: { readonly record: QueueRecord },): ShapeOfAQueueEntry {
  if ((typeof record.reviewOfRange) === 'string') {
    /**
     What reading the entry as a review of a range said.
     */
    const review = reviewOfARangeInAQueueEntry({ record, },);
    return (review.entry === undefined)
      ? {
        kind: 'unreadable',
        reason: review.reason ?? 'it is no review of a range',
      }
      : {
        kind: 'a-review-of-a-range',
        entry: review.entry,
      };
  }
  /**
   What reading the entry as a run said.
   */
  const run = runInAQueueEntry({ record, },);
  return (run.entry === undefined)
    ? {
      kind: 'unreadable',
      reason: run.reason ?? 'it is no run',
    }
    : {
      kind: 'a-run',
      entry: run.entry,
    };
}

/**
 Reads the pool an entry asks for, which is what tells whether the machine has a place for it now.

 A review of a range runs on the reviewing provider and model, which the package names once, so a queue file never carries them.

 @param record - the entry

 @returns the pool it asks for, every pool its name could hold, or none

 @example
 ```ts
 const pool = poolOfAQueueEntry({ record: { name: 'a', branch: 'feat/a' } });
 ```
 */
export function poolOfAQueueEntry({ record, }: { readonly record: QueueRecord },): PoolMembership {
  if ((typeof record.reviewOfRange) === 'string') {
    return poolNamed({
      provider: REVIEWING_PROVIDER,
      model: REVIEWING_MODEL,
    },);
  }
  return poolNamed({
    provider: (typeof record.provider) === 'string' ? record.provider : PROVIDER_OF_A_RUN_NAMING_NONE,
    model: (typeof record.model) === 'string' ? record.model : MODEL_OF_A_RUN_NAMING_NONE,
  },);
}

/**
 Reads the pools a place asked for holds until its run appears.

 A run that named no provider or model of its own is placed on the first default pool with room by the pool gate, which is not read
 here: until the run's own unit says which model it runs, the place it will fill is every pool its name could hold, which is how the
 package counts a run whose command line nobody has read. Counting it in one guessed pool is what let a level be passed while the
 launchers that were promised the places waited.

 @param record - the entry

 @returns the pools the place holds, or none when no shared pool holds it

 @example
 ```ts
 const pools = poolAPlaceAskedForWouldHold({ record: { name: 'a', branch: 'feat/a' } });
 ```
 */
export function poolAPlaceAskedForWouldHold({ record, }: { readonly record: QueueRecord },): PoolMembership {
  if ((typeof record.reviewOfRange) === 'string') {
    return poolNamed({
      provider: REVIEWING_PROVIDER,
      model: REVIEWING_MODEL,
    },);
  }
  return ((typeof record.provider) === 'string') || ((typeof record.model) === 'string')
    ? poolNamed({
      provider: (typeof record.provider) === 'string' ? record.provider : PROVIDER_OF_A_RUN_NAMING_NONE,
      model: (typeof record.model) === 'string' ? record.model : MODEL_OF_A_RUN_NAMING_NONE,
    },)
    : membershipOf({ runName: record.name, },);
}

/**
 Says whether an entry resumes a finished run.

 @param record - the entry

 @returns whether it names a session to go on from

 @example
 ```ts
 const resumes = resumesAQueueEntry({ record: { name: 'a', sessionOf: 'a-1' } });
 ```
 */
export function resumesAQueueEntry({ record, }: { readonly record: QueueRecord },): boolean {
  return (typeof record.sessionOf) === 'string';
}

/**
 Reads the change a review's run name is about.

 @param run - the run's name, as `review-<change>`

 @returns the change's name, or the run's own name when it does not begin with the review prefix

 @example
 ```ts
 const change = changeOfAReviewRun({ run: 'review-rewrite-the-gate' });
 ```
 */
export function changeOfAReviewRun({ run, }: { readonly run: string },): string {
  return run.startsWith(REVIEW_RUN_PREFIX,)
    ? run.slice(REVIEW_RUN_PREFIX.length,)
    : run;
}
