/**
 What one review's check answers: its verdict, the reasons it prints, the coverage it names, whether it is thin, and the reviewer
 its run started on.

 The shapes live in a file of their own so the check that fills them stays within the line budget, and so a caller that only reads a
 verdict imports no reader.

 @module
 */

import type { ReviewCoverage, } from './review-check-reading.ts';
import type { ReviewStatement, } from './review-statement-shape.ts';

/**
 What one review's check answered.
 */
export type ReviewVerdict =
  | 'complete'
    | 'ran nothing'
    | 'statement missing or not matching its session'
    | 'no review file'
    | 'cut';

/**
 One review's verdict and the reasons for it.
 */
export type ReviewCheck = {
  /**
   The change the review read.
   */
  readonly review: string;

  /**
   The verdict.
   */
  readonly verdict: ReviewVerdict;

  /**
   One line per fact that decided the verdict, and per claim the check could not confirm either way.
   */
  readonly reasons: readonly string[];

  /**
   The review file's path.
   */
  readonly reviewFile: string;

  /**
   The newest finished run of the review, when one finished.
   */
  readonly run?: string;

  /**
   Why that run ended, as its summary says.
   */
  readonly endReason?: string;

  /**
   Whether that run ended on its own terms, which is false for a run a cap, a wall clock, an abort or a model error ended.
   */
  readonly endedByItself: boolean;

  /**
   The statement the review wrote, when it wrote one the reader can read.
   */
  readonly statement?: ReviewStatement;

  /**
   What the statement says the review read, when the diff and the statement both read.
   */
  readonly coverage?: ReviewCoverage;

  /**
   The programs the statement names and the session confirms as ones the review's worktree built.
   */
  readonly builtPrograms: readonly string[];

  /**
   Whether the review is thin: it did not read every file it was given whole, or no command ran a program built from the commit.

   A thin review is the one a second start answers, once; a review whose statement cannot be read at all is a separate case.
   */
  readonly thin: boolean;

  /**
   The provider and model the run started on, as its session records them; empty when no session could be read.
   */
  readonly reviewer: {
    readonly provider: string;
    readonly model: string;
  };
};

/**
 The line one check's verdict prints, and the limits every verdict is read with.
 */
export const CHECK_LIMITS_LINE: string =
  'Compared: each statement command against the bash and process-start calls its session records, the exit code a bash result '
    + 'holds, the statement\'s files and line count against the diff file, and the commit against what the review recorded it '
    + 'read. Not visible: commands run inside a codemode call, a process start\'s exit code (its result records none), a script '
    + 'that builds a built program\'s path from a variable rather than holding it, and anything written after the run ended.';
