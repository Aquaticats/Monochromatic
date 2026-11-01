/**
 Container integration test for the encrypted env store reader.
 
 Runs inside an Ubuntu matrix container as root and as a non-root user.
 The test installs sops and age through the mounted repo's mise config,
 creates a synthetic age identity and encrypted store, then exercises the
 reader through its built artifact. No real credential is read or written.
 
 @module
 */

import { execFileSync, } from 'node:child_process';
import {
  chmod,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { homedir, } from 'node:os';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  readEnvLocalJson,
  type ReadEnvLocalJsonOptions,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Repository root from the mounted container path.
 */
const REPO_ROOT = join(
  import.meta.dirname,
  '..',
  '..',
  '..',
  '..',
);

/**
 Mise executable installed by the matrix node runtime.
 */
const MISE = join(homedir(), '.local', 'bin', 'mise',);

/**
 Real container home where mise installed node, sops, and age. The fixture
 keeps its own temp root, but the sops shim must resolve tools from this home.
 */
const REAL_HOME = process.env.HOME
  ?? homedir();

/**
 Synthetic provider values.
 */
const VALUES = {
  EXA_API_KEY: 'matrix-exa-sentinel',
  LINKUP_API_KEY: 'matrix-linkup-sentinel',
} as const;

/**
 Install sops and age from the mounted repo config and expose their shims.
 */
function installStoreTools(): void {
  /**
   Environment that trusts the mounted repo and ignores the absent store.
   */
  const env = {
    ...process.env,
    MISE_QUIET: '1',
    MISE_TERMINAL_PROGRESS: 'false',
    MISE_SOPS_STRICT: 'false',
    MISE_TRUSTED_CONFIG_PATHS: REPO_ROOT,
  };
  execFileSync(
    MISE,
    ['install', 'sops', 'age',],
    {
      cwd: REPO_ROOT,
      env,
      stdio: 'inherit',
    },
  );
  process.env.MISE_SOPS_STRICT = 'false';
  process.env.MISE_TRUSTED_CONFIG_PATHS = REPO_ROOT;
  process.env.PATH = `${join(homedir(), '.local', 'share', 'mise', 'shims',)}:${process.env.PATH ?? ''}`;
}

/**
 Create an age identity and an encrypted store in a disposable directory.
 
 @returns fixture root, identity path, and store path
 */
async function createFixture(): Promise<{
  readonly root: string;
  readonly identityPath: string;
  readonly storePath: string;
}> {
  /**
   Local value for root.
   */
  const root = await mkdtemp(join(scratchRoot(), 'env-local-json-matrix-',),);
  await chmod(root, 0o700,);
  /**
   Local value for identityPath.
   */
  const identityPath = join(root, 'identity.txt',);
  execFileSync('age-keygen', ['--output', identityPath,], { stdio: 'ignore', },);
  await chmod(identityPath, 0o600,);
  /**
   Local value for identityText.
   */
  const identityText = await readFile(identityPath, 'utf8',);
  /**
   Public key extracted from the identity file's comment line.
   */
  const recipient = publicKeyFromIdentity({ identityText, },);
  /**
   Local value for plaintextPath.
   */
  const plaintextPath = join(root, 'plain.json',);
  /**
   Local value for storePath.
   */
  const storePath = join(root, '.env.local.json',);
  await writeFile(plaintextPath, JSON.stringify(VALUES,), 'utf8',);
  execFileSync(
    'sops',
    [
      '--encrypt',
      '--age',
      recipient,
      '--input-type',
      'json',
      '--output-type',
      'json',
      '--output',
      storePath,
      plaintextPath,
    ],
    { stdio: 'ignore', },
  );
  await chmod(storePath, 0o600,);
  return { root, identityPath, storePath, };
}

//endregion Fixtures

/**
 Extracts the age public key from an identity file's `public key:` comment.

 @param identityText - full text of the generated identity file

 @returns age recipient string

 @throws Error when the identity file has no public-key comment

 @example
 ```ts
 const recipient = publicKeyFromIdentity({ identityText });
 ```
 */
function publicKeyFromIdentity({ identityText, }: {
  readonly identityText: string;
}): string {
  /**
   Marker preceding the public key in an age identity file.
   */
  const marker = 'public key: ';
  /**
   Offset of the marker in the identity text.
   */
  const markerStart = identityText.indexOf(marker,);
  if (markerStart === (-1))
    throw new Error('synthetic age identity has no public-key comment',);
  /**
   Text after the marker, possibly including later lines.
   */
  const rest = identityText.slice(markerStart + marker.length,);
  /**
   Offset of the newline ending the comment line.
   */
  const lineEnd = rest.indexOf('\n',);
  return (lineEnd === (-1) ? rest : rest.slice(0, lineEnd)).trim();
}

/**
 Reads a store and returns its rejection.

 @param options - reader options for the expected-failure read

 @returns rejection value

 @throws Error when the read unexpectedly resolves

 @example
 ```ts
 const rejection = await captureRejection({ keys: ['A'] });
 ```
 */
async function captureRejection(
  options: Readonly<ReadEnvLocalJsonOptions>,
): Promise<unknown> {
  try {
    await readEnvLocalJson(options,);
  }
  catch (error) {
    return error;
  }
  throw new Error('expected the encrypted store reader to reject',);
}

installStoreTools();
const fixture = await createFixture();

await describe({
  name: 'env-local-json container reader',
  children: [
    it({
      name: 'decrypts requested keys with a real sops and age toolchain',
      fn: async () => {
        /**
         Local value for result.
         */
        const result = await readEnvLocalJson({
          keys: ['EXA_API_KEY', 'LINKUP_API_KEY',],
          path: fixture.storePath,
          cwd: fixture.root,
          env: {
            PATH: process.env.PATH ?? '',
            HOME: REAL_HOME,
            MISE_SOPS_STRICT: 'false',
            MISE_TRUSTED_CONFIG_PATHS: REPO_ROOT,
            SOPS_AGE_KEY_FILE: fixture.identityPath,
          },
          home: REAL_HOME,
        },);

        expect(result.loaded,).toBe(true,);
        expect(result.values.EXA_API_KEY,).toBe(VALUES.EXA_API_KEY,);
        expect(result.values.LINKUP_API_KEY,).toBe(VALUES.LINKUP_API_KEY,);
      },
    },),
    it({
      name: 'rejects a plaintext store without touching the identity',
      fn: async () => {
        /**
         Local value for plaintextPath.
         */
        const plaintextPath = join(fixture.root, 'plaintext-store.json',);
        await writeFile(plaintextPath, JSON.stringify(VALUES,), 'utf8',);

        /**
         Rejection raised by the plaintext store read.
         */
        const caught = await captureRejection({
          keys: ['EXA_API_KEY',],
          path: plaintextPath,
          cwd: fixture.root,
          env: {
            PATH: process.env.PATH ?? '',
            HOME: REAL_HOME,
            MISE_TRUSTED_CONFIG_PATHS: REPO_ROOT,
          },
          home: REAL_HOME,
        },);
        if (!(Error.isError(caught,)))
          throw new Error('plaintext store rejection was not an Error',);
        expect(caught.name,).toBe('EnvLocalJsonPlaintextError',);
        expect(caught.message,).not.toContain(VALUES.EXA_API_KEY,);
        await rm(plaintextPath,);
      },
    },),
  ],
},);
