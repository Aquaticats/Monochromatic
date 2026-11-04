#!/usr/bin/env node
/**
 Verify native wrapper modules against the selected real Git in an isolated source snapshot.
 */
import {
  mkdir,
  mkdtemp,
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
import { gatePassedRecord, } from './native-gate-proof.ts';
import {
  imageRootDirectory,
  writeDependencyStubs,
} from './native-layered-build.ts';
import { copyNextest, } from './native-nextest-snapshot.ts';
import { ensureTestBase, } from './native-test-base.ts';
import { copyPnpm, } from './native-pnpm-snapshot.ts';
import { vendorLockedDependencies, } from './native-scanner-snapshot.ts';
import { copyStraceSource, } from './native-strace-snapshot.ts';
import { testContainerfile, } from './native-test-containerfile.ts';
import { copySourceSnapshot, } from './native-test-snapshot.ts';
import {
  NativeVerificationError,
  runCommand,
} from './native-verification-process.ts';
import { wrapperCrates, } from './native-wrapper-snapshot.ts';

// Resource limits default to the repository's test-container bounds; `--memory`, `--cpus` and `--pids-limit` override them.
/**
 Container limits of this gate, and the arguments left over, which are a usage error.
 */
const {
  limits: containerLimits,
  rest: unknownOptions,
} = parseContainerLimitOptions({
  options: process.argv
    .slice(2),
  acceptPidsLimit: true,
});
if (unknownOptions.length > 0)
  throw new NativeVerificationError(`Only --memory <size>, --cpus <count> and --pids-limit <count> are accepted, got ${unknownOptions.join(' ')}.`);

// Scratch lives inside the package's gitignored node_modules/.monochromatic, never the system temporary directory,
// whose backing device is unknowable.
/**
 Parent of every scratch directory this gate creates.
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
 Source snapshot image this gate builds, tests, and lints.
 */
const testImage = `localhost/git-policy-native-test:${imageTag}`;

// Build one mount-free source image, then run tests and Clippy against that exact image identity.
/**
 Disposable build context that holds the snapshot, the vendored sources and the Containerfile.
 */
await using context = await mkdtempDisposable(join(
  scratchBase,
  'cli-git-native-test-',
));
/**
 Directory under the package's `target/` that keeps the evidence of every gate run.
 */
const evidenceRoot = resolve('target/verification');
await mkdir(
  evidenceRoot,
  { recursive: true },
);
/**
 Evidence directory of this gate run.
 */
const evidence = await mkdtemp(join(
  evidenceRoot,
  'native-',
));
// The base is the newest CachyOS image on its `latest` tag with the audited Git 2.56.0 and the Rust toolchain on top
// (doc/decision/container-base-distro.md): the pull happens here, once, and every step after it is offline.
/**
 The derived base image, built now when the registry's current CachyOS image or its Containerfile is new to this host.
 */
const testBase = await ensureTestBase({ limits: containerLimits, });
/**
 Content-addressed ID of the base image shared by every stage of the gate image.
 */
const base = testBase.id;
await copySourceSnapshot({ context: context.path });
// The dependency layers copy only these stand-in trees, the vendored sources and the Cargo mapping.
await writeDependencyStubs({
  treeRoot: context.path,
  crates: wrapperCrates,
});
await vendorLockedDependencies({
  context: context.path,
  vendorName: `${imageRootDirectory}/vendor`,
  configName: `${imageRootDirectory}/.cargo`,
});
await copyNextest({ context: context.path });
await copyPnpm({ context: context.path });
await copyStraceSource({ context: context.path });
await writeFile(
  join(
    context.path,
    'Containerfile',
  ),
  testContainerfile({
    base,
    jobs: parallelJobs(containerLimits),
  }),
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
    testImage,
    context.path,
  ],
});
/**
 Result of inspecting the built image for its ID.
 */
const inspected = await runCommand({
  command: 'podman',
  args: [
    'image',
    'inspect',
    testImage,
    '--format',
    '{{.Id}}',
  ],
  capture: true,
});
/**
 Content-addressed ID of the built image, so the run and Clippy steps use exactly the tested snapshot.
 */
const image = inspected.stdout
  .trim();
/**
 Bounds of every gate container: removed on exit, no network, and the memory, CPUs and processes of the container limits.
 */
const limits = [
  '--rm',
  '--init',
  '--network=none',
  ...runLimitArguments({
    limits: containerLimits,
    withPidsLimit: true,
  }),
];
await writeFile(
  join(
    evidence,
    'manifest.json',
  ),
  `${JSON.stringify(
    {
      base,
      baseTag: testBase.tag,
      cachyosImage: testBase.cachyosImage,
      cachyosImageId: testBase.cachyosImageId,
      image,
      limits,
      user: '1000:1000',
    },
    null,
    2,
  )}\n`,
);
console.log(`Native wrapper verification evidence: ${evidence}`);
await runCommand({
  command: 'podman',
  args: [
    'run',
    ...limits,
    image,
  ],
});
/**
 Result of asking the host's `rustc` for its sysroot.
 */
const sysroot = await runCommand({
  command: 'rustc',
  args: [
    '--print',
    'sysroot',
  ],
  capture: true,
});
/**
 Host Rust toolchain directory that the Clippy step mounts read-only.
 */
const compiler = sysroot.stdout
  .trim();
await runCommand({
  command: 'podman',
  args: [
    'run',
    ...limits,
    '--security-opt',
    'label=disable',
    '--volume',
    `${compiler}:/toolchain:ro`,
    '--env',
    'PATH=/toolchain/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin',
    '--env',
    'RUSTC=/toolchain/bin/rustc',
    '--env',
    'RUSTDOC=/toolchain/bin/rustdoc',
    image,
    '/toolchain/bin/cargo',
    'clippy',
    '--offline',
    '--locked',
    '--all-targets',
    '--',
    '-D',
    'warnings',
  ],
});
await writeFile(
  join(
    evidence,
    'passed.json',
  ),
  `${JSON.stringify(gatePassedRecord(image))}\n`,
);
