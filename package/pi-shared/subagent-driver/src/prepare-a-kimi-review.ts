/**
 Prepares one Kimi K3.1 Pro review: its worktree at the commit under review, the diff of the range, the list of changed files,
 and the brief, each in the folder the reviewer writes into.

 The range names what changed; the worktree is the code as it stands at the commit, which may be newer than the range's end.

 @module
 */

import {
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';
import {
  briefOf,
  DEFAULT_REVIEW_SOURCE,
  KIMI_FALLBACK_MODEL,
  KIMI_FALLBACK_PROVIDER,
  type KimiReview,
} from './kimi-reviews.ts';
import {
  promptFileOf,
  worktreeOfARun,
} from './pi-runs-folders.ts';
import type { RunCommand, } from './run-command.ts';

/**
 What a review's run and worktree are called: `kimi-review-<name>`, which is also what puts the run outside the Neuralwatt
 pools.
 */
const REVIEW_RUN_PREFIX = 'kimi-review-';

/**
 The branch a review's worktree checks out.
 */
const REVIEW_BRANCH_PREFIX = 'test/kimi-review-';

/**
 The folder inside a review's folder that its reproduction scripts go in.
 */
const REPRODUCTIONS_FOLDER = 'reproductions';

/**
 The file the range's diff is written to.
 */
const DIFF_FILE = 'change.diff';

/**
 The suffix of a Rust source file, which a review may be given.
 */
const RUST_SUFFIX = '.rs';

/**
 The suffix of a TypeScript source file, which a review may be given.
 */
const TYPESCRIPT_SUFFIX = '.ts';

/**
 The part of a test file's name that leaves it out of a review: a reviewer given only tests reads no product code.
 */
const RUST_TEST_PART = '_tests.rs';

/**
 The suffix of a TypeScript test file, which a review is not given.
 */
const TYPESCRIPT_TEST_SUFFIX = '.test.ts';

/**
 The part of a path that holds a test's own support code, which a review is not given.
 */
const TEST_SUPPORT_PART = 'test_support';

/**
 The part of a file's name that makes it a stand-in for another program, which a review is not given.
 */
const STAND_IN_PART = 'stand-in-';

/**
 How many times a worktree add is tried, which a repository under load can need: the first two attempts once left no worktree
 and the review was never started.
 */
const WORKTREE_ADD_ATTEMPTS = 3;

/**
 The review could not be prepared, so no run of it was started.
 */
export class ReviewNotPreparedError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'ReviewNotPreparedError';
}

/**
 What one prepared review is started with.
 */
export type PreparedReview = {
  /**
   The review as the list gave it.
   */
  readonly review: KimiReview;

  /**
   The run's name.
   */
  readonly runName: string;

  /**
   The branch its worktree checks out.
   */
  readonly branch: string;

  /**
   Where the reviewer works.
   */
  readonly worktree: string;

  /**
   Where the reviewer writes its answer and its reproductions.
   */
  readonly runFolder: string;

  /**
   The brief.
   */
  readonly promptFile: string;

  /**
   How many files the reviewer is told to read.
   */
  readonly filesRead: number;
};

/**
 Reads whether a path is a source file a review is given.

 @param path - the file the range changed

 @param worktreeHasIt - whether the worktree at the reviewed commit holds the file

 @returns whether the reviewer is told to read it

 @example
 ```ts
 const read = isReviewed({ path: 'a/src/b.ts', worktreeHasIt: true, });
 ```
 */
export function isReviewed({
  path,
  worktreeHasIt,
}: {
  readonly path: string;
  readonly worktreeHasIt: boolean;
},): boolean {
  if (!(path.endsWith(RUST_SUFFIX,) || path.endsWith(TYPESCRIPT_SUFFIX,))) {
    return false;
  }
  if (
    path.endsWith(RUST_TEST_PART,)
      || path.endsWith(TYPESCRIPT_TEST_SUFFIX,)
      || path.includes(TEST_SUPPORT_PART,)
      || path.includes(STAND_IN_PART,)
  ) {
    return false;
  }
  return worktreeHasIt;
}

/**
 Reads whether a path is there.

 @param path - the path to read

 @returns whether the file system holds it

 @example
 ```ts
 const there = await pathIsThere({ path: '/a/b.ts', });
 ```
 */
async function pathIsThere({
  path,
}: {
  readonly path: string;
},): Promise<boolean> {
  try {
    await readFile(path,);
    return true;
  }
  catch (error) {
    if (errorCodeOf(error,) === 'ENOENT') {
      return false;
    }
    throw error;
  }
}

