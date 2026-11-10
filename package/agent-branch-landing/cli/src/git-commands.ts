/**
 Every Git question a landing asks, in one place, so the words a landing passes to Git are read here and not scattered across the
 sequences that use them. The commands that change a checkout are in `git-changes.ts`.

 @module
 */

import { stat, } from 'node:fs/promises';
import { dirname, } from 'node:path';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { GIT_ENVIRONMENT, } from './constants.ts';
import { LandingConfigurationError, } from './errors.ts';
import {
  found,
  type Looked,
  missing,
} from './lookups.ts';
import type {
  ProgramOutput,
  RunProgram,
} from './run-a-program.ts';

/**
 One checkout, the way a landing asks it questions.
 */
export type Repository = {
  /**
   The folder every command of this repository starts in: the main checkout, or one agent's worktree.
   */
  readonly folder: string;

  /**
   How a program is run, so a test hands over a stand-in rather than a fake in the code.
   */
  readonly run: RunProgram;

  /**
   Milliseconds one Git command may take.
   */
  readonly timeoutMilliseconds: number;

  /**
   Most bytes of one Git command's output kept.
   */
  readonly outputBytes: number;
};

/**
 One worktree as `git worktree list --porcelain` reports it.
 */
export type WorktreeRecord = {
  /**
   Its folder.
   */
  readonly path: string;

  /**
   The commit it is at.
   */
  readonly head: string;

  /**
   The branch checked out in it, or empty text when its HEAD is detached.
   */
  readonly branch: string;

  /**
   Whether Git holds it locked, which means every command over it is refused.
   */
  readonly locked: boolean;

  /**
   Whether its HEAD is detached, so no branch checked out there follows its commits.
   */
  readonly detached: boolean;
};

/**
 What one Git question is asked with.

 @internal
 */
export type AskGitRequest = {
  /**
   The checkout to ask.
   */
  readonly repository: Repository;

  /**
   The Git command and its arguments.
   */
  readonly words: readonly string[];

  /**
   Milliseconds it may take, or the checkout's own bound.
   */
  readonly timeoutMilliseconds?: number;
};

/**
 Asks Git one question in a checkout's folder.

 @param repository - the checkout to ask

 @param words - the words Git is asked

 @returns what Git printed and how it ended

 @example
 ```ts
 const head = await askGit({ repository, words: ['rev-parse', 'HEAD',], },);
 ```
 */
export function askGit({
  repository,
  words,
  timeoutMilliseconds = repository.timeoutMilliseconds,
}: AskGitRequest,): Promise<ProgramOutput> {
  return repository.run({
    program: 'git',
    words,
    folder: repository.folder,
    environment: GIT_ENVIRONMENT,
    timeoutMilliseconds,
    outputBytes: repository.outputBytes,
  },);
}

/**
 Reads the trimmed output of a Git command that answered, and refuses when it did not.

 @param asked - what the command printed and how it ended

 @param what - what was being asked, in the caller's words

 @returns the answer

 @throws LandingConfigurationError when Git did not answer at all

 @example
 ```ts
 const head = answered({ asked, what: 'the commit this checkout is at', },);
 ```
 */
export function answered({
  asked,
  what,
}: {
  readonly asked: ProgramOutput;
  readonly what: string;
},): string {
  if (asked.status !== 0) {
    throw new LandingConfigurationError(
      `Git could not answer ${what}: ${asked.problem ?? (asked.stderr
        .trim()
        || 'it printed nothing')}`,
    );
  }
  return asked.stdout
    .trim();
}

/**
 Splits output into the paths it names, dropping the empty line a trailing newline leaves.

 @param output - what a Git command printed

 @returns the lines that hold something

 @example
 ```ts
 const paths = pathsIn('doc/a.md\ndoc/b.md\n',);
 ```
 */
export function pathsIn(output: string,): readonly string[] {
  return output.split('\n',)
    .filter(function isPath(line: string,) {
      return line.trim() !== '';
    },);
}

/**
 Finds the main checkout of the repository a folder belongs to, which is where worktrees and `main` live.

 @param folder - the folder the program starts in

 @param run - how a program is run

 @param timeoutMilliseconds - how long it may take

 @returns the main checkout's folder

 @throws LandingConfigurationError when the folder is not in a repository

 @example
 ```ts
 const main = await mainCheckoutOf({ folder: '/repository/.claude/worktrees/a', run, timeoutMilliseconds: 1_000, },);
 ```
 */
export async function mainCheckoutOf({
  folder,
  run,
  timeoutMilliseconds,
}: {
  readonly folder: string;
  readonly run: RunProgram;
  readonly timeoutMilliseconds: number;
},): Promise<string> {
  return dirname(answered({
    asked: await run({
      program: 'git',
      words: [
        'rev-parse',
        '--path-format=absolute',
        '--git-common-dir',
      ],
      folder,
      environment: GIT_ENVIRONMENT,
      timeoutMilliseconds,
      outputBytes: 4_096,
    },),
    what: 'where the repository it runs in is kept',
  },),);
}

