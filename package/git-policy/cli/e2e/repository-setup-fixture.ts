/**
 Setup steps of a scenario repository:
 seeded main repository with its bare remote,
 SSH signing,
 and hooks.

 @module
 */

import {
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';

import { configHookArguments, } from './hook-program-fixture.ts';
import { runChecked, } from './process-fixture.ts';
import {
  configure,
  realGit,
  type RealGitTarget,
} from './real-git-fixture.ts';
import type {
  RepositoryOptions,
  ScenarioRepository,
} from './repository-model-fixture.ts';

//region Constants

/**
 Committer identity used by every scenario repository.
 */
const IDENTITY = {
  name: 'e2e',
  email: 'e2e@example.invalid',
} as const;

//endregion Constants

//region Steps

/**
 Configures SSH signing with a fresh key.

 @param repository - repository before scenario start

 @example
 ```ts
 await configureSigning(repository);
 ```
 */
async function configureSigning(repository: ScenarioRepository,): Promise<void> {
  /**
   Private key path; the public key sits beside it.
   */
  const key = join(
    repository.root,
    'signing-key',
  );
  await runChecked({
    command: 'ssh-keygen',
    args: [
      '-q',
      '-t',
      'ed25519',
      '-N',
      '',
      '-C',
      IDENTITY.email,
      '-f',
      key,
    ],
    cwd: repository.root,
    env: repository.realEnv,
  },);
  /**
   Allowed-signers file so `git verify-commit` can check signatures.
   */
  const allowed = join(
    repository.root,
    'allowed-signers',
  );
  await writeFile(
    allowed,
    `${IDENTITY.email} ${(await readFile(
      `${key}.pub`,
      'utf8',
    )).trim()}\n`,
  );
  await configure({
    repository,
    entries: [
      [
        'gpg.format',
        'ssh',
      ],
      [
        'user.signingKey',
        `${key}.pub`,
      ],
      [
        'commit.gpgSign',
        'true',
      ],
      [
        'gpg.ssh.allowedSignersFile',
        allowed,
      ],
    ],
  },);
}

/**
 Creates the bare remote and the main repository with its seed commit pushed upstream.

 @param bootstrap - real-Git target for the main repository

 @param root - scenario root

 @param remote - bare remote path

 @param seedFiles - files of the seed commit

 @example
 ```ts
 await createSeededRepository({ bootstrap, root, remote, seedFiles });
 ```
 */
export async function createSeededRepository({
  bootstrap,
  root,
  remote,
  seedFiles,
}: Readonly<{
  bootstrap: RealGitTarget;
  root: string;
  remote: string;
  seedFiles: RepositoryOptions['seedFiles'];
}>,): Promise<void> {
  await realGit({
    repository: bootstrap,
    args: [
      'init',
      '--quiet',
      '--bare',
      remote,
    ],
    cwd: root,
  },);
  await realGit({
    repository: bootstrap,
    args: [
      'init',
      '--quiet',
      '--initial-branch=main',
      bootstrap.worktree,
    ],
    cwd: root,
  },);
  await configure({
    repository: bootstrap,
    entries: [
      [
        'user.name',
        IDENTITY.name,
      ],
      [
        'user.email',
        IDENTITY.email,
      ],
      [
        'core.autocrlf',
        'false',
      ],
    ],
  },);
  await realGit({
    repository: bootstrap,
    args: [
      'remote',
      'add',
      'origin',
      remote,
    ],
  },);
  await Promise.all(seedFiles.map(async function seed(file,) {
    await mkdir(
      dirname(join(
        bootstrap.worktree,
        file.path,
      ),),
      { recursive: true, },
    );
    await writeFile(
      join(
        bootstrap.worktree,
        file.path,
      ),
      file.bytes,
    );
  },),);
  await realGit({
    repository: bootstrap,
    args: [
      'add',
      '--',
      ...seedFiles.map(function seedPath(file,) {
        return file.path;
      },),
    ],
  },);
  await realGit({
    repository: bootstrap,
    args: [
      'commit',
      '--quiet',
      '--message',
      'e2e seed',
    ],
  },);
  await realGit({
    repository: bootstrap,
    args: [
      'push',
      '--quiet',
      '--set-upstream',
      'origin',
      'main',
    ],
  },);
}

/**
 Registers config hooks and signing requested by the options.
 Traditional hooks (files in the hooks directory) are unsupported by the wrapper and never installed.

 @param repository - repository whose setup finishes

 @param hookProgram - shared hook program path

 @example
 ```ts
 await installExtras({ repository, hookProgram });
 ```
 */
export async function installExtras({
  repository,
  hookProgram,
}: Readonly<{
  repository: ScenarioRepository;
  hookProgram: string;
}>,): Promise<void> {
  if (repository.options
    .signing)
    await configureSigning(repository,);
  if (repository.options
    .hooks
    === 'config') {
    // Sequential: each `git config` write takes config.lock.
    await configHookArguments({
      hookProgram,
      events: repository.options
        .hookEvents,
    },)
      .reduce(
      async function configAfter(
        previous,
        args,
      ) {
        await previous;
        await realGit({
          repository,
          args,
        },);
      },
      Promise.resolve(),
      );
  }
}

//endregion Steps
