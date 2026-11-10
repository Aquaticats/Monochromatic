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
  mkdir,
  mkdtemp,
  readFile,
  rm,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { SubprocessError, } from 'nano-spawn';

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
            expect((caught as Error).cause,).toBeInstanceOf(SubprocessError,);
          },
        },),

        it({
          name: 'reports a conversion that could not start, which captures no stderr',
          timeout: 300_000,
          fn: async () => {
            /**
             Disposable directory standing in for the package's output directory.
             */
            const directory = await mkdtemp(join(
              scratchRoot(),
              'aquaticat-woff2-start-',
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
             Empty directory holding no `uv` executable.
             */
            const emptyBin = join(
              directory,
              'bin',
            );
            await mkdir(
              emptyBin,
              { recursive: true, },
            );
            /**
             PATH the conversion's `uv` lookup runs under.
             */
            const originalPath = process.env.PATH;
            process.env.PATH = emptyBin;
            await using pathGuard = {
              [Symbol.asyncDispose]: async function restorePath(): Promise<void> {
                if (originalPath === undefined) {
                  delete process.env.PATH;
                }
                else {
                  process.env.PATH = originalPath;
                }
              },
            };
            // The binding exists solely for its dispose hook.
            void pathGuard;
            /**
             Value the conversion rejected with, when it rejects.
             */
            let caught: unknown;

            try {
              await convertToWoff2({
                otfPath: join(
                  directory,
                  'Aquaticat-Regular.otf',
                ),
                distDir: directory,
              },);
            }
            catch (error) {
              caught = error;
            }

            expect(caught,).toBeInstanceOf(Error,);
            expect((caught as Error).message,).toContain('ENOENT',);
          },
        },),

        it({
          name: 'narrows the caught value instead of asserting its shape',
          fn: async () => {
            /**
             Source of the conversion module.
             */
            const source = await readFile(
              join(
                import.meta.dirname,
                'build-font-woff2.ts',
              ),
              'utf8',
            );

            expect(source,).not.toContain('no-unsafe-type-assertion',);
            expect(source,).not.toContain('as { stderr',);
            expect(source,).toContain('instanceof SubprocessError',);
          },
        },),
      ],
    },),
  ],
},);
