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

 @module
 */

import { basename, } from 'node:path';
import { isRecord, } from './read-pi-fields.ts';
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
 The folder names whose files are tests whatever their extension: cargo puts a package's integration tests under `tests/`, and
 such a test is written like the code it tests, so no suffix of its tells it from shipped code.
 */
const TEST_FOLDER_NAMES: ReadonlySet<string> = new Set<string>([
  'tests',
]);

/**
 The names of the files a review reads although no extension of theirs says they are code: the manifests and tool
 configurations that decide how the code beside them is built, checked or run.

 A change to one of them changes what a build, check or run of that folder produces or is allowed to do, while the code that
 runs stays as it was, so a review that answered nothing to review for it left that decision unread. The names are listed instead
 of the extensions they share (`.toml`, `.json`, `.jsonc`, `.yaml`, `.yml`), because an extension cannot tell a manifest from a
 fixture: on 2026-11-13 this repository held 1345 files of those extensions under `package/`, 487 of them inside the fixture,
 evidence, performance, design and witness folders, among them the 92 TOML specification cases named `sample`, and a review of a
 change to one of those must still answer nothing to review.

 A name list costs the manifest of a tool nobody has added here yet, which goes unread until its name is listed; each name below
 is one this repository's packages or its root carry, or a standard manifest of a tool a package of this repository could run (a
 pinned toolchain, `rustfmt`, `cargo-deny`, a Gradle build), listed so that its first appearance is reviewed rather than missed.
 */
const BUILD_FILE_NAME_LISTS: readonly (readonly string[])[] = [
  // The folder's own manifest and task file, which say what it builds, from which dependencies, and how its tasks run.
  [
    'mise.toml',
    'mise.local.toml',
    'package.json',
    'Cargo.toml',
    'build.gradle.kts',
    'settings.gradle.kts',
    'gradle.properties',
  ],
  // The versions of a dependency every package is built against.
  [
    'pnpm-workspace.yaml',
    'rust-catalog.toml',
    'bunfig.toml',
    'toolchain.plan.jsonc',
  ],
  // What the compiler, the linters, the tests and the mutation checks are told to do.
  [
    'clippy.toml',
    'rust-toolchain.toml',
    'rustfmt.toml',
    'deny.toml',
    'dprint.json',
    'monochromatic-lint.config.jsonc',
    'monochromatic-lint.prerequisites.jsonc',
    'file-enforcer.config.jsonc',
    'mutants.toml',
  ],
  // What the build produces beside the code, and what the program that runs it is allowed to do.
  [
    'about.toml',
    'wrangler.toml',
    'node.config.json',
    'sea-config.json',
  ],
];

/**
 The words the name of a build decision begins with, which a variant extends after a dot (`tsconfig.json` and
 `tsconfig.build.json`, `nextest.toml` and `nextest.windows-gnu.toml`, `Containerfile` and `Containerfile.release`), so a list of
 whole names would miss the next variant. The word alone names a file too (`Containerfile`), and a note that merely begins with
 the word (`ContainerfileREADME.md`) is not one of these.
 */
const BUILD_FILE_NAME_WORDS: readonly string[] = [
  'Containerfile',
  'Dockerfile',
  'nextest',
  'tsconfig',
];

/**
 The ends of the names a review reads as build decisions: a container file named for what it builds
 (`native-test-base.Containerfile`) ends with its tool rather than beginning with it.
 */
const BUILD_FILE_NAME_ENDINGS: readonly string[] = [
  '.Containerfile',
  '.Dockerfile',
];

/**
 The ends of the paths a review reads as build decisions, each anchored to the folder that holds it: `.cargo/config.toml` is a
 cargo configuration while a file inside a folder named `foo.cargo` is not, and the dprint package's `index.json` is the format
 every file of this repository is checked against, which its own generic name would never say.
 */
