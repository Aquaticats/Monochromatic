/**
 What a review of a change reads: the range that names what changed, the diff of it under the reviewed source folder, and the
 files a reviewer is given.

 A reviewer given no file reports nothing, which reads as a clean review although it reviewed nothing, so a range that leaves
 nothing to read under the source folder is answered as nothing to review rather than started. A manifest or a configuration
 that decides how the code beside it is built, checked or run is something to read, so a change of one is reviewed.

 A range whose change is tests and fixtures alone leaves no code to judge, and a reviewer given nothing for it would report
 nothing, so those files are read too, as a change to tests: the brief asks the reviewer to plant a break under each changed test
 and say whether it fails. Where the change's own code is written in a language a review does not read, as a `.cjs` file is, the
 range is still answered as nothing to review, because a review of its tests alone would leave the change itself unread.

 Which of those a range is decided from every way a range can change a path, as `git diff --name-status` reports it, and not from
 added and modified paths alone: a range that deletes or renames code is a change of code even when every file the commit still
 holds at a touched path is a test, and its brief names what was removed or renamed and asks what depended on it.

 @module
 */

import { isRecord, } from './read-pi-fields.ts';
import { isABuildDecisionAReviewerReads, } from './build-decisions-a-review-reads.ts';
import type { CommandOutput, } from './run-command.ts';
import {
  type ChangedPath,
  type PathKind,
  type RemovedOrRenamedPath,
  changedPathsIn,
  kindOfChangedPath,
  readablePathsOf,
  removedOrRenamedAmong,
} from './what-a-range-changed.ts';

// The record of what a range removed or renamed, and the refusal of a status this writer cannot read, are read and kept where the
// range's statuses are; they are named from here because a review's material is what a caller of this module holds.
export {
  type RemovedOrRenamedPath,
  ReviewDiffError,
} from './what-a-range-changed.ts';

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
 The folder names whose files are tests whatever their extension: cargo puts a package's integration tests under `tests/`, and
 such a test is written like the code it tests, so no suffix of its tells it from shipped code.
 */
const TEST_FOLDER_NAMES: ReadonlySet<string> = new Set<string>([
  'tests',
]);

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

 A change of code or of a build decision is read as `code-and-build-decisions`, and a change of tests and fixtures alone as
 `tests-and-fixtures`, whose reviewer is asked to plant a break under each changed test. A range that changes nothing a review
 reads is `nothing-a-review-reads`.

 A review's kind is decided from every path a range changed, whatever Git's status for it: a deletion or a rename of code is a
 change of code even when the paths the commit still holds are tests alone, because a review of the tests would leave the removal
 unread.
 */
