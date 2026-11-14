/**
 Puts every review run that the reviewer's provider ended on its rate limit back at the head of the queue, as an entry that goes on
 from the run's own session, so no session has to notice the ending first.

 Why: on 2026-11-13 twenty-one review runs ended on "429 ... Allocated quota exceeded", three of them with only two or three runs on
 the model going, so no count of runs prevents it. Pi never retries that answer, so each one ends its run. The provider's own page
 on the plan says to wait about a minute and try again, and the filling calls this once a round.

 A run is put back only when it is the newest attempt of its name, its summary names the quota answer, no unit of that name is going,
 the queue does not hold it already, and its review file is not marked final.

 @module
 */

import {
  readFile,
  rename,
  stat,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';
import {
  entriesOrNothingIn,
  isThere,
  SUMMARY_FILE_NAME,
} from './newest-finished-run-of-a-name.ts';
import {
  isDigitsOnly,
} from './text-of-an-option.ts';
import {
  putAQueueEntryIn,
  type QueueRecord,
  readQueueOfRuns,
} from './queue-of-runs.ts';
import { summaryIn, } from './report-of-a-run.ts';
import {
  REVIEWING_MODEL,
  REVIEWING_PROVIDER,
  REVIEW_RUN_PREFIX,
  branchOfAReview,
  reviewFileOfAReview,
} from './reviewer-of-a-change.ts';
import { MILLISECONDS_PER_SECOND, } from './driver-constants.ts';
import { REVIEW_WALL_SECONDS, } from './reviews-of-finished-branches.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The end reason a run the model provider ended carries.
 */
const RATE_LIMIT_END_REASON = 'model-error';

/**
 The words the provider's quota answer carries, which is what tells that ending from every other model error.
 */
const QUOTA_REFUSAL_WORD = 'insufficient_quota';

/**
 How many characters of a review file say whether it is marked final.
 */
const REVIEW_HEAD_CHARACTERS = 600;

/**
 The start of the entries an integration pass keeps at the head of the queue, whose place a put-back review must not take: on
 2026-11-13 at 16:07:34 a review put back took the one place that freed while the pass carrying the jam fix had waited at the head
 since 16:04.
 */
const INTEGRATION_PASS_PREFIX = 'integration-pass-';

/**
 Seconds a run may have ended within and still be put back: the filling unit's own lifetime, so a filling never resurrects a review
 the previous filling already put back or decided about.
 */
export const REVIEWS_PUT_BACK_LOOKBACK_SECONDS = 28_800;

/**
 The file the prompt of a review put back after the rate limit is written to, below the runs folder.
 */
export const RATE_LIMIT_REVIEW_PROMPT_FILE_NAME = 'go-on-with-a-review-after-the-rate-limit-prompt.md';

/**
 A folder with no closing record yet, as a symbol so no record can be mistaken for none.
 */
const NOT_FINISHED_YET: unique symbol = Symbol('the folder checked holds no closing record, so that work has not stopped',);

/**
 What a review put back after the rate limit is told.

 The text is the package's own, so a queue file needs no prompt a person wrote, and it names what the run must know: the ending was
 not its doing, its session is as it was, and the same limit can end it again.
 */
export const RATE_LIMIT_REVIEW_PROMPT: string = `Your review run was ended from outside, and nothing you did caused it: the provider
answered one request with "429 ... Allocated quota exceeded", which its documentation gives as a limit on tokens per second or
minute, and neither Pi nor the driver asked again. You are started again with your session as it was, in the same worktree, with the
same review folder.

Go on with the review from where you were. Your brief is the first message of this session and still holds, word for word.

Before you go on:

- Read your review folder and your worktree's state. The last command you started may not have finished, and a file you were
  writing may be cut short: read its end before you trust it.
- A build or a test that was running when the run ended is gone. Start it again only if you still need its result.

Never generate synthetic host load in any session or subagent: no \`stress-ng\`, busy loops, \`--stress-count\`, or repeated full
test runs; cap the test's own container (\`--cpus\` sized to the test) instead.

The same limit can end this run too. Write what you have established into your review file as you go, marked not final, so that
what you found is on disk whenever the run ends. Take the time the work needs.
`;

/**
 What putting ended reviews back reads and answers.
 */
export type EndedReviewsPutBack = {
  /**
   The lines to print, one per review put back and one per review left alone for its own reason.
   */
  readonly lines: readonly string[];

  /**
   The names of the reviews put back, in the order they were put back.
   */
  readonly putBack: readonly string[];
};

/**
 Reads one run's summary and when it was written.

 @param file - the summary's path

 @returns what it says and when it was last written, or nothing when it is not there

 @throws the error a read gave, other than a summary that is not there

 @example
 ```ts
 const summary = await summaryOfARunIn({ file: '/runs/a-1/summary.json' });
 ```
 */
async function summaryOfARunIn({ file, }: { readonly file: string },): Promise<{
  readonly shown: string;
  readonly endedAt: number;
} | typeof NOT_FINISHED_YET> {
  try {
    /**
     What the summary says, and when it was written.
     */
    const [shown, written,] = await Promise.all([
      readFile(
        file,
        'utf8',
      ),
      stat(file,),
    ],);
    return {
      shown,
      endedAt: written.mtimeMs,
    };
  }
  catch (error) {
    // A run still going has no summary, which is the answer that puts nothing back for it.
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return NOT_FINISHED_YET;
    }
    throw error;
  }
}

