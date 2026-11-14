/**
 Fills the brief a review of a change reads: the rules file first, the commit under review, the files to read, and where the
 review's own files go.

 A change of code and a change to tests and fixtures alone read the same opening and ending, and differ in what the files are
 judged by: the code's effect on the data and the waits it holds, or whether each changed test still fails under a break planted
 beneath it. The two texts live in `briefs/review-of-a-change-template.ts`, one placeholder for each part this module fills, so a
 placeholder nobody fills fails a test rather than reaching a reviewer as its task.

 @module
 */

import {
  REVIEW_OF_A_CHANGE_TEMPLATE,
  REVIEW_OF_A_CHANGE_TO_TESTS_TEMPLATE,
} from './briefs/review-of-a-change-template.ts';
import {
  reproductionsFolderOfAReview,
  reviewFolderOf,
  reviewFileOfAReview,
  runNameOfAReview,
} from './reviewer-of-a-change.ts';
import type { RemovedOrRenamedPath, } from './material-of-a-review.ts';
import {
  filledTemplate,
  placeholdersIn,
} from './template-with-placeholders.ts';

/**
 Every placeholder the review briefs carry, in the order the text reads them. Each template's own placeholders are compared with
 this list, so a placeholder added to the text and not filled here fails a test.
 */
export const REVIEW_BRIEF_PLACEHOLDERS: readonly string[] = [
  'RULES',
  'COMMIT',
  'REVIEWS_FOLDER',
  'RUNS_FOLDER',
  'SCRATCH_FOLDER',
  'SUBJECT',
  'WHAT_THE_REVIEW_READS',
  'REPRODUCTIONS_FOLDER',
  'FOCUS',
  'REVIEW_FILE',
];

/**
 What a reviewer of a change looks hardest at, when its caller names no focus: the defects a person meets while the code runs,
 which are the ones the project's requirements name.
 */
export const FOCUS_OF_A_CHANGE_REVIEW: string =
  'data lost or overwritten, a commit or a merge that fails or lands twice, a command that waits without bound or waits on '
    + 'another command, and a race with commands running at the same time or a process killed at any instruction';

/**
 Reads the sentence that says what the review reads.

 @param source - the folder whose changes are reviewed, from the top of the checkout

 @returns the sentence's object, without the full stop

 @example
 ```ts
 const subject = subjectOfAReview({ source: 'package/a/src', },);
 ```
 */
export function subjectOfAReview({ source, }: { readonly source: string },): string {
  return `a change under \`${source}\``;
}

/**
 Reads the sentence that says what a review of a change to tests alone reads.

 @param source - the folder whose changes are reviewed, from the top of the checkout

 @returns the sentence's object, without the full stop

 @example
 ```ts
 const subject = subjectOfAChangeToTests({ source: 'package/a/src', },);
 ```
 */
export function subjectOfAChangeToTests({ source, }: { readonly source: string },): string {
  return `a change to the tests under \`${source}\``;
}

/**
 Reads the part of a brief that says what the reviewer reads: every file the reviewed commit holds at a path the change touched,
 and every path the change removed or renamed, with what happened to each one.

 A removed or renamed path is named even though the commit holds no file to read in full at it, because a review of what is left
 would leave the removal unread: the diff holds every line such a path lost, and the reviewer is asked what depended on it.

 @param files - the files the reviewer reads in full, each a path from the top of the checkout

 @param removedOrRenamed - the paths the change removed or renamed

 @returns the lines of the brief that say what to read and what to say about what is gone

 @example
 ```ts
 const reads = whatTheReviewReads({ files: ['package/a/src/gate.rs'], removedOrRenamed: [{ status: 'deleted', path: 'package/a/src/old_gate.rs' }], },);
 ```
 */
