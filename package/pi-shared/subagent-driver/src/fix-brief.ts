/**
 Fills the brief a fix agent gets for the defects one review reported: the rules file first, the branch and commit the fix starts
 at, the review's own files, and the package whose tests the fix must pass.

 The text lives in `briefs/fix-the-defects-a-review-found-template.ts`, one placeholder for each part this module fills, so a
 placeholder nobody fills fails a test rather than reaching an agent as its task.

 @module
 */

import { FIX_THE_DEFECTS_A_REVIEW_FOUND_TEMPLATE, } from './briefs/fix-the-defects-a-review-found-template.ts';
import { worktreeFolderOf, } from './parse-worktree-launcher-options.ts';
import {
  diffFileOfAReview,
  reproductionsFolderOfAReview,
  reviewFileOfAReview,
} from './reviewer-of-a-change.ts';
import {
  filledTemplate,
  placeholdersIn,
} from './template-with-placeholders.ts';

/**
 Every placeholder the fix brief carries, in the order the text reads them. The template's own placeholders are compared with
 this list, so a placeholder added to the text and not filled here fails a test.
 */
export const FIX_BRIEF_PLACEHOLDERS: readonly string[] = [
  'RULES',
  'PACKAGE',
  'WORKTREE',
  'BRANCH',
  'BASE',
  'REVIEW_FILE',
  'DIFF_FILE',
  'REPRODUCTIONS_FOLDER',
];

/**
 The middle of a review's name that says which round wrote it, which is not part of the change it reviewed.
 */
const ROUND_MARKER = '-round-';

/**
 Reads whether text is one or more digits and nothing else.

 @param text - the text to read

 @returns whether it is digits alone

 @example
 ```ts
 const digits = isDigits('133');
 ```
 */
function isDigits(text: string,): boolean {
  if (text === '') {
    return false;
  }
  for (const character of text) {
    if ((character < '0') || (character > '9')) {
      return false;
    }
  }
  return true;
}

/**
 Reads the change a review read, which is the review's name without the round that wrote it.

 @param review - the review's name, as `fix-the-gate-round-133` or `fix-the-gate`

 @returns the change's name, which a fix agent's run and branch are named after

 @example
 ```ts
 const change = changeNameOfAFixFor({ review: 'fix-the-gate-round-133', },);
 ```
 */
export function changeNameOfAFixFor({ review, }: { readonly review: string },): string {
  /**
   Where the round marker starts, or minus one when the name carries none.
   */
  const marker = review.lastIndexOf(ROUND_MARKER,);
  if ((marker === (-1)) || (!isDigits(review.slice(marker + ROUND_MARKER.length,)))) {
    return review;
  }
  return review.slice(
    0,
    marker,
  );
}

/**
 Reads the branch a fix agent's work goes on.

 @param review - the review whose defects the fix answers

 @param branchPrefix - the namespace the fix branches of this package live under, such as `fix/`

 @returns the branch's name

 @example
 ```ts
 const branch = branchOfAFixFor({ review: 'fix-the-gate-round-133', branchPrefix: 'fix/', },);
 ```
 */
export function branchOfAFixFor({
  review,
  branchPrefix,
}: {
  readonly review: string;
  readonly branchPrefix: string;
}): string {
  return `${branchPrefix}${changeNameOfAFixFor({ review, },)}`;
}

/**
 Builds the value of every placeholder the fix brief carries.

 @param review - the review whose defects the fix answers

 @param branch - the branch the fix's work goes on

 @param base - the commit or branch the fix starts at

 @param packageFolder - the package the fix changes, from the top of the checkout

 @param reviewsFolder - folder that holds one folder per review

 @param checkoutFolder - the top of the checkout the fix works in

 @param rulesFile - the rules file's path, which the brief names first

 @returns the value of each placeholder, by its name

 @example
 ```ts
 const values = fixBriefValues({ review: 'fix-the-gate-round-133', branch: 'fix/fix-the-gate', base: 'abc123', packageFolder: 'package/a', reviewsFolder: '/reviews', checkoutFolder: '/repo', rulesFile: '/repo/package/pi-shared/subagent-driver/doc/rules-every-pi-brief-carries.md', },);
 ```
 */
export function fixBriefValues({
  review,
  branch,
  base,
  packageFolder,
  reviewsFolder,
  checkoutFolder,
  rulesFile,
}: {
  readonly review: string;
  readonly branch: string;
  readonly base: string;
  readonly packageFolder: string;
  readonly reviewsFolder: string;
  readonly checkoutFolder: string;
  readonly rulesFile: string;
}): Readonly<Record<string, string>> {
  return {
    RULES: rulesFile,
    PACKAGE: packageFolder,
    WORKTREE: worktreeFolderOf({
      checkoutFolder,
      runName: changeNameOfAFixFor({ review, },),
    },),
    BRANCH: branch,
    BASE: base,
    REVIEW_FILE: reviewFileOfAReview({
      reviewsFolder,
      review,
    },),
    DIFF_FILE: diffFileOfAReview({
      reviewsFolder,
      review,
    },),
    REPRODUCTIONS_FOLDER: reproductionsFolderOfAReview({
      reviewsFolder,
      review,
    },),
  };
}

/**
 Fills the brief of one fix agent.

 @param review - the review whose defects the fix answers

 @param branch - the branch the fix's work goes on

 @param base - the commit or branch the fix starts at

 @param packageFolder - the package the fix changes, from the top of the checkout

 @param reviewsFolder - folder that holds one folder per review

 @param checkoutFolder - the top of the checkout the fix works in

 @param rulesFile - the rules file's path, which the brief names first

 @returns the brief's text

 @throws TemplatePlaceholderError when the text carries a placeholder no value fills

 @example
 ```ts
 const brief = briefOfAFix({ review: 'fix-the-gate-round-133', branch: 'fix/fix-the-gate', base: 'abc123', packageFolder: 'package/a', reviewsFolder: '/reviews', checkoutFolder: '/repo', rulesFile: '/repo/package/pi-shared/subagent-driver/doc/rules-every-pi-brief-carries.md', },);
 ```
 */
export function briefOfAFix({
  review,
  branch,
  base,
  packageFolder,
  reviewsFolder,
  checkoutFolder,
  rulesFile,
}: {
  readonly review: string;
  readonly branch: string;
  readonly base: string;
  readonly packageFolder: string;
  readonly reviewsFolder: string;
  readonly checkoutFolder: string;
  readonly rulesFile: string;
}): string {
  return filledTemplate({
    template: FIX_THE_DEFECTS_A_REVIEW_FOUND_TEMPLATE,
    values: fixBriefValues({
      review,
      branch,
      base,
      packageFolder,
      reviewsFolder,
      checkoutFolder,
      rulesFile,
    },),
    name: `the brief of the fix for ${review}`,
  },);
}

/**
 Reads the placeholders the fix brief's text carries.

 @returns every placeholder the text carries, in the order it reads them

 @example
 ```ts
 const placeholders = placeholdersInTheFixBrief();
 ```
 */
export function placeholdersInTheFixBrief(): readonly string[] {
  return placeholdersIn({ template: FIX_THE_DEFECTS_A_REVIEW_FOUND_TEMPLATE, },);
}

/**
 Seconds a fix agent's run may last, which is what the scratch brief writers gave one: a fix reproduces each defect on a
 disposable fixture, writes the test that fails first, and runs the package's tests, so it lasts longer than the review it
 answers.
 */
export const FIX_BRIEF_WALL_SECONDS = 21_600;
