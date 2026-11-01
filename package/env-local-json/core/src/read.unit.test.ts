/**
 Public reader tests against real sops/age fixtures.

 Every store is synthetic and disposable; no repository store, developer
 identity, or credential is read.

 @module
 */

import {
  mkdir,
  realpath,
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
  encryptJson,
  readerEnvironment,
} from './fixture.ts';
import {
  EnvLocalJsonDecryptError,
  EnvLocalJsonMissingIdentityError,
  EnvLocalJsonPathError,
  EnvLocalJsonPlaintextError,
  EnvLocalJsonShapeError,
  EnvLocalJsonSopsMissingError,
  readEnvLocalJson,
  type ReadEnvLocalJsonOptions,
} from '../dist/final/node/index.mjs';

//region Constants

/**
 Sentinel value for the first fixture key.
 */
const EXA_VALUE = 'sentinel-exa-read-value';

/**
 Sentinel value for the second fixture key.
 */
const LINKUP_VALUE = 'sentinel-linkup-read-value';

/**
 Sentinel value for a key that was never requested.
 */
const UNREQUESTED_VALUE = 'sentinel-unrequested-value';

/**
 Environment value that must win over the store.
 */
const ENVIRONMENT_VALUE = 'sentinel-environment-precedence';

//endregion Constants

//region Fixtures

/**
 Shared real-toolchain fixture with three encrypted keys.
 */
await using fixture = await createSopsFixture({
  values: {
    EXA_API_KEY: EXA_VALUE,
    LINKUP_API_KEY: LINKUP_VALUE,
    UNREQUESTED_KEY: UNREQUESTED_VALUE,
  },
},);

/**
 Disposable root for absence, explicit-path, and tooling cases.
 */
await using temp = await createTempRoot({ prefix: 'env-local-json-read-', },);

/**
 Fake home with no identity configured.
 */
const emptyHome = join(temp.root, 'empty-home',);
await mkdir(emptyHome, { recursive: true, },);

/**
 Directory containing no executable, used to prove sops is required.
 */
const emptyPath = join(temp.root, 'empty-bin',);
await mkdir(emptyPath, { recursive: true, },);

//endregion Fixtures

//region Helpers

/**
 Captures the rejection of a reader call.

 @param options - reader options for the expected-failure read

 @returns rejection value

 @throws Error when the read unexpectedly resolves

 @example
 ```ts
 const error = await captureReadError({ keys: ['A'], path: '/missing' });
 ```
 */
async function captureReadError(
  options: Readonly<ReadEnvLocalJsonOptions>,
): Promise<unknown> {
  try {
    await readEnvLocalJson(options,);
  }
  catch (error) {
    return error;
  }
  throw new Error('expected the encrypted store reader to fail',);
}

/**
 Builds an environment that can decrypt the shared fixture.

 @param extra - extra environment values to layer in

 @returns reader environment with the fixture identity

 @example
 ```ts
 const env = fixtureEnvironment({ LINKUP_API_KEY: 'x' });
 ```
 */
function fixtureEnvironment({
  extra,
}: {
  readonly extra?: Readonly<Record<string, string>>;
},): Readonly<Record<string, string>> {
  return readerEnvironment({
    extra: {
      SOPS_AGE_KEY_FILE: fixture.identityPath,
      ...extra,
    },
    home: fixture.root,
  },);
}

//endregion Helpers

//region Requested keys

await describe({
  children: [
    it({
      fn: async () => {
        /**
         Reader result for both requested keys.
         */
        const result = await readEnvLocalJson({
          cwd: fixture.root,
          env: fixtureEnvironment({}),
          home: fixture.root,
          keys: ['EXA_API_KEY', 'LINKUP_API_KEY',],
          path: fixture.storePath,
        },);
        expect(result.loaded,).toBe(true,);
        expect(result.values,).toEqual({
          EXA_API_KEY: EXA_VALUE,
          LINKUP_API_KEY: LINKUP_VALUE,
        },);
        expect(result.path,).toBe(await realpath(fixture.storePath,),);
        expect(result.values.UNREQUESTED_KEY,).toBeUndefined();
      },
      name: 'returns requested keys only',
    },),
    it({
      fn: async () => {
        /**
         Reader result when the environment already supplies one key.
         */
        const result = await readEnvLocalJson({
          cwd: fixture.root,
          env: fixtureEnvironment({
            extra: { EXA_API_KEY: ENVIRONMENT_VALUE, },
          }),
          home: fixture.root,
          keys: ['EXA_API_KEY', 'LINKUP_API_KEY',],
          path: fixture.storePath,
        },);
        expect(result.loaded,).toBe(true,);
        expect(result.values,).toEqual({ LINKUP_API_KEY: LINKUP_VALUE, },);
      },
      name: 'returns only fill-in values so environment precedence survives',
    },),
    it({
      fn: async () => {
        /**
         Reader result when every requested key is already present.
         */
        const result = await readEnvLocalJson({
          cwd: fixture.root,
          env: readerEnvironment({
            extra: {
              EXA_API_KEY: ENVIRONMENT_VALUE,
              LINKUP_API_KEY: ENVIRONMENT_VALUE,
              PATH: emptyPath,
            },
            home: fixture.root,
          },),
          home: fixture.root,
          keys: ['EXA_API_KEY', 'LINKUP_API_KEY',],
          path: join(temp.root, 'missing-store.json',),
        },);
        expect(result.loaded,).toBe(false,);
        expect(result.values,).toEqual({},);
        expect(result.path,).toBeUndefined();
      },
      name: 'skips the store entirely when the environment supplies every key',
    },),
  ],
  name: readEnvLocalJson.name,
},);

