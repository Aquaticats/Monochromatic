/**
 Shared test fixture tests: temp directories,
 config file round-trips,
 disposable homes,
 and the encryption fixtures used by store tests.
 
 @module
 */

import {
  existsSync,
  readFileSync,
  readdirSync,
  writeFileSync,
} from 'node:fs';
import path from 'node:path';
import process from 'node:process';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  decryptConfigData,
  encryptSerializedStore,
} from '../dist/final/neutral/index.mjs';

import {
  createDisposableHome,
  createTempDirectory,
  decryptConfigFile,
  readMigrationVersion,
  restoreHome,
  writeConfigFile,
  writeLegacyEncryptedConfigFile,
} from './test-support.ts';

/**
 Encryption key shared by the writer and reader fixtures under test.
 */
const ENCRYPTION_KEY = 'unit-test-encryption-key';

await describe({
  name: 'test fixtures',
  children: [
    it({
      name: 'creates an existing empty directory',
      fn: async () => {
        /**
         Disposable directory asserted on below.
         */
        const directory = createTempDirectory();

        expect(existsSync(directory,),).toBe(true,);
        expect(readdirSync(directory,),).toEqual([],);
      },
    },),

    it({
      name: 'returns a distinct directory path per call',
      fn: async () => {
        /**
         Directory minted by the first of two calls.
         */
        const first = createTempDirectory();
        /**
         Directory minted by the second of two calls.
         */
        const second = createTempDirectory();

        expect(first === second,).toBe(false,);
      },
    },),

    it({
      name: 'round-trips config data as recorded migration bookkeeping',
      fn: async () => {
        /**
         Fixture directory holding the config file under test.
         */
        const directory = createTempDirectory();
        /**
         Absolute path of the default config file name.
         */
        const configPath = path.join(directory, 'config.json',);
        writeConfigFile({
          directory,
          data: {
            theme: 'dark',
            __internal__: {
              migrations: {
                version: '1.2.3',
              },
            },
          },
        },);

        expect(readMigrationVersion({ configPath, },),).toBe('1.2.3',);
      },
    },),

    it({
      name: 'reports no migration version without bookkeeping',
      fn: async () => {
        /**
         Fixture directory holding the config file under test.
         */
        const directory = createTempDirectory();
        /**
         Absolute path of the default config file name.
         */
        const configPath = path.join(directory, 'config.json',);
        writeConfigFile({
          directory,
          data: {
            theme: 'dark',
          },
        },);

        expect(readMigrationVersion({ configPath, },),).toBeUndefined();
      },
    },),

    it({
      name: 'ignores non-string recorded migration versions',
      fn: async () => {
        /**
         Fixture directory holding the config file under test.
         */
        const directory = createTempDirectory();
        /**
         Absolute path of the default config file name.
         */
        const configPath = path.join(directory, 'config.json',);
        writeConfigFile({
          directory,
          data: {
            __internal__: {
              migrations: {
                version: 42,
              },
            },
          },
        },);

        expect(readMigrationVersion({ configPath, },),).toBeUndefined();
      },
    },),

    it({
      name: 'swaps HOME and XDG_CONFIG_HOME to the disposable home',
      fn: async () => {
        /**
         Injected home whose environment swap is asserted below.
         */
        const handle = createDisposableHome();

        expect(process.env.HOME,).toBe(handle.home,);
        expect(process.env.XDG_CONFIG_HOME,).toBe(path.join(handle.home, '.config',),);
        expect(existsSync(handle.home,),).toBe(true,);

        restoreHome(handle,);
      },
    },),

    it({
      name: 'restores prior environment values after injection',
      fn: async () => {
        /**
         Real HOME present before this test's fixture work,
         put back at the end.
         */
        const realHome = process.env.HOME;
        /**
         Real XDG root present before this test's fixture work.
         */
        const realXdg = process.env.XDG_CONFIG_HOME;
        /**
         Known prior home the injection must hand back on restore.
         */
        const priorHome = createTempDirectory();
        /**
         Known prior XDG root the injection must hand back on restore.
         */
        const priorXdg = createTempDirectory();
        process.env.HOME = priorHome;
        process.env.XDG_CONFIG_HOME = priorXdg;
        /**
         Injected home recording the known prior values.
         */
        const handle = createDisposableHome();

        expect(handle.previousValues.HOME,).toBe(priorHome,);
        expect(handle.previousValues.XDG_CONFIG_HOME,).toBe(priorXdg,);

        restoreHome(handle,);

        expect(process.env.HOME,).toBe(priorHome,);
        expect(process.env.XDG_CONFIG_HOME,).toBe(priorXdg,);

        if (realHome === undefined)
          Reflect.deleteProperty(process.env, 'HOME',);
        else
          process.env.HOME = realHome;
        if (realXdg === undefined)
          Reflect.deleteProperty(process.env, 'XDG_CONFIG_HOME',);
        else
          process.env.XDG_CONFIG_HOME = realXdg;
      },
    },),

    it({
      name: 'restores environment variables that were absent before injection',
      fn: async () => {
        /**
         Real HOME present before this test's fixture work,
         put back at the end.
         */
        const realHome = process.env.HOME;
        /**
         Real XDG root present before this test's fixture work.
         */
        const realXdg = process.env.XDG_CONFIG_HOME;
        Reflect.deleteProperty(process.env, 'HOME',);
        Reflect.deleteProperty(process.env, 'XDG_CONFIG_HOME',);
        /**
         Injected home recording both names as previously absent.
         */
        const handle = createDisposableHome();

        expect(handle.previousAbsent,).toContain('HOME',);
        expect(handle.previousAbsent,).toContain('XDG_CONFIG_HOME',);

        restoreHome(handle,);

        expect(process.env.HOME,).toBeUndefined();
        expect(process.env.XDG_CONFIG_HOME,).toBeUndefined();

        if (realHome === undefined)
          Reflect.deleteProperty(process.env, 'HOME',);
        else
          process.env.HOME = realHome;
        if (realXdg === undefined)
          Reflect.deleteProperty(process.env, 'XDG_CONFIG_HOME',);
        else
          process.env.XDG_CONFIG_HOME = realXdg;
      },
    },),

    it({
      name: 'decrypts a config file written by encryptSerializedStore with aes-256-cbc',
      fn: async () => {
        /**
         Fixture directory holding the encrypted file under test.
         */
        const directory = createTempDirectory();
        /**
         Absolute path of the encrypted file under test.
         */
        const filePath = path.join(directory, 'config.json',);
        /**
         Serialized store text protected on disk.
         */
        const serialized = '{"theme":"dark"}';
        writeFileSync(filePath, encryptSerializedStore({
          serialized,
          encryptionKey: ENCRYPTION_KEY,
          encryptionAlgorithm: 'aes-256-cbc',
        },),);

        expect(decryptConfigFile({
          filePath,
          encryptionKey: ENCRYPTION_KEY,
          encryptionAlgorithm: 'aes-256-cbc',
        },),).toBe(serialized,);
      },
    },),

    it({
      name: 'decrypts a config file written by encryptSerializedStore with aes-256-gcm',
      fn: async () => {
        /**
         Fixture directory holding the encrypted file under test.
         */
        const directory = createTempDirectory();
        /**
         Absolute path of the encrypted file under test.
         */
        const filePath = path.join(directory, 'config.json',);
        /**
         Serialized store text protected on disk.
         */
        const serialized = '{"theme":"dark"}';
        writeFileSync(filePath, encryptSerializedStore({
          serialized,
          encryptionKey: ENCRYPTION_KEY,
          encryptionAlgorithm: 'aes-256-gcm',
        },),);

        expect(decryptConfigFile({
          filePath,
          encryptionKey: ENCRYPTION_KEY,
          encryptionAlgorithm: 'aes-256-gcm',
        },),).toBe(serialized,);
      },
    },),

    it({
      name: 'writes legacy salt layouts that decryptConfigData reads via the legacy fallback',
      fn: async () => {
        /**
         Fixture directory holding the legacy file under test.
         */
        const directory = createTempDirectory();
        /**
         Absolute path of the legacy file under test.
         */
        const filePath = path.join(directory, 'legacy.json',);
        /**
         Serialized store text protected under the legacy salt layout.
         */
        const serialized = '{"theme":"legacy"}';
        writeLegacyEncryptedConfigFile({
          filePath,
          encryptionKey: ENCRYPTION_KEY,
          serialized,
        },);
        /**
         Raw bytes of the legacy-layout file,
         fed to the store's own reader below.
         */
        const data = new Uint8Array(readFileSync(filePath,),);

        expect(decryptConfigData({
          data,
          encryptionKey: ENCRYPTION_KEY,
          encryptionAlgorithm: 'aes-256-cbc',
        },),).toBe(serialized,);
      },
    },),
  ],
},);
