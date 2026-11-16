/**
 What a review's own file says it covered, read against what the review was given: the ranges its brief named, and the files that
 brief listed. The one answer both commands that read a finished review act on is `answerOfAReview`, so this file checks the
 coverage it reports and whether each row of its table covers the change.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  REVIEW_COMPLETE_LINE,
  REVIEW_IN_PROGRESS_LINE,
  type ReviewCoverage,
  type ReviewReading,
  answerOfAReview,
  completionIn,
  coverageOfAReview,
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

 @param files - the files the brief listed, absent for a record written before a brief listed them

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
 The completion line a review's file carries, from the two the brief gives it.

 @param line - which of the two the file carries, or none

 @returns the line, with the blank line that follows it, and nothing for a file that carries neither

 @example
 ```ts
 const opening = aCompletionLine({ line: 'in-progress', },);
 ```
 */
function aCompletionLine({ line, }: { readonly line: 'in-progress' | 'complete' | 'neither' },): string {
  if (line === 'in-progress') {
    return `${REVIEW_IN_PROGRESS_LINE}\n\n`;
  }
  return (line === 'complete') ? `${REVIEW_COMPLETE_LINE}\n\n` : '';
}

/**
 The text of a review's own file, with the lines its brief asks for at its top.

 @param ranges - the ranges the review says it read

 @param filesRead - how many of the listed files it says it read

 @param filesListed - how many files it says the brief listed

 @param completion - which completion line the file carries, `complete` unless a test says otherwise

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
  completion = 'complete',
}: {
  readonly ranges: readonly string[];
  readonly filesRead: number;
  readonly filesListed: number;
  readonly completion?: 'in-progress' | 'complete' | 'neither';
}): string {
  return `${aCompletionLine({ line: completion, },)}Ranges reviewed: `
    + `${ranges.map(function inBackticks(range,) {
      return `\`${range}\``;
    },)
      .join(', ',)}\nFiles read in full: ${String(filesRead,)} of ${String(filesListed,)}\n\nNo concrete defects found.\n`;
}

/**
 Says what one review's own file is read as, given the text it holds and what it was given.

 @param shown - the review's own file

 @param reading - what the review was given, absent when no record can be read

 @returns what the review is answered as

 @example
 ```ts
 const coverage = coverageOf({ shown: 'A review.\n', reading: theReadingGiven({ files: theFilesGiven(), },), },);
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

/**
 Says whether a review covers the change it was given, and what its sentence reads.

 @param request - the review's own file and what it was given

 @returns the answer both commands act on

 @example
 ```ts
 const answer = answerOf({ shown: 'A review.\n' },);
 ```
 */
function answerOf({
  shown,
  reading,
}: {
  readonly shown: string;
  readonly reading?: ReviewReading;
},): { readonly covers: boolean; readonly sentence: string } {
  /**
   The whole answer, of which this test reads what a command acts on.
   */
  const answer = answerOfAReview({
    shown,
    readingFile: theReadingFile(),
    ...((reading === undefined) ? {} : { reading, }),
  },);
  return {
    covers: answer.covers,
    sentence: answer.sentence,
  };
}


/**
 What the statement's lines are each put into, and what a review whose file holds them is answered as.

 The grammar this holds is CommonMark's: a wanted line counts where it stands as the message's own text, which is a paragraph or a
 list item, and not inside a fenced code block, inside a block quote, or indented four spaces, which is an indented code block. A
 line after a sentence saying the review stopped counts, because the reader holds a grammar and does not argue with prose: the
 fourth finding's message is refused because its lines are fenced, not because of the sentence before them.

 `worksMessage`, `fencedMessage`, `quotedMessage`, `indentedMessage` and `stoppedMessage` below are the five shapes: the wanted
 lines plain; fenced as an example of the brief's header; quoted; indented; and plain after a sentence that says the review
 stopped.
 */