/**
 Reads the commit a checkout is at.

 @param repository - the checkout to ask

 @returns the commit's name

 @example
 ```ts
 const before = await headOf(repository,);
 ```
 */
export async function headOf(repository: Repository,): Promise<string> {
  return answered({
    asked: await askGit({
      repository,
      words: [
        'rev-parse',
        'HEAD',
      ],
    },),
    what: 'the commit this checkout is at',
  },);
}

/**
 Reads the branch checked out in a worktree.

 @param repository - the worktree to ask

 @returns the branch's name, or empty text when its HEAD is detached

 @example
 ```ts
 const branch = await currentBranchOf(worktreeRepository,);
 ```
 */
export async function currentBranchOf(repository: Repository,): Promise<string> {
  return answered({
    asked: await askGit({
      repository,
      words: [
        'branch',
        '--show-current',
      ],
    },),
    what: 'which branch this worktree has checked out',
  },);
}

/**
 Checks whether a merge is open in a checkout, which is the one state a landing refuses to start in.

 @param repository - the checkout to ask

 @returns true when the checkout holds an uncommitted merge

 @example
 ```ts
 if (await mergeIsOpen(repository,)) return;
 ```
 */
export async function mergeIsOpen(repository: Repository,): Promise<boolean> {
  return (await openMergeMarker(repository,)).found;
}

/**
 Finds the marker of the merge a checkout holds, which is the one state a landing refuses to start in.

 @param repository - the checkout to ask

 @returns the marker's path when a merge is open, and why it is not when one is not

 @example
 ```ts
 if ((await openMergeMarker(repository,)).found) return;
 ```
 */
export async function openMergeMarker(repository: Repository,): Promise<Looked<string>> {
  /**
   Where Git keeps the marker of an open merge, which is inside the worktree's own Git folder.
   */
  const path = answered({
    asked: await askGit({
      repository,
      words: [
        'rev-parse',
        '--path-format=absolute',
        '--git-path',
        'MERGE_HEAD',
      ],
    },),
    what: 'where an open merge would be recorded',
  },);
  try {
    await stat(path,);
    return found(path,);
  }
  catch (caught) {
    // A marker that is not there is what a checkout with no merge reports, and the operating system's own words are the only
    // thing it says about it.
    return missing(caughtValueText(caught,),);
  }
}

/**
 Lists the paths a merge staged, which are the paths that merge touches.

 @param repository - the checkout to ask

 @returns the staged paths, repository-relative

 @example
 ```ts
 const staged = await stagedPaths(repository,);
 ```
 */
export async function stagedPaths(repository: Repository,): Promise<readonly string[]> {
  return pathsIn(answered({
    asked: await askGit({
      repository,
      words: [
        'diff',
        '--cached',
        '--name-only',
      ],
    },),
    what: 'which paths this checkout has staged',
  },),);
}

/**
 Reads one file as a revision holds it.

 @param repository - the checkout to ask

 @param revision - the revision

 @param path - the path

 @returns what Git printed and how it ended

 @example
 ```ts
 const committed = await fileAtRevision({ repository, revision: 'HEAD', path: 'doc/a.md', },);
 ```
 */
export function fileAtRevision({
  repository,
  revision,
  path,
}: {
  readonly repository: Repository;
  readonly revision: string;
  readonly path: string;
},): Promise<ProgramOutput> {
  return askGit({
    repository,
    words: [
      'show',
      `${revision}:${path}`,
    ],
  },);
}

/**
 Counts the commits of a branch that `main` does not hold.

 @param repository - the checkout to ask

 @param branch - the branch

 @returns how many commits are not on `main`

 @example
 ```ts
 const left = await commitsOfBranchNotOnMain({ repository, branch: 'fix/a', },);
 ```
 */
export async function commitsOfBranchNotOnMain({
  repository,
  branch,
}: {
  readonly repository: Repository;
  readonly branch: string;
},): Promise<number> {
  return Number(answered({
    asked: await askGit({
      repository,
      words: [
        'rev-list',
        '--count',
        `main..${branch}`,
      ],
    },),
    what: `how many commits ${branch} has that main does not`,
  },),);
}

/**
 Reads the last commit of `main` as `git log` shows it, which is the landing's own proof of what it did.

 @param repository - the main checkout

 @returns the short name and the subject

 @example
 ```ts
 const last = await lastCommitOnMain(repository,);
 ```
 */
