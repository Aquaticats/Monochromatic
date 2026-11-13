/**
 The shape of the statement a review writes at a fixed place in its review file: the section's heading, the language of its one
 fenced block, the JSON object's types, and the skeleton a reviewer copies.

 The types live in a file of their own so the reader that validates the block and the reader that finds the section can each stay
 within the line budget, and so a caller reads one shape rather than two.

 @module
 */

/**
 The heading the statement section carries, as the first section after a review file's title.
 */
export const REVIEW_STATEMENT_HEADING = '## What this review read and ran';

/**
 The language a review file's statement block is tagged with.
 */
export const REVIEW_STATEMENT_LANGUAGE = 'review-statement';

/**
 How a review read one file of the diff.
 */
export type FileReadState = 'whole' | 'part' | 'none';

/**
 One file of the diff, as the review read it.
 */
export type ReviewFileRead = {
  /**
   The file's path, from the top of the checkout.
   */
  readonly path: string;

  /**
   Whether the review read the file whole, read part of it, or read none of it.
   */
  readonly read: FileReadState;
};

/**
 One command a review ran.
 */
export type ReviewCommandRun = {
  /**
   The command, as the review ran it.
   */
  readonly command: string;

  /**
   The code the command exited with.
   */
  readonly exit: number;

  /**
   The path of the program the review's own worktree built that this command ran, when it ran one.
   */
  readonly ranBuiltProgram?: string;
};

/**
 Whether a finding rests on a command a review ran or on reading alone.
 */
export type FindingShownBy = 'run' | 'reading';

/**
 One finding the review reports, as the statement names it.
 */
export type ReviewFinding = {
  /**
   The finding's number or short name, as the review file gives it.
   */
  readonly finding: string;

  /**
   Whether a command the review ran showed it, or only reading did.
   */
  readonly shownBy: FindingShownBy;
};

/**
 What one review states it read and ran.
 */
export type ReviewStatement = {
  /**
   The commit the review reads.
   */
  readonly commit: string;

  /**
   The lines of the diff file in total, and the lines of it the review read.
   */
  readonly diff: {
    readonly lines: number;
    readonly linesRead: number;
  };

  /**
   Every file the diff touches, each once, with how the review read it.
   */
  readonly files: readonly ReviewFileRead[];

  /**
   Each command the review ran, with its exit code and the built program it ran when it ran one.
   */
  readonly commands: readonly ReviewCommandRun[];

  /**
   Each finding, with whether a run or reading showed it.
   */
  readonly findings: readonly ReviewFinding[];
};

/**
 The shape a review copies, with each field's value to replace. It is valid JSON on its own, so a review that copies it and
 replaces nothing is answered by the check rather than by a parser that cannot name the field.
 */
export const REVIEW_STATEMENT_SKELETON: string = `{
  "commit": "<the commit this review reads>",
  "diff": {
    "lines": "<how many lines the diff file holds>",
    "linesRead": "<how many of those lines this review read>"
  },
  "files": [
    { "path": "<a file the diff touches, from the top of the checkout>", "read": "whole" },
    { "path": "<another file the diff touches>", "read": "part" },
    { "path": "<a third file the diff touches>", "read": "none" }
  ],
  "commands": [
    {
      "command": "<a command this review ran, as it ran it>",
      "exit": 0,
      "ranBuiltProgram": "<the path of a program this worktree built>"
    },
    { "command": "<a command that ran no built program>", "exit": 0 }
  ],
  "findings": [
    { "finding": "<the finding's number or short name>", "shownBy": "run" },
    { "finding": "<another finding>", "shownBy": "reading" }
  ]
}`;

/**
 A statement section that cannot be read as the review's statement, naming the file and the field that does not fit.
 */
export class ReviewStatementError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'ReviewStatementError';
}
