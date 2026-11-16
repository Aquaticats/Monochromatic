/**
 The sentences one second review's brief carries: what the earlier run did and did not do, which files the second review reads, and
 the note that says the second review is owed to a harness no Pi run here can reach.

 The briefs themselves are the review brief the package already writes, and these sentences are what a second review adds to it, so
 the words a first review and a second one read stay in one template.

 @module
 */

import { join, } from 'node:path';
import { collapsed, } from './command-words.ts';
import type { BatchEntry, } from './batch-list.ts';
import { checkLineOf, } from './review-check-line.ts';
import type { ReviewCheck, } from './review-check-types.ts';
import { textAtAPath, } from './review-check-reading.ts';
import {
  diffOfOnlyFiles,
  filesInDiff,
} from './review-diff.ts';
import {
  baseReviewNameOf,
  runWordOf,
} from './review-runs-of-a-change.ts';
import {
  type ReviewerOfAChange,
  reproductionsFolderOfAReview,
  reviewFileOfAReview,
  runNameOfAReview,
} from './reviewer-of-a-change.ts';

/**
 Reads the sentence a restart's brief opens with about what the earlier run's statement says.

 @param check - what the earlier run's check answered

 @returns the sentence

 @example
 ```ts
 const stated = statementSentenceOf({ check });
 ```
 */
function statementSentenceOf({ check, }: { readonly check: ReviewCheck },): string {
  if (check.statement === undefined) {
    return 'It wrote no statement of what it read and ran.';
  }
  /**
   What the earlier run's statement says it read.
   */
  const {coverage} = check;
  /**
   The lines of the diff it read.
   */
  const linesRead = coverage?.linesRead ?? 0;
  /**
   The lines the diff holds.
   */
  const diffLines = coverage?.diffLines ?? 0;
  /**
   The files it read whole.
   */
  const whole = coverage?.filesReadWhole
    .length
    ?? 0;
  /**
   The files it did not read whole.
   */
  const notWhole = coverage?.filesNotReadWhole ?? [];
  /**
   The sentence about the files it did not read whole, when it left any.
   */
  const notWholeNote = (notWhole.length === 0) ? '' : `; it did not read whole: ${notWhole.join(', ')}`;
  return `Its statement says it read ${String(linesRead,)} of the diff's ${String(diffLines,)} lines, and `
    + `${String(whole,)} of ${String(whole + notWhole.length,)} files whole${notWholeNote}. `
    + `It ran ${String(check.builtPrograms
      .length,)} command(s) through a program a build wrote, named for commit `
    + `${collapsed(check.statement
      .commit,)}.`;
}

/**
 Reads the sentence about where the earlier run's own notes are.

 @param review - the review the restart answers

 @param reviewsFolder - folder that holds one folder per review

 @returns the sentence

 @example
 ```ts
 const notes = await notesSentenceOf({ review: 'a', reviewsFolder: '/reviews' });
 ```
 */
async function notesSentenceOf({
  review,
  reviewsFolder,
}: {
  readonly review: string;
  readonly reviewsFolder: string;
},): Promise<string> {
  /**
   The earlier run's own review file.
   */
  const reviewFile = reviewFileOfAReview({
    reviewsFolder,
    review,
  },);
  /**
   What is at the earlier run's review file's path.
   */
  const notes = await textAtAPath({ path: reviewFile, },);
  if (notes.kind === 'no file') {
    return 'It wrote no review file, so its findings were not saved.';
  }
  if (notes.kind === 'unreadable') {
    return `Its review file is at \`${reviewFile}\` and cannot be read: ${notes.reason}.`;
  }
  return `Its own review file is at \`${reviewFile}\`; read it before you begin, and do not repeat a finding it makes unless `
    + `what you read shows that finding is wrong. Its reproduction scripts are under `
    + `\`${reproductionsFolderOfAReview({
      reviewsFolder,
      review,
    },)}\`.`;
}

/**
 Reads the paragraph a restart's brief opens with, which says what the earlier run did and did not do.

 @param check - what the earlier run's check answered

 @param number - which run of the review the restart is

 @param reviewsFolder - folder that holds one folder per review

 @returns the paragraph, ending with a blank line

 @example
 ```ts
 const earlier = await earlierRunTextOf({ check, number: 2, reviewsFolder: '/reviews' });
 ```
 */