//endregion Requested keys

//region Absence and explicit paths

await describe({
  children: [
    it({
      fn: async () => {
        /**
         Reader result in a directory with no store.
         */
        const result = await readEnvLocalJson({
          cwd: temp.root,
          env: readerEnvironment({ home: temp.root, },),
          home: temp.root,
          keys: ['EXA_API_KEY',],
        },);
        expect(result.loaded,).toBe(false,);
        expect(result.values,).toEqual({},);
      },
      name: 'returns absent when discovery finds no store',
    },),
    it({
      fn: async () => {
        /**
         Rejection for a missing explicit store path.
         */
        const caught = await captureReadError({
          env: fixtureEnvironment({}),
          home: fixture.root,
          keys: ['EXA_API_KEY',],
          path: join(temp.root, 'no-such-store.json',),
        },);
        expect(caught,).toBeInstanceOf(EnvLocalJsonPathError,);
      },
      name: 'raises a path error for a missing explicit path',
    },),
    it({
      fn: async () => {
        /**
         Rejection for an explicit path that is a directory.
         */
        const caught = await captureReadError({
          env: fixtureEnvironment({}),
          home: fixture.root,
          keys: ['EXA_API_KEY',],
          path: temp.root,
        },);
        expect(caught,).toBeInstanceOf(EnvLocalJsonPathError,);
        if (!(caught instanceof EnvLocalJsonPathError))
          throw new Error('directory rejection was not the path error',);
        expect(caught.reason,).toBe('not-a-file',);
      },
      name: 'raises a path error for an explicit path that is not a file',
    },),
    it({
      fn: async () => {
        /**
         Plaintext store that must never be read as configuration.
         */
        const plaintextPath = join(temp.root, 'plaintext-store.json',);
        await writeFile(
          plaintextPath,
          JSON.stringify({ EXA_API_KEY: EXA_VALUE, },),
          'utf8',
        );
        /**
         Rejection for the plaintext store.
         */
        const caught = await captureReadError({
          env: fixtureEnvironment({}),
          home: fixture.root,
          keys: ['EXA_API_KEY',],
          path: plaintextPath,
        },);
        if (!(caught instanceof EnvLocalJsonPlaintextError))
          throw new Error('plaintext rejection was not the named error',);
        expect(caught.message,).not.toContain(EXA_VALUE,);
      },
      name: 'rejects a plaintext store before resolving an identity',
    },),
    it({
      fn: async () => {
        /**
         Reader result using the environment path override.
         */
        const result = await readEnvLocalJson({
          cwd: temp.root,
          env: fixtureEnvironment({
            extra: { ENV_LOCAL_JSON_PATH: fixture.storePath, },
          }),
          home: fixture.root,
          keys: ['EXA_API_KEY',],
        },);
        expect(result.loaded,).toBe(true,);
        expect(result.values.EXA_API_KEY,).toBe(EXA_VALUE,);
        expect(result.path,).toBe(await realpath(fixture.storePath,),);
      },
      name: 'uses ENV_LOCAL_JSON_PATH as the authoritative explicit path',
    },),
    it({
      fn: async () => {
        /**
         Rejection from a set-but-missing environment path override.
         */
        const caught = await captureReadError({
          cwd: temp.root,
          env: fixtureEnvironment({
            extra: {
              ENV_LOCAL_JSON_PATH: join(temp.root, 'missing-env-store.json',),
            },
          }),
          home: fixture.root,
          keys: ['EXA_API_KEY',],
        },);
        expect(caught,).toBeInstanceOf(EnvLocalJsonPathError,);
        if (!(caught instanceof EnvLocalJsonPathError))
          throw new Error('environment override rejection was not a path error',);
        expect(caught.reason,).toBe('missing',);
      },
      name: 'treats a missing ENV_LOCAL_JSON_PATH as a hard path error',
    },),
  ],
  name: 'absence and explicit paths',
},);

