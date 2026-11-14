/**
 Putting the reviews the provider's rate limit ended back at the head of the queue: only the newest attempt of a name whose summary
 names the quota answer, which no unit is going for, which the queue does not hold, and whose review is not marked final, and after the
 integration passes that keep their place at the head.

 @module
 */

import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
  stat,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  newestAttemptOfEachReview,
  putBackSince,
  putEndedReviewsBack,
  RATE_LIMIT_REVIEW_PROMPT,
  RATE_LIMIT_REVIEW_PROMPT_FILE_NAME,
  REVIEWS_PUT_BACK_LOOKBACK_SECONDS,
  readQueueOfRuns,
} from '../dist/final/node/index.mjs';

/**
 A folder a test may write in, with its runs, its reviews and its queue, removed when the test ends.
 */
type AThrowawayFolder = {
  /**
   The folder's path.
   */
  readonly path: string;

  /**
   The folder the runs live in.
   */
  readonly runs: string;

  /**
   The folder one folder per review goes in.
   */
  readonly reviews: string;

  /**
   The queue's path.
   */
  readonly queueFile: string;

  /**
   Removes the folder and everything under it.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 A folder a test may write in, with its runs, its reviews and its queue, removed when the test ends.

 @param entries - what the queue holds

 @returns the folder
 */
async function makeFolder(entries: readonly Record<string, unknown>[],): Promise<AThrowawayFolder> {
  /**
   The folder's path.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-put-back-',),);
  /**
   The folder the runs live in.
   */
  const runs = join(path, 'runs',);
  /**
   The folder one folder per review goes in.
   */
  const reviews = join(path, 'reviews',);
  await mkdir(runs, { recursive: true, },);
  await mkdir(reviews, { recursive: true, },);
  /**
   The queue's path.
   */
  const queueFile = join(path, 'queue.json',);
  await writeFile(
    queueFile,
    `${JSON.stringify(entries,)}\n`,
  );
  return {
    path,
    runs,
    reviews,
    queueFile,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Writes one finished review run's folder and summary.

 @param request - where the runs live, the run's folder and its attempt, and how it ended

 @returns the moment the summary was written
 */
async function writeFinishedReview({
  runs,
  name,
  attempt,
  quota = true,
}: {
  readonly runs: string;
  readonly name: string;
  readonly attempt: number;
  readonly quota?: boolean;
}): Promise<number> {
  /**
   The run's folder.
   */
  const folder = join(runs, `${name}-${String(attempt,)}`,);
  await mkdir(folder, { recursive: true, },);
  await writeFile(
    join(folder, 'summary.json',),
    `${JSON.stringify({
      endReason: 'model-error',
      lastAssistantText: quota
        ? '429 {"error":{"code":"insufficient_quota","message":"You exceeded your current quota"}}'
        : '429 {"error":{"message":"Too many requests"}}',
    },)}\n`,
  );
  return attempt;
}

await describe({
  name: '',
  children: [
    describe({
      name: newestAttemptOfEachReview.name,
      children: [
        it({
          name: 'names the newest attempt of every review run, and no other run folder',
          fn: async () => {
            await using folder = await makeFolder([],);
            await writeFinishedReview({
              runs: folder.runs,
              name: 'review-a',
              attempt: 1,
            },);
            await writeFinishedReview({
              runs: folder.runs,
              name: 'review-a',
              attempt: 2,
            },);
            await mkdir(join(folder.runs, 'fresh-run-1',), { recursive: true, },);
            expect(await newestAttemptOfEachReview({ runsFolder: folder.runs, },),).toEqual([{
              name: 'review-a',
              runFolder: 'review-a-2',
            },],);
          },
        },),
      ],
    },),
    describe({
      name: putBackSince.name,
      children: [
        it({
          name: 'bounds what comes back by the filling unit its own lifetime, so no older review is resurrected',
          fn: async () => {
            expect(REVIEWS_PUT_BACK_LOOKBACK_SECONDS,).toBe(28_800,);
            expect(putBackSince({ now: 100_000_000, },),).toBe(100_000_000 - (28_800 * 1_000),);
          },
        },),
      ],
    },),
    describe({
      name: putEndedReviewsBack.name,
      children: [
        it({
          name: 'puts a review the quota ended back at the head, after the integration pass waiting there, and writes the prompt it reads',
          fn: async () => {
            await using folder = await makeFolder([
              { name: 'integration-pass-1', branch: 'fix/pass', },
              { name: 'other', branch: 'feat/other', },
            ],);
            await writeFinishedReview({
              runs: folder.runs,
              name: 'review-a',
              attempt: 1,
            },);
            /**
             What putting the ended reviews back answered.
             */
            const putBack = await putEndedReviewsBack({
              queueFile: folder.queueFile,
              runsFolder: folder.runs,
              reviewsFolder: folder.reviews,
              since: 0,
              going: new Set(),
              now: Date.now,
            },);
            expect(putBack.putBack,).toEqual(['review-a',],);
            expect(putBack.lines.join('',),).toContain('put back at the head of the queue, to go on from its session: review-a-1',);
            /**
             The queue as it stands.
             */
            const queue = await readQueueOfRuns({ queueFile: folder.queueFile, },);
            expect(queue.map(function nameOf(entry,) {
              return entry.name;
            },),).toEqual(['integration-pass-1', 'review-a', 'other',],);
            expect(queue[1],).toMatchObject({
              name: 'review-a',
              branch: 'test/review-a',
              sessionOf: 'review-a-1',
              provider: 'qwen-token-plan-individual',
              model: 'qwen4-max-preview',
            },);
            expect(await readFile(join(folder.runs, RATE_LIMIT_REVIEW_PROMPT_FILE_NAME,), 'utf8',),).toBe(RATE_LIMIT_REVIEW_PROMPT,);
          },
        },),
        it({
          name: 'leaves alone a review whose ending was another model error, one that is going, one already queued, one marked final, and one older than the bound',
          fn: async () => {
            await using folder = await makeFolder([
              { name: 'review-queued', branch: 'test/review-queued', },
            ],);
            await writeFinishedReview({
              runs: folder.runs,
              name: 'review-other-error',
              attempt: 1,
              quota: false,
            },);
            await writeFinishedReview({
              runs: folder.runs,
              name: 'review-going',
              attempt: 1,
            },);
            await writeFinishedReview({
              runs: folder.runs,
              name: 'review-queued',
              attempt: 1,
            },);
            await writeFinishedReview({
              runs: folder.runs,
              name: 'review-final',
              attempt: 1,
            },);
            await mkdir(join(folder.reviews, 'final',), { recursive: true, },);
            await writeFile(
              join(folder.reviews, 'final', 'review.md',),
              'This review is final.\n',
            );
            await writeFinishedReview({
              runs: folder.runs,
              name: 'review-old',
              attempt: 1,
            },);
            /**
             The bound, taken as the moment after the last summary was written, so every summary here is older than it.
             */
            const bound = (await stat(join(folder.runs, 'review-old-1', 'summary.json',),)).mtimeMs + 1;
            /**
             What putting the ended reviews back answered.
             */
            const putBack = await putEndedReviewsBack({
              queueFile: folder.queueFile,
              runsFolder: folder.runs,
              reviewsFolder: folder.reviews,
              since: bound,
              going: new Set(['review-going',],),
              now: Date.now,
            },);
            // Every summary was written now, so the bound set to now leaves all of them out; the review already queued and the final
            // one are named for their own reasons only when the bound lets them through.
            expect(putBack.putBack,).toEqual([],);
            expect((await readQueueOfRuns({ queueFile: folder.queueFile, },)).map(function nameOf(entry,) {
              return entry.name;
            },),).toEqual(['review-queued',],);
          },
        },),
        it({
          name: 'names the final review it leaves alone once the bound lets it through',
          fn: async () => {
            await using folder = await makeFolder([],);
            await writeFinishedReview({
              runs: folder.runs,
              name: 'review-final',
              attempt: 1,
            },);
            await mkdir(join(folder.reviews, 'final',), { recursive: true, },);
            await writeFile(
              join(folder.reviews, 'final', 'review.md',),
              'Not final.\n',
            );
            /**
             What putting the ended reviews back answered.
             */
            const notFinal = await putEndedReviewsBack({
              queueFile: folder.queueFile,
              runsFolder: folder.runs,
              reviewsFolder: folder.reviews,
              since: 0,
              going: new Set(),
              now: Date.now,
            },);
            expect(notFinal.putBack,).toEqual(['review-final',],);
            await writeFile(
              join(folder.reviews, 'final', 'review.md',),
              'This review is final.\n',
            );
            /**
             What putting the same review back answered once its file was final.
             */
            const finalized = await putEndedReviewsBack({
              queueFile: folder.queueFile,
              runsFolder: folder.runs,
              reviewsFolder: folder.reviews,
              since: 0,
              going: new Set(),
              now: Date.now,
            },);
            expect(finalized.putBack,).toEqual([],);
            expect(finalized.lines.join('',),).toContain('marked final: not put back',);
          },
        },),
      ],
    },),
  ],
},);
