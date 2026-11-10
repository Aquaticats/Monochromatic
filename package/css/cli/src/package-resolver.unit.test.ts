/**
 Package resolution at the boundaries a build depends on: the `node_modules`
 walk, the manifest read, the `exports` field shapes Node accepts, and the
 fallbacks a package without an `exports` field relies on.
 Absence in this module is a unique Symbol rather than a nullish value, so a
 case that expects "nothing found" asserts the returned value is a symbol and
 a case that expects a path asserts the path itself.
 */
import {
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { dirname, join, } from 'node:path';

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  CssPackageManifestError,
  findPackageDir,
  readPackageJson,
  resolveExports,
  resolvePackage,
} from '@monochromatic-dev/css-cli';

//region Test helpers

/**
 One disposable directory per suite run, seeded with whatever tree the cases
 need, so no fixture lands in the package or outlives the run.
 */
type DisposableDirectory = {
  /**
   Absolute path of the directory.
   */
  readonly path: string;
  /**
   Removes the directory and everything written into it.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates a disposable directory seeded with the given files.
 
 @param files - Relative path to file content map written before the tests run.
 
 @returns Directory path plus the disposal hook that removes it.
 */
async function seededDirectory(
  files: Readonly<Record<string, string>>,
): Promise<DisposableDirectory> {
  /**
   Directory owning every file this run writes.
   */
  const directoryPath = await mkdtemp(join(
    scratchRoot(),
    'css-cli-resolver-',
  ),);
  await Promise.all(
    Object.entries(files,)
      .map(async function writeFixture([relativePath, content,],) {
        /**
         Absolute path of the seeded file.
         */
        const absolutePath = join(
          directoryPath,
          relativePath,
        );
        await mkdir(
          dirname(absolutePath,),
          { recursive: true, },
        );
        await writeFile(
          absolutePath,
          content,
        );
      },),
  );
  return {
    path: directoryPath,
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        directoryPath,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

/**
 Runs a call that is expected to throw.
 
 @param call - Call to run.
 
 @returns The thrown value, or undefined when the call returned.
 */
function thrownBy(call: () => unknown,): unknown {
  try {
    call();
    return undefined;
  }
  catch (error) {
    return error;
  }
}

//endregion Test helpers

await describe({
  name: 'package resolution',
  children: [
    describe({
      name: findPackageDir.name,
      children: [
        it({
          name: 'finds a package in the starting directory node_modules',
          fn: async () => {
            await using directory = await seededDirectory({
              'node_modules/@scope/pkg/package.json': '{}',
              'src/main.css': '',
            },);

            expect(findPackageDir({
              startDir: join(directory.path, 'src',),
              packageName: '@scope/pkg',
            },),)
              .toBe(join(directory.path, 'node_modules/@scope/pkg',),);
          },
        },),
        it({
          name: 'walks up from a nested directory to find the package',
          fn: async () => {
            await using directory = await seededDirectory({
              'node_modules/plain-pkg/package.json': '{}',
              'deep/deeper/main.css': '',
            },);

            expect(findPackageDir({
              startDir: join(directory.path, 'deep/deeper',),
              packageName: 'plain-pkg',
            },),)
              .toBe(join(directory.path, 'node_modules/plain-pkg',),);
          },
        },),
        it({
          name: 'reports absence when no node_modules holds the package up to the root',
          fn: async () => {
            await using directory = await seededDirectory({
              'src/main.css': '',
            },);

            expect(typeof findPackageDir({
              startDir: join(directory.path, 'src',),
              packageName: '@fixture-absent/never-installed',
            },),)
              .toBe('symbol',);
          },
        },),
      ],
    },),
    describe({
      name: readPackageJson.name,
      children: [
        it({
          name: 'parses a manifest it can read',
          fn: async () => {
            await using directory = await seededDirectory({
              'package.json': '{ "name": "readable" }',
            },);

            expect(readPackageJson(directory.path,),)
              .toEqual({ name: 'readable', },);
          },
        },),
        it({
          name: 'reports absence when the package holds no manifest',
          fn: async () => {
            await using directory = await seededDirectory({
              'src/main.css': '',
            },);

            expect(typeof readPackageJson(directory.path,),).toBe('symbol',);
          },
        },),
        it({
          name: 'throws naming the manifest when it is not parseable',
          fn: async () => {
            await using directory = await seededDirectory({
              'package.json': '{ "exports": ',
            },);

            /**
             Failure the unparseable manifest raises.
             */
            const caught = thrownBy(function readBrokenManifest() {
              readPackageJson(directory.path,);
            },);
            expect(caught,).toBeInstanceOf(CssPackageManifestError,);
            expect(String(caught,),).toContain(join(directory.path, 'package.json',),);
          },
        },),
      ],
    },),
    describe({
      name: resolveExports.name,
      children: [
        it({
          name: 'resolves an exact subpath key',
          fn: async () => {
            expect(resolveExports({
              exports: { './index.css': './src/index.css', },
              subpath: './index.css',
            },),)
              .toBe('./src/index.css',);
          },
        },),
        it({
          name: 'resolves a style condition',
          fn: async () => {
            expect(resolveExports({
              exports: { '.': { style: './style.css', }, },
              subpath: '.',
            },),)
              .toBe('./style.css',);
          },
        },),
        it({
          name: 'falls back to an import condition',
          fn: async () => {
            expect(resolveExports({
              exports: { '.': { import: './import.css', }, },
              subpath: '.',
            },),)
              .toBe('./import.css',);
          },
        },),
        it({
          name: 'falls back to a default condition',
          fn: async () => {
            expect(resolveExports({
              exports: { '.': { default: './default.css', }, },
              subpath: '.',
            },),)
              .toBe('./default.css',);
          },
        },),
        it({
          name: 'reads a top-level string exports as the package entry',
          fn: async () => {
            expect(resolveExports({
              exports: './entry.css',
              subpath: '.',
            },),)
              .toBe('./entry.css',);
          },
        },),
        it({
          name: 'reads a top-level array exports as alternatives in order',
          fn: async () => {
            expect(resolveExports({
              exports: ['./first.css', './second.css',],
              subpath: '.',
            },),)
              .toBe('./first.css',);
          },
        },),
        it({
          name: 'resolves a subpath through a wildcard pattern',
          fn: async () => {
            expect(resolveExports({
              exports: { './*': './src/*', },
              subpath: './theme.css',
            },),)
              .toBe('./src/theme.css',);
          },
        },),
        it({
          name: 'prefers the pattern with the longer prefix',
          fn: async () => {
            expect(resolveExports({
              exports: {
                './*': './loose/*',
                './a/*': './specific/*',
              },
              subpath: './a/theme.css',
            },),)
              .toBe('./specific/theme.css',);
          },
        },),
        it({
          name: 'resolves a nested condition object',
          fn: async () => {
            expect(resolveExports({
              exports: { '.': { import: { style: './nested.css', }, }, },
              subpath: '.',
            },),)
              .toBe('./nested.css',);
          },
        },),
        it({
          name: 'reports no match when the map has no such subpath',
          fn: async () => {
            expect(typeof resolveExports({
              exports: { './index.css': './src/index.css', },
              subpath: './other.css',
            },),)
              .toBe('symbol',);
          },
        },),
        it({
          name: 'reports no match when the exports field cannot describe a subpath',
          fn: async () => {
            expect(typeof resolveExports({
              exports: 42,
              subpath: '.',
            },),)
              .toBe('symbol',);
          },
        },),
      ],
    },),
    describe({
      name: resolvePackage.name,
      children: [
        it({
          name: 'resolves a subpath the exports map names',
          fn: async () => {
            await using directory = await seededDirectory({
              'node_modules/exporting-pkg/package.json': '{ "exports": { "./index.css": "./src/index.css" } }',
              'node_modules/exporting-pkg/src/index.css': '.entry { top: 0; }',
              'src/main.css': '',
            },);

            expect(resolvePackage({
              specifier: 'exporting-pkg/index.css',
              fromDir: join(directory.path, 'src',),
            },),)
              .toBe(join(directory.path, 'node_modules/exporting-pkg/src/index.css',),);
          },
        },),
        it({
          name: 'throws when the exports map names no such subpath',
          fn: async () => {
            await using directory = await seededDirectory({
              'node_modules/sealed-pkg/package.json': '{ "exports": { "./index.css": "./src/index.css" } }',
              'node_modules/sealed-pkg/src/index.css': '.entry { top: 0; }',
              'node_modules/sealed-pkg/src/unexported.css': '.hidden { top: 0; }',
              'src/main.css': '',
            },);

            /**
             Failure the unexported subpath raises.
             */
            const caught = thrownBy(function resolveUnexportedSubpath() {
              resolvePackage({
                specifier: 'sealed-pkg/src/unexported.css',
                fromDir: join(directory.path, 'src',),
              },);
            },);
            expect(String(caught,),).toContain("does not export subpath './src/unexported.css'",);
          },
        },),
        it({
          name: 'throws when an exported target does not exist',
          fn: async () => {
            await using directory = await seededDirectory({
              'node_modules/dangling-pkg/package.json': '{ "exports": { "./index.css": "./missing.css" } }',
              'src/main.css': '',
            },);

            /**
             Failure the dangling export target raises.
             */
            const caught = thrownBy(function resolveDanglingExport() {
              resolvePackage({
                specifier: 'dangling-pkg/index.css',
                fromDir: join(directory.path, 'src',),
              },);
            },);
            expect(String(caught,),).toContain("exports subpath './index.css' as './missing.css'",);
          },
        },),
        it({
          name: 'falls back to the style field for the package entry',
          fn: async () => {
            await using directory = await seededDirectory({
              'node_modules/styled-pkg/package.json': '{ "style": "src/style.css" }',
              'node_modules/styled-pkg/src/style.css': '.styled { top: 0; }',
              'src/main.css': '',
            },);

            expect(resolvePackage({
              specifier: 'styled-pkg',
              fromDir: join(directory.path, 'src',),
            },),)
              .toBe(join(directory.path, 'node_modules/styled-pkg/src/style.css',),);
          },
        },),
        it({
          name: 'falls back to the main field when no style field answers',
          fn: async () => {
            await using directory = await seededDirectory({
              'node_modules/main-pkg/package.json': '{ "main": "src/main.css" }',
              'node_modules/main-pkg/src/main.css': '.main { top: 0; }',
              'src/main.css': '',
            },);

            expect(resolvePackage({
              specifier: 'main-pkg',
              fromDir: join(directory.path, 'src',),
            },),)
              .toBe(join(directory.path, 'node_modules/main-pkg/src/main.css',),);
          },
        },),
        it({
          name: 'resolves a direct file path when the package has no manifest',
          fn: async () => {
            await using directory = await seededDirectory({
              'node_modules/manifestless-pkg/src/tokens.css': '.tokens { top: 0; }',
              'src/main.css': '',
            },);

            expect(resolvePackage({
              specifier: 'manifestless-pkg/src/tokens.css',
              fromDir: join(directory.path, 'src',),
            },),)
              .toBe(join(directory.path, 'node_modules/manifestless-pkg/src/tokens.css',),);
          },
        },),
        it({
          name: 'throws when no node_modules holds the package',
          fn: async () => {
            await using directory = await seededDirectory({
              'src/main.css': '',
            },);

            /**
             Failure the absent package raises.
             */
            const caught = thrownBy(function resolveAbsentPackage() {
              resolvePackage({
                specifier: '@fixture-absent/never-installed/index.css',
                fromDir: join(directory.path, 'src',),
              },);
            },);
            expect(String(caught,),).toContain("Cannot find package '@fixture-absent/never-installed'",);
          },
        },),
      ],
    },),
  ],
},);
