/**
 Sops-envelope and decrypted-content validation.

 The envelope check runs on ciphertext and rejects plaintext before any
 identity lookup or sops invocation. The decrypted check validates every
 top-level value and never includes parsed content in an error or log.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  SOPS_ENCRYPTED_VALUE_PREFIX,
  SOPS_KEY_GROUPS,
} from './constants.ts';
import {
  EnvLocalJsonPlaintextError,
  EnvLocalJsonShapeError,
} from './errors.ts';

//region Constants

/**
 Logger for shape failures; names keys and reasons only.
 */
const shapeLogger = tagged({ tag: 'envLocalJsonShape', },);

//endregion Constants

//region Predicates

/**
 Whether a parsed JSON value is a non-null, non-array object.

 @param value - parsed JSON value

 @returns `true` for Object-like JSON records

 @example
 ```ts
 isJsonRecord({ a: 1 }); // true
 ```
 */
function isJsonRecord(value: unknown,): value is Readonly<Record<string, unknown>> {
  return ((typeof value) === 'object')
    && (value !== null)
    && (!Array.isArray(value,));
}

/**
 Parses JSON without exposing the input to callers or logs.

 @param content - JSON text

 @returns parsed value, or `undefined` when parsing fails

 @example
 ```ts
 parseJsonQuietly('{"a":1}');
 ```
 */
function parseJsonQuietly(content: string,): unknown {
  try {
    return JSON.parse(content,) as unknown;
  }
  catch (error) {
    shapeLogger.debug(
      `JSON parsing failed with ${Error.isError(error,) ? error.name : 'non-error rejection'}`,
    );
    return undefined;
  }
}

//endregion Predicates

//region Envelope

/**
 Asserts that ciphertext carries sops metadata with a version, a MAC, and one
 supported key group.

 @param content - raw file content read from the store

 @param path - store path named in the failure

 @throws EnvLocalJsonPlaintextError when the content is plaintext, malformed,
 or lacks required sops metadata

 @example
 ```ts
 assertSopsEnvelope({ content: await readFile(path, 'utf8'), path });
 ```
 */
export function assertSopsEnvelope({
  content,
  path,
}: {
  readonly content: string;
  readonly path: string;
},): void {
  /**
   Parsed envelope candidate; `undefined` when the file is not JSON.
   */
  const parsed = parseJsonQuietly(content,);
  if (!isJsonRecord(parsed,))
    throw new EnvLocalJsonPlaintextError({ path, },);

  /**
   Sops metadata object candidate.
   */
  const metadata = parsed.sops;
  if (!isJsonRecord(metadata,))
    throw new EnvLocalJsonPlaintextError({ path, },);

  /**
   Sops format version recorded by the envelope.
   */
  const { version, } = metadata;
  if (((typeof version) !== 'string') || (version.trim() === ''))
    throw new EnvLocalJsonPlaintextError({ path, },);

  /**
   Message authentication code recorded by the envelope.
   */
  const { mac, } = metadata;
  if ((typeof mac) !== 'string')
    throw new EnvLocalJsonPlaintextError({ path, },);

  /**
   Whether any supported key group holds at least one recipient entry.
   */
  const hasKeyGroup = SOPS_KEY_GROUPS.some(function hasGroup(group: string,): boolean {
    /**
     Candidate key group value.
     */
    const groupValue = metadata[group];
    return Array.isArray(groupValue,) && (groupValue.length > 0);
  },);
  if (!hasKeyGroup)
    throw new EnvLocalJsonPlaintextError({ path, },);

  shapeLogger.debug(`store ${path} carries sops metadata version ${version}`,);
}

//endregion Envelope

//region Decrypted content

/**
 Validates and returns decrypted top-level string values.

 @param content - sops `--decrypt --output-type json` output

 @param path - store path named in failures

 @returns every top-level key mapped to its non-blank string value

 @throws EnvLocalJsonShapeError when the document is not a JSON object, or a
 value is non-string, blank after trimming, or still sops-encrypted

 @example
 ```ts
 const store = parseDecryptedValues({ content: stdout, path });
 ```
 */
export function parseDecryptedValues({
  content,
  path,
}: {
  readonly content: string;
  readonly path: string;
},): Readonly<Record<string, string>> {
  /**
   Parsed decrypted document; `undefined` when it is not JSON.
   */
  const parsed = parseJsonQuietly(content,);
  if (parsed === undefined)
    throw new EnvLocalJsonShapeError({
      path,
      reason: 'not-json',
    },);
  if (!isJsonRecord(parsed,))
    throw new EnvLocalJsonShapeError({
      path,
      reason: 'not-object',
    },);

  /**
   Validated top-level values being collected.
   */
  const values: Record<string, string> = {};
  for (const [key, value,] of Object.entries(parsed,)) {
    if ((typeof value) !== 'string') {
      throw new EnvLocalJsonShapeError({
        path,
        reason: 'non-string',
        key,
      },);
    }
    if (value.trim() === '') {
      throw new EnvLocalJsonShapeError({
        path,
        reason: 'blank',
        key,
      },);
    }
    if (value.trimStart()
      .startsWith(SOPS_ENCRYPTED_VALUE_PREFIX,)) {
      throw new EnvLocalJsonShapeError({
        path,
        reason: 'encrypted-token',
        key,
      },);
    }
    values[key] = value;
  }
  return values;
}

//endregion Decrypted content
