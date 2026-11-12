/**
 Starting one review per finished branch: the words the retrying start is given, the review a branch or its merge makes, the
 branches a review is not started for, and the line each start and save printed. Every seam is answered here by the test, so no
 unit, worktree or process is involved.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  BATCH_RETRY_PERIOD_SECONDS,
  BATCH_STAGGER_MILLISECONDS,
  REVIEWING_MODEL,
  REVIEWING_PROVIDER,
  REVIEW_START_DEADLINE_SECONDS,
  REVIEW_TURN_CAP,
  REVIEW_WALL_SECONDS,
  type BriefOfOneReview,
  entryLineOf,
  type ReviewStartingSeams,
  type StartOfOneReview,
  branchOfAReview,
  retryWordsOfAReview,
  runNameOfAReview,
  startReviews,
} from '../dist/final/node/index.mjs';

/**
 A change's name, which the expectations are built from.

 @returns the change's name

 @example
 ```ts
 const change = aChange();
 ```
 */
function aChange(): string {
  return 'rewrite-the-gate';
}

/**
 Another change's name, for a batch of two.

 @returns the change's name

 @example
 ```ts
 const change = anotherChange();
 ```
 */
function anotherChange(): string {
  return 'keep-the-fold';
}

/**
 The branch of one change.

 @param change - the change's name

 @returns the branch's name

 @example
 ```ts
 const branch = branchOf({ change: aChange(), });
 ```
 */
function branchOf({ change, }: { readonly change: string },): string {
  return `fix/${change}`;
}

/**
 The change a branch reviews, read without a regex because every branch here starts with its type.

 @param branch - the branch's name

 @returns the change's name

 @example
 ```ts
 const change = changeOfBranch({ branch: 'fix/a', });
 ```
 */
function changeOfBranch({ branch, }: { readonly branch: string },): string {
  /**
   The type that ends at the first slash.
   */
  const slash = branch.indexOf('/',);
  return (slash === (-1))
    ? branch
    : branch.slice(slash + 1,);
}

/**
 The commit a branch's tip is, in this file's answers.

 @param change - the change the commit belongs to

 @returns the commit's id

 @example
 ```ts
 const tip = aTip({ change: aChange(), });
 ```
 */
function aTip({ change, }: { readonly change: string },): string {
  return `${change}-tip`;
}

/**
 The integration branch a change is read against.

 @returns the branch's name

 @example
 ```ts
 const integration = anIntegrationBranch();
 ```
 */
function anIntegrationBranch(): string {
  return 'main';
}

/**
 The folder the driver writes its runs into.

 @returns the folder's path

 @example
 ```ts
 const folder = aRunsFolder();
 ```
 */
function aRunsFolder(): string {
  return '/runs';
}

/**
 The top of the checkout the worktrees are made in.

 @returns the checkout's path

 @example
 ```ts
 const folder = aCheckoutFolder();
 ```
 */
function aCheckoutFolder(): string {
  return '/repo';
}

/**
 What the seams were asked, in order.
 */
type Asked = {
  readonly starts: { readonly review: string; readonly words: readonly string[] }[];
  readonly saves: string[];
  readonly waits: number[];
  readonly briefs: { readonly review: string; readonly commit: string; readonly range: string }[];
};

/**
 Builds the seams a test answers, recording what they were asked.

 @param answers - what the brief, the start and the save answer, by change name

 @returns the seams and what they were asked

 @example
 ```ts
 const { seams, asked } = seamsAnswering({ briefs: new Map(), starts: new Map(), saves: new Map(), written: [], merges: [] });
 ```
 */
