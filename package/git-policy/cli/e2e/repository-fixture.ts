/**
 Fresh dummy repositories for one scenario:
 a local bare remote,
 a repository with an upstream,
 optionally a linked worktree,
 hooks,
 and SSH signing.
 Nothing here touches host repositories;
 every path lives under the scenario root inside the container.

 @module
 */

import { mkdir, } from 'node:fs/promises';
import { join, } from 'node:path';

import { installPrograms, } from './hook-program-fixture.ts';
import {
  realGit,
  type RealGitTarget,
} from './real-git-fixture.ts';
import type {
  RepositoryOptions,
  ScenarioRepository,
} from './repository-model-fixture.ts';
import {
  createSeededRepository,
  installExtras,
} from './repository-setup-fixture.ts';
import {
  WRAPPER_BINS,
  WRAPPER_KIND,
} from './wrapper-selection-fixture.ts';

export { realGit, } from './real-git-fixture.ts';
export type {
  RepositoryOptions,
  ScenarioRepository,
} from './repository-model-fixture.ts';

//region Constants

/**
 Directory whose `git` shadows the real Git for wrapper runs:
 the native executable's link (see `wrapper-selection-fixture.ts`).
 */
export const WRAPPER_BIN: string = WRAPPER_BINS[WRAPPER_KIND];

/**
 System directories after the selected Git.
 */
const SYSTEM_PATH = '/usr/local/bin:/usr/bin:/bin';

//endregion Constants



//region Creation

/**
 Creates the scenario repository.

 @param root - empty scenario root

 @param gitVersion - Git version under `/opt/git`

 @param options - setup options

 @returns ready repository

 @example
 ```ts
 await createScenarioRepository({ root: '/work/2.55.0/baseline', gitVersion: '2.55.0', options });
 ```
 */
export async function createScenarioRepository({
  root,
  gitVersion,
  options,
}: Readonly<{
  root: string;
  gitVersion: string;
  options: RepositoryOptions;
}>,): Promise<ScenarioRepository> {
  /**
   Directory holding the selected Git.
   */
  const gitBin = `/opt/git/${gitVersion}/bin`;
  /**
   Scenario directories.
   */
  const directories = {
    home: join(
      root,
      'home',
    ),
    markers: join(
      root,
      'markers',
    ),
    logs: join(
      root,
      'logs',
    ),
    tools: join(
      root,
      'tools',
    ),
    main: join(
      root,
      'repo',
    ),
  };
  await Promise.all(Object.values(directories,)
    .map(async function create(directory,) {
      await mkdir(
        directory,
        { recursive: true, },
      );
    },),);
  /**
   Environment shared by wrapper and real-Git runs.
   */
  const baseEnv: NodeJS.ProcessEnv = {
    HOME: directories.home,
    XDG_CONFIG_HOME: join(
      directories.home,
      '.config',
    ),
    GIT_CONFIG_NOSYSTEM: '1',
    LANG: 'C',
    LC_ALL: 'C',
    E2E_MARKER_DIR: directories.markers,
    E2E_LOG_DIR: directories.logs,
    ...(options.hookMode === undefined ? {} : { E2E_HOOK_MODE: options.hookMode, }),
  };
  /**
   Real-Git target for the main repository.
   */
  const bootstrap: RealGitTarget = {
    realGit: join(
      gitBin,
      'git',
    ),
    realEnv: {
      ...baseEnv,
      PATH: `${gitBin}:${SYSTEM_PATH}`,
    },
    worktree: directories.main,
  };
  /**
   Local bare remote.
   */
  const remote = join(
    root,
    'remote.git',
  );
  await createSeededRepository({
    bootstrap,
    root,
    remote,
    seedFiles: options.seedFiles,
  },);
  /**
   Directory scenario commands run in.
   */
  const worktree = options.worktree === 'linked' ? join(
    root,
    'linked',
  ) : directories.main;
  if (options.worktree === 'linked') {
    await realGit({
      repository: bootstrap,
      args: [
        'worktree',
        'add',
        '--quiet',
        '-b',
        'work',
        worktree,
      ],
    },);
    await realGit({
      repository: {
        ...bootstrap,
        worktree,
      },
      args: [
        'push',
        '--quiet',
        '--set-upstream',
        'origin',
        'work',
      ],
    },);
  }
  /**
   Installed programs.
   */
  const programs = await installPrograms(directories.tools,);
  /**
   Finished repository handle.
   */
  const repository: ScenarioRepository = {
    root,
    worktree,
    commonDir: join(
      directories.main,
      '.git',
    ),
    remote,
    branch: options.worktree === 'linked' ? 'work' : 'main',
    realGit: bootstrap.realGit,
    markerDir: directories.markers,
    logDir: directories.logs,
    editorProgram: programs.editor,
    wrapperEnv: {
      ...baseEnv,
      PATH: `${WRAPPER_BIN}:${gitBin}:${SYSTEM_PATH}`,
    },
    realEnv: bootstrap.realEnv,
    options,
  };
  await installExtras({
    repository,
    hookProgram: programs.hook,
  },);
  return repository;
}

//endregion Creation