export type ReviewMaterial = {
  /**
   The diff of every range under the source folder, joined in the order the ranges were given.
   */
  readonly diff: string;
} & (
  | {
    /**
     The change holds code or a build decision, which a reviewer judges as the change itself.
     */
    readonly kind: 'code-and-build-decisions';

    /**
     The files the reviewer is given, each a path from the top of the checkout, in the order git named them: the code and the
     build decisions among what the ranges touched that the commit still holds, without the tests and fixtures beside them.
     */
    readonly files: readonly string[];

    /**
     The paths the ranges removed or renamed, which the commit no longer holds as files to read in full.
     */
    readonly removedOrRenamed: readonly RemovedOrRenamedPath[];
  }
    | {
      /**
       The change is tests and fixtures alone, so the tests themselves are what a reviewer can show something about.
       */
      readonly kind: 'tests-and-fixtures';

      /**
       The tests and fixtures the reviewer is given, each a path from the top of the checkout, in the order git named them: the
       tests and fixtures among what the ranges touched that the commit still holds, and nothing else.
       */
      readonly files: readonly string[];

      /**
       The test and fixture paths the ranges removed or renamed, which the commit no longer holds as files to read in full.
       */
      readonly removedOrRenamed: readonly RemovedOrRenamedPath[];
    }
    | {
      /**
       The ranges changed nothing under the source folder that a review reads, so there is no review to start.
       */
      readonly kind: 'nothing-a-review-reads';
    }
);

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
  if (!isRecord(kept,)) {
    throw new ReviewReadingError(`${file} holds no JSON object naming what the review read`,);
  }
  /**
   The fields the record was written with, kept as the record holds them.
   */
  const fields = {
    review: kept.review,
    commit: kept.commit,
    source: kept.source,
    ranges: kept.ranges,
  };
  if (
    ((typeof fields.review) !== 'string') || ((typeof fields.commit) !== 'string')
      || ((typeof fields.source) !== 'string')
      || (!Array.isArray(fields.ranges,))
  ) {
    throw new ReviewReadingError(
      `${file} names no review, commit, ranges and source folder, so where a fix for it would start cannot be read`,
    );
  }
  /**
   The ranges the record kept, which a fix reads the review against.
   */
  const ranges = fields.ranges
    .filter(function isARange(range,): range is string {
      return (typeof range) === 'string';
    },);
  if (ranges.length
    !== fields.ranges
      .length) {
    throw new ReviewReadingError(
      `${file} names a range that is no text, so where a fix for it would start cannot be read`,
    );
  }
  return {
    review: fields.review,
    commit: fields.commit,
    ranges,
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
 Says whether a path is a file a review reads because of the language it is written in.

 @param path - a path from the top of the checkout

 @returns whether the path is code a reviewer reads

 @example
 ```ts
 const isCode = isCodeAReviewerReads('src/gate.rs',);
 ```
 */
function isCodeAReviewerReads(path: string,): boolean {
  return REVIEWED_EXTENSIONS.some(function endsWith(extension,) {
    return path.endsWith(extension,);
  },);
}

/**
 Says whether a path sits in a folder of tests, whose files are tests whatever their extension.

 @param path - a path from the top of the checkout

 @returns whether a folder above the file is named for tests

 @example
 ```ts
 const isATest = isInATestFolder('package/cargo-shadow/cli/tests/shim.rs',);
 ```
 */
function isInATestFolder(path: string,): boolean {
  /**
   Every segment of the path.
   */
  const segments = path.split('/',);
  /**
   The path's folders, which are every segment before the file's own name.
   */
  const folders = segments.slice(
    0,
    -1,
  );
  return folders.some(function isATestFolder(folder,) {
    return TEST_FOLDER_NAMES.has(folder,);
  },);
}

/**
 Says whether a path is one of the files a review leaves out beside the change: the tests that check it, which a reviewer cannot
 judge as the change itself, and the fixtures and stand-in programs they use.

 @param path - a path from the top of the checkout

 @returns whether the path is a test or a fixture

 @example
 ```ts
 const isATest = isATestOrAFixture('src/gate_tests.rs',);
 ```
 */
function isATestOrAFixture(path: string,): boolean {
  return TEST_FILE_SUFFIXES.some(function isATest(extension,) {
    return path.endsWith(extension,);
  },)
    || TEST_FIXTURE_PATH_PARTS.some(function isAFixture(part,) {
      return path.includes(part,);
    },)
    || isInATestFolder(path,);
}

/**
 Says whether a path is a file a reviewer judges as the change itself: code, or a build decision that decides how that code is
 built, checked or run.

 @param path - a path from the top of the checkout

 @returns whether the path is a file a review of code reads

 @example
 ```ts
 const isRead = isAFileAReviewerReads('src/gate.rs',);
 ```
 */
function isAFileAReviewerReads(path: string,): boolean {
  return (isCodeAReviewerReads(path,) || isABuildDecisionAReviewerReads(path,))
    && (!isATestOrAFixture(path,));
}

/**
 Reads what a path is to a review.

 @param path - a path from the top of the checkout

 @returns the change itself, a test, or neither

 @example
 ```ts
 const kind = pathKindOf('src/gate.rs');
 ```
 */
function pathKindOf(path: string,): PathKind {
  if (isAFileAReviewerReads(path,)) {
    return 'code';
  }
  return isATestOrAFixture(path,) ? 'test' : 'unread';
}

/**
 Filters the paths a change touched down to the files a reviewer reads: the code, and the manifests and tool configurations that
 decide how that code is built, checked or run.

 @param paths - the paths a change touched under the source folder

 @returns the files among them, in the order given

 @example
 ```ts
 const files = codeFilesAmong({ paths: ['src/gate.rs', 'src/gate_tests.rs', 'Cargo.toml'], },);
 ```
 */
export function codeFilesAmong({ paths, }: { readonly paths: readonly string[] },): readonly string[] {
  return paths.filter(isAFileAReviewerReads,);
}

/**
 Filters the paths a change touched down to the tests that check it and the fixtures those tests use, which a review of code
 leaves out and a review of a change to tests alone reads.

 @param paths - the paths a change touched under the source folder

 @returns the tests and fixtures among them, in the order given

 @example
 ```ts
 const tests = testsAndFixturesAmong({ paths: ['src/gate.rs', 'src/gate_tests.rs', 'src/stand-in-git.ts'], },);
 ```
 */
export function testsAndFixturesAmong({ paths, }: { readonly paths: readonly string[] },): readonly string[] {
  return paths.filter(function isATestOrAFixturePath(path,) {
    return isATestOrAFixture(path,);
  },);
}

/**
 Reads what a review of one change reads: the diff of every range under the source folder, the files a reviewer is given, and the
 paths the ranges removed or renamed.

 The kind is decided from every path a range changed, whatever Git's status for it, so a range that deletes or renames code is a
 change of code even when every file the commit still holds at a touched path is a test.

 @param git - how one git question is run

 @param checkoutFolder - the checkout the ranges are read from

 @param ranges - what changed, each as `git diff` takes one

 @param source - the folder whose changes are reviewed, from the top of the checkout

 @param commit - the commit under review, whose tree says which files a reviewer can read

 @returns the diff and what kind of change it is, with the files to read and the paths it removed or renamed

 @throws ReviewGitError when git did not answer with a success

 @throws ReviewDiffError when git reported a change this writer cannot read as code or as tests

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
   Every path a range changed, each once: what tells one change from another across the ranges is its status and both names.
   */
  const changes = new Map<string, ChangedPath>();
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
    for (const one of changedPathsIn({
      printed: await outputOfGit({
        git,
        checkoutFolder,
        words: [
          'diff',
          '--name-status',
          '-z',
          '--find-renames',
          range,
          '--',
          source,
        ],
      },),
    },)) {
      /**
       What tells this change from another one the ranges named.
       */
      const key = `${one.status} ${one.from ?? ''} ${one.path}`;
      if (!changes.has(key,)) {
        changes.set(
          key,
          one,
        );
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
  /**
   Every path the ranges changed, in the order git named them.
   */
  const changed = [...changes.values(),];
  /**
   What each changed path is to a review, which decides the kind of review the ranges ask for.
   */
  const kinds = changed.map(function kindOfOne(one,) {
    return kindOfChangedPath({
      changed: one,
      pathKindOf,
    },);
  },);
  /**
   The paths the ranges left as files the commit holds, which a review of either kind reads in full.
   */
  const readable = readablePathsOf({
    changed,
    held,
  },);
  /**
   The diff of every range, in the order the ranges were given.
   */
  const diff = diffs.join('\n',);
  if (kinds.includes('code',)) {
    return {
      diff,
      kind: 'code-and-build-decisions',
      files: codeFilesAmong({ paths: readable, },),
      removedOrRenamed: removedOrRenamedAmong({
        changed,
        kind: 'code',
        pathKindOf,
      },),
    };
  }
  return ((changed.length > 0) && kinds.every(function isATest(kind,) {
    return kind === 'test';
  },))
    ? {
      diff,
      kind: 'tests-and-fixtures',
      files: testsAndFixturesAmong({ paths: readable, },),
      removedOrRenamed: removedOrRenamedAmong({
        changed,
        kind: 'test',
        pathKindOf,
      },),
    }
    : {
      diff,
      kind: 'nothing-a-review-reads',
    };
}
