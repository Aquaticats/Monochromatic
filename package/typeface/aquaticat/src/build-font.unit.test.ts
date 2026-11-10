/**
 The build script reads the master glyph strip from a checkout path of any shape.

 `new URL(import.meta.url).pathname` is a URL path, not a filesystem path: Node
 percent-encodes spaces and non-ASCII characters in it, so a checkout under such
 a directory asked `readFile` for a `%20` spelling that did not exist and the
 build failed with ENOENT although `glyphs.svg` sat beside the script. This
 suite copies the package into a directory whose name contains a space, runs the
 real build script there, and asserts that the parse step found the strip.

 The copy is a disposable fixture: `uv` is replaced by a stub that writes the
 WOFF2 file the conversion step expects, so the suite exercises the glyph
 pipeline without a network or a Python install.
 */
/* oxlint-disable test-import/require-eventual-artifact -- this package ships src/ (package.json `files`) and builds font binaries rather than a JS artifact, so the modules beside this test are the shipped ones; see doc/planning/oxlint-test-import-eventual-artifact.md, section "Packages that ship source". */
import {
  cp,
  mkdir,
  mkdtemp,
  readFile,
  rm,
  symlink,
  writeFile,
} from 'node:fs/promises';
import {
  chmodSync,
  existsSync,
} from 'node:fs';
import { spawnSync, } from 'node:child_process';
import {
  dirname,
  join,
} from 'node:path';
import { fileURLToPath, } from 'node:url';

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseSvg, } from './parse-svg.ts';

//region Fixture helpers

/**
 Package directory whose sources the fixture copy reuses.
 */
const packageDirectory = join(
  import.meta.dirname,
  '..',
);

/**
 Runtime dependencies the build script imports, linked into the fixture so its
 copies resolve them the way this package does.
 */
const runtimeDependencies = [
  'nano-spawn',
  'opentype.js',
] as const;

/**
 Name of the stub that stands in for `uv` in the fixture.
 */
const uvStubName = 'uv';

/**
 Finds the root directory of an installed package.
 
 @param specifier - Module specifier to resolve from this test file
 
 @returns Absolute path of the package directory holding the resolved entry
 */
function installedPackageDirectory({ specifier, }: { readonly specifier: string; },): string {
  /**
   URL the module specifier resolves to.
   */
  const entryUrl = import.meta.resolve(specifier,);
  /**
   Directory the entry sits in, walked up to its package manifest.
   */
  let directory = dirname(fileURLToPath(entryUrl,),);
  while (!existsSync(join(
    directory,
    'package.json',
  ),)) {
    /**
     Parent of the directory being examined.
     */
    const parent = dirname(directory,);
    if (parent === directory)
      throw new Error(`no package.json above the entry of ${specifier}`,);
    directory = parent;
  }
  return directory;
}

/**
 Script the stub runs when the build asks for the WOFF2 conversion.
 
 @param woff2Path - Absolute path the conversion is expected to produce
 
 @returns Source of an executable Node script that writes the file
 */
function uvStubSource({ woff2Path, }: { readonly woff2Path: string; },): string {
  return [
    '#!/usr/bin/env node',
    `import { mkdirSync, writeFileSync } from 'node:fs';`,
    `mkdirSync(${JSON.stringify(dirname(woff2Path,),)}, { recursive: true });`,
    `writeFileSync(${JSON.stringify(woff2Path,)}, 'woff2 fixture');`,
    '',
  ].join('\n',);
}

/**
 Copies the package into a directory whose name contains a space and stubs `uv`.
 
 @param directory - Disposable directory that becomes the checkout copy
 
 @returns Absolute path of the fixture's `uv` stub directory
 */
async function seedCheckoutCopy({ directory, }: { readonly directory: string; },): Promise<string> {
  await cp(
    join(
      packageDirectory,
      'src',
    ),
    join(
      directory,
      'src',
    ),
    { recursive: true, },
  );
  await cp(
    join(
      packageDirectory,
      'package.json',
    ),
    join(
      directory,
      'package.json',
    ),
  );
  /**
   Dependency directory of the copy, holding one link per runtime dependency.
   */
  const dependencyDirectory = join(
    directory,
    'node_modules',
  );
  await mkdir(
    dependencyDirectory,
    { recursive: true, },
  );
  await Promise.all(runtimeDependencies.map(
    async function linkDependency(specifier,) {
      await symlink(
        installedPackageDirectory({ specifier, },),
        join(
          dependencyDirectory,
          specifier,
        ),
        'dir',
      );
    },
  ),);

  /**
   Directory holding the stub executables the fixture's PATH exposes.
   */
  const stubDirectory = join(
    directory,
    'bin',
  );
  await mkdir(
    stubDirectory,
    { recursive: true, },
  );
  /**
   Absolute path of the stub executable.
   */
  const stubPath = join(
    stubDirectory,
    uvStubName,
  );
  await writeFile(
    stubPath,
    uvStubSource({
      woff2Path: join(
        directory,
        'dist',
        'Aquaticat-Regular.woff2',
      ),
    },),
  );
  chmodSync(
    stubPath,
    0o755,
  );
  return stubDirectory;
}

//endregion Fixture helpers

await describe({
  name: '',
  children: [
    describe({
      name: 'build-font.ts',
      children: [
        it({
          name: 'reads glyphs.svg from a checkout directory whose name contains a space',
          timeout: 120_000,
          fn: async () => {
            /**
             Disposable copy of the package, named so the path carries a space.
             */
            const directory = await mkdtemp(join(
              scratchRoot(),
              'aquaticat glyph strip-',
            ),);
            await using guard = {
              [Symbol.asyncDispose]: async function removeCopy(): Promise<void> {
                await rm(
                  directory,
                  { recursive: true, force: true, },
                );
              },
            };
            // The binding exists solely for its dispose hook.
            void guard;

            /**
             Stub directory prepended to the child's PATH so `uv` resolves to the fixture.
             */
            const stubDirectory = await seedCheckoutCopy({ directory, },);
            /**
             Glyph count the copy must report, derived from the strip itself.
             */
            const expectedCellCount = parseSvg(
              await readFile(
                join(
                  directory,
                  'src',
                  'glyphs.svg',
                ),
                'utf8',
              ),
            ).length;
            /**
             WOFF2 file the stubbed conversion step must leave behind.
             */
            const woff2Path = join(
              directory,
              'dist',
              'Aquaticat-Regular.woff2',
            );

            /**
             Finished build run inside the copy.
             */
            const result = spawnSync(
              process.execPath,
              ['src/build-font.ts',],
              {
                cwd: directory,
                encoding: 'utf8',
                env: {
                  ...process.env,
                  PATH: `${stubDirectory}:${dirname(process.execPath,)}`,
                },
              },
            );

            expect(result.stdout,).toContain(`Reading glyph SVG: ${join(
              directory,
              'src',
              'glyphs.svg',
            )}`,);
            expect(result.stdout,).toContain(`Parsed ${expectedCellCount} glyph cells`,);
            expect(result.stderr,).toBe('',);
            expect(result.status,).toBe(0,);
            expect(existsSync(woff2Path,),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
