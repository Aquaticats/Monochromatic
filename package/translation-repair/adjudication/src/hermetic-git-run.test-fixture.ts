import { devNull, } from 'node:os';

import { resolveRealGit as resolveGit, } from '@monochromatic-dev/git-executable-core/ts';

import { spawnKeyless, } from './child-environment.test-fixture.ts';

//region Hermetic git run
// RUNS GIT AGAINST A THROWAWAY CLONE, hermetic against user and system git
// configuration, so a contributor's own settings cannot change what a
// fixture commits.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. Several corpus-run fixture-git tests
// kept their own copy of this resolver and runner; all now import them from
// here.

/**
 Real git binary for fixture setup and pinned reads, resolved once.

 The repo PATH exposes a policy shim whose staging guards reject the staging
 patterns these fixtures need, so each fixture resolves the real binary
 rather than trusting PATH.
 */
export const REAL_GIT: string = await resolveGit();

/**
 Runs one git command inside the throwaway clone, hermetic against user and
 system git configuration.

 @param cloneDir - throwaway repository directory

 @param args - git argument vector

 @returns Captured stdout

 @example
 ```ts
 const sha = await fixtureGit({ cloneDir, args: ['rev-parse', 'HEAD',], },);
 ```
 */
export async function fixtureGit(
  {
    cloneDir,
    args,
  }: {
    readonly cloneDir: string;
    readonly args: readonly string[];
  },
): Promise<string> {
  /**
   Subprocess result; only stdout is consumed.
   */
  const { stdout, } = await spawnKeyless({
    file: REAL_GIT,
    args: [
      '-C',
      cloneDir,
      ...args,
    ],
    extra: {
      GIT_CONFIG_GLOBAL: devNull,
      GIT_CONFIG_SYSTEM: devNull,
    },
  },);
  return stdout;
}

//endregion Hermetic git run
