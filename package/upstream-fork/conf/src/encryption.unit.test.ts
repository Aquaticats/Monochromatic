/**
 Covers encryption-algorithm recognition and the config-file encryption wire format.
 
 @module
 */

import {
  readFileSync,
  writeFileSync,
} from 'node:fs';

import {
  join,
} from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  bytesToString,
  DecryptionFailedError,
  decryptConfigData,
  decryptWithSalt,
  encryptSerializedStore,
  InvalidAuthenticationTagError,
  isSupportedEncryptionAlgorithm,
  stringToBytes,
} from '../dist/final/neutral/index.mjs';

import {
  createTempDirectory,
  decryptConfigFile,
  writeLegacyEncryptedConfigFile,
} from './test-support.ts';

/**
 Runs a call expected to throw and returns the captured error so class and message text can be asserted.
 
 @param call - Call that must throw.
 
 @returns The captured thrown value.
 */
function captureThrown(call: () => unknown,): Error {
  try {
    call();
  }
  catch (error) {
    return error as Error;
  }
  throw new Error('expected the call to throw, but it returned',);
}

await describe({
  name: 'config file encryption',
  children: [
    it({
      name: 'recognizes each supported algorithm identifier',
      fn: async () => {
        expect(isSupportedEncryptionAlgorithm('aes-256-cbc',),).toBe(true,);
        expect(isSupportedEncryptionAlgorithm('aes-256-gcm',),).toBe(true,);
        expect(isSupportedEncryptionAlgorithm('aes-256-ctr',),).toBe(true,);
      },
    },),
    it({
      name: 'rejects other strings and non-string values as algorithm identifiers',
      fn: async () => {
        expect(isSupportedEncryptionAlgorithm('rot13',),).toBe(false,);
        expect(isSupportedEncryptionAlgorithm('AES-256-CBC',),).toBe(false,);
        expect(isSupportedEncryptionAlgorithm('',),).toBe(false,);
        expect(isSupportedEncryptionAlgorithm(42,),).toBe(false,);
        expect(isSupportedEncryptionAlgorithm(true,),).toBe(false,);
        expect(isSupportedEncryptionAlgorithm({},),).toBe(false,);
        expect(isSupportedEncryptionAlgorithm(null,),).toBe(false,);
        expect(isSupportedEncryptionAlgorithm(undefined,),).toBe(false,);
      },
    },),
    it({
      name: 'round-trips serialized store text under every supported algorithm',
      fn: async () => {
        /**
         Algorithms whose encryption must decrypt back to the exact input.
         */
        const algorithms = [
          'aes-256-cbc',
          'aes-256-gcm',
          'aes-256-ctr',
        ] as const;
        for (const encryptionAlgorithm of algorithms) {
          /**
           Serialized store text protected under this algorithm.
           */
          const encrypted = encryptSerializedStore({
            serialized: '{"theme":"dark"}',
            encryptionKey: 'correct horse',
            encryptionAlgorithm,
          },);
          expect(decryptConfigData({
            data: encrypted,
            encryptionKey: 'correct horse',
            encryptionAlgorithm,
          },),).toBe('{"theme":"dark"}',);
        }
      },
    },),
    it({
      name: 'frames the payload with a 16-byte initialization vector and a colon separator every algorithm reads back',
      fn: async () => {
        /**
         Disposable directory holding one config file per algorithm.
         */
        const directory = createTempDirectory();
        /**
         Algorithms whose on-disk framing must follow the shared layout.
         */
        const algorithms = [
          'aes-256-cbc',
          'aes-256-gcm',
          'aes-256-ctr',
        ] as const;
        for (const encryptionAlgorithm of algorithms) {
          /**
           Serialized store text protected under this algorithm.
           */
          const encrypted = encryptSerializedStore({
            serialized: '{"theme":"dark"}',
            encryptionKey: 'correct horse',
            encryptionAlgorithm,
          },);
          // Code point of the `:` separator following the initialization vector.
          expect(encrypted[16],).toBe(58,);
          /**
           Config file path the store would have written these bytes to.
           */
          const filePath = join(
            directory,
            'config.json',
          );
          writeFileSync(
            filePath,
            encrypted,
          );
          /**
           Decrypted text re-read through the store's on-disk layout,
           which derives the key from the first 16 bytes as the initialization vector.
           */
          const decrypted = decryptConfigFile({
            filePath,
            encryptionKey: 'correct horse',
            encryptionAlgorithm,
          },);
          expect(decrypted,).toBe('{"theme":"dark"}',);
        }
      },
    },),
    it({
      name: 'appends a 16-byte authentication tag after the aes-256-gcm ciphertext',
      fn: async () => {
        /**
         Serialized store text protected under aes-256-gcm.
         */
        const encrypted = encryptSerializedStore({
          serialized: '{"theme":"dark"}',
          encryptionKey: 'correct horse',
          encryptionAlgorithm: 'aes-256-gcm',
        },);
        // GCM ciphertext length equals plaintext length, so the trailing 16 bytes past the framing are exactly the tag.
        expect(encrypted.length,).toBe(17 + '{"theme":"dark"}'.length + 16,);
      },
    },),
    it({
      name: 'returns unframed data unchanged under aes-256-cbc',
      fn: async () => {
        /**
         Long plaintext whose byte 16 is not the separator code point.
         */
        const longPlaintext = '{"theme":"dark","nested":{"a":1}}';
        expect(decryptConfigData({
          data: stringToBytes(longPlaintext,),
          encryptionKey: 'correct horse',
          encryptionAlgorithm: 'aes-256-cbc',
        },),).toBe(longPlaintext,);
        /**
         Short plaintext with no byte 16 to hold a separator at all.
         */
        const shortPlaintext = 'plain';
        expect(decryptConfigData({
          data: stringToBytes(shortPlaintext,),
          encryptionKey: 'correct horse',
          encryptionAlgorithm: 'aes-256-cbc',
        },),).toBe(shortPlaintext,);
      },
    },),
    it({
      name: 'throws DecryptionFailedError for unframed aes-256-gcm data',
      fn: async () => {
        /**
         Error thrown because aes-256-gcm has no plaintext passthrough.
         */
        const error = captureThrown((): unknown => decryptConfigData({
          data: stringToBytes('{"theme":"dark","nested":{"a":1}}',),
          encryptionKey: 'correct horse',
          encryptionAlgorithm: 'aes-256-gcm',
        },),);
        expect(error,).toBeInstanceOf(DecryptionFailedError,);
        expect(error.name,).toBe('DecryptionFailedError',);
      },
    },),
    it({
      name: 'throws DecryptionFailedError for unframed aes-256-ctr data',
      fn: async () => {
        /**
         Error thrown because aes-256-ctr has no plaintext passthrough.
         */
        const error = captureThrown((): unknown => decryptConfigData({
          data: stringToBytes('{"theme":"dark","nested":{"a":1}}',),
          encryptionKey: 'correct horse',
          encryptionAlgorithm: 'aes-256-ctr',
        },),);
        expect(error,).toBeInstanceOf(DecryptionFailedError,);
        expect(error.name,).toBe('DecryptionFailedError',);
      },
    },),
    it({
      name: 'falls back to raw bytes as text when aes-256-cbc decryption fails under a wrong key',
      fn: async () => {
        /**
         Frozen ciphertext fixture written under `fixture-right-key`.
         Fixed bytes keep the wrong-key case deterministic:
         a random draw can otherwise decrypt with lucky padding roughly one
         run in 256 and break the fallback assertion.
         */
        const encrypted = new Uint8Array([
          116, 30, 39, 133, 139, 153, 21, 164, 156, 14, 85, 251, 128, 136, 13,
          204, 58, 178, 108, 117, 240, 251, 67, 86, 209, 100, 201, 75, 200, 245,
          59, 98, 194, 219, 142, 0, 121, 171, 142, 24, 42, 42, 108, 61, 236, 36,
          83, 255, 119,
        ],);
        /**
         Decryption attempt under a key that cannot open the framing.
         */
        const fallback = decryptConfigData({
          data: encrypted,
          encryptionKey: 'fixture-wrong-key',
          encryptionAlgorithm: 'aes-256-cbc',
        },);
        // Upstream mirrors this plaintext passthrough of the raw encrypted bytes as text.
        expect(fallback,).toBe(bytesToString(encrypted,),);
        expect(fallback === '{"theme":"dark"}',).toBe(false,);
      },
    },),
    it({
      name: 'falls back to raw bytes as text when aes-256-cbc framing carries an undecryptable payload',
      fn: async () => {
        /**
         Framed data whose 5-byte payload cannot be a block cipher output.
         */
        const framed = stringToBytes('0123456789abcdef:short',);
        expect(decryptConfigData({
          data: framed,
          encryptionKey: 'correct horse',
          encryptionAlgorithm: 'aes-256-cbc',
        },),).toBe('0123456789abcdef:short',);
      },
    },),
    it({
      name: 'throws DecryptionFailedError when aes-256-gcm decryption fails under a wrong key',
      fn: async () => {
        /**
         Ciphertext written under the correct key.
         */
        const encrypted = encryptSerializedStore({
          serialized: '{"theme":"dark"}',
          encryptionKey: 'correct horse',
          encryptionAlgorithm: 'aes-256-gcm',
        },);
        /**
         Error thrown because authenticated decryption rejects the wrong key.
         */
        const error = captureThrown((): unknown => decryptConfigData({
          data: encrypted,
          encryptionKey: 'wrong horse',
          encryptionAlgorithm: 'aes-256-gcm',
        },),);
        expect(error,).toBeInstanceOf(DecryptionFailedError,);
        expect(error.name,).toBe('DecryptionFailedError',);
      },
    },),
    it({
      name: 'returns silent garbage text when aes-256-ctr decryption runs under a wrong key',
      fn: async () => {
        /**
         Ciphertext written under the correct key.
         */
        const encrypted = encryptSerializedStore({
          serialized: '{"theme":"dark"}',
          encryptionKey: 'correct horse',
          encryptionAlgorithm: 'aes-256-ctr',
        },);
        /**
         Decryption attempt under a wrong key.
         */
        const garbage = decryptConfigData({
          data: encrypted,
          encryptionKey: 'wrong horse',
          encryptionAlgorithm: 'aes-256-ctr',
        },);
        // Unauthenticated stream decryption cannot detect the wrong key, so it reports garbage instead of failing.
        expect(garbage === '{"theme":"dark"}',).toBe(false,);
      },
    },),
    it({
      name: 'throws InvalidAuthenticationTagError from decryptWithSalt for an aes-256-gcm payload shorter than its authentication tag',
      fn: async () => {
        /**
         Framed data whose 5-byte payload cannot hold a 16-byte authentication tag.
         */
        const framed = stringToBytes('0123456789abcdef:short',);
        /**
         Error thrown where the authentication tag length is checked.
         */
        const error = captureThrown((): unknown => decryptWithSalt({
          data: framed,
          encryptionKey: 'correct horse',
          encryptionAlgorithm: 'aes-256-gcm',
          salt: stringToBytes('0123456789abcdef',),
        },),);
        expect(error,).toBeInstanceOf(InvalidAuthenticationTagError,);
        expect(error.name,).toBe('InvalidAuthenticationTagError',);
        expect(error.message,).toBe('Invalid authentication tag length.',);
      },
    },),
    it({
      name: 'surfaces DecryptionFailedError from decryptConfigData for the same truncated aes-256-gcm payload',
      fn: async () => {
        /**
         Framed data whose 5-byte payload cannot hold a 16-byte authentication tag.
         */
        const framed = stringToBytes('0123456789abcdef:short',);
        /**
         Error surfaced after both salt attempts fail and no passthrough applies.
         */
        const error = captureThrown((): unknown => decryptConfigData({
          data: framed,
          encryptionKey: 'correct horse',
          encryptionAlgorithm: 'aes-256-gcm',
        },),);
        // Current behavior: decryptConfigData swallows the authentication-tag failure into its salt fallback.
        expect(error,).toBeInstanceOf(DecryptionFailedError,);
      },
    },),
    it({
      name: 'reads a legacy-salt encrypted file back with the same key',
      fn: async () => {
        /**
         Disposable directory holding the legacy-layout config file.
         */
        const directory = createTempDirectory();
        /**
         Config file path carrying the legacy salt layout.
         */
        const filePath = join(
          directory,
          'config.json',
        );
        writeLegacyEncryptedConfigFile({
          filePath,
          encryptionKey: 'correct horse',
          serialized: '{"theme":"dark"}',
        },);
        /**
         Raw legacy file bytes as written by older releases.
         */
        const data = new Uint8Array(readFileSync(filePath,),);
        expect(decryptConfigData({
          data,
          encryptionKey: 'correct horse',
          encryptionAlgorithm: 'aes-256-cbc',
        },),).toBe('{"theme":"dark"}',);
      },
    },),
    it({
      name: 'round-trips an empty serialized payload under aes-256-gcm',
      fn: async () => {
        /**
         Encryption of the empty serialized store.
         */
        const encrypted = encryptSerializedStore({
          serialized: '',
          encryptionKey: 'correct horse',
          encryptionAlgorithm: 'aes-256-gcm',
        },);
        expect(decryptConfigData({
          data: encrypted,
          encryptionKey: 'correct horse',
          encryptionAlgorithm: 'aes-256-gcm',
        },),).toBe('',);
      },
    },),
  ],
},);
