/**
 Atomic writer tests against a real sops/age fixture.

 Every case uses a disposable encrypted store; no repository store or
 developer identity is touched.

 @module
 */

import {
  readdir,
  readFile,
  stat,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createIdentityFile,
  createSopsFixture,
  createTempRoot,
  decryptStore,
  readerEnvironment,
} from './fixture.ts';
import {
  EnvLocalJsonPathError,
  EnvLocalJsonPlaintextError,
} from '../dist/final/node/index.mjs';
import {
  EnvLocalJsonWriteError,
  writeEnvLocalJsonValues,
  type WriteEnvLocalJsonValuesOptions,
} from '../dist/final/node/testing.mjs';

//region Fixtures

/**
 Missing-path fixture root for the store-absence case.
 */
const tempMissingRoot = await createTempRoot({ prefix: 'env-local-json-write-', },);

//endregion Fixtures

//region Constants

/**
 Original value encrypted into every fixture store.
 */
const ORIGINAL_EXA = 'sentinel-original-exa-value';

/**
 Original value preserved by every write.
 */
const ORIGINAL_LINKUP = 'sentinel-original-linkup-value';

/**
 Replacement value never allowed to appear in plaintext on disk.
 */
const REPLACEMENT_EXA = 'sentinel-replacement-exa-value';

/**
 Added value never allowed to appear in plaintext on disk.
 */
const ADDED_GAMMA = 'sentinel-added-gamma-value';

//endregion Constants

//region Helpers

/**
 Captures the rejection of an atomic write.

 @param options - writer options for the expected-failure write

 @returns rejection value

 @throws Error when the write unexpectedly resolves

 @example
 ```ts
 const error = await captureWriteError({ path, values: { A: 'x' }, env, home });
 ```
 */
async function captureWriteError(
  options: Readonly<WriteEnvLocalJsonValuesOptions>,
): Promise<unknown> {
  try {
    await writeEnvLocalJsonValues(options,);
  }
  catch (error) {
    return error;
  }
  throw new Error('expected the store write to fail',);
}

/**
 Reads every regular file directly under a directory.

 @param root - directory to inspect

 @returns file-name to content map

 @example
 ```ts
 const files = await readRootFiles(fixture.root);
 ```
 */
async function readRootFiles({
  root,
}: {
  readonly root: string;
}): Promise<Readonly<Record<string, string>>> {
  /**
   Names of the directory entries.
   */
  const names = await readdir(root,);
  /**
   Name and content pairs for every entry.
   */
  const entries = await Promise.all(names.map(async function readEntry(
    name: string,
  ): Promise<readonly [string, string]> {
    /**
     Full path of this entry.
     */
    const path = join(
      root,
      name,
    );
    /**
     Metadata of the entry.
     */
    const metadata = await stat(path,);
    /**
     Content for regular files, empty for other entries.
     */
    const content = metadata.isFile()
      ? await readFile(
        path,
        'utf8',
      )
      : '';
    return [name, content,];
  },),);
  /**
   File contents collected for the sentinel scan.
   */
  const contents: Record<string, string> = {};
  for (const [name, content,] of entries)
    contents[name] = content;
  return contents;
}

/**
 Builds a writer environment with the fixture identity.

 @param fixture - fixture owning the identity

 @returns environment for the writer

 @example
 ```ts
 const env = writerEnvironment(fixture);
 ```
 */
function writerEnvironment(
  fixture: Readonly<{
    readonly root: string;
    readonly identityPath: string;
  }>,
): Readonly<Record<string, string>> {
  return readerEnvironment({
    extra: { SOPS_AGE_KEY_FILE: fixture.identityPath, },
    home: fixture.root,
  },);
}

//endregion Helpers

//region Atomic write

