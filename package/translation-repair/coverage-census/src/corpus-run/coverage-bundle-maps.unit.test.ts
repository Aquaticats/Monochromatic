/**
 Tests for how the coverage census splits a build's bundles by their source
 maps (ledger T8). A bundle of import and export statements alone gets no
 map from rolldown, as the package's `index.mjs` has since 2026-09-29, and
 the census refused the whole build over it; a build with no map at all is
 the normal build, which the census still refuses, and so is a minified
 build, whose folded guards would read as run (ledger M79). Names are
 cat-themed invention.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import {
  bundleMapsOf,
  requireMapFor,
  requireUnminifiedBuild,
} from '../../dist/final/node/index.mjs';

/**
 Build directory the refusals name.
 */
const DIST = '/cats/dist/final/node';

/**
 How every refusal here tells the reader to run the census.
 */
const RUN_THE_TASK = 'run the census through mise run //package/translation-repair/coverage-census:coverage-census';

/**
 A chunk as the unminified coverage build writes it: each module's code
 opened by rolldown's region comment.
 */
const UNMINIFIED_CHUNK = [
  '//#region src/nap.ts',
  'function nap(cat) {',
  '\tif (cat === void 0) return;',
  '\treturn cat.sleep();',
  '}',
  '//#endregion',
  'export { nap as t };',
  '',
].join('\n',);

/**
 The same chunk as the compressed normal build writes it.
 */
const MINIFIED_CHUNK = 'function nap(cat){if(cat!==void 0)return cat.sleep()}export{nap as t};\n';

/**
 A chunk of import and export statements alone, which carries no region
 comment in either build.
 */
const RE_EXPORT_CHUNK = 'import { t as nap } from "./nap-Cat1.mjs";\nexport { nap };\n';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: bundleMapsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SPLITS the bundles by the map beside each, sorted, and reads nothing but .mjs files as bundles',
          fn: async () => {
            expect(bundleMapsOf({
              built: [
                'purr.mjs',
                'purr.mjs.map',
                'index.mjs',
                'index.d.mts',
                'nap.mjs.map',
                'nap.mjs',
                'litter.mjs.map',
                'notes.txt',
              ],
              distDirectory: DIST,
            },),).toEqual({
              mapped: ['nap.mjs', 'purr.mjs',],
              unmapped: ['index.mjs',],
            },);
          },
        },),
        it({
          name: 'REFUSES a build with no map beside any bundle, naming the directory and the bundle count: the '
            + 'normal build',
          fn: async () => {
            /**
             What the split threw on a build without maps.
             */
            const refusal = caught(function normalBuild(): void {
              bundleMapsOf({
                built: ['nap.mjs', 'index.mjs', 'index.d.mts',],
                distDirectory: DIST,
              },);
            },);
            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(refusal,).toHaveProperty(
              'message',
              `${DIST} holds 2 bundles and no source map beside any of them, so it is the normal build; `
                + `${RUN_THE_TASK}, which builds with maps first`,
            );
          },
        },),
        it({
          name: 'REFUSES a build that holds no bundle at all, as a directory no build wrote rather than the normal '
            + 'build',
          fn: async () => {
            /**
             What the split threw on an empty build.
             */
            const refusal = caught(function emptyBuild(): void {
              bundleMapsOf({
                built: ['index.d.mts',],
                distDirectory: DIST,
              },);
            },);
            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(refusal,).toHaveProperty(
              'message',
              `${DIST} holds no bundle, so no build has written it; ${RUN_THE_TASK}, which builds with maps first`,
            );
          },
        },),
      ],
    },),

    describe({
      name: requireMapFor.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'PASSES a bundle with a map, for either need',
          fn: async () => {
            expect(requireMapFor({
              bundle: 'nap.mjs',
              need: 'cold-code',
              unmapped: ['index.mjs',],
              distDirectory: DIST,
            },),).toBeUndefined();
            expect(requireMapFor({
              bundle: 'nap.mjs',
              need: 'unloaded-sources',
              unmapped: ['index.mjs',],
              distDirectory: DIST,
            },),).toBeUndefined();
          },
        },),
        it({
          name: 'REFUSES code no test ran in a bundle with no map, naming the bundle and what cannot be placed',
          fn: async () => {
            /**
             What the check threw for cold code in an unmapped bundle.
             */
            const refusal = caught(function coldUnmapped(): void {
              requireMapFor({
                bundle: 'index.mjs',
                need: 'cold-code',
                unmapped: ['index.mjs',],
                distDirectory: DIST,
              },);
            },);
            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(refusal,).toHaveProperty(
              'message',
              `coverage finds code no test ran in index.mjs, which ${DIST} holds with no source map beside it, so `
                + 'the census cannot place that code on a source line; the coverage build writes a map for every '
                + `bundle that holds code, so this one came from another build: ${RUN_THE_TASK}, which builds with `
                + 'maps first',
            );
          },
        },),
        it({
          name: 'REFUSES a bundle no test loaded with no map, naming the bundle and the sources it cannot name',
          fn: async () => {
            /**
             What the check threw for an unloaded unmapped bundle.
             */
            const refusal = caught(function unloadedUnmapped(): void {
              requireMapFor({
                bundle: 'index.mjs',
                need: 'unloaded-sources',
                unmapped: ['index.mjs',],
                distDirectory: DIST,
              },);
            },);
            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(refusal,).toHaveProperty(
              'message',
              `no test loaded index.mjs, which ${DIST} holds with no source map beside it, so the census cannot say `
                + 'which sources it carries; the coverage build writes a map for every bundle that holds code, so '
                + `this one came from another build: ${RUN_THE_TASK}, which builds with maps first`,
            );
          },
        },),
      ],
    },),

    describe({
      name: requireUnminifiedBuild.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'PASSES a build whose chunks open module code with rolldown\'s region comment, beside a re-export '
            + 'chunk that carries none in any build',
          fn: async () => {
            expect(requireUnminifiedBuild({
              texts: [RE_EXPORT_CHUNK, UNMINIFIED_CHUNK,],
              distDirectory: DIST,
            },),).toBeUndefined();
          },
        },),
        it({
          name: 'REFUSES a minified build before the suite runs, naming the directory, the bundle count and both '
            + 'ways back, and reads a region comment only where it opens a line (ledger M79)',
          fn: async () => {
            /**
             What the check threw for a build of compressed chunks.
             */
            const minified = caught(function compressed(): void {
              requireUnminifiedBuild({
                texts: [
                  RE_EXPORT_CHUNK,
                  MINIFIED_CHUNK,
                  `${MINIFIED_CHUNK.trimEnd()} //#region src/nap.ts\n`,
                ],
                distDirectory: DIST,
              },);
            },);
            expect(minified,).toBeInstanceOf(StatedRefusalError,);
            expect(minified,).toHaveProperty(
              'message',
              `${DIST} holds 3 bundles with a source map and no module region comment in any of them, so the build `
                + 'was minified, and minification folds guards into expressions the coverage gives no range, which '
                + `would read as run (ledger M79): ${RUN_THE_TASK}, whose build keeps the code as written, or `
                + 'restore minify: false in rolldown.coverage.config.ts if it was changed',
            );
          },
        },),
      ],
    },),
  ],
},);
