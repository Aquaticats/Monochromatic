/**
 Fixed names, paths and bounds of the scanner's source snapshots.
 */
import { resolve } from 'node:path';

import { diskWeightArguments } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts';

/**
 Content-addressed ID of the base image every snapshot builds on.
 */
export const baseImage = '6ec87f6d290a2f59bda5b3ffd4197058fe0749d02b4978c877e8edf6dc38802a';

/**
 Repository-relative directory of the scanner inside the image.
 */
export const scanner = 'package/forbidden-strings/cli';

/**
 Container bounds every verification run uses: memory, CPUs, processes, descriptors and, from `diskWeightArguments`, the
 lowest disk weight (repository rule HXR in `AGENTS.md`: a rootless container sits outside the launcher's unit and ignores
 the launcher's `IOWeight`).
 */
export const limits: readonly string[] = [
  '--init',
  '--network=none',
  '--memory=2g',
  '--cpus=2',
  '--pids-limit=128',
  '--ulimit',
  'nofile=4096:4096',
  ...diskWeightArguments(),
];

/**
 Directory of the scanner package, which holds this script's `bin`.
 */
export const scannerDirectory: string = resolve(
  import.meta.dirname,
  '..',
);

/**
 Root of the repository.
 */
export const repository: string = resolve(
  scannerDirectory,
  '../../..',
);
