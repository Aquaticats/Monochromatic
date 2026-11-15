/**
 Reads what a review's own file says it covered, and answers one table of outcomes that both commands that read a finished review
 answer from: the command that saves a review's last message, and the command that starts reviews of finished branches.

 A review that reads a part of a change and answers "nothing found" has its verdict taken for the whole. So a review covers its
 change only when its own file says its work is done and states the ranges it was given and that it read every listed file, which
 is what the project's rule for a review asks of it: it reads the paths its brief names, whole files, and the diff. A review whose
 runs are still going, a part of the files, no statement at all, and a record that cannot be read are each not covering.

 The brief asks for one line that says whether the review's work is done, and two more that say what it covered, the first of the
 two copied from the brief and the second the reviewer's count:

 ```text
 Review complete: every run it started has finished, and this is its verdict.
 Ranges reviewed: `main...fix/a`
 Files read in full: 12 of 12
 ```

 The line that says whether the work is done is one of the two the brief gives in `review-completion-lines.ts`, and it is not the
 mark that says the report is not final and not approved: every report of an agent carries that mark, and only the repository's
 owner declares work complete, so a check that refused the mark would refuse every honest review.

 ## The answers, and what each command says and answers with

 Both commands print the sentence under its own row after the state of the review file ("has its review.md", or "saved its last
 message as review.md (N characters)"). The status is the same the first time and when the same save is repeated, because it is
 read from the review's own file and its record, never from whether this run wrote the file.

 - `covers`: the file says every run the review started has finished, the ranges stated are the ranges given, and every listed
   file was read.
   Sentence: "which states the ranges it was given (`A`) and 12 of 12 listed files read in full".
   Save command exit `0`; starter exit `0`.
 - `another-range`: the review states a range it was not given, or leaves one out.
   Sentence: "which does not cover its change: it states `B`, and the ranges it was given are `A`".
   Save command exit `1`; starter exit `1`.
 - `fewer-files`: the review states that it read fewer files than the brief listed.
   Sentence: "which does not cover its change: it states 9 of 12 listed files read in full, and a review covers its change only
   when it read every one of them". Save command exit `1`; starter exit `1`.
 - `other-count`: the count's total is not the number of files the brief listed, or more files are read than listed.
   Sentence: "which does not cover its change: it states 9 of 12 files read, and the brief listed 12".
   Save command exit `1`; starter exit `1`.
 - `no-range`: the review's file states no range.
   Sentence: "which does not cover its change: it states no range, and the ranges it was given are `A`".
   Save command exit `1`; starter exit `1`.
 - `no-count`: the review's file states no count of the files it read.
   Sentence: "which does not cover its change: it states no count of the files it read, and the brief listed 12".
   Save command exit `1`; starter exit `1`.
 - `cannot-be-told`: the record is missing, unreadable, or was written before a review stated what it read.
   Sentence: "and whether it covers its change cannot be told: <why>".
   Save command exit `1`; starter exit `1`.
 - `unfinished`: the file still carries the line that opens a review whose runs are going, as a reviewer killed before it replaced
   the line leaves it.
   Sentence: "which does not cover its change: it still carries the line that opens a review whose runs are still going, so its
   reviewer stopped before finishing it". Save command exit `1`; starter exit `1`. The starter prints this review's own line and
   counts it; it does not start the review again, because the review's branch is fixed by its name and starting over is the
   caller's decision (`README.md`, "`start-reviews-of-finished-branches`").
 - `no-completion`: the file carries neither line.
   Sentence: "which does not cover its change: it carries neither the line that opens a review whose runs are still going nor the
   line that replaces it when they have finished". Save command exit `1`; starter exit `1`.

 A review that wrote no file at all answers none of these: the save command prints why it wrote none ("no review.md and no finished
 run", the run's end reason, or an empty last message), and both commands answer `1`, the first time and again.

 @module
 */

import { rangesInALine, } from './ranges-in-words.ts';
import type { ReviewReading, } from './material-of-a-review.ts';
import {
  completionIn,
  statementIn,
} from './a-review-statement-lines.ts';

/**
 What a finished review is answered as, one kind per row of the table in this module's doc.
 */
export type ReviewCoverage =
  | {
    /**
     The review says every run it started has finished, states the ranges it was given, and says it read every listed file.
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
     How many files the brief listed.
     */
    readonly filesListed: number;
  }
    | {
      /**
       The review states a range it was not given, or leaves one out.
       */
      readonly kind: 'another-range';

      /**
       The ranges the review states.
       */
      readonly stated: readonly string[];

      /**
       The ranges the review was given.
       */
      readonly given: readonly string[];
    }
    | {
      /**
       The review states that it read fewer files than the brief listed.
       */
      readonly kind: 'fewer-files';

      /**
       How many files the review says it read in full.
       */
      readonly filesRead: number;

      /**
       How many files the brief listed.
       */
      readonly filesListed: number;
    }
    | {
      /**
       The count's total is not the number of files the brief listed, or more files are read than listed.
       */
      readonly kind: 'other-count';

      /**
       How many files the review says it read in full.
       */
      readonly filesRead: number;

      /**
       How many files the review says its brief listed.
       */
      readonly filesTotal: number;

      /**
       How many files the brief listed.
       */
      readonly filesListed: number;
    }
    | {
      /**
       The review's file states no range.
       */
      readonly kind: 'no-range';

      /**
       The ranges the review was given.
       */
      readonly given: readonly string[];
    }
    | {
      /**
       The review's file states no count of the files it read.
       */
      readonly kind: 'no-count';

      /**
       How many files the brief listed.
       */
      readonly filesListed: number;
    }
    | {
      /**
       The record of what the review was given is missing, unreadable, or was written before a review stated what it read.
       */
      readonly kind: 'cannot-be-told';

      /**
       Why nothing can be told, naming the record that would have said it.
       */
      readonly why: string;
    }
    | {
      /**
       The review's own file still carries the line that opens a review whose runs are going, so its reviewer stopped mid-work.
       */
      readonly kind: 'unfinished';
    }
    | {
      /**
       The review's own file carries neither the line that opens a review whose runs are going nor the line that replaces it.
       */
      readonly kind: 'no-completion';
    };

