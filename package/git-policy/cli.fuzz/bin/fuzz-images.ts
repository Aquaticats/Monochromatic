/**
 Images and bounds the fuzz container campaign shares between its build and run phases.
 */
import {
  type ContainerLimits,
  defaultContainerLimits,
  runLimitArguments,
} from '../../cli/bin/container-limits.ts';

/**
 Same installed Rust runtime image as the linter's fuzz sidecar; the targets are pure and need no Git.
 */
export const baseImage = '62ba2f7ce22ba9bc501110d3452c7ae814fba367c46f7eea3629a79286353884';

/**
 Concurrent worktrees set GIT_POLICY_NATIVE_IMAGE_TAG so one snapshot never runs another's image.
 */
const imageTag = process.env
  .GIT_POLICY_NATIVE_IMAGE_TAG
  ?? 'development';

/**
 Image holding the vendored sources and the toolchain-free build inputs.
 */
export const buildImage: string = `localhost/git-policy-cli-fuzz-build:${imageTag}`;

/**
 Image holding only the instrumented binaries, corpus and dictionaries.
 */
export const runImage: string = `localhost/git-policy-cli-fuzz-run:${imageTag}`;

/**
 Largest input libFuzzer generates, in bytes; recorded in the manifest.
 */
export const maxInputBytes = 4_096;

/**
 Memory the containers that compile the targets take, in Podman's `--memory` form.

 cargo-fuzz passes `-Ccodegen-units=1`, so the whole wrapper library is one code generator, and the default 2 GiB bound
 is not enough for it: `mise run //package/git-policy/cli.fuzz:build` had the kernel kill the compiler of
 `monochromatic-git-policy-cli` at that bound (the retained container state held `OOMKilled: true`) while the same
 build fitted 8 GiB and its container cgroup reported `memory.peak` 5,059,244,032 bytes (4.71 GiB) for the whole
 AddressSanitizer build of every target (`git-policy-cli-fuzz-build:pi-fuzz-sidecar-smoke-20261110`, 2 CPUs,
 2026-11-10).
 8 GiB keeps a margin over that measured peak for the dependencies the wrapper gains next, which is why the peak was
 reached at all: the sidecar's bound was older than the wrapper's graph.
 */
export const buildMemory: string = '8g';

/**
 The default CPUs, PIDs and disk weight with the memory the instrumented build needs.
 */
export const buildContainerLimits: ContainerLimits = {
  ...defaultContainerLimits,
  memory: buildMemory,
};

/**
 No network, and the default memory, CPUs and PIDs of the container limits, for every container that runs a target.
 */
export const limits: readonly string[] = [
  '--init',
  '--network=none',
  ...runLimitArguments({
    limits: defaultContainerLimits,
    withPidsLimit: true,
  }),
];

/**
 No network, and the build memory with the default CPUs and PIDs, for the containers that compile and lint the targets.
 */
export const buildLimits: readonly string[] = [
  '--init',
  '--network=none',
  ...runLimitArguments({
    limits: buildContainerLimits,
    withPidsLimit: true,
  }),
];