//endregion Absence and explicit paths

//region Identity and tooling failures

await describe({
  children: [
    it({
      fn: async () => {
        /**
         Rejection when no identity candidate exists.
         */
        const caught = await captureReadError({
          cwd: fixture.root,
          env: readerEnvironment({ home: emptyHome, },),
          home: emptyHome,
          keys: ['EXA_API_KEY',],
          path: fixture.storePath,
        },);
        expect(caught,).toBeInstanceOf(EnvLocalJsonMissingIdentityError,);
      },
      name: 'raises a missing-identity error naming the candidates',
    },),
    it({
      fn: async () => {
        /**
         Rejection when sops is not on the curated PATH.
         */
        const caught = await captureReadError({
          cwd: fixture.root,
          env: readerEnvironment({
            extra: {
              PATH: emptyPath,
              SOPS_AGE_KEY_FILE: fixture.identityPath,
            },
            home: fixture.root,
          },),
          home: fixture.root,
          keys: ['EXA_API_KEY',],
          path: fixture.storePath,
        },);
        expect(caught,).toBeInstanceOf(EnvLocalJsonSopsMissingError,);
      },
      name: 'raises a missing-sops error with remediation text',
    },),
    it({
      fn: async () => {
        /**
         Identity that did not encrypt the fixture store.
         */
        const wrongIdentity = await createIdentityFile({
          fileName: 'wrong-read-identity.txt',
          fixture,
        },);
        /**
         Rejection from the wrong identity.
         */
        const caught = await captureReadError({
          env: readerEnvironment({
            extra: { SOPS_AGE_KEY_FILE: wrongIdentity, },
            home: fixture.root,
          },),
          home: fixture.root,
          keys: ['EXA_API_KEY',],
          path: fixture.storePath,
        },);
        expect(caught,).toBeInstanceOf(EnvLocalJsonDecryptError,);
        if (!(caught instanceof EnvLocalJsonDecryptError))
          throw new Error('wrong-identity rejection was not the decrypt error',);
        expect(caught.kind,).toBe('exit',);
        expect(caught.message,).not.toContain(EXA_VALUE,);
        expect(caught.message,).not.toContain(LINKUP_VALUE,);
      },
      name: 'raises a decrypt error for a wrong identity without values',
    },),
  ],
  name: 'identity and tooling failures',
},);

//endregion Identity and tooling failures

//region Shape validation

/**
 Decrypted documents that must fail shape validation.
 */
const SHAPE_CASES: readonly {
  readonly name: string;
  readonly json: string;
  readonly reason: string;
  readonly key: string;
}[] = [
  {
    json: '{"BAD":42}',
    key: 'BAD',
    name: 'rejects a non-string value',
    reason: 'non-string',
  },
  {
    json: '{"BAD":"   "}',
    key: 'BAD',
    name: 'rejects a blank value',
    reason: 'blank',
  },
  {
    json: '{"BAD":"ENC[AES256_GCM,data:abc]"}',
    key: 'BAD',
    name: 'rejects a still-encrypted value',
    reason: 'encrypted-token',
  },
];

await describe({
  children: SHAPE_CASES.map(function toShapeTest(shapeCase,) {
    return it({
      fn: async () => {
        /**
         Encrypted store holding the malformed document.
         */
        const storePath = await encryptJson({
          fileName: `shape-${shapeCase.reason}.json`,
          fixture,
          json: shapeCase.json,
        },);
        /**
         Rejection from the malformed store.
         */
        const caught = await captureReadError({
          env: fixtureEnvironment({}),
          home: fixture.root,
          keys: ['BAD',],
          path: storePath,
        },);
        expect(caught,).toBeInstanceOf(EnvLocalJsonShapeError,);
        if (!(caught instanceof EnvLocalJsonShapeError))
          throw new Error('shape rejection was not the named error',);
        expect(caught.reason,).toBe(shapeCase.reason,);
        expect(caught.key,).toBe(shapeCase.key,);
        expect(caught.message,).not.toContain(shapeCase.json,);
      },
      name: shapeCase.name,
    },);
  },),
  name: 'decrypted shape validation',
},);

//endregion Shape validation
