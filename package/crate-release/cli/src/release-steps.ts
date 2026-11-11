import {
  join,
  posix,
} from 'node:path';

import { buildCommitMessage } from './commit-message.ts';
import {
  cratesIoRoute,
  goesToCratesIo,
  isAlphaVersion,
} from './crates-io-version-label.ts';
import {
  cargoProgram,
  gitProgram,
  lockFileName,
  manifestFileName,
  systemdRunProgram,
  unitIoWeight,
} from './constants.ts';
import type {
  ReleaseHost,
  ReleaseOptions,
} from './release-options.ts';

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
 Paths a release commits: the crate's manifest and lockfile, then each pin file's manifest and lockfile, then the
 lockfile of each lock directory.

 A lock directory holds a manifest that depends on the crate without pinning it (it reaches the crate through another
 crate), so the lock refresh changes its lockfile only; naming the lockfile keeps that change in the commit, where a
 refreshed but uncommitted lockfile would stay stale on the branch.

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
    ...new Set([
      ...manifests.flatMap(function withLockfile(manifest) {
        return [
          manifest,
          lockFileBeside(manifest)
        ];
      }),
      ...options.lockDirectories
        .map(function lockfileOf(directory) {
          return posix.join(
            directory,
            lockFileName
          );
        }),
    ]),
  ];
}

/**
 The cargo flag that keeps a call off the network, on the host that keeps it off.

 @param host - where the release runs

 @returns `--offline` on a local host, nothing on a runner, whose registry cache is empty and whose network is open

 @example
 ```ts
 offlineArguments('local');
 ```
 */
function offlineArguments(host: ReleaseHost): readonly string[] {
  return host === 'local' ? ['--offline'] : [];
}

/**
 Arguments of the check that proves the edited crate still builds.

 @param checkBins - binaries to build, each as its own `--bin`; empty to check the whole crate

 @param host - where the release runs: a local host works offline, a runner may use the network

 @returns the arguments after `cargo`

 @example
 ```ts
 buildCheckArguments({ checkBins: ['monochromatic-git-policy-cli'], host: 'local' });
 ```
 */
export function buildCheckArguments({
  checkBins,
  host,
}: {
  readonly checkBins: readonly string[];
  readonly host: ReleaseHost;
}): readonly string[] {
  return [
    'check',
    ...offlineArguments(host),
    ...checkBins.flatMap(function binArguments(checkBin) {
      return [
        '--bin',
        checkBin
      ];
    }),
  ];
}

/**
 Arguments of the lockfile refresh that records a changed workspace member version without moving any other locked
 version.

 @param host - where the release runs: a local host works offline, a runner may use the network

 @returns the arguments after `cargo`

 @example
 ```ts
 buildLockArguments('runner');
 ```
 */
export function buildLockArguments(host: ReleaseHost): readonly string[] {
  return [
    'update',
    ...offlineArguments(host),
    '--workspace',
  ];
}

/**
 Arguments of the publish dry run, which follow `cargo` on every host.

 @returns the arguments after `cargo`; the locked lockfile must already hold the new version

 @example
 ```ts
 buildPublishDryRunTail();
 ```
 */
export function buildPublishDryRunTail(): readonly string[] {
  return [
    'publish',
    '--dry-run',
    '--locked',
    '--allow-dirty',
  ];
}

/**
 Arguments of `systemd-run` that run `cargo publish --dry-run` in a capped transient unit and wait for it.

 @param unitName - unit name without the `.service` suffix

 @param workingDirectory - absolute directory of the crate

 @param memoryMax - systemd `MemoryMax` value of the unit

 @param cpuQuota - systemd `CPUQuota` value of the unit

 @param tasksMax - systemd `TasksMax` value of the unit, which counts threads and processes

 @returns the arguments after `systemd-run`; the cargo call is last, so no shell reads anything

 @example
 ```ts
 buildPublishDryRunArguments({ unitName: 'u', workingDirectory: '/r/c', memoryMax: '12G', cpuQuota: '800%', tasksMax: '2048' });
 ```
 */
