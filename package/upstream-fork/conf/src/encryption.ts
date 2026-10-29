/**
 Optional config-file encryption and decryption.
 
 Mirrors upstream `conf` 15.1.0's wire format exactly: a 16-byte
 initialization vector,
 a `:` separator,
 then the ciphertext,
 with `aes-256-gcm` appending its 16-byte authentication tag. Keys derive
 through PBKDF2-SHA512 over the initialization vector,
 with a legacy salt fallback for files written by older `conf` releases.
 The feature is obscurity,
 not security:
 the key lives beside the app.
 
 @module
 */

import {
  createCipheriv,
  createDecipheriv,
  pbkdf2Sync,
  randomBytes,
} from 'node:crypto';
import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  DecryptionFailedError,
  InvalidAuthenticationTagError,
} from './errors.ts';
import {
  bytesToString,
  concatBytes,
  stringToBytes,
} from './bytes.ts';

//region Types

/**
 Encryption algorithm identifier accepted by the store.
 
 @example
 ```ts
 const algorithm: EncryptionAlgorithm = 'aes-256-cbc';
 ```
 */
export type EncryptionAlgorithm = 'aes-256-cbc' | 'aes-256-gcm' | 'aes-256-ctr';

/**
 Key material accepted by the store's `encryptionKey` option.
 
 @example
 ```ts
 const key: EncryptionKey = 'correct horse battery staple';
 ```
 */
export type EncryptionKey = string | Uint8Array | NodeJS.TypedArray | DataView;

//endregion Types

//region Constants

/**
 Algorithm used when `encryptionKey` is set without an explicit algorithm.
 
 @example
 ```ts
 DEFAULT_ENCRYPTION_ALGORITHM; // 'aes-256-cbc'
 ```
 */
export const DEFAULT_ENCRYPTION_ALGORITHM: EncryptionAlgorithm = 'aes-256-cbc';

/**
 Every algorithm the store accepts.
 
 @example
 ```ts
 SUPPORTED_ENCRYPTION_ALGORITHMS.has('aes-256-gcm'); // => true
 ```
 */
export const SUPPORTED_ENCRYPTION_ALGORITHMS: ReadonlySet<string> = new Set<string>([
  'aes-256-cbc',
  'aes-256-gcm',
  'aes-256-ctr',
],);

/**
 Byte length of the random initialization vector written before the
 separator.
 
 @example
 ```ts
 INITIALIZATION_VECTOR_LENGTH; // 16
 ```
 */
const INITIALIZATION_VECTOR_LENGTH = 16;

/**
 Code point of the `:` byte separating the initialization vector from the
 ciphertext.
 
 @example
 ```ts
 SEPARATOR_CODE_POINT; // 58
 ```
 */
const SEPARATOR_CODE_POINT = 58;

/**
 Byte length of the `aes-256-gcm` authentication tag appended after the
 ciphertext.
 
 @example
 ```ts
 AUTHENTICATION_TAG_LENGTH; // 16
 ```
 */
const AUTHENTICATION_TAG_LENGTH = 16;

/**
 PBKDF2 round count used to derive the file key.
 
 @example
 ```ts
 PBKDF2_ITERATIONS; // 10_000
 ```
 */
const PBKDF2_ITERATIONS = 10_000;

/**
 Byte length of the derived cipher key.
 
 @example
 ```ts
 DERIVED_KEY_LENGTH; // 32
 ```
 */
const DERIVED_KEY_LENGTH = 32;

/**
 PBKDF2 digest backing key derivation.
 
 @example
 ```ts
 PBKDF2_DIGEST; // 'sha512'
 ```
 */
const PBKDF2_DIGEST = 'sha512';

//endregion Constants

//region Algorithm checks

/**
 Reports whether a value names a supported encryption algorithm.
 
 @param value - Candidate from options or an external caller.
 
 @returns `true` when `value` is one of {@link SUPPORTED_ENCRYPTION_ALGORITHMS}.
 
 @example
 ```ts
 isSupportedEncryptionAlgorithm('aes-256-cbc'); // => true
 isSupportedEncryptionAlgorithm('rot13'); // => false
 ```
 */
export function isSupportedEncryptionAlgorithm(value: unknown,): value is EncryptionAlgorithm {
  return ((typeof value) === 'string') && SUPPORTED_ENCRYPTION_ALGORITHMS.has(value);
}