await describe({
  children: [
    it({
      fn: async () => {
        await using fixture = await createSopsFixture({
          values: {
            EXA_API_KEY: ORIGINAL_EXA,
            LINKUP_API_KEY: ORIGINAL_LINKUP,
          },
        },);
        await writeEnvLocalJsonValues({
          env: writerEnvironment(fixture,),
          home: fixture.root,
          path: fixture.storePath,
          values: {
            EXA_API_KEY: REPLACEMENT_EXA,
            GAMMA_KEY: ADDED_GAMMA,
          },
        },);
        /**
         Decrypted store after the atomic write.
         */
        const decrypted = await decryptStore({
          fixture,
          storePath: fixture.storePath,
        },);
        expect(decrypted.EXA_API_KEY,).toBe(REPLACEMENT_EXA,);
        expect(decrypted.LINKUP_API_KEY,).toBe(ORIGINAL_LINKUP,);
        expect(decrypted.GAMMA_KEY,).toBe(ADDED_GAMMA,);

        /**
         Metadata of the published store.
         */
        const metadata = await stat(fixture.storePath,);
        expect(metadata.mode & 0o777,).toBe(0o600,);

        /**
         Directory entries after the write.
         */
        const entries = await readdir(fixture.root,);
        expect(
          entries.some(function isTemporary(name,): boolean {
            return name.includes('.tmp-',);
          },),
        ).toBe(false,);

        /**
         Direct file contents scanned for plaintext leakage.
         */
        const files = await readRootFiles({ root: fixture.root, },);
        expect(JSON.stringify(files,),).not.toContain(REPLACEMENT_EXA,);
        expect(JSON.stringify(files,),).not.toContain(ADDED_GAMMA,);
      },
      name: 'changes requested values, preserves other keys, and leaves no temp',
    },),
    it({
      fn: async () => {
        await using fixture = await createSopsFixture({
          values: { EXA_API_KEY: ORIGINAL_EXA, },
        },);
        /**
         Rejection from a key containing a JSON-path delimiter.
         */
        const caught = await captureWriteError({
          env: writerEnvironment(fixture,),
          home: fixture.root,
          path: fixture.storePath,
          values: { 'BAD KEY': 'x', },
        },);
        expect(caught,).toBeInstanceOf(EnvLocalJsonWriteError,);
        if (!(caught instanceof EnvLocalJsonWriteError))
          throw new Error('invalid-key rejection was not the named error',);
        expect(caught.reason,).toBe('invalid-key',);
        /**
         Metadata of the untouched store.
         */
        const metadata = await stat(fixture.storePath,);
        expect(metadata.size,).toBeGreaterThan(0,);
        /**
         Decrypted store that must still hold the original value.
         */
        const decrypted = await decryptStore({
          fixture,
          storePath: fixture.storePath,
        },);
        expect(decrypted.EXA_API_KEY,).toBe(ORIGINAL_EXA,);
      },
      name: 'rejects a key that would break the sops path expression',
    },),
    it({
      fn: async () => {
        await using fixture = await createSopsFixture({
          values: { EXA_API_KEY: ORIGINAL_EXA, },
        },);
        /**
         Rejection from a blank value.
         */
        const caught = await captureWriteError({
          env: writerEnvironment(fixture,),
          home: fixture.root,
          path: fixture.storePath,
          values: { EXA_API_KEY: '   ', },
        },);
        expect(caught,).toBeInstanceOf(EnvLocalJsonWriteError,);
        if (!(caught instanceof EnvLocalJsonWriteError))
          throw new Error('blank-value rejection was not the named error',);
        expect(caught.reason,).toBe('blank-value',);
      },
      name: 'rejects a blank value before touching the store',
    },),
    it({
      fn: async () => {
        /**
         Missing explicit store path.
         */
        const missingPath = join(tempMissingRoot.root, 'absent.json',);
        /**
         Rejection from the missing store.
         */
        const caught = await captureWriteError({
          env: {},
          home: missingPath,
          path: missingPath,
          values: { EXA_API_KEY: 'x', },
        },);
        expect(caught,).toBeInstanceOf(EnvLocalJsonPathError,);
      },
      name: 'requires an existing store',
    },),
    it({
      fn: async () => {
        await using fixture = await createSopsFixture({
          values: { EXA_API_KEY: ORIGINAL_EXA, },
        },);
        /**
         Plaintext store that must be rejected.
         */
        const plaintextPath = join(fixture.root, 'plaintext.json',);
        await writeFile(
          plaintextPath,
          JSON.stringify({ EXA_API_KEY: ORIGINAL_EXA, },),
          'utf8',
        );
        /**
         Rejection from the plaintext store.
         */
        const caught = await captureWriteError({
          env: writerEnvironment(fixture,),
          home: fixture.root,
          path: plaintextPath,
          values: { EXA_API_KEY: REPLACEMENT_EXA, },
        },);
        expect(caught,).toBeInstanceOf(EnvLocalJsonPlaintextError,);
      },
      name: 'rejects a plaintext store before writing',
    },),
    it({
      fn: async () => {
        await using fixture = await createSopsFixture({
          values: { EXA_API_KEY: ORIGINAL_EXA, },
        },);
        /**
         Identity that did not encrypt the store.
         */
        const wrongIdentity = await createIdentityFile({
          fileName: 'wrong-identity.txt',
          fixture,
        },);
        /**
         Rejection from the wrong decryption identity.
         */
        const caught = await captureWriteError({
          env: readerEnvironment({
            extra: { SOPS_AGE_KEY_FILE: wrongIdentity, },
            home: fixture.root,
          },),
          home: fixture.root,
          path: fixture.storePath,
          values: { EXA_API_KEY: REPLACEMENT_EXA, },
        },);
        expect(caught,).toBeInstanceOf(EnvLocalJsonWriteError,);
        if (!(caught instanceof EnvLocalJsonWriteError))
          throw new Error('wrong-identity rejection was not the named error',);
        expect(caught.reason,).toBe('sops-exit',);

        /**
         Directory entries after the failed write; the temp file must be gone.
         */
        const entries = await readdir(fixture.root,);
        expect(
          entries.some(function isTemporary(name,): boolean {
            return name.includes('.tmp-',);
          },),
        ).toBe(false,);

        /**
         Decrypted store that must still hold the original value.
         */
        const decrypted = await decryptStore({
          fixture,
          storePath: fixture.storePath,
        },);
        expect(decrypted.EXA_API_KEY,).toBe(ORIGINAL_EXA,);
      },
      name: 'cleans the temporary file and preserves the store on sops failure',
    },),
  ],
  concurrency: 1,
  name: writeEnvLocalJsonValues.name,
},);

//endregion Atomic write
