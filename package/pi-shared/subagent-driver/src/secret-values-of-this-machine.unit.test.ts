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
} from 'node:fs/promises';
import { join, } from 'node:path';
import { createSopsFixture, } from '@monochromatic-dev/env-local-json-core/ts/fixture.ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
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
