#!/usr/bin/env node
/**
 Run the concurrent-commit end-to-end suite (`e2e/`) against the native executable.

 Three subcommands, selected by the first argument:

 - `base` builds `localhost/cli-git-concurrent-e2e:<base tag>` from `e2e/concurrent-commits.Containerfile`:
   Node 24.11.0, Git 2.55.0 under `/opt/git`, and `ssh-keygen`.
   It needs a network (package manager and Git source downloads); the suite itself does not.
   The base tag is `latest` unless `GIT_POLICY_E2E_BASE_TAG` names another, so a trial build never replaces the
   image other runs use.
 - `image` builds `localhost/cli-git-concurrent-e2e-native:<tag>` in two stages.
   The first compiles the release `cli-git-native` from a mount-free snapshot of this checkout in `rust:1.97-bookworm`,
   the Debian release of the end-to-end image, so the executable runs there without a glibc mismatch.
   The second starts from the base image (built first when it does not exist) and links the executable as
   `/opt/native/bin/git`.
   That image has no `/usr/bin/git`, so the native wrapper can only find the scenario's Git through `PATH` order.
 - `run` starts the suite in that image with `E2E_WRAPPER=native` (the container limits, no network, `e2e/` mounted
   read-only).
   `usage_seed`, `usage_scenario` and `usage_git` (set by mise from `--seed`, `--scenario` and `--git`)
   select the seed, the scenario names and the Git versions.

 Every subcommand accepts `--memory <size>` and `--cpus <count>` after it (2 GiB and 2 CPUs unless given);
 mise passes the `run` task's flags as `usage_memory` and `usage_cpus`.
 */