export async function lastCommitOnMain(repository: Repository,): Promise<string> {
  return answered({
    asked: await askGit({
      repository,
      words: [
        'log',
        '-1',
        '--format=%h %s',
        'main',
      ],
    },),
    what: 'the last commit of main',
  },);
}

/**
 Lists the paths a branch changed since it left `main`, which is how a landing knows which packages to check.

 @param repository - the checkout to ask

 @param branch - the branch

 @returns the changed paths, repository-relative

 @example
 ```ts
 const changed = await changedPathsSinceMain({ repository, branch: 'fix/a', },);
 ```
 */
export async function changedPathsSinceMain({
  repository,
  branch,
}: {
  readonly repository: Repository;
  readonly branch: string;
},): Promise<readonly string[]> {
  return pathsIn(answered({
    asked: await askGit({
      repository,
      words: [
        'diff',
        '--name-only',
        `main...${branch}`,
      ],
    },),
    what: `which paths ${branch} changed since main`,
  },),);
}

/**
 Lists the paths a merge left unresolved, which must be settled before its commit can be made.

 @param repository - the checkout holding the merge

 @returns the unresolved paths, repository-relative

 @example
 ```ts
 const unresolved = await unresolvedPaths(repository,);
 ```
 */
export async function unresolvedPaths(repository: Repository,): Promise<readonly string[]> {
  return pathsIn(answered({
    asked: await askGit({
      repository,
      words: [
        'diff',
        '--name-only',
        '--diff-filter=U',
      ],
    },),
    what: 'which paths this merge left unresolved',
  },),);
}

/**
 Reads every worktree of the repository.

 @param repository - the main checkout

 @returns one record per worktree, in the order Git lists them

 @example
 ```ts
 const records = await worktreeRecords(repository,);
 ```
 */
export async function worktreeRecords(repository: Repository,): Promise<readonly WorktreeRecord[]> {
  return answered({
    asked: await askGit({
      repository,
      words: [
        'worktree',
        'list',
        '--porcelain',
      ],
    },),
    what: 'which worktrees this repository has',
  },)
    .split('\n\n',)
    .flatMap(function recordOf(block: string,) {
      /**
       The record's own lines.
       */
      const lines: readonly string[] = block.split('\n',);
      /**
       The folder the record names, which every record has.
       */
      const path = lines.find(function isPath(line: string,) {
        return line.startsWith('worktree ',);
      },);
      if (path === undefined) return [];
      return [{
        path: path.slice('worktree '.length,),
        head: lines.find(function isHead(line: string,) {
          return line.startsWith('HEAD ',);
        },)
          ?.slice('HEAD '.length,)
          ?? '',
        branch: lines.find(function isBranch(line: string,) {
          return line.startsWith('branch refs/heads/',);
        },)
          ?.slice('branch refs/heads/'.length,)
          ?? '',
        locked: lines.some(function isLocked(line: string,) {
          return (line === 'locked') || line.startsWith('locked ',);
        },),
        detached: lines.includes('detached',),
      },];
    },);
}

/**
 Asks which refs hold a commit, which is what makes a detached worktree's commit safe to leave behind.

 @param repository - the checkout to ask

 @param commit - what committing came to

 @returns the refs that hold it, possibly none

 @example
 ```ts
 const holders = await refsHolding({ repository, commit: 'abc123', },);
 ```
 */
export async function refsHolding({
  repository,
  commit,
}: {
  readonly repository: Repository;
  readonly commit: string;
},): Promise<string> {
  return answered({
    asked: await askGit({
      repository,
      words: [
        'for-each-ref',
        '--count=1',
        '--contains',
        commit,
        'refs/heads',
        'refs/remotes',
      ],
    },),
    what: `which refs hold ${commit}`,
  },);
}

/**
 Reads a checkout's status, one line per changed or untracked path.

 @param repository - the checkout to ask

 @returns what Git printed and how it ended, which a caller judges

 @example
 ```ts
 const status = await statusOf(repository,);
 ```
 */
export function statusOf(repository: Repository,): Promise<ProgramOutput> {
  return askGit({
    repository,
    words: [
      'status',
      '--porcelain=v1',
      '--untracked-files=normal',
    ],
  },);
}

/**
 Lists the ignored entries of a checkout without removing anything, which `AGENTS.md` rule `GCL` asks before a cleanup.

 @param repository - the checkout to ask

 @returns the ignored entries as the dry run printed them, empty when Git refused the question

 @example
 ```ts
 const ignored = await ignoredEntriesOf(repository,);
 ```
 */
export async function ignoredEntriesOf(repository: Repository,): Promise<readonly string[]> {
  /**
   What the dry run listed.
   */
  const listed = await askGit({
    repository,
    words: [
      'clean',
      '--dry-run',
      '-d',
      '-X',
    ],
  },);
  return listed.status === 0 ? pathsIn(listed.stdout,) : [];
}
