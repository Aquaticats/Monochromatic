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
 - `run` starts the suite in that image with `E2E_WRAPPER=native` (2 GiB, 2 CPUs, no network, `e2e/` mounted
   read-only).
   `usage_seed`, `usage_scenario` and `usage_git` (set by mise from `--seed`, `--scenario` and `--git`)
   select the seed, the scenario names and the Git versions.
 */
import {
  cp,
  mkdtempDisposable,
  writeFile,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import {
  scannerSnapshotEntries,
  vendorLockedDependencies,
} from './native-scanner-snapshot.ts';
import {
  NativeVerificationError,
  runCommand,
} from './native-verification-process.ts';
import { mkdirSync as makeScratchBase } from 'node:fs';

// Scratch lives inside the package's gitignored node_modules/.monochromatic, never the system temporary directory,
// whose backing device is unknowable.
const scratchBase = join(
  import.meta.dirname,
  '..',
  'node_modules',
  '.monochromatic',
  'git-policy'
);
makeScratchBase(
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
 Converts the base image build time from milliseconds to the seconds the log line reports.
 */
const MILLISECONDS_PER_SECOND = 1_000;
/**
 Compiler image on the same Debian release (bookworm) as the base image's Node image.
 */
const compilerImage = 'docker.io/library/rust:1.97-bookworm';

/**
 Copy the native crate, its path dependencies and the scanner crates into the build context.
 Copies run concurrently, and every copy settles before this returns or throws.

 @param request - build context directory receiving the repository-shaped snapshot
 */
async function copySnapshot({ context }: { context: string }) {
  const subject = join(
    context,
    'package/git-policy/cli',
  );
  const dependency = join(
    context,
    'package/jsonc-edit/core-rust',
  );
  const entries = [
    ...[
      'Cargo.toml',
      'Cargo.lock',
      'README.md',
      'src/native',
    ].map(function crateEntry(name) {
      return {
        from: resolve(name),
        to: join(
          subject,
          name,
        ),
      };
    }),
    ...[
      'Cargo.toml',
      'Cargo.lock',
      'src',
      'fixtures',
    ].map(function dependencyEntry(name) {
      return {
        from: resolve(
          '../../jsonc-edit/core-rust',
          name,
        ),
        to: join(
          dependency,
          name,
        ),
      };
    }),
    ...scannerSnapshotEntries({ context }),
  ];
  const copies = await Promise.allSettled(entries.map(function copyEntry(entry) {
    return cp(
      entry.from,
      entry.to,
      { recursive: true },
    );
  }));
  const failures = copies.filter(function isRejected(copy) {
    return copy.status === 'rejected';
  });
  if (failures.length > 0)
    throw new NativeVerificationError(
      `Copying ${String(failures.length)} snapshot entries into ${context} failed.`,
      { cause: failures },
    );
}

/**
 Build the base image (Git builds, Node, `ssh-keygen`) from the end-to-end Containerfile; nothing is copied in.
 */
async function buildBase() {
  const startedAt = performance.now();
  await runCommand({
    command: 'podman',
    args: [
      'build',
      '--memory=2g',
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
async function buildImage() {
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
  await using context = await mkdtempDisposable(join(
    scratchBase,
    'cli-git-e2e-native-',
  ));
  await copySnapshot({ context: context.path });
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
      'ENV CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0',
      'WORKDIR /work/package/git-policy/cli',
      'RUN ["cargo", "build", "--offline", "--locked", "--release", "--bin", "cli-git-native"]',
      '',
      `FROM ${baseImage}`,
      'RUN ["mkdir", "--parents", "/opt/native/bin"]',
      // The artifact is named `cli-git-native` so it never shadows Git by accident; inside this disposable image it
      // becomes the `git` the scenarios find first on PATH.
      'COPY --from=compile /work/package/git-policy/cli/target/release/cli-git-native /opt/native/cli-git-native',
      'RUN ["ln", "--symbolic", "/opt/native/cli-git-native", "/opt/native/bin/git"]',
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
      '--memory=2g',
      '--cpu-period=100000',
      '--cpu-quota=200000',
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
async function runSuite() {
  await runCommand({
    command: 'podman',
    args: [
      'run',
      '--memory=2g',
      '--cpus=2',
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
      `${resolve('e2e')}:/fixture/e2e:ro,Z`,
      '--volume',
      `${resolve('../../caught-value/core')}:/fixture/caught-value:ro,Z`,
      nativeImage,
      'node',
      '/fixture/e2e/concurrent-e2e-consumer.ts',
    ],
  });
}

// Dispatch on the subcommand.
const [subcommand, ...extra] = process.argv
  .slice(2);
const actions = {
  base: buildBase,
  image: buildImage,
  run: runSuite,
};
const action = Object.entries(actions)
  .find(function named([name]) {
    return name === subcommand;
  })?.[1];
if ((extra.length > 0) || (action === undefined))
  throw new NativeVerificationError('Usage: node bin/e2e-native-container.ts base|image|run');
await action();