/**
 What a finished review is answered as: its coverage, the words a command prints for it, and whether it covers its change.
 */
export type ReviewAnswer = {
  /**
   What the review's own file says, one kind per row of the table in this module's doc.
   */
  readonly coverage: ReviewCoverage;

  /**
   Whether the review covers the change it was given, which it does only when its file states the ranges it was given and that
   it read every listed file.
   */
  readonly covers: boolean;

  /**
   The words a command prints for it, after the state of the review file.
   */
  readonly sentence: string;
};

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

 @returns what the review is answered as

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
  /**
   What the review's own file says about whether its work is done, which is read first: a file its reviewer was killed before
   finishing is not a review of the change, whatever its record says.
   */
  const completion = completionIn({ shown, },);
  if (completion.kind === 'in-progress') {
    return { kind: 'unfinished', };
  }
  if (completion.kind === 'neither') {
    return { kind: 'no-completion', };
  }
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
      kind: 'no-range',
      given: reading.ranges,
    };
  }
  if (!sameRanges({
    left: stated.ranges,
    right: reading.ranges,
  },)) {
    return {
      kind: 'another-range',
      stated: stated.ranges,
      given: reading.ranges,
    };
  }
  if ((stated.filesRead === undefined) || (stated.filesTotal === undefined)) {
    return {
      kind: 'no-count',
      filesListed: reading.files
        .length,
    };
  }
  if ((stated.filesTotal
    !== reading.files
      .length) || (stated.filesRead > stated.filesTotal)) {
    return {
      kind: 'other-count',
      filesRead: stated.filesRead,
      filesTotal: stated.filesTotal,
      filesListed: reading.files
        .length,
    };
  }
  if (stated.filesRead < stated.filesTotal) {
    return {
      kind: 'fewer-files',
      filesRead: stated.filesRead,
      filesListed: stated.filesTotal,
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
 Writes the sentence a command prints for a review's coverage, after the state of its review file.

 @param coverage - what the review is answered as

 @returns the sentence, without a preceding comma

 @example
 ```ts
 const sentence = sentenceOfCoverage({ coverage, },);
 ```
 */
function sentenceOfCoverage({ coverage, }: { readonly coverage: ReviewCoverage },): string {
  if (coverage.kind === 'covers') {
    return `which states the ranges it was given (${rangesInALine({ ranges: coverage.ranges, },)}) `
      + `and ${String(coverage.filesRead,)} of ${String(coverage.filesListed,)} listed files read in full`;
  }
  if (coverage.kind === 'another-range') {
    return `which does not cover its change: it states ${rangesInALine({ ranges: coverage.stated, },)}, `
      + `and the ranges it was given are ${rangesInALine({ ranges: coverage.given, },)}`;
  }
  if (coverage.kind === 'fewer-files') {
    return `which does not cover its change: it states ${String(coverage.filesRead,)} of `
      + `${String(coverage.filesListed,)} listed files read in full, and a review covers its change only when it read every `
      + 'one of them';
  }
  if (coverage.kind === 'other-count') {
    return `which does not cover its change: it states ${String(coverage.filesRead,)} of `
      + `${String(coverage.filesTotal,)} files read, and the brief listed ${String(coverage.filesListed,)}`;
  }
  if (coverage.kind === 'no-range') {
    return `which does not cover its change: it states no range, `
      + `and the ranges it was given are ${rangesInALine({ ranges: coverage.given, },)}`;
  }
  if (coverage.kind === 'no-count') {
    return `which does not cover its change: it states no count of the files it read, `
      + `and the brief listed ${String(coverage.filesListed,)}`;
  }
  if (coverage.kind === 'unfinished') {
    return 'which does not cover its change: it still carries the line that opens a review whose runs are still going, '
      + 'so its reviewer stopped before finishing it';
  }
  if (coverage.kind === 'no-completion') {
    return 'which does not cover its change: it carries neither the line that opens a review whose runs are still going '
      + 'nor the line that replaces it when they have finished';
  }
  return `and whether it covers its change cannot be told: ${coverage.why}`;
}

/**
 Answers a finished review: what its own file says, whether that covers the change, and the words a command prints for it.

 This is the one function both commands that read a finished review answer from, so that the command that saves a review and the
 command that starts reviews cannot disagree about a review's coverage.

 @param shown - the review's own file, as it was written

 @param reading - what the review recorded it read, absent when it recorded nothing

 @param readingFile - the file the record is read from, which a message names when nothing can be told

 @returns the review's coverage, the words for it, and whether it covers its change

 @example
 ```ts
 const answer = answerOfAReview({ shown, reading, readingFile: '/reviews/a/what-the-review-read.json', },);
 ```
 */
export function answerOfAReview({
  shown,
  reading,
  readingFile,
}: {
  readonly shown: string;
  readonly reading?: ReviewReading;
  readonly readingFile: string;
}): ReviewAnswer {
  /**
   What the review's own file says.
   */
  const coverage = coverageOfAReview({
    shown,
    readingFile,
    ...((reading === undefined) ? {} : { reading, }),
  },);
  return {
    coverage,
    covers: coverage.kind === 'covers',
    sentence: sentenceOfCoverage({ coverage, },),
  };
}
