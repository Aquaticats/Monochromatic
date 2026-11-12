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
  'REVIEW_SUBJECT',
  'REVIEW_FILE',
  'DIFF_FILE',
  'REPRODUCTIONS_FOLDER',
];

/**
 The start of the run name of a fix, so a fix that answers the review `x` is the run `fix-of-x` and never the run of the review it
 answers.
 */
export const FIX_RUN_PREFIX = 'fix-of-';

/**
 Reads the run name of a fix agent, which says what it fixes and shares nothing with the review's own run: the two used to be
 named alike, and their worktrees and units could not stand beside each other.

 @param review - the review whose defects the fix answers, as `rewrite-the-gate-round-133`

 @returns the run's name, as `fix-of-rewrite-the-gate-round-133`

 @example
 ```ts
 const run = fixRunNameOf({ review: 'rewrite-the-gate-round-133', },);
 ```
 */
export function fixRunNameOf({ review, }: { readonly review: string },): string {
  return `${FIX_RUN_PREFIX}${review}`;
}

/**
 Reads the branch a fix agent's work goes on.

 The branch is named after the review it answers, never after the change alone: a change's own branch is one an agent already has
 checked out, so Git would refuse the fix's worktree, and two rounds of review would otherwise answer on one branch.

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
  return `${branchPrefix}${review}`;
}

/**
 Builds the value of every placeholder the fix brief carries.

 @param review - the review whose defects the fix answers

 @param branch - the branch the fix's work goes on

 @param base - the commit the review read, which the fix starts at

 @param subject - what the review read, in the review's own words, so the brief names the change it answers

 @param packageFolder - the package the fix changes, from the top of the checkout

 @param reviewsFolder - folder that holds one folder per review

 @param checkoutFolder - the top of the checkout the fix works in

 @param rulesFile - the rules file's path, which the brief names first

 @returns the value of each placeholder, by its name

 @example
 ```ts
 const values = fixBriefValues({ review: 'fix-the-gate-round-133', branch: 'fix/fix-the-gate-round-133', base: 'abc123', subject: 'main...fix/fix-the-gate at commit abc123', packageFolder: 'package/a', reviewsFolder: '/reviews', checkoutFolder: '/repo', rulesFile: '/repo/package/pi-shared/subagent-driver/doc/rules-every-pi-brief-carries.md', },);
 ```
 */
export function fixBriefValues({
  review,
  branch,
  base,
  subject,
  packageFolder,
  reviewsFolder,
  checkoutFolder,
  rulesFile,
}: {
  readonly review: string;
  readonly branch: string;
  readonly base: string;
  readonly subject: string;
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
      runName: fixRunNameOf({ review, },),
    },),
    BRANCH: branch,
    BASE: base,
    REVIEW_SUBJECT: subject,
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

 @param base - the commit the review read, which the fix starts at

 @param subject - what the review read, as its brief words it

 @param packageFolder - the package the fix changes, from the top of the checkout

 @param reviewsFolder - folder that holds one folder per review

 @param checkoutFolder - the top of the checkout the fix works in

 @param rulesFile - the rules file's path, which the brief names first

 @returns the brief's text

 @throws TemplatePlaceholderError when the text carries a placeholder no value fills

 @example
 ```ts
 const brief = briefOfAFix({ review: 'fix-the-gate-round-133', branch: 'fix/fix-the-gate-round-133', base: 'abc123', subject: 'main...fix/fix-the-gate at commit abc123', packageFolder: 'package/a', reviewsFolder: '/reviews', checkoutFolder: '/repo', rulesFile: '/repo/package/pi-shared/subagent-driver/doc/rules-every-pi-brief-carries.md', },);
 ```
 */
export function briefOfAFix({
  review,
  branch,
  base,
  subject,
  packageFolder,
  reviewsFolder,
  checkoutFolder,
  rulesFile,
}: {
  readonly review: string;
  readonly branch: string;
  readonly base: string;
  readonly subject: string;
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
      subject,
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
