/**
 What a review of a change reads: the range that names what changed, the diff of it under the reviewed source folder, and the
 files a reviewer is given.

 A reviewer given no file reports nothing, which reads as a clean review although it reviewed nothing, so a change that leaves no
 code under the source folder is answered as nothing to review rather than started.

 @module
 */

import type { CommandOutput, } from './run-command.ts';

/**
 How this module asks git a question, which a test replaces with the answers it expects.
 */
export type GitQuestion = (request: {
  readonly words: readonly string[];
  readonly folder: string;
}) => Promise<CommandOutput>;

/**
 The file extensions a review reads, which are the languages the reviewed code is written in.
 */
const REVIEWED_EXTENSIONS: readonly string[] = [
  '.rs',
  '.ts',
];

/**
 The file suffixes a review does not read: the tests that check the change, which a reviewer cannot judge as the change itself,
 and which the package's own tests already run.
 */
const TEST_FILE_SUFFIXES: readonly string[] = [
  '_tests.rs',
  '.test.ts',
];

/**
 The path fragments a review does not read: the fixtures and stand-in programs the tests use, which are not shipped code.
 */
const TEST_FIXTURE_PATH_PARTS: readonly string[] = [
  'test_support',
  'stand-in-',
];

/**
 A git question a review could not get an answer to.
 */
export class ReviewGitError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'ReviewGitError';
}

/**
 What a review of a change reads.
 */
export type ReviewMaterial = {
  /**
   The diff of every range under the source folder, joined in the order the ranges were given.
   */
  readonly diff: string;

  /**
   The files the reviewer is given, each a path from the top of the checkout, in the order git named them.
   */
  readonly files: readonly string[];
};

/**
 Asks git one question in the checkout and answers with what it printed.

 @param git - how one git question is run

 @param checkoutFolder - the checkout the question is about

 @param words - the question's words

 @returns what git printed

 @throws ReviewGitError when git did not answer with a success

 @example
 ```ts
 const shown = await outputOfGit({ git, checkoutFolder: '/repo', words: ['rev-parse', 'main'], },);
 ```
 */
export async function outputOfGit({
  git,
  checkoutFolder,
  words,
}: {
  readonly git: GitQuestion;
  readonly checkoutFolder: string;
  readonly words: readonly string[];
}): Promise<string> {
  /**
   What git answered.
   */
  const answered = await git({
    words,
    folder: checkoutFolder,
  },);
  if (answered.status !== 0) {
    throw new ReviewGitError(
      `git ${words.join(' ',)} in ${checkoutFolder} answered status ${String(answered.status,)}: ${answered.stderr
        .trim()}`,
    );
  }
  return answered.stdout;
}

/**
 What a review read, kept beside its own files so a fix that answers it knows where to start.

 A review reads a commit, the ranges that named the change, and the source folder under it. The branch a review read moves on as
 soon as anyone commits to it, and a fix started at the branch would answer a change nobody reviewed, so the commit itself is
 what is kept.
 */
export type ReviewReading = {
  /**
   The change the review read.
   */
  readonly review: string;

  /**
   The commit under review, which a fix that answers it starts at.
   */
  readonly commit: string;

  /**
   What changed, each as `git diff` takes a range.
   */
  readonly ranges: readonly string[];

  /**
   The folder whose changes were reviewed, from the top of the checkout.
   */
  readonly source: string;
};

/**
 A record of what a review read that cannot be read as one.
 */
export class ReviewReadingError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'ReviewReadingError';
}

/**
 Reads what a review recorded it read.

 @param shown - the record as it was written

 @param file - the file the record came from, for the message

 @returns what the review read

 @throws ReviewReadingError when the record does not name a commit, its ranges and its source folder

 @example
 ```ts
 const reading = readingIn({ shown: '{"review":"a","commit":"abc","ranges":["main...fix/a"],"source":"package/a/src"}', file: 'a/what-the-review-read.json', },);
 ```
 */
export function readingIn({
  shown,
  file,
}: {
  readonly shown: string;
  readonly file: string;
}): ReviewReading {
  /**
   The record, of any shape.
   */
  const kept: unknown = JSON.parse(shown,);
  if ((typeof kept) !== 'object') {
    throw new ReviewReadingError(`${file} holds no JSON object naming what the review read`,);
  }
  /**
   The fields the record was written with.
   */
  const fields = kept as Partial<Record<keyof ReviewReading, unknown>>;
  if (
    ((typeof fields.review) !== 'string') || ((typeof fields.commit) !== 'string')
    || ((typeof fields.source) !== 'string') || (!Array.isArray(fields.ranges,))
    || (!fields.ranges.every(function isARange(range,) {
      return (typeof range) === 'string';
    },))
  ) {
    throw new ReviewReadingError(
      `${file} names no review, commit, ranges and source folder, so where a fix for it would start cannot be read`,
    );
  }
  return {
    review: fields.review,
    commit: fields.commit,
    ranges: fields.ranges as readonly string[],
    source: fields.source,
  };
}

