/**
 Turns the revisions a review's range names into commits once, so that every later question about the range is asked of those
 commits and no later read can see another one.

 A branch moves on as soon as anyone commits to it, and this machine merges into its integration many times an hour, so a range
 written `main...fix/a` is a question whose answer changes between two commands: a diff read before a merge and a file list read
 after it can speak of two different changes. A revision is therefore turned into a commit here, once, and the range that is written
 into the brief and the record, the diff, the merge base and the tree listing are all asked of those commits. A reader can ask the
 same range later and get the same answer, because both its sides are commits.

 @module
 */

import {
  type GitQuestion,
  outputOfGit,
} from './a-git-question.ts';

/**
 The revision a range's left side names when it names none, which is what `git diff ...B` reads.
 */
const THE_REVISION_A_RANGE_STARTS_FROM_BY_DEFAULT = 'HEAD';

/**
 The three dots of a range read from its left side's merge base with its right side.
 */
const THREE_DOTS = '...';

/**
 The two dots of a range read as the difference between its two sides.
 */
const TWO_DOTS = '..';

/**
 What one range says once its sides are commits.
 */
export type PinnedRange = {
  /**
   The range, with both sides written as commits, which a reader can ask later and get the same answer.
   */
  readonly range: string;
};

/**
 What a review's ranges and its commit under review are once every name is a commit.
 */
export type PinnedRanges = {
  /**
   The commit under review, which the ranges are read to.
   */
  readonly commit: string;

  /**
   The ranges, each with both sides written as commits, in the order they were given.
   */
  readonly ranges: readonly string[];
};

/**
 Reads the commit one revision names.

 @param git - how one git question is run

 @param checkoutFolder - the checkout the question is asked in

 @param revision - the revision, which may be a branch, a tag, a commit or an expression such as `<commit>^1`

 @returns the commit's full name

 @throws ReviewGitError when git did not answer with a commit

 @example
 ```ts
 const commit = await commitOfARevision({ git, checkoutFolder: '/repo', revision: 'main', },);
 ```
 */
export async function commitOfARevision({
  git,
  checkoutFolder,
  revision,
}: {
  readonly git: GitQuestion;
  readonly checkoutFolder: string;
  readonly revision: string;
}): Promise<string> {
  return (await outputOfGit({
    git,
    checkoutFolder,
    words: [
      'rev-parse',
      '--verify',
      `${revision}^{commit}`,
    ],
  },)).trim();
}

/**
 Reads the revision a range's left side names.

 @param range - the range, as `git diff` takes one

 @returns the revision before its first two dots, and the default revision for a range that names none before them

 @example
 ```ts
 const revision = leftRevisionIn({ range: 'main...fix/a', },);
 ```
 */
function leftRevisionIn({ range, }: { readonly range: string },): string {
  /**
   Where the range's left side ends.
   */
  const at = range.includes(THREE_DOTS,)
    ? range.indexOf(THREE_DOTS,)
    : range.indexOf(TWO_DOTS,);
  /**
   The revision written before those dots.
   */
  const revision = (at === (-1)) ? range : range.slice(
    0,
    at,
  );
  return (revision === '') ? THE_REVISION_A_RANGE_STARTS_FROM_BY_DEFAULT : revision;
}

/**
 Turns a review's ranges and its commit under review into commits, each name read once.

 The left side of every range is read before the commit under review is, so a range written against a branch that moves is pinned
 before anything else is asked; a name that more than one range gives is read once and its commit kept.

 @param git - how one git question is run

 @param checkoutFolder - the checkout the questions are asked in

 @param ranges - what changed, each as `git diff` takes one

 @param commit - the commit under review, or a revision that names it

 @returns the commit under review and the ranges with both sides written as commits

 @throws ReviewGitError when git did not answer with a commit

 @example
 ```ts
 const pinned = await pinnedRangesOf({ git, checkoutFolder: '/repo', ranges: ['main...fix/a'], commit: 'abc1234', },);
 ```
 */
export async function pinnedRangesOf({
  git,
  checkoutFolder,
  ranges,
  commit,
}: {
  readonly git: GitQuestion;
  readonly checkoutFolder: string;
  readonly ranges: readonly string[];
  readonly commit: string;
}): Promise<PinnedRanges> {
  /**
   The commit each revision names, so a revision written more than once is read once.
   */
  const commits = new Map<string, string>();
  /**
   Reads a revision as a commit, and answers the commit it read before when it has read it.

   @param revision - the revision to read

   @returns the commit the revision names

   @example
   ```ts
   const commit = await commitOf('main');
   ```
   */
  async function commitOf(revision: string,): Promise<string> {
    /**
     The commit this revision was read as before, when it was.
     */
    const known = commits.get(revision,);
    if (known !== undefined) {
      return known;
    }
    /**
     The commit the revision names now.
     */
    const read = await commitOfARevision({
      git,
      checkoutFolder,
      revision,
    },);
    commits.set(
      revision,
      read,
    );
    return read;
  }
  /**
   The revision every range starts from, in the order the ranges were given.
   */
  const lefts: string[] = [];
  /* oxlint-disable eslint/no-await-in-loop -- one range after another, because every left side is read before the commit under
     review is, so a name that moves is read once. See doc/lint/sequential-awaits.md. */
  for (const range of ranges) {
    lefts.push(
      await commitOf(leftRevisionIn({ range, },),),
    );
  }
  /* oxlint-enable eslint/no-await-in-loop */
  /**
   The commit under review, read after every left side.
   */
  const reviewed = await commitOf(commit,);
  /**
   The ranges, each with both sides written as commits.
   */
  const pinned: string[] = [];
  /* oxlint-disable eslint/no-await-in-loop -- one range after another, because each asks its merge base with the commit under
     review, and the order of the ranges is what the caller gave. See doc/lint/sequential-awaits.md. */
  for (const [at, range] of ranges.entries()) {
    /**
     The range's left side, as a commit.
     */
    const left = lefts[at] ?? reviewed;
    if (range.includes(THREE_DOTS,)) {
      pinned.push(`${(await outputOfGit({
        git,
        checkoutFolder,
        words: [
          'merge-base',
          left,
          reviewed,
        ],
      },)).trim()}...${reviewed}`,);
      continue;
    }
    pinned.push(`${left}..${reviewed}`,);
  }
  /* oxlint-enable eslint/no-await-in-loop */
  return {
    commit: reviewed,
    ranges: pinned,
  };
}