/**
 Runs one git command and insists on an answer.

 @param run - how a command is run

 @param repository - the checkout to ask

 @param words - the command's arguments

 @returns what it printed

 @throws ReviewNotPreparedError when git refused

 @example
 ```ts
 const diff = await gitOf({ run: runCommandWithChildProcess, repository: '/repo', words: ['diff', 'a...b',], });
 ```
 */
async function gitOf({
  run,
  repository,
  words,
}: {
  readonly run: RunCommand;
  readonly repository: string;
  readonly words: readonly string[];
},): Promise<string> {
  /**
   What git printed.
   */
  const shown = await run({
    command: 'git',
    words: [
      '-C',
      repository,
      ...words,
    ],
  },);
  if (shown.status !== 0) {
    throw new ReviewNotPreparedError(`git ${words.join(' ',)} answered ${String(shown.status,)}: ${shown.problem
      ?? shown.stderr
        .trim()}`,);
  }
  return shown.stdout;
}

/**
 Tries one worktree add, and the next one when it left none.

 @param review - the review to make the worktree for

 @param branch - the branch it checks out

 @param repository - the checkout that holds the worktrees folder

 @param run - how a command is run

 @param attempt - which attempt this is, counting from one

 @returns where the reviewer works

 @throws ReviewNotPreparedError when the last attempt left no worktree

 @example
 ```ts
 const worktree = await worktreeAttempt({ review, branch, repository, run, attempt: 1 });
 ```
 */
async function worktreeAttempt({
  review,
  branch,
  repository,
  run,
  attempt,
}: {
  readonly review: KimiReview;
  readonly branch: string;
  readonly repository: string;
  readonly run: RunCommand;
  readonly attempt: number;
},): Promise<string> {
  /**
   Where the reviewer works.
   */
  const worktree = worktreeOfARun({
    repository,
    name: `${REVIEW_RUN_PREFIX}${review.name}`,
  },);
  if (await pathIsThere({ path: join(
    worktree,
    '.git',
  ), },)) {
    return worktree;
  }
  if (attempt > WORKTREE_ADD_ATTEMPTS) {
    throw new ReviewNotPreparedError(`${REVIEW_RUN_PREFIX}${review.name}: no worktree at ${worktree} after ${String(WORKTREE_ADD_ATTEMPTS,)} attempts`,);
  }
  /**
   Whether the branch already exists, in which case the worktree checks it out rather than making one.
   */
  const branchThere = await run({
    command: 'git',
    words: [
      '-C',
      repository,
      'rev-parse',
      '--verify',
      '--quiet',
      `refs/heads/${branch}`,
    ],
  },);
  await run({
    command: 'git',
    words: [
      '-C',
      repository,
      'worktree',
      'add',
      ...((branchThere.status === 0)
        ? [
          worktree,
          branch,
        ]
        : [
          '-b',
          branch,
          worktree,
          review.commit,
        ]),
    ],
  },);
  return await worktreeAttempt({
    review,
    branch,
    repository,
    run,
    attempt: attempt + 1,
  },);
}

/**
 Prepares one review.

 @param review - the review to prepare

 @param repository - the checkout that holds the worktrees folder and the code under review

 @param runsFolder - the folder the driver writes one run folder per run into

 @param reviewsFolder - the folder every review writes into

 @param run - how a command is run

 @returns what the review is started with

 @throws ReviewNotPreparedError when the worktree, the diff or the file list could not be made

 @example
 ```ts
 const prepared = await prepareKimiReview({ review, repository: '/repo', runsFolder: '/runs', reviewsFolder: '/reviews', run: runCommandWithChildProcess, });
 ```
 */
