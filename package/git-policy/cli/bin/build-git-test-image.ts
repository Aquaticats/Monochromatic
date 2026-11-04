#!/usr/bin/env node
/**
 Build the gate's base image: the newest CachyOS image on its `latest` tag with the Rust toolchain, Git, Node, pnpm, `strace` and
 cargo-nextest, all current pacman packages, and report which release of each the image got.

 `native:test:container` does the same before it builds the gate image, so this runner exists to build the base ahead of
 time, to rebuild it after a CachyOS update with chosen limits, and to prove what it holds. It pulls the `latest` tag and
 builds only when the pull or `native-test-base.Containerfile` is new to this host.
 */
import {
  mkdir,
  mkdtemp,
  writeFile,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import {
  parseContainerLimitOptions,
  runLimitArguments,
} from './container-limits.ts';
import { ensureTestBase, } from './native-test-base.ts';
import {
  NativeVerificationError,
  runCommand,
} from './native-verification-process.ts';

/**
 Spaces of indentation in the evidence files.
 */
const jsonIndent = 2;

/**
 Pacman packages whose installed releases the build reports: the tools the tests run, taken from the distribution as
 it is (the owner chose no pins, 2026-11-04), so the release each run got is evidence worth keeping.
 */
const reportedPackages = [
  'git',
  'nodejs',
  'pnpm',
  'strace',
  'cargo-nextest',
  'rust',
] as const;

/**
 Container limits from the `--memory`, `--cpus` and `--pids-limit` options, and any extra arguments, which are a usage error.
 */
const {
  limits: containerLimits,
  rest: extra,
} = parseContainerLimitOptions({
  options: process.argv
    .slice(2),
  acceptPidsLimit: true,
});
if (extra.length > 0)
  throw new NativeVerificationError(`Only --memory <size>, --cpus <count> and --pids-limit <count> are accepted, got ${extra.join(' ')}.`);
/**
 Directory under the package's `target/` that keeps the evidence of every image build.
 */
const evidenceRoot = resolve('target/verification');
await mkdir(
  evidenceRoot,
  { recursive: true },
);
/**
 Evidence directory of this build.
 */
const evidence = await mkdtemp(join(
  evidenceRoot,
  'git-image-',
));
/**
 The base image, built now when the registry's current CachyOS image or its Containerfile is new to this host.
 */
const base = await ensureTestBase({ limits: containerLimits, });
/**
 Result of asking the base which release of each reported package it holds, in a container without network.
 */
const reported = await runCommand({
  command: 'podman',
  args: [
    'run',
    '--rm',
    '--init',
    '--network=none',
    ...runLimitArguments({
      limits: containerLimits,
      withPidsLimit: true,
    }),
    base.id,
    'pacman',
    '--query',
    ...reportedPackages,
  ],
  capture: true,
});
/**
 One `<package> <release>` line per reported package, as pacman printed them.
 */
const packages = reported.stdout
  .trim()
  .split('\n');
if (packages.length !== reportedPackages.length)
  throw new NativeVerificationError(`The base image reported ${String(packages.length)} packages, not the ${String(reportedPackages.length)} asked for: ${reported.stdout}`);
/**
 Result of asking the base's Git for its version line, in a container without network.
 */
const gitReported = await runCommand({
  command: 'podman',
  args: [
    'run',
    '--rm',
    '--init',
    '--network=none',
    ...runLimitArguments({
      limits: containerLimits,
      withPidsLimit: true,
    }),
    base.id,
    '/usr/bin/git',
    '--version',
  ],
  capture: true,
});
/**
 The version line the base's Git printed.
 */
const version = gitReported.stdout
  .trim();
if (!version.startsWith('git version '))
  throw new NativeVerificationError(`The base image returned an unexpected Git version line: ${version}`);
await writeFile(
  join(
    evidence,
    'result.json',
  ),
  `${JSON.stringify(
    {
      ...base,
      version,
      packages,
    },
    null,
    jsonIndent,
  )}\n`,
);
console.log(`${base.built ? 'Built' : 'Found'} the gate base ${base.tag} (${base.id}) on ${base.cachyosImage} (${base.cachyosImageId}); ${version}; ${packages.join(', ')}; evidence ${evidence}`);
