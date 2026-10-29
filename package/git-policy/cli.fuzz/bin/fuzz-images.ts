/**
 Images and bounds the fuzz container campaign shares between its build and run phases.
 */

/** Same installed Rust runtime image as the linter's fuzz sidecar; the targets are pure and need no Git. */
export const baseImage = '62ba2f7ce22ba9bc501110d3452c7ae814fba367c46f7eea3629a79286353884';

/** Concurrent worktrees set GIT_POLICY_NATIVE_IMAGE_TAG so one snapshot never runs another's image. */
const imageTag = process.env
  .GIT_POLICY_NATIVE_IMAGE_TAG
  ?? 'development';

/** Image holding the vendored sources and the toolchain-free build inputs. */
export const buildImage: string = `localhost/git-policy-cli-fuzz-build:${imageTag}`;

/** Image holding only the instrumented binaries, corpus and dictionaries. */
export const runImage: string = `localhost/git-policy-cli-fuzz-run:${imageTag}`;

/** Largest input libFuzzer generates, in bytes; recorded in the manifest. */
export const maxInputBytes = 4_096;

/** No network, 2 GiB, 2 CPUs and 128 PIDs for every container the campaign starts. */
export const limits: readonly string[] = [
  '--init',
  '--network=none',
  '--memory=2g',
  '--cpus=2',
  '--pids-limit=128',
];
