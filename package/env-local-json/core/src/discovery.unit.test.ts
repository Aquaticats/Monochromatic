/**
 Discovery tests: explicit path authority, repository anchoring, and walk
 bounds.

 Every tree lives in a disposable fixture; the repository's real store is
 never read.

 @module
 */

import {
  mkdir,
  realpath,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createSopsFixture,
  createTempRoot,
  encryptJson,
  readerEnvironment,
} from './fixture.ts';
import {
  readEnvLocalJson,
} from '../dist/final/node/index.mjs';

//region Constants

/**
 Sentinel value encrypted into every discovery fixture store.
 */
const STORE_VALUE = 'sentinel-discovery-store-value';

//endregion Constants

//region Fixtures

/**
 Disposable root for non-repository discovery trees.
 */
await using temp = await createTempRoot({ prefix: 'env-local-json-discovery-', },);

/**
 Home directory outside the fixture trees.
 */
const outsideHome = join(temp.root, 'outside-home',);
await mkdir(outsideHome, { recursive: true, },);

//endregion Fixtures

//region Helpers

/**
 Writes a structurally valid Git administrative directory.

 @param dir - directory that becomes the repository root

 @example
 ```ts
 await writeGitMarker({ dir: repoRoot });
 ```
 */
async function writeGitMarker({ dir, }: { readonly dir: string; },): Promise<void> {
  await mkdir(
    join(
      dir,
      '.git',
      'objects',
    ),
    { recursive: true, },
  );
  await mkdir(
    join(
      dir,
      '.git',
      'refs',
    ),
    { recursive: true, },
  );
  await writeFile(
    join(
      dir,
      '.git',
      'HEAD',
    ),
    'ref: refs/heads/main\n',
    'utf8',
  );
}

/**
 Writes the mise monorepo marker.

 @param dir - directory that becomes the monorepo root

 @example
 ```ts
 await writeMonorepoMarker({ dir: repoRoot });
 ```
 */
async function writeMonorepoMarker({ dir, }: { readonly dir: string; },): Promise<void> {
  await writeFile(
    join(
      dir,
      'mise.toml',
    ),
    '[monorepo]\n',
    'utf8',
  );
}

/**
 Builds a reader environment with the fixture identity.

 @param fixture - fixture whose store and identity are read

 @returns reader environment

 @example
 ```ts
 const env = discoveryEnvironment(fixture);
 ```
 */
function discoveryEnvironment(
  fixture: Readonly<{ readonly root: string; readonly identityPath: string; }>,
): Readonly<Record<string, string>> {
  return readerEnvironment({
    extra: { SOPS_AGE_KEY_FILE: fixture.identityPath, },
    home: fixture.root,
  },);
}

//endregion Helpers

//region Repository anchoring

await describe({
  children: [
    it({
      fn: async () => {
        await using fixture = await createSopsFixture({
          values: { EXA_API_KEY: STORE_VALUE, },
        },);
        await writeGitMarker({ dir: fixture.root, },);
        /**
         Nested working directory inside the repository.
         */
        const cwd = join(
          fixture.root,
          'src',
          'deep',
        );
        await mkdir(cwd, { recursive: true, },);
        /**
         Reader result anchored at the Git repository root.
         */
        const result = await readEnvLocalJson({
          cwd,
          env: discoveryEnvironment(fixture,),
          home: outsideHome,
          keys: ['EXA_API_KEY',],
        },);
        expect(result.loaded,).toBe(true,);
        expect(result.values.EXA_API_KEY,).toBe(STORE_VALUE,);
        expect(result.path,).toBe(await realpath(fixture.storePath,),);
      },
      name: 'anchors at a Git repository root containing the store',
    },),
    it({
      fn: async () => {
        await using fixture = await createSopsFixture({
          values: { EXA_API_KEY: STORE_VALUE, },
        },);
        await writeMonorepoMarker({ dir: fixture.root, },);
        /**
         Nested working directory inside the monorepo.
         */
        const cwd = join(
          fixture.root,
          'package',
          'deep',
        );
        await mkdir(cwd, { recursive: true, },);
        /**
         Reader result anchored at the mise monorepo root.
         */
        const result = await readEnvLocalJson({
          cwd,
          env: discoveryEnvironment(fixture,),
          home: outsideHome,
          keys: ['EXA_API_KEY',],
        },);
        expect(result.loaded,).toBe(true,);
        expect(result.path,).toBe(await realpath(fixture.storePath,),);
      },
      name: 'anchors at a mise monorepo root containing the store',
    },),
    it({
      fn: async () => {
        await using fixture = await createSopsFixture({
          values: { EXA_API_KEY: STORE_VALUE, },
        },);
        await writeGitMarker({ dir: fixture.root, },);
        await rm(fixture.storePath,);
        /**
         Nested directory holding the only store in the repository.
         */
        const storeDirectory = join(
          fixture.root,
          'nested',
        );
        await mkdir(storeDirectory, { recursive: true, },);
        await encryptJson({
          fileName: join('nested', '.env.local.json',),
          fixture,
          json: JSON.stringify({ EXA_API_KEY: STORE_VALUE, },),
        },);
        /**
         Working directory below the nested store.
         */
        const cwd = join(
          storeDirectory,
          'deep',
        );
        await mkdir(cwd, { recursive: true, },);
        /**
         Reader result from the bounded walk inside the repository.
         */
        const result = await readEnvLocalJson({
          cwd,
          env: discoveryEnvironment(fixture,),
          home: outsideHome,
          keys: ['EXA_API_KEY',],
        },);
        expect(result.loaded,).toBe(true,);
        expect(result.path,).toBe(
          await realpath(join(storeDirectory, '.env.local.json',),),
        );
      },
      name: 'finds a store in a repository subdirectory by bounded walk',
    },),
  ],
  name: 'repository anchoring',
},);

