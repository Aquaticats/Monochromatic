/**
 What a review of a change reads: the range that names what changed, the diff of it under the reviewed source folder, and the
 files a reviewer is given.

 A reviewer given no file reports nothing, which reads as a clean review although it reviewed nothing, so a range that leaves
 nothing to read under the source folder is answered as nothing to review rather than started, and that answer says what the
 ranges changed there and why a review reads none of it. A manifest or a configuration that decides how the code beside it is
 built, checked or run is something to read, so a change of one is reviewed.

 A range whose change is tests and fixtures alone leaves no code to judge, and a reviewer given nothing for it would report
 nothing, so those files are read too, as a change to tests: the brief asks the reviewer to plant a break under each changed test
 and say whether it fails. A file beside a test that is neither a test nor a fixture nor prose is read with them, because the
 writer cannot tell whether a test reads it and a test and the recorded answer it reads must never be answered as nothing to
 review, while a note beside a test leaves the change not a change to tests alone.

 Which file is a test or a fixture, and which extension is code and which is left out, is `which-files-a-review-reads.ts`'s
 answer, measured against what this repository tracks. A file of an extension in neither list is a kind this writer does not
 know: a review reads it as code rather than drop it, and the material names it, so the line the writer prints says that a kind
 nobody listed was read.

 What a review of a change reads, and what it leaves out, follows the table in `plan-of-a-review.ts`, which is one row for every
 combination of the kinds a range can hold: a file a review cannot judge never decides for the files beside it, and a test is read
 whatever else the range holds.

 @module
 */

import { basename, } from 'node:path';
import { isRecord, } from './read-pi-fields.ts';
import { pathsInAListing, } from './names-a-git-listing-prints.ts';
import {
  type LeftOutFile,
  planOfAChange,
} from './plan-of-a-review.ts';
import type { CommandOutput, } from './run-command.ts';
import {
  type KindOfAFileAReviewReads,
  kindOfAFileAReviewReads,
} from './which-files-a-review-reads.ts';

/**
 How this module asks git a question, which a test replaces with the answers it expects.
 */
export type GitQuestion = (request: {
  readonly words: readonly string[];
  readonly folder: string;

  /**
   How what it prints is read: as text, or one character per byte, which a listing of paths needs so a name comes back as the
   bytes Git printed rather than as text a quote or a UTF-8 replacement changes.
   */
  readonly readAs?: 'text' | 'one-character-per-byte';
}) => Promise<CommandOutput>;

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
 What a review of a change reads when it reads anything, which is the one answer its brief and the line it prints are written
 from: the files, the code among them, the files of a kind no list names, and the files it leaves out with the reason.

 Every path of the change is in exactly one of `files` and `filesLeftOut`. A file of a kind no list names is code wherever it is
 read, beside a test or not, so it is in `filesOfCode` and in `filesOfAnUnlistedKind`, and the brief asks the project's
 requirements for it and the line prints the notice.
 */
export type AReviewReads = {
  /**
   The files the reviewer is given, each a path from the top of the checkout, in the order git named them, without the files the
   review leaves out.
   */
  readonly files: readonly string[];

  /**
   The files the change removed, each a path from the top of the checkout, in the order git named them: a review reads them in the
   change's diff, because the reviewed commit no longer holds them. They are read as the kind their path says, so a removed code
   file is code the brief asks the project's requirements for.
   */
  readonly filesRemoved: readonly string[];

  /**
   The tests among the files this review reads in full, in the order given. A brief asks a break under each of them, because the
   reviewed commit holds them; a test the change removed is in `filesRemoved` and is read in the diff instead.
   */
  readonly filesOfTests: readonly string[];

  /**
   The code among the files this review reads, in the order given, a removed file and a file of a kind no list names being code
   wherever they are read. Its presence is what makes a change to tests ask the project's requirements as well as the breaks under
   the tests.
   */
  readonly filesOfCode: readonly string[];

  /**
   The files among those whose extension is in no list of `which-files-a-review-reads.ts`, in the order given. The line the
   writer prints names them, because a kind nobody listed must not be read as though somebody had.
   */
  readonly filesOfAnUnlistedKind: readonly string[];

  /**
   The files the change touched that this review does not read, each with the reason its brief gives, in the order git named
   them.
   */
  readonly filesLeftOut: readonly LeftOutFile[];
};

