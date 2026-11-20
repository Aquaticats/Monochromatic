import { createHash, } from 'node:crypto';
import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { testBaseTag, browserslistDataLockPath, } from './native-test-base.ts';

/**
 Lock keys that hold something other than a pinned npm package: the format version and the Firefox ESR value, which
 `DataLock::from_json` in `package/browserslist/core-rust/src/lock.rs` reads beside the packages.
 */
const lockKeysThatHoldNoPackage: ReadonlySet<string> = new Set([
  'lockVersion',
  'firefoxEsr',
],);

/**
 Digits of a hash kept in an image tag, which these tests compute independently of the module they check.
 */
const tagHashDigits = 12;

/**
 Names of the npm packages a lock lists, derived from the lock so an added or renamed package is checked without an edit
 here.

 @param lock - parsed text of the repository's browserslist data lock

 @returns every key that is not one of the lock's own non-package keys
 */
function pinnedPackageNames(lock: Readonly<Record<string, unknown>>,): readonly string[] {
  return Object.keys(lock,)
    .filter(function holdsPackage(name) {
      return !lockKeysThatHoldNoPackage.has(name,);
    },);
}

/**
 Image ID of a CachyOS image, as `podman image inspect` prints it.
 */
const imageId = '4872c09919c69a45df3b604ebb2792bb6c5e67ba9004e948bbf40626b1235d78';

/**
 Repository the tags of these tests belong to.
 */
const repository = 'localhost/example';

/**
 Containerfiles of the two derived bases, relative to the package root.
 */
const derivedContainerfiles = [
  'native-test-base.Containerfile',
  join(
    'e2e',
    'concurrent-commits.Containerfile',
  ),
] as const;

await describe({
  name: testBaseTag.name,
  children: [
    it({
      name: 'names the same inputs by the same tag, with or without the digest prefix',
      fn: async () => {
        expect(testBaseTag({
          repository,
          cachyosImageId: `sha256:${imageId}`,
          containerfileText: 'FROM x\n',
        })).toBe(testBaseTag({
          repository,
          cachyosImageId: imageId,
          containerfileText: 'FROM x\n',
        }));
      },
    }),
    it({
      name: 'starts with the repository and the first digits of the image ID',
      fn: async () => {
        expect(testBaseTag({
          repository,
          cachyosImageId: imageId,
          containerfileText: 'FROM x\n',
        })
          .startsWith(`${repository}:${imageId.slice(
            0,
            12,
          )}-`)).toBe(true);
      },
    }),
    it({
      name: 'changes when the pulled CachyOS image changes',
      fn: async () => {
        expect(testBaseTag({
          repository,
          cachyosImageId: imageId,
          containerfileText: 'FROM x\n',
        })).not.toBe(testBaseTag({
          repository,
          cachyosImageId: `0${imageId.slice(1)}`,
          containerfileText: 'FROM x\n',
        }));
      },
    }),
    it({
      name: 'changes when the Containerfile text changes',
      fn: async () => {
        expect(testBaseTag({
          repository,
          cachyosImageId: imageId,
          containerfileText: 'FROM x\n',
        })).not.toBe(testBaseTag({
          repository,
          cachyosImageId: imageId,
          containerfileText: 'FROM y\n',
        }));
      },
    }),
    it({
      name: 'keeps the tag of a build that takes no arguments the one its Containerfile text alone produces',
      fn: async () => {
        /**
         Text of a Containerfile the end-to-end base is built from.
         */
        const containerfileText = 'FROM x\n';
        /**
         Full SHA-256 hex digest of the Containerfile text, computed here instead of through the module, so a changed
         composition of the hashed text shows up as a failure.
         */
        const containerfileDigest = createHash('sha256',)
          .update(containerfileText,)
          .digest('hex',);
        /**
         The Containerfile's own hash, as the module's tags keep it.
         */
        const containerfileOnlyHash = containerfileDigest.slice(0, tagHashDigits);
        /**
         Tag of a build that is given no arguments: the end-to-end base's tag, which must not move for a Containerfile
         and image that did not change.
         */
        const expected = `${repository}:${imageId.slice(0, tagHashDigits)}-${containerfileOnlyHash}`;
        expect(testBaseTag({
          repository,
          cachyosImageId: imageId,
          containerfileText,
        },)).toBe(expected,);
        expect(testBaseTag({
          repository,
          cachyosImageId: imageId,
          containerfileText,
          buildArgumentsText: '',
        },)).toBe(expected,);
      },
    }),
    it({
      name: 'changes when the build arguments change',
      fn: async () => {
        expect(testBaseTag({
          repository,
          cachyosImageId: imageId,
          containerfileText: 'FROM x\n',
          buildArgumentsText: 'A=1',
        })).not.toBe(testBaseTag({
          repository,
          cachyosImageId: imageId,
          containerfileText: 'FROM x\n',
          buildArgumentsText: 'A=2',
        }));
      },
    }),
    it({
      name: 'native-test-base.Containerfile reads the pinned browserslist data from the lock through a build argument, typing none of its values',
      fn: async () => {
        /**
         Text of the gate base's Containerfile.
         */
        const text = await readFile(
          join(
            import.meta.dirname,
            '..',
            'native-test-base.Containerfile',
          ),
          'utf8',
        );
        /**
         The lock the build script passes to it, which stays the one source of the versions and digests.
         */
        const lock = JSON.parse(await readFile(
          browserslistDataLockPath,
          'utf8',
        )) as Record<string, unknown>;
        /**
         The lock's packages, which the Containerfile takes from the lock rather than naming itself.
         */
        const pinnedDataPackages = pinnedPackageNames(lock,);
        expect(text.includes('ARG BROWSERSLIST_DATA_LOCK',)).toBe(true,);
        expect(pinnedDataPackages.length,).toBeGreaterThan(0,);
        for (const name of pinnedDataPackages) {
          /**
           The pinned entry of one npm package.
           */
          const pinned = lock[name] as {
            readonly version: string;
            readonly integrity: string;
          };
          expect(text.includes(name,),).toBe(false,);
          expect(text.includes(pinned.version,),).toBe(false,);
          expect(text.includes(pinned.integrity,),).toBe(false,);
        }
      },
    }),
    ...derivedContainerfiles.map(function baseRuleTest(path) {
      return it({
        name: `${path} takes its base from the CACHYOS_IMAGE argument on the latest tag, with no version or digest`,
        fn: async () => {
          /**
           Text of the Containerfile.
           */
          const text = await readFile(
            join(
              import.meta.dirname,
              '..',
              path,
            ),
            'utf8',
          );
          /**
           Instructions that start a stage or declare the base argument.
           */
          const instructions = text.split('\n')
            .filter(function startsStage(line) {
              return line.startsWith('FROM ') || line.startsWith('ARG CACHYOS_IMAGE');
            });
          expect(instructions[0]).toBe('ARG CACHYOS_IMAGE=docker.io/cachyos/cachyos:latest');
          expect(instructions[1]?.startsWith(`FROM \${CACHYOS_IMAGE}`)).toBe(true);
          expect(text.includes('@sha256:')).toBe(false);
        },
      });
    }),
  ],
});
