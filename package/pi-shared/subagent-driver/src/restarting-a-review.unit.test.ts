/**
 Writing and starting a review again: the files one restart writes, the refusal of any path that already holds somebody's text, the
 guard that keeps a third run from being written under a second that has not ended, and the one place a start happens, handed a fake
 launcher.

 Every fixture is a folder the test makes and throws away; no run, unit or Pi is started.

 @module
 */

import {
  mkdir,
  mkdtemp,
  readFile,
  readdir,
  rm,
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
  CHANGE_DIFF_FILE_NAME,
  REVIEW_FILE_NAME,
  REVIEW_TURN_CAP,
  REVIEW_WALL_SECONDS,
  REVIEW_READING_FILE_NAME,
  type ReviewCheck,
  aRestartIsPending,
  baseReviewNameOf,
  restartNumberIn,
  restartOfAReview,
  restartPlanOf,
  restartReviewNameOf,
  runWordOf,
  startRestarts,
  writeRestartPlan,
} from '../dist/final/node/index.mjs';

/**
 The change every fixture reviews.

 @returns the review's name

 @example
 ```ts
 const review = aReview();
 ```
 */
function aReview(): string {
  return 'a-round-1';
}

/**
 The commit every fixture reviews.

 @returns the commit's id

 @example
 ```ts
 const commit = aCommit();
 ```
 */
function aCommit(): string {
  return 'abc1234def5678';
}

/**
 The diff every fixture review was given.

 @returns the diff's text, touching two files

 @example
 ```ts
 const diff = aDiff();
 ```
 */
function aDiff(): string {
  return [
    'diff --git a/package/a/src/one.rs b/package/a/src/one.rs',
    '--- a/package/a/src/one.rs',
    '+++ b/package/a/src/one.rs',
    '@@ -1 +1 @@',
    '-old',
    '+new',
    'diff --git a/package/a/src/two.rs b/package/a/src/two.rs',
    '--- a/package/a/src/two.rs',
    '+++ b/package/a/src/two.rs',
    '@@ -1 +1 @@',
    '-old',
    '+new',
    '',
  ].join('\n',);
}

/**
 A reviews folder, a runs folder and the earlier review's own files, all under one folder the test throws away.
 */
