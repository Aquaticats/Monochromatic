/**
 Real sops/age fixtures for the env-local-json tests.

 Every fixture lives in a disposable `mktemp` directory with a freshly
 generated age identity. No repository store, credential, or committed key
 material is read or written. A missing `sops` or `age-keygen` fails with the
 install command to run.

 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  chmod,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  join,
} from 'node:path';
import { randomUUID, } from 'node:crypto';

import {
  caughtValueText,
} from '@monochromatic-dev/caught-value-core/ts';

//region Constants

/**
 Prefix for every freshly generated fixture directory.
 */
const FIXTURE_PREFIX = 'env-local-json-test-';

/**
 File name of the encrypted store inside a fixture root.
 */
const STORE_FILE_NAME = '.env.local.json';

/**
 File name of the generated age identity inside a fixture root.
 */
const IDENTITY_FILE_NAME = 'identity.txt';

/**
 Marker preceding the public key in an age identity file.
 */
const PUBLIC_KEY_MARKER = 'public key: ';

/**
 Marker starting the comment line that holds the public key.
 */
const PUBLIC_KEY_COMMENT_START = '#';

/**
 Install command named by preflight failures.
 */
const TOOLS_REMEDIATION =
  "sops and age-keygen must be on PATH; install them with 'mise install sops age' and retry";

//endregion Constants

//region Types

/**
 Disposable real-toolchain fixture.
 */