/**
 One review's newest attempt: its name, and the folder that attempt wrote.
 */
type NewestAttemptOfAReview = {
  /**
   The review run's name.
   */
  readonly name: string;

  /**
   The folder the attempt wrote.
   */
  readonly runFolder: string;
};

/**
 Reads the newest attempt of every review run in the runs folder.

 @param runsFolder - folder the driver writes one run folder per run into

 @returns one entry per review name, the folder of its newest attempt

 @example
 ```ts
 const newest = await newestAttemptOfEachReview({ runsFolder: '/runs' });
 ```
 */
export async function newestAttemptOfEachReview({ runsFolder, }: { readonly runsFolder: string },): Promise<readonly {
  readonly name: string;
  readonly runFolder: string;
}[]> {
  /**
   The newest attempt of each review name found so far.
   */
  const newest = new Map<string, {
    readonly attempt: number;
    readonly runFolder: string;
  }>();
  for (const entry of await entriesOrNothingIn(runsFolder,)) {
    if (!entry.startsWith(REVIEW_RUN_PREFIX,)) {
      continue;
    }
    /**
     Where the attempt's number begins.
     */
    const separator = entry.lastIndexOf('-',);
    if (separator <= 0) {
      continue;
    }
    /**
     The attempt's number, as the folder writes it.
     */
    const written = entry.slice(separator + 1,);
    if (!isDigitsOnly(written,)) {
      continue;
    }
    /**
     The review's name.
     */
    const name = entry.slice(
      0,
      separator,
    );
    /**
     The attempt's number.
     */
    const attempt = Number(written,);
    /**
     The newest attempt of that name read so far.
     */
    const known = newest.get(name,);
    if ((known === undefined) || (attempt > known.attempt)) {
      newest.set(
        name,
        {
          attempt,
          runFolder: entry,
        },
      );
    }
  }
  /**
   The newest attempt of each review name, as a list.
   */
  const newestAttempts: NewestAttemptOfAReview[] = [];
  for (const [name, attempt,] of newest) {
    newestAttempts.push({
      name,
      runFolder: attempt.runFolder,
    },);
  }
  return newestAttempts;
}

/**
 Says whether a review is marked final, which is a review a run must not be started again for.

 @param reviewsFolder - folder that holds one folder per review

 @param name - the review run's name

 @returns whether its review file is there and marked final

 @example
 ```ts
 const final = await reviewIsFinal({ reviewsFolder: '/reviews', name: 'review-a' });
 ```
 */
async function reviewIsFinal({
  reviewsFolder,
  name,
}: {
  readonly reviewsFolder: string;
  readonly name: string;
},): Promise<boolean> {
  /**
   The review's file.
   */
  const file = reviewFileOfAReview({
    reviewsFolder,
    review: name.slice(REVIEW_RUN_PREFIX.length,),
  },);
  if (!(await isThere(file,))) {
    return false;
  }
  /**
   The head of the file, in one case, where the marking is looked for.
   */
  const head = (await readFile(
    file,
    {
      encoding: 'utf8',
      flag: 'r',
    },
  )).slice(
    0,
    REVIEW_HEAD_CHARACTERS,
  )
    .toLowerCase();
  return head.includes('final',) && (!head.includes('not final',))
    && (!head.includes('not-final',));
}

/**
 Writes the prompt a review put back after the rate limit reads, through a file beside it that is renamed over it.

 @param runsFolder - folder the driver writes its runs into

 @returns the prompt's path

 @example
 ```ts
 const prompt = await rateLimitReviewPromptIn({ runsFolder: '/runs' });
 ```
 */
async function rateLimitReviewPromptIn({ runsFolder, }: { readonly runsFolder: string },): Promise<string> {
  /**
   The prompt's path.
   */
  const file = join(
    runsFolder,
    RATE_LIMIT_REVIEW_PROMPT_FILE_NAME,
  );
  /**
   The file written beside it and renamed over it.
   */
  const beside = `${file}.being-written-${String(process.pid,)}`;
  await writeFile(
    beside,
    RATE_LIMIT_REVIEW_PROMPT,
  );
  await rename(
    beside,
    file,
  );
  return file;
}

