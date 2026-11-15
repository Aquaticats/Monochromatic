/**
 Reads what a review's own file says it covered, and judges it against what the review was given: the ranges its brief named, and
 the files that brief listed.

 A review that reads a part of a change and answers "nothing found" has its verdict taken for the whole, so a review whose stated
 ranges are not the ones it was given, or which states no range at all, is answered as a review that did not cover its change
 rather than as a clean one. The brief asks for two lines at the top of the review file:

 ```text
 Ranges reviewed: `main...fix/a`
 Files read in full: 9 of 12
 ```

 The first line is copied from the brief, so the ranges a review states are the brief's own words; the second is the reviewer's
 count, and its total is checked against the files the brief listed. A review written before the brief asked for these lines has
 a record that names no file, and whether it covers its change cannot be told: that is said rather than answered as a failure.

 @module
 */

import { rangesInALine, } from './ranges-in-words.ts';
import type { ReviewReading, } from './material-of-a-review.ts';

/**
 The label the line that names the ranges a review read begins with, as the brief writes it.
 */
export const RANGES_REVIEWED_LABEL = 'Ranges reviewed:';

/**
 The label the line that names how many of the listed files a review read begins with, as the brief writes it.
 */
export const FILES_READ_LABEL = 'Files read in full:';

/**
 The word a stated count puts between how many files were read and how many the brief listed.
 */
const COUNT_JOINER = 'of';

/**
 The word that joins one stated range to the next, which a range never holds.
 */
const RANGE_JOINER = ' and ';

/**
 The character that ends one stated range and begins the next, which a range never holds.
 */
const RANGE_SEPARATOR = ',';

/**
 The other character that ends one stated range and begins the next, which a reviewer may write instead.
 */
const OTHER_RANGE_SEPARATOR = ';';

/**
 The markdown a reviewer may put around a stated range, which is not part of the range.
 */
const DECORATION: ReadonlySet<string> = new Set<string>([
  '`',
  '*',
  '_',
]);

/**
 The characters a reviewer may put before a stated range, which are not part of it.
 */
const RANGE_OPENERS: ReadonlySet<string> = new Set<string>([
  '(',
  '"',
  "'",
  '[',
]);

/**
 The characters a reviewer may put after a stated range, which are not part of it.
 */
const RANGE_CLOSERS: ReadonlySet<string> = new Set<string>([
  ')',
  '"',
  "'",
  ']',
  '.',
]);

/**
 What a review's own file states about what it covered.
 */
export type ReviewStatement = {
  /**
   The ranges the review says it read, absent when its file states none.
   */
  readonly ranges?: readonly string[];

  /**
   How many of the listed files the review says it read in full, absent when its file states no count.
   */
  readonly filesRead?: number;

  /**
   How many files the review says its brief listed, absent when its file states no count.
   */
  readonly filesTotal?: number;
};

/**
 What a review's own file states about the files it read.
 */
type StatedFileCount =
  | {
    /**
     The review states both numbers of a count.
     */
    readonly kind: 'counted';

    /**
     How many of the listed files the review says it read in full.
     */
    readonly filesRead: number;

    /**
     How many files the review says its brief listed.
     */
    readonly filesTotal: number;
  }
    | {
      /**
       The review states no count.
       */
      readonly kind: 'no-count';
    };

/**
 What a finished review is read as: one that covered the change it was given, one that did not, or one whose own file cannot be
 read against a record written before the brief asked for the statement.
 */
export type ReviewCoverage =
  | {
    /**
     The review states the ranges it was given, and a count of the files that does not contradict the brief.
     */
    readonly kind: 'covers';

    /**
     The ranges the review states it read, which are the ones it was given.
     */
    readonly ranges: readonly string[];

    /**
     How many of the listed files the review says it read in full.
     */
    readonly filesRead: number;

    /**
     How many files the brief listed, as the review states it and as the record holds it.
     */
    readonly filesListed: number;
  }
    | {
      /**
       The review's own file states a range it was not given, states none, or states a count of files that contradicts the brief.
       */
      readonly kind: 'does-not-cover';

      /**
       Why the review does not cover its change, in words a reader of the verdict needs.
       */
      readonly why: string;
    }
    | {
      /**
       The review was written before the brief asked for the statement, so what it covered cannot be read.
       */
      readonly kind: 'cannot-be-told';

      /**
       Why nothing can be told, naming the record that would have said it.
       */
      readonly why: string;
    };

