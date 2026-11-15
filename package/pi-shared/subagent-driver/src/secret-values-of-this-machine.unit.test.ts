/**
 What reading the values of this machine gives: each source by its own rules, the store through the reader that owns it, and a
 store that cannot be read stopping the read rather than passing for a machine with nothing to mask.

 Every value here is a stand-in written into a disposable folder with a key made for it. No repository store, credential file or
 environment of this machine is read, and no value this test holds is written to a log.

 @module
 */

import {
  chmod,
  mkdir,
  mkdtemp,
  rm,
  writeFile,
  constants,
  open
} from 'node:fs/promises';
import { join, } from 'node:path';
import { createSopsFixture, } from '@monochromatic-dev/env-local-json-core/ts/fixture.ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { spawnSync, } from 'node:child_process';
import { setTimeout as wait, } from 'node:timers/promises';
import {
  readSecretValuesOfThisMachine,
  SecretEnvironmentUnreadableError,
  type SecretValuesOfThisMachine,
} from '../dist/final/node/index.mjs';

/**
 A stand-in value of the disposable store, long enough to be masked.
 */
const STORE_VALUE = 'stand-in-store-value-0123456789';

/**
 The name the store declares for it.
 */
const STORE_NAME = 'STAND_IN_API_KEY';

/**
 A stand-in provider key of Pi's own file.
 */
const PI_KEY_VALUE = 'stand-in-provider-key-abcdefghij';

/**
 A stand-in provider key of Pi's other file, which must differ from the first so both names are kept.
 */
const PI_MODEL_KEY_VALUE = 'stand-in-model-key-uvwxyzabcd';

/**
 Opens a named pipe for writing, which is what a reader waiting on one needs.

 @param path - the named pipe

 @returns the writer, or nothing when no reader is waiting for one
 */
async function openForWriting({ path, }: { readonly path: string },) {
  try {
    return await open(
      path,
      constants.O_WRONLY | constants.O_NONBLOCK,
    );
  }
  catch (error) {
    if (!Error.isError(error,)) {
      throw error;
    }
    return undefined;
  }
}

/**
 Gives a named pipe a writer when the test is done with it, so a read that was waiting on one ends and the test process can go.

 Without this, a read that waits on a pipe with no writer keeps the whole file alive after the test has failed.

 @param path - the named pipe

 @returns a disposer that opens the pipe for writing and closes it
 */
async function aPipeThatReleases({ path, }: { readonly path: string },): Promise<{
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  return {
    async [Symbol.asyncDispose]() {
      /**
       A writer, which unblocks whoever is waiting to read.
       */
      const writer = await openForWriting({ path, },);
      await writer?.close();
    },
  };
}

/**
 How long a source read may take in this file before the test calls it a wait, which is what a read of a named pipe would be.
 */
const SOURCE_READ_BOUND_MILLISECONDS = 10_000;

/**
 A stand-in OAuth access token of Pi's own credential file.
 */
const OAUTH_ACCESS_VALUE = 'stand-in-oauth-access-0123456789';

/**
 A stand-in OAuth refresh token of Pi's own credential file.
 */
const OAUTH_REFRESH_VALUE = 'stand-in-oauth-refresh-9876543210';

/**
 A stand-in provider key that begins with a slash, which is a key and not a path.
 */
const SLASH_KEY_VALUE = '/stand-in-provider-key-abcdefghij';

/**
 A stand-in token of the shell start-up file.
 */
const SHELL_VALUE = 'stand-in-shell-token-klmnopqrst';

/**
 The name the shell start-up file exports it as.
 */
const SHELL_NAME = 'STAND_IN_GITHUB_TOKEN';

/**
 A stand-in variable of the driver's own environment.
 */
const ENVIRONMENT_VALUE = 'stand-in-environment-token-0123456789';

/**
 The name that variable has.
 */
const ENVIRONMENT_NAME = 'STAND_IN_ENV_TOKEN';

/**
 An age identity file that opens nothing: a key of the right shape that the store was not encrypted to.

 It is built from parts so the file holds no text any scanner would take for a key, and it is not one: no store anywhere is
 encrypted to it, and this machine has none.
 */
const NOT_A_KEY_FOR_THIS_STORE: string = `${['AGE', 'SECRET', 'KEY',].join('-',)}-1${'Q'.repeat(58,)}`.concat(String.fromCodePoint(10,),);

/**
 A folder this test makes, removed when it is done.

 @param prefix - what the folder holds, so a reader of the scratch root knows which test left it

 @returns the folder's path and its disposer
 */
