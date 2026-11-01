/**
 Synthetic encrypted-store fixtures for config tests.
 
 The fixtures use real `age-keygen` and `sops` against disposable temp
 directories. No fixture value is a credential.
 
 @module
 */

import {
  chmod,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import spawn from 'nano-spawn';

/**
 Prefix for disposable fixture directories.
 */
const FIXTURE_PREFIX = 'search-fetch-store-test-';

/**
 Sentinel value written when a test needs a plaintext store.
 */
const PLAINTEXT_SENTINEL = 'plaintext-sentinel-value';

/**
 Mode for a disposable fixture directory.
 */
const FIXTURE_DIRECTORY_MODE = 0o700;

/**
 Mode for private fixture files.
 */
const FIXTURE_FILE_MODE = 0o600;

/**
 Fail with remediation when a real crypto tool is unavailable.
 
 @param command - executable name to probe
 
 @throws when the command is not on `PATH`
 
 @example
 ```ts
 await requireTool('sops');
 ```
 */
async function requireTool(command: string,): Promise<void> {
  try {
    await spawn(
      'sh',
      [
        '-c',
        `command -v ${command}`,
      ],
      {
        stdout: 'ignore',
        stderr: 'ignore',
      },
    );
  }
  catch (error: unknown) {
    throw new Error(
      `${command} is required for these tests; run them from a mise shell where sops and age are installed.`,
      { cause: error, },
    );
  }
}

/**
 Resolve the public age recipient with the age-keygen identity converter.
 
 @param identityPath - generated identity path
 
 @returns age public recipient
 
 @throws when age-keygen cannot read the identity
 
 @example
 ```ts
 const recipient = await readRecipient('/tmp/identity.txt');
 ```
 */
async function readRecipient(identityPath: string,): Promise<string> {
  /**
   Local value for result.
   */
  const result = await spawn(
    'age-keygen',
    [
      '-y',
      identityPath,
    ],
  );
  /**
   Local value for recipient.
   */
  const recipient = result.stdout
    .trim();
  if (recipient === '')
    throw new Error('synthetic age identity has no public key',);
  return recipient;
}

/**
 Create a disposable root with an age identity.
 
 @returns fixture root and identity path
 
 @throws when sops or age-keygen is unavailable
 
 @example
 ```ts
 const { root, identityPath } = await createIdentityFixture();
 ```
 */
async function createIdentityFixture(): Promise<{
  readonly root: string;
  readonly identityPath: string;
}> {
  await requireTool('sops',);
  await requireTool('age-keygen',);
  /**
   Local value for root.
   */
  const root = await mkdtemp(
    join(
      scratchRoot(),
      FIXTURE_PREFIX,
    ),
  );
  await chmod(
    root,
    FIXTURE_DIRECTORY_MODE,
  );
  /**
   Local value for identityPath.
   */
  const identityPath = join(
    root,
    'identity.txt',
  );
  await spawn(
    'age-keygen',
    [
      '--output',
      identityPath,
    ],
    {
      stdout: 'ignore',
      stderr: 'ignore',
    },
  );
  await chmod(
    identityPath,
    FIXTURE_FILE_MODE,
  );
  return {
    root,
    identityPath,
  };
}

/**
 Create an encrypted store at a fixture root.
 
 @param root - fixture root
 
 @param identityPath - generated age identity path
 
 @param values - synthetic values to encrypt
 
 @returns store path and the environment that decrypts it
 
 @throws when sops encryption fails
 
 @example
 ```ts
 const { storePath } = await writeEncryptedStore({ root, identityPath, values: { A: 'x' } });
 ```
 */
async function writeEncryptedStore(
  {
    root,
    identityPath,
    values,
  }: {
    readonly root: string;
    readonly identityPath: string;
    readonly values: Readonly<Record<string, string>>;
  },
): Promise<{
  readonly storePath: string;
  readonly env: Readonly<Record<string, string>>;
}> {
  /**
   Local value for recipient.
   */
  const recipient = await readRecipient(identityPath,);
  /**
   Local value for plaintextPath.
   */
  const plaintextPath = join(
    root,
    'store.plain.json',
  );
  /**
   Local value for storePath.
   */
  const storePath = join(
    root,
    '.env.local.json',
  );
  await writeFile(
    plaintextPath,
    JSON.stringify(values,),
    'utf8',
  );
  await spawn(
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
    {
      stdout: 'ignore',
      stderr: 'ignore',
    },
  );
  await chmod(
    storePath,
    FIXTURE_FILE_MODE,
  );
  await rm(plaintextPath,);
  return {
    storePath,
    env: {
      PATH: process.env
        .PATH
        ?? '',
      HOME: root,
      SOPS_AGE_KEY_FILE: identityPath,
    },
  };
}

/**
 Write a plaintext JSON store without sops metadata.
 
 @param root - fixture root
 
 @param values - synthetic values to write
 
 @returns store path
 
 @example
 ```ts
 const storePath = await writePlaintextStore({ root, values: { A: 'x' } });
 ```
 */
async function writePlaintextStore(
  {
    root,
    values,
  }: {
    readonly root: string;
    readonly values: Readonly<Record<string, string>>;
  },
): Promise<string> {
  /**
   Local value for storePath.
   */
  const storePath = join(
    root,
    '.env.local.json',
  );
  await writeFile(
    storePath,
    JSON.stringify(values,),
    'utf8',
  );
  return storePath;
}

export {
  createIdentityFixture,
  PLAINTEXT_SENTINEL,
  writeEncryptedStore,
  writePlaintextStore,
};
