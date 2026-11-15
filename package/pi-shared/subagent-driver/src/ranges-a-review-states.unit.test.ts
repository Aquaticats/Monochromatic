/**
 The ranges a review states, read byte for byte from the line its brief gives it to copy: for every byte Git allows in a reference
 name, at the start, in the middle and at the end of a branch, the brief's own line for a branch of that name is read back as the
 same range and the review covers its change.

 Git decides which bytes those are: every case here asks `git check-ref-format`, so the set tested is Git's own and not a list kept
 beside this test. A reference may hold a comma, a semicolon, an underscore or a backtick, which is why a range is read from the
 code span its brief writes it in rather than by splitting the line on punctuation.

 @module
 */

import { spawnSync, } from 'node:child_process';
import { dirname, join, } from 'node:path';
import { fileURLToPath, } from 'node:url';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  REVIEW_COMPLETE_LINE,
  type ReviewReading,
  answerOfAReview,
  rangesInALine,
  statementIn,
} from '../dist/final/node/index.mjs';

/**
 The first byte a reference name can be made of, which is the space every printable run starts after.
 */
const FIRST_PRINTABLE_BYTE = 0x20;

/**
 The first byte after the printable run, which is not a byte a brief writes in a range's line.
 */
const AFTER_LAST_PRINTABLE_BYTE = 0x7F;

/**
 The places a byte is tried at in a branch name: the start, the middle and the end.
 */
const PLACES: readonly string[] = [
  'start',
  'middle',
  'end',
];

/**
 The bytes a reference name may not hold anywhere, which calibrate the set Git accepts.

 Space, `~`, `^`, `:`, `?`, `*`, `[` and `\` are each refused by `git check-ref-format`, so a test that found them accepted would be
 asking its own list rather than Git.
 */
const BYTES_A_REFERENCE_MAY_NOT_HOLD: readonly string[] = [
  ' ',
  '~',
  '^',
  ':',
  '?',
  '*',
  '[',
  '\\',
];

/**
 The branch a byte is tried in, on one side of the change every range here is read against.
 */
const BRANCH_FOLDER = 'fix';

/**
 How many failing cases a failure names, so a control that went wrong reads as the cases it broke rather than as a count.
 */
const CASES_NAMED_IN_A_FAILURE = 10;

/**
 Names a byte by its code point, so a failure speaks of U+00A0 rather than of a character a reader cannot see.

 @param byte - the byte to name

 @returns its code point, as `U+00A0`

 @example
 ```ts
 const name = byteNameOf({ byte: '\u00a0', },);
 ```
 */
function byteNameOf({ byte, }: { readonly byte: string },): string {
  /**
   The byte's code point, which is one number for every byte this file tries.
   */
  const point = byte.codePointAt(0,) ?? 0;
  return `U+${point.toString(16,)
    .toUpperCase()
    .padStart(4, '0',)}`;
}

/**
 Writes the cases that failed as one sentence, naming the first few.

 @param failures - every case that did not come back as it was written

 @returns the sentence a test asserts is empty

 @example
 ```ts
 const summary = failuresNamed({ failures: ['"_" at middle: read back...',], },);
 ```
 */
function failuresNamed({ failures, }: { readonly failures: readonly string[] },): string {
  if (failures.length === 0) {
    return '';
  }
  /**
   The failing cases the sentence names, joined.
   */
  const named = failures.slice(0, CASES_NAMED_IN_A_FAILURE,)
    .join('; ');
  return `${String(failures.length,)} case${(failures.length === 1) ? '' : 's'} failed: ${named}`;
}

/**
 Reads the top of the checkout the test runs in, which every question about a reference name is asked in: this repository's Git
 wrapper answers only at the top.

 @returns the checkout's top folder

 @example
 ```ts
 const root = theCheckoutsTop();
 ```
 */
function theCheckoutsTop(): string {
  return join(
    import.meta.dirname,
    '..',
    '..',
    '..',
    '..',
  );
}

/**
 Asks Git whether a reference name is one it accepts.

 @param name - the reference name, from the top of the ref namespace

 @returns whether `git check-ref-format` accepted it

 @throws Error when Git could not be started at all

 @example
 ```ts
 const accepted = gitAccepts({ name: 'refs/heads/fix/a_b', },);
 ```
 */
function gitAccepts({ name, }: { readonly name: string },): boolean {
  /**
   What Git answered.
   */
  const answered = spawnSync(
    'git',
    [
      '-C',
      theCheckoutsTop(),
      'check-ref-format',
      name,
    ],
    { encoding: 'utf8', },
  );
  // A Git that could not be started tells nothing about the name, and a test that read its silence as a refusal would prove
  // nothing.
  if (answered.error !== undefined) {
    throw answered.error;
  }
  return answered.status === 0;
}

/**
 Writes the branch a byte is tried in, with the byte at one place of its name.

 @param byte - the byte at that place

 @param place - where the byte stands: `start`, `middle` or `end`

 @returns the branch's name, which is the byte in one of those places between ordinary letters

 @example
 ```ts
 const branch = branchNameOf({ byte: '_', place: 'middle', },);
 ```
 */
