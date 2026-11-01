/**
 Encryption behavior: on-disk layout,
 authenticated and unauthenticated algorithms,
 legacy salt files,
 corrupt-file triage,
 and plaintext upgrades.
 
 @module
 */

import {
  appendFileSync,
  readFileSync,
  writeFileSync,
} from 'node:fs';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createConf,
  DecryptionFailedError,
  SchemaViolationError,
} from '../dist/final/neutral/index.mjs';

import {
  createTempDirectory,
  decryptConfigFile,
  writeConfigFile,
  writeLegacyEncryptedConfigFile,
} from './test-support.ts';

/**
 Value stored under one key in every fixture below.
 */
const FIXTURE_VALUE = '🦄';

await describe({
  name: 'encryption',
  children: [
    it({
      name: 'writes a framed encrypted file instead of plaintext JSON and reads it back across instances',
      fn: async () => {
        /**
         Directory holding the encrypted config file.
         */
        const directory = createTempDirectory();
        /**
         Store encrypting under the default `aes-256-cbc` algorithm.
         */
        const conf = createConf({
          cwd: directory,
          encryptionKey: 'abc123',
        },);
        conf.set({
          key: 'foo',
          value: FIXTURE_VALUE,
        },);
        conf.set({
          key: 'baz.boo',
          value: FIXTURE_VALUE,
        },);

        /**
         Raw file text;
         it must not expose the store as plaintext JSON.
         */
        const fileText = readFileSync(
          conf.path,
          'utf8',
        );
        expect(fileText,).not.toContain('"foo"',);
        expect(fileText,).not.toContain(FIXTURE_VALUE,);

        /**
         File bytes used to check the framing layout.
         */
        const fileBytes = new Uint8Array(readFileSync(conf.path,),);
        expect(fileBytes[16],).toBe(':'.codePointAt(0),);

        /**
         Decrypted serialized store recovered with the store's wire format.
         */
        const decrypted = decryptConfigFile({
          filePath: conf.path,
          encryptionKey: 'abc123',
          encryptionAlgorithm: 'aes-256-cbc',
        },);
        const parsed = JSON.parse(decrypted,) as Record<string, unknown>;
        expect(parsed.foo,).toBe(FIXTURE_VALUE,);
        expect(parsed.baz,).toEqual({
          boo: FIXTURE_VALUE,
        },);

        /**
         Second store standing in for another process reading the file.
         */
        const reloaded = createConf({
          cwd: directory,
          encryptionKey: 'abc123',
        },);
        expect(reloaded.get('foo',),).toBe(FIXTURE_VALUE,);
        expect(reloaded.get('baz.boo',),).toBe(FIXTURE_VALUE,);
      },
    },),

    it({
      name: 'round-trips values through aes-256-gcm and decrypts them with the wire format',
      fn: async () => {
        /**
         Directory holding the authenticated encrypted config file.
         */
        const directory = createTempDirectory();
        /**
         Store encrypting under `aes-256-gcm`.
         */
        const conf = createConf({
          cwd: directory,
          encryptionKey: 'abc123',
          encryptionAlgorithm: 'aes-256-gcm',
        },);
        conf.set({
          key: 'foo',
          value: FIXTURE_VALUE,
        },);

        /**
         Decrypted serialized store recovered with the store's wire format.
         */
        const decrypted = decryptConfigFile({
          filePath: conf.path,
          encryptionKey: 'abc123',
          encryptionAlgorithm: 'aes-256-gcm',
        },);
        const parsed = JSON.parse(decrypted,) as Record<string, unknown>;
        expect(parsed.foo,).toBe(FIXTURE_VALUE,);

        /**
         Second store standing in for another process reading the file.
         */
        const reloaded = createConf({
          cwd: directory,
          encryptionKey: 'abc123',
          encryptionAlgorithm: 'aes-256-gcm',
        },);
        expect(reloaded.get('foo',),).toBe(FIXTURE_VALUE,);
      },
    },),

    it({
      name: 'round-trips an empty serialized payload through aes-256-gcm',
      fn: async () => {
        /**
         Directory whose serializer writes the empty store as an empty string.
         */
        const directory = createTempDirectory();
        /**
         Serializer writing the empty store as an empty payload.
         
         @param value - Store contents to serialize.
         
         @returns Serialized text,
         empty for the empty store.
         */
        const serialize = (value: Record<string, unknown>,): string =>
          Object.keys(value,)
            .length === 0
            ? ''
            : JSON.stringify(value,);
        /**
         Deserializer mapping the empty payload back to the empty store.
         
         @param text - Serialized text read from the file.
         
         @returns Parsed store contents.
         */
        const deserialize = (text: string,): Record<string, unknown> => {
          if (text === '')
            return {};
          return JSON.parse(text,) as Record<string, unknown>;
        };
        /**
         Store writing the empty payload under `aes-256-gcm`.
         */
        const conf = createConf({
          cwd: directory,
          encryptionKey: 'abc123',
          encryptionAlgorithm: 'aes-256-gcm',
          serialize,
          deserialize,
        },);
        conf.store = {};

        /**
         Decrypted payload of the written file,
         which the fixture deserializer maps to the empty store.
         */
        const decrypted = decryptConfigFile({
          filePath: conf.path,
          encryptionKey: 'abc123',
          encryptionAlgorithm: 'aes-256-gcm',
        },);
        expect(decrypted,).toBe('',);

        /**
         Second store standing in for another process reading the file.
         */
        const reloaded = createConf({
          cwd: directory,
          encryptionKey: 'abc123',
          encryptionAlgorithm: 'aes-256-gcm',
          serialize,
          deserialize,
        },);
        expect(reloaded.store,).toEqual({},);
      },
    },),

    it({
      name: 'round-trips values through aes-256-ctr and decrypts them with the wire format',
      fn: async () => {
        /**
         Directory holding the stream-ciphered config file.
         */
        const directory = createTempDirectory();
        /**
         Store encrypting under `aes-256-ctr`.
         */
        const conf = createConf({
          cwd: directory,
          encryptionKey: 'abc123',
          encryptionAlgorithm: 'aes-256-ctr',
        },);
        conf.set({
          key: 'foo',
          value: FIXTURE_VALUE,
        },);

        /**
         Decrypted serialized store recovered with the store's wire format.
         */
        const decrypted = decryptConfigFile({
          filePath: conf.path,
          encryptionKey: 'abc123',
          encryptionAlgorithm: 'aes-256-ctr',
        },);
        const parsed = JSON.parse(decrypted,) as Record<string, unknown>;
        expect(parsed.foo,).toBe(FIXTURE_VALUE,);

        /**
         Second store standing in for another process reading the file.
         */
        const reloaded = createConf({
          cwd: directory,
          encryptionKey: 'abc123',
          encryptionAlgorithm: 'aes-256-ctr',
        },);
        expect(reloaded.get('foo',),).toBe(FIXTURE_VALUE,);
      },
    },),

    it({
      name: 'reads a file written with the legacy initialization vector salt across instances',
      fn: async () => {
        /**
         Directory whose config file uses the legacy text-form salt layout.
         */
        const directory = createTempDirectory();
        writeLegacyEncryptedConfigFile({
          filePath: `${directory}/config.json`,
          encryptionKey: 'abc123',
          serialized: JSON.stringify({
            foo: FIXTURE_VALUE,
          },),
        },);

        /**
         Store that must fall back to the legacy salt derivation.
         */
        const conf = createConf({
          cwd: directory,
          encryptionKey: 'abc123',
        },);
        expect(conf.get('foo',),).toBe(FIXTURE_VALUE,);

        /**
         Second store standing in for another process reading the file.
         */
        const reloaded = createConf({
          cwd: directory,
          encryptionKey: 'abc123',
        },);
        expect(reloaded.get('foo',),).toBe(FIXTURE_VALUE,);
      },
    },),

    it({
      name: 'preserves a schema violation error raised while reading encrypted data',
      fn: async () => {
        /**
         Directory whose encrypted file holds a schema-invalid value.
         */
        const directory = createTempDirectory();
        /**
         Writer without a schema so the invalid value can be stored.
         */
        const writer = createConf({
          cwd: directory,
          encryptionKey: 'abc123',
        },);
        writer.store = {
          foo: 'not-boolean',
        };

        expect(function constructReaderWithSchema(): unknown {
          return createConf({
            cwd: directory,
            encryptionKey: 'abc123',
            schema: {
              foo: {
                type: 'boolean',
              },
            },
          },);
        },).toThrow(SchemaViolationError,);
        expect(function constructReaderWithSchema(): unknown {
          return createConf({
            cwd: directory,
            encryptionKey: 'abc123',
            schema: {
              foo: {
                type: 'boolean',
              },
            },
          },);
        },).toThrow('Config schema violation: `foo` must be boolean',);
      },
    },),

    it({
      name: 'throws DecryptionFailedError when a tampered aes-256-gcm file is read',
      fn: async () => {
        /**
         Directory holding an authenticated encrypted config file.
         */
        const directory = createTempDirectory();
        /**
         Store writing the file that will be tampered.
         */
        const conf = createConf({
          cwd: directory,
          encryptionKey: 'abc123',
          encryptionAlgorithm: 'aes-256-gcm',
        },);
        conf.set({
          key: 'foo',
          value: FIXTURE_VALUE,
        },);

        /**
         File bytes with the trailing authentication-tag byte inverted.
         */
        const tampered = Uint8Array.from(readFileSync(conf.path,),);
        /**
         Index of the trailing authentication-tag byte the test inverts.
         */
        const lastByteIndex = tampered.length - 1;
        tampered[lastByteIndex] = (tampered[lastByteIndex] ?? 0) ^ 0xFF;
        writeFileSync(
          conf.path,
          tampered,
        );

        expect(function constructReaderOverTamperedFile(): unknown {
          return createConf({
            cwd: directory,
            encryptionKey: 'abc123',
            encryptionAlgorithm: 'aes-256-gcm',
          },);
        },).toThrow(DecryptionFailedError,);
        expect(function constructReaderOverTamperedFile(): unknown {
          return createConf({
            cwd: directory,
            encryptionKey: 'abc123',
            encryptionAlgorithm: 'aes-256-gcm',
          },);
        },).toThrow('Failed to decrypt config data.',);
      },
    },),

    it({
      name: 'clears a corrupt aes-256-cbc file when clearInvalidConfig is set',
      fn: async () => {
        /**
         Directory whose encrypted file is corrupted behind the store.
         */
        const directory = createTempDirectory();
        /**
         Store whose file is about to be corrupted.
         */
        const before = createConf({
          cwd: directory,
          encryptionKey: 'abc123',
          clearInvalidConfig: true,
        },);
        before.set({
          key: 'foo',
          value: FIXTURE_VALUE,
        },);
        expect(before.get('foo',),).toBe(FIXTURE_VALUE,);
        appendFileSync(
          before.path,
          'corrupt file',
        );

        /**
         Store that must read the cleared store instead of the corruption.
         */
        const after = createConf({
          cwd: directory,
          encryptionKey: 'abc123',
          clearInvalidConfig: true,
        },);
        expect(after.get('foo',),).toBeUndefined();
        expect(after.store,).toEqual({},);
      },
    },),

    it({
      name: 'clears a corrupt encrypted file carrying a schema when clearInvalidConfig is set',
      fn: async () => {
        /**
         Directory whose encrypted file is replaced with garbage.
         */
        const directory = createTempDirectory();
        /**
         Store validating `enabled` as a boolean.
         */
        const before = createConf({
          cwd: directory,
          encryptionKey: 'enc-schema',
          schema: {
            enabled: {
              type: 'boolean',
            },
          },
          clearInvalidConfig: true,
        },);
        before.set({
          key: 'enabled',
          value: true,
        },);
        writeFileSync(
          before.path,
          'corrupt-data',
        );

        /**
         Store that must read the cleared store instead of the corruption.
         */
        const after = createConf({
          cwd: directory,
          encryptionKey: 'enc-schema',
          schema: {
            enabled: {
              type: 'boolean',
            },
          },
          clearInvalidConfig: true,
        },);
        expect(after.get('enabled',),).toBeUndefined();
      },
    },),

    it({
      name: 'throws DecryptionFailedError on a corrupt aes-256-gcm file when clearInvalidConfig is off',
      fn: async () => {
        /**
         Directory whose authenticated encrypted file is replaced with garbage.
         */
        const directory = createTempDirectory();
        /**
         Store whose file is about to be corrupted.
         */
        const before = createConf({
          cwd: directory,
          encryptionKey: 'abc123',
          encryptionAlgorithm: 'aes-256-gcm',
        },);
        before.set({
          key: 'foo',
          value: FIXTURE_VALUE,
        },);
        writeFileSync(
          before.path,
          'corrupt-data',
        );

        expect(function constructReaderOverCorruptFile(): unknown {
          return createConf({
            cwd: directory,
            encryptionKey: 'abc123',
            encryptionAlgorithm: 'aes-256-gcm',
          },);
        },).toThrow(DecryptionFailedError,);
      },
    },),

    it({
      name: 'reads an existing plaintext config file as an aes-256-cbc upgrade',
      fn: async () => {
        /**
         Directory whose config file starts as plaintext JSON.
         */
        const directory = createTempDirectory();
        writeConfigFile({
          directory,
          data: {
            foo: FIXTURE_VALUE,
          },
        },);

        /**
         Store that must treat unframed `aes-256-cbc` data as plaintext.
         */
        const conf = createConf({
          cwd: directory,
          encryptionKey: 'abc123',
        },);
        expect(conf.get('foo',),).toBe(FIXTURE_VALUE,);

        /**
         Second store standing in for another process reading the file.
         */
        const reloaded = createConf({
          cwd: directory,
          encryptionKey: 'abc123',
        },);
        expect(reloaded.get('foo',),).toBe(FIXTURE_VALUE,);
      },
    },),

    it({
      name: 'fails to read an existing plaintext config file as aes-256-gcm',
      fn: async () => {
        /**
         Directory whose config file starts as plaintext JSON.
         */
        const directory = createTempDirectory();
        writeConfigFile({
          directory,
          data: {
            foo: FIXTURE_VALUE,
          },
        },);

        expect(function constructGcmReaderOverPlaintext(): unknown {
          return createConf({
            cwd: directory,
            encryptionKey: 'abc123',
            encryptionAlgorithm: 'aes-256-gcm',
          },);
        },).toThrow(DecryptionFailedError,);
        expect(function constructGcmReaderOverPlaintext(): unknown {
          return createConf({
            cwd: directory,
            encryptionKey: 'abc123',
            encryptionAlgorithm: 'aes-256-gcm',
          },);
        },).toThrow('Failed to decrypt config data.',);
      },
    },),

    it({
      name: 'fails to read an existing plaintext config file as aes-256-ctr',
      fn: async () => {
        /**
         Directory whose config file starts as plaintext JSON.
         */
        const directory = createTempDirectory();
        writeConfigFile({
          directory,
          data: {
            foo: FIXTURE_VALUE,
          },
        },);

        expect(function constructCtrReaderOverPlaintext(): unknown {
          return createConf({
            cwd: directory,
            encryptionKey: 'abc123',
            encryptionAlgorithm: 'aes-256-ctr',
          },);
        },).toThrow(DecryptionFailedError,);
      },
    },),

    it({
      name: 'throws SyntaxError when the framing separator byte is invalid',
      fn: async () => {
        /**
         Mirrors upstream `encryption - invalid separator fails`: the
         corrupted separator makes the unauthenticated fallback treat the
         ciphertext as plaintext, so parsing fails with a SyntaxError.
         */
        /**
         Directory whose framed file will lose its separator byte.
         */
        const directory = createTempDirectory();
        /**
         Store whose framing byte will be corrupted.
         */
        const before = createConf({
          cwd: directory,
          encryptionKey: 'abc123',
        },);
        before.set({
          key: 'foo',
          value: FIXTURE_VALUE,
        },);

        /**
         File bytes with the framing separator byte replaced by `x`.
         */
        const corrupted = Uint8Array.from(readFileSync(before.path,),);
        corrupted[16] = 'x'.codePointAt(0,) ?? 0;
        writeFileSync(
          before.path,
          corrupted,
        );

        expect(function constructReaderOverCorruptedFraming(): unknown {
          return createConf({
            cwd: directory,
            encryptionKey: 'abc123',
          },);
        },).toThrow(SyntaxError,);
      },
    },),
  ],
},);
