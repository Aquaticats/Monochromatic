/**
 Starting one entry of a queue: what the entry is read as, what is written or asked for before its run exists, and which of the two
 kinds of failure each step's refusal is.

 The start ends when the entry's own unit is asked for, which is the point the filling can see: the batch inside that unit waits for
 a pool, makes the worktree and starts the driver, and a failure after this point belongs to the batch and the run, whose output
 file and summary hold it. So this module answers whether the entry was handed on, and never whether its agent fared well.

 A failure is the machine's when the step that refused is the machine's to answer: no place in a pool, a file that cannot be
 written (a full drive), the environment's `PATH` that cannot be read, systemd refusing a unit, or git refusing because a lock is
 held or its volume is full. Every other refusal is the entry's own: it names no run or review the package reads, the change under
 its range leaves nothing to review, its range names no commit, or a unit of its name already exists.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import {
  BATCH_DEFAULT_CPU_PERCENT,
  BATCH_DEFAULT_DEADLINE_SECONDS,
  BATCH_DEFAULT_MEMORY,
  BATCH_DEFAULT_TASK_COUNT,
  type BatchEntry,
} from './batch-list.ts';
import { caughtValueStack, } from '@monochromatic-dev/caught-value-core/ts';
import {
  changeOfAReviewRun,
  type ReviewOfARangeEntry,
  shapeOfAQueueEntry,
} from './entry-of-a-queue.ts';
import {
  type GitQuestion,
  ReviewGitError,
} from './material-of-a-review.ts';
import type { QueueRecord, } from './queue-of-runs.ts';
import { FOCUS_OF_A_CHANGE_REVIEW, } from './review-brief.ts';
import {
  REVIEW_WALL_SECONDS,
} from './reviews-of-finished-branches.ts';
import {
  REVIEWING_MODEL,
  REVIEWING_PROVIDER,
  branchOfAReview,
  briefFileOfAReview,
  runNameOfAReview,
} from './reviewer-of-a-change.ts';
import { rulesFileIn, } from './rules-every-pi-brief-carries.ts';
import {
  type RunCommand,
  runCommandWithChildProcess,
} from './run-command.ts';
import { startABatchFile, } from './starting-a-batch-in-its-own-unit.ts';
import { writeReviewBrief, } from './writing-a-review-brief.ts';

/**
 How many characters of an entry's name go into the batch list's file name, which also becomes its unit's name.
 */
const NAME_CHARACTERS_IN_A_FILE = 60;

/**
 Characters in the ISO 8601 time a batch list's name begins with, up to the second: `2026-11-14T09:21:14`.
 */
const STAMP_CHARACTERS = 19;

/**
 The words a git refusal uses when the machine, not the entry, is why git did not answer: a lock file another process holds, a
 volume with no room, or a filesystem that may not be written.
 */
const MACHINE_REFUSAL_WORDS: readonly string[] = [
  'index.lock',
  'Unable to create',
  'File exists',
  'could not lock',
  'No space left on device',
  'ENOSPC',
  'Read-only file system',
  'Disk quota exceeded',
];

/**
 Which side is to answer a failed start: the entry's own contents, or the machine every entry shares.
 */
export type KindOfAStartFailure = 'the-entry' | 'the-machine';

/**
 Why one entry's start failed.
 */
export type FailureOfAStart = {
  /**
   Whose failure it is.
   */
  readonly kind: KindOfAStartFailure;

  /**
   What the failed step said, in one line.
   */
  readonly reason: string;

  /**
   The whole of what the failed step printed.
   */
  readonly output: string;

  /**
   Where the whole of what the failed step printed was kept beside the runs, when it was kept.
   */
  readonly keptOutputAt?: string;
};

/**
 What starting one entry of a queue answered.
 */
export type StartOfAnEntry =
  | {
    readonly started: true;

    /**
     The lines the start printed, in the order they happened.
     */
    readonly lines: readonly string[];

    /**
     The batch list the entry was started from.
     */
    readonly batchFile: string;

    /**
     The unit the batch was started in, without its `.service` suffix.
     */
    readonly unit: string;

    /**
     Where that unit writes both of its streams.
     */
    readonly outputFile: string;
  }
    | {
      readonly started: false;
      readonly failure: FailureOfAStart;
    };

/**
 The seams one entry's start asks, so a test answers them without a git, a unit or a repository.
 */
export type StartOfAnEntrySeams = {
  /**
   How one git question is run.
   */
  readonly git: GitQuestion;

  /**
   How a command is run.
   */
  readonly run?: RunCommand;
};

/**
 Says whether a git refusal is the machine's rather than the entry's.

 @param reason - what the review's git question refused with

 @returns whether the refusal names a held lock, a full volume or a filesystem that may not be written

 @example
 ```ts
 const theMachine = gitRefusalIsTheMachines({ reason: 'fatal: Unable to create .../index.lock: File exists' });
 ```
 */