/**
 What a review of a change reads.

 A change that holds a test or a fixture is read as `tests-and-fixtures`, whose reviewer is asked to plant a break under each
 changed test, whatever else the change holds. A change that holds code or a build decision, and no test, is
 `code-and-build-decisions`, which a reviewer judges against the project's requirements. A range that changes nothing a review
 reads is `nothing-a-review-reads`, with the words that say why.

 Both of the first two name the files they leave out, with the reason, so a file a review cannot judge never decides for the files
 beside it.
 */
export type ReviewMaterial = {
  /**
   The diff of every range under the source folder, joined in the order the ranges were given.
   */
  readonly diff: string;
} & (
  | ({
    /**
     The change holds code or a build decision, and no test, which a reviewer judges as the change itself.
     */
    readonly kind: 'code-and-build-decisions';
  } & AReviewReads)
    | ({
      /**
       The change holds a test or a fixture, so the tests themselves are what a reviewer can show something about.
       */
      readonly kind: 'tests-and-fixtures';
    } & AReviewReads)
    | {
      /**
       The ranges changed nothing under the source folder that a review reads, so there is no review to start.
       */
      readonly kind: 'nothing-a-review-reads';

      /**
       The words that say why nothing is read: that the ranges changed nothing there, or which files they changed and the rule a
       review reads a change by. They follow the line the writer prints, so a refusal names its cause instead of reading as an
       empty review.
       */
      readonly whyNothingIsRead: string;
    }
);

