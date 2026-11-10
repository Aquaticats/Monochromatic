/**
 Reads a branch through `git`, one narrow question at a time.

 Every question here is one a settlement needs:
 whether a pattern still matches,
 whether a path is there,
 what a file holds,
 and whether a commit is on the branch.
 Nothing here writes.

 @module
 */

import { runChild, } from './child-process.ts';
import { ChildProcessError, } from './errors.ts';

/**
 Separator before the count in a `git grep --count` result line,
 which reads `<tree>:<path>:<count>`.
 */
const COUNT_SEPARATOR = ':';

/**
 Reports `git grep` finding nothing,
 which is a result and not a failure.
 */
const GREP_NO_MATCH_STATUS = 1;

/**
 Exit status `git merge-base --is-ancestor` and `git cat-file -e` use for a yes.
 */
const GIT_TRUE_STATUS = 0;

/**
 Git's answer for an object or revision that is not there.
 */
const GIT_MISSING_STATUS = 128;

/**
 What a branch holds at one path.
 */
export type FileText =
  | {
    readonly kind: 'text';
    readonly text: string
  }
    | { readonly kind: 'absent' };

/**
 The questions a settlement asks of one branch.
 */
export type GitReader = {
  /**
   Counts pattern matches over the given pathspecs.
   */
  readonly countMatches: (request: {
    readonly pattern: string;
    readonly paths: readonly string[];
  },) => Promise<number>;

  /**
   Reads one tracked file's text, or says the branch has no such path.
   */
  readonly fileText: (path: string,) => Promise<FileText>;

  /**
   Answers whether the branch tracks one path.
   */
  readonly pathExists: (path: string,) => Promise<boolean>;

  /**
   Lists every path the branch tracks.
   */
  readonly trackedPaths: () => Promise<readonly string[]>;

  /**
   Answers whether one commit is an ancestor of the branch.
   */
  readonly isOnBranch: (commit: string,) => Promise<boolean>;
};

/**
 How to read one branch.
 */
export type GitReaderOptions = {
  readonly checkout: string;
  readonly branch: string;
  readonly executable?: string;
};

/**
 Turns one `git cat-file -e` answer into a presence answer,
 refusing the answer that means Git could not read the branch at all.
 
 @param status - Exit status of the presence question.
 
 @param stderr - Standard error of the presence question.
 
 @param path - Path that was asked about, for the failure message.
 
 @returns Whether the branch tracks the path.
 
 @throws {@link ChildProcessError} when the checkout or branch itself is unreadable.
 
 @example
 ```ts
 const present = isPresent({ status: 0, stderr: '', path: 'README.md' });
 ```
 */
function isPresent({
  status,
  stderr,
  path,
}: {
  readonly status: number;
  readonly stderr: string;
  readonly path: string;
},): boolean {
  if (status === GIT_TRUE_STATUS) {
    return true;
  }
  if (status === GIT_MISSING_STATUS) {
    return false;
  }
  throw new ChildProcessError({
    message: `git cat-file could not answer whether ${path} exists`,
    stderr,
    exitCode: status,
  },);
}

/**
 Sums the per-file counts of a `git grep --count` result.
 
 @param stdout - Standard output of the count run.
 
 @returns Total matches over every reported file.
 
 @example
 ```ts
 const total = totalCounts('main:README.md:2');
 ```
 */
function totalCounts(stdout: string,): number {
  /**
   Matches summed so far.
   */
  let total = 0;
  for (const line of stdout.split('\n',)) {
    if (line === '') {
      continue;
    }
    /**
     Count of one reported file, read from after its last separator.
     */
    const count = Number(line.slice(line.lastIndexOf(COUNT_SEPARATOR,) + 1,),);
    if (!Number.isSafeInteger(count,)) {
      throw new ChildProcessError({
        message: `git grep --count answered a count this reader cannot read: ${line}`,
        stderr: '',
      },);
    }
    total += count;
  }
  return total;
}

