/**
 In-container path constants shared by the matrix entrypoint and its steps.
 
 The monorepo is mounted read-only at {@link REPO_DIR}; the fixture is written
 and installed in the writable tmpfs at {@link WORK_DIR}; the corepack `pnpm`
 shim goes in {@link PNPM_BIN_DIR} because the rootfs is read-only.
 */

import {
  join,
} from 'node:path';

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
  'dev-script',
  'catalog-tighten',
  'src',
  'index.ts',
);

/**
 Writable tmpfs directory the corepack `pnpm` shim is installed into.
 */
export const PNPM_BIN_DIR = '/tmp/cbin';

//endregion Container paths