/**
 Asks git one question in the checkout and answers with what it printed.

 @param git - how one git question is run

 @param checkoutFolder - the checkout the question is about

 @param words - the question's words

 @param readAs - how what it prints is read, as text when the caller names nothing

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
  readAs = 'text',
}: {
  readonly git: GitQuestion;
  readonly checkoutFolder: string;
  readonly words: readonly string[];
  readonly readAs?: 'text' | 'one-character-per-byte';
}): Promise<string> {
  /**
   What git answered.
   */
  const answered = await git({
    words,
    folder: checkoutFolder,
    readAs,
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
 Says what kind of file a changed path is to this review, which is what decides whether the change is read as code or as tests.

 A name that decides a build is code although its extension says data, and that rule lives in this module because the names do. The
 test and fixture lists are asked first, as they are everywhere: a `Cargo.toml` inside a `fixtures/` folder is that fixture, and no
 build decision.

 @param path - a path from the top of the checkout

 @returns the kind of file the path is

 @example
 ```ts
 const kind = kindOfAChangedPath('package/a/src/gate.unit.test.ts',);
 ```
 */
function kindOfAChangedPath(path: string,): KindOfAFileAReviewReads {
  /**
   The kind the lists of `which-files-a-review-reads.ts` give the file.
   */
  const kind = kindOfAFileAReviewReads({ path, },);
  if ((kind === 'a-test') || (kind === 'a-fixture')) {
    return kind;
  }
  return ((kind === 'code') || isABuildDecisionAReviewerReads(path,)) ? 'code' : kind;
}

/**
 Filters the paths a change touched down to the files a reviewer reads: the code, the files of a kind no list names, and the
 manifests and tool configurations that decide how that code is built, checked or run.

 @param paths - the paths a change touched under the source folder

 @returns the files among them, in the order given

 @example
 ```ts
 const files = codeFilesAmong({ paths: ['src/gate.rs', 'src/gate_tests.rs', 'Cargo.toml'], },);
 ```
 */
export function codeFilesAmong({ paths, }: { readonly paths: readonly string[] },): readonly string[] {
  return paths.filter(function isAFileAReviewerReads(path,) {
    /**
     The kind of file the path is.
     */
    const kind = kindOfAChangedPath(path,);
    return (kind === 'code') || (kind === 'a-unlisted-kind');
  },);
}

/**
 Filters the paths a change touched down to the files whose extension is in no list of `which-files-a-review-reads.ts`, which a
 review reads as code and the line the writer prints names, because a kind nobody listed was read as though somebody had.

 @param paths - the paths a change touched under the source folder

 @returns the files of an unlisted kind among them, in the order given

 @example
 ```ts
 const files = filesOfAnUnlistedKindAmong({ paths: ['src/gate.rs', 'src/thing.zig'], },);
 ```
 */
export function filesOfAnUnlistedKindAmong({ paths, }: { readonly paths: readonly string[] },): readonly string[] {
  return paths.filter(function isOfAnUnlistedKind(path,) {
    return kindOfAChangedPath(path,) === 'a-unlisted-kind';
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
    /**
     The kind of file the path is.
     */
    const kind = kindOfAChangedPath(path,);
    return (kind === 'a-test') || (kind === 'a-fixture');
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
        // A rename is asked for as the deletion and the addition it is, like the name listing below, so the diff a reviewer reads
        // holds the old side's contents instead of a similarity index and two names.
        '--no-renames',
        range,
        '--',
        source,
      ],
    },),);
    for (const path of pathsInAListing({
      listing: await outputOfGit({
        git,
        checkoutFolder,
        readAs: 'one-character-per-byte',
        words: [
          'diff',
          '--name-only',
          '-z',
          // A rename is two names, the one taken away and the one put in its place, and a removal is a name too: the kind of a
          // change follows every path the range touched, not only the ones a reviewer can open.
          '--no-renames',
          range,
          '--',
          source,
        ],
      },),
    },)) {
      if (!touched.includes(path,)) {
        touched.push(path,);
      }
    }
  }
  /* oxlint-enable eslint/no-await-in-loop */
  /**
   The paths the reviewed commit still holds, so a file a later commit deleted is not given to a reviewer to read.
   */
  const held = new Set(pathsInAListing({
    listing: await outputOfGit({
      git,
      checkoutFolder,
      readAs: 'one-character-per-byte',
      words: [
        'ls-tree',
        '--name-only',
        '-r',
        '-z',
        commit,
        '--',
        source,
      ],
    },),
  },),);
  /**
   The touched paths the reviewed commit no longer holds, which a review reads in the change's diff: a removal is something the
   range does to the repository, and its kind is that of the path it removed.
   */
  const removed = touched.filter(function isNotHeldByTheCommit(path,) {
    return !held.has(path,);
  },);
  /**
   The diff of every range, in the order the ranges were given.
   */
  const diff = diffs.join('\n',);
  /**
   What this review reads, what it reads only in the diff, and what it leaves out, which is the table of `plan-of-a-review.ts`
   for the kinds every touched path has.
   */
  const plan = planOfAChange({
    paths: touched,
    kinds: touched.map(function kindOf(path,) {
      return kindOfAChangedPath(path,);
    },),
    removed,
  },);
  if (plan.kind === 'code-and-build-decisions') {
    return {
      diff,
      kind: 'code-and-build-decisions',
      files: plan.files,
      filesRemoved: plan.filesRemoved,
      filesOfTests: plan.filesOfTests,
      filesOfCode: plan.filesOfCode,
      filesOfAnUnlistedKind: plan.filesOfAnUnlistedKind,
      filesLeftOut: plan.filesLeftOut,
    };
  }
  if (plan.kind === 'tests-and-fixtures') {
    return {
      diff,
      kind: 'tests-and-fixtures',
      files: plan.files,
      filesRemoved: plan.filesRemoved,
      filesOfTests: plan.filesOfTests,
      filesOfCode: plan.filesOfCode,
      filesOfAnUnlistedKind: plan.filesOfAnUnlistedKind,
      filesLeftOut: plan.filesLeftOut,
    };
  }
  return {
    diff,
    kind: 'nothing-a-review-reads',
    whyNothingIsRead: whyNothingIsRead({
      source,
      changed: touched,
    },),
  };
}

/**
 Reads the words that say why a range that changed nothing a review reads is answered so, which follow that answer's line.

 @param source - the folder whose changes were reviewed, from the top of the checkout

 @param changed - the paths the ranges changed under that folder that the reviewed commit still holds

 @returns the words, without the colon before them

 @example
 ```ts
 const why = whyNothingIsRead({ source: 'package/a/src', changed: ['package/a/src/notes.md'], },);
 ```
 */
function whyNothingIsRead({
  source,
  changed,
}: {
  readonly source: string;
  readonly changed: readonly string[];
},): string {
  if (changed.length === 0) {
    return `it changes no file under ${source}`;
  }
  /**
   The rule a review reads a change by, which the words name before the files that broke it.
   */
  const rule = 'a review reads the code of a change, the files that decide its build, or its tests and fixtures alone';
  /**
   The changed paths as the words list them, each in backticks.
   */
  const listed = changed.map(function asAPath(path,) {
    return `\`${path}\``;
  },)
    .join(', ',);
  return `${rule}, and it changes ${listed}`;
}
