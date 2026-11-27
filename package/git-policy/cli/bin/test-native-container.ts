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
} from './container-limits.ts';
import {
  gateImageTagOf,
  nativeTestImageRepository,
} from './native-gate-image-tag.ts';
import { runGateChecks, } from './native-gate-checks.ts';
import { gateClippyToolchain, } from './native-gate-clippy-toolchain.ts';
import { runGateContainer, } from './native-gate-container.ts';
import { gateRunLineOf, } from './native-gate-container-identity.ts';
import { gateRunContainersOf, } from './native-gate-run-containers.ts';
import { gateRunManifestOf, } from './native-gate-run-manifest.ts';
import {
  buildMemoryLine,
  gateContainerBounds,
  gateContainerLimitOptions,
  gateLimitsFrom,
} from './native-gate-container-limits.ts';
import {
  fixtureRootArguments,
  gateFixtureRoot,
  gateFixtureRootMountOptions,
} from './native-gate-fixture-root.ts';
import { gatePassedRecord, } from './native-gate-proof.ts';
import { removeReplacedImage, } from './native-gate-replaced-image.ts';
import { syscallProfileForTheGateContainers, } from './native-gate-syscall-profile.ts';
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
  localImageNames,
} from './native-image-retention.ts';
import { testContainerfile, } from './native-test-containerfile.ts';
import { repositorySettingsFileName, } from './native-test-settings-file.ts';
import { copySourceSnapshot, } from './native-test-snapshot.ts';
import {
  NativeVerificationError,
  runCommand,
} from './native-verification-process.ts';
import { wrapperCrates, } from './native-wrapper-snapshot.ts';

// Resource limits default to the repository's test-container bounds, with the gate's own CPU count; `--memory`, `--cpus`
// and `--pids-limit` after the `--` of the task override them, except that the process bound never goes below what the
// test threads need, the image build takes its own measured memory bound rather than the suite's, and Clippy takes one of
// its own for the same reason (bin/native-gate-container-limits.ts).
/**
 The limits the parser read, the arguments left over, which are a usage error, and which limit options it read.
 */