/**
 Creates a reader for one checkout and branch.
 
 @param checkout - Path of the checkout the branch is read from.
 
 @param branch - Branch or revision to read.
 
 @param executable - `git` executable, which tests replace to record the argument vectors.
 
 @returns The branch reader.
 
 @example
 ```ts
 const reader = createGitReader({ checkout: '/repo', branch: 'main' });
 ```
 */
export function createGitReader({
  checkout,
  branch,
  executable = 'git',
}: GitReaderOptions,): GitReader {
  /**
   Runs one `git` command in the checkout, keeping the checkout path in the argument vector.
   
   @param childArguments - Arguments after `-C <checkout>`.
   
   @returns Exit status and captured output.
   
   @throws {@link ChildProcessError} when `git` never starts or ends at its deadline.
   
   @example
   ```ts
   const listed = await git(['ls-tree', '-r', '--name-only', 'main']);
   ```
   */
  async function git(childArguments: readonly string[],): ReturnType<typeof runChild> {
    return await runChild({
      file: executable,
      arguments: [
        '-C',
        checkout,
        ...childArguments,
      ],
      description: `git ${childArguments[0] ?? ''}`,
    },);
  }

  /**
   Asks whether the branch tracks one path.
   
   @param path - Path inside the branch.
   
   @returns Whether the branch tracks it.
   
   @throws {@link ChildProcessError} when Git could not answer.
   
   @example
   ```ts
   const present = await pathExists('README.md');
   ```
   */
  async function pathExists(path: string,): Promise<boolean> {
    /**
     Answer of the presence question.
     */
    const answer = await git([
      'cat-file',
      '-e',
      `${branch}:${path}`,
    ],);
    return isPresent({
      status: answer.status,
      stderr: answer.stderr,
      path,
    },);
  }

  return {
    countMatches: async function countMatches({
      pattern,
      paths,
    },) {
      /**
       Result of the count run over the pathspecs.
       */
      const outcome = await git([
        'grep',
        '--count',
        '-E',
        '-e',
        pattern,
        branch,
        '--',
        ...paths,
      ],);
      if (outcome.status === GREP_NO_MATCH_STATUS) {
        return 0;
      }
      if (outcome.status !== GIT_TRUE_STATUS) {
        throw new ChildProcessError({
          message: `git grep failed for /${pattern}/ under ${paths.join(', ',)}`,
          stderr: outcome.stderr,
          exitCode: outcome.status,
        },);
      }
      return totalCounts(outcome.stdout,);
    },

    pathExists,

    fileText: async function fileText(path,) {
      if (!(await pathExists(path,))) {
        return { kind: 'absent', };
      }
      /**
       Result of reading the tracked file.
       */
      const shown = await git([
        'show',
        `${branch}:${path}`,
      ],);
      if (shown.status !== GIT_TRUE_STATUS) {
        throw new ChildProcessError({
          message: `git show could not read ${path} on ${branch}`,
          stderr: shown.stderr,
          exitCode: shown.status,
        },);
      }
      return {
        kind: 'text',
        text: shown.stdout,
      };
    },

    trackedPaths: async function trackedPaths() {
      /**
       Result of listing the branch's tree.
       */
      const listed = await git([
        'ls-tree',
        '-r',
        '--name-only',
        branch,
      ],);
      if (listed.status !== GIT_TRUE_STATUS) {
        throw new ChildProcessError({
          message: `git ls-tree could not list ${branch}`,
          stderr: listed.stderr,
          exitCode: listed.status,
        },);
      }
      return listed.stdout
        .split('\n',)
        .filter(function isPath(line,): boolean {
          return line !== '';
        },);
    },

    isOnBranch: async function isOnBranch(commit,) {
      /**
       Result of the ancestry question.
       */
      const merged = await git([
        'merge-base',
        '--is-ancestor',
        commit,
        branch,
      ],);
      if (merged.status === GIT_MISSING_STATUS) {
        throw new ChildProcessError({
          message: `git cannot resolve ${commit}, so whether it is on ${branch} is unknown`,
          stderr: merged.stderr,
          exitCode: merged.status,
        },);
      }
      return merged.status === GIT_TRUE_STATUS;
    },
  };
}
