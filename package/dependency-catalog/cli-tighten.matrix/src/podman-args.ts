/**
 The `podman run` argv for one matrix scenario.
 
 The container gets two read-only mounts and two writable tmpfs mounts, and
 nothing else. The monorepo is mounted at {@link REPO_DIR} so the tool and its
 imports resolve inside it; the store its `node_modules` entries link into is
 mounted at the path those links spell, because a relative link out of the
 repository reaches nothing the container is not given. The fixture is written
 and installed in the tmpfs at {@link WORK_DIR}, and the rootfs stays read-only.
 */

import {
  FIXTURE_PNPM_ENV,
  type Scenario,
} from './combos.ts';
import {
  HOST_REPO_ROOT,
  IN_CONTAINER_ENTRY,
  REPO_DIR,
  WORK_DIR,
} from './container-paths.ts';

//region Container configuration

/**
 Base image: node with corepack and unflagged TypeScript stripping; pnpm is provisioned at runtime.
 */
export const NODE_IMAGE = 'docker.io/library/node:24-slim';

/**
 Per-container memory cap.
 */
const MEMORY_LIMIT = '2g';

/**
 Per-container CPU cap.
 */
const CPU_LIMIT = '2';

/**
 Per-container process cap.
 */
const PIDS_LIMIT = 512;

/**
 Writable tmpfs size for the fixture work tree.
 */
const WORK_TMPFS_SIZE = '512m';

/**
 Writable tmpfs size for `/tmp` (corepack cache, pnpm store, downloaded binaries).
 */
const TMP_TMPFS_SIZE = '1g';

/**
 Container home directory: the tmpfs corepack caches pnpm under, since the
 rootfs is read-only and the image's own home is not writable.
 */
const CONTAINER_HOME = '/tmp';

//endregion Container configuration

//region Argv

/**
 Builds the `podman run` argv for one combination, passing the resolved
 monorepo pnpm spec into the container through {@link FIXTURE_PNPM_ENV} and
 mounting the store the workspace's `node_modules` entries link into.
 
 @param scenario - scenario to run; serialised as the entrypoint argument
 
 @param pnpmSpec - monorepo `pnpm@<version>` spec the fixture installs with
 
 @param storeMount - store directory on the host and the path inside the container its links name
 
 @returns argument vector excluding the `podman` executable
 
 @example
 ```ts
 buildPodmanArgs({ scenario: SCENARIOS[0], pnpmSpec: 'pnpm\@12.18.0', storeMount: { hostStoreDir: '/srv/store/v11', containerStoreDir: '/srv/store/v11' } }).at(0) // 'run'
 ```
 */
export function buildPodmanArgs(
  {
    scenario,
    pnpmSpec,
    storeMount,
  }: {
    readonly scenario: Scenario;
    readonly pnpmSpec: string;
    readonly storeMount: {
      /**
       Store directory on the host, from `pnpm store path`.
       */
      readonly hostStoreDir: string;
      /**
       Path inside the container the workspace's links resolve to.
       */
      readonly containerStoreDir: string;
    };
  },
): readonly string[] {
  return [
    'run',
    '--rm',
    '--memory',
    MEMORY_LIMIT,
    '--cpus',
    CPU_LIMIT,
    '--pids-limit',
    String(PIDS_LIMIT,),
    '--read-only',
    '--security-opt=label=disable',
    '--tmpfs',
    `${WORK_DIR}:rw,exec,size=${WORK_TMPFS_SIZE}`,
    '--tmpfs',
    `${CONTAINER_HOME}:rw,exec,size=${TMP_TMPFS_SIZE}`,
    '--env',
    `HOME=${CONTAINER_HOME}`,
    '--env',
    `${FIXTURE_PNPM_ENV}=${pnpmSpec}`,
    '--volume',
    `${HOST_REPO_ROOT}:${REPO_DIR}:ro`,
    '--volume',
    `${storeMount.hostStoreDir}:${storeMount.containerStoreDir}:ro`,
    '--workdir',
    WORK_DIR,
    NODE_IMAGE,
    'node',
    IN_CONTAINER_ENTRY,
    JSON.stringify(scenario,),
  ];
}

//endregion Argv