function seamsAnswering({
  briefs,
  starts,
  saves,
  written,
  merges,
}: {
  readonly briefs: ReadonlyMap<string, BriefOfOneReview>;
  readonly starts: ReadonlyMap<string, StartOfOneReview>;
  readonly saves: ReadonlyMap<string, { readonly line: string }>;
  readonly written: readonly string[];
  readonly merges: readonly string[];
}): { readonly seams: ReviewStartingSeams; readonly asked: Asked } {
  /**
   What the seams were asked, filled as they answer.
   */
  const asked: {
    starts: { review: string; words: readonly string[] }[];
    saves: string[];
    waits: number[];
    briefs: { review: string; commit: string; range: string }[];
  } = {
    starts: [],
    saves: [],
    waits: [],
    briefs: [],
  };
  return {
    asked,
    seams: {
      reviewIsWritten: async function reviewIsWritten({ review, },) {
        return written.includes(review,);
      },
      branchTip: async function branchTip({ branch, },) {
        return aTip({ change: changeOfBranch({ branch, },), },);
      },
      integrationMerges: async function integrationMerges() {
        return merges;
      },
      brief: async function brief({ review, commit, range, },) {
        asked.briefs.push({
          review,
          commit,
          range,
        },);
        return briefs.get(review,)
          ?? {
            written: false,
            line: `${runNameOfAReview({ review, },)}: no answer for this brief`,
          };
      },
      start: async function start({ review, words, },) {
        asked.starts.push({
          review,
          words,
        },);
        return starts.get(review,)
          ?? {
            output: '',
            status: 0,
          };
      },
      save: async function save({ review, },) {
        asked.saves.push(review,);
        return saves.get(review,)
          ?? { line: `${runNameOfAReview({ review, },)}: has its review.md`, };
      },
      wait: async function wait(milliseconds,) {
        asked.waits.push(milliseconds,);
      },
    },
  };
}

/**
 The request every start in this file is made with.

 @param change - the change to review

 @param seams - the seams to answer with

 @returns the request

 @example
 ```ts
 const request = aStartRequest({ change: aChange(), seams });
 ```
 */