const MESSAGE_SHAPES: readonly {
  readonly name: string;
  readonly shown: () => string;
  readonly covers: boolean;
  readonly sentence: string;
}[] = [
  {
    name: 'counts the wanted lines where they stand as the message\'s own text',
    shown: function worksMessage() {
      return aReviewStating({
        ranges: theRangesGiven(),
        filesRead: theFilesGiven().length,
        filesListed: theFilesGiven().length,
      },);
    },
    covers: true,
    sentence: 'which states the ranges it was given',
  },
  {
    name: 'reads no wanted line out of a fenced code block, where a message quotes the brief\'s header as an example',
    shown: function fencedMessage() {
      return 'I stopped before reviewing and have no verdict. The header the brief requested was:\n\n'
        + '```text\n'
        + `${REVIEW_COMPLETE_LINE}\n`
        + `Ranges reviewed: \`${theRangesGiven().join('`, `',)}\`\n`
        + `Files read in full: ${String(theFilesGiven().length,)} of ${String(theFilesGiven().length,)}\n`
        + '```\n';
    },
    covers: false,
    sentence: 'which does not cover its change',
  },
  {
    name: 'reads no wanted line out of a block quote',
    shown: function quotedMessage() {
      return 'Not final and not approved.\n\n'
        + `> ${REVIEW_COMPLETE_LINE}\n`
        + `> Ranges reviewed: \`${theRangesGiven().join('`, `',)}\`\n`
        + `> Files read in full: ${String(theFilesGiven().length,)} of ${String(theFilesGiven().length,)}\n`;
    },
    covers: false,
    sentence: 'which does not cover its change',
  },
  {
    name: 'reads no wanted line out of an indented code block',
    shown: function indentedMessage() {
      return 'Not final and not approved.\n\n'
        + `    ${REVIEW_COMPLETE_LINE}\n`
        + `    Ranges reviewed: \`${theRangesGiven().join('`, `',)}\`\n`
        + `    Files read in full: ${String(theFilesGiven().length,)} of ${String(theFilesGiven().length,)}\n`;
    },
    covers: false,
    sentence: 'which does not cover its change',
  },
  {
    name: 'counts the wanted lines after a sentence that says the review stopped, because the grammar, not the prose, decides',
    shown: function stoppedMessage() {
      return `I stopped early and may not have read everything.\n\n${
         aReviewStating({
          ranges: theRangesGiven(),
          filesRead: theFilesGiven().length,
          filesListed: theFilesGiven().length,
        },)}`;
    },
    covers: true,
    sentence: 'which states the ranges it was given',
  },
];

