/**
 Fixed names, bounds and the error class of the fuzz campaign containers.
 */
import { join } from 'node:path';

/**
 Preserve the failed verification boundary rather than masking it with a later copy error.
 */
export class FuzzVerificationError extends Error {
  /**
   Stable class name that tells a failed fuzz step apart from an unrelated error in a log.
   */
  override name = 'FuzzVerificationError';
}

/**
 Scratch lives inside the package's gitignored node_modules/.monochromatic, never the system temporary directory,
 whose backing device is unknowable.
 */
export const scratchBase: string = join(
  import.meta.dirname,
  '..',
  'node_modules',
  '.monochromatic',
  'monochromatic-lint-fuzz',
);

/**
 Concurrent worktrees and agents set MONOCHROMATIC_LINT_IMAGE_TAG so one campaign never runs another's image.
 */
const imageTag = process.env
  .MONOCHROMATIC_LINT_IMAGE_TAG
  ?? 'development';

/**
 Instrumented build image: package inputs, vendored dependencies and cargo-fuzz.
 */
export const buildImage: string = `localhost/monochromatic-lint-fuzz-build:${imageTag}`;

/**
 Mount-free run image: instrumented binaries, seeds and dictionary only.
 */
export const runImage: string = `localhost/monochromatic-lint-fuzz-run:${imageTag}`;

/**
 Same installed Rust runtime image as the subject container tests; the compiler is a separate read-only input.
 */
export const baseImage = '62ba2f7ce22ba9bc501110d3452c7ae814fba367c46f7eea3629a79286353884';

/**
 Container bounds every campaign container uses.
 */
export const limits: readonly string[] = [
  '--init',
  '--network=none',
  '--memory=2g',
  '--cpus=2',
  '--pids-limit=128',
];

/**
 The fuzz targets the campaign builds and runs.
 */
export const targets: readonly string[] = [
  'merge_values',
  'configuration',
  'rust_style',
  'rust_explicit_types',
  'markdown',
  'orchestration',
];

/**
 Arguments of `podman build` that bound the build container, shared by both images.
 */
export const buildBounds: readonly string[] = [
  '--network=none',
  '--http-proxy=false',
  '--pull=never',
  '--memory=2g',
  '--cpu-period=100000',
  '--cpu-quota=200000',
];
