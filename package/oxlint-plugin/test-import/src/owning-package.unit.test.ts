import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { resolve, } from 'node:path';

import {
  owningPackage,
  PACKAGE_UNRESOLVED,
} from '../dist/final/node/index.mjs';

/** Fixture package holding one nested pseudo-package per manifest shape. */
const FIXTURE_ROOT = resolve(
  import.meta.dirname,
  '../../../test-fixture/oxlint-test-import/case',
);

/**
 Resolves the owning package of a file inside one fixture case.
 
 @param caseName - nested pseudo-package directory name
 
 @param fileName - file inside that case's `src`
 
 @returns owning package facts, or the unresolved sentinel
 
 @example
 ```ts
 ownerOfCase({ caseName: 'standard', fileName: 'allowed.test.ts' });
 ```
 */
function ownerOfCase({
  caseName,
  fileName,
}: {
  /**
   Nested pseudo-package directory name.
   */
  readonly caseName: string;
  /**
   File inside that case's `src`.
   */
  readonly fileName: string;
},): ReturnType<typeof owningPackage> {
  return owningPackage({
    fileName: resolve(
      FIXTURE_ROOT,
      caseName,
      'src',
      fileName,
    ),
  },);
}

await describe({
  name: owningPackage.name,
  children: [
    it({
      name: 'finds the nearest named manifest rather than an outer one',
      fn: async () => {
        /** Owner of a file inside the standard fixture case. */
        const owner = ownerOfCase({
          caseName: 'standard',
          fileName: 'allowed.test.ts',
        },);
        expect(owner === PACKAGE_UNRESOLVED ? '' : owner.name,)
          .toBe('@monochromatic-dev/test-fixture-case-standard',);
      },
    },),
    it({
      name: 'reports a build task when the package declares one',
      fn: async () => {
        /** Owner of a file inside the standard fixture case. */
        const owner = ownerOfCase({
          caseName: 'standard',
          fileName: 'allowed.test.ts',
        },);
        expect(owner === PACKAGE_UNRESOLVED ? false : owner.buildsArtifact,).toBe(true,);
      },
    },),
    it({
      name: 'derives artifact directories from the manifest',
      fn: async () => {
        /** Owner of a file inside the electron fixture case. */
        const owner = ownerOfCase({
          caseName: 'electron',
          fileName: 'entry-directory.test.ts',
        },);
        expect(owner === PACKAGE_UNRESOLVED ? [] : owner.artifactDirectories.toSorted(),)
          .toEqual([
            `${FIXTURE_ROOT}/electron/dist/app`,
            `${FIXTURE_ROOT}/electron/dist/final`,
          ],);
      },
    },),
    it({
      name: 'reports no build task for a package declaring none',
      fn: async () => {
        /** Owner of a file inside the buildless fixture case. */
        const owner = ownerOfCase({
          caseName: 'buildless',
          fileName: 'exempt.test.ts',
        },);
        expect(owner === PACKAGE_UNRESOLVED ? true : owner.buildsArtifact,).toBe(false,);
      },
    },),
    it({
      name: 'returns the sentinel when no ancestor holds a named manifest',
      fn: async () => {
        expect(owningPackage({ fileName: '/nonexistent-tree-for-owning-package/src/a.test.ts', },),)
          .toBe(PACKAGE_UNRESOLVED,);
      },
    },),
    it({
      name: 'memoizes, so a repeated lookup returns the identical record',
      fn: async () => {
        /** First lookup, which populates the directory cache. */
        const first = ownerOfCase({
          caseName: 'standard',
          fileName: 'allowed.test.ts',
        },);
        /** Second lookup of a sibling file, served from the same cached record. */
        const second = ownerOfCase({
          caseName: 'standard',
          fileName: 'rejected.test.ts',
        },);
        expect(first,).toBe(second,);
      },
    },),
  ],
},);