export function gitRefusalIsTheMachines({ reason, }: { readonly reason: string },): boolean {
  return MACHINE_REFUSAL_WORDS.some(function isNamed(word,) {
    return reason.includes(word,);
  },);
}

/**
 Reads a failure that threw rather than answering.

 @param error - what the step threw

 @returns the reason and the whole of what it said

 @example
 ```ts
 const failure = failureOfAThrow({ error: new Error('no PATH') });
 ```
 */
function failureOfAThrow({ error, }: { readonly error: unknown },): FailureOfAStart {
  /**
   The text the throw carries, which is its message and the stack Node printed with it.
   */
  const output = caughtValueStack(error,);
  return {
    kind: 'the-machine',
    reason: output.split('\n',)[0]
      ?? output,
    output,
  };
}

/**
 Reads the failure of a step whose failure is the entry's own.

 @param reason - what the step answered

 @param output - the whole of what the step printed

 @returns the failure

 @example
 ```ts
 const failure = entryFailureOf({ reason: 'nothing to review under src in main...a', output: 'nothing to review under src in main...a\n' });
 ```
 */
function entryFailureOf({
  reason,
  output,
}: {
  readonly reason: string;
  readonly output: string;
},): FailureOfAStart {
  return {
    kind: 'the-entry',
    reason,
    output,
  };
}

/**
 Reads the batch entry one review of a range is started as: the run named after the change, on the branch and from the brief the
 package's writer made, on the provider and model every review runs on.

 @param review - the review of a range

 @param reviewsFolder - folder that holds one folder per review

 @returns the batch entry to write

 @example
 ```ts
 const entry = reviewRunEntryOf({ review: { name: 'review-a', reviewOfRange: 'main...a', commit: 'a', source: 'src' }, reviewsFolder: '/reviews' });
 ```
 */
export function reviewRunEntryOf({
  review,
  reviewsFolder,
}: {
  readonly review: ReviewOfARangeEntry;
  readonly reviewsFolder: string;
},): BatchEntry {
  /**
   The change under review.
   */
  const change = changeOfAReviewRun({ run: review.name, },);
  return {
    name: runNameOfAReview({ review: change, },),
    branch: branchOfAReview({ review: change, },),
    base: review.commit,
    prompt: briefFileOfAReview({
      reviewsFolder,
      review: change,
    },),
    cpu: BATCH_DEFAULT_CPU_PERCENT,
    memory: BATCH_DEFAULT_MEMORY,
    tasks: BATCH_DEFAULT_TASK_COUNT,
    wall: REVIEW_WALL_SECONDS,
    provider: REVIEWING_PROVIDER,
    model: REVIEWING_MODEL,
  };
}

/**
 What writing one review's brief leaves for the start: the run's batch entry, or why there is none.
 */
type BriefedForAStart =
  | {
    readonly handedOn: {
      readonly entry: BatchEntry;
      readonly lines: readonly string[];
    };
  }
    | { readonly refused: FailureOfAStart };

/**
 Reads what one review of a range left to read, writing its diff, brief and reproductions folder.

 @param record - the entry

 @param review - the review of a range

 @param request - the folders the review reads and writes, and the checkout its range is read in

 @param seams - how git is asked

 @returns the run's batch entry, or the failure that kept it from being written

 @example
 ```ts
 const briefed = await briefOfAReviewOfARange({ record, review, request, seams });
 ```
 */
async function briefOfAReviewOfARange({
  record,
  review,
  request,
  seams,
}: {
  readonly record: QueueRecord;
  readonly review: ReviewOfARangeEntry;
  readonly request: StartOfAnEntryRequest;
  readonly seams: StartOfAnEntrySeams;
},): Promise<BriefedForAStart> {
  /**
   What writing the review's brief answered.
   */
  try {
    /**
     The brief written, or the line saying the change leaves nothing to review.
     */
    const briefed = await writeReviewBrief({
      git: seams.git,
      checkoutFolder: request.checkoutFolder,
      review: changeOfAReviewRun({ run: record.name, },),
      commit: review.commit,
      ranges: [review.reviewOfRange,],
      source: review.source,
      focus: FOCUS_OF_A_CHANGE_REVIEW,
      reviewsFolder: request.reviewsFolder,
      runsFolder: request.runsFolder,
      rulesFile: rulesFileIn({ checkoutFolder: request.checkoutFolder, },),
    },);
    return briefed.written
      ? {
        handedOn: {
          entry: reviewRunEntryOf({
            review,
            reviewsFolder: request.reviewsFolder,
          },),
          lines: [briefed.line,],
        },
      }
      : {
        refused: entryFailureOf({
          reason: briefed.line,
          output: briefed.line,
        },),
      };
  }
  catch (error) {
    // A git question that did not answer may be the entry's failure (a range that names no commit) or the machine's (a lock held,
    // a full volume), and git's own words are the only thing that tells them apart here.
    if (!(error instanceof ReviewGitError)) {
      return {
        refused: failureOfAThrow({ error, },),
      };
    }
    return {
      refused: {
        kind: gitRefusalIsTheMachines({ reason: error.message, }) ? 'the-machine' : 'the-entry',
        reason: error.message,
        output: caughtValueStack(error,),
      },
    };
  }
}