export function buildPublishDryRunArguments({
  unitName,
  workingDirectory,
  memoryMax,
  cpuQuota,
  tasksMax,
}: {
  readonly unitName: string;
  readonly workingDirectory: string;
  readonly memoryMax: string;
  readonly cpuQuota: string;
  readonly tasksMax: string;
}): readonly string[] {
  return [
    '--user',
    `--unit=${unitName}`,
    '--collect',
    '--wait',
    '--pipe',
    `--working-directory=${workingDirectory}`,
    `--property=MemoryMax=${memoryMax}`,
    `--property=CPUQuota=${cpuQuota}`,
    `--property=TasksMax=${tasksMax}`,
    `--property=IOWeight=${String(unitIoWeight)}`,
    cargoProgram,
    ...buildPublishDryRunTail(),
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
 Plan the step that dry-runs the publish of the crate to crates.io.

 @param options - validated release options

 @param crateRoot - absolute directory of the crate

 @returns the step: a capped transient unit on a local host, cargo directly on a runner

 @example
 ```ts
 publishDryRunStep({ options, crateRoot: '/r/package/c' });
 ```
 */
function publishDryRunStep({
  options,
  crateRoot,
}: {
  readonly options: ReleaseOptions;
  readonly crateRoot: string;
}): ReleaseStep {
  return options.host === 'local'
    ? {
      label: 'publish dry run',
      program: systemdRunProgram,
      args: buildPublishDryRunArguments({
        unitName: options.unitName,
        workingDirectory: crateRoot,
        memoryMax: options.memoryMax,
        cpuQuota: options.cpuQuota,
        tasksMax: options.tasksMax,
      }),
      cwd: crateRoot,
    }
    : {
      label: 'publish dry run',
      program: cargoProgram,
      args: buildPublishDryRunTail(),
      cwd: crateRoot,
    };
}

/**
 Whether a release of this version compiles the crate before it commits.

 An alpha release does not: the owner's standing rule of 2026-11-11 is "make the release pipeline avoid running any
 extra work for alphas", after "Alphas don't even have to resolve all flakes", and `cargo-publish.yml` builds the binary
 of the same commit on both Linux targets after the push, which is the compile evidence an alpha gets. Every other
 release, a beta included, still checks here, because the check is the last compile before a version that can never be
 taken back exists.

 @param version - version being released

 @returns false for `-alpha.N`, true for every other version

 @example
 ```ts
 releaseRunsCargoCheck('0.2.7-alpha.1');
 ```
 */
export function releaseRunsCargoCheck(version: string): boolean {
  return !isAlphaVersion(version);
}

/**
 Say what a release of this version leaves out of the full plan, and why, for the person reading the run.

 The plan itself omits these steps (`buildReleaseSteps`), and the lines are printed before the first step runs, so a
 real release and a `--dry-run` describe the same run. A step that silently disappears would make a log look like a
 release that checked and dry-ran its publish.

 @param options - validated release options

 @returns one line per omitted step; empty when the plan is whole

 @example
 ```ts
 releasePlanNotes(options);
 ```
 */
export function releasePlanNotes(options: ReleaseOptions): readonly string[] {
  /**
   Where the version goes with respect to crates.io, which decides the publish dry run.
   */
  const route = cratesIoRoute(options.to);
  return [
    ...(releaseRunsCargoCheck(options.to)
      ? []
      : ['no cargo check: an alpha is committed without compiling here, and cargo-publish.yml builds the binary of the pushed commit on both Linux targets instead']),
    ...(route.goes
      ? []
      : [`no publish dry run: ${route.reason}`]),
  ];
}

/**
 Plan the program calls of a release, in the order they run: check, lockfile refreshes, publish dry run, commit. A
 version that never goes to crates.io (alpha, beta, and every form the label rule does not name) has no publish dry run,
 and an alpha has no check either (`releaseRunsCargoCheck`).

 A local host keeps cargo offline and runs the dry run in a capped transient unit; a runner lets cargo use the network
 and runs the dry run directly, because the runner is a disposable machine of its own.

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
    ...(releaseRunsCargoCheck(options.to)
      ? [{
        label: 'cargo check',
        program: cargoProgram,
        args: buildCheckArguments({
          checkBins: options.checkBins,
          host: options.host
        }),
        cwd: crateRoot,
      }]
      : []),
    ...options.lockDirectories
      .map(function lockStep(directory): ReleaseStep {
        return {
          label: 'lock members',
          program: cargoProgram,
          args: buildLockArguments(options.host),
          cwd: join(
            options.root,
            ...directory.split(posix.sep)
          ),
        };
      }),
    // The dry run asks crates.io, which never holds an alpha or a beta, so those versions skip it.
    ...(goesToCratesIo(options.to) ? [publishDryRunStep({
      options,
      crateRoot
    })] : []),
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
