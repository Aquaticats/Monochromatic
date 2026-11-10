/**
 Paths shared by the matrix orchestrator and its in-container entrypoint.
 
 The monorepo sits at {@link HOST_REPO_ROOT} on the host and is mounted
 read-only at {@link REPO_DIR} inside each container, so every in-container
 path is the host path with that prefix swapped. The fixture is written and
 installed in the writable tmpfs at {@link WORK_DIR}; the corepack `pnpm` shim
 goes in {@link PNPM_BIN_DIR} because the rootfs is read-only.
 */

import {
  join,
} from 'node:path';

//region Host paths

/**
 Monorepo root on the host: this file lives at `<root>/<package>/src`.
 */
export const HOST_REPO_ROOT: string = join(
  import.meta.dirname,
  '..',
  '..',
  '..',
  '..',
);

//endregion Host paths

//region Container paths

/**
 Writable fixture work directory (tmpfs) inside the container.
 */
export const WORK_DIR = '/work';

/**
 Read-only monorepo mount inside the container; the tool and its deps resolve here.
 */
export const REPO_DIR = '/repo';

/**
 catalog-tighten entrypoint inside the mounted repo.
 */
export const TOOL_ENTRY: string = join(
  REPO_DIR,
  'package',
  'dependency-catalog',
  'cli-tighten',
  'src',
  'index.ts',
);

/**
 This matrix's own in-container entrypoint, inside the mounted repo.
 */
export const IN_CONTAINER_ENTRY = '/repo/packages/dev-script/catalog-tighten.matrix/src/in-container.ts';

/**
 Writable tmpfs directory the corepack `pnpm` shim is installed into.
 */
export const PNPM_BIN_DIR = '/tmp/cbin';

//endregion Container paths
