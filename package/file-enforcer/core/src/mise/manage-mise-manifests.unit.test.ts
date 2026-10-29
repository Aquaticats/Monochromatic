import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';

import { manageMiseManifests, } from '../../dist/final/node/index.mjs';

//region Temporary fixture helpers

/**
 Creates isolated temporary directory for manifest discovery tests.
 
 @returns Absolute temporary directory path
 
 @example
 ```ts
 const tempDir = await setup();
 ```
 */
async function setup(): Promise<string> {
  return mkdtemp(join(tmpdir(), 'file-enforcer-mise-manifests-',),);
}

/**
 Builds a disposable staleness manifest path inside a test's own fixture directory,
 so writes never contend for the workspace's shared manifest and lock.

 @param tempDir - Fixture directory returned by {@link setup}.

 @returns Manifest path under the fixture directory, removed with it by the test cleanup.

 @example
 ```ts
 const manifestPath = stalenessManifestPathFor(tempDir);
 ```
 */
function stalenessManifestPathFor(tempDir: string,): string {
  return join(tempDir, 'staleness-manifest.json',);
}

//endregion Temporary fixture helpers

await describe({
  name: manageMiseManifests.name,
  children: [
    it({
      name: 'rewrites a literal-quoted header key as a basic string',
      fn: async () => {
        const tempDir = await setup();
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return rm(tempDir, { recursive: true, force: true, },);
          },
        };
        /**
         Package directory standing in for one workspace entry.
         */
        const packageDir = join(tempDir, 'a',);
        await mkdir(packageDir, { recursive: true, },);
        /**
         Manifest path with one literal-quoted header key and one shared line.
         */
        const manifestPath = join(packageDir, 'mise.toml',);
        await writeFile(manifestPath, `[tasks.'lint:types']
extends = 'lint:types'
`,);
        await manageMiseManifests({
          manifestGlobs: [join(tempDir, '*', 'mise.toml',),],
          manifestPath: stalenessManifestPathFor(tempDir,),
        },);
        expect(await readFile(manifestPath, 'utf8',),).toBe(`[tasks."lint:types"]
extends = 'lint:types'
`,);
      },
    }),
    it({
      name: 'leaves already-canonical manifests byte-identical',
      fn: async () => {
        const tempDir = await setup();
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return rm(tempDir, { recursive: true, force: true, },);
          },
        };
        /**
         Manifest whose headers already use basic quoting.
         */
        const manifestPath = join(tempDir, 'mise.toml',);
        /**
         Canonical text used as both input and expected output.
         */
        const canonical = `[tasks."lint"]
extends = "lint"
`;
        await writeFile(manifestPath, canonical,);
        await manageMiseManifests({
          manifestGlobs: [join(tempDir, 'mise.toml',),],
          manifestPath: stalenessManifestPathFor(tempDir,),
        },);
        expect(await readFile(manifestPath, 'utf8',),).toBe(canonical,);
      },
    }),
    it({
      name: 'normalizes a manifest matched by overlapping globs once',
      fn: async () => {
        const tempDir = await setup();
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return rm(tempDir, { recursive: true, force: true, },);
          },
        };
        /**
         Manifest matched by both the literal path and the wildcard pattern.
         */
        const manifestPath = join(tempDir, 'mise.toml',);
        await writeFile(manifestPath, `[tasks.'build']
extends = "build"
`,);
        await manageMiseManifests({
          manifestGlobs: [
            join(tempDir, 'mise.toml',),
            join(tempDir, '*', 'mise.toml',),
          ],
          manifestPath: stalenessManifestPathFor(tempDir,),
        },);
        expect(await readFile(manifestPath, 'utf8',),).toBe(`[tasks."build"]
extends = "build"
`,);
      },
    }),
    it({
      name: 'is idempotent across two runs',
      fn: async () => {
        const tempDir = await setup();
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return rm(tempDir, { recursive: true, force: true, },);
          },
        };
        /**
         Manifest with one literal-quoted sub-table header.
         */
        const manifestPath = join(tempDir, 'mise.toml',);
        await writeFile(manifestPath, `[tasks.'x'.env]
A = "1"
`,);
        const manifestGlobs = [join(tempDir, 'mise.toml',),];
        await manageMiseManifests({
          manifestGlobs,
          manifestPath: stalenessManifestPathFor(tempDir,),
        },);
        /**
         Text after the first run; the second run must not change it again.
         */
        const firstRun = await readFile(manifestPath, 'utf8',);
        await manageMiseManifests({
          manifestGlobs,
          manifestPath: stalenessManifestPathFor(tempDir,),
        },);
        expect(await readFile(manifestPath, 'utf8',),).toBe(firstRun,);
        expect(firstRun,).toBe(`[tasks."x".env]
A = "1"
`,);
      },
    }),
    it({
      name: 'resolves without a write when no manifest matches',
      fn: async () => {
        const tempDir = await setup();
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return rm(tempDir, { recursive: true, force: true, },);
          },
        };
        await manageMiseManifests({
          manifestGlobs: [join(tempDir, 'missing', '*', 'mise.toml',),],
          manifestPath: stalenessManifestPathFor(tempDir,),
        },);
      },
    }),
  ],
});
