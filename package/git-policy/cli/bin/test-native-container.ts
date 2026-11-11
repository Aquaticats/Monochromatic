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
import {
  gateImageTagOf,
  nativeTestImageRepository,
} from './native-gate-image-tag.ts';
import { runGateChecks, } from './native-gate-checks.ts';
import {
  gateContainerLimitOptions,
  gateLimitsFrom,
} from './native-gate-container-limits.ts';
import { gatePassedRecord, } from './native-gate-proof.ts';
import {
  imageRootDirectory,
  writeDependencyStubs,
} from './native-layered-build.ts';
import { ensureTestBase, } from './native-test-base.ts';
import { vendorLockedDependencies, } from './native-scanner-snapshot.ts';
import {
  describeProblem,
  sidecarLockProblems,
} from './native-sidecar-lock-check.ts';
import {
  type ImageName,
  imageNames,
  imagesToRemove,
} from './native-image-retention.ts';
import { testContainerfile, } from './native-test-containerfile.ts';
import { repositorySettingsFileName, } from './native-test-settings-file.ts';
import { copySourceSnapshot, } from './native-test-snapshot.ts';
import {
  NativeVerificationError,
  runCommand,
} from './native-verification-process.ts';
import { wrapperCrates, } from './native-wrapper-snapshot.ts';

/**
 Every name of the local image store, as the retention decisions read it.

 @returns one entry per name podman lists; an unreadable listing is empty, so a gate never fails over it
 */
async function localImageNames(): Promise<readonly ImageName[]> {
  /**
   Result of asking podman for the store, which a host without podman answers with a failure instead of output.
   */
  const listing = await runCommand({
    command: 'podman',
    args: [
      'images',
      '--format',
      '{{.ID}} {{.Repository}}:{{.Tag}}',
    ],
    capture: true,
    allowFailure: true,
  });
  return imageNames({ output: listing.stdout, },);
}

// Resource limits default to the repository's test-container bounds, with the gate's own CPU count; `--memory`, `--cpus`
// and `--pids-limit` after the `--` of the task override them, except that the process bound never goes below what the
// test threads need (bin/native-gate-container-limits.ts).
/**
 The limits the parser read, and the arguments left over, which are a usage error.
 */
const {
  limits: givenLimits,
  rest: unknownOptions,
} = parseContainerLimitOptions({
  // The gate's own bounds come first, so a person's option after them wins: the parser keeps the last repeat.
  options: [
    ...gateContainerLimitOptions,
    ...process.argv
      .slice(2),
  ],
  acceptPidsLimit: true,
});
if (unknownOptions.length > 0)
  throw new NativeVerificationError(`Only --memory <size>, --cpus <count> and --pids-limit <count> are accepted, got ${unknownOptions.join(' ')}.`);
/**
 The limits this gate's containers run under, with the process bound raised to what its test threads need.
 */
const {
  limits: containerLimits,
  raised: raisedProcessBound,
} = gateLimitsFrom({ given: givenLimits, });
if (raisedProcessBound !== undefined)
  console.log(raisedProcessBound);

// The gate is this package's authority, so it reads the sidecars' locks before it builds anything: a dependency this
// package gained, dropped or moved makes a sidecar's lock stale, and the sidecar's own build only finds that later.
/**
 The package directory this gate runs in, which its sidecars sit beside.
 */
const subjectDirectory = resolve(
  import.meta.dirname,
  '..',
);
/**
 Every sidecar beside the package whose lock no longer agrees with its own.
 */
const sidecarProblems = await sidecarLockProblems({ packageDirectory: subjectDirectory, });
/**
 What each disagreeing sidecar says, one block of lines each.
 */
const sidecarReports = sidecarProblems.map(function described(problem) {
  return describeProblem({
    problem,
    packageDirectory: subjectDirectory,
  });
});
if (sidecarProblems.length > 0)
  throw new NativeVerificationError(sidecarReports.join('\n'));

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
const imageTag = gateImageTagOf(process.env);
/**
 Source snapshot image this gate builds, tests, and lints.
 */
const testImage = `${nativeTestImageRepository}:${imageTag}`;
/**
 Every name of the image store before this build, so the build removes the image it replaced instead of leaving one
 untagged leftover of every earlier gate on the disk (issue 991).
 */
const namesBeforeBuild: readonly ImageName[] = await localImageNames();

// Build one mount-free source image, then run tests and Clippy against that exact image identity.
/**
 Disposable build context that holds the snapshot, the vendored sources and the Containerfile.
 */
await using context = await mkdtempDisposable(join(
  scratchBase,
  'monochromatic-git-policy-cli-test-',
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
// The base is the newest CachyOS image on its `latest` tag with pacman's current Git, Node, pnpm, strace, cargo-nextest and the Rust toolchain on top
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
await writeFile(
  join(
    context.path,
    'Containerfile',
  ),
  testContainerfile({
    base,
    jobs: parallelJobs(containerLimits),
    settingsFile: await repositorySettingsFileName(),
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
 Images this build replaced, which is the one the tag carried before it while only the untagged marker names it
 now: an image another tag names, and one a container still runs, stay where they are.
 */
const replacedImages: readonly string[] = imagesToRemove({
  before: namesBeforeBuild,
  after: await localImageNames(),
  name: testImage,
},);
/**
 The image this build replaced, which is none when the build reused the image the tag carried or a name holds it.
 */
const [replacedImage] = replacedImages;
if (replacedImage !== undefined) {
  /**
   Result of the removal, which podman refuses while a container still runs the image.
   */
  const removal = await runCommand({
    command: 'podman',
    args: [
      'image',
      'rm',
      replacedImage,
    ],
    capture: true,
    allowFailure: true,
  });
  console.log(removal.status === 0
    ? `Removed the gate image this build replaced: ${replacedImage}`
    : `Kept the gate image this build replaced: ${replacedImage} (${removal.stderr
      .trim()})`,);
}
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
/**
 Result of asking the host's `rustc` for its sysroot, which the Clippy container mounts read-only.
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
// The suite and Clippy start together and the gate waits for both: a clean Clippy is silent, and a Clippy compile
// behind the suite cost a fifth of the gate's wall time while both checks had nothing to say to each other.
await runGateChecks({
  testRun: function runTheWholeSuite() {
    return runCommand({
      command: 'podman',
      args: [
        'run',
        ...limits,
        image,
      ],
    });
  },
  // Its diagnostics are captured rather than streamed, so the run stays the one stream a person reads and a refused
  // Clippy still quotes what Clippy said.
  clippyRun: function runClippy() {
    return runCommand({
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
      capture: true,
      allowFailure: true,
    });
  },
});
await writeFile(
  join(
    evidence,
    'passed.json',
  ),
  `${JSON.stringify(gatePassedRecord(image))}\n`,
);
