/**
 Container matrix runner for the toolchain providers.

 Mirrors `src/package/mise.container-test.ts` (the ensure-package pattern:
 a `*.unit.matrix.test.ts` executed inside a container built by
 `@monochromatic-dev/test-harness-core-matrix`'s command builder), with three hard
 bounds the matrix runner's own spawn lacks: `podman run --memory=2g
 --cpus=2` per cell, an explicit output `maxBuffer`, and one cell at a time.

 Cells: ubuntu as root and as a passwordless-sudo user. Runtime bun,
 matching the ensure-package matrix default: it installs into each cell's own
 home, so the non-root cell needs no write access to the mounted worktree.

 @example
 ```bash
 node package/file-enforcer/core/src/toolchain/mise.container-test.ts
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
} from './container-runner.ts';

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
 Cells of the matrix: one distro across both user contexts, one at a time.
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

//endregion Matrix definition

//region Run

/**
 Directories the matrix test reads inside the container.
 */
const READ_DIRS: readonly string[] = [
  'package/module',
  'package/file-enforcer/core/src/toolchain',
];

// Rootless podman shows the mounted worktree as container-root-owned, so the
// non-root cell can only read world-readable files. Agent tooling in this
// worktree creates sources with mode 600; heal readability (read and traverse
// bits only, never content) for the directories the test reads.
await Promise.all(READ_DIRS.map(
  function healReadability(dir: string,): Promise<unknown> {
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
 Labels of the cells that failed.
 */
const failedCells: string[] = [];

for (const cell of CELLS) {
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
    `[toolchain-matrix] ${cell.os} user=${cell.user}: podman run ${
      PODMAN_BOUNDS.join(' ',)
    }, maxBuffer ${
      String(MAX_CAPTURED_OUTPUT_BYTES,)
    }`,
  );
  /**
   Bounded cell run: capped capture, capped wall clock, capped container.
   */
  // oxlint-disable-next-line no-await-in-loop -- one cell at a time bounds the host load (AGENTS.md RXI)
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
    continue;
  }
  failedCells.push(`${cell.os} user=${cell.user}`,);
  console.error(
    `[toolchain-matrix] ${cell.os} user=${cell.user} failed (exit ${
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
}

process.stdout
  .write(
  failedCells.length === 0
    ? `[toolchain-matrix] ${
      CELLS.length
    } cells passed\n`
    : `[toolchain-matrix] ${failedCells.length} of ${
      CELLS.length
    } cells failed\n`,
  );
if (failedCells.length > 0)
  process.exitCode = 1;

//endregion Run