import {
  mkdir,
  mkdtempDisposable,
  writeFile,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import {
  buildLimitArguments,
  memoryArguments,
  parallelJobs,
  parseContainerLimitOptions,
  runLimitArguments,
} from './container-limits.ts';
import { copyEndToEndSnapshot, } from './native-e2e-snapshot.ts';
import { vendorLockedDependencies, } from './native-scanner-snapshot.ts';
import {
  NativeVerificationError,
  runCommand,
} from './native-verification-process.ts';

// Scratch lives inside the package's gitignored node_modules/.monochromatic, never the system temporary directory,
// whose backing device is unknowable.
/**
 Parent of every scratch directory this script creates.
 */
const scratchBase = join(
  import.meta.dirname,
  '..',
  'node_modules',
  '.monochromatic',
  'git-policy'
);
await mkdir(
  scratchBase,
  { recursive: true }
);

/**
 Concurrent worktrees set GIT_POLICY_NATIVE_IMAGE_TAG so one snapshot never runs another's image.
 */
const imageTag = process.env
  .GIT_POLICY_NATIVE_IMAGE_TAG
  ?? 'development';
/**
 Image this script builds and runs.
 */
const nativeImage = `localhost/cli-git-concurrent-e2e-native:${imageTag}`;
/**
 Tag of the base image; a trial build sets `GIT_POLICY_E2E_BASE_TAG` so it never replaces the shared `latest`.
 */
const baseTag = process.env
  .GIT_POLICY_E2E_BASE_TAG
  ?? 'latest';
/**
 Image holding the Git builds, Node and `ssh-keygen`.
 */
const baseImage = `localhost/cli-git-concurrent-e2e:${baseTag}`;
/**
 Command-line option for a flag mise passed as an environment variable.

 @param flag - option the variable stands for

 @param variable - name of the environment variable holding the flag's value, unset or empty when the flag was not given

 @returns the option and its value, or nothing when the flag was not given
 */
function environmentOption({
  flag,
  variable,
}: {
  readonly flag: string;
  readonly variable: string;
},): string[] {
  /**
   The flag's value as mise passed it.
   */
  const value = process.env[variable];
  if ((value === undefined) || (value === ''))
    return [];
  return [
    flag,
    value,
  ];
}

/**
 The subcommand and the options after it. Mise passes the `run` task's `--memory` and `--cpus` flags as
 `usage_memory` and `usage_cpus`, so those are read as the leading options.
 */
const [subcommand, ...subcommandOptions] = process.argv
  .slice(2);
/**
 Container limits from the `--memory` and `--cpus` options, and the arguments left over, which are a usage error.
 */
const {
  limits: containerLimits,
  rest: extra,
} = parseContainerLimitOptions({
  options: [
    ...environmentOption({
      flag: '--memory',
      variable: 'usage_memory',
    }),
    ...environmentOption({
      flag: '--cpus',
      variable: 'usage_cpus',
    }),
    ...subcommandOptions,
  ],
  acceptPidsLimit: false,
});
/**
 Converts the base image build time from milliseconds to the seconds the log line reports.
 */
const MILLISECONDS_PER_SECOND = 1_000;
/**
 Compiler image on the same Debian release (bookworm) as the base image's Node image.
 */
const compilerImage = 'docker.io/library/rust:1.97-bookworm';

/**
 Build the base image (Git builds, Node, `ssh-keygen`) from the end-to-end Containerfile; nothing is copied in.
 */
async function buildBase(): Promise<void> {
  /**
   Start time of the build, for the duration the log line reports.
   */
  const startedAt = performance.now();
  await runCommand({
    command: 'podman',
    args: [
      'build',
      ...memoryArguments(containerLimits),
      '--tag',
      baseImage,
      '--file',
      resolve('e2e/concurrent-commits.Containerfile'),
      resolve('e2e'),
    ],
  });
  console.log(`built ${baseImage} in ${String(Math.round((performance.now() - startedAt) / MILLISECONDS_PER_SECOND))} s`);
}

/**
 Build the two-stage image: compile the release executable, then link it as `git` into the end-to-end image.
 */
async function buildImage(): Promise<void> {
  /**
   Result of asking whether the base image exists; a missing image is built first.
   */
  const base = await runCommand({
    command: 'podman',
    args: [
      'image',
      'exists',
      baseImage,
    ],
    allowFailure: true,
    capture: true,
  });
  if (base.status !== 0)
    await buildBase();
  /**
   Disposable build context that holds the snapshot, the vendored sources and the Containerfile.
   */
  await using context = await mkdtempDisposable(join(
    scratchBase,
    'cli-git-e2e-native-',
  ));
  await copyEndToEndSnapshot({ context: context.path });
  await vendorLockedDependencies({ context: context.path });
  await writeFile(
    join(
      context.path,
      'Containerfile',
    ),
    [
      `FROM ${compilerImage} AS compile`,
      'COPY package /work/package',
      'COPY vendor /work/vendor',
      'COPY cargo-config /work/.cargo',
      `ENV CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=${String(parallelJobs(containerLimits))} CARGO_INCREMENTAL=0`,
      'WORKDIR /work/package/git-policy/cli',
      'RUN ["cargo", "build", "--offline", "--locked", "--release", "--bin", "cli-git-native", "--bin", "cli-git-service"]',
      '',
      `FROM ${baseImage}`,
      'RUN ["mkdir", "--parents", "/opt/native/bin"]',
      // The artifact is named `cli-git-native` so it never shadows Git by accident; inside this disposable image it
      // becomes the `git` the scenarios find first on PATH.
      'COPY --from=compile /work/package/git-policy/cli/target/release/cli-git-native /opt/native/cli-git-native',
      'RUN ["ln", "--symbolic", "/opt/native/cli-git-native", "/opt/native/bin/git"]',
      // The commit service sits beside the wrapper, where the wrapper looks for it; the suite starts it before the scenarios.
      'COPY --from=compile /work/package/git-policy/cli/target/release/cli-git-service /opt/native/cli-git-service',
      '',
    ].join('\n'),
  );
  await runCommand({
    command: 'podman',
    args: [
      'build',
      '--network=none',
      '--http-proxy=false',
      '--pull=never',
      ...buildLimitArguments(containerLimits),
      '--tag',
      nativeImage,
      context.path,
    ],
  });
  console.log(`built ${nativeImage}`);
}

/**
 Run the suite in the native image under the suite's bounds.
 */
async function runSuite(): Promise<void> {
  await runCommand({
    command: 'podman',
    args: [
      'run',
      ...runLimitArguments({
        limits: containerLimits,
        withPidsLimit: false,
      }),
      '--rm',
      '--network=none',
      '--env',
      'E2E_WRAPPER=native',
      '--env',
      `E2E_SEED=${process.env
        .usage_seed
        ?? ''}`,
      '--env',
      `E2E_SCENARIOS=${process.env
        .usage_scenario
        ?? ''}`,
      '--env',
      `E2E_GIT_VERSIONS=${process.env
        .usage_git
        ?? ''}`,
      '--volume',
      // The shared label (`z`), not the private one (`Z`): runs of two scenarios from one checkout mount the same
      // directories, and a private label given to the second container takes the access away from the first
      // (observed: `EACCES` and `Cannot find module` inside runs that started together).
      `${resolve('e2e')}:/fixture/e2e:ro,z`,
      '--volume',
      `${resolve('../../caught-value/core')}:/fixture/caught-value:ro,z`,
      nativeImage,
      'node',
      '/fixture/e2e/concurrent-e2e-consumer.ts',
    ],
  });
}

// Dispatch on the subcommand.
/**
 The step each subcommand runs.
 */
const actions = {
  base: buildBase,
  image: buildImage,
  run: runSuite,
};
/**
 The step the subcommand names, absent for an unknown subcommand.
 */
const action = Object.entries(actions)
  .find(function named([name]) {
    return name === subcommand;
  })?.[1];
if ((extra.length > 0) || (action === undefined))
  throw new NativeVerificationError('Usage: node bin/e2e-native-container.ts base|image|run [--memory <size>] [--cpus <count>]');
await action();
