/**
 The Git commands a landing runs that change a checkout: the merge, the commit, the staging, and the worktree removal. The
 questions a landing asks are in `git-commands.ts`.

 @module
 */

import type { Repository, } from './git-commands.ts';
import type { ProgramOutput, } from './run-a-program.ts';

/**
 Merges a branch into the checkout without committing it, which makes the compiled tree the one that would land.

 @param repository - the checkout to ask

 @param branch - the branch

 @returns what Git printed and how it ended

 @example
 ```ts
 const attempt = await mergeWithoutCommitting({ repository, branch: 'fix/a', },);
 ```
 */
export function mergeWithoutCommitting({
  repository,
  branch,
}: {
  readonly repository: Repository;
  readonly branch: string;
},): Promise<ProgramOutput> {
  return repository.run({
    program: 'git',
    words: [
      'merge',
      '--no-ff',
      '--no-commit',
      '--quiet',
      branch,
    ],
    folder: repository.folder,
    timeoutMilliseconds: repository.timeoutMilliseconds,
    outputBytes: repository.outputBytes,
  },);
}

/**
 Merges `main` into a worktree, which is how a branch is seen combined with what landed since its agent checked it.

 @param repository - the agent's worktree

 @returns what Git printed and how it ended

 @example
 ```ts
 const merged = await mergeMainInto(repository,);
 ```
 */
export function mergeMainInto(repository: Repository,): Promise<ProgramOutput> {
  return repository.run({
    program: 'git',
    words: [
      'merge',
      '--no-edit',
      '--quiet',
      'main',
    ],
    folder: repository.folder,
    timeoutMilliseconds: repository.timeoutMilliseconds,
    outputBytes: repository.outputBytes,
  },);
}

/**
 Commits what a checkout has staged, naming the paths when a caller wants only those.

 @param repository - the checkout to ask

 @param message - the message its commit carries

 @param paths - the paths

 @param timeoutMilliseconds - how long it may take

 @returns what Git printed and how it ended

 @example
 ```ts
 await commitStaged({ repository, message: 'style(*): a reflow', paths: ['doc/a.md',], timeoutMilliseconds: 120_000, },);
 ```
 */
export function commitStaged({
  repository,
  message,
  paths,
  timeoutMilliseconds,
}: {
  readonly repository: Repository;
  readonly message: string;
  readonly paths: readonly string[];
  readonly timeoutMilliseconds: number;
},): Promise<ProgramOutput> {
  return repository.run({
    program: 'git',
    words: [
      'commit',
      '--quiet',
      ...(message === '' ? ['--no-edit',] : [
        '--message',
        message,
      ]),
      ...(paths.length === 0 ? [] : [
        '--',
        ...paths,
      ]),
    ],
    folder: repository.folder,
    timeoutMilliseconds,
    outputBytes: repository.outputBytes,
  },);
}

/**
 Stages paths whose conflict regions are settled.

 @param repository - the checkout to ask

 @param paths - the paths

 @returns what Git printed and how it ended

 @example
 ```ts
 await stagePaths({ repository, paths: ['doc/a.md',], },);
 ```
 */
export function stagePaths({
  repository,
  paths,
}: {
  readonly repository: Repository;
  readonly paths: readonly string[];
},): Promise<ProgramOutput> {
  return repository.run({
    program: 'git',
    words: [
      'add',
      '--',
      ...paths,
    ],
    folder: repository.folder,
    timeoutMilliseconds: repository.timeoutMilliseconds,
    outputBytes: repository.outputBytes,
  },);
}

/**
 Removes one worktree of the repository.

 @param repository - the checkout to ask

 @param path - the path

 @param timeoutMilliseconds - how long it may take

 @returns what Git printed and how it ended

 @example
 ```ts
 const removed = await removeWorktree({ repository, path: '/repository/.claude/worktrees/a', timeoutMilliseconds: 600_000, },);
 ```
 */
export function removeWorktree({
  repository,
  path,
  timeoutMilliseconds,
}: {
  readonly repository: Repository;
  readonly path: string;
  readonly timeoutMilliseconds: number;
},): Promise<ProgramOutput> {
  return repository.run({
    program: 'git',
    words: [
      'worktree',
      'remove',
      '--force',
      path,
    ],
    folder: repository.folder,
    timeoutMilliseconds,
    outputBytes: repository.outputBytes,
  },);
}
