/**
 The two lines that tell a review whose runs are still going from one whose work is done: the line a review of a change opens
 with while any run it started is still going, and the line that replaces it when every run has finished.

 A review of a change writes its own file while its reproductions still run, because a review that ends with no file is lost, and a
 process can be killed at any instruction. Such a file holds the right ranges and the right count and no verdict, so those two
 lines cannot tell it from a finished review; neither can a mark that the report is not final, because every report an agent
 writes is marked that way, and only the repository's owner declares work complete. So the brief gives the reviewer these exact
 lines, this module holds them once so the brief and the check cannot drift apart, and `a-review-statement-lines.ts` reads the one
 line a review's file carries.

 The words are chosen not to hold "final", so that no reader confuses the line that says whether a review's work is done with the
 mark that says the owner has not approved the report yet.

 @module
 */

/**
 The line a review of a change opens with while any run it started is still going.
 */
export const REVIEW_IN_PROGRESS_LINE =
  'Review in progress: the runs it started are still going, so no verdict is written yet.';

/**
 The line that replaces the one above when every run the review started has finished.
 */
export const REVIEW_COMPLETE_LINE = 'Review complete: every run it started has finished, and this is its verdict.';
