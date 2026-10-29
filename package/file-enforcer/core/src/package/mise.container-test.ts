/**
 Container test runner for ensurePackage.
 Launches podman containers across the test matrix via
 `@monochromatic-dev/test-harness-core-matrix`'s command builder.

 Hard bounds (AGENTS.md RXI, the synthetic-load ban): every cell runs
 `podman run --memory=2g --cpus=2 --rm` with an explicit output `maxBuffer`
 and a wall-clock timeout (`../toolchain/container-runner.ts`), one cell at a
 time. The matrix runner's own spawn carries none of these bounds: an
 uncapped capture drove this runner to 42 GB RSS on 2026-10-26 and was OOM
 killed.

 Test matrix:
 - OS: ubuntu (apt), fedora (dnf)
 - User: root, non-root (uid 1000)

 @example
 ```bash
 node package/file-enforcer/core/src/package/mise.container-test.ts
 ```
 */

import {
  join,
  resolve,
} from 'node:path';
import {
  buildContainerCommand,
  parseOs,
  type Combination,
  type UserContext,
} from '@monochromatic-dev/test-harness-core-matrix/ts';
import spawn from 'nano-spawn';
import {
  MAX_CAPTURED_OUTPUT_BYTES,
  PODMAN_BOUNDS,
  runContainerCell,
} from '../toolchain/container-runner.ts';

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
  'ensure-package.unit.matrix.test.ts',
);

/**
 Cells of the matrix, one at a time.
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
  {
    os: 'container:fedora',
    user: 'root',
  },
  {
    os: 'container:fedora',
    user: 'user',
  },
] as const;

//endregion Matrix definition

//region Run

/**
 Directories the matrix test reads inside the container.
 */
const READ_DIRS: readonly string[] = [
  'package/module',
  'package/file-enforcer/core/src',
];

// Rootless podman shows the mounted worktree as container-root-owned, so the
// non-root cells can only read world-readable files; heal readability (read
// and traverse bits only) for the directories the tests read.
await Promise.all(READ_DIRS.map(
  function healReadability(dir,): Promise<unknown> {
    return spawn(
      'chmod',
      [
        '-R',
        'a+rX',
        join(
          MONOREPO_ROOT,
          dir
        ),
      ],
    );
  },
),);

/**
 Runs one matrix cell in a bounded container and reports its output.

 @param cell - Matrix cell to run

 @returns `true` when the cell's test run exited zero
 */
async function runCell(cell: (typeof CELLS)[number],): Promise<boolean> {
  /**
   Combination the matrix runner would have built for this cell.
   */
  const combination: Combination = {
    file: TEST_FILE,
    os: cell.os,
    user: cell.user,
    runtime: 'bun',
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
  const {distro} = parseOs(cell.os,);
  /**
   Fully tagged image reference.
   */
  const image = distro.includes(':',)
    ? distro
    : `${distro}:latest`;
  console.log(
    `[ensure-package-matrix] ${cell.os} user=${cell.user}: podman run ${
      PODMAN_BOUNDS.join(' ',)
    }, maxBuffer ${
      String(MAX_CAPTURED_OUTPUT_BYTES,)
    }`,
  );
  /**
   Bounded cell run: capped capture, capped wall clock, capped container.
   */
  const result = await runContainerCell({
    image,
    command,
    monorepoRoot: MONOREPO_ROOT,
  },);
  if (result.code === 0) {
    process.stdout
      .write(`${
      result.stdout
    }\n`,);
    return true;
  }
  console.error(
    `[ensure-package-matrix] ${cell.os} user=${cell.user} failed (exit ${
      String(result.code,)
    }):`,
  );
  process.stderr
    .write(`${
    result.stderr
  }\n`,);
  process.stdout
    .write(`${
    result.stdout
  }\n`,);
  return false;
}

/**
 Cells that failed, as `os/user` labels.
 */
const failedCells: string[] = [];

for (const cell of CELLS) {
  // oxlint-disable-next-line no-await-in-loop -- one cell at a time keeps podman memory and CPU use bounded
  if (!(await runCell(cell,)))
    failedCells.push(`${cell.os}/${cell.user}`,);
}

process.stdout
  .write(
  failedCells.length === 0
    ? `[ensure-package-matrix] ${
      CELLS.length
    } cells passed\n`
    : `[ensure-package-matrix] ${failedCells.length} of ${
      CELLS.length
    } cells failed\n`,
  );
if (failedCells.length > 0)
  process.exitCode = 1;

//endregion Run