/**
 Reads the text a label is followed by on one line, whatever markdown decorates the line.

 @param line - one line of a review's own file

 @param label - the label, with its colon

 @returns the text after the label, or empty text when the line does not carry the label

 @example
 ```ts
 const text = textAfterLabelIn({ line: '**Ranges reviewed:** `main...fix/a`', label: RANGES_REVIEWED_LABEL, },);
 ```
 */
function textAfterLabelIn({
  line,
  label,
}: {
  readonly line: string;
  readonly label: string;
}): string {
  /**
   Where the label is written, whatever case the reviewer wrote it in.
   */
  const at = line.toLowerCase()
    .indexOf(label.toLowerCase(),);
  return (at === (-1))
    ? ''
    : line.slice(at + label.length,);
}

/**
 Says whether a character is one of the digits a stated count is written with.

 @param character - the character to read

 @returns whether it is a digit

 @example
 ```ts
 const isADigit = isDigit('4');
 ```
 */
function isDigit(character: string,): boolean {
  return (character >= '0') && (character <= '9');
}

/**
 Reads where the first number of a stated count begins.

 @param text - the text a count is written in

 @returns the index of its first digit, or minus one when it holds none

 @example
 ```ts
 const from = firstDigitIn(' 9 of 12');
 ```
 */
function firstDigitIn(text: string,): number {
  for (let index = 0; index < text.length; index += 1) {
    if (isDigit(text.charAt(index,),)) {
      return index;
    }
  }
  return -1;
}

/**
 Reads where the run of digits that starts at a place ends.

 @param text - the text the run is in

 @param from - where the run starts, which is a digit

 @returns the index after its last digit

 @example
 ```ts
 const end = endOfDigitsIn({ text: ' 9 of 12', from: 1, },);
 ```
 */
function endOfDigitsIn({
  text,
  from,
}: {
  readonly text: string;
  readonly from: number;
}): number {
  for (let index = from; index < text.length; index += 1) {
    if (!isDigit(text.charAt(index,),)) {
      return index;
    }
  }
  return text.length;
}

/**
 Reads the count of files a review states, out of the text after the label.

 The two numbers must be joined by the word a count puts between them, so a line that happens to hold two numbers while stating no
 count is not read as one.

 @param text - the text after the label

 @returns the count the line states, or that it states none

 @example
 ```ts
 const counted = countStatedIn(' 9 of 12 listed files read in full');
 ```
 */
function countStatedIn(text: string,): StatedFileCount {
  /**
   Where the number of files read begins.
   */
  const readStart = firstDigitIn(text,);
  if (readStart === (-1)) {
    return { kind: 'no-count', };
  }
  /**
   Where the number of files read ends.
   */
  const readEnd = endOfDigitsIn({
    text,
    from: readStart,
  },);
  /**
   What follows the number of files read.
   */
  const rest = text.slice(readEnd,);
  /**
   Where the number of files listed begins.
   */
  const listedStart = firstDigitIn(rest,);
  if (listedStart === (-1)) {
    return { kind: 'no-count', };
  }
  if (rest.slice(
    0,
    listedStart,
  )
    .trim()
    !== COUNT_JOINER) {
    return { kind: 'no-count', };
  }
  return {
    kind: 'counted',
    filesRead: Number(text.slice(
      readStart,
      readEnd,
    ),),
    filesTotal: Number(rest.slice(
      listedStart,
      endOfDigitsIn({
        text: rest,
        from: listedStart,
      },),
    ),),
  };
}

/**
 Reads a stated range without the markdown or the punctuation a reviewer may have put around it.

 @param token - one piece of the text after the ranges label

 @returns the range it states, or empty text when the piece holds nothing but decoration

 @example
 ```ts
 const range = statedRangeIn('(`main...fix/a`),');
 ```
 */
function statedRangeIn(token: string,): string {
  /**
   The piece without the markdown around it.
   */
  const undecorated = token.split('',)
    .filter(function isNotDecoration(character,) {
      return !DECORATION.has(character,);
    },)
    .join('',)
    .trim();
  /**
   Where the range starts, past any opening punctuation.
   */
  const from = rangeStartIn(undecorated,);
  if (from === (-1)) {
    return '';
  }
  /**
   How much of the piece is the range, before any closing punctuation.
   */
  const to = rangeEndIn(undecorated,);
  return (to < from) ? '' : undecorated.slice(
    from,
    to + 1,
  );
}