function branchNameOf({
  byte,
  place,
}: {
  readonly byte: string;
  readonly place: string;
}): string {
  if (place === 'start') {
    return `${BRANCH_FOLDER}/${byte}ab`;
  }
  if (place === 'middle') {
    return `${BRANCH_FOLDER}/a${byte}b`;
  }
  return `${BRANCH_FOLDER}/ab${byte}`;
}

/**
 Writes the two lines a brief asks a review to copy, for the ranges it was given.

 @param ranges - the ranges the brief named

 @returns the review's own text, with the ranges as the brief writes them

 @example
 ```ts
 const shown = reviewStating({ ranges: ['main...fix/a',], },);
 ```
 */
function reviewStating({ ranges, }: { readonly ranges: readonly string[] },): string {
  return `Not final and not approved.\n\n${REVIEW_COMPLETE_LINE}\n\nRanges reviewed: `
    + `${rangesInALine({ ranges, },)}\nFiles read in full: 1 of 1\n\nNo concrete defects found.\n`;
}

/**
 Writes what a review of those ranges was given.

 @param ranges - the ranges the brief named

 @returns the record beside the brief, as the brief writer keeps it

 @example
 ```ts
 const reading = readingOf({ ranges: ['main...fix/a',], },);
 ```
 */
function readingOf({ ranges, }: { readonly ranges: readonly string[] },): ReviewReading {
  return {
    review: 'a-round-1',
    commit: 'abc1234',
    ranges,
    source: 'package/a/src',
    files: ['package/a/src/gate.rs',],
  };
}

/**
 Says whether a review that copies the brief's line for one range covers its change, and reads the range back.

 @param range - the range the brief named

 @returns the failure this case is, or empty text when it holds

 @example
 ```ts
 const failure = failureOfOneRange({ range: 'main...fix/a_b', },);
 ```
 */
function failureOfOneRange({ range, }: { readonly range: string },): string {
  /**
   The review's own text, copied from the brief.
   */
  const shown = reviewStating({ ranges: [range,], },);
  /**
   The ranges read back out of it.
   */
  const stated = statementIn({ shown, },).ranges ?? [];
  if ((stated.length !== 1) || (stated[0] !== range)) {
    return `it read ${JSON.stringify(stated,)} back, not ${JSON.stringify([range,],)}`;
  }
  if (!answerOfAReview({
    shown,
    reading: readingOf({ ranges: [range,], },),
    readingFile: '/reviews/a-round-1/what-the-review-read.json',
  },).covers) {
    return 'the review did not cover the change it was given';
  }
  return '';
}

/**
 Says whether a review that copies the brief's line for two ranges covers its change, and reads both back.

 @param ranges - the two ranges the brief named

 @returns the failure this case is, or empty text when it holds

 @example
 ```ts
 const failure = failureOfTwoRanges({ ranges: ['main...fix/a', 'main...fix/b',], },);
 ```
 */
function failureOfTwoRanges({ ranges, }: { readonly ranges: readonly string[] },): string {
  /**
   The review's own text, copied from the brief.
   */
  const shown = reviewStating({ ranges, },);
  /**
   The ranges read back out of it.
   */
  const stated = statementIn({ shown, },).ranges ?? [];
  if ((stated.length !== ranges.length) || (!ranges.every(function isTheSame(range, at,) {
    return stated[at] === range;
  },))) {
    return `it read ${JSON.stringify(stated,)} back, not ${JSON.stringify(ranges,)}`;
  }
  if (!answerOfAReview({
    shown,
    reading: readingOf({ ranges, },),
    readingFile: '/reviews/a-round-1/what-the-review-read.json',
  },).covers) {
    return 'the review did not cover the change it was given';
  }
  return '';
}

