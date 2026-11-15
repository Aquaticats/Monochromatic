/**
 What a review's own file says it covered, read against what the review was given: the ranges its brief named, and the files that
 brief listed.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type ReviewCoverage,
  type ReviewReading,
  coverageOfAReview,
  sentenceOfCoverage,
  statementIn,
} from '../dist/final/node/index.mjs';

/**
 The file the record of what a review was given is read from, which every message names.

 @returns the file's path

 @example
 ```ts
 const file = theReadingFile();
 ```
 */
function theReadingFile(): string {
  return '/reviews/a-round-1/what-the-review-read.json';
}

/**
 The ranges every test here gives a review.

 @returns the ranges, as `git diff` takes them

 @example
 ```ts
 const ranges = theRangesGiven();
 ```
 */
function theRangesGiven(): readonly string[] {
  return ['main...fix/a', 'merge^1...fix/a',];
}

/**
 The files every test here gives a review's brief, from which a stated total is derived.

 @returns the files the brief listed

 @example
 ```ts
 const files = theFilesGiven();
 ```
 */
function theFilesGiven(): readonly string[] {
  return ['package/a/src/gate.rs', 'package/a/src/gate_second.rs',];
}

/**
 What a review was given, as the record beside its brief keeps it.

 @param files - the files the brief listed, or nothing for a record written before a brief listed them

 @returns what the review was given

 @example
 ```ts
 const reading = theReadingGiven({ files: theFilesGiven(), },);
 ```
 */
function theReadingGiven({ files, }: { readonly files?: readonly string[] },): ReviewReading {
  return {
    review: 'a-round-1',
    commit: 'abc1234',
    ranges: theRangesGiven(),
    source: 'package/a/src',
    ...((files === undefined) ? {} : { files, }),
  };
}

/**
 The text of a review's own file, with the two lines its brief asks for at its top.

 @param ranges - the ranges the review says it read

 @param filesRead - how many of the listed files it says it read

 @param filesListed - how many files it says the brief listed

 @returns the review's text

 @example
 ```ts
 const shown = aReviewStating({ ranges: theRangesGiven(), filesRead: 2, filesListed: 2, },);
 ```
 */
