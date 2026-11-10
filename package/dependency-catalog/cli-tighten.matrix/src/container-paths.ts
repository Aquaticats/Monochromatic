/**
 Paths shared by the matrix orchestrator and its in-container entrypoint.
 
 The monorepo sits at {@link HOST_REPO_ROOT} on the host and is mounted
 read-only at {@link REPO_DIR} inside each container, so every in-container
 path is the host path with that prefix swapped. The fixture is written and
 installed in the writable tmpfs at {@link WORK_DIR}; the corepack `pnpm` shim
 goes in {@link PNPM_BIN_DIR} because the rootfs is read-only. The pnpm store
 the workspace's `node_modules` entries link into is mounted at the path those
 links spell, which is not always the path pnpm reports, so
 {@link containerStoreDir} maps one to the other.
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
export const IN_CONTAINER_ENTRY: string = join(
  REPO_DIR,
  'package',
  'dependency-catalog',
  'cli-tighten.matrix',
  'src',
  'in-container.ts',
);

/**
 Writable tmpfs directory the corepack `pnpm` shim is installed into.
 */
export const PNPM_BIN_DIR = '/tmp/cbin';

//endregion Container paths

//region Store mount

/**
 Content-addressable store path as the container has to mount it.
 
 The workspace's `node_modules` entries are relative links out of the
 repository that spell the store the way pnpm resolved the workspace's
 `storeDir` at install time, so a store written under the home the installing
 shell named is linked as `<home>/.local/share/pnpm/store/v11` even when that
 home is a mount of somewhere else. pnpm's own report of the store carries
 either spelling depending on where it runs, and a container has no
 `/repo`-external path beyond the mounts it is given, so the mount destination
 has to be the spelling the links use.
 
 @param hostStoreDir - store path pnpm reports, on the host
 
 @param homeDir - home directory as the mounting process spells it
 
 @param realHomeDir - where that home actually lives, from `realpathSync`
 
 @returns absolute path inside the container the store has to be mounted at
 
 @example
 ```ts
 containerStoreDir({ hostStoreDir: '/var/canonical/home/user/.local/share/pnpm/store/v11', homeDir: '/home/user', realHomeDir: '/var/canonical/home/user' }) // '/home/user/.local/share/pnpm/store/v11'
 ```
 */
export function containerStoreDir(
  {
    hostStoreDir,
    homeDir,
    realHomeDir,
  }: {
    readonly hostStoreDir: string;
    readonly homeDir: string;
    readonly realHomeDir: string;
  },
): string {
  /**
   {@link realHomeDir} as a directory prefix, so a sibling directory sharing a
   name prefix cannot match as if it were the home.
   */
  const realHomePrefix = realHomeDir.endsWith('/',)
    ? realHomeDir
    : `${realHomeDir}/`;
  if (!hostStoreDir.startsWith(realHomePrefix,))
    return hostStoreDir;
  return join(
    homeDir,
    hostStoreDir.slice(realHomePrefix.length,),
  );
}

//endregion Store mount
