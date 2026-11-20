import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { testBaseTag, browserslistDataLockPath, } from './native-test-base.ts';

/**
 npm packages the repository's browserslist data lock pins.
 */
const pinnedDataPackages = [
  'caniuse-lite',
  'node-releases',
] as const;

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
      name: 'native-test-base.Containerfile reads the pinned browserslist data through a build argument, typing none of its values',
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
        expect(text.includes('ARG BROWSERSLIST_DATA_LOCK')).toBe(true);
        for (const name of pinnedDataPackages) {
          /**
           The pinned entry of one npm package.
           */
          const pinned = lock[name] as {
            readonly version: string;
            readonly integrity: string;
          };
          expect(text.includes(pinned.version)).toBe(false);
          expect(text.includes(pinned.integrity)).toBe(false);
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
