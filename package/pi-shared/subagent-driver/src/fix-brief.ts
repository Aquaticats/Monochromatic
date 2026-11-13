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
 A fix run's name that repeats a segment of the branch prefix it lives under.
 */
export class DoubledBranchSegmentError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'DoubledBranchSegmentError';
}

/**
 The word that ends one segment of a branch name, which a run name repeating a segment also repeats.
 */
const SEGMENT_SEPARATOR = '-';

/**
 The word that separates a branch's namespaces.
 */
const BRANCH_SEPARATOR = '/';

/**
 Reads the branch a fix agent's work goes on: the namespace the fixes of this package live under, then the run's own name.

 The name comes from the caller, so the branch says what the fix fixes rather than which review round found it. A prefix whose own
 last segment the name begins with is refused: `fix/git-policy-cli-review-findings-` with the name
 `review-findings-a-late-runtime` reads `fix/git-policy-cli-review-findings-review-findings-a-late-runtime`, which says the same
 thing twice.

 @param name - the fix run's own name, as the caller gave it

 @param branchPrefix - the namespace the fix branches of this package live under, such as `fix/git-policy-cli-`

 @returns the branch's name

 @throws DoubledBranchSegmentError when the name repeats the last segment of the prefix

 @example
 ```ts
 const branch = branchOfAFixFor({ name: 'a-late-runtime', branchPrefix: 'fix/git-policy-cli-', },);
 ```
 */
export function branchOfAFixFor({
  name,
  branchPrefix,
}: {
  readonly name: string;
  readonly branchPrefix: string;
}): string {
  /**
   The prefix's own last segment, which is the part a repeated name would double.
   */
  const segments = branchPrefix.split(BRANCH_SEPARATOR,);
  /**
   The prefix's own last segment.
   */
  const segment = segments.at(-1,) ?? '';
  /**
   The segment's words, which are what a repeated run is counted in.
   */
  const segmentWords = segment.split(SEGMENT_SEPARATOR,)
    .filter(function isAWord(word,) {
      return word !== '';
    },);
  /**
   The longest run of the prefix's words that the name begins with, which is the run it would repeat.
   */
  const repeated = longestRepeatedRun({
    name,
    segmentWords,
  },);
  if (repeated !== '') {
    throw new DoubledBranchSegmentError(
      `the branch ${branchPrefix}${name} says "${repeated}" twice, because the name begins with the prefix's own last words: `
        + `name the fix run for what it fixes, as the branch ${branchPrefix}<what it fixes>`,
    );
  }
  return `${branchPrefix}${name}`;
}

/**
 Reads the longest run of a prefix's words that a run name begins with.

 @param name - the run's own name

 @param segmentWords - the words of the prefix's last segment, in order

 @returns the repeated run with its own trailing separator, or empty text when the name repeats none of it

 @example
 ```ts
 const repeated = longestRepeatedRun({ name: 'review-findings-a', segmentWords: ['cli', 'review', 'findings'], },);
 ```
 */
function longestRepeatedRun({
  name,
  segmentWords,
}: {
  readonly name: string;
  readonly segmentWords: readonly string[];
},): string {
  for (let taken = segmentWords.length; taken > 0; taken -= 1) {
    /**
     The run of the prefix's last words, with the separator that ends one word.
     */
    const run = `${segmentWords.slice(-taken,)
      .join(SEGMENT_SEPARATOR,)}${SEGMENT_SEPARATOR}`;
    if (name.startsWith(run,)) {
      return run;
    }
  }
  return '';
}

/**
 Builds the value of every placeholder the fix brief carries.

 @param name - the fix run's own name

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
  name,
  review,
  branch,
  base,
  subject,
  packageFolder,
  reviewsFolder,
  checkoutFolder,
  rulesFile,
}: {
  readonly name: string;
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
      runName: name,
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

 @param name - the fix run's own name

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
  name,
  review,
  branch,
  base,
  subject,
  packageFolder,
  reviewsFolder,
  checkoutFolder,
  rulesFile,
}: {
  readonly name: string;
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
      name,
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
