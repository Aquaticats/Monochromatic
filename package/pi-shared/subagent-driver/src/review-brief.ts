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
 The last code point of the C0 controls, which a line of a brief cannot carry.
 */
const LAST_C0_CONTROL = 0x1F;

/**
 The first code point of the C1 controls, which a line of a brief cannot carry either.
 */
const FIRST_C1_CONTROL = 0x7F;

/**
 The last code point of the C1 controls.
 */
const LAST_C1_CONTROL = 0x9F;

/**
 The character that ends a code span, which a path shown as itself cannot hold.
 */
const CODE_SPAN_END = '`';

/**
 Says whether a path needs the JSON form: it holds a control character, the backtick that would end the code span it is shown in,
 or a space at both of its ends, which a code span trims.

 @param path - a path from the top of the checkout

 @returns whether a brief shows it as a JSON string

 @example
 ```ts
 const needsJSON = aPathNeedsJSONText('a\nb.ts',);
 ```
 */
function aPathNeedsJSONText(path: string,): boolean {
  if (path.startsWith(' ',) && path.endsWith(' ',)) {
    return true;
  }
  for (const character of path) {
    /**
     The code point this character is.
     */
    const codePoint = character.codePointAt(0,) ?? 0;
    /**
     Whether it is a control a line cannot carry.
     */
    const isAControl = (codePoint <= LAST_C0_CONTROL) || ((codePoint >= FIRST_C1_CONTROL) && (codePoint <= LAST_C1_CONTROL));
    if (isAControl || (character === CODE_SPAN_END)) {
      return true;
    }
  }
  return false;
}

/**
 Reads a path as a JSON string, which carries a control character in text a line can hold, and which never holds the backtick that
 would end the code span around it.

 @param path - a path from the top of the checkout

 @returns the path as JSON text, its backticks escaped where JSON leaves them raw

 @example
 ```ts
 const shown = jsonTextOf('a\nb.ts',);
 ```
 */
function jsonTextOf(path: string,): string {
  /**
   The path as JSON text, which carries a control character in text a line can hold.
   */
  const asJSON = JSON.stringify(path,);
  // JSON leaves a backtick raw, and the code span around this text is written with backticks.
  return asJSON.replaceAll(
    CODE_SPAN_END,
    String.raw`\u0060`,
  );
}

/**
 Reads how a brief shows one path: in a code span as itself, or as a JSON string when the name holds a character a line cannot
 carry.

 A path is text Git gave as bytes, and a brief is a page of lines: a name with a line break, a tab, another control, or the
 backtick that ends a code span cannot stand in it. The JSON string names the same bytes unambiguously, and the brief says what
 that form is where it is used.

 @param path - a path from the top of the checkout

 @returns the path as a brief writes it, inside a code span

 @example
 ```ts
 const shown = aPathAsABriefShowsIt({ path: 'a\nb.ts', },);
 ```
 */
export function aPathAsABriefShowsIt({ path, }: { readonly path: string },): string {
  return aPathNeedsJSONText(path,) ? `\`${jsonTextOf(path,)}\`` : `\`${path}\``;
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
        return `- ${aPathAsABriefShowsIt({ path, },)}`;
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
          ? `- ${aPathAsABriefShowsIt({ path: removed.path, },)} (deleted)`
          : `- ${aPathAsABriefShowsIt({ path: removed.was, },)} (renamed to ${aPathAsABriefShowsIt({ path: removed.now, },)})`;
      },),
      'Say what depended on each of them, and what does its work now or why nothing does.',
    );
  }
  /**
   Whether any path the part names needs the JSON form, which the part explains where it is used.
   */
  const anyPathNeedsJSON = files.some(aPathNeedsJSONText,)
    || removedOrRenamed.some(function needsJSON(removed,) {
      return aPathNeedsJSONText(removed.status === 'deleted' ? removed.path : removed.was,)
        || aPathNeedsJSONText(removed.status === 'deleted' ? removed.path : removed.now,);
    },);
  if (anyPathNeedsJSON) {
    lines.push('A path written in double quotes is shown as a JSON string, because it holds a character a line of this brief cannot carry: read it as the JSON string it is, and open the file it names.',);
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
