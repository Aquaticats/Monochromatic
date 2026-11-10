/**
 A WOFF2 conversion that fails fails the build.

 `convertToWoff2` caught every failure, printed only the caught value's `stderr`,
 and returned normally, so the build task exited 0 with no
 `dist/Aquaticat-Regular.woff2` on disk while the package's `./woff2` export
 still pointed at it. The conversion now throws, carrying the caught value as
 its cause so the build stops and the diagnostic survives.

 The case runs the real `uv` and fonttools, pointed at an OTF file that does not
 exist: the shortest failure that reaches the conversion step.
 */
/* oxlint-disable test-import/require-eventual-artifact -- this package ships src/ (package.json `files`) and builds font binaries rather than a JS artifact, so the modules beside this test are the shipped ones; see doc/planning/oxlint-test-import-eventual-artifact.md, section "Packages that ship source". */
import {
  mkdtemp,
  rm,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { convertToWoff2, } from './build-font-woff2.ts';

await describe({
  name: '',
  children: [
    describe({
      name: convertToWoff2.name,
      concurrency: 1,
      children: [
        it({
          name: 'rejects when the OTF file cannot be converted',
          timeout: 300_000,
          fn: async () => {
            /**
             Disposable directory standing in for the package's output directory.
             */
            const directory = await mkdtemp(join(
              scratchRoot(),
              'aquaticat-woff2-',
            ),);
            await using guard = {
              [Symbol.asyncDispose]: async function removeDirectory(): Promise<void> {
                await rm(
                  directory,
                  { recursive: true, force: true, },
                );
              },
            };
            // The binding exists solely for its dispose hook.
            void guard;
            /**
             OTF path no converter can read.
             */
            const otfPath = join(
              directory,
              'Aquaticat-Regular.otf',
            );
            /**
             Value the conversion rejected with, when it rejects.
             */
            let caught: unknown;

            try {
              await convertToWoff2({
                otfPath,
                distDir: directory,
              },);
            }
            catch (error) {
              caught = error;
            }

            expect(caught,).toBeInstanceOf(Error,);
            expect((caught as Error).message,).toContain(otfPath,);
            expect((caught as Error).cause,).toBeDefined();
          },
        },),
      ],
    },),
  ],
},);