function aStartRequest({
  changes,
  seams,
}: {
  readonly changes: readonly string[];
  readonly seams: ReviewStartingSeams;
}) {
  return {
    branches: changes.map(function branchOfOne(change,) {
      return branchOf({ change, },);
    },),
    integration: anIntegrationBranch(),
    runsFolder: aRunsFolder(),
    checkoutFolder: aCheckoutFolder(),
    briefFileOf: function briefFileOf({ review, }: { readonly review: string }): string {
      return `/reviews/${review}/brief.md`;
    },
    seams,
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: retryWordsOfAReview.name,
      children: [
        it({
          name: 'gives the retrying start its own bounds, then the launcher its run, branch, reviewed commit, caps, wall, turn cap and the reviewer\'s provider and model',
          fn: async () => {
            /**
             The words the retrying start would be given for one review.
             */
            const words = retryWordsOfAReview({
              review: aChange(),
              commit: aTip({ change: aChange(), },),
              briefFile: `/reviews/${aChange()}/brief.md`,
              runsFolder: aRunsFolder(),
              checkoutFolder: aCheckoutFolder(),
            },);
            expect(words.slice(0, words.indexOf('--',),),).toEqual([
              `--deadline-seconds=${String(REVIEW_START_DEADLINE_SECONDS,)}`,
              `--period-seconds=${String(BATCH_RETRY_PERIOD_SECONDS,)}`,
            ],);
            expect(words,).toContain(`--name=${runNameOfAReview({ review: aChange(), },)}`,);
            expect(words,).toContain(`--branch=${branchOfAReview({ review: aChange(), },)}`,);
            expect(words,).toContain(`--base=${aTip({ change: aChange(), },)}`,);
            expect(words,).toContain(`--prompt=/reviews/${aChange()}/brief.md`,);
            expect(words,).toContain(`--wall=${String(REVIEW_WALL_SECONDS,)}`,);
            expect(words,).toContain(`--turn-cap=${String(REVIEW_TURN_CAP,)}`,);
            expect(words,).toContain(`--provider=${REVIEWING_PROVIDER}`,);
            expect(words,).toContain(`--model=${REVIEWING_MODEL}`,);
            expect(words,).toContain(`--runs-folder=${aRunsFolder()}`,);
            expect(words,).toContain(`--checkout-folder=${aCheckoutFolder()}`,);
          },
        },),
      ],
    },),
    describe({
      name: startReviews.name,
      children: [
        it({
          name: 'says a review already written in one line and starts nothing for it',
          fn: async () => {
            /**
             The seams, with the review already written.
             */
            const { seams, asked } = seamsAnswering({
              briefs: new Map(),
              starts: new Map(),
              saves: new Map(),
              written: [aChange(),],
              merges: [],
            },);
            /**
             What the starter answered.
             */
            const started = await startReviews(aStartRequest({ changes: [aChange(),], seams, },),);
            expect(started.lines,).toEqual([`${runNameOfAReview({ review: aChange(), },)}: already reviewed`,],);
            expect(started.statuses,).toEqual([],);
            expect(asked.starts,).toEqual([],);
          },
        },),
        it({
          name: 'says a branch whose change leaves nothing to review in one line, and goes on with the rest',
          fn: async () => {
            /**
             The seams, with the first change leaving nothing to review.
             */
            const { seams, asked } = seamsAnswering({
              briefs: new Map([
                [aChange(), {
                  written: false,
                  line: `nothing to review under package/a/src in ${anIntegrationBranch()}...${branchOf({ change: aChange(), },)}`,
                },],
                [anotherChange(), { written: true, line: 'another brief', },],
              ],),
              starts: new Map(),
              saves: new Map(),
              written: [],
              merges: [],
            },);
            /**
             What the starter answered.
             */
            const started = await startReviews(aStartRequest({ changes: [aChange(), anotherChange(),], seams, },),);
            expect(started.lines[0],).toBe(`nothing to review under package/a/src in `
              + `${anIntegrationBranch()}...${branchOf({ change: aChange(), },)}`,);
            expect(asked.starts.map(function reviewOf(start,) {
              return start.review;
            },),).toEqual([anotherChange(),],);
            expect(started.statuses,).toEqual(['0',],);
          },
        },),
        it({
          name: 'reads the range of a branch the integration already merged from the merge that took it, and hands the reviewed commit to the brief',
          fn: async () => {
            /**
             The merge that took the branch's tip.
             */
            const merge = 'def456';
            /**
             The seams, whose integration holds the branch's tip.
             */
            const { seams, asked } = seamsAnswering({
              briefs: new Map([[aChange(), {
                written: true,
                line: `${runNameOfAReview({ review: aChange(), },)}: ${String(1,)} files to read`,
              },],]),
              starts: new Map(),
              saves: new Map(),
              written: [],
              merges: [`${merge} 789abc ${aTip({ change: aChange(), },)}`,],
            },);
            await startReviews(aStartRequest({ changes: [aChange(),], seams, },),);
            expect(asked.briefs,).toEqual([{
              review: aChange(),
              commit: aTip({ change: aChange(), },),
              range: `${merge}^1...${branchOf({ change: aChange(), },)}`,
            },],);
          },
        },),
        it({
          name: 'waits between one start and the next, so the reviews ask for a slot in the order its branches were given',
          fn: async () => {
            /**
             The seams, with a brief for each change.
             */
            const { seams, asked } = seamsAnswering({
              briefs: new Map([
                [aChange(), { written: true, line: 'a brief', },],
                [anotherChange(), { written: true, line: 'another brief', },],
              ],),
              starts: new Map(),
              saves: new Map(),
              written: [],
              merges: [],
            },);
            await startReviews(aStartRequest({ changes: [aChange(), anotherChange(),], seams, },),);
            expect(asked.waits,).toEqual([
              BATCH_STAGGER_MILLISECONDS,
              BATCH_STAGGER_MILLISECONDS,
            ],);
          },
        },),
        it({
          name: 'saves each review after its start ends, names a start that ended badly or could not be started, and answers every status',
          fn: async () => {
            /**
             A start that failed with a status.
             */
            const failedStatus = 1;
            /**
             The seams, whose first start failed and whose second could not be started.
             */
            const { seams, asked } = seamsAnswering({
              briefs: new Map([
                [aChange(), { written: true, line: 'a brief', },],
                [anotherChange(), { written: true, line: 'another brief', },],
              ],),
              starts: new Map([
                [aChange(), { output: 'why', status: failedStatus, },],
                [anotherChange(), { output: '', startProblem: 'no such command', },],
              ],),
              saves: new Map([
                [aChange(), { line: `${runNameOfAReview({ review: aChange(), },)}: saved its last message`, },],
                [anotherChange(), { line: `${runNameOfAReview({ review: anotherChange(), },)}: no finished run`, },],
              ],),
              written: [],
              merges: [],
            },);
            /**
             What the starter answered.
             */
            const started = await startReviews(aStartRequest({ changes: [aChange(), anotherChange(),], seams, },),);
            expect(asked.saves,).toEqual([aChange(), anotherChange(),],);
            expect(started.lines,).toContain(entryLineOf({
              name: runNameOfAReview({ review: aChange(), },),
              started: {
                output: 'why',
                status: failedStatus,
              },
            },),);
            expect(started.lines,).toContain(entryLineOf({
              name: runNameOfAReview({ review: anotherChange(), },),
              started: {
                output: '',
                startProblem: 'no such command',
              },
            },),);
            expect(started.lines,).toContain(`reviews ended with statuses ${String(failedStatus,)} none`,);
            expect(started.statuses,).toEqual([String(failedStatus,), 'none',],);
          },
        },),
      ],
    },),
  ],
},);