/**
 What starting one entry of a queue reads and where it writes.
 */
export type StartOfAnEntryRequest = {
  /**
   The entry, as the queue holds it.
   */
  readonly record: QueueRecord;

  /**
   Folder the runs live in, where the batch list goes.
   */
  readonly runsFolder: string;

  /**
   Folder that holds one folder per review.
   */
  readonly reviewsFolder: string;

  /**
   The checkout the review's range is read in.
   */
  readonly checkoutFolder: string;

  /**
   The home folder the entry's unit runs with.
   */
  readonly home: string;


  /**
   Where the start reads the time its batch list is named by.
   */
  readonly now: () => number;

  /**
   How long the batch inside the unit keeps asking for a slot.
   */
  readonly deadlineSeconds?: number;
};

/**
 Writes the one-entry batch list an entry is started from.

 @param request - the entry, the folders and the time

 @param entry - the batch entry to write

 @returns the list's path, or the failure that kept it from being written

 @example
 ```ts
 const written = await batchListOf({ request, entry });
 ```
 */
async function batchListOf({
  request,
  entry,
}: {
  readonly request: StartOfAnEntryRequest;
  readonly entry: BatchEntry;
},): Promise<{ readonly written: string } | { readonly refused: FailureOfAStart }> {
  /**
   The stamp the list's name begins with, so its unit's name is one a reader can place in time.
   */
  const stamp = new Date(request.now(),).toISOString()
    .slice(
    0,
    STAMP_CHARACTERS,
    )
    .replaceAll(
    ':',
    '',
    );
  /**
   The list's path, beside the runs, named after the entry and the moment.
   */
  const file = join(
    request.runsFolder,
    `batch-${stamp}-${String(process.pid,)}-queue-entry-${entry.name
      .slice(
      0,
      NAME_CHARACTERS_IN_A_FILE,
    )}.json`,
  );
  try {
    await writeFile(
      file,
      `${JSON.stringify(
        [entry,],
        undefined,
        2,
      )}\n`,
    );
    return { written: file, };
  }
  catch (error) {
    return { refused: failureOfAThrow({ error, },), };
  }
}

/**
 Starts one entry of a queue.

 @param request - the entry, the folders it reads and writes, and the time

 @param seams - how git and the commands are run

 @returns whether the entry was handed to its own unit, the lines that said so, or why its start failed

 @example
 ```ts
 const started = await startAnEntryOfAQueue({ request, seams });
 ```
 */
export async function startAnEntryOfAQueue({
  request,
  seams,
}: {
  readonly request: StartOfAnEntryRequest;
  readonly seams: StartOfAnEntrySeams;
},): Promise<StartOfAnEntry> {
  /**
   What the entry is.
   */
  const shape = shapeOfAQueueEntry({ record: request.record, },);
  if (shape.kind === 'unreadable') {
    return {
      started: false,
      failure: entryFailureOf({
        reason: shape.reason,
        output: shape.reason,
      },),
    };
  }
  /**
   What the start needs before the unit can be asked for: the run's batch entry and the lines written for it, or why there is none.
   */
  const prepared: BriefedForAStart = (shape.kind === 'a-review-of-a-range')
    ? await briefOfAReviewOfARange({
      record: request.record,
      review: shape.entry,
      request,
      seams,
    },)
    : {
      handedOn: {
        entry: shape.entry,
        lines: [],
      },
    };
  if ('refused' in prepared) {
    return {
      started: false,
      failure: prepared.refused,
    };
  }
  /**
   The batch entry the unit starts, and the lines written for it.
   */
  const { handedOn, } = prepared;
  /**
   The one-entry batch list the unit starts.
   */
  const list = await batchListOf({
    request,
    entry: handedOn.entry,
  },);
  if ('refused' in list) {
    return {
      started: false,
      failure: list.refused,
    };
  }
  try {
    /**
     What starting the batch's unit answered.
     */
    const started = await startABatchFile({
      request: {
        batchFile: list.written,
        deadlineSeconds: request.deadlineSeconds ?? BATCH_DEFAULT_DEADLINE_SECONDS,
        checkoutFolder: request.checkoutFolder,
        home: request.home,
      },
      ...((seams.run === undefined) ? {} : { run: seams.run, }),
    },);
    if (started.status !== 0) {
      return {
        started: false,
        failure: {
          kind: 'the-machine',
          reason: started.problem,
          output: started.problem,
        },
      };
    }
    return {
      started: true,
      lines: [
        ...handedOn.lines,
        started.line,
      ],
      batchFile: list.written,
      unit: started.unit,
      outputFile: started.outputFile,
    };
  }
  catch (error) {
    // A `PATH` that cannot be read, a checkout that cannot be found or a batch command that is not there is nothing the entry
    // could have been written to avoid, so the machine answers it.
    return {
      started: false,
      failure: failureOfAThrow({ error, },),
    };
  }
}