function aReviewStating({
  ranges,
  filesRead,
  filesListed,
}: {
  readonly ranges: readonly string[];
  readonly filesRead: number;
  readonly filesListed: number;
}): string {
  return `Not final.\n\nRanges reviewed: ${ranges.map(function inBackticks(range,) {
    return `\`${range}\``;
  },)
    .join(', ',)}\nFiles read in full: ${String(filesRead,)} of ${String(filesListed,)}\n\nNo concrete defects found.\n`;
}

/**
 Says what one review's own file is read as, given the text it holds and what it was given.

 @param shown - the review's own file

 @param reading - what the review was given, or nothing when no record can be read

 @returns what the review is read as

 @example
 ```ts
 const coverage = coverageOf({ shown: 'Not final.\n', reading: theReadingGiven({ files: theFilesGiven(), },), },);
 ```
 */
function coverageOf({
  shown,
  reading,
}: {
  readonly shown: string;
  readonly reading?: ReviewReading;
},): ReviewCoverage {
  return coverageOfAReview({
    shown,
    readingFile: theReadingFile(),
    ...((reading === undefined) ? {} : { reading, }),
  },);
}

await describe({
  name: '',
  children: [
    describe({
      name: statementIn.name,
      children: [
        it({
          name: 'reads the ranges and the count of files the two lines a brief asks for state',
          fn: async () => {
            expect(statementIn({
              shown: aReviewStating({
                ranges: theRangesGiven(),
                filesRead: theFilesGiven().length,
                filesListed: theFilesGiven().length,
              },),
            },),).toEqual({
              ranges: theRangesGiven(),
              filesRead: theFilesGiven().length,
              filesTotal: theFilesGiven().length,
            },);
          },
        },),
        it({
          name: 'reads both lines whatever markdown decorates them, because a review file is written by hand',
          fn: async () => {
            expect(statementIn({
              shown: '**Not final.**\n\n- **Ranges reviewed:** `main...fix/a`\n- Files read in full: 1 of 2\n',
            },),).toEqual({
              ranges: ['main...fix/a',],
              filesRead: 1,
              filesTotal: 2,
            },);
          },
        },),
        it({
          name: 'states nothing for a review whose file carries neither line',
          fn: async () => {
            expect(statementIn({ shown: 'Not final.\n\nNo concrete defects found.\n', },),).toEqual({},);
          },
        },),
        it({
          name: 'states no range for a label a review left empty, rather than reading the label as a range',
          fn: async () => {
            expect(statementIn({ shown: 'Ranges reviewed:\nFiles read in full: 1 of 2\n', },),).toEqual({
              filesRead: 1,
              filesTotal: 2,
            },);
          },
        },),
      ],
    },),
    describe({
      name: coverageOfAReview.name,
      children: [
        it({
          name: 'reads a review that states the ranges it was given and the count of the files the brief listed as one that covers its change',
          fn: async () => {
            expect(coverageOf({
              shown: aReviewStating({
                ranges: theRangesGiven(),
                filesRead: theFilesGiven().length,
                filesListed: theFilesGiven().length,
              },),
              reading: theReadingGiven({ files: theFilesGiven(), },),
            },),).toEqual({
              kind: 'covers',
              ranges: theRangesGiven(),
              filesRead: theFilesGiven().length,
              filesListed: theFilesGiven().length,
            },);
          },
        },),
        it({
          name: 'counts a review that read a part of the listed files as one that covers its change, because it says so',
          fn: async () => {
            expect(coverageOf({
              shown: aReviewStating({
                ranges: theRangesGiven(),
                filesRead: 1,
                filesListed: theFilesGiven().length,
              },),
              reading: theReadingGiven({ files: theFilesGiven(), },),
            },),).toEqual({
              kind: 'covers',
              ranges: theRangesGiven(),
              filesRead: 1,
              filesListed: theFilesGiven().length,
            },);
          },
        },),
        it({
          name: 'counts the ranges in whatever order a review states them, because the same ranges in another order are the same change',
          fn: async () => {
            expect(coverageOf({
              shown: aReviewStating({
                ranges: theRangesGiven().toReversed(),
                filesRead: theFilesGiven().length,
                filesListed: theFilesGiven().length,
              },),
              reading: theReadingGiven({ files: theFilesGiven(), },),
            },),).toEqual({
              kind: 'covers',
              ranges: theRangesGiven().toReversed(),
              filesRead: theFilesGiven().length,
              filesListed: theFilesGiven().length,
            },);
          },
        },),
        it({
          name: 'says a review that states another range than the one it was given does not cover its change, naming both',
          fn: async () => {
            expect(coverageOf({
              shown: aReviewStating({
                ranges: ['main...fix/b',],
                filesRead: theFilesGiven().length,
                filesListed: theFilesGiven().length,
              },),
              reading: theReadingGiven({ files: theFilesGiven(), },),
            },),).toEqual({
              kind: 'does-not-cover',
              why: `it states \`main...fix/b\`, and the ranges it was given are \`${theRangesGiven().join('`, `',)}\``,
            },);
          },
        },),
        it({
          name: 'says a review that states no range does not cover its change, naming the ranges it was given',
          fn: async () => {
            expect(coverageOf({
              shown: 'Not final.\n\nNo concrete defects found.\n',
              reading: theReadingGiven({ files: theFilesGiven(), },),
            },),).toEqual({
              kind: 'does-not-cover',
              why: `it states no range, and the ranges it was given are \`${theRangesGiven().join('`, `',)}\``,
            },);
          },
        },),
        it({
          name: 'says a review whose count of files contradicts the brief does not cover its change',
          fn: async () => {
            expect(coverageOf({
              shown: aReviewStating({
                ranges: theRangesGiven(),
                filesRead: 1,
                filesListed: theFilesGiven().length + 1,
              },),
              reading: theReadingGiven({ files: theFilesGiven(), },),
            },),).toEqual({
              kind: 'does-not-cover',
              why: `it states 1 of ${String(theFilesGiven().length + 1,)} files read, and the brief listed ${String(theFilesGiven().length,)}`,
            },);
          },
        },),
        it({
          name: 'says a review that states no count of the files it read does not cover its change',
          fn: async () => {
            expect(coverageOf({
              shown: `Ranges reviewed: \`${theRangesGiven().join('`, `',)}\`\n`,
              reading: theReadingGiven({ files: theFilesGiven(), },),
            },),).toEqual({
              kind: 'does-not-cover',
              why: `it states no count of the files it read, and the brief listed ${String(theFilesGiven().length,)}`,
            },);
          },
        },),
        it({
          name: 'says whether a review written before the statement was asked for covers its change cannot be told, and names the record',
          fn: async () => {
            expect(coverageOf({
              shown: 'Not final.\n\nNo concrete defects found.\n',
              reading: theReadingGiven({},),
            },),).toEqual({
              kind: 'cannot-be-told',
              why: `${theReadingFile()} names no listed files, so its brief was written before a review stated what it read`,
            },);
          },
        },),
        it({
          name: 'says whether a review with no record beside it covers its change cannot be told, and names the record',
          fn: async () => {
            expect(coverageOf({ shown: 'Not final.\n', },),).toEqual({
              kind: 'cannot-be-told',
              why: `nothing kept in ${theReadingFile()} can be read as what this review was given`,
            },);
          },
        },),
      ],
    },),
    describe({
      name: sentenceOfCoverage.name,
      children: [
        it({
          name: 'says a whole review states the ranges it was given and how many of the listed files it read',
          fn: async () => {
            expect(sentenceOfCoverage({
              coverage: {
                kind: 'covers',
                ranges: theRangesGiven(),
                filesRead: theFilesGiven().length,
                filesListed: theFilesGiven().length,
              },
            },),).toBe(`which states the ranges it was given (\`${theRangesGiven().join('`, `',)}\`) and ${String(theFilesGiven().length,)} of ${String(theFilesGiven().length,)} listed files read in full`,);
          },
        },),
        it({
          name: 'says a review that did not cover its change does not cover it, and why',
          fn: async () => {
            expect(sentenceOfCoverage({
              coverage: {
                kind: 'does-not-cover',
                why: 'it states no range',
              },
            },),).toBe('which does not cover its change: it states no range',);
          },
        },),
        it({
          name: 'says whether a review written before the statement cannot be told, rather than that it failed',
          fn: async () => {
            expect(sentenceOfCoverage({
              coverage: {
                kind: 'cannot-be-told',
                why: 'nothing is kept',
              },
            },),).toBe('and whether it covers its change cannot be told: nothing is kept',);
          },
        },),
      ],
    },),
  ],
},);
