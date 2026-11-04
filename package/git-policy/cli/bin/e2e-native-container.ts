#!/usr/bin/env node
/**
 Run the concurrent-commit end-to-end suite (`e2e/`) against the native executable.

 Three subcommands, selected by the first argument:

 - `base` builds `localhost/cli-git-concurrent-e2e:<CachyOS image>-<Containerfile>` from
   `e2e/concurrent-commits.Containerfile`: CachyOS with pacman's current `git`, `nodejs` and `openssh` (`ssh-keygen`).
   It pulls the CachyOS `latest` tag first (one small request when nothing changed) and builds only when that pull
   or the Containerfile is new to the host, so the tag changes with either input and no run reuses a stale base.
   The build needs a network (the package manager); the suite itself does not.
 - `image` builds `localhost/cli-git-concurrent-e2e-native:<tag>` in two stages.
   The first compiles the release `cli-git-native` from a mount-free snapshot of this checkout in the gate's base
   image (`native-test-base.Containerfile`: the same CachyOS image the end-to-end base starts from, plus Rust),
   so the executable runs there without a glibc mismatch.
   The second starts from the end-to-end base image (built first when it does not exist) and links the executable as
   `/opt/native/bin/git`.
   The scenario's `PATH` puts the wrapper's directory before `/usr/bin`, so the wrapper finds the packaged Git behind
   itself.
 - `run` starts the suite in that image with `E2E_WRAPPER=native` (the container limits, no network, `e2e/` mounted
   read-only).
   `usage_seed`, `usage_scenario` and `usage_git` (set by mise from `--seed`, `--scenario` and `--git`)
   select the seed, the scenario names and the Git release (the installed Git's only).

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
  parallelJobs,
  parseContainerLimitOptions,
  runLimitArguments,
} from './container-limits.ts';
import { copyEndToEndSnapshot, } from './native-e2e-snapshot.ts';
import { vendorLockedDependencies, } from './native-scanner-snapshot.ts';
import {
  ensureEndToEndBase,
  ensureTestBase,
  pullCachyosLatest,
} from './native-test-base.ts';
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
 Make sure the base image (Git builds, Node, `ssh-keygen`) of the current CachyOS image exists; nothing is copied in.
 */
async function buildBase(): Promise<void> {
  /**
   Start time, for the duration the log line reports.
   */
  const startedAt = performance.now();
  /**
   The base image and whether this run built it.
   */
  const endToEndBase = await ensureEndToEndBase({ limits: containerLimits, });
  console.log(`${endToEndBase.built ? 'built' : 'found'} ${endToEndBase.tag} in ${String(Math.round((performance.now() - startedAt) / MILLISECONDS_PER_SECOND))} s`);
}

/**
 Build the two-stage image: compile the release executable, then link it as `git` into the end-to-end image.
 */
async function buildImage(): Promise<void> {
  // One pull serves both bases, so the executable is compiled and run on the same C library.
  /**
   The CachyOS image the pull left on the host.
   */
  const pulled = await pullCachyosLatest();
  /**
   The base that compiles the executable: CachyOS with Rust.
   */
  const compileBase = await ensureTestBase({
    limits: containerLimits,
    source: pulled,
  });
  /**
   The base the executable is linked into: CachyOS with Node and the Git under test.
   */
  const endToEndBase = await ensureEndToEndBase({
    limits: containerLimits,
    source: pulled,
  });
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
      `FROM ${compileBase.id} AS compile`,
      'COPY package /work/package',
      'COPY vendor /work/vendor',
      'COPY cargo-config /work/.cargo',
      `ENV CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=${String(parallelJobs(containerLimits))} CARGO_INCREMENTAL=0`,
      'WORKDIR /work/package/git-policy/cli',
      'RUN ["cargo", "build", "--offline", "--locked", "--release", "--bin", "cli-git-native"]',
      '',
      `FROM ${endToEndBase.id}`,
      'RUN ["mkdir", "--parents", "/opt/native/bin"]',
      // The artifact is named `cli-git-native` so it never shadows Git by accident; inside this disposable image it
      // becomes the `git` the scenarios find first on PATH.
      'COPY --from=compile /work/package/git-policy/cli/target/release/cli-git-native /opt/native/cli-git-native',
      'RUN ["ln", "--symbolic", "/opt/native/cli-git-native", "/opt/native/bin/git"]',
      // The executable was linked in another stage; `ldd` exits zero even for a missing library or symbol version, so the step
      // looks for its `not found` lines: a C library older than the one the executable was linked against fails the build here.
      'RUN ["sh", "-c", "! ldd /opt/native/cli-git-native 2>&1 | grep --quiet \'not found\'"]',
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
