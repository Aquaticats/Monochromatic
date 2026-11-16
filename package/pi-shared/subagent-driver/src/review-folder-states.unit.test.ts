/**
 The review-folder states a start can meet, against every command that starts or continues a review: the reviews folder absent, empty
 or already holding generations; a record absent, empty, whole, cut off at every byte a kill can leave, or still being written; and
 change names drawn from what Git allows in a branch name, slashes included.

 The control is generated: every name is run against every state, and the rows of the table below are what a failure names. It was
 written before the fixes and fails on this commit for the missing folder, the nested name and the truncated record.

 @module
 */

import {
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { dirname, join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  NO_RECORD,
  NO_REVIEWED_FOLDER,
  REVIEW_BRANCH_PREFIX,
  branchOfAReview,
  runNameOfAReview,
  unitNameOfGiven,
  changeNameOfBranch,
  recordFileOfAReview,
  recordIn,
  recordTextOf,
  reviewFileOfAReview,
  reviewFolderOf,
  startReviews,
  takeTheNextReviewFolder,
  theReviewOfABranch,
} from '../dist/final/node/index.mjs';

/**
 The change names every state is run against, as Git allows them in a branch name: a plain name, nested names, dots, underscores,
 hyphens and an at sign.
 */
const NAMES: readonly string[] = [
  'shared-change',
  'subsystem/fix-bug',
  'a/b/c',
  'release.2026.11',
  'with_underscore',
  'with-hyphen',
  'at@sign',
];

/**
 The states a reviews folder and a record can be in, each named as the row names it.
 */
const STATES: readonly string[] = [
  'reviews-folder-absent',
  'reviews-folder-empty',
  'record-absent',
  'record-empty',
  'record-whole',
  'record-being-written',
  'review-written',
];

/**
 A folder the test throws away.
 */
async function makeFolder(): Promise<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  /**
   The folder.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-review-states-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 The branch a change name is reviewed from, which keeps every segment of a nested name.
 */
function branchOf({ name, }: { readonly name: string },): string {
  return `fix/${name}`;
}

/**
 Says whether a text is a whole JSON document already, which a cut can be when it stops just before trailing whitespace.

 @param text - the text to read

 @returns whether it is whole JSON

 @example
 ```ts
 const whole = isWholeJson({ text: '{"commit":"abc"}', },);
 ```
 */
function isWholeJson({ text, }: { readonly text: string },): boolean {
  try {
    JSON.parse(text,);
    return true;
  }
  catch (error) {
    if (error instanceof SyntaxError) {
      return false;
    }
    throw error;
  }
}

/**
 The whole record of one change, as a start leaves it.
 */
function recordTextOfAChange({ name, }: { readonly name: string },): string {
  return recordTextOf({
    branch: branchOf({ name, },),
    commit: 'a'.repeat(40),
    ranges: [`main...${branchOf({ name, },)}`,],
    source: 'src',
    claimedAt: '2026-11-15T00:00:00.000Z',
  },);
}

/**
 Makes the reviews folder and the folder of one review, which a state needs.

 @param reviewsFolder - where the reviews live

 @param name - the review name the folder carries
 */
async function makeTheReviewFolder({
  reviewsFolder,
  name,
}: {
  readonly reviewsFolder: string;
  readonly name: string;
},): Promise<string> {
  const folder = reviewFolderOf({ reviewsFolder, review: name, },);
  await mkdir(dirname(folder,), { recursive: true, },);
  await mkdir(folder, { recursive: true, },);
  return folder;
}

/**
 Runs the lookups of one state and names every row that does not answer as the state allows.

 @param name - the change name the row is for

 @param state - the state the folder and its record are in

 @returns the rows that failed, each naming the name and the state
 */
async function rowsOfOneState({
  name,
  state,
}: {
  readonly name: string;
  readonly state: string;
},): Promise<readonly string[]> {
  await using folder = await makeFolder();
  const reviewsFolder = join(folder.path, 'reviews',);
  const branch = branchOf({ name, },);
  const recordFile = recordFileOfAReview({ reviewsFolder, review: name, },);
  const reviewFile = reviewFileOfAReview({ reviewsFolder, review: name, },);
  if (state !== 'reviews-folder-absent') {
    await mkdir(reviewsFolder, { recursive: true, },);
  }
  if ((state === 'record-absent') || (state === 'record-empty') || (state === 'record-whole')
    || (state === 'record-being-written') || (state === 'review-written')) {
    await makeTheReviewFolder({ reviewsFolder, name, },);
  }
  if ((state === 'record-empty') || (state === 'record-whole') || (state === 'review-written')) {
    await writeFile(recordFile, (state === 'record-empty') ? '' : recordTextOfAChange({ name, },), 'utf8',);
  }
  if (state === 'record-being-written') {
    await writeFile(
      `${recordFile}.${String(process.pid,)}.a-temporary-file.tmp`,
      recordTextOfAChange({ name, },),
      'utf8',
    );
  }
  if (state === 'review-written') {
    await writeFile(reviewFile, 'the review\n', 'utf8',);
  }
  /**
   The rows this state failed, by what was asked.
   */
  const failed: string[] = [];
  /**
   Records a row that did not answer as it must.
   */
  function note(question: string, detail: string,): void {
    failed.push(`${name} / ${state} / ${question}: ${detail}`);
  }
  /**
   What the record reader answered, which must never throw and must say no record for every incomplete state.
   */
  let read: unknown;
  let readThrew = false;
  try {
    read = await recordIn({ recordFile, },);
  }
  catch (error) {
    readThrew = true;
    note('recordIn', `threw ${String(error,)}`,);
  }
  if (!readThrew) {
    if ((state === 'record-whole') || (state === 'review-written')) {
      if ((typeof read) === 'symbol') {
        note('recordIn', 'said no record for a whole record',);
      }
    }
    else if (read !== NO_RECORD) {
      note('recordIn', 'found a record where no record is the only answer',);
    }
  }
  /**
   What the starter's lookup answered, which must never throw and must find only the review that wrote its file.
   */
  let reviewed: unknown;
  let lookupThrew = false;
  try {
    reviewed = await theReviewOfABranch({ reviewsFolder, review: name, branch, },);
  }
  catch (error) {
    lookupThrew = true;
    note('theReviewOfABranch', `threw ${String(error,)}`,);
  }
  if (!lookupThrew) {
    if (state === 'review-written') {
      if (reviewed !== name) {
        note('theReviewOfABranch', `answered ${String(reviewed,)} where the review is ${name}`,);
      }
    }
    else if (reviewed !== NO_REVIEWED_FOLDER) {
      note('theReviewOfABranch', `answered ${String(reviewed,)} where no review is written`,);
    }
  }
  /**
   What the folder writer answered, which must make a folder for every name and leave a taken one alone.
   */
  try {
    const taken = await takeTheNextReviewFolder({ reviewsFolder, review: name, },);
    const expected = folderExistsInState({ state, },)
      ? `${name}-2`
      : name;
    if (taken.review !== expected) {
      note('takeTheNextReviewFolder', `took ${taken.review} where ${expected} is the first free name`,);
    }
    if (taken.folder !== reviewFolderOf({ reviewsFolder, review: expected, },)) {
      note('takeTheNextReviewFolder', `answered folder ${taken.folder} for ${expected}`,);
    }
  }
  catch (error) {
    note('takeTheNextReviewFolder', `threw ${String(error,)}`,);
  }
  // The folder the review state uses is the same path the writer takes for the base name, so the check above has made it.
  if (state === 'reviews-folder-absent') {
    const taken = await takeTheNextReviewFolder({ reviewsFolder, review: name, },);
    if (taken.review !== `${name}-2`) {
      note('takeTheNextReviewFolder', `took ${taken.review} where the second call must take ${name}-2`,);
    }
  }
  return failed;
}

/**
 Says whether a state already made the review folder, so the next name is a generation.
 */
function folderExistsInState({ state, }: { readonly state: string },): boolean {
  return (state !== 'reviews-folder-absent') && (state !== 'reviews-folder-empty');
}

/**
 Runs `startReviews` for one branch in one state.

 @param name - the change name

 @param state - the state of the reviews folder and the record

 @returns the rows that failed
 */
async function rowsOfOneBatch({
  name,
  state,
}: {
  readonly name: string;
  readonly state: string;
},): Promise<readonly string[]> {
  await using folder = await makeFolder();
  const reviewsFolder = join(folder.path, 'reviews',);
  if (state !== 'reviews-folder-absent') {
    await mkdir(reviewsFolder, { recursive: true, },);
  }
  if ((state === 'record-whole') || (state === 'review-written')) {
    await makeTheReviewFolder({ reviewsFolder, name, },);
    await writeFile(recordFileOfAReview({ reviewsFolder, review: name, },), recordTextOfAChange({ name, },), 'utf8',);
  }
  if (state === 'record-empty') {
    await makeTheReviewFolder({ reviewsFolder, name, },);
    await writeFile(recordFileOfAReview({ reviewsFolder, review: name, },), '', 'utf8',);
  }
  if (state === 'review-written') {
    await writeFile(reviewFileOfAReview({ reviewsFolder, review: name, },), 'the review\n', 'utf8',);
  }
  const branch = branchOf({ name, },);
  let briefCalls = 0;
  let lines: readonly string[] = [];
  const failed: string[] = [];
  try {
    const started = await startReviews({
      branches: [branch,],
      integration: 'main',
      runsFolder: join(folder.path, 'runs',),
      checkoutFolder: folder.path,
      briefFileOf({ review, },) {
        return join(folder.path, 'unused', review, 'brief.md',);
      },
      seams: {
        reviewOfAChange({ review, branch: askedBranch, },) {
          return theReviewOfABranch({ reviewsFolder, review, branch: askedBranch, },);
        },
        async branchTip() {
          return 'a'.repeat(40);
        },
        async integrationMerges() {
          return [];
        },
        async brief() {
          briefCalls += 1;
          return { written: false, line: 'a line for the brief', };
        },
        async start() {
          throw new Error('start must not be reached',);
        },
        async save() {
          throw new Error('save must not be reached',);
        },
        async wait() {},
      },
    },);
    ({ lines, } = started);
  }
  catch (error) {
    failed.push(`${name} / ${state} / startReviews: threw ${String(error,)}`,);
  }
  if (state === 'review-written') {
    if (briefCalls !== 0) {
      failed.push(`${name} / ${state} / startReviews: wrote ${String(briefCalls,)} briefs for a written review`,);
    }
    if (!lines.some(function saysAlreadyReviewed(line,) {
      return line.endsWith(': already reviewed',);
    },)) {
      failed.push(`${name} / ${state} / startReviews: said no line that the review is written`,);
    }
  }
  else if (briefCalls !== 1) {
    failed.push(`${name} / ${state} / startReviews: reached the brief ${String(briefCalls,)} times where one branch was asked for`,);
  }
  return failed;
}

/**
 Runs the state rows, one test per name and state, each naming its row.
 */
const rows: { readonly name: string; readonly state: string }[] = [];
for (const name of NAMES) {
  for (const state of STATES) {
    rows.push({ name, state, },);
  }
}

await describe({
  name: '',
  children: [
    describe({
      name: 'the names a run, a branch and a unit carry',
      children: NAMES.map(function oneName(name,) {
        return it({
          name,
          fn: async () => {
            /**
             The run name the review gets.
             */
            const run = runNameOfAReview({ review: name, },);
            expect(run,).not.toContain('/',);
            /**
             The branch a review works on, whose only slash is the one the package puts between `test` and the review.
             */
            const branch = branchOfAReview({ review: name, },);
            expect(branch.startsWith(REVIEW_BRANCH_PREFIX,),).toBe(true,);
            expect(branch.slice(REVIEW_BRANCH_PREFIX.length,),).not.toContain('/',);
            expect(unitNameOfGiven({ name: run, },),).not.toContain('/',);
          },
        },);
      },),
    },),
    describe({
      name: 'a record cut off at every byte a kill can leave',
      children: NAMES.map(function oneName(name,) {
        return it({
          name,
          fn: async () => {
            await using folder = await makeFolder();
            const reviewsFolder = join(folder.path, 'reviews',);
            const recordFile = recordFileOfAReview({ reviewsFolder, review: name, },);
            const branch = branchOf({ name, },);
            const whole = recordTextOfAChange({ name, },);
            const failed: string[] = [];
            /* oxlint-disable eslint/no-await-in-loop -- one cut is written, read and looked up, then the next; each row is the
               state of one byte boundary. See doc/lint/sequential-awaits.md. */
            for (let length = 1; length < whole.length; length += 1) {
              await mkdir(dirname(recordFile,), { recursive: true, },);
              /**
               The text a kill leaves when it stops after this many bytes.
               */
              const cut = whole.slice(0, length,);
              await writeFile(recordFile, cut, 'utf8',);
              /**
               Whether the cut happens to be a whole JSON document already, which is a whole record and not a torn one.
               */
              const wholeAlready = isWholeJson({ text: cut, },);
              try {
                /**
                 What the record reader made of the cut.
                 */
                const read = await recordIn({ recordFile, },);
                if (wholeAlready ? ((typeof read) === 'symbol') : (read !== NO_RECORD)) {
                  failed.push(
                    `${name} / cut at ${String(length,)}: recordIn ${wholeAlready ? 'said no record for a whole document' : 'found a record where no record is the only answer'}`,
                  );
                }
              }
              catch (error) {
                failed.push(`${name} / cut at ${String(length,)}: recordIn threw ${String(error,)}`,);
              }
              try {
                if ((await theReviewOfABranch({ reviewsFolder, review: name, branch, },)) !== NO_REVIEWED_FOLDER) {
                  failed.push(`${name} / cut at ${String(length,)}: theReviewOfABranch found a review`,);
                }
              }
              catch (error) {
                failed.push(`${name} / cut at ${String(length,)}: theReviewOfABranch threw ${String(error,)}`,);
              }
            }
            /* oxlint-enable eslint/no-await-in-loop */
            expect(failed.join('\n',),).toBe('',);
          },
        },);
      },),
    },),
    describe({
      name: 'the lookups a start makes over every folder and record state',
      children: rows.map(function oneRow(row,) {
        return it({
          name: `${row.name} / ${row.state}`,
          fn: async () => {
            expect((await rowsOfOneState(row,)).join('\n',),).toBe('',);
            expect((await rowsOfOneBatch(row,)).join('\n',),).toBe('',);
          },
        },);
      },),
    },),
  ],
},);