//endregion Algorithm checks

//region Key derivation

/**
 Derives the per-file cipher key from the user key and a salt.
 
 @param encryptionKey - User-supplied key material.
 
 @param salt - Salt bytes; the initialization vector in current files.
 
 @returns Derived 32-byte key for the configured algorithm.
 
 @example
 ```ts
 deriveKey({ encryptionKey: 'k', salt: initializationVector, });
 ```
 */
export function deriveKey({
  encryptionKey,
  salt,
}: {
  readonly encryptionKey: EncryptionKey;
  readonly salt: Uint8Array | string;
},): Buffer {
  // oxlint-disable-next-line no-restricted-syntax/no-sync -- Structurally synchronous boundary: the store's get/set contract writes before returning, so key derivation must finish inside the write call; see package/upstream-fork/conf/DECISION.sync-api.md.
  return pbkdf2Sync(
    encryptionKey,
    salt,
    PBKDF2_ITERATIONS,
    DERIVED_KEY_LENGTH,
    PBKDF2_DIGEST,
  );
}

//endregion Key derivation

//region Encryption

/**
 Encrypts serialized store text into the config-file byte layout.
 
 @param serialized - Serialized store text to protect.
 
 @param encryptionKey - User-supplied key material.
 
 @param encryptionAlgorithm - Algorithm to apply.
 
 @returns Initialization vector,
 separator,
 ciphertext,
 and (for `aes-256-gcm`) authentication tag as one buffer.
 
 @example
 ```ts
 encryptSerializedStore({
   serialized: '{"theme":"dark"}',
   encryptionKey: 'k',
   encryptionAlgorithm: 'aes-256-cbc',
 }); // => Uint8Array
 ```
 */
export function encryptSerializedStore({
  serialized,
  encryptionKey,
  encryptionAlgorithm,
}: {
  readonly serialized: string;
  readonly encryptionKey: EncryptionKey;
  readonly encryptionAlgorithm: EncryptionAlgorithm;
},): Uint8Array {
  /**
   Random salt and nonce for this write.
   */
  const initializationVector = randomBytes(INITIALIZATION_VECTOR_LENGTH,);
  /**
   Derived per-file cipher key.
   */
  const key = deriveKey({
    encryptionKey,
    salt: initializationVector,
  },);
  /**
   Framed parts joined below in wire order.
   */
  const parts = [
    initializationVector,
    stringToBytes(':',),
  ];
  if (encryptionAlgorithm === 'aes-256-gcm') {
    /**
     GCM cipher whose authentication tag is appended after the ciphertext.
     */
    const cipher = createCipheriv(
      'aes-256-gcm',
      key,
      initializationVector,
    );
    parts.push(
      concatBytes([
        cipher.update(stringToBytes(serialized,),),
        cipher.final(),
      ],),
      cipher.getAuthTag(),
    );
    return concatBytes(parts,);
  }
  /**
   Block or stream cipher for the unauthenticated algorithms.
   */
  const cipher = createCipheriv(
    encryptionAlgorithm,
    key,
    initializationVector,
  );
  parts.push(
    concatBytes([
      cipher.update(stringToBytes(serialized,),),
      cipher.final(),
    ],),
  );
  return concatBytes(parts,);
}

//endregion Encryption

//region Decryption

/**
 Decrypts one payload with a given salt,
 or reports why it failed.
 
 @param data - Full file bytes including framing.
 
 @param encryptionKey - User-supplied key material.
 
 @param encryptionAlgorithm - Algorithm the file was written with.
 
 @param salt - Salt candidate to try.
 
 @returns Decrypted serialized store text.
 
 @throws InvalidAuthenticationTagError when an `aes-256-gcm` payload is
 too short for its authentication tag.
 
 @example
 ```ts
 decryptWithSalt({
   data: fileBytes,
   encryptionKey: 'k',
   encryptionAlgorithm: 'aes-256-cbc',
   salt: initializationVector,
 });
 ```
 */