async function makeFolder({ prefix, }: { readonly prefix: string },): Promise<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  /**
   The folder, under the scratch root the test harness names rather than the system's temporary directory.
   */
  const path = await mkdtemp(join(
    scratchRoot(),
    prefix,
  ),);
  await chmod(
    path,
    0o700,
  );
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(
        path,
        { recursive: true, force: true, },
      );
    },
  };
}

/**
 Writes the files a home folder holds for the sources this package reads: Pi's two credential files and a shell start-up file.

 @param home - the stand-in home folder

 @param providerKey - the stand-in provider key Pi's own file declares

 @param shellToken - the stand-in token the shell start-up file exports
 */
async function writeStandInHome({
  home,
  providerKey,
  modelKey,
  shellToken,
}: {
  readonly home: string;
  readonly providerKey: string;
  readonly modelKey: string;
  readonly shellToken: string;
},): Promise<void> {
  /**
   Pi's own folder, which holds the two files that name a provider key.
   */
  const agentFolder = join(
    home,
    '.pi',
    'agent',
  );
  await mkdir(
    agentFolder,
    { recursive: true, },
  );
  await writeFile(
    join(agentFolder, 'auth.json',),
    JSON.stringify({ example: { type: 'api_key', key: providerKey, }, },),
  );
  await writeFile(
    join(agentFolder, 'models.json',),
    JSON.stringify({ providers: { example: { apiKey: modelKey, }, }, },),
  );
  await writeFile(
    join(home, '.bashrc',),
    `export ${SHELL_NAME}=${shellToken}\n# a comment\nalias ll='ls -l'\n`,
  );
}

/**
 Says whether any value the read returned stands in a text, which is how a test looks for a leak without printing one.

 @param read - what reading the values gave

 @param text - the text to look through

 @returns whether a value stands in it
 */
function holdsAValue(read: SecretValuesOfThisMachine, text: string,): boolean {
  return read.values.some(function standsIn(value,) {
    return text.includes(value.value,);
  },);
}