export function whatTheReviewReads({
  files,
  removedOrRenamed,
}: {
  readonly files: readonly string[];
  readonly removedOrRenamed: readonly RemovedOrRenamedPath[];
}): string {
  /**
   The lines of the part, in the order a reviewer reads them.
   */
  const lines: string[] = [];
  if (files.length > 0) {
    lines.push(
      'Read each of these files in full before you answer:',
      ...files.map(function asAListItem(path,) {
        return `- \`${path}\``;
      },),
    );
  }
  else {
    lines.push('The reviewed commit holds no file at a path this change touches, so the diff holds everything there is to read.',);
  }
  if (removedOrRenamed.length > 0) {
    lines.push(
      'The change removes or renames these paths, and the diff holds each one\'s change:',
      ...removedOrRenamed.map(function asARemoval(removed,) {
        return (removed.status === 'deleted')
          ? `- \`${removed.path}\` (deleted)`
          : `- \`${removed.was}\` (renamed to \`${removed.now}\`)`;
      },),
      'Say what depended on each of them, and what does its work now or why nothing does.',
    );
  }
  return lines.join('\n',);
}

/**
 Builds the value of every placeholder the review brief carries.

 @param review - the change the review reads

 @param commit - the commit under review

 @param source - the folder whose changes are reviewed

 @param files - the files the reviewer reads, from the top of the checkout

 @param removedOrRenamed - the paths the change removed or renamed

 @param focus - what the reviewer looks hardest at

 @param reviewsFolder - folder that holds one folder per review

 @param runsFolder - folder the driver writes its runs into

 @param rulesFile - the rules file's path, which the brief names first

 @returns the value of each placeholder, by its name

 @example
 ```ts
 const values = reviewBriefValues({ review: 'a', commit: 'abc123', source: 'package/a/src', files: [], removedOrRenamed: [], focus: 'x', reviewsFolder: '/reviews', runsFolder: '/runs', rulesFile: '/repo/package/pi-shared/subagent-driver/doc/rules-every-pi-brief-carries.md', },);
 ```
 */
export function reviewBriefValues({
  review,
  commit,
  source,
  files,
  removedOrRenamed,
  focus,
  reviewsFolder,
  runsFolder,
  rulesFile,
}: {
  readonly review: string;
  readonly commit: string;
  readonly source: string;
  readonly files: readonly string[];
  readonly removedOrRenamed: readonly RemovedOrRenamedPath[];
  readonly focus: string;
  readonly reviewsFolder: string;
  readonly runsFolder: string;
  readonly rulesFile: string;
}): Readonly<Record<string, string>> {
  return {
    RULES: rulesFile,
    COMMIT: commit,
    REVIEWS_FOLDER: reviewsFolder,
    RUNS_FOLDER: runsFolder,
    SCRATCH_FOLDER: reviewFolderOf({
      reviewsFolder,
      review,
    },),
    SUBJECT: subjectOfAReview({ source, },),
    WHAT_THE_REVIEW_READS: whatTheReviewReads({
      files,
      removedOrRenamed,
    },),
    REPRODUCTIONS_FOLDER: reproductionsFolderOfAReview({
      reviewsFolder,
      review,
    },),
    FOCUS: focus,
    REVIEW_FILE: reviewFileOfAReview({
      reviewsFolder,
      review,
    },),
  };
}

/**
 Fills the brief of one review.

 @param review - the change the review reads

 @param commit - the commit under review

 @param source - the folder whose changes are reviewed

 @param files - the files the reviewer reads

 @param removedOrRenamed - the paths the change removed or renamed

 @param focus - what the reviewer looks hardest at

 @param reviewsFolder - folder that holds one folder per review

 @param runsFolder - folder the driver writes its runs into

 @param rulesFile - the rules file's path, which the brief names first

 @returns the brief's text

 @throws TemplatePlaceholderError when the text carries a placeholder no value fills

 @example
 ```ts
 const brief = briefOfAReview({ review: 'a', commit: 'abc123', source: 'package/a/src', files: ['package/a/src/x.ts'], removedOrRenamed: [], focus: 'data loss', reviewsFolder: '/reviews', runsFolder: '/runs', rulesFile: '/repo/package/pi-shared/subagent-driver/doc/rules-every-pi-brief-carries.md', },);
 ```
 */