const {
  limits: givenLimits,
  rest: unknownOptions,
  givenFlags: givenLimitFlags,
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
 The limits this gate's containers run under, with the process bound raised to what its test threads need, Clippy's own
 memory bound, and the bound its image build runs under.
 */
const {
  limits: containerLimits,
  clippyLimits,
  buildLimits,
  raised: raisedProcessBound,
} = gateLimitsFrom({
  given: givenLimits,
  givenFlags: givenLimitFlags,
});
if (raisedProcessBound !== undefined)
  console.log(raisedProcessBound);
// The build compiles the crate rather than running it, so it has a bound of its own; the line says which bound each
// container of this gate has, which is what a log needs when a build or a suite meets its own cap.
console.log(buildMemoryLine({
  limits: containerLimits,
  clippyLimits,
  buildLimits,
},),);

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
// Every container this run starts carries this id in its name and its label, so a session that wants to stop its own
// gate stops a container whose label is this run's and never one it guessed at (issue 6334).
/**
 The containers of this gate run, whose run id is the random name of the run's evidence directory.
 */
const run = gateRunContainersOf({ evidenceDirectory: evidence, },);
// Podman's default seccomp profile allows `mount`, `mount_setattr` and `move_mount` but not `statmount`, and answers an
// unlisted call with ENOSYS, which the sweep reads as a kernel too old to name a mount's source. The gate tests the
// product where the product runs, so its containers get the host's own profile with the two mount-number calls added,
// and they state that allowance, which is what makes a control that cannot read the call fail instead of skip
// (doc/troubleshooting/podman-default-seccomp-blocks-statmount-in-a-container.md).
/**
 The seccomp profile, and the statement about it, that this run's containers carry.
 */
const syscallProfile = await syscallProfileForTheGateContainers({ evidenceDirectory: evidence, },);
for (const line of syscallProfile.lines) console.log(line,);
// The run's id is printed before the build, so the name and label every container of this run carries are in the log
// from its first minute, not only once a container starts.
console.log(gateRunLineOf({ runId: run.runId, },),);
// The base is the newest CachyOS image on its `latest` tag with pacman's current Git, Node, pnpm, strace, cargo-nextest and the Rust toolchain on top
// (doc/decision/container-base-distro.md): the pull happens here, once, and every step after it is offline.
// Its own build keeps the repository's shared memory default: it compiles nothing, and no measurement of it exists, so
// the gate's build bound above is the test image's alone.
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
    ...buildLimitArguments(buildLimits),
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
await removeReplacedImage({
  before: namesBeforeBuild,
  name: testImage,
},);
/**
 Bounds both gate containers share: removed on exit, no network, and the syscall profile the sweep needs.
 */
const sharedLimits = [
  '--rm',
  '--init',
  '--network=none',
  ...syscallProfile.arguments,
];
/**
 Each gate container's complete `podman run` bounds: the flags above and its own memory, CPUs, processes and disk weight,
 which is the suite's repository default and Clippy's own measured bound.
 */
const bounds = gateContainerBounds({
  shared: sharedLimits,
  limits: containerLimits,
  clippyLimits,
});
// The manifest is written before any container starts, and after the image exists because it carries the image's ID: a
// gate that refuses during the build leaves only the log's run-id line, which names the run and what every one of its
// containers would have been called.
/**
 This run's manifest, whose writer records each container's ID as its client reports it.
 */
const evidenceManifest = gateRunManifestOf({
  evidenceDirectory: evidence,
  base,
  baseTag: testBase.tag,
  cachyosImage: testBase.cachyosImage,
  cachyosImageId: testBase.cachyosImageId,
  image,
  fixtureRoot: {
    path: gateFixtureRoot,
    mount: gateFixtureRootMountOptions,
  },
  limits: bounds.suite,
  clippyLimits: bounds.clippy,
  run,
  user: '1000:1000',
},);
await evidenceManifest.writeManifest();
console.log(`Native wrapper verification evidence: ${evidence}`);
// The Clippy step takes the repository's locked toolchain and refuses when its sysroot holds no Clippy: a sysroot
// taken from the caller's environment was the default `nightly`, which has no Clippy component, so the image's older
// `clippy-driver` compiled this crate against metadata a newer rustc wrote and failed with E0514
// (doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md).
/**
 The repository's locked Rust toolchain, and the sysroot the Clippy container mounts read-only.
 */
const clippyToolchain = await gateClippyToolchain({ startDirectory: subjectDirectory, },);
console.log(`The Clippy step runs the repository's locked Rust toolchain ${clippyToolchain.toolchain}: ${clippyToolchain.sysroot}`);
// A container's ID exists only while its `--rm` container does, so the manifest is written again as each ID is read and
// once this scope ends, whether the gate passed or refused: a refused gate is the one whose containers a person most
// needs to name.
/**
 This run's manifest, rewritten when the scope ends with the IDs its containers' clients reported.
 */
await using manifestWithContainerIds = {
  async [Symbol.asyncDispose](): Promise<void> {
    await evidenceManifest.writeManifest();
  },
};
// The suite and Clippy start together and the gate waits for both: a clean Clippy is silent, and a Clippy compile
// behind the suite cost a fifth of the gate's wall time while both checks had nothing to say to each other. Each is
// bounded, so a container that never ends refuses the gate instead of holding it open
// (bin/native-gate-checks.ts), and each removes its container whether it ended by itself or at that bound.
await runGateChecks({
  testRun: function runTheWholeSuite(deadlineMilliseconds) {
    return runGateContainer({
      cidFile: run.suite
        .cidFile,
      identity: run.suite
        .identity,
      limits: bounds.suite,
      deadlineMilliseconds,
      args: [
        ...fixtureRootArguments(),
        image,
      ],
      onStarted: evidenceManifest.recorded('suite',),
    },);
  },
  // Its diagnostics are captured rather than streamed, so the run stays the one stream a person reads and a refused
  // Clippy still quotes what Clippy said.
  clippyRun: function runClippy(deadlineMilliseconds) {
    return runGateContainer({
      cidFile: run.clippy
        .cidFile,
      identity: run.clippy
        .identity,
      limits: bounds.clippy,
      deadlineMilliseconds,
      args: [
        '--security-opt',
        'label=disable',
        '--volume',
        `${clippyToolchain.sysroot}:/toolchain:ro`,
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
      onStarted: evidenceManifest.recorded('clippy',),
    },);
  },
},);
await writeFile(
  join(
    evidence,
    'passed.json',
  ),
  `${JSON.stringify(gatePassedRecord(image))}\n`,
);