/**
 Reads the range that names what a branch changed, from the integration branch that holds it.

 A branch the integration already merged has an empty `integration...branch`, and a reviewer given it reads nothing. Its own
 change is then the range from the merge that took it: the merge's first parent to the branch's tip.

 @param branch - the branch under review

 @param tip - the branch's tip commit

 @param integration - the integration branch the change was or will be merged into

 @param merges - the integration's own first-parent merges, each as `<merge> <parent>...`

 @returns the range, as `git diff` and `git log` take one

 @example
 ```ts
 const range = rangeOfABranch({ branch: 'fix/a', tip, integration: 'main', merges: [], },);
 ```
 */
export function rangeOfABranch({
  branch,
  tip,
  integration,
  merges,
}: {
  readonly branch: string;
  readonly tip: string;
  readonly integration: string;
  readonly merges: readonly string[];
}): string {
  /**
   The merge that took this branch's tip, when the integration already holds it.
   */
  const takingMerge = merges
    .map(function fieldsOf(line,): readonly string[] {
      // git's own `%H %P` format joins the fields with one space and nothing else.
      return line.trim()
        .split(' ',);
    },)
    .find(function tookTheTip(fields,) {
      return fields.slice(1,)
        .includes(tip,);
    },)?.[0];
  return (takingMerge === undefined)
    ? `${integration}...${branch}`
    : `${takingMerge}^1...${branch}`;
}

/**
 Filters the paths a change touched down to the code a reviewer reads.

 @param paths - the paths a change touched under the source folder

 @returns the code files among them, in the order given

 @example
 ```ts
 const files = codeFilesAmong({ paths: ['src/gate.rs', 'src/gate_tests.rs'], },);
 ```
 */
export function codeFilesAmong({ paths, }: { readonly paths: readonly string[] },): readonly string[] {
  return paths.filter(function isCodeAReviewerReads(path,) {
    return REVIEWED_EXTENSIONS.some(function endsWith(extension,) {
      return path.endsWith(extension,);
    },)
      && (!TEST_FILE_SUFFIXES.some(function isATest(extension,) {
        return path.endsWith(extension,);
      },))
      && (!TEST_FIXTURE_PATH_PARTS.some(function isAFixture(part,) {
        return path.includes(part,);
      },));
  },);
}

/**
 Reads what a review of one change reads: the diff of every range under the source folder, and the code files among what those
 ranges touched that the reviewed commit still holds.

 @param git - how one git question is run

 @param checkoutFolder - the checkout the ranges are read from

 @param ranges - what changed, each as `git diff` takes one

 @param source - the folder whose changes are reviewed, from the top of the checkout

 @param commit - the commit under review, whose tree says which files a reviewer can read

 @returns the diff and the files to read

 @throws ReviewGitError when git did not answer with a success

 @example
 ```ts
 const material = await materialOfAReview({ git, checkoutFolder: '/repo', ranges: ['main...fix/a'], source: 'package/a/src', commit: 'abc123', },);
 ```
 */
export async function materialOfAReview({
  git,
  checkoutFolder,
  ranges,
  source,
  commit,
}: {
  readonly git: GitQuestion;
  readonly checkoutFolder: string;
  readonly ranges: readonly string[];
  readonly source: string;
  readonly commit: string;
}): Promise<ReviewMaterial> {
  /**
   The diff of every range, in the order the ranges were given.
   */
  const diffs: string[] = [];
  /**
   The paths every range touched, each once and in the order git named them.
   */
  const touched: string[] = [];
  /* oxlint-disable eslint/no-await-in-loop -- one range's diff after another, because each range's answer is joined in the order
     the caller gave them. See doc/lint/sequential-awaits.md. */
  for (const range of ranges) {
    diffs.push(await outputOfGit({
      git,
      checkoutFolder,
      words: [
        'diff',
        range,
        '--',
        source,
      ],
    },),);
    for (const path of (await outputOfGit({
      git,
      checkoutFolder,
      words: [
        'diff',
        '--name-only',
        '--diff-filter=AM',
        range,
        '--',
        source,
      ],
    },)).split('\n',)) {
      if ((path !== '') && (!touched.includes(path,))) {
        touched.push(path,);
      }
    }
  }
  /* oxlint-enable eslint/no-await-in-loop */
  /**
   The paths the reviewed commit still holds, so a file a later commit deleted is not given to a reviewer to read.
   */
  const held = new Set((await outputOfGit({
    git,
    checkoutFolder,
    words: [
      'ls-tree',
      '--name-only',
      '-r',
      commit,
      '--',
      source,
    ],
  },)).split('\n',)
    .filter(function isPath(line,) {
      return line !== '';
    },),);
  return {
    diff: diffs.join('\n',),
    files: codeFilesAmong({ paths: touched, },)
      .filter(function isHeldByTheCommit(path,) {
        return held.has(path,);
      },),
  };
}