export async function earlierRunTextOf({
  check,
  number,
  reviewsFolder,
}: {
  readonly check: ReviewCheck;
  readonly number: number;
  readonly reviewsFolder: string;
},): Promise<string> {
  /**
   The review the restart answers.
   */
  const baseReview = baseReviewNameOf({ review: check.review, },);
  /**
   The run whose work the restart continues.
   */
  const run = runNameOfAReview({ review: check.review, },);
  /**
   What the earlier run's ending says.
   */
  const ending = (check.run === undefined)
    ? `No finished run of ${run} is in the runs folder, so the earlier run left no summary.`
    : `${run} ended ${check.endReason ?? 'with no reason its summary names'}, `
      + `${check.endedByItself ? 'so it ended by itself' : 'so it was cut rather than ended by itself'}.`;
  return `This is the ${runWordOf(number,)} run of the review \`${baseReview}\`. ${ending} `
    + `Its check answered \`${check.verdict}\`. `
    + `${statementSentenceOf({ check, },)} `
    + `${await notesSentenceOf({
      review: check.review,
      reviewsFolder,
    },)}\n\n`;
}

/**
 Says whether the earlier review read every file of its diff whole.

 @param check - what the earlier run's check answered

 @returns whether its reading was complete and left files unread whole

 @example
 ```ts
 const partial = leftFilesUnread({ check });
 ```
 */
export function leftFilesUnread({ check, }: { readonly check: ReviewCheck },): boolean {
  return (check.verdict === 'complete')
    && ((check.coverage
      ?.filesNotReadWhole
      .length
      ?? 0) > 0);
}

/**
 Reads the files a restart's reviewer should read: the files the earlier review did not read whole, when its reading was
 complete, and the whole diff when the earlier run left no complete reading.

 @param check - what the earlier run's check answered

 @param diff - the earlier review's diff

 @returns the files to read, from the top of the checkout

 @example
 ```ts
 const files = filesForARestart({ check, diff: await readFile('change.diff', 'utf8') });
 ```
 */
export function filesForARestart({
  check,
  diff,
}: {
  readonly check: ReviewCheck;
  readonly diff: string;
},): readonly string[] {
  if (leftFilesUnread({ check, },)) {
    return (check.coverage
      ?.filesNotReadWhole
      ?? []).filter(function isOneLine(path,) {
        return (path !== '') && (!path.includes('\n',))
          && (!path.includes('\r',));
      },);
  }
  return filesInDiff({ diff, },)
    .files
    .filter(function isOneLine(path,) {
      return (path !== '') && (!path.includes('\n',))
        && (!path.includes('\r',));
    },);
}

/**
 Reads the batch list that starts one restart.

 @param entry - the entry the launcher takes

 @returns the list's text

 @example
 ```ts
 const shown = batchListOfOne({ entry });
 ```
 */
export function batchListOfOne({ entry, }: { readonly entry: BatchEntry },): string {
  return `${JSON.stringify(
    [entry,],
    undefined,
    2
  )}\n`;
}

/**
 Reads the note that says a second review is owed to a harness no Pi run here can reach.

 The note is what is written when the other model of the pair cannot be reached: it names the first review's own line, the
 reviewer owed, the commit, and the brief a harness with that model reads, so whoever coordinates can hand it on without reading
 this command's output.

 @param check - what the first review's check answered

 @param restartReview - the second review's name

 @param reviewer - the reviewer the second review is owed to

 @param briefFile - the brief the harness reads

 @param commit - the commit the second review reads

 @returns the note's text

 @example
 ```ts
 const note = owedNoteTextOf({ check, restartReview: 'a-restart-2', reviewer: REVIEWING_PEER, briefFile: '/reviews/a-restart-2/brief.md', commit: 'abc1234' });
 ```
 */
export function owedNoteTextOf({
  check,
  restartReview,
  reviewer,
  briefFile,
  commit,
}: {
  readonly check: ReviewCheck;
  readonly restartReview: string;
  readonly reviewer: ReviewerOfAChange;
  readonly briefFile: string;
  readonly commit: string;
},): string {
  return `# A second review is owed: ${restartReview}\n\n`
    + `The review \`${check.review}\` is thin, and a second review answers a thin review once. It has to run on `
    + `${reviewer.name}, the other of the pair the project's rules name, and no Pi model on this machine is `
    + `${reviewer.name}.\n\n`
    + `What the first review left, as the check read it:\n\n`
    + `    ${collapsed(checkLineOf({ check, },),)}\n\n`
    + `The second review reads commit \`${commit}\`, and the brief it reads is \`${briefFile}\`.\n\n`
    + `Hand that brief to a harness that has ${reviewer.name}, or run this command again once Pi lists that model: it starts `
    + `this second review then, once, and never a third.\n`;
}
