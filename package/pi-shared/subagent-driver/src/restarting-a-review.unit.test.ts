/**
 Writing and starting the second review a thin one is owed: the once-only rule, the other reviewer of the pair, a plan written and
 never started, a brief a kill cut short, the refusal of other text, and the start marker that keeps two checks from starting one
 review twice.

 Every fixture is a folder the test makes and throws away, and every start is a fake launcher, so no run, unit or Pi is started.

 @module
 */

import {
  spawn,
} from 'node:child_process';
import {
  once,
} from 'node:events';
import {
  mkdir,
  mkdtemp,
  readFile,
  readdir,
  rm,
  truncate,
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
  REVIEW_READING_FILE_NAME,
  REVIEW_TURN_CAP,
  REVIEW_WALL_SECONDS,
  REVIEWING_PEER,
  type ReviewerOfAChange,
  type ReviewCheck,
  processIsAlive,
  restartOfAReview,
  restartPlanOf,
  startRestarts,
  takeTheStart,
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
 A peer a Pi run can serve, which the shipped peer is not.

 @returns the reviewer

 @example
 ```ts
 const peer = aReachablePeer();
 ```
 */
function aReachablePeer(): ReviewerOfAChange {
  return {
    name: 'Claude Sonnet',
    provider: 'fixture-provider',
    model: 'fixture-model',
  };
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
 A reviews folder, a runs folder and the first review's own files, all under one folder the test throws away.
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
 Makes a fixture folder with what the first review left.

 @param endReason - why the first run's summary says it ended

 @returns the fixture

 @example
 ```ts
 await using fixture = await makeFixture({ endReason: 'idle' });
 ```
 */
async function makeFixture({
  endReason = 'idle',
}: {
  readonly endReason?: string;
} = {},): Promise<Fixture> {
  const root = await mkdtemp(join(scratchRoot(), 'pi-second-review-',),);
  const reviewsFolder = join(root, 'reviews',);
  const runsFolder = join(root, 'runs',);
  const reviewFolder = join(reviewsFolder, aReview(),);
  await mkdir(join(reviewFolder, 'reproductions',), { recursive: true, },);
  await mkdir(join(runsFolder, `review-${aReview()}-1`,), { recursive: true, },);
  await writeFile(join(reviewFolder, CHANGE_DIFF_FILE_NAME,), aDiff(),);
  await writeFile(join(reviewFolder, REVIEW_FILE_NAME,), '# Review: a change\n\nA finding.\n',);
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
  return {
    root,
    reviewsFolder,
    runsFolder,
    async [Symbol.asyncDispose]() {
      await rm(root, { recursive: true, force: true, },);
    },
  };
}

/**
 What the check says about a review, as the second review's own input.

 @param verdict - the review's verdict

 @param filesNotReadWhole - the files it did not read whole

 @param review - the review's name

 @returns the check

 @example
 ```ts
 const check = aCheck({ verdict: 'ran nothing' });
 ```
 */
function aCheck({
  verdict,
  filesNotReadWhole = ['package/a/src/two.rs',],
  review = aReview(),
}: {
  readonly verdict: ReviewCheck['verdict'];
  readonly filesNotReadWhole?: readonly string[];
  readonly review?: string;
},): ReviewCheck {
  return {
    review,
    verdict,
    reasons: ['a reason',],
    reviewFile: `/reviews/${review}/review.md`,
    run: `review-${review}-1`,
    endReason: 'idle',
    endedByItself: true,
    coverage: {
      unnamedNotWhole: false,
      diffLines: aDiff()
        .split('\n',)
        .length - 1,
      linesRead: 2,
      filesReadWhole: ['package/a/src/one.rs',],
      filesNotReadWhole,
    },
    builtPrograms: verdict === 'complete' ? ['/w/target/debug/a',] : [],
    thin: (verdict !== 'complete') || (filesNotReadWhole.length > 0),
    reviewer: {
      provider: 'qwen-token-plan-individual',
      model: 'qwen4-max-preview',
    },
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
 Reads a process number that is surely gone: a process this test starts, and waits for.

 @returns the number its process had

 @example
 ```ts
 const pid = await aPidThatIsGone();
 ```
 */
async function aPidThatIsGone(): Promise<number> {
  /**
   The process, which exits at once.
   */
  const child = spawn(
    process.execPath,
    ['--input-type=module', '-e', '',],
    { stdio: 'ignore', },
  );
  /**
   The number the process had.
   */
  const pid = child.pid ?? 0;
  await once(child, 'exit',);
  return pid;
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
function entriesIn(folder: string,): Promise<readonly string[]> {
  return readdir(folder,);
}

await describe({
  name: '',
  children: [
    describe({
      name: restartOfAReview.name,
      children: [
        //region Once, on the other model
        it({
          name: 'writes the second review of a thin first one for the other reviewer, and a start starts exactly one run on it',
          fn: async () => {
            await using fixture = await makeFixture();
            const check = aCheck({ verdict: 'ran nothing', },);
            const outcome = await restartOfAReview({
              check,
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
              rulesFile: '/repo/rules.md',
              focus: 'data loss',
              reviewer: aReachablePeer(),
            },);
            expect(outcome.kind,).toBe('written',);
            const plan = (outcome.kind === 'written') ? outcome.plan : undefined;
            if (plan === undefined) {
              return;
            }
            expect(plan.restartReview,).toBe(`${aReview()}-restart-2`,);
            expect(plan.entry?.provider,).toBe(aReachablePeer().provider,);
            expect(plan.entry?.model,).toBe(aReachablePeer().model,);
            expect(plan.entry?.turnCap,).toBe(REVIEW_TURN_CAP,);
            expect(plan.entry?.wall,).toBe(REVIEW_WALL_SECONDS,);
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
                wait: async function wait() {
                  // The stagger has nothing to wait for in a test whose launcher is fake.
                },
              },
            },);
            expect(started,).toHaveLength(1,);
            expect(started[0]?.words.join(' ',),).toContain(`--provider=${aReachablePeer().provider}`,);
            expect(started[0]?.words.join(' ',),).toContain(`--model=${aReachablePeer().model}`,);
            expect(started[0]?.words.join(' ',),).toContain(`--turn-cap=${String(REVIEW_TURN_CAP,)}`,);
            expect(lines,).toHaveLength(1,);
            expect(lines[0],).toContain('status 0',);
            // No run folder was made beside the first review's own: the launcher is a fake.
            expect(await entriesIn(fixture.runsFolder),).toEqual([`review-${aReview()}-1`,],);
          },
        },),
        it({
          name: 'writes nothing and starts nothing for a complete review that read every file whole and ran a build program',
          fn: async () => {
            await using fixture = await makeFixture();
            const outcome = await restartOfAReview({
              check: aCheck({
                verdict: 'complete',
                filesNotReadWhole: [],
              },),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
              rulesFile: '/repo/rules.md',
              focus: 'data loss',
              reviewer: aReachablePeer(),
            },);
            expect(outcome.kind,).toBe('not needed',);
            expect(await entriesIn(fixture.reviewsFolder),).toEqual([aReview(),],);
          },
        },),
        it({
          name: 'reports a second review that is still thin and starts no third',
          fn: async () => {
            await using fixture = await makeFixture();
            const outcome = await restartOfAReview({
              check: aCheck({
                verdict: 'ran nothing',
                review: `${aReview()}-restart-2`,
              },),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
              rulesFile: '/repo/rules.md',
              focus: 'data loss',
              reviewer: aReachablePeer(),
            },);
            expect(outcome.kind,).toBe('left thin',);
            expect((outcome.kind === 'left thin') ? outcome.line : '',).toContain('not started a third time');
            expect((outcome.kind === 'left thin') ? outcome.line : '',).toContain('left for whoever coordinates');
            expect(await entriesIn(fixture.reviewsFolder),).toEqual([aReview(),],);
          },
        },),
        it({
          name: 'starts no third review when the second one is already written, naming it instead',
          fn: async () => {
            await using fixture = await makeFixture();
            const first = await restartOfAReview({
              check: aCheck({ verdict: 'ran nothing', },),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
              rulesFile: '/repo/rules.md',
              focus: 'data loss',
              reviewer: aReachablePeer(),
            },);
            expect(first.kind,).toBe('written',);
            // The second review wrote its file.
            await writeFile(
              join(fixture.reviewsFolder, `${aReview()}-restart-2`, REVIEW_FILE_NAME,),
              '# Review: a change\n\nNothing.\n',
            );
            const again = await restartOfAReview({
              check: aCheck({ verdict: 'ran nothing', },),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
              rulesFile: '/repo/rules.md',
              focus: 'data loss',
              reviewer: aReachablePeer(),
            },);
            expect(again.kind,).toBe('already written',);
            expect((again.kind === 'already written') ? again.line : '',).toContain(`--review=${aReview()}-restart-2`);
            expect((again.kind === 'already written') ? again.line : '',).toContain('no review is started a third time');
          },
        },),
        //endregion Once, on the other model

        //region A plan written and not started
        it({
          name: 'gives a plan written and never started back to a later check, which starts it once',
          fn: async () => {
            await using fixture = await makeFixture();
            const first = await restartOfAReview({
              check: aCheck({ verdict: 'ran nothing', },),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
              rulesFile: '/repo/rules.md',
              focus: 'data loss',
              reviewer: aReachablePeer(),
            },);
            expect(first.kind,).toBe('written',);
            // The process that wrote the plan died before starting it: no run folder of the second review exists.
            const again = await restartOfAReview({
              check: aCheck({ verdict: 'ran nothing', },),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
              rulesFile: '/repo/rules.md',
              focus: 'data loss',
              reviewer: aReachablePeer(),
            },);
            expect(again.kind,).toBe('written',);
            if (again.kind !== 'written') {
              return;
            }
            /**
             How many starts a fake launcher answered.
             */
            const started: string[] = [];
            await startRestarts({
              plans: [again.plan,],
              runsFolder: fixture.runsFolder,
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
                  // The stagger has nothing to wait for in a test whose launcher is fake.
                },
              },
            },);
            expect(started,).toEqual([`review-${aReview()}-restart-2`,],);
          },
        },),
        it({
          name: 'repairs a brief a kill cut short, and writes the plan whole again',
          fn: async () => {
            await using fixture = await makeFixture();
            const first = await restartOfAReview({
              check: aCheck({ verdict: 'ran nothing', },),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
              rulesFile: '/repo/rules.md',
              focus: 'data loss',
              reviewer: aReachablePeer(),
            },);
            expect(first.kind,).toBe('written',);
            const briefFile = (first.kind === 'written') ? first.plan.briefFile : '';
            const whole = await readFile(briefFile, 'utf8',);
            await truncate(briefFile, Math.floor(whole.length / 2,),);
            const again = await restartOfAReview({
              check: aCheck({ verdict: 'ran nothing', },),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
              rulesFile: '/repo/rules.md',
              focus: 'data loss',
              reviewer: aReachablePeer(),
            },);
            expect(again.kind,).toBe('written',);
            expect(await readFile(briefFile, 'utf8',),).toBe(whole,);
          },
        },),
        //endregion A plan written and not started

        //region The other model unreachable
        it({
          name: 'writes the brief and the note a harness with the other model takes, and starts nothing, when Pi serves no such model',
          fn: async () => {
            await using fixture = await makeFixture();
            const outcome = await restartOfAReview({
              check: aCheck({ verdict: 'ran nothing', },),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
              rulesFile: '/repo/rules.md',
              focus: 'data loss',
              reviewer: REVIEWING_PEER,
            },);
            expect(outcome.kind,).toBe('owed',);
            const plan = (outcome.kind === 'owed') ? outcome.plan : undefined;
            if (plan === undefined) {
              return;
            }
            expect(plan.entry,).toBeUndefined();
            expect(plan.owedNoteFile,).toBeDefined();
            const note = await readFile(plan.owedNoteFile ?? '', 'utf8',);
            expect(note,).toContain('A second review is owed');
            expect(note,).toContain(REVIEWING_PEER.name,);
            expect(note,).toContain('no Pi model on this machine is');
            expect(note,).toContain('never a third');
            expect(await readFile(plan.briefFile, 'utf8',),).toContain('This is the second run of the review');
            expect(await entriesIn(fixture.reviewsFolder),).toEqual([aReview(), `${aReview()}-restart-2`,],);
          },
        },),
        //endregion The other model unreachable

        //region Its files
        it({
          name: 'writes, for a review that read part of its diff, a diff of only the files it did not read whole',
          fn: async () => {
            await using fixture = await makeFixture();
            const outcome = await restartOfAReview({
              check: aCheck({ verdict: 'complete', },),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
              rulesFile: '/repo/rules.md',
              focus: 'data loss',
              reviewer: aReachablePeer(),
            },);
            expect(outcome.kind,).toBe('written',);
            if (outcome.kind !== 'written') {
              return;
            }
            const {plan} = outcome;
            const diff = await readFile(plan.diffFile, 'utf8',);
            expect(diff,).toContain('package/a/src/two.rs',);
            expect(diff,).not.toContain('package/a/src/one.rs',);
            const brief = await readFile(plan.briefFile, 'utf8',);
            expect(brief,).toContain('it did not read whole: package/a/src/two.rs',);
            expect(brief,).not.toContain('- `package/a/src/one.rs`',);
          },
        },),
        it({
          name: 'cannot be planned when the first review recorded nothing it read',
          fn: async () => {
            await using fixture = await makeFixture();
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
              reviewer: aReachablePeer(),
            },);
            expect(outcome.kind,).toBe('cannot',);
            expect((outcome.kind === 'cannot') ? outcome.line : '',).toContain('names no commit, ranges and source folder');
          },
        },),
        //endregion Its files
      ],
    },),
    describe({
      name: 'the refusal of other text',
      children: [
        it({
          name: 'keeps a brief that already holds other text, naming it, and writes none of the plan\'s other files',
          fn: async () => {
            await using fixture = await makeFixture();
            const planned = await restartPlanOf({
              check: aCheck({ verdict: 'ran nothing', },),
              number: 2,
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
              rulesFile: '/repo/rules.md',
              focus: 'data loss',
              reviewer: aReachablePeer(),
            },);
            const {plan} = planned;
            if (plan === undefined) {
              return;
            }
            const kept = '# a brief nobody may replace\n';
            await mkdir(join(fixture.reviewsFolder, plan.restartReview,), { recursive: true, },);
            await writeFile(plan.briefFile, kept,);
            const outcome = await restartOfAReview({
              check: aCheck({ verdict: 'ran nothing', },),
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
              rulesFile: '/repo/rules.md',
              focus: 'data loss',
              reviewer: aReachablePeer(),
            },);
            expect(outcome.kind,).toBe('refused',);
            expect((outcome.kind === 'refused') ? outcome.line : '',).toContain('already there with other text');
            expect(await readFile(plan.briefFile, 'utf8',),).toBe(kept,);
            expect(
              await entriesIn(join(fixture.reviewsFolder, plan.restartReview,),),
            ).toEqual(['brief.md',],);
          },
        },),
        it({
          name: 'lets one of two concurrent writers of the same restart write, and never discards the other\'s text',
          fn: async () => {
            await using fixture = await makeFixture();
            /**
             A check whose plan differs from the fixture's own inputs, so the two writers' texts differ.
             */
            const other = {
              ...aCheck({ verdict: 'cut', },),
              endReason: 'wall-clock',
              endedByItself: false,
            };
            const ask = {
              reviewsFolder: fixture.reviewsFolder,
              runsFolder: fixture.runsFolder,
              rulesFile: '/repo/rules.md',
              focus: 'data loss',
              reviewer: aReachablePeer(),
            };
            const [one, two,] = await Promise.all([
              restartOfAReview({
                check: aCheck({ verdict: 'ran nothing', },),
                ...ask,
              },),
              restartOfAReview({ check: other, ...ask, },),
            ],);
            expect([one.kind, two.kind,].toSorted(),).toEqual(['refused', 'written',],);
            // What the winner wrote is whole, and the loser's text is not there to replace it.
            const briefFile = (one.kind === 'written') ? one.plan.briefFile : (two.kind === 'written') ? two.plan.briefFile : '';
            const brief = await readFile(briefFile, 'utf8',);
            expect(brief,).toContain('This is the second run of the review');
            expect(brief,).toContain('nobody can read what it found');
          },
        },),
      ],
    },),
    describe({
      name: 'the start marker',
      children: [
        it({
          name: 'lets one of two starts of one restart take it, and says who holds it to the other',
          fn: async () => {
            await using fixture = await makeFixture();
            const first = await takeTheStart({
              reviewsFolder: fixture.reviewsFolder,
              review: aReview(),
              number: 2,
            },);
            expect(first.kind,).toBe('mine',);
            const second = await takeTheStart({
              reviewsFolder: fixture.reviewsFolder,
              review: aReview(),
              number: 2,
            },);
            expect(second.kind,).toBe('somebody else',);
            expect((second.kind === 'somebody else') ? second.line : '',).toContain(`process ${String(process.pid,)}`);
          },
        },),
        it({
          name: 'is taken over once by a later check when the process that took it is gone',
          fn: async () => {
            await using fixture = await makeFixture();
            /**
             A process number that is surely gone, from a process this test started and waited for.
             */
            const pid = await aPidThatIsGone();
            await writeFile(
              join(fixture.reviewsFolder, aReview(), 'restart-2-started.json',),
              `${JSON.stringify({ pid, },)}\n`,
            );
            expect(processIsAlive(pid,),).toBe(false,);
            const taken = await takeTheStart({
              reviewsFolder: fixture.reviewsFolder,
              review: aReview(),
              number: 2,
            },);
            expect(taken.kind,).toBe('mine',);
          },
        },),
      ],
    },),
  ],
},);
