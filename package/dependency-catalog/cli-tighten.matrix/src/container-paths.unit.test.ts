/**
 Unit tests for the matrix path constants.
 
 Pure host tests (no containers): prove every in-container entrypoint constant
 points at a file that exists in the monorepo the container mounts at
 {@link REPO_DIR}, so a package rename cannot leave the container argv naming a
 path that no longer exists. Both entrypoints used to be spelled out per file,
 and one kept the pre-rename `packages/dev-script/...` level, which made every
 scenario die before catalog-tighten ran.
 
 @module
 */

import {
  access,
} from 'node:fs/promises';
import {
  join,
} from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  HOST_REPO_ROOT,
  IN_CONTAINER_ENTRY,
  REPO_DIR,
  TOOL_ENTRY,
} from './container-paths.ts';

//region Host mapping

/**
 Prefix every path inside the container's monorepo mount starts with.
 */
const MOUNT_PREFIX = `${REPO_DIR}/`;

/**
 Translates an absolute path inside the container's `/repo` mount into the
 matching path on the host, so a test can check that an in-container constant
 names something that exists.
 
 @param inContainerPath - absolute path under {@link REPO_DIR}
 
 @returns the same path under {@link HOST_REPO_ROOT}
 
 @throws Error when the path is not inside the `/repo` mount
 
 @example
 ```ts
 hostPathOf('/repo/package/x/src/index.ts') // '<repo>/package/x/src/index.ts'
 ```
 */
function hostPathOf(inContainerPath: string,): string {
  if (inContainerPath === REPO_DIR)
    return HOST_REPO_ROOT;
  if (!inContainerPath.startsWith(MOUNT_PREFIX,)) {
    throw new Error(
      `not a path inside the container mount ${REPO_DIR}: ${inContainerPath}`,
    );
  }
  return join(
    HOST_REPO_ROOT,
    inContainerPath.slice(MOUNT_PREFIX.length,),
  );
}

//endregion Host mapping

await describe({
  name: 'container paths',
  children: [
    //region Entrypoints named by the container argv

    ...[
      ['TOOL_ENTRY', TOOL_ENTRY,],
      ['IN_CONTAINER_ENTRY', IN_CONTAINER_ENTRY,],
    ].map(function entryCase([
      name,
      inContainerPath,
    ],) {
      return it({
        name: `${name} names a file the /repo mount provides`,
        fn: async () => {
          /**
           Host path the container resolves `name` to through the mount.
           */
          const hostPath = hostPathOf(inContainerPath,);
          /**
           Rejection from `access`, or undefined when the file is there.
           */
          let missing: unknown;
          try {
            await access(hostPath,);
          }
          catch (error) {
            missing = error;
          }
          expect(
            missing,
          ).toBeUndefined();
        },
      },);
    },),

    //endregion Entrypoints named by the container argv

    //region Mount mapping

    it({
      name: 'hostPathOf maps the monorepo root to the host repo root',
      fn: async () => {
        expect(hostPathOf(REPO_DIR,),).toBe(HOST_REPO_ROOT,);
      },
    },),

    it({
      name: 'hostPathOf rejects a path outside the /repo mount',
      fn: async () => {
        /**
         Rejection from `hostPathOf` for a path the mount does not provide.
         */
        let caught: unknown;
        try {
          hostPathOf('/work/package.json',);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(Error,);
        expect((caught as Error).message,).toContain('not a path inside the container mount',);
      },
    },),

    //endregion Mount mapping
  ],
},);