export function decryptWithSalt({
  data,
  encryptionKey,
  encryptionAlgorithm,
  salt,
}: {
  readonly data: Uint8Array;
  readonly encryptionKey: EncryptionKey;
  readonly encryptionAlgorithm: EncryptionAlgorithm;
  readonly salt: Uint8Array | string;
},): string {
  /**
   Initialization vector framing the payload.
   */
  const initializationVector = data.slice(
    0,
    INITIALIZATION_VECTOR_LENGTH,
  );
  /**
   Ciphertext region following the separator byte.
   */
  const payload = data.slice(INITIALIZATION_VECTOR_LENGTH + 1,);
  if (encryptionAlgorithm === 'aes-256-gcm') {
    /**
     Index where the trailing authentication tag begins.
     */
    const authenticationTagStart = payload.length - AUTHENTICATION_TAG_LENGTH;
    if (authenticationTagStart < 0)
      throw new InvalidAuthenticationTagError();
    /**
     GCM decipher authenticated against the trailing tag.
     */
    const decipher = createDecipheriv(
      'aes-256-gcm',
      deriveKey({
        encryptionKey,
        salt,
      },),
      initializationVector,
    );
    decipher.setAuthTag(payload.slice(authenticationTagStart,),);
    return bytesToString(
      concatBytes([
        decipher.update(payload.slice(
          0,
          authenticationTagStart,
        ),),
        decipher.final(),
      ],),
    );
  }
  /**
   Block or stream decipher for the unauthenticated algorithms.
   */
  const decipher = createDecipheriv(
    encryptionAlgorithm,
    deriveKey({
      encryptionKey,
      salt,
    },),
    initializationVector,
  );
  return bytesToString(
    concatBytes([
      decipher.update(payload,),
      decipher.final(),
    ],),
  );
}

/**
 Decrypts config-file bytes back into serialized store text.
 
 Mirrors upstream `conf` 15.1.0: files without framing pass through
 untouched under `aes-256-cbc`,
 current-salt then legacy-salt decryption are attempted,
 and only then does the plaintext-or-fail fallback apply.
 
 @param data - Full file bytes as read from disk.
 
 @param encryptionKey - User-supplied key material.
 
 @param encryptionAlgorithm - Algorithm the file was written with.
 
 @param logger - Logger for the expected salt-scheme fallback diagnostics.
 
 @returns Serialized store text.
 
 @throws DecryptionFailedError when no plaintext passthrough applies and
 every decryption attempt failed.
 
 @example
 ```ts
 decryptConfigData({
   data: fileBytes,
   encryptionKey: 'k',
   encryptionAlgorithm: 'aes-256-cbc',
 }); // => '{"theme":"dark"}'
 ```
 */
export function decryptConfigData({
  data,
  encryptionKey,
  encryptionAlgorithm,
  logger,
}: {
  readonly data: Uint8Array;
  readonly encryptionKey: EncryptionKey;
  readonly encryptionAlgorithm: EncryptionAlgorithm;
  readonly logger?: Logger;
},): string {
  /**
   Logger wrapped with this function's name so fallback diagnostics name
   their origin.
   */
  const log = logger ?? tagged({
    tag: decryptConfigData.name,
  },);
  /**
   Byte where the initialization vector ends and the separator lives.
   */
  const separatorByte = data[INITIALIZATION_VECTOR_LENGTH];
  if (separatorByte !== SEPARATOR_CODE_POINT) {
    if (encryptionAlgorithm === 'aes-256-cbc')
      return bytesToString(data,);
    throw new DecryptionFailedError();
  }
  /**
   Initialization vector framing the payload.
   */
  const initializationVector = data.slice(
    0,
    INITIALIZATION_VECTOR_LENGTH,
  );
  try {
    return decryptWithSalt({
      data,
      encryptionKey,
      encryptionAlgorithm,
      salt: initializationVector,
    },);
  }
  catch (currentSaltError) {
    log.debug(`current-salt decryption failed (${caughtValueText(currentSaltError,)}); trying legacy salt`,);
    try {
      return decryptWithSalt({
        data,
        encryptionKey,
        encryptionAlgorithm,
        salt: bytesToString(initializationVector,),
      },);
    }
    catch (legacySaltError) {
      log.debug(`legacy-salt decryption failed (${caughtValueText(legacySaltError,)}); falling back`,);
    }
  }
  if (encryptionAlgorithm === 'aes-256-cbc')
    return bytesToString(data,);
  throw new DecryptionFailedError();
}

//endregion Decryption