/**
 Reads where a stated range begins, past any opening punctuation.

 @param text - the piece of a review's own file a range was stated in

 @returns the index of its first character that is not opening punctuation, or minus one when it holds nothing else

 @example
 ```ts
 const from = rangeStartIn('(`main...fix/a`)');
 ```
 */
function rangeStartIn(text: string,): number {
  for (let index = 0; index < text.length; index += 1) {
    if (!RANGE_OPENERS.has(text.charAt(index,),)) {
      return index;
    }
  }
  return -1;
}

/**
 Reads where a stated range ends, before any closing punctuation.

 @param text - the piece of a review's own file a range was stated in

 @returns the index of its last character that is not closing punctuation, or minus one when it holds nothing else

 @example
 ```ts
 const to = rangeEndIn('main...fix/a.');
 ```
 */
function rangeEndIn(text: string,): number {
  for (let index = text.length - 1; index >= 0; index -= 1) {
    if (!RANGE_CLOSERS.has(text.charAt(index,),)) {
      return index;
    }
  }
  return -1;
}

/**
 Reads the ranges a review states, out of the text after the label.

 A range holds no comma, semicolon or the word `and`, so those separate one from the next; the markdown and the punctuation a
 reviewer may have put around each are not part of the range and are dropped.

 @param text - the text after the label

 @returns the ranges stated, each once and in the order written, and nothing when none is left

 @example
 ```ts
 const ranges = rangesStatedIn(' `main...fix/a`, `main...fix/b`');
 ```
 */
function rangesStatedIn(text: string,): readonly string[] {
  /**
   The pieces one separator or another left, in the order the text reads them.
   */
  const pieces: string[] = [];
  for (const piece of text.split(RANGE_SEPARATOR,)) {
    for (const part of piece.split(OTHER_RANGE_SEPARATOR,)) {
      pieces.push(...part.split(RANGE_JOINER,),);
    }
  }
  return pieces.map(function asARange(piece,) {
    return statedRangeIn(piece,);
  },)
    .filter(function isARange(range,) {
      return range !== '';
    },);
}

/**
 Reads the ranges a review's own file states, from the first line that states any.

 @param shown - the review's own file, as it was written

 @returns the ranges the first line that states any names, and nothing when no line states one

 @example
 ```ts
 const ranges = rangesStatedFirst('Ranges reviewed: `main...fix/a`\n');
 ```
 */
function rangesStatedFirst(shown: string,): readonly string[] {
  for (const line of shown.split('\n',)) {
    /**
     The ranges this line states.
     */
    const ranges = rangesStatedIn(textAfterLabelIn({
      line,
      label: RANGES_REVIEWED_LABEL,
    },),);
    if (ranges.length > 0) {
      return ranges;
    }
  }
  return [];
}

/**
 Reads the count of files a review's own file states, from the first line that states one.

 @param shown - the review's own file, as it was written

 @returns the count the first line that states one names, or that no line states one

 @example
 ```ts
 const counted = countStatedFirst('Files read in full: 9 of 12\n');
 ```
 */
function countStatedFirst(shown: string,): StatedFileCount {
  for (const line of shown.split('\n',)) {
    /**
     The count this line states.
     */
    const counted = countStatedIn(textAfterLabelIn({
      line,
      label: FILES_READ_LABEL,
    },),);
    if (counted.kind === 'counted') {
      return counted;
    }
  }
  return { kind: 'no-count', };
}

/**
 Reads what a review's own file states about what it covered.

 @param shown - the review's own file, as it was written

 @returns the ranges and the count it states, each absent when its file states none

 @example
 ```ts
 const stated = statementIn({ shown: 'Ranges reviewed: `main...fix/a`\nFiles read in full: 9 of 12\n', },);
 ```
 */
export function statementIn({ shown, }: { readonly shown: string },): ReviewStatement {
  /**
   The ranges the review states, which are none when it states none.
   */
  const ranges = rangesStatedFirst(shown,);
  /**
   What the review states about the files it read.
   */
  const counted = countStatedFirst(shown,);
  /**
   The statement so far, with the ranges when the review states any.
   */
  const stated: ReviewStatement = (ranges.length === 0) ? {} : { ranges, };
  if (counted.kind === 'no-count') {
    return stated;
  }
  return {
    ...stated,
    filesRead: counted.filesRead,
    filesTotal: counted.filesTotal,
  };
}