export function briefOfAReview({
  review,
  commit,
  source,
  files,
  removedOrRenamed,
  focus,
  reviewsFolder,
  runsFolder,
  rulesFile,
}: {
  readonly review: string;
  readonly commit: string;
  readonly source: string;
  readonly files: readonly string[];
  readonly removedOrRenamed: readonly RemovedOrRenamedPath[];
  readonly focus: string;
  readonly reviewsFolder: string;
  readonly runsFolder: string;
  readonly rulesFile: string;
}): string {
  return filledTemplate({
    template: REVIEW_OF_A_CHANGE_TEMPLATE,
    values: reviewBriefValues({
      review,
      commit,
      source,
      files,
      removedOrRenamed,
      focus,
      reviewsFolder,
      runsFolder,
      rulesFile,
    },),
    name: `the brief of ${runNameOfAReview({ review, },)}`,
  },);
}

/**
 Fills the brief of one review of a change to tests and fixtures alone: the same rules, commit and folders, the changed tests and
 fixtures to read, and the breaks a reviewer plants under them.

 @param review - the change the review reads

 @param commit - the commit under review

 @param source - the folder whose changes are reviewed

 @param files - the changed tests and fixtures the reviewer reads

 @param removedOrRenamed - the test and fixture paths the change removed or renamed

 @param focus - what the reviewer looks hardest at

 @param reviewsFolder - folder that holds one folder per review

 @param runsFolder - folder the driver writes its runs into

 @param rulesFile - the rules file's path, which the brief names first

 @returns the brief's text

 @throws TemplatePlaceholderError when the text carries a placeholder no value fills

 @example
 ```ts
 const brief = briefOfAChangeToTests({ review: 'a', commit: 'abc123', source: 'package/a/src', files: ['package/a/src/gate_tests.rs'], removedOrRenamed: [], focus: 'a test that stopped checking', reviewsFolder: '/reviews', runsFolder: '/runs', rulesFile: '/repo/package/pi-shared/subagent-driver/doc/rules-every-pi-brief-carries.md', },);
 ```
 */
export function briefOfAChangeToTests({
  review,
  commit,
  source,
  files,
  removedOrRenamed,
  focus,
  reviewsFolder,
  runsFolder,
  rulesFile,
}: {
  readonly review: string;
  readonly commit: string;
  readonly source: string;
  readonly files: readonly string[];
  readonly removedOrRenamed: readonly RemovedOrRenamedPath[];
  readonly focus: string;
  readonly reviewsFolder: string;
  readonly runsFolder: string;
  readonly rulesFile: string;
}): string {
  return filledTemplate({
    template: REVIEW_OF_A_CHANGE_TO_TESTS_TEMPLATE,
    values: {
      ...reviewBriefValues({
        review,
        commit,
        source,
        files,
        removedOrRenamed,
        focus,
        reviewsFolder,
        runsFolder,
        rulesFile,
      },),
      SUBJECT: subjectOfAChangeToTests({ source, },),
    },
    name: `the brief of ${runNameOfAReview({ review, },)}`,
  },);
}

/**
 Reads the placeholders the review brief's text carries.

 @returns every placeholder the text carries, in the order it reads them

 @example
 ```ts
 const placeholders = placeholdersInTheReviewBrief();
 ```
 */
export function placeholdersInTheReviewBrief(): readonly string[] {
  return placeholdersIn({ template: REVIEW_OF_A_CHANGE_TEMPLATE, },);
}

/**
 Reads the placeholders the brief of a change to tests alone carries.

 @returns every placeholder the text carries, in the order it reads them

 @example
 ```ts
 const placeholders = placeholdersInTheBriefOfAChangeToTests();
 ```
 */
export function placeholdersInTheBriefOfAChangeToTests(): readonly string[] {
  return placeholdersIn({ template: REVIEW_OF_A_CHANGE_TO_TESTS_TEMPLATE, },);
}