//endregion Repository anchoring

//region Walk bounds

await describe({
  children: [
    it({
      fn: async () => {
        await using fixture = await createSopsFixture({
          values: { EXA_API_KEY: STORE_VALUE, },
        },);
        /**
         Nested working directory with no repository marker.
         */
        const cwd = join(
          fixture.root,
          'nested',
          'deep',
        );
        await mkdir(cwd, { recursive: true, },);
        /**
         Reader result from the walk toward the filesystem root.
         */
        const result = await readEnvLocalJson({
          cwd,
          env: discoveryEnvironment(fixture,),
          home: outsideHome,
          keys: ['EXA_API_KEY',],
        },);
        expect(result.loaded,).toBe(true,);
      },
      name: 'walks outside a repository up to the filesystem root',
    },),
    it({
      fn: async () => {
        await using fixture = await createSopsFixture({
          values: { EXA_API_KEY: STORE_VALUE, },
        },);
        /**
         Home directory below the store, so the store is above the bound.
         */
        const home = join(fixture.root, 'home',);
        /**
         Working directory below the fake home.
         */
        const cwd = join(
          home,
          'project',
          'deep',
        );
        await mkdir(cwd, { recursive: true, },);
        /**
         Reader result that must not climb above home.
         */
        const result = await readEnvLocalJson({
          cwd,
          env: discoveryEnvironment(fixture,),
          home,
          keys: ['EXA_API_KEY',],
        },);
        expect(result.loaded,).toBe(false,);
      },
      name: 'stops at the home directory bound',
    },),
    it({
      fn: async () => {
        await using fixture = await createSopsFixture({
          values: { EXA_API_KEY: STORE_VALUE, },
        },);
        /**
         Repository root below the outer store.
         */
        const repositoryRoot = join(fixture.root, 'repo',);
        await writeGitMarker({ dir: repositoryRoot, },);
        /**
         Working directory inside the repository.
         */
        const cwd = join(
          repositoryRoot,
          'src',
        );
        await mkdir(cwd, { recursive: true, },);
        /**
         Reader result that must not climb above the repository root.
         */
        const result = await readEnvLocalJson({
          cwd,
          env: discoveryEnvironment(fixture,),
          home: outsideHome,
          keys: ['EXA_API_KEY',],
        },);
        expect(result.loaded,).toBe(false,);
      },
      name: 'stops at the repository-root bound',
    },),
  ],
  name: 'walk bounds',
},);

//endregion Walk bounds

//region Explicit authority

await describe({
  children: [
    it({
      fn: async () => {
        await using fixture = await createSopsFixture({
          values: { EXA_API_KEY: STORE_VALUE, },
        },);
        /**
         Plaintext store in the working directory that discovery would reject.
         */
        const decoyPath = join(temp.root, '.env.local.json',);
        await writeFile(
          decoyPath,
          JSON.stringify({ EXA_API_KEY: 'plaintext-decoy', },),
          'utf8',
        );
        /**
         Reader result for the explicit path.
         */
        const result = await readEnvLocalJson({
          cwd: temp.root,
          env: discoveryEnvironment(fixture,),
          home: outsideHome,
          keys: ['EXA_API_KEY',],
          path: fixture.storePath,
        },);
        expect(result.loaded,).toBe(true,);
        expect(result.values.EXA_API_KEY,).toBe(STORE_VALUE,);
      },
      name: 'uses an explicit path without running discovery',
    },),
  ],
  name: 'explicit authority',
},);

//endregion Explicit authority