/**
 Puts the review runs the rate limit ended back at the head of the queue.

 @param queueFile - the queue's path

 @param runsFolder - folder the driver writes its runs into

 @param reviewsFolder - folder that holds one folder per review

 @param since - the earliest end a run is put back from, as milliseconds since the epoch

 @param going - the runs going now, by name

 @param now - where this reads the clock

 @returns the lines to print and the reviews put back

 @example
 ```ts
 const putBack = await putEndedReviewsBack({ queueFile: '/runs/q.json', runsFolder: '/runs', reviewsFolder: '/reviews', since: Date.now() - 28_800_000, going: new Set() });
 ```
 */
export async function putEndedReviewsBack({
  queueFile,
  runsFolder,
  reviewsFolder,
  since,
  going,
  now,
}: {
  readonly queueFile: string;
  readonly runsFolder: string;
  readonly reviewsFolder: string;
  readonly since: number;
  readonly going: ReadonlySet<string>;
  readonly now: () => number;
},): Promise<EndedReviewsPutBack> {
  /**
   The lines to print.
   */
  const lines: string[] = [];
  /**
   The queue as it stands, which the put-backs add to one at a time.
   */
  const queue = await readQueueOfRuns({ queueFile, },);
  /**
   Where a put-back review goes: after the integration passes waiting at the head.
   */
  const firstNotAPass = queue.findIndex(function isNoPass(entry,) {
    return !entry.name
      .startsWith(INTEGRATION_PASS_PREFIX,);
  },);
  /**
   How many entries stay ahead of a put-back review.
   */
  const headLength = (firstNotAPass === (-1)) ? queue.length : firstNotAPass;
  /**
   The reviews put back, in the order they were put back.
   */
  const putBack: string[] = [];
  for (const review of await newestAttemptOfEachReview({ runsFolder, },)) {
    /**
     The run's summary.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- one review run after another, each reading its own summary
    const summary = await summaryOfARunIn({
      file: join(
        runsFolder,
        review.runFolder,
        SUMMARY_FILE_NAME,
      ),
    },);
    if ((summary === NOT_FINISHED_YET) || (summary.endedAt < since)) {
      continue;
    }
    if ((summaryIn({ shown: summary.shown, },)
      .endReason
      !== RATE_LIMIT_END_REASON) || (!summary.shown
        .includes(QUOTA_REFUSAL_WORD,))) {
      continue;
    }
    if (going.has(review.name,)) {
      continue;
    }
    // oxlint-disable-next-line eslint/no-await-in-loop -- the review file of the review just read
    if (await reviewIsFinal({
      reviewsFolder,
      name: review.name,
    },)) {
      lines.push(`${new Date(now(),).toISOString()} ${review.runFolder} ended on the rate limit with its review marked final: not put back`,);
      continue;
    }
    /**
     The prompt the review just put back reads.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- the prompt of the review just put back
    const prompt = await rateLimitReviewPromptIn({ runsFolder, },);
    /**
     The entry that goes on from the run's own session.
     */
    const entry: QueueRecord = {
      name: review.name,
      branch: branchOfAReview({ review: review.name
        .slice(REVIEW_RUN_PREFIX.length,), },),
      sessionOf: review.runFolder,
      prompt,
      provider: REVIEWING_PROVIDER,
      model: REVIEWING_MODEL,
      wall: REVIEW_WALL_SECONDS,
    };
    // The queue is read again at the moment of writing, so an entry another session queued meanwhile is not written away, and a
    // review already queued is refused rather than queued twice.
    try {
      // oxlint-disable-next-line eslint/no-await-in-loop -- the put-back of the review just read
      await putAQueueEntryIn({
        queueFile,
        entry,
        position: headLength + putBack.length,
      },);
    }
    catch (error) {
      lines.push(`${new Date(now(),).toISOString()} ${review.name} not put back: ${String(error,)}`,);
      continue;
    }
    putBack.push(review.name,);
    lines.push(`${new Date(now(),).toISOString()} put back at the head of the queue, to go on from its session: ${review.runFolder}`,);
  }
  return {
    lines,
    putBack,
  };
}

/**
 Reads the end a run must have after to be put back.

 @param now - the moment the filling reads the clock

 @returns the earliest end, as milliseconds since the epoch

 @example
 ```ts
 const since = putBackSince({ now: Date.now() });
 ```
 */
export function putBackSince({ now, }: { readonly now: number },): number {
  return now - (REVIEWS_PUT_BACK_LOOKBACK_SECONDS * MILLISECONDS_PER_SECOND);
}
