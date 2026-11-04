import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { testBaseTag, } from './native-test-base.ts';

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
      fn: () => {
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
      fn: () => {
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
      fn: () => {
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
      fn: () => {
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
          expect(instructions[1]?.startsWith('FROM ${CACHYOS_IMAGE}')).toBe(true);
          expect(text.includes('@sha256:')).toBe(false);
        },
      });
    }),
  ],
});
