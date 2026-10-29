import {
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { rolldown, } from 'rolldown';

import {
  binShebangPlugin,
  binTargetPaths,
  injectNodeShebang,
  readBinTargetPaths,
} from './bin-shebang.ts';

/**
 Disposable manifest and entry-source fixture directory.
 */
type BinBuildFixture = {
  readonly path: string;
  [Symbol.dispose]: () => void;
};

/**
 Creates a disposable package directory holding one manifest and one
 shebangless entry source.
 
 @param manifestText - Raw package.json content declaring bin targets.
 
 @param entryText - Entry source content, never carrying a shebang.
 
 @returns Disposable fixture outside repository state.
 
 @example
 ```ts
 using fixture = createBinBuildFixture('{"bin":"dist/final/node/cli.mjs"}', 'run();');
 ```
 */
function createBinBuildFixture(
  manifestText: string,
  entryText: string,
): BinBuildFixture {
  /**
   Fresh fixture directory under the system temp root.
   */
  const path = mkdtempSync(join(
    scratchRoot(),
    'bin-shebang-',),);
  writeFileSync(join(
    path,
    'package.json',
  ), manifestText,);
  writeFileSync(join(
    path,
    'cli.ts',
  ), entryText,);
  return {
    path,
    [Symbol.dispose]: function removeFixture(): void {
      rmSync(path, {
        force: true,
        recursive: true,
      },);
    },
  };
}

/**
 Build the fixture entry through the plugin under test and read the
 emitted bin file back from disk.
 
 Uses the shared Node preset's chunk naming and the real write path so
 bin target matching and the on-disk artifact are exercised together.
 
 @param packageDir - Fixture directory holding package.json and cli.ts.
 
 @returns Content of the emitted `dist/final/node/cli.mjs` file.
 
 @example
 ```ts
 const emitted = await buildFixtureEntry(fixture.path);
 ```
 */
async function buildFixtureEntry(packageDir: string,): Promise<string> {
  /**
   Real rolldown build of the fixture entry.
   */
  const build = await rolldown({
    input: [join(
      packageDir,
      'cli.ts',
    ),],
    plugins: [binShebangPlugin({
      outputDir: 'dist/final/node',
      packageDir,
    },),],
  },);
  await build.write({
    dir: join(
      packageDir,
      'dist/final/node',
    ),
    format: 'es',
    entryFileNames: '[name].mjs',
  },);
  return readFileSync(join(
    packageDir,
    'dist/final/node/cli.mjs',
  ), 'utf8',);
}

await describe({
  name: injectNodeShebang.name,
  children: [
    it({
      name: 'prepends the Node shebang to chunk code without one',
      fn: async function prependsWhenMissing(): Promise<void> {
        expect(injectNodeShebang("run();\n",),).toBe("#!/usr/bin/env node\nrun();\n",);
      },
    },),
    it({
      name: 'leaves code starting with a hashbang byte-identical',
      fn: async function leavesExistingAlone(): Promise<void> {
        /**
         Chunk code carrying a hashbang the author chose.
         */
        const code = '#!/usr/bin/env bun\nrun();\n';
        expect(injectNodeShebang(code,),).toBe(code,);
      },
    },),
  ],
},);

await describe({
  name: binTargetPaths.name,
  children: [
    it({
      name: 'normalizes map and string forms with and without dot-slash',
      fn: async function normalizesForms(): Promise<void> {
        expect(binTargetPaths({
          manifest: {
            bin: {
              'fixture-a': './dist/final/node/a.mjs',
              'fixture-b': 'dist/final/node/b.mjs',
            },
          },
        },),).toEqual([
          'dist/final/node/a.mjs',
          'dist/final/node/b.mjs',
        ],);
        expect(binTargetPaths({
          manifest: { bin: './bundle/node/index.mjs', },
        },),).toEqual(
          ['bundle/node/index.mjs',],
        );
      },
    },),
    it({
      name: 'yields no targets when the manifest declares no bin',
      fn: async function noBinField(): Promise<void> {
        expect(binTargetPaths({ manifest: {}, },).length,).toBe(0,);
      },
    },),
  ],
},);

await describe({
  name: readBinTargetPaths.name,
  children: [
    it({
      name: 'reads bin targets from the consuming package manifest',
      fn: async function readsManifest(): Promise<void> {
        using fixture = createBinBuildFixture(
          '{"bin":{"fixture-cli":"./dist/final/node/cli.mjs"}}',
          "run();\n",
        );
        expect(await readBinTargetPaths({ packageDir: fixture.path, },),).toEqual(
          ['dist/final/node/cli.mjs',],
        );
      },
    },),
    it({
      name: 'rejects manifests whose bin field is neither string nor object',
      fn: async function rejectsMalformedBin(): Promise<void> {
        using fixture = createBinBuildFixture('{"bin":42}', "run();\n",);
        await expect(readBinTargetPaths({ packageDir: fixture.path, },),)
          .rejects
          .toThrow('package.json bin must be a string or an object',);
      },
    },),
    it({
      name: 'rejects bin maps holding non-string targets',
      fn: async function rejectsNonStringTargets(): Promise<void> {
        using fixture = createBinBuildFixture('{"bin":{"fixture-cli":7}}', "run();\n",);
        await expect(readBinTargetPaths({ packageDir: fixture.path, },),)
          .rejects
          .toThrow('package.json bin targets must be strings',);
      },
    },),
  ],
},);

await describe({
  name: binShebangPlugin.name,
  children: [
    it({
      name: 'emits the Node shebang for a bin-targeted chunk of a shebangless entry',
      fn: async function injectsBinTarget(): Promise<void> {
        using fixture = createBinBuildFixture(
          '{"bin":{"fixture-cli":"./dist/final/node/cli.mjs"}}',
          "console.log('bin shebang fixture');\n",
        );
        /**
         Emitted bin file content after the plugin ran.
         */
        const emitted = await buildFixtureEntry(fixture.path,);
        expect(emitted.startsWith('#!/usr/bin/env node\n',),).toBe(true,);
        expect(emitted,).toContain('bin shebang fixture',);
      },
    },),
    it({
      name: 'leaves entry chunks no bin target names untouched',
      fn: async function skipsNonTargets(): Promise<void> {
        using fixture = createBinBuildFixture(
          '{"bin":{"fixture-cli":"./dist/final/node/other.mjs"}}',
          "console.log('bin shebang fixture');\n",
        );
        /**
         Emitted bin file content after the plugin ran.
         */
        const emitted = await buildFixtureEntry(fixture.path,);
        expect(emitted.startsWith('#!',),).toBe(false,);
      },
    },),
  ],
},);
