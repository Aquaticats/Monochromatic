/**
 Real Git by absolute path for fixture setup and observation,
 never through the wrapper.

 @module
 */

import { runChecked, } from './process-fixture.ts';
import type { ScenarioRepository, } from './repository-model-fixture.ts';

//region Real Git

/**
 Repository fields real-Git runs need.
 */
export type RealGitTarget = Pick<ScenarioRepository, 'realEnv' | 'realGit' | 'worktree'>;

/**
 Runs real Git by absolute path in a scenario repository;
 fails on non-zero exit.

 @param repository - scenario repository

 @param args - Git arguments

 @param cwd - directory override

 @returns standard output

 @example
 ```ts
 await realGit({ repository, args: ['rev-parse', 'HEAD'] });
 ```
 */
export async function realGit({
  repository,
  args,
  cwd,
}: Readonly<{
  repository: RealGitTarget;
  args: readonly string[];
  cwd?: string;
}>,): Promise<string> {
  return await runChecked({
    command: repository.realGit,
    args,
    cwd: cwd ?? repository.worktree,
    env: repository.realEnv,
  },);
}

/**
 Writes repository config keys one at a time,
 because each `git config` write takes `config.lock`.

 @param repository - repository to configure

 @param entries - key and value pairs

 @example
 ```ts
 await configure({ repository, entries: [['user.name', 'e2e']] });
 ```
 */
export async function configure({
  repository,
  entries,
}: Readonly<{
  repository: RealGitTarget;
  entries: readonly (readonly [
    string,
    string
  ])[];
}>,): Promise<void> {
  await entries.reduce(
    async function configureAfter(
      previous,
      [key, value,],
    ) {
      await previous;
      await realGit({
        repository,
        args: [
          'config',
          key,
          value,
        ],
      },);
    },
    Promise.resolve(),
  );
}

//endregion Real Git
