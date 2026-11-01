import spawn from 'nano-spawn';

import { childEnvironment, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type SourceCommit,
  sourceCommitOf,
} from './cache-account-commits.ts';
import { resolveGit, } from '@monochromatic-dev/translation-repair-glossary/ts';

//region Cache account git
// WHAT THE PRE-LAUNCH CACHE CHECK ASKS GIT (ledger M28): read-only commands
// whose answers the pure readers of `cache-account-read.ts` and
// `cache-account-commits.ts` take.

/**
 Log format of one source commit: hash, committer time, subject.
 */
export const CACHE_ACCOUNT_COMMIT_FORMAT = '--format=%H%x09%ct%x09%s';


/**
 Runs one read-only git command and returns what it printed.

 @param args - arguments after the git command

 @returns Standard output

 @example
 ```ts
 const top = await cacheAccountGitOutput({ args: ['rev-parse', '--show-toplevel',], },);
 ```
 */
export async function cacheAccountGitOutput({ args, }: { readonly args: readonly string[]; },): Promise<string> {
  /**
   Git's answer.
   */
  const { stdout, } = await spawn(
    await resolveGit(),
    [...args,],
    { env: childEnvironment({ parent: process.env, },), },
  );
  return stdout;
}

/**
 Reads non-empty log lines into commits.

 @param output - `git log` output in {@link CACHE_ACCOUNT_COMMIT_FORMAT}

 @returns Commits in the order git printed them

 @example
 ```ts
 cacheAccountCommitsOf({ output, },);
 ```
 */
export function cacheAccountCommitsOf({ output, }: { readonly output: string; },): readonly SourceCommit[] {
  return output
    .split('\n',)
    .filter(function written(line,): boolean {
      return line !== '';
    },)
    .map(function commitOf(line,): SourceCommit {
      return sourceCommitOf({ line, },);
    },);
}

//endregion Cache account git