export async function prepareKimiReview({
  review,
  repository,
  runsFolder,
  reviewsFolder,
  run,
}: {
  readonly review: KimiReview;
  readonly repository: string;
  readonly runsFolder: string;
  readonly reviewsFolder: string;
  readonly run: RunCommand;
},): Promise<PreparedReview> {
  /**
   The run's name, which names its unit and its run folder.
   */
  const runName = `${REVIEW_RUN_PREFIX}${review.name}`;
  /**
   The branch its worktree checks out.
   */
  const branch = `${REVIEW_BRANCH_PREFIX}${review.name}`;
  /**
   Where the reviewer works.
   */
  const worktree = await worktreeAttempt({
    review,
    branch,
    repository,
    run,
    attempt: 1,
  },);
  /**
   Where the reviewer writes.
   */
  const runFolder = join(
    reviewsFolder,
    review.name,
  );
  await mkdir(
    join(
      runFolder,
      REPRODUCTIONS_FOLDER,
    ),
    { recursive: true, },
  );
  /**
   The folder the range is read in, relative to the repository.
   */
  const source = review.source ?? DEFAULT_REVIEW_SOURCE;
  /**
   Where the range's diff is written.
   */
  const diffFile = join(
    runFolder,
    DIFF_FILE,
  );
  await writeFile(
    diffFile,
    await gitOf({
      run,
      repository,
      words: [
        'diff',
        review.range,
        '--',
        source,
      ],
    },),
  );
  /**
   The files the range changed, as git names them.
   */
  const changed = (await gitOf({
    run,
    repository,
    words: [
      'diff',
      '--name-only',
      '--diff-filter=AM',
      review.range,
      '--',
      source,
    ],
  },)).split('\n',)
    .filter(function holdsAPath(path,) {
      return path.trim() !== '';
    },);
  /**
   The changed files the worktree holds, which is what the reviewer can read.
   */
  const reviewed = (await Promise.all(changed.map(async function inWorktree(path,) {
    /**
     The file as the reviewer's worktree spells it.
     */
    const there = join(
      worktree,
      path,
    );
    return {
      path,
      held: await pathIsThere({ path: there, },),
    };
  },),))
    .filter(function isReviewedHere(file: {
      readonly path: string;
      readonly held: boolean
    },) {
      return isReviewed({
        path: file.path,
        worktreeHasIt: file.held,
      },);
    },)
    .map(function asPath(file: {
      readonly path: string;
      readonly held: boolean
    },) {
      return join(
        worktree,
        file.path,
      );
    },);
  if (reviewed.length === 0) {
    throw new ReviewNotPreparedError(
      `${runName}: nothing to review under ${source} in ${review.range}; a reviewer given no files reports nothing, which is not a clean review`,
    );
  }
  /**
   Every file the reviewer reads in full, the diff first.
   */
  const filesToRead = [
    diffFile,
    ...reviewed,
  ];
  /**
   The brief.
   */
  const promptFile = promptFileOf({
    runsFolder,
    name: runName,
  },);
  await writeFile(
    promptFile,
    briefOf({
      review,
      filesToRead,
      runFolder,
      runsFolder,
      reviewsFolder,
    },),
    { encoding: 'utf8', },
  );
  return {
    review,
    runName,
    branch,
    worktree,
    runFolder,
    promptFile,
    filesRead: filesToRead.length,
  };
}

/**
 No summary was left, as a symbol so no summary's text can be mistaken for it.
 */
export const NO_SUMMARY: unique symbol = Symbol('the attempt left no summary',);

/**
 What the next attempt at a review is, after one ended.

 `resume` is the session file the next attempt continues, which is the same session on the fallback provider: the owner's
 "Use neuralwatt/kimi-k3.1-pro-flex whenever Synthetic 429s."
 */
export type ReviewFallback =
  | { readonly resume: false }
    | {
      readonly resume: true;

    /**
     The session file the next attempt continues.
     */
      readonly session: string;

    /**
     What the next attempt runs on.
     */
      readonly provider: string;
      readonly model: string;
    };

/**
 Reads what a finished review attempt leaves for the next one: nothing, unless the model provider answered with a rate limit
 while the review ran on Synthetic.

 @param summaryText - what the attempt's summary says, or {@link NO_SUMMARY}

 @returns whether another attempt is owed, and what it resumes

 @example
 ```ts
 const next = fallbackAfterARateLimit({ summaryText: '{"endReason":"idle"}', });
 ```
 */
export function fallbackAfterARateLimit({
  summaryText,
}: {
  readonly summaryText: string | typeof NO_SUMMARY;
},): ReviewFallback {
  if ((typeof summaryText) === 'symbol') {
    return { resume: false, };
  }
  /**
   What the attempt left last.
   */
  const summary = JSON.parse(summaryText,) as {
    readonly endReason?: unknown;
    readonly rateLimitErrors?: unknown;
    readonly sessionStats?: { readonly sessionFile?: unknown };
  };
  /**
   The session the next attempt continues, when the attempt left one.
   */
  const session = summary.sessionStats
    ?.sessionFile;
  if (
    (summary.endReason !== 'model-error')
      || ((typeof summary.rateLimitErrors) !== 'number')
      || (summary.rateLimitErrors < 1)
      || ((typeof session) !== 'string')
  ) {
    return { resume: false, };
  }
  return {
    resume: true,
    session,
    provider: KIMI_FALLBACK_PROVIDER,
    model: KIMI_FALLBACK_MODEL,
  };
}