/**
 Says whether two lists of ranges name the same ranges, whatever order they are written in.

 @param left - one list of ranges

 @param right - the other list of ranges

 @returns whether every range of one is a range of the other and both hold as many

 @example
 ```ts
 const same = sameRanges({ left: ['main...fix/a',], right: ['main...fix/a',], },);
 ```
 */
function sameRanges({
  left,
  right,
}: {
  readonly left: readonly string[];
  readonly right: readonly string[];
},): boolean {
  /**
   One list's ranges, sorted, so the order they were written in does not decide.
   */
  const sortedLeft = left.toSorted();
  /**
   The other list's ranges, sorted the same way.
   */
  const sortedRight = right.toSorted();
  return (sortedLeft.length === sortedRight.length)
    && sortedLeft.every(function isTheSame(
      range,
      at,
    ) {
      return range === sortedRight[at];
    },);
}

/**
 Judges a finished review's own file against what the review was given.

 @param shown - the review's own file, as it was written

 @param reading - what the review recorded it read, absent when it recorded nothing

 @param readingFile - the file the record is read from, which a message names when nothing can be told

 @returns what the review is read as

 @example
 ```ts
 const coverage = coverageOfAReview({ shown, reading, readingFile: '/reviews/a/what-the-review-read.json', },);
 ```
 */
export function coverageOfAReview({
  shown,
  reading,
  readingFile,
}: {
  readonly shown: string;
  readonly reading?: ReviewReading;
  readonly readingFile: string;
}): ReviewCoverage {
  if (reading === undefined) {
    return {
      kind: 'cannot-be-told',
      why: `nothing kept in ${readingFile} can be read as what this review was given`,
    };
  }
  // A record written before this statement was asked for names no file its brief listed, and the review it belongs to holds no
  // line that could be checked: only its own text says what it read, and no brief asked it to.
  if (reading.files === undefined) {
    return {
      kind: 'cannot-be-told',
      why: `${readingFile} names no listed files, so its brief was written before a review stated what it read`,
    };
  }
  /**
   What the review's own file states.
   */
  const stated = statementIn({ shown, },);
  if (stated.ranges === undefined) {
    return {
      kind: 'does-not-cover',
      why: `it states no range, and the ranges it was given are ${rangesInALine({ ranges: reading.ranges, },)}`,
    };
  }
  if (!sameRanges({
    left: stated.ranges,
    right: reading.ranges,
  },)) {
    return {
      kind: 'does-not-cover',
      why: `it states ${rangesInALine({ ranges: stated.ranges, },)}, `
        + `and the ranges it was given are ${rangesInALine({ ranges: reading.ranges, },)}`,
    };
  }
  if ((stated.filesRead === undefined) || (stated.filesTotal === undefined)) {
    return {
      kind: 'does-not-cover',
      why: `it states no count of the files it read, and the brief listed ${String(reading.files
        .length,)}`,
    };
  }
  if (stated.filesTotal
    !== reading.files
      .length) {
    return {
      kind: 'does-not-cover',
      why: `it states ${String(stated.filesRead,)} of ${String(stated.filesTotal,)} files read, `
        + `and the brief listed ${String(reading.files
          .length,)}`,
    };
  }
  if (stated.filesRead > stated.filesTotal) {
    return {
      kind: 'does-not-cover',
      why: `it states ${String(stated.filesRead,)} files read of ${String(stated.filesTotal,)} listed`,
    };
  }
  return {
    kind: 'covers',
    ranges: stated.ranges,
    filesRead: stated.filesRead,
    filesListed: stated.filesTotal,
  };
}

/**
 Writes what a review is read as, as the clause that follows the state of its review file.

 @param coverage - what the review is read as

 @returns the clause, without a preceding comma

 @example
 ```ts
 const sentence = sentenceOfCoverage({ coverage, },);
 ```
 */
export function sentenceOfCoverage({ coverage, }: { readonly coverage: ReviewCoverage },): string {
  if (coverage.kind === 'covers') {
    return `which states the ranges it was given (${rangesInALine({ ranges: coverage.ranges, },)}) `
      + `and ${String(coverage.filesRead,)} of ${String(coverage.filesListed,)} listed files read in full`;
  }
  if (coverage.kind === 'does-not-cover') {
    return `which does not cover its change: ${coverage.why}`;
  }
  return `and whether it covers its change cannot be told: ${coverage.why}`;
}
