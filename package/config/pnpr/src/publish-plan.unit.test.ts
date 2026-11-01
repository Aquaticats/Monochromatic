import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isTypeScriptSourcePath,
  orderForPublishing,
  PnprConfigShapeError,
  prepareManifestForPnpr,
  readPnprPublishTarget,
} from '../dist/final/node/index.mjs';

/**
 Minimal config mirroring the generator's OIDC shape.
 */
const VALID_CONFIG = `
auth:
  oidc:
    - name: github
      audience: https://pnpr.example
      workloads:
        - identity:
            subject: 'repo:owner/repo:ref:refs/heads/main'
          registry: hosted-name
          packages:
            - '@scope/a'
            - '@scope/b'
`;

await describe({
  name: '',
  children: [
    describe({
      name: readPnprPublishTarget.name,
      children: [
        it({
          name: 'reads origin, registry name, and package names',
          fn: async () => {
            expect(readPnprPublishTarget(VALID_CONFIG,),).toEqual({
              origin: 'https://pnpr.example',
              registryName: 'hosted-name',
              packageNames: [
                '@scope/a',
                '@scope/b',
              ],
            },);
          },
        },),
        ...[
          {
            label: 'a non-object document',
            text: 'just text',
            field: 'auth.oidc[0].audience',
          },
          {
            label: 'a missing audience',
            text: VALID_CONFIG.replace('      audience: https://pnpr.example\n', '',),
            field: 'auth.oidc[0].audience',
          },
          {
            label: 'a missing registry',
            text: VALID_CONFIG.replace('          registry: hosted-name\n', '',),
            field: 'auth.oidc[0].workloads[0].registry',
          },
          {
            label: 'non-string package names',
            text: VALID_CONFIG.replace("            - '@scope/b'", '            - 7',),
            field: 'auth.oidc[0].workloads[0].packages',
          },
        ].map(function invalidConfigCase({
          label,
          text,
          field,
        },) {
          return it({
            name: `throws PnprConfigShapeError for ${label}`,
            fn: async () => {
              /**
               Error thrown for this config.
               */
              const caught = (function readConfig(): unknown {
                try {
                  readPnprPublishTarget(text,);
                  return 'no error';
                }
                catch (error) {
                  return error;
                }
              })();
              expect(caught,).toBeInstanceOf(PnprConfigShapeError,);
              expect(String(caught,),).toContain(field,);
            },
          },);
        },),
      ],
    },),
    describe({
      name: isTypeScriptSourcePath.name,
      children: [
        it({
          name: 'recognizes every TypeScript source ending',
          fn: async () => {
            expect([
              'src/index.ts',
              'src/index.mts',
              'src/index.cts',
              'src/view.tsx',
            ].map(function classify(path,) {
              return isTypeScriptSourcePath(path,);
            },),).toEqual([true, true, true, true,],);
          },
        },),
        it({
          name: 'treats declarations, JavaScript, and JSON as publishable',
          fn: async () => {
            expect([
              'dist/index.d.ts',
              'dist/index.d.mts',
              'dist/index.d.cts',
              'dist/index.mjs',
              'index.json',
            ].map(function classify(path,) {
              return isTypeScriptSourcePath(path,);
            },),).toEqual([false, false, false, false, false,],);
          },
        },),
      ],
    },),
    describe({
      name: prepareManifestForPnpr.name,
      children: [
        it({
          name: 'drops private and ./ts subpaths from a subpath exports map',
          fn: async () => {
            expect(prepareManifestForPnpr({
              name: '@scope/a',
              private: true,
              exports: {
                '.': './dist/index.mjs',
                './ts': './src/index.ts',
                './ts/*': './src/*',
              },
            },),).toEqual({
              name: '@scope/a',
              exports: { '.': './dist/index.mjs', },
            },);
          },
        },),
        it({
          name: 'keeps condition maps, string exports, and array exports unchanged',
          fn: async () => {
            expect(prepareManifestForPnpr({ exports: { import: './a.mjs', types: './a.d.mts', }, },),).toEqual({
              exports: { import: './a.mjs', types: './a.d.mts', },
            },);
            expect(prepareManifestForPnpr({ exports: './index.mjs', },),).toEqual({ exports: './index.mjs', },);
            expect(prepareManifestForPnpr({ exports: ['./a.mjs',], },),).toEqual({ exports: ['./a.mjs',], },);
          },
        },),
        it({
          name: 'drops main and module fields that name TypeScript source and keeps loadable ones',
          fn: async () => {
            expect(prepareManifestForPnpr({
              name: '@scope/cli',
              main: 'src/index.ts',
              module: 'dist/index.mjs',
              bin: { cli: 'dist/cli.mjs', },
            },),).toEqual({
              name: '@scope/cli',
              module: 'dist/index.mjs',
              bin: { cli: 'dist/cli.mjs', },
            },);
            expect(prepareManifestForPnpr({ module: 'src/index.mts', },),).toEqual({},);
            expect(prepareManifestForPnpr({ main: 'index.json', },),).toEqual({ main: 'index.json', },);
          },
        },),
        it({
          name: 'drops TypeScript-source bins, keeps loadable ones, and removes bin when none remain',
          fn: async () => {
            expect(prepareManifestForPnpr({
              bin: {
                built: 'dist/cli.mjs',
                source: 'src/cli.ts',
              },
            },),).toEqual({ bin: { built: 'dist/cli.mjs', }, },);
            expect(prepareManifestForPnpr({ bin: { source: './src/cli.ts', }, },),).toEqual({},);
            expect(prepareManifestForPnpr({ bin: 'src/cli.ts', },),).toEqual({},);
            expect(prepareManifestForPnpr({ bin: 'dist/cli.mjs', },),).toEqual({ bin: 'dist/cli.mjs', },);
          },
        },),
        it({
          name: 'adds no exports or publishConfig keys when the manifest had none',
          fn: async () => {
            expect(prepareManifestForPnpr({ name: '@scope/a', main: './index.js', },),).toEqual({
              name: '@scope/a',
              main: './index.js',
            },);
          },
        },),
        it({
          name: 'removes npmjs-only publishConfig settings and strips its exports',
          fn: async () => {
            expect(prepareManifestForPnpr({
              publishConfig: {
                access: 'public',
                provenance: true,
                registry: 'https://registry.npmjs.org/',
                exports: {
                  '.': './dist/index.mjs',
                  './ts': './src/index.ts',
                },
              },
            },),).toEqual({
              publishConfig: {
                access: 'public',
                exports: { '.': './dist/index.mjs', },
              },
            },);
          },
        },),
        it({
          name: 'drops a non-object publishConfig',
          fn: async () => {
            expect(prepareManifestForPnpr({ publishConfig: 'nonsense', },),).toEqual({},);
          },
        },),
        it({
          name: 'does not modify its input',
          fn: async () => {
            /**
             Manifest checked for mutation.
             */
            const manifest = { private: true, exports: { './ts': './src/index.ts', }, };
            prepareManifestForPnpr(manifest,);
            expect(manifest,).toEqual({ private: true, exports: { './ts': './src/index.ts', }, },);
          },
        },),
      ],
    },),
    describe({
      name: orderForPublishing.name,
      children: [
        it({
          name: 'places dependencies before dependents',
          fn: async () => {
            expect(orderForPublishing([
              { name: 'c', dependencyNames: ['b',], },
              { name: 'b', dependencyNames: ['a',], },
              { name: 'a', dependencyNames: [], },
            ],),).toEqual({ order: ['a', 'b', 'c',], cycleMembers: [], },);
          },
        },),
        it({
          name: 'breaks ties by name and ignores dependencies outside the batch and self edges',
          fn: async () => {
            expect(orderForPublishing([
              { name: 'z', dependencyNames: ['outside', 'z',], },
              { name: 'm', dependencyNames: [], },
            ],),).toEqual({ order: ['m', 'z',], cycleMembers: [], },);
          },
        },),
        it({
          name: 'appends cycle members in name order after everything placeable',
          fn: async () => {
            expect(orderForPublishing([
              { name: 'y', dependencyNames: ['x',], },
              { name: 'x', dependencyNames: ['y',], },
              { name: 'free', dependencyNames: [], },
            ],),).toEqual({ order: ['free', 'x', 'y',], cycleMembers: ['x', 'y',], },);
          },
        },),
        it({
          name: 'returns empty results for no candidates',
          fn: async () => {
            expect(orderForPublishing([],),).toEqual({ order: [], cycleMembers: [], },);
          },
        },),
      ],
    },),
  ],
},);
