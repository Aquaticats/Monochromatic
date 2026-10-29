/**
 Tests for the census's checks of its build directory, which cost no suite
 run: a directory that cannot be listed, a build with no bundle, no map or a
 minified one, and the progress line a build it can read earns. Each case
 builds a disposable package directory. Names are cat-themed invention.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import {
  readUtf8Text,
  requireCoverageBuild,
} from '../../dist/final/node/index.mjs';
import { capturingLoggerPair, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { refusalOrder, } from '@monochromatic-dev/translation-repair-test-fixture/ts/refusal-order.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/rejection-of.test-fixture.ts';
import {
  makeBuiltPackage,
  MINIFIED_BUNDLE,
  UNMINIFIED_BUNDLE,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/coverage-census-package.test-fixture.ts';

/**
 The task that builds with maps, which every refusal sends the operator to.
 */
const TASK = 'mise run //package/translation-repair/coverage-census:coverage-census';

/**
 What checking a build directory refuses with.

 @param distDirectory - build directory to check

 @returns Thrown value

 @throws Error where the check refused nothing

 @example
 ```ts
 const refusal = await refusalOfBuild({ distDirectory, },);
 ```
 */
async function refusalOfBuild({ distDirectory, }: { readonly distDirectory: string; },): Promise<unknown> {
  try {
    await requireCoverageBuild({
      distDirectory,
      l: capturingLoggerPair().logger,
      readText: readUtf8Text,
    },);
  }
  catch (error) {
    return error;
  }
  throw new Error('the check refused no build',);
}

