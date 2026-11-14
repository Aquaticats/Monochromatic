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
  'FILES',
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
 Builds the value of every placeholder the review brief carries.

 @param review - the change the review reads

 @param commit - the commit under review

 @param source - the folder whose changes are reviewed

 @param files - the files the reviewer reads, from the top of the checkout

 @param focus - what the reviewer looks hardest at

 @param reviewsFolder - folder that holds one folder per review

 @param runsFolder - folder the driver writes its runs into

 @param rulesFile - the rules file's path, which the brief names first

 @returns the value of each placeholder, by its name

 @example
 ```ts
 const values = reviewBriefValues({ review: 'a', commit: 'abc123', source: 'package/a/src', files: [], focus: 'x', reviewsFolder: '/reviews', runsFolder: '/runs', rulesFile: '/repo/package/pi-shared/subagent-driver/doc/rules-every-pi-brief-carries.md', },);
 ```
 */
export function reviewBriefValues({
  review,
  commit,
  source,
  files,
  focus,
  reviewsFolder,
  runsFolder,
  rulesFile,
}: {
  readonly review: string;
  readonly commit: string;
  readonly source: string;
  readonly files: readonly string[];
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
    FILES: files.map(function asAListItem(path,) {
      return `- \`${path}\``;
    },)
      .join('\n',),
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

 @param focus - what the reviewer looks hardest at

 @param reviewsFolder - folder that holds one folder per review

 @param runsFolder - folder the driver writes its runs into

 @param rulesFile - the rules file's path, which the brief names first

 @returns the brief's text

 @throws TemplatePlaceholderError when the text carries a placeholder no value fills

 @example
 ```ts
 const brief = briefOfAReview({ review: 'a', commit: 'abc123', source: 'package/a/src', files: ['package/a/src/x.ts'], focus: 'data loss', reviewsFolder: '/reviews', runsFolder: '/runs', rulesFile: '/repo/package/pi-shared/subagent-driver/doc/rules-every-pi-brief-carries.md', },);
 ```
 */
export function briefOfAReview({
  review,
  commit,
  source,
  files,
  focus,
  reviewsFolder,
  runsFolder,
  rulesFile,
}: {
  readonly review: string;
  readonly commit: string;
  readonly source: string;
  readonly files: readonly string[];
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

 @param focus - what the reviewer looks hardest at

 @param reviewsFolder - folder that holds one folder per review

 @param runsFolder - folder the driver writes its runs into

 @param rulesFile - the rules file's path, which the brief names first

 @returns the brief's text

 @throws TemplatePlaceholderError when the text carries a placeholder no value fills

 @example
 ```ts
 const brief = briefOfAChangeToTests({ review: 'a', commit: 'abc123', source: 'package/a/src', files: ['package/a/src/gate_tests.rs'], focus: 'a test that stopped checking', reviewsFolder: '/reviews', runsFolder: '/runs', rulesFile: '/repo/package/pi-shared/subagent-driver/doc/rules-every-pi-brief-carries.md', },);
 ```
 */
export function briefOfAChangeToTests({
  review,
  commit,
  source,
  files,
  focus,
  reviewsFolder,
  runsFolder,
  rulesFile,
}: {
  readonly review: string;
  readonly commit: string;
  readonly source: string;
  readonly files: readonly string[];
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