await describe({
  name: '',
  children: [
    describe({
      name: completionIn.name,
      children: [
        it({
          name: 'reads the line the brief gives a review whose runs are still going as a review that has not finished',
          fn: async () => {
            expect(completionIn({ shown: `Not final and not approved.\n\n${REVIEW_IN_PROGRESS_LINE}\n\nNo verdict yet.\n`, },),)
              .toEqual({ kind: 'in-progress', },);
          },
        },),
        it({
          name: 'reads the line that replaces it as a review whose work is done, whatever mark the report carries',
          fn: async () => {
            expect(completionIn({
              shown: `**Not final and not approved.**\n\n**${REVIEW_COMPLETE_LINE}**\n\nNo concrete defects found.\n`,
            },),).toEqual({ kind: 'complete', },);
          },
        },),
        it({
          name: 'says a file carries neither line when it carries neither, so a review cannot be completed by saying nothing',
          fn: async () => {
            expect(completionIn({ shown: 'Not final and not approved.\n\nNo concrete defects found.\n', },),)
              .toEqual({ kind: 'neither', },);
          },
        },),
        it({
          name: 'decides for the line that says the runs are going when a file carries both, because the reviewer left it behind',
          fn: async () => {
            expect(completionIn({ shown: `${REVIEW_IN_PROGRESS_LINE}\n${REVIEW_COMPLETE_LINE}\n`, },),)
              .toEqual({ kind: 'in-progress', },);
          },
        },),
        it({
          name: 'does not read the line from a sentence that merely mentions it, because the whole line has to be the line',
          fn: async () => {
            expect(completionIn({ shown: `The brief asks for "${REVIEW_COMPLETE_LINE}" at the top.\n`, },),)
              .toEqual({ kind: 'neither', },);
          },
        },),
      ],
    },),
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
              shown: `**Not final and not approved.**\n\n${REVIEW_COMPLETE_LINE}\n\n`
                + '- **Ranges reviewed:** `main...fix/a`\n- Files read in full: 1 of 2\n',
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
            expect(statementIn({ shown: `${REVIEW_COMPLETE_LINE}\n\nNo concrete defects found.\n`, },),).toEqual({},);
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
        it({
          name: 'reads a range in a code span byte for byte, so an underscore inside a reference survives',
          fn: async () => {
            expect(statementIn({ shown: 'Ranges reviewed: `main...fix/a_b`\nFiles read in full: 1 of 1\n', },),).toEqual({
              ranges: ['main...fix/a_b',],
              filesRead: 1,
              filesTotal: 1,
            },);
          },
        },),
      ],
    },),
    describe({
      name: coverageOfAReview.name,
      children: [
        it({
          name: 'reads a review that states the ranges it was given and that it read every listed file as one that covers its change',
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
              kind: 'another-range',
              stated: ['main...fix/b',],
              given: theRangesGiven(),
            },);
          },
        },),
        it({
          name: 'says a review that states it read fewer files than the brief listed does not cover its change, even when it read some',
          fn: async () => {
            expect(coverageOf({
              shown: aReviewStating({
                ranges: theRangesGiven(),
                filesRead: 1,
                filesListed: theFilesGiven().length,
              },),
              reading: theReadingGiven({ files: theFilesGiven(), },),
            },),).toEqual({
              kind: 'fewer-files',
              filesRead: 1,
              filesListed: theFilesGiven().length,
            },);
          },
        },),
        it({
          name: 'says a review that states it read none of the listed files does not cover its change',
          fn: async () => {
            expect(coverageOf({
              shown: aReviewStating({
                ranges: theRangesGiven(),
                filesRead: 0,
                filesListed: theFilesGiven().length,
              },),
              reading: theReadingGiven({ files: theFilesGiven(), },),
            },),).toEqual({
              kind: 'fewer-files',
              filesRead: 0,
              filesListed: theFilesGiven().length,
            },);
          },
        },),
        it({
          name: 'says a review that states no range does not cover its change, naming the ranges it was given',
          fn: async () => {
            expect(coverageOf({
              shown: `${REVIEW_COMPLETE_LINE}\n\nNo concrete defects found.\n`,
              reading: theReadingGiven({ files: theFilesGiven(), },),
            },),).toEqual({
              kind: 'no-range',
              given: theRangesGiven(),
            },);
          },
        },),
        it({
          name: 'says a review that states no count of the files it read does not cover its change',
          fn: async () => {
            expect(coverageOf({
              shown: `${REVIEW_COMPLETE_LINE}\nRanges reviewed: \`${theRangesGiven().join('`, `',)}\`\n`, 
              reading: theReadingGiven({ files: theFilesGiven(), },),
            },),).toEqual({
              kind: 'no-count',
              filesListed: theFilesGiven().length,
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
              kind: 'other-count',
              filesRead: 1,
              filesTotal: theFilesGiven().length + 1,
              filesListed: theFilesGiven().length,
            },);
          },
        },),
        it({
          name: 'says whether a review written before the statement was asked for covers its change cannot be told, and names the record',
          fn: async () => {
            expect(coverageOf({
              shown: `${REVIEW_COMPLETE_LINE}\n\nNo concrete defects found.\n`,
              reading: theReadingGiven({},),
            },),).toEqual({
              kind: 'cannot-be-told',
              why: `${theReadingFile()} names no listed files, so its brief was written before a review stated what it read`,
            },);
          },
        },),
        it({
          name: 'says a review whose runs are still going does not cover its change, and names the line it still carries',
          fn: async () => {
            expect(coverageOf({
              shown: aReviewStating({
                ranges: theRangesGiven(),
                filesRead: theFilesGiven().length,
                filesListed: theFilesGiven().length,
                completion: 'in-progress',
              },),
              reading: theReadingGiven({ files: theFilesGiven(), },),
            },),).toEqual({ kind: 'unfinished', },);
          },
        },),
        it({
          name: 'says a review whose file carries neither completion line does not cover its change',
          fn: async () => {
            expect(coverageOf({
              shown: aReviewStating({
                ranges: theRangesGiven(),
                filesRead: theFilesGiven().length,
                filesListed: theFilesGiven().length,
                completion: 'neither',
              },),
              reading: theReadingGiven({ files: theFilesGiven(), },),
            },),).toEqual({ kind: 'no-completion', },);
          },
        },),
        it({
          name: 'reads a finished review that carries the usual not-final marking and the completion line as one that covers its change',
          fn: async () => {
            expect(coverageOf({
              shown: `**Not final and not approved.**\n\n${REVIEW_COMPLETE_LINE}\n\n${
                 aReviewStating({
                  ranges: theRangesGiven(),
                  filesRead: theFilesGiven().length,
                  filesListed: theFilesGiven().length,
                },)}`,
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
          name: 'says whether a review with no record beside it covers its change cannot be told, and names the record',
          fn: async () => {
            expect(coverageOf({ shown: `${REVIEW_COMPLETE_LINE}\n`, },),).toEqual({
              kind: 'cannot-be-told',
              why: `nothing kept in ${theReadingFile()} can be read as what this review was given`,
            },);
          },
        },),
      ],
    },),
    describe({
      name: answerOfAReview.name,
      children: [
        it({
          name: 'covers the change only for a review that states the ranges it was given and that it read every listed file',
          fn: async () => {
            expect(answerOf({
              shown: aReviewStating({
                ranges: theRangesGiven(),
                filesRead: theFilesGiven().length,
                filesListed: theFilesGiven().length,
              },),
              reading: theReadingGiven({ files: theFilesGiven(), },),
            },),).toEqual({
              covers: true,
              sentence: `which states the ranges it was given (\`${theRangesGiven().join('`, `',)}\`) and `
                + `${String(theFilesGiven().length,)} of ${String(theFilesGiven().length,)} listed files read in full`,
            },);
          },
        },),
        it({
          name: 'does not cover the change for a review that read fewer files, and says why a part is not the whole',
          fn: async () => {
            expect(answerOf({
              shown: aReviewStating({
                ranges: theRangesGiven(),
                filesRead: 1,
                filesListed: theFilesGiven().length,
              },),
              reading: theReadingGiven({ files: theFilesGiven(), },),
            },),).toEqual({
              covers: false,
              sentence: `which does not cover its change: it states 1 of ${String(theFilesGiven().length,)} listed files read in `
                + 'full, and a review covers its change only when it read every one of them',
            },);
          },
        },),
        it({
          name: 'does not cover the change for a review whose runs are still going, and says its reviewer stopped before finishing it',
          fn: async () => {
            expect(answerOf({
              shown: aReviewStating({
                ranges: theRangesGiven(),
                filesRead: theFilesGiven().length,
                filesListed: theFilesGiven().length,
                completion: 'in-progress',
              },),
              reading: theReadingGiven({ files: theFilesGiven(), },),
            },),).toEqual({
              covers: false,
              sentence: 'which does not cover its change: it still carries the line that opens a review whose runs are still '
                + 'going, so its reviewer stopped before finishing it',
            },);
          },
        },),
        it({
          name: 'does not cover the change for a file that carries neither completion line, and names both lines it lacks',
          fn: async () => {
            expect(answerOf({
              shown: aReviewStating({
                ranges: theRangesGiven(),
                filesRead: theFilesGiven().length,
                filesListed: theFilesGiven().length,
                completion: 'neither',
              },),
              reading: theReadingGiven({ files: theFilesGiven(), },),
            },),).toEqual({
              covers: false,
              sentence: 'which does not cover its change: it carries neither the line that opens a review whose runs are still '
                + 'going nor the line that replaces it when they have finished',
            },);
          },
        },),
        it({
          name: 'does not cover the change when nothing can be told, and says that rather than calling the review clean',
          fn: async () => {
            expect(answerOf({ shown: `${REVIEW_COMPLETE_LINE}\n`, },),).toEqual({
              covers: false,
              sentence: `and whether it covers its change cannot be told: nothing kept in ${theReadingFile()} can be read as `
                + 'what this review was given',
            },);
          },
        },),
      ],
    },),
    describe({
      name: 'the messages a review\'s own file may be',
      children: MESSAGE_SHAPES.map(function oneShapeOfMessage(shape,) {
        return it({
          name: shape.name,
          fn: async () => {
            /**
             What that message is answered as.
             */
            const answered = answerOf({
              shown: shape.shown(),
              reading: theReadingGiven({ files: theFilesGiven(), },),
            },);
            expect(answered.covers,).toBe(shape.covers,);
            expect(answered.sentence,).toContain(shape.sentence,);
          },
        },);
      },),
    },),
  ],
},);
