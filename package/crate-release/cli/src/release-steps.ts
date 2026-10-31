import {
  join,
  posix,
} from 'node:path';

import { buildCommitMessage } from './commit-message.ts';
import {
  cargoProgram,
  gitProgram,
  lockFileName,
  manifestFileName,
  systemdRunProgram,
  taskRunnerProgram,
} from './constants.ts';
import type { ReleaseOptions } from './release-options.ts';

/**
 One program call of a release, with where it runs.
 */
export type ReleaseStep = {
  /**
   Name printed before the step and in its failure line.
   */
  readonly label: string;
  readonly program: string;
  /**
   Arguments, each passed as one argument; no shell reads them.
   */
  readonly args: readonly string[];
  /**
   Absolute working directory.
   */
  readonly cwd: string;
};

/**
 Path of the lockfile that sits beside a manifest.

 @param manifestPath - repository-relative path of a `Cargo.toml`

 @returns the repository-relative path of the `Cargo.lock` in the same directory

 @example
 ```ts
 lockFileBeside('package/a/Cargo.toml');
 ```
 */
export function lockFileBeside(manifestPath: string): string {
  return posix.join(
    posix.dirname(manifestPath),
    lockFileName
  );
}

/**
 Paths a release commits: the crate's manifest and lockfile, then each pin file's manifest and lockfile.

 @param options - validated release options

 @returns repository-relative paths, each once, in commit order

 @example
 ```ts
 commitPaths(options);
 ```
 */
export function commitPaths(options: ReleaseOptions): readonly string[] {
  /**
   Manifests whose version or pin moves, crate first.
   */
  const manifests = [
    posix.join(
      options.crateDirectory,
      manifestFileName
    ),
    ...options.pinFiles,
  ];
  return [
    ...new Set(manifests.flatMap(function withLockfile(manifest) {
      return [
        manifest,
        lockFileBeside(manifest)
      ];
    })),
  ];
}

/**
 Arguments of the offline check that proves the edited crate still builds.

 @param checkBins - binaries to build, each as its own `--bin`; empty to check the whole crate

 @returns the arguments after `cargo`

 @example
 ```ts
 buildCheckArguments({ checkBins: ['cli-git-native'] });
 ```
 */
export function buildCheckArguments({ checkBins }: { readonly checkBins: readonly string[] }): readonly string[] {
  return [
    'check',
    '--offline',
    ...checkBins.flatMap(function binArguments(checkBin) {
      return [
        '--bin',
        checkBin
      ];
    }),
  ];
}

/**
 Arguments of `systemd-run` that run `cargo publish --dry-run` in a capped transient unit and wait for it.

 @param unitName - unit name without the `.service` suffix

 @param workingDirectory - absolute directory of the crate

 @param memoryMax - systemd `MemoryMax` value of the unit

 @returns the arguments after `systemd-run`; the cargo call is last, so no shell reads anything

 @example
 ```ts
 buildPublishDryRunArguments({ unitName: 'u', workingDirectory: '/r/c', memoryMax: '12G' });
 ```
 */
export function buildPublishDryRunArguments({
  unitName,
  workingDirectory,
  memoryMax,
}: {
  readonly unitName: string;
  readonly workingDirectory: string;
  readonly memoryMax: string;
}): readonly string[] {
  return [
    '--user',
    `--unit=${unitName}`,
    '--collect',
    '--wait',
    '--pipe',
    `--working-directory=${workingDirectory}`,
    `--property=MemoryMax=${memoryMax}`,
    cargoProgram,
    'publish',
    '--dry-run',
    '--locked',
    '--allow-dirty',
  ];
}

/**
 Arguments of the release commit: exactly the named paths, with the message as one argument.

 @param message - commit subject

 @param paths - repository-relative paths to commit

 @returns the arguments after `git`

 @example
 ```ts
 buildCommitArguments({ message: 'chore(x): release 1.0.0: n', paths: ['a/Cargo.toml'] });
 ```
 */
export function buildCommitArguments({
  message,
  paths,
}: {
  readonly message: string;
  readonly paths: readonly string[];
}): readonly string[] {
  return [
    'commit',
    '--message',
    message,
    '--',
    ...paths,
  ];
}

/**
 Plan the program calls of a release, in the order they run: offline check, lockfile refreshes, publish dry run,
 commit.

 @param options - validated release options

 @returns the steps; none runs until the caller starts it

 @example
 ```ts
 buildReleaseSteps(options);
 ```
 */
export function buildReleaseSteps(options: ReleaseOptions): readonly ReleaseStep[] {
  /**
   Absolute directory of the crate.
   */
  const crateRoot = join(
    options.root,
    options.crateDirectory
  );
  return [
    {
      label: 'cargo check',
      program: cargoProgram,
      args: buildCheckArguments({ checkBins: options.checkBins }),
      cwd: crateRoot,
    },
    ...options.lockTasks
      .map(function lockStep(task): ReleaseStep {
        return {
          label: 'lock members',
          program: taskRunnerProgram,
          args: [
            'run',
            task
          ],
          cwd: options.root,
        };
      }),
    {
      label: 'publish dry run',
      program: systemdRunProgram,
      args: buildPublishDryRunArguments({
        unitName: options.unitName,
        workingDirectory: crateRoot,
        memoryMax: options.memoryMax,
      }),
      cwd: crateRoot,
    },
    {
      label: 'commit',
      program: gitProgram,
      args: buildCommitArguments({
        message: buildCommitMessage({
          scope: options.scope,
          version: options.to,
          note: options.note
        }),
        paths: commitPaths(options),
      }),
      cwd: options.root,
    },
  ];
}