const BUILD_FILE_PATH_ENDINGS: readonly string[] = [
  '.cargo/config.toml',
  'config/dprint/index.json',
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

 A change of code or of a build decision is read as `code-and-build-decisions`, and a change of tests and fixtures alone as
 `tests-and-fixtures`, whose reviewer is asked to plant a break under each changed test. A range that changes nothing a review
 reads is `nothing-a-review-reads`.
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
     build decisions among what the ranges touched, without the tests and fixtures beside them.
     */
    readonly files: readonly string[];
  }
    | {
      /**
       The change is tests and fixtures alone, so the tests themselves are what a reviewer can show something about.
       */
      readonly kind: 'tests-and-fixtures';

      /**
       The tests and fixtures the reviewer is given, each a path from the top of the checkout, in the order git named them: every
       file the ranges touched under the source folder, which are tests and fixtures and nothing else.
       */
      readonly files: readonly string[];
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
 Says whether a path is a manifest or a tool configuration that decides how the code beside it is built, checked or run.

 @param path - a path from the top of the checkout

 @returns whether the path is one of the build decisions a reviewer reads

 @example
 ```ts
 const isABuildDecision = isABuildDecisionAReviewerReads('package/a/Cargo.toml',);
 ```
 */
function isABuildDecisionAReviewerReads(path: string,): boolean {
  /**
   The file's own name, which the name lists are about.
   */
  const name = basename(path,);
  return BUILD_FILE_NAME_LISTS.some(function holdsTheName(names,) {
    return names.includes(name,);
  },)
    || BUILD_FILE_NAME_WORDS.some(function holdsTheWord(word,) {
      return (name === word) || name.startsWith(`${word}.`,);
    },)
    || BUILD_FILE_NAME_ENDINGS.some(function endsWithTheEnding(ending,) {
      return name.endsWith(ending,);
    },)
    || BUILD_FILE_PATH_ENDINGS.some(function endsWithThePathEnding(ending,) {
      // A git path is separated by `/` alone, and the ending is anchored to the folder that holds it, so `.cargo/config.toml`
      // matches `package/a/.cargo/config.toml` and not `package/a/foo.cargo/config.toml`; a review of the whole checkout reads
      // paths from its top, where the same file carries no folder before its name.
      return (path === ending) || path.endsWith(`/${ending}`,);
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
  return paths.filter(function isAFileAReviewerReads(path,) {
    return (isCodeAReviewerReads(path,) || isABuildDecisionAReviewerReads(path,))
      && (!isATestOrAFixture(path,));
  },);
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
 Reads what a review of one change reads: the diff of every range under the source folder, and the files a reviewer is given,
 which are the code files and build decisions the reviewed commit holds, the tests and fixtures it holds when the change is tests
 and fixtures alone, or none.

 @param git - how one git question is run

 @param checkoutFolder - the checkout the ranges are read from

 @param ranges - what changed, each as `git diff` takes one

 @param source - the folder whose changes are reviewed, from the top of the checkout

 @param commit - the commit under review, whose tree says which files a reviewer can read

 @returns the diff and what kind of change it is, with the files to read

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
  /**
   The touched paths the reviewed commit holds.
   */
  const readable = touched.filter(function isHeldByTheCommit(path,) {
    return held.has(path,);
  },);
  /**
   The code and build decisions among them, which a reviewer reads whenever the change leaves any.
   */
  const files = codeFilesAmong({ paths: readable, },);
  /**
   The diff of every range, in the order the ranges were given.
   */
  const diff = diffs.join('\n',);
  if (files.length > 0) {
    return {
      diff,
      kind: 'code-and-build-decisions',
      files,
    };
  }
  /**
   The tests and fixtures among them, which are read only when they are the whole change: a file a review reads no part of, a
   `.cjs` source or a note, leaves the change itself unread, and a review of its tests alone would say nothing about it.
   */
  const testsAndFixtures = testsAndFixturesAmong({ paths: readable, },);
  return ((testsAndFixtures.length > 0) && (testsAndFixtures.length === readable.length))
    ? {
      diff,
      kind: 'tests-and-fixtures',
      files: testsAndFixtures,
    }
    : {
      diff,
      kind: 'nothing-a-review-reads',
    };
}