await describe({
  name: '',
  children: [
    describe({
      name: 'the ranges a review states',
      children: [
        it({
          name: 'asks Git which bytes a reference may hold, and refuses the ones it refuses, so the set it accepts is Git\'s own',
          fn: async () => {
            for (const byte of BYTES_A_REFERENCE_MAY_NOT_HOLD) {
              expect({
                byte,
                accepted: gitAccepts({ name: `refs/heads/fix/a${byte}b`, },),
              },).toEqual({
                byte,
                accepted: false,
              },);
            }
            for (const byte of ['_', ',', ';', '`', '.', '-', '/', '@', '{', '}', '(', ')', '"', "'", '<', '>', '|', ']',
              '!', '#', '$', '%', '&', '+', '=', '0', 'A', 'z',]) {
              expect({
                byte,
                accepted: gitAccepts({ name: `refs/heads/fix/a${byte}b`, },),
              },).toEqual({
                byte,
                accepted: true,
              },);
            }
          },
        },),
        it({
          name: 'reads the brief\'s line for a branch of any byte Git allows, at the start, in the middle and at the end of its name, and the review covers its change',
          fn: async () => {
            /**
             Every case that did not come back as it was written.
             */
            const failures: string[] = [];
            /**
             How many cases Git accepted, which shows the test was not empty.
             */
            let cases = 0;
            for (let code = FIRST_PRINTABLE_BYTE; code < AFTER_LAST_PRINTABLE_BYTE; code += 1) {
              /**
               The byte at this code.
               */
              const byte = String.fromCodePoint(code,);
              for (const place of PLACES) {
                /**
                 The branch this byte is tried in.
                 */
                const branch = branchNameOf({
                  byte,
                  place,
                },);
                if (!gitAccepts({ name: `refs/heads/${branch}`, },)) {
                  continue;
                }
                cases += 1;
                /**
                 What went wrong for this case, if anything.
                 */
                const failure = failureOfOneRange({ range: `main...${branch}`, },);
                if (failure !== '') {
                  failures.push(`${byteNameOf({ byte, },)} at ${place}: ${failure}`);
                }
              }
            }
            expect(failuresNamed({ failures, },),).toBe('',);
            expect(cases,).toBeGreaterThan(0,);
          },
        },),
        it({
          name: 'reads two ranges from two code spans beside each other for a branch of any byte Git allows, so a comma or a semicolon cannot split one range in two',
          fn: async () => {
            /**
             Every case that did not come back as it was written.
             */
            const failures: string[] = [];
            for (let code = FIRST_PRINTABLE_BYTE; code < AFTER_LAST_PRINTABLE_BYTE; code += 1) {
              /**
               The byte at this code.
               */
              const byte = String.fromCodePoint(code,);
              /**
               The branch this byte is tried in.
               */
              const branch = branchNameOf({
                byte,
                place: 'middle',
              },);
              if (!gitAccepts({ name: `refs/heads/${branch}`, },)) {
                continue;
              }
              /**
               The two ranges the brief would name.
               */
              const ranges = [`main...${branch}`, `main...${branch}z`,];
              /**
               What went wrong for this case, if anything.
               */
              const failure = failureOfTwoRanges({ ranges, },);
              if (failure !== '') {
                failures.push(`${byteNameOf({ byte, },)}: ${failure}`);
              }
            }
            expect(failuresNamed({ failures, },),).toBe('',);
          },
        },),
        it({
          name: 'reads the brief\'s line for a branch of a byte above ASCII that Git allows, at the start, in the middle and at the end of its name',
          fn: async () => {
            // A non-breaking space is a byte `String.trim()` calls space and Git allows inside a reference: a range ending in
            // one is copied from the brief byte for byte, and nothing inside the code span may be cut.
            /**
             The bytes above ASCII that Git allows in a branch name, which this test asks Git about first.
             */
            const bytes = ['\u00A0', '\u00AD',];
            /**
             Every case that did not come back as it was written.
             */
            const failures: string[] = [];
            /**
             How many cases Git accepted, which shows the test was not empty.
             */
            let cases = 0;
            for (const byte of bytes) {
              for (const place of PLACES) {
                /**
                 The branch this byte is tried in.
                 */
                const branch = branchNameOf({
                  byte,
                  place,
                },);
                // A byte Git refuses is not this test's case; the proof above asks Git for the ASCII ones, and this asks for
                // these, so a byte Git would refuse cannot be read as covered here.
                if (!gitAccepts({ name: `refs/heads/${branch}`, },)) {
                  continue;
                }
                cases += 1;
                /**
                 What went wrong for this case, if anything.
                 */
                const failure = failureOfOneRange({ range: `main...${branch}`, },);
                if (failure !== '') {
                  failures.push(`${byteNameOf({ byte, },)} at ${place}: ${failure}`);
                }
              }
            }
            expect(failuresNamed({ failures, },),).toBe('',);
            expect(cases,).toBeGreaterThan(0,);
          },
        },),
        it({
          name: 'can fail: a review that states a range with a byte Git refuses in place of a letter does not cover its change, and neither does one an underscore lighter',
          fn: async () => {
            // The control that shows the two tests above can fail: the comparison reads every byte, so a range that differs from
            // the recorded one by a byte Git refuses, or by the underscore the parser used to remove, is not the range it was
            // given.
            /**
             Every byte Git refuses that the comparison still read as covering.
             */
            const failures: string[] = [];
            for (const byte of BYTES_A_REFERENCE_MAY_NOT_HOLD) {
              if (answerOfAReview({
                shown: reviewStating({ ranges: [`main...fix/a${byte}b`,], },),
                reading: readingOf({ ranges: ['main...fix/ab',], },),
                readingFile: '/reviews/a-round-1/what-the-review-read.json',
              },).covers) {
                failures.push(byteNameOf({ byte, },),);
              }
            }
            expect(failures,).toEqual([],);
            expect(answerOfAReview({
              shown: reviewStating({ ranges: ['main...fix/ab',], },),
              reading: readingOf({ ranges: ['main...fix/a_b',], },),
              readingFile: '/reviews/a-round-1/what-the-review-read.json',
            },).covers,).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