export type SopsFixture = {
  /**
   Disposable fixture root.
   */
  readonly root: string;

  /**
   Path of the generated age identity.
   */
  readonly identityPath: string;

  /**
   Public age recipient matching the identity.
   */
  readonly recipient: string;

  /**
   Path of the default encrypted store.
   */
  readonly storePath: string;

  /**
   Values encrypted into the default store.
   */
  readonly values: Readonly<Record<string, string>>;

  /**
   Removes the fixture tree after the test.
   */
  [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Disposable temporary directory owned by one test.
 */
export type TempRoot = {
  /**
   Absolute temporary directory path.
   */
  readonly root: string;

  /**
   Removes the directory tree after the test.
   */
  [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Captured result of one fixture subprocess.
 */
type CommandResult = {
  /**
   Accumulated standard output.
   */
  readonly stdout: string;
};

/**
 Options for {@link runCommand}.
 */
type RunCommandOptions = {
  /**
   Executable to run.
   */
  readonly command: string;

  /**
   Arguments passed without a shell.
   */
  readonly args: readonly string[];

  /**
   Extra environment values layered over the test process environment.
   */
  readonly extraEnv?: Readonly<Record<string, string>>;
};

//endregion Types

//region Command execution

/**
 Runs one real tool and captures its standard output.

 @param command - executable to run

 @param args - arguments passed without a shell

 @param extraEnv - extra environment values layered over the process environment

 @returns captured standard output

 @throws Error naming the install command when the tool is missing

 @throws Error when the tool exits non-zero; the message names the tool and
 status only, never its output

 @example
 ```ts
 const result = await runCommand({ command: 'age-keygen', args: ['--version'] });
 ```
 */
async function runCommand({
  command,
  args,
  extraEnv,
}: RunCommandOptions,): Promise<CommandResult> {
  /**
   Output chunks captured from the child process.
   */
  const captured: string[] = [];
  try {
    /**
     Tool child process with captured standard output.
     */
    const child = spawn(
      command,
      [...args,],
      {
        env: {
          ...process.env,
          ...extraEnv,
        },
        stdio: [
          'ignore',
          'pipe',
          'ignore',
        ],
      },
    );
    child.stdout
      .setEncoding('utf8',);
    child.stdout
      .on(
      'data',
      function captureChunk(chunk: string,): void {
        captured.push(chunk,);
      },
      );
    /**
     Raw exit event of the tool process.
     */
    const exitEvent: unknown = await once(
      child,
      'exit',
    );
    /**
     Reported exit code from the event tuple.
     */
    const code: unknown = Array.isArray(exitEvent,) ? exitEvent[0] : undefined;
    if ((typeof code) !== 'number')
      throw new Error(`${command} did not exit with a status`,);
    if (code !== 0)
      throw new Error(`${command} exited with status ${String(code,)}`,);
  }
  catch (error) {
    throw new Error(
      `${command} failed: ${caughtValueText(error,)}. ${TOOLS_REMEDIATION}`,
      { cause: error, },
    );
  }
  return { stdout: captured.join('',), };
}

//endregion Command execution

//region Fixture helpers

/**
 Extracts the age public key from an identity file's comment line.

 @param identityText - full text of the generated identity file

 @returns age recipient string

 @throws Error when the identity file has no public-key comment

 @example
 ```ts
 const recipient = publicKeyFromIdentity({ identityText });
 ```
 */
function publicKeyFromIdentity({
  identityText,
}: {
  readonly identityText: string;
},): string {
  /**
   Comment line holding the public key, or the whole file when absent.
   */
  const lines = identityText.split('\n',);
  for (const line of lines) {
    if (!line.startsWith(PUBLIC_KEY_COMMENT_START,))
      continue;
    /**
     Offset of the public-key marker on this comment line.
     */
    const markerStart = line.indexOf(PUBLIC_KEY_MARKER,);
    if (markerStart === (-1))
      continue;
    return line.slice(markerStart + PUBLIC_KEY_MARKER.length,)
      .trim();
  }
  throw new Error('generated age identity has no public-key comment',);
}

/**
 Creates one disposable temporary directory.

 @param prefix - directory-name prefix

 @returns disposable temporary root

 @example
 ```ts
 await using temp = await createTempRoot({ prefix: 'env-local-json-' });
 ```
 */
export async function createTempRoot({
  prefix,
}: {
  readonly prefix?: string;
} = {},): Promise<TempRoot> {
  /**
   Fresh temporary directory.
   */
  const root = await mkdtemp(join(
    scratchRoot(),
    prefix ?? FIXTURE_PREFIX,
  ),);
  await chmod(
    root,
    0o700,
  );
  return {
    root,
    [Symbol.asyncDispose]: async function removeRoot(): Promise<void> {
      await rm(
        root,
        {
          force: true,
          recursive: true,
        },
      );
    },
  };
}

/**
 Encrypts one JSON document with the fixture's age recipient.

 @param fixture - fixture owning the recipient and root

 @param json - plaintext JSON document

 @param fileName - store file name inside the fixture root

 @returns path of the encrypted store

 @example
 ```ts
 const storePath = await encryptJson({ fixture, json: '{"A":"x"}', fileName: 'store.json' });
 ```
 */
export async function encryptJson({
  fixture,
  json,
  fileName,
}: {
  readonly fixture: SopsFixture;
  readonly json: string;
  readonly fileName: string;
},): Promise<string> {
  /**
   Temporary plaintext file removed after encryption.
   */
  const plaintextPath = join(
    fixture.root,
    `plaintext-${randomUUID()}.json`,
  );
  /**
   Encrypted output path.
   */
  const storePath = join(
    fixture.root,
    fileName,
  );
  await writeFile(
    plaintextPath,
    json,
    'utf8',
  );
  /**
   Disposable guard that removes the plaintext file on every exit path.
   */
  await using _plaintext = {
    [Symbol.asyncDispose]: async function removePlaintext(): Promise<void> {
      await rm(
        plaintextPath,
        { force: true, },
      );
    },
  };
  /**
   Encryption result carrying the ciphertext on stdout.
   */
  const result = await runCommand({
    args: [
      '--encrypt',
      '--age',
      fixture.recipient,
      '--input-type',
      'json',
      '--output-type',
      'json',
      plaintextPath,
    ],
    command: 'sops',
  },);
  await writeFile(
    storePath,
    result.stdout,
    {
      encoding: 'utf8',
      mode: 0o600,
    },
  );
  return storePath;
}

/**
 Decrypts an encrypted store with the fixture identity.

 @param fixture - fixture owning the identity

 @param storePath - encrypted store path

 @returns validated top-level string values

 @throws Error when the store is not a JSON object of strings

 @example
 ```ts
 const values = await decryptStore({ fixture, storePath: fixture.storePath });
 ```
 */
export async function decryptStore({
  fixture,
  storePath,
}: {
  readonly fixture: SopsFixture;
  readonly storePath: string;
},): Promise<Readonly<Record<string, string>>> {
  /**
   Decryption result carrying plaintext JSON on stdout.
   */
  const result = await runCommand({
    args: [
      '--decrypt',
      '--output-type',
      'json',
      storePath,
    ],
    command: 'sops',
    extraEnv: { SOPS_AGE_KEY_FILE: fixture.identityPath, },
  },);
  /**
   Parsed decrypted document.
   */
  const parsed: unknown = JSON.parse(
    result.stdout,
  );
  if (((typeof parsed) !== 'object') || (parsed === null)
    || Array.isArray(parsed,)) {
    throw new Error('decrypted fixture store is not a JSON object',);
  }
  /**
   Validated string values collected from the document.
   */
  const values: Record<string, string> = {};
  for (const [key, value,] of Object.entries(parsed,)) {
    if ((typeof value) !== 'string')
      throw new Error(`decrypted fixture key ${key} is not a string`,);
    values[key] = value;
  }
  return values;
}

/**
 Creates a disposable fixture with a generated identity and encrypted store.

 @param values - values encrypted into the default store

 @returns disposable fixture

 @throws Error naming the install command when sops or age-keygen is absent

 @example
 ```ts
 await using fixture = await createSopsFixture({ values: { A: 'x' } });
 ```
 */
export async function createSopsFixture({
  values,
}: {
  readonly values: Readonly<Record<string, string>>;
},): Promise<SopsFixture> {
  /**
   Disposable fixture root.
   */
  const root = await mkdtemp(join(
    scratchRoot(),
    FIXTURE_PREFIX,
  ),);
  await chmod(
    root,
    0o700,
  );
  /**
   Generated age identity path.
   */
  const identityPath = join(
    root,
    IDENTITY_FILE_NAME,
  );
  await runCommand({
    args: [
      '--output',
      identityPath,
    ],
    command: 'age-keygen',
  },);
  await chmod(
    identityPath,
    0o600,
  );
  /**
   Generated identity text.
   */
  const identityText = await readFile(
    identityPath,
    'utf8',
  );
  /**
   Fixture under construction; the store path is filled in after encryption.
   */
  const fixture: SopsFixture = {
    identityPath,
    recipient: publicKeyFromIdentity({ identityText, },),
    root,
    storePath: join(
      root,
      STORE_FILE_NAME,
    ),
    values,
    [Symbol.asyncDispose]: async function removeFixture(): Promise<void> {
      await rm(
        root,
        {
          force: true,
          recursive: true,
        },
      );
    },
  };
  await encryptJson({
    fileName: STORE_FILE_NAME,
    fixture,
    json: JSON.stringify(values,),
  },);
  return fixture;
}

/**
 Generates an additional age identity inside a fixture root.

 @param fixture - fixture owning the root

 @param fileName - identity file name inside the fixture root

 @returns path of the generated identity

 @example
 ```ts
 const wrongIdentity = await createIdentityFile({ fixture, fileName: 'wrong.txt' });
 ```
 */
export async function createIdentityFile({
  fixture,
  fileName,
}: {
  readonly fixture: SopsFixture;
  readonly fileName: string;
},): Promise<string> {
  /**
   Generated identity path.
   */
  const identityPath = join(
    fixture.root,
    fileName,
  );
  await runCommand({
    args: [
      '--output',
      identityPath,
    ],
    command: 'age-keygen',
  },);
  await chmod(
    identityPath,
    0o600,
  );
  return identityPath;
}

/**
 Builds a reader environment that cannot see the developer's identity
 variables or real store configuration.

 @param home - fixture home directory

 @param extra - extra environment values to layer in

 @returns environment ready for `readEnvLocalJson`

 @example
 ```ts
 const env = readerEnvironment({ home: fixture.root });
 ```
 */
export function readerEnvironment({
  home,
  extra,
}: {
  readonly home: string;
  readonly extra?: Readonly<Record<string, string>>;
},): Readonly<Record<string, string>> {
  return {
    PATH: process.env
      .PATH
      ?? '',
    HOME: home,
    TMPDIR: scratchRoot(),
    ...extra,
  };
}

//endregion Fixture helpers
