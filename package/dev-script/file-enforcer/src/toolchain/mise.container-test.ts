/**
 Container matrix runner for the toolchain providers.

 Mirrors `src/package/mise.container-test.ts` (the ensure-package pattern:
 a `*.unit.matrix.test.ts` executed inside a container built by
 `@monochromatic-dev/module-matrix`), with one difference: every `podman run`
 is bounded to 2 GiB and 2 CPUs (AGENTS.md RXI), because the matrix runner's
 own spawn carries no resource bounds.

 Cells: ubuntu as root and as a passwordless-sudo user, sequentially
 (the synthetic-load ban; one bounded container at a time).

 @example
 ```bash
 node package/dev-script/file-enforcer/src/toolchain/mise.container-test.ts
 ```
 */

import { resolve, } from 'node:path';
import {
  buildContainerCommand,
  parseOs,
  type Combination,
  type UserContext,
} from '@monochromatic-dev/module-matrix/ts';
import spawn from 'nano-spawn';

//region Matrix definition

/**
 Monorepo root: the container mounts it at `/workspace`.
 */
const MONOREPO_ROOT = resolve(
  import.meta.dirname,
  '..',
  '..',
  '..',
  '..',
  '..',
);

/**
 The matrix test file executed inside each container.
 */
const TEST_FILE = resolve(
  import.meta.dirname,
  'providers.unit.matrix.test.ts',
);

/**
 Cells of the matrix: one distro across both user contexts.
 */
const CELLS: readonly {
  readonly os: string;
  readonly user: UserContext;
}[] = [
  {
    os: 'container:ubuntu',
    user: 'root',
  },
  {
    os: 'container:ubuntu',
    user: 'user',
  },
] as const;

/**
 Podman resource bounds applied to every cell.
 */
const PODMAN_BOUNDS: readonly string[] = [
  '--memory=2g',
  '--cpus=2',
] as const;

//endregion Matrix definition

//region Run

/**
 Number of cells that failed.
 */
let failures = 0;

for (const cell of CELLS) {
  /**
   Combination the matrix runner would have built for this cell.
   */
  const combination: Combination = {
    file: TEST_FILE,
    os: cell.os,
    user: cell.user,
    runtime: 'node',
  };
  /**
   Container-side command: prerequisites, user setup, runtime install, test.
   */
  const command = buildContainerCommand({
    combination,
    monorepoRoot: MONOREPO_ROOT,
  },);
  /**
   Distro half of the cell's `os` spec, tagged for `podman run`.
   */
  const distro = parseOs(cell.os,)
    .distro;
  /**
   Fully tagged image reference.
   */
  const image = distro.includes(':',)
    ? distro
    : `${distro}:latest`;
  console.log(
    `[toolchain-matrix] ${cell.os} user=${cell.user}: podman run ${
      PODMAN_BOUNDS.join(' ',)
    }`,
  );
  try {
    /**
     Container run result; its stdout carries the test report.
     */
    const result = await spawn(
      'podman',
      [
        'run',
        '--rm',
        ...PODMAN_BOUNDS,
        // label=disable instead of a :Z relabel: the repository sits on a
        // volume where SELinux label xattrs are refused (lsetxattr: operation
        // not permitted), which aborts the mount setup before any test runs.
        '--security-opt',
        'label=disable',
        '--volume',
        `${MONOREPO_ROOT}:/workspace`,
        image,
        'sh',
        '-c',
        command,
      ],
    );
    process.stdout.write(`${
      result.stdout
    }\n`,);
  } catch (error) {
    failures += 1;
    /**
     Failure details from the spawn error.
     */
    const failure = error as {
      readonly stdout?: string;
      readonly stderr?: string;
      readonly message?: string;
    };
    console.error(
      `[toolchain-matrix] ${cell.os} user=${cell.user} failed: ${
        failure.stderr ?? failure.message ?? String(error,)
      }`,
    );
    if (failure.stdout !== undefined && failure.stdout !== '')
      process.stdout.write(`${
        failure.stdout
      }\n`,);
  }
}

process.stdout.write(
  failures === 0
    ? `[toolchain-matrix] ${
      CELLS.length
    } cells passed\n`
    : `[toolchain-matrix] ${failures} of ${
      CELLS.length
    } cells failed\n`,
);
if (failures > 0)
  process.exitCode = 1;

//endregion Run