type Fixture = {
  /**
   The folder everything is written under.
   */
  readonly root: string;

  /**
   The folder that holds one folder per review.
   */
  readonly reviewsFolder: string;

  /**
   The folder the runs live in.
   */
  readonly runsFolder: string;

  /**
   How to remove the whole folder.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Makes a fixture folder with what the earlier review left.

 @param verdict - the earlier review's verdict

 @param endReason - why the earlier run's summary says it ended

 @returns the fixture

 @example
 ```ts
 await using fixture = await makeFixture({ verdict: 'ran nothing', endReason: 'idle' });
 ```
 */
async function makeFixture({
  verdict,
  endReason = 'idle',
}: {
  readonly verdict: ReviewCheck['verdict'];
  readonly endReason?: string;
},): Promise<Fixture> {
  const root = await mkdtemp(join(scratchRoot(), 'pi-review-restart-',),);
  const reviewsFolder = join(root, 'reviews',);
  const runsFolder = join(root, 'runs',);
  const reviewFolder = join(reviewsFolder, aReview(),);
  await mkdir(join(reviewFolder, 'reproductions',), { recursive: true, },);
  await mkdir(join(runsFolder, `review-${aReview()}-1`,), { recursive: true, },);
  await writeFile(join(reviewFolder, CHANGE_DIFF_FILE_NAME,), aDiff(),);
  await writeFile(
    join(reviewFolder, REVIEW_READING_FILE_NAME,),
    `${JSON.stringify({
      review: aReview(),
      commit: aCommit(),
      source: 'package/a/src',
      ranges: ['main...fix/a',],
    },)}\n`,
  );
  await writeFile(
    join(runsFolder, `review-${aReview()}-1`, 'summary.json',),
    JSON.stringify({
      endReason,
      turns: 40,
      sessionStats: { sessionFile: join(root, 'session.jsonl',), },
    },),
  );
  if (verdict !== 'no review file') {
    await writeFile(
      join(reviewFolder, REVIEW_FILE_NAME,),
      '# Review: a change\n\nA finding.\n',
    );
  }
  return {
    root,
    reviewsFolder,
    runsFolder,
    async [Symbol.asyncDispose]() {
      await rm(
        root,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

/**
 What the check says about the earlier review, as the restart's own input.

 @param verdict - the earlier review's verdict

 @param filesNotReadWhole - the files it did not read whole

 @returns the check

 @example
 ```ts
 const check = aCheck({ verdict: 'ran nothing' });
 ```
 */
function aCheck({
  verdict,
  filesNotReadWhole = ['package/a/src/two.rs',],
}: {
  readonly verdict: ReviewCheck['verdict'];
  readonly filesNotReadWhole?: readonly string[];
},): ReviewCheck {
  return {
    review: aReview(),
    verdict,
    reasons: ['a reason',],
    reviewFile: `/reviews/${aReview()}/review.md`,
    run: `review-${aReview()}-1`,
    endReason: 'idle',
    endedByItself: true,
    coverage: {
      diffLines: aDiff()
        .split('\n',)
        .length - 1,
      linesRead: 2,
      filesReadWhole: ['package/a/src/one.rs',],
      filesNotReadWhole,
    },
    builtPrograms: [],
    statement: {
      commit: aCommit(),
      diff: {
        lines: aDiff()
          .split('\n',)
          .length - 1,
        linesRead: 2,
      },
      files: [
        {
          path: 'package/a/src/one.rs',
          read: 'whole',
        },
        {
          path: 'package/a/src/two.rs',
          read: 'part',
        },
      ],
      commands: [],
      findings: [],
    },
  };
}

/**
 The words of a folder's entries.

 @param folder - the folder to list

 @returns its entries' names

 @example
 ```ts
 const names = await entriesIn('/reviews');
 ```
 */
async function entriesIn(folder: string,): Promise<readonly string[]> {
  return await readdir(folder,);
}

await describe({
  name: '',
  children: [
    describe({
      name: restartOfAReview.name,
      children: [
        it({
          name: 'writes the brief, the diff, the reading and the batch list of a review that ran nothing, reading the whole diff again',
          fn: async () => {
            await using fixture = await makeFixture({ verdict: 'ran nothing', },);
            const outcome = await restartOfAReview({
              check: aCheck({ verdict: 'ran nothing', },),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
              rulesFile: '/repo/rules.md',
              focus: 'data loss',
            },);
            expect(outcome.kind,).toBe('written',);
            const plan = (outcome.kind === 'written') ? outcome.plan : undefined;
            expect(plan?.restartReview,).toBe(`${aReview()}-restart-2`,);
            expect(await entriesIn(fixture.reviewsFolder),).toEqual([(aReview()), `${aReview()}-restart-2`,],);
            const brief = await readFile(plan?.briefFile ?? '', 'utf8',);
            expect(brief,).toContain('This is the second run of the review',);
            expect(brief,).toContain('Its check answered `ran nothing`.',);
            expect(brief,).toContain('It ran 0 command(s) through a program built from commit',);
            expect(brief,).toContain(`Its own review file is at \`${join(
              fixture.reviewsFolder,
              aReview(),
              REVIEW_FILE_NAME,
            )}\``,);
            // The whole diff is read again, because the earlier run left no complete reading of it.
            expect(brief,).toContain('package/a/src/one.rs',);
            expect(brief,).toContain('package/a/src/two.rs',);
            const list = await readFile(plan?.batchFile ?? '', 'utf8',);
            expect(JSON.parse(list,),).toHaveLength(1,);
          },
        },),
        it({
          name: 'writes, for a review that read part of its diff, a diff of only the files it did not read whole',
          fn: async () => {
            await using fixture = await makeFixture({ verdict: 'complete', },);
            const outcome = await restartOfAReview({
              check: aCheck({ verdict: 'complete', },),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
              rulesFile: '/repo/rules.md',
              focus: 'data loss',
            },);
            expect(outcome.kind,).toBe('written',);
            const plan = (outcome.kind === 'written') ? outcome.plan : undefined;
            const diff = await readFile(plan?.diffFile ?? '', 'utf8',);
            expect(diff,).toContain('package/a/src/two.rs',);
            expect(diff,).not.toContain('package/a/src/one.rs',);
            const brief = await readFile(plan?.briefFile ?? '', 'utf8',);
            expect(brief,).toContain('it did not read whole: package/a/src/two.rs',);
            expect(brief,).not.toContain('- `package/a/src/one.rs`',);
          },
        },),
        it({
          name: 'writes nothing for a review that read every file whole, saying a restart is not needed',
          fn: async () => {
            await using fixture = await makeFixture({ verdict: 'complete', },);
            const outcome = await restartOfAReview({
              check: aCheck({
                verdict: 'complete',
                filesNotReadWhole: [],
              },),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
              rulesFile: '/repo/rules.md',
              focus: 'data loss',
            },);
            expect(outcome.kind,).toBe('not needed',);
            expect(await entriesIn(fixture.reviewsFolder),).toEqual([(aReview()),],);
          },
        },),
        it({
          name: 'writes no later run while the run written before it is still pending, and says which one is',
          fn: async () => {
            await using fixture = await makeFixture({ verdict: 'ran nothing', },);
            const first = await restartOfAReview({
              check: aCheck({ verdict: 'ran nothing', },),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
              rulesFile: '/repo/rules.md',
              focus: 'data loss',
            },);
            expect(first.kind,).toBe('written',);
            const again = await restartOfAReview({
              check: aCheck({ verdict: 'ran nothing', },),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
              rulesFile: '/repo/rules.md',
              focus: 'data loss',
            },);
            expect(again.kind,).toBe('already written',);
            expect((again.kind === 'already written') ? again.line : '',).toContain('has not ended, so no later run was written');
            expect(await entriesIn(fixture.reviewsFolder),).toEqual([(aReview()), `${aReview()}-restart-2`,],);
          },
        },),
        it({
          name: 'writes the third run, saying so, once the second has ended',
          fn: async () => {
            await using fixture = await makeFixture({ verdict: 'ran nothing', },);
            const first = await restartOfAReview({
              check: aCheck({ verdict: 'ran nothing', },),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
              rulesFile: '/repo/rules.md',
              focus: 'data loss',
            },);
            expect(first.kind,).toBe('written',);
            // The second run ends, and its own check is the one the third is planned from.
            const secondFolder = join(fixture.reviewsFolder, `${aReview()}-restart-2`,);
            await writeFile(join(secondFolder, REVIEW_FILE_NAME,), '# Review: a change\n\nNothing.\n',);
            const third = await restartOfAReview({
              check: {
                ...aCheck({ verdict: 'ran nothing', },),
                review: `${aReview()}-restart-2`,
              },
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
              rulesFile: '/repo/rules.md',
              focus: 'data loss',
            },);
            expect(third.kind,).toBe('written',);
            expect((third.kind === 'written') ? third.plan.restartReview : '',).toBe(`${aReview()}-restart-3`,);
            const brief = await readFile((third.kind === 'written') ? third.plan.briefFile : '', 'utf8',);
            expect(brief,).toContain('This is the third run of the review',);
          },
        },),
        it({
          name: 'cannot be planned when the earlier review recorded nothing it read',
          fn: async () => {
            await using fixture = await makeFixture({ verdict: 'ran nothing', },);
            await rm(join(
              fixture.reviewsFolder,
              aReview(),
              REVIEW_READING_FILE_NAME,
            ),);
            const outcome = await restartOfAReview({
              check: aCheck({ verdict: 'ran nothing', },),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
              rulesFile: '/repo/rules.md',
              focus: 'data loss',
            },);
            expect(outcome.kind,).toBe('cannot',);
            expect((outcome.kind === 'cannot') ? outcome.line : '',).toContain('names no commit, ranges and source folder');
          },
        },),
      ],
    },),
    describe({
      name: writeRestartPlan.name,
      children: [
        it({
          name: 'keeps a brief that already holds other text, naming it, and writes none of the plan\'s other files',
          fn: async () => {
            await using fixture = await makeFixture({ verdict: 'ran nothing', },);
            const planned = await restartPlanOf({
              check: aCheck({ verdict: 'ran nothing', },),
              number: 2,
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
              rulesFile: '/repo/rules.md',
              focus: 'data loss',
            },);
            const {plan} = planned;
            if (plan === undefined) {
              return;
            }
            // Somebody's brief is already at the path, as an earlier round left it, and none of the plan was written.
            const kept = '# a brief nobody may replace\n';
            await mkdir(join(fixture.reviewsFolder, plan.restartReview,), { recursive: true, },);
            await writeFile(plan.briefFile, kept,);
            const written = await writeRestartPlan({ plan, },);
            expect(written.refused,).toContain(plan.briefFile,);
            expect(written.refused,).toContain('already there with other text',);
            expect(await readFile(plan.briefFile, 'utf8',),).toBe(kept,);
            // The refusal was decided before any file was written, so the plan's other files are not there.
            expect(
              await entriesIn(join(fixture.reviewsFolder, plan.restartReview,),),
            ).toEqual(['brief.md',],);
          },
        },),
        it({
          name: 'refuses the whole plan when the last file it would write already holds other text, leaving the first one unwritten',
          fn: async () => {
            await using fixture = await makeFixture({ verdict: 'ran nothing', },);
            const planned = await restartPlanOf({
              check: aCheck({ verdict: 'ran nothing', },),
              number: 2,
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
              rulesFile: '/repo/rules.md',
              focus: 'data loss',
            },);
            const {plan} = planned;
            if (plan === undefined) {
              return;
            }
            // A batch list somebody else wrote is in the way of this plan's own.
            const kept = '[{"name": "another run"}]\n';
            await mkdir(join(fixture.reviewsFolder, plan.restartReview,), { recursive: true, },);
            await writeFile(plan.batchFile, kept,);
            const written = await writeRestartPlan({ plan, },);
            expect(written.refused,).toContain(plan.batchFile,);
            expect(await readFile(plan.batchFile, 'utf8',),).toBe(kept,);
            expect(
              await entriesIn(join(fixture.reviewsFolder, plan.restartReview,),),
            ).toEqual([],);
          },
        },),
        it({
          name: 'refuses a restart whose review file is already there, so a run that already read the change is not restarted',
          fn: async () => {
            await using fixture = await makeFixture({ verdict: 'ran nothing', },);
            const outcome = await restartOfAReview({
              check: aCheck({ verdict: 'ran nothing', },),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
              rulesFile: '/repo/rules.md',
              focus: 'data loss',
            },);
            expect(outcome.kind,).toBe('written',);
            const plan = (outcome.kind === 'written') ? outcome.plan : undefined;
            if (plan === undefined) {
              return;
            }
            const ran = join(
              fixture.reviewsFolder,
              plan.restartReview,
              REVIEW_FILE_NAME,
            );
            await writeFile(ran, '# a review that ran\n',);
            const written = await writeRestartPlan({ plan, },);
            expect(written.refused,).toContain('already ran',);
            expect(await readFile(ran, 'utf8',),).toBe('# a review that ran\n',);
          },
        },),
        it({
          name: 'leaves a file that already holds exactly the plan\'s own text alone, so a plan written twice says the same thing',
          fn: async () => {
            await using fixture = await makeFixture({ verdict: 'ran nothing', },);
            const outcome = await restartOfAReview({
              check: aCheck({ verdict: 'ran nothing', },),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
              rulesFile: '/repo/rules.md',
              focus: 'data loss',
            },);
            const plan = (outcome.kind === 'written') ? outcome.plan : undefined;
            if (plan === undefined) {
              return;
            }
            const again = await writeRestartPlan({ plan, },);
            expect(again.refused,).toBeUndefined();
            expect(await readFile(plan.briefFile, 'utf8',),).toBe(plan.texts.find(function isTheBrief(target,) {
              return target.path === plan.briefFile;
            },)?.text,);
          },
        },),
      ],
    },),
    describe({
      name: aRestartIsPending.name,
      children: [
        it({
          name: 'says a written restart is pending until its review file exists, and not after',
          fn: async () => {
            await using fixture = await makeFixture({ verdict: 'ran nothing', },);
            const restartReview = `${aReview()}-restart-2`;
            expect(await aRestartIsPending({
              restartReview,
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
            },),).toBe(false,);
            await restartOfAReview({
              check: aCheck({ verdict: 'ran nothing', },),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
              rulesFile: '/repo/rules.md',
              focus: 'data loss',
            },);
            expect(await aRestartIsPending({
              restartReview,
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
            },),).toBe(true,);
            await writeFile(
              join(fixture.reviewsFolder, restartReview, REVIEW_FILE_NAME,),
              '# Review\n',
            );
            expect(await aRestartIsPending({
              restartReview,
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
            },),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: startRestarts.name,
      children: [
        it({
          name: 'hands each restart to the launcher it is given, once, with the words that name its run, branch, commit and brief',
          fn: async () => {
            await using fixture = await makeFixture({ verdict: 'ran nothing', },);
            const outcome = await restartOfAReview({
              check: aCheck({ verdict: 'ran nothing', },),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
              rulesFile: '/repo/rules.md',
              focus: 'data loss',
            },);
            expect(outcome.kind,).toBe('written',);
            const plan = (outcome.kind === 'written') ? outcome.plan : undefined;
            if (plan === undefined) {
              return;
            }
            /**
             What the fake launcher was asked to start.
             */
            const started: { name: string; words: readonly string[] }[] = [];
            const lines = await startRestarts({
              plans: [plan,],
              runsFolder: fixture.runsFolder,
              checkoutFolder: '/repo',
              seams: {
                start: async function start(request,) {
                  started.push(request,);
                  return {
                    status: 0,
                    output: '',
                  };
                },
                wait: async function wait(milliseconds,) {
                  expect(milliseconds,).toBeGreaterThan(0,);
                },
                report: function report(line,) {
                  expect(line,).toContain('status 0',);
                },
              },
            },);
            expect(started,).toHaveLength(1,);
            expect(started[0]?.name,).toBe(plan.entry.name,);
            expect(started[0]?.words.join(' ',),).toContain(`--name=${plan.entry.name}`,);
            expect(started[0]?.words.join(' ',),).toContain(`--branch=${plan.entry.branch}`,);
            expect(started[0]?.words.join(' ',),).toContain(`--base=${aCommit()}`,);
            expect(started[0]?.words.join(' ',),).toContain(`--prompt=${plan.briefFile}`,);
            // A restart's own name has no finished run to size a turn cap from, and the review's own limits are what it gets.
            expect(started[0]?.words.join(' ',),).toContain(`--turn-cap=${String(plan.entry.turnCap,)}`,);
            expect(plan.entry.wall,).toBe(REVIEW_WALL_SECONDS,);
            expect(plan.entry.turnCap,).toBe(REVIEW_TURN_CAP,);
            expect(lines,).toHaveLength(1,);
            expect(lines[0],).toContain('status 0',);
            // The launcher is the only thing that ran: no run folder was made beside the earlier run's own.
            expect(await entriesIn(fixture.runsFolder),).toEqual([`review-${aReview()}-1`,],);
          },
        },),
        it({
          name: 'starts nothing when no restart was written',
          fn: async () => {
            /**
             How many starts a fake launcher would answer.
             */
            const started: string[] = [];
            const lines = await startRestarts({
              plans: [],
              runsFolder: '/runs',
              checkoutFolder: '/repo',
              seams: {
                start: async function start(request,) {
                  started.push(request.name,);
                  return {
                    status: 0,
                    output: '',
                  };
                },
                wait: async function wait() {
                  started.push('waited',);
                },
              },
            },);
            expect(started,).toEqual([],);
            expect(lines,).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: 'the names of a review\'s runs',
      children: [
        it({
          name: 'reads the review a restart answers, its own name, and the words its run is named by',
          fn: async () => {
            expect(baseReviewNameOf({ review: `${aReview()}-restart-2`, },),).toBe(aReview(),);
            expect(baseReviewNameOf({ review: `${aReview()}-restart-2-restart-3`, },),).toBe(aReview(),);
            expect(baseReviewNameOf({ review: aReview(), },),).toBe(aReview(),);
            expect(restartReviewNameOf({
              review: aReview(),
              number: 2,
            },),).toBe(`${aReview()}-restart-2`,);
            expect(restartNumberIn({ review: `${aReview()}-restart-2`, },),).toBe(2,);
            expect(runWordOf(2,),).toBe('second',);
            expect(runWordOf(9,),).toBe('run 9',);
          },
        },),
      ],
    },),
  ],
},);
