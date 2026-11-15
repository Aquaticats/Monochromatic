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
import type { LeftOutFile, } from './plan-of-a-review.ts';
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
  'LEFT_OUT',
  'REMOVED',
  'REQUIREMENTS',
  'REPRODUCTIONS_FOLDER',
  'FOCUS',
  'REVIEW_FILE',
];

/**
 The requirements a change of code is judged by, which are the project's own.
 */
export const WHAT_A_CHANGE_TO_CODE_MUST_MEET: string =
  'It must meet these requirements: "never discard user data", "Commits should never fail.", and "nothing should ever wait '
    + 'forever or indefinitely".';

/**
 The requirements a change of tests and fixtures alone is judged by: the files themselves are the change, so what a reviewer can
 show about them is that each test still fails when the code it checks does the wrong thing.
 */
export const WHAT_A_CHANGE_TO_TESTS_MUST_PROVE: string =
  'These files are tests and the fixtures tests use, and the change is to them alone. It must meet this requirement: a test '
    + 'proves what it proved before, so it still fails when the code it checks does the wrong thing, and still passes only when '
    + 'that code does what it must.';

/**
 The requirements a change is judged by when it holds a test and the code that test checks: both, because a reviewer reads the
 code as well as planting a break under each test.
 */
export const WHAT_TESTS_AND_THEIR_CODE_MUST_MEET: string =
  'These files are the tests and the fixtures tests use, and the code beside them. The code must meet these requirements: '
    + '"never discard user data", "Commits should never fail.", and "nothing should ever wait forever or indefinitely". Each '
    + 'test must meet this requirement: a test proves what it proved before, so it still fails when the code it checks does the '
    + 'wrong thing, and still passes only when that code does what it must.';

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
 Reads the file list a brief gives its reviewer, and says what an escape means when a name needs one.

 Git's names are bytes, and a brief is text: a name whose bytes are not UTF-8, or that holds a newline, a tab, a backslash or a
 backtick, is shown with `\x` escapes, which are the file's own bytes. A shell takes those bytes as a `$'...'` word, so the
 reviewer can still reach the file.

 @param files - the files the reviewer reads, as the material shows each name

 @returns the list items, and the sentence that explains an escape when one appears

 @example
 ```ts
 const items = filesToListItems({ files: ['package/a/src/gate.rs'], },);
 ```
 */
export function filesToListItems({ files }: { readonly files: readonly string[] },): string {
  if (files.length === 0) {
    return 'Nothing the change added or changed is to be read in full: every file it removed is read in the change diff.';
  }
  /**
   The list, one path to a line.
   */
  const listed = files.map(function asAListItem(path,) {
    return `- \`${path}\``;
  },)
    .join('\n',);
  /**
   Whether any name needed an escape, which is what the sentence is for.
   */
  const escaped = files.some(function holdsAnEscape(path,) {
    return path.includes(String.raw`\x`,);
  },);
  return escaped
    ? `${listed}\nA path shown with \`\\x\` escapes is that file's own bytes, which a shell reads as a \`$'...'\` word.`
    : listed;
}

/**
 Reads the paragraph a brief gives the files its review leaves out, each by name with the reason, so a file a review cannot judge
 never decides for the files beside it.

 @param filesLeftOut - the files the review leaves out, each with its reason

 @returns the paragraph, or a sentence that says every changed file is read

 @example
 ```ts
 const paragraph = leftOutParagraphOf({ filesLeftOut: [{ path: 'package/a/src/notes.md', why: 'a note, which no test reads', }], },);
 ```
 */
export function leftOutParagraphOf({ filesLeftOut }: {
  readonly filesLeftOut: readonly LeftOutFile[];
},): string {
  if (filesLeftOut.length === 0) {
    return 'No file the change touched is left out.';
  }
  return `These files are left out, with the reason:\n${filesLeftOut.map(function asALeftOutItem(file,) {
    return `- \`${file.path}\` is left out: ${file.why}.`;
  },)
    .join('\n',)}`;
}

/**
 Reads the paragraph a brief gives the files the change removed, which a review reads in the change's diff because the reviewed
 commit no longer holds them.

 @param filesRemoved - the files the change removed, in the order git named them

 @returns the paragraph, or an empty string when the change removed no file the review reads

 @example
 ```ts
 const paragraph = removedParagraphOf({ filesRemoved: ['package/a/src/prod.ts'], },);
 ```
 */
export function removedParagraphOf({ filesRemoved }: {
  readonly filesRemoved: readonly string[];
},): string {
  if (filesRemoved.length === 0) {
    return '';
  }
  return `These files were removed by the change, and the change diff is where they are read:\n${filesRemoved.map(
    function asARemovedItem(path,) {
      return `- \`${path}\``;
    },
  )
    .join('\n',)}`;
}

/**
 Builds the value of every placeholder the review brief carries.

 @param review - the change the review reads

 @param commit - the commit under review

 @param source - the folder whose changes are reviewed

 @param files - the files the reviewer reads, from the top of the checkout

 @param filesLeftOut - the files the review leaves out, each with the reason

 @param filesRemoved - the files the change removed, which the review reads in its diff

 @param requirements - the requirements the change is judged by, as the brief reads them

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
  filesLeftOut,
  filesRemoved,
  requirements,
  focus,
  reviewsFolder,
  runsFolder,
  rulesFile,
}: {
  readonly review: string;
  readonly commit: string;
  readonly source: string;
  readonly files: readonly string[];
  readonly filesLeftOut: readonly LeftOutFile[];
  readonly filesRemoved: readonly string[];
  readonly requirements: string;
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
    FILES: filesToListItems({ files, },),
    LEFT_OUT: leftOutParagraphOf({ filesLeftOut, },),
    REMOVED: removedParagraphOf({ filesRemoved, },),
    REQUIREMENTS: requirements,
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

 @param filesLeftOut - the files the review leaves out, each with the reason

 @param filesRemoved - the files the change removed, which the review reads in its diff

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
  filesLeftOut,
  filesRemoved,
  focus,
  reviewsFolder,
  runsFolder,
  rulesFile,
}: {
  readonly review: string;
  readonly commit: string;
  readonly source: string;
  readonly files: readonly string[];
  readonly filesLeftOut: readonly LeftOutFile[];
  readonly filesRemoved: readonly string[];
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
      filesLeftOut,
      filesRemoved,
      requirements: WHAT_A_CHANGE_TO_CODE_MUST_MEET,
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

 @param files - the changed tests, fixtures and material beside them the reviewer reads

 @param filesOfCode - the code among those files, which the brief asks the project's requirements for

 @param filesLeftOut - the files the review leaves out, each with the reason

 @param filesRemoved - the files the change removed, which the review reads in its diff

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
  filesOfCode,
  filesLeftOut,
  filesRemoved,
  focus,
  reviewsFolder,
  runsFolder,
  rulesFile,
}: {
  readonly review: string;
  readonly commit: string;
  readonly source: string;
  readonly files: readonly string[];
  readonly filesOfCode: readonly string[];
  readonly filesLeftOut: readonly LeftOutFile[];
  readonly filesRemoved: readonly string[];
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
        filesLeftOut,
        filesRemoved,
        requirements: (filesOfCode.length > 0) ? WHAT_TESTS_AND_THEIR_CODE_MUST_MEET : WHAT_A_CHANGE_TO_TESTS_MUST_PROVE,
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