await describe({
  name: 'the values of this machine',
  children: [
    it({
      name: 'reads the store, Pi own files, the shell start-up file and the environment, naming each source',
      fn: async () => {
        await using store = await createSopsFixture({ values: { [STORE_NAME]: STORE_VALUE, }, },);
        await using home = await makeFolder({ prefix: 'pi-secret-values-home-', },);
        await writeStandInHome({
          home: home.path,
          providerKey: PI_KEY_VALUE,
          modelKey: PI_MODEL_KEY_VALUE,
          shellToken: SHELL_VALUE,
        },);
        /**
         Every value the read found, by the name a marker would give it.
         */
        const read = await readSecretValuesOfThisMachine({
          environment: {
            // The reader resolves sops on the curated PATH it is given, which a real driver inherits from its own shell.
            PATH: process.env.PATH ?? '',
            ENV_LOCAL_JSON_PATH: store.storePath,
            SOPS_AGE_KEY_FILE: store.identityPath,
            [ENVIRONMENT_NAME]: ENVIRONMENT_VALUE,
          },
          home: home.path,
          checkoutFolder: home.path,
        },);
        /**
         The names the read found.
         */
        const names = read.values.map(function nameOf(value,) {
          return value.name;
        },);
        expect(names,).toContain(STORE_NAME,);
        expect(names,).toContain('auth.json example.key',);
        expect(names,).toContain('models.json providers.example.apiKey',);
        expect(names,).toContain(`.bashrc ${SHELL_NAME}`,);
        expect(names,).toContain(ENVIRONMENT_NAME,);
        // Every source reported is a name and a count, and no value stands in what it says.
        expect(holdsAValue(read, JSON.stringify(read.sources,),),).toBe(false,);
        expect(read.unread,).toEqual([],);
      },
    },),
    it({
      name: 'stops the run, naming the cause, when the store cannot be decrypted',
      fn: async () => {
        await using store = await createSopsFixture({ values: { [STORE_NAME]: STORE_VALUE, }, },);
        await using otherIdentity = await makeFolder({ prefix: 'pi-secret-values-other-key-', },);
        /**
         A second identity file, which is not the one the store was encrypted to.
         */
        const otherKey = join(otherIdentity.path, 'identity.txt',);
        await writeFile(
          otherKey,
          NOT_A_KEY_FOR_THIS_STORE,
        );
        await using home = await makeFolder({ prefix: 'pi-secret-values-refused-home-', },);
        /**
         The read that must stop the run.
         */
        const read = readSecretValuesOfThisMachine({
          environment: {
            PATH: process.env.PATH ?? '',
            ENV_LOCAL_JSON_PATH: store.storePath,
            SOPS_AGE_KEY_FILE: otherKey,
          },
          home: home.path,
          checkoutFolder: home.path,
        },);
        await expect(read,).rejects.toBeInstanceOf(SecretEnvironmentUnreadableError,);
        /**
         The message the run would say, which may not hold a value.
         */
        const said: { message: string } = { message: '', };
        try {
          await read;
        }
        catch (error) {
          said.message = String(error,);
        }
        expect(said.message,).not.toContain(STORE_VALUE,);
        // The cause names what a reader has to fix, and decryption is what failed rather than a missing tool.
        expect(said.message.toLowerCase(),).toContain('decrypt',);
      },
    },),
    it({
      name: 'arms an OAuth credential Pi own schema names, a shell value with a comment after it, and a key that begins with a slash',
      fn: async () => {
        await using store = await createSopsFixture({ values: { [STORE_NAME]: STORE_VALUE, }, },);
        await using home = await makeFolder({ prefix: 'pi-secret-values-schema-home-', },);
        /**
         Pi's own agent folder, holding a credential in the shape Pi's schema documents.
         */
        const agentFolder = join(
          home.path,
          '.pi',
          'agent',
        );
        await mkdir(
          agentFolder,
          { recursive: true, },
        );
        await writeFile(
          join(agentFolder, 'auth.json',),
          JSON.stringify({
            example: {
              type: 'oauth',
              access: OAUTH_ACCESS_VALUE,
              refresh: OAUTH_REFRESH_VALUE,
              expires: 0,
            },
          },),
        );
        await writeFile(
          join(agentFolder, 'models.json',),
          JSON.stringify({ providers: { example: { apiKey: SLASH_KEY_VALUE, }, }, },),
        );
        await writeFile(
          join(home.path, '.bashrc',),
          `export ${SHELL_NAME}="${SHELL_VALUE}" # an ordinary inline comment\n`,
        );
        /**
         What reading found, through the sources a run reads.
         */
        const read = await readSecretValuesOfThisMachine({
          environment: {
            PATH: process.env.PATH ?? '',
            ENV_LOCAL_JSON_PATH: store.storePath,
            SOPS_AGE_KEY_FILE: store.identityPath,
          },
          home: home.path,
          checkoutFolder: home.path,
        },);
        /**
         The values, by the names a marker gives them.
         */
        const byName = new Map(read.values.map(function toEntry(value,) {
          return [value.name, value.value,] as const;
        },),);
        expect(byName.get('auth.json example.access',),).toBe(OAUTH_ACCESS_VALUE,);
        expect(byName.get('auth.json example.refresh',),).toBe(OAUTH_REFRESH_VALUE,);
        expect(byName.get(`models.json providers.example.apiKey`,),).toBe(SLASH_KEY_VALUE,);
        // The shell exports the quoted literal and not the comment after it.
        expect(byName.get(`.bashrc ${SHELL_NAME}`,),).toBe(SHELL_VALUE,);
      },
    },),
    it({
      name: 'stops the run when Pi own credential file is there and cannot be read',
      fn: async () => {
        await using home = await makeFolder({ prefix: 'pi-secret-values-malformed-home-', },);
        /**
         Pi's own agent folder, holding a file that is not JSON.
         */
        const agentFolder = join(
          home.path,
          '.pi',
          'agent',
        );
        await mkdir(
          agentFolder,
          { recursive: true, },
        );
        await writeFile(
          join(agentFolder, 'auth.json',),
          `{"example":{"key":"${PI_KEY_VALUE}"`,
        );
        /**
         The read that must stop the run.
         */
        const read = readSecretValuesOfThisMachine({
          environment: { PATH: process.env.PATH ?? '', },
          home: home.path,
          checkoutFolder: home.path,
        },);
        await expect(read,).rejects.toBeInstanceOf(SecretEnvironmentUnreadableError,);
        /**
         The message the run would say, which names the file and holds no value.
         */
        const said: { message: string } = { message: '', };
        try {
          await read;
        }
        catch (error) {
          said.message = String(error,);
        }
        expect(said.message,).toContain('auth.json',);
        expect(said.message,).not.toContain(PI_KEY_VALUE,);
      },
    },),
    it({
      name: 'refuses a named pipe at a source path instead of waiting on it',
      fn: async () => {
        await using home = await makeFolder({ prefix: 'pi-secret-values-fifo-home-', },);
        /**
         Pi's own agent folder, whose credential file is a named pipe with no writer.
         */
        const agentFolder = join(
          home.path,
          '.pi',
          'agent',
        );
        await mkdir(
          agentFolder,
          { recursive: true, },
        );
        /**
         The program that makes a named pipe.
         */
        const made = spawnSync(
          'mkfifo',
          [join(agentFolder, 'auth.json',),],
          { encoding: 'utf8', },
        );
        expect(made.status,).toBe(0,);
        /**
         The read, which must answer rather than wait for a writer.
         */
        const read = readSecretValuesOfThisMachine({
          environment: { PATH: process.env.PATH ?? '', },
          home: home.path,
          checkoutFolder: home.path,
        },);
        await using release = await aPipeThatReleases({ path: join(agentFolder, 'auth.json',), },);
        void release;
        /**
         What the read gave before this test's bound: its refusal, or nothing when it was still waiting. A read that waits for a
         writer shows as a failure here rather than holding the suite open.
         */
        const outcome: { refusal?: unknown } = {};
        try {
          await Promise.race([
            read,
            wait(
              SOURCE_READ_BOUND_MILLISECONDS,
              'still-waiting' as const,
              { ref: false, },
            ),
          ],);
        }
        catch (error) {
          outcome.refusal = error;
        }
        expect(outcome.refusal,).toBeInstanceOf(SecretEnvironmentUnreadableError,);
      },
    },),
    it({
      name: 'stops the run when an age identity the machine declares cannot be read',
      fn: async () => {
        await using home = await makeFolder({ prefix: 'pi-secret-values-identity-home-', },);
        /**
         A folder where the identity file should be, which is there and is not an ordinary file.
         */
        const identityFolder = join(home.path, 'identity-is-a-directory',);
        await mkdir(
          identityFolder,
          { recursive: true, },
        );
        /**
         The read that must stop the run.
         */
        const read = readSecretValuesOfThisMachine({
          environment: {
            PATH: process.env.PATH ?? '',
            SOPS_AGE_KEY_FILE: identityFolder,
          },
          home: home.path,
          checkoutFolder: home.path,
        },);
        await expect(read,).rejects.toBeInstanceOf(SecretEnvironmentUnreadableError,);
        /**
         The message the run would say, which names what could not be read.
         */
        const said: { message: string } = { message: '', };
        try {
          await read;
        }
        catch (error) {
          said.message = String(error,);
        }
        expect(said.message,).toContain('age identity',);
      },
    },),
    it({
      name: 'arms the literal a shell start-up file exports with a hash at its start, and stops the run on a value it cannot give',
      fn: async () => {
        await using home = await makeFolder({ prefix: 'pi-secret-values-hash-home-', },);
        /**
         A start-up file whose credential is a literal beginning with a hash, which is what bash exports there.
         */
        await writeFile(
          join(home.path, '.bashrc',),
          `export ${SHELL_NAME}=#a-literal-value-with-a-hash\n`,
        );
        /**
         What the read found.
         */
        const read = await readSecretValuesOfThisMachine({
          environment: { PATH: process.env.PATH ?? '', },
          home: home.path,
          checkoutFolder: home.path,
        },);
        expect(read.values.map(function nameOf(value,) {
          return value.value;
        },),).toContain('#a-literal-value-with-a-hash',);
        // A value the shell computes is one this code cannot give, and a run that started without it would mask nothing of it.
        await writeFile(
          join(home.path, '.bashrc',),
          `export ${SHELL_NAME}=$(printf %s a-computed-value)\n`,
        );
        await expect(readSecretValuesOfThisMachine({
          environment: { PATH: process.env.PATH ?? '', },
          home: home.path,
          checkoutFolder: home.path,
        },),).rejects.toBeInstanceOf(SecretEnvironmentUnreadableError,);
      },
    },),
    it({
      name: 'reads no store from a checkout that declares none, and still reads the other sources',
      fn: async () => {
        await using home = await makeFolder({ prefix: 'pi-secret-values-no-store-', },);
        await writeStandInHome({
          home: home.path,
          providerKey: PI_KEY_VALUE,
          modelKey: PI_MODEL_KEY_VALUE,
          shellToken: SHELL_VALUE,
        },);
        /**
         What a checkout with no store gives.
         */
        const read = await readSecretValuesOfThisMachine({
          environment: {
            PATH: process.env.PATH ?? '',
            [ENVIRONMENT_NAME]: ENVIRONMENT_VALUE,
          },
          home: home.path,
          checkoutFolder: home.path,
        },);
        /**
         The names the read found.
         */
        const names = read.values.map(function nameOf(value,) {
          return value.name;
        },);
        expect(names,).not.toContain(STORE_NAME,);
        expect(names,).toContain(ENVIRONMENT_NAME,);
        expect(read.sources.map(function sourceOf(source,) {
          return source.source;
        },),).not.toContain(
          `the .env.local.json of ${home.path}`,
        );
      },
    },),
  ],
},);