await describe({
  name: requireCoverageBuild.name,
  children: [
    it({
      name: 'REFUSES A BUILD DIRECTORY THAT IS NOT THERE as stated, naming the filesystem code and the way out, '
        + 'rather than faulting on the listing',
      fn: async () => {
        await using built = await makeBuiltPackage({ files: {}, },);

        /**
         A build directory no build ever wrote.
         */
        const distDirectory = join(
          built.path,
          'no-such-dist',
        );
        const refusal = await refusalOfBuild({ distDirectory, },);
        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          `StatedRefusalError: ${distDirectory} could not be listed (ENOENT), so no build has written it there; `
            + `run the census from the package directory through ${TASK}, which builds with maps first`,
        );
      },
    },),
    it({
      name: 'REFUSES A BUILD DIRECTORY THAT IS A FILE as stated, naming ENOTDIR',
      fn: async () => {
        await using built = await makeBuiltPackage({ files: {}, },);

        /**
         A path that holds a file where the build directory belongs.
         */
        const distDirectory = join(
          built.path,
          'a-file',
        );
        await writeFile(
          distDirectory,
          'purr\n',
          'utf8',
        );
        const refusal = await refusalOfBuild({ distDirectory, },);
        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          `StatedRefusalError: ${distDirectory} could not be listed (ENOTDIR), so no build has written it there; `
            + `run the census from the package directory through ${TASK}, which builds with maps first`,
        );
      },
    },),
    it({
      name: 'REFUSES A BUILD DIRECTORY HOLDING NO BUNDLE as stated, naming it and the task that builds with maps',
      fn: async () => {
        await using built = await makeBuiltPackage({ files: {}, },);
        const refusal = await refusalOfBuild({ distDirectory: built.distDirectory, },);
        expect(String(refusal,),).toBe(
          `StatedRefusalError: ${built.distDirectory} holds no bundle, so no build has written it; run the `
            + `census through ${TASK}, which builds with maps first`,
        );
      },
    },),
    it({
      name: 'REFUSES A BUILD WITH NO SOURCE MAP BESIDE ONE BUNDLE as the normal build, in the singular',
      fn: async () => {
        await using built = await makeBuiltPackage({ files: { 'nap.mjs': MINIFIED_BUNDLE, }, },);
        const refusal = await refusalOfBuild({ distDirectory: built.distDirectory, },);
        expect(String(refusal,),).toBe(
          `StatedRefusalError: ${built.distDirectory} holds 1 bundle and no source map beside any of them, so it `
            + `is the normal build; run the census through ${TASK}, which builds with maps first`,
        );
      },
    },),
    it({
      name: 'REFUSES A BUILD WITH NO SOURCE MAP BESIDE TWO BUNDLES as the normal build, in the plural',
      fn: async () => {
        await using built = await makeBuiltPackage({
          files: {
            'nap.mjs': MINIFIED_BUNDLE,
            'purr.mjs': MINIFIED_BUNDLE,
          },
        },);
        const refusal = await refusalOfBuild({ distDirectory: built.distDirectory, },);
        expect(String(refusal,),).toBe(
          `StatedRefusalError: ${built.distDirectory} holds 2 bundles and no source map beside any of them, so it `
            + `is the normal build; run the census through ${TASK}, which builds with maps first`,
        );
      },
    },),
    it({
      name: 'REFUSES A MINIFIED BUILD with one mapped bundle as stated, naming the ledger entry, in the singular',
      fn: async () => {
        await using built = await makeBuiltPackage({
          files: {
            'nap.mjs': MINIFIED_BUNDLE,
            'nap.mjs.map': '{}',
          },
        },);
        const refusal = await refusalOfBuild({ distDirectory: built.distDirectory, },);
        expect(String(refusal,),).toBe(
          `StatedRefusalError: ${built.distDirectory} holds 1 bundle with a source map and no module region `
            + 'comment in any of them, so the build was minified, and minification folds guards into expressions '
            + 'the coverage gives no range, which would read as run (ledger M79): run the census through '
            + `${TASK}, whose build keeps the code as written, or restore minify: false in `
            + 'rolldown.coverage.config.ts if it was changed',
        );
      },
    },),
    it({
      name: 'REFUSES A MINIFIED BUILD with two mapped bundles in the plural, counting only the mapped ones',
      fn: async () => {
        await using built = await makeBuiltPackage({
          files: {
            'nap.mjs': MINIFIED_BUNDLE,
            'nap.mjs.map': '{}',
            'purr.mjs': MINIFIED_BUNDLE,
            'purr.mjs.map': '{}',
            'loaf.mjs': MINIFIED_BUNDLE,
          },
        },);
        const refusal = await refusalOfBuild({ distDirectory: built.distDirectory, },);
        expect(String(refusal,),).toBe(
          `StatedRefusalError: ${built.distDirectory} holds 2 bundles with a source map and no module region `
            + 'comment in any of them, so the build was minified, and minification folds guards into expressions '
            + 'the coverage gives no range, which would read as run (ledger M79): run the census through '
            + `${TASK}, whose build keeps the code as written, or restore minify: false in `
            + 'rolldown.coverage.config.ts if it was changed',
        );
      },
    },),
    it({
      name: 'SPLITS THE BUILD\'S BUNDLES by whether a map stands beside each and logs that none lacks one',
      fn: async () => {
        await using built = await makeBuiltPackage({
          files: {
            'nap.mjs': UNMINIFIED_BUNDLE,
            'nap.mjs.map': '{}',
            'purr.mjs': UNMINIFIED_BUNDLE,
            'purr.mjs.map': '{}',
          },
        },);
        const { logger, lines, } = capturingLoggerPair();
        expect(await requireCoverageBuild({
          distDirectory: built.distDirectory,
          l: logger,
          readText: readUtf8Text,
        },),).toEqual({
          mapped: [
            'nap.mjs',
            'purr.mjs',
          ],
          unmapped: [],
        },);
        expect(lines,).toEqual([
          'bundles with no source map, read only where the census must place code in them: none',
        ],);
      },
    },),
    it({
      name: 'NAMES THE BUNDLES WITH NO MAP in its log line and still accepts the build, since one mapped bundle '
        + 'carries a module region comment',
      fn: async () => {
        await using built = await makeBuiltPackage({
          files: {
            'nap.mjs': UNMINIFIED_BUNDLE,
            'nap.mjs.map': '{}',
            'index.mjs': MINIFIED_BUNDLE,
            'loaf.mjs': MINIFIED_BUNDLE,
          },
        },);
        const { logger, lines, } = capturingLoggerPair();
        expect(await requireCoverageBuild({
          distDirectory: built.distDirectory,
          l: logger,
          readText: readUtf8Text,
        },),).toEqual({
          mapped: ['nap.mjs',],
          unmapped: [
            'index.mjs',
            'loaf.mjs',
          ],
        },);
        expect(lines,).toEqual([
          'bundles with no source map, read only where the census must place code in them: index.mjs, loaf.mjs',
        ],);
      },
    },),
    it({
      name: 'REFUSES with the first mapped bundle\'s read when two bundles cannot be read and the second '
        + 'bundle\'s read is refused first',
      fn: async () => {
        await using built = await makeBuiltPackage({
          files: {
            'nap.mjs': UNMINIFIED_BUNDLE,
            'nap.mjs.map': '{}',
            'purr.mjs': UNMINIFIED_BUNDLE,
            'purr.mjs.map': '{}',
          },
        },);
        /**
         The two refusals the bundle reads end in.
         */
        const { refuseAtOnce, refuseAfterThat, } = refusalOrder();
        /**
         What the check refused with.
         */
        const refusal = await rejectionOf({
          promise: requireCoverageBuild({
            distDirectory: built.distDirectory,
            l: capturingLoggerPair().logger,
            readText: async function refusesSecondFirst({ path, },): Promise<string> {
              return await (path.endsWith('nap.mjs',)
                ? refuseAfterThat(new Error('the nap bundle cannot be read',),)
                : refuseAtOnce(new Error('the purr bundle cannot be read',),));
            },
          },),
        },);
        expect(String(refusal,),).toBe('Error: the nap bundle cannot be read',);
      },
    },),
  ],
},);
