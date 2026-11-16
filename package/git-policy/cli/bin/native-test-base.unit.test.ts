import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  buildArgumentsText,
  testBaseTag,
} from './native-test-base.ts';
import {
  LOCK_FILE_NAME,
  RUST_TOOLCHAIN_ROW,
  isInstalableToolchain,
  lockedRustToolchain,
  rustToolchainOfLock,
} from './locked-rust-toolchain.ts';

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
    it({
      name: 'changes when the build argument a Containerfile reads changes, so a moved toolchain builds a new base',
      fn: async () => {
        expect(testBaseTag({
          repository,
          cachyosImageId: imageId,
          containerfileText: 'FROM x\n',
          buildArguments: { RUST_TOOLCHAIN: 'nightly-2026-11-15', },
        })).not.toBe(testBaseTag({
          repository,
          cachyosImageId: imageId,
          containerfileText: 'FROM x\n',
          buildArguments: { RUST_TOOLCHAIN: 'nightly-2026-11-16', },
        }));
      },
    }),
    it({
      name: 'names no argument the base image does not pass, so a base that takes none keeps the tag it had',
      fn: async () => {
        expect(testBaseTag({
          repository,
          cachyosImageId: imageId,
          containerfileText: 'FROM x\n',
        })).toBe(testBaseTag({
          repository,
          cachyosImageId: imageId,
          containerfileText: 'FROM x\n',
          buildArguments: {},
        }));
      },
    }),
    it({
      name: 'writes the build arguments in name order, so the order a caller passes them cannot change a tag',
      fn: async () => {
        expect(buildArgumentsText({
          RUST_TOOLCHAIN: 'nightly-2026-11-15',
          ANOTHER_TOOL: '1.0.0',
        },)).toBe('ANOTHER_TOOL=1.0.0\nRUST_TOOLCHAIN=nightly-2026-11-15\n');
      },
    }),
    it({
      name: 'names the gate base image after the Rust toolchain the repository lock records',
      fn: async () => {
        /**
         The toolchain the checkout's lock names.
         */
        const toolchain = await lockedRustToolchain();
        expect(isInstalableToolchain(toolchain,),).toBe(true,);
        /**
         The gate's Containerfile, which the base is built from.
         */
        const text = await readFile(
          join(
            import.meta.dirname,
            '..',
            'native-test-base.Containerfile',
          ),
          'utf8',
        );
        expect(text.includes('toolchain install "$RUST_TOOLCHAIN"',),).toBe(true,);
        // A date written into the Containerfile would be a second place to move; the build argument is the only one.
        expect(text.includes('nightly-',),).toBe(false,);
      },
    }),
    it({
      name: 'reads the version out of the lock row the workflows read, and refuses a lock that holds no usable one',
      fn: async () => {
        expect(rustToolchainOfLock({ lock: { tools: { rust: { version: 'nightly-2026-11-15', }, }, }, },),)
          .toBe('nightly-2026-11-15',);
        /**
         Locks the reading must refuse, one per shape it cannot install.
         */
        const refused: readonly unknown[] = [
          'not an object',
          [],
          null,
          {},
          { tools: { rust: {}, }, },
          { tools: { rust: { version: 'nightly', }, }, },
          { tools: { rust: { version: 'nightly-2026-11-15-x86_64-unknown-linux-gnu', }, }, },
        ];
        for (const lock of refused) {
          let message = '';
          try {
            rustToolchainOfLock({ lock, },);
          }
          catch (error) {
            expect(error,).toBeInstanceOf(Error,);
            message = (error as Error).message;
          }
          expect(message,).toContain(LOCK_FILE_NAME,);
          expect(message,).toContain(RUST_TOOLCHAIN_ROW,);
        }
      },
    }),
  ],
});
