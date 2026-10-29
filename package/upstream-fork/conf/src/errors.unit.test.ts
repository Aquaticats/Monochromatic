/**
 Covers every exported error class's base type,
 `name`,
 and exact upstream message text.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  DecryptionFailedError,
  InvalidAuthenticationTagError,
  InvalidCallbackError,
  InvalidEncryptionAlgorithmError,
  InvalidKeyError,
  InvalidSchemaError,
  MigrationFailedError,
  MissingProjectNameError,
  MissingProjectVersionError,
  MissingValueError,
  NonArrayValueError,
  ReservedKeyError,
  RootSchemaPropertiesError,
  SchemaViolationError,
  UnsupportedValueTypeError,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: 'error classes',
  children: [
    it({
      name: 'InvalidKeyError extends TypeError carrying the caller-supplied upstream key message',
      fn: async () => {
        /**
         Error under test built with the upstream `get` key-type message.
         */
        const error = new InvalidKeyError('Expected `key` to be of type `string`, got number',);
        expect(error,).toBeInstanceOf(TypeError,);
        expect(error.name,).toBe('InvalidKeyError',);
        expect(error.message,).toBe('Expected `key` to be of type `string`, got number',);
      },
    },),
    it({
      name: 'InvalidCallbackError extends TypeError carrying the caller-supplied upstream callback message',
      fn: async () => {
        /**
         Error under test built with the upstream `onDidAnyChange` callback message.
         */
        const error = new InvalidCallbackError('Expected `callback` to be of type `function`, got string',);
        expect(error,).toBeInstanceOf(TypeError,);
        expect(error.name,).toBe('InvalidCallbackError',);
        expect(error.message,).toBe('Expected `callback` to be of type `function`, got string',);
      },
    },),
    it({
      name: 'MissingValueError extends TypeError naming delete() as the clearing call',
      fn: async () => {
        /**
         Error under test for a `set` receiving `undefined`.
         */
        const error = new MissingValueError();
        expect(error,).toBeInstanceOf(TypeError,);
        expect(error.name,).toBe('MissingValueError',);
        expect(error.message,).toBe('Use `delete()` to clear values',);
      },
    },),
    it({
      name: 'ReservedKeyError extends TypeError naming the reserved __internal__ bookkeeping key',
      fn: async () => {
        /**
         Error under test for a write touching `__internal__`.
         */
        const error = new ReservedKeyError();
        expect(error,).toBeInstanceOf(TypeError,);
        expect(error.name,).toBe('ReservedKeyError',);
        expect(error.message,).toBe(
          "Please don't use the __internal__ key, as it's used to manage this module internal operations.",
        );
      },
    },),
    it({
      name: 'UnsupportedValueTypeError extends TypeError formatting the rejected type and store key',
      fn: async () => {
        /**
         Error under test for a value JSON cannot round-trip.
         */
        const error = new UnsupportedValueTypeError({
          type: 'function',
          key: 'fn',
        },);
        expect(error,).toBeInstanceOf(TypeError,);
        expect(error.name,).toBe('UnsupportedValueTypeError',);
        expect(error.message,).toBe(
          "Setting a value of type `function` for key `fn` is not allowed as it's not supported by JSON",
        );
      },
    },),
    it({
      name: 'NonArrayValueError extends TypeError formatting the key holding the non-array value',
      fn: async () => {
        /**
         Error under test for `appendToArray` over a non-array value.
         */
        const error = new NonArrayValueError({ key: 'items', },);
        expect(error,).toBeInstanceOf(TypeError,);
        expect(error.name,).toBe('NonArrayValueError',);
        expect(error.message,).toBe('The key `items` is already set to a non-array value',);
      },
    },),
    it({
      name: 'InvalidEncryptionAlgorithmError extends TypeError listing supported algorithms joined by comma-space',
      fn: async () => {
        /**
         Error under test listing the three supported algorithm identifiers.
         */
        const error = new InvalidEncryptionAlgorithmError({
          supported: [
            'aes-256-cbc',
            'aes-256-gcm',
            'aes-256-ctr',
          ],
        },);
        expect(error,).toBeInstanceOf(TypeError,);
        expect(error.name,).toBe('InvalidEncryptionAlgorithmError',);
        expect(error.message,).toBe('The `encryptionAlgorithm` option must be one of: aes-256-cbc, aes-256-gcm, aes-256-ctr',);
      },
    },),
    it({
      name: 'MissingProjectNameError extends Error asking for the projectName option',
      fn: async () => {
        /**
         Error under test for an unresolvable config directory.
         */
        const error = new MissingProjectNameError();
        expect(error,).toBeInstanceOf(Error,);
        expect(error instanceof TypeError,).toBe(false,);
        expect(error.name,).toBe('MissingProjectNameError',);
        expect(error.message,).toBe('Please specify the `projectName` option.',);
      },
    },),
    it({
      name: 'MissingProjectVersionError extends Error asking for the projectVersion option',
      fn: async () => {
        /**
         Error under test for migrations without a version.
         */
        const error = new MissingProjectVersionError();
        expect(error,).toBeInstanceOf(Error,);
        expect(error instanceof TypeError,).toBe(false,);
        expect(error.name,).toBe('MissingProjectVersionError',);
        expect(error.message,).toBe('Please specify the `projectVersion` option.',);
      },
    },),
    it({
      name: 'InvalidSchemaError extends TypeError requiring the schema option to be an object',
      fn: async () => {
        /**
         Error under test for a non-object `schema` option.
         */
        const error = new InvalidSchemaError();
        expect(error,).toBeInstanceOf(TypeError,);
        expect(error.name,).toBe('InvalidSchemaError',);
        expect(error.message,).toBe('The `schema` option must be an object.',);
      },
    },),
    it({
      name: 'RootSchemaPropertiesError extends TypeError steering properties to the schema option',
      fn: async () => {
        /**
         Error under test for `rootSchema` carrying `properties`.
         */
        const error = new RootSchemaPropertiesError();
        expect(error,).toBeInstanceOf(TypeError,);
        expect(error.name,).toBe('RootSchemaPropertiesError',);
        expect(error.message,).toBe(
          'The `rootSchema` option must not contain a `properties` key. Use the `schema` option for properties.',
        );
      },
    },),
    it({
      name: 'SchemaViolationError extends Error joining per-property violations with semicolons',
      fn: async () => {
        /**
         Error under test carrying two per-property violation texts.
         */
        const error = new SchemaViolationError({
          violations: [
            '`port` must be integer',
            '`name` must be string',
          ],
        },);
        expect(error,).toBeInstanceOf(Error,);
        expect(error instanceof TypeError,).toBe(false,);
        expect(error.name,).toBe('SchemaViolationError',);
        expect(error.message,).toBe('Config schema violation: `port` must be integer; `name` must be string',);
      },
    },),
    it({
      name: 'DecryptionFailedError extends Error carrying the upstream decrypt failure message',
      fn: async () => {
        /**
         Error under test for undecryptable config data.
         */
        const error = new DecryptionFailedError();
        expect(error,).toBeInstanceOf(Error,);
        expect(error instanceof TypeError,).toBe(false,);
        expect(error.name,).toBe('DecryptionFailedError',);
        expect(error.message,).toBe('Failed to decrypt config data.',);
      },
    },),
    it({
      name: 'InvalidAuthenticationTagError extends Error carrying the upstream tag length message',
      fn: async () => {
        /**
         Error under test for an `aes-256-gcm` payload without room for its tag.
         */
        const error = new InvalidAuthenticationTagError();
        expect(error,).toBeInstanceOf(Error,);
        expect(error instanceof TypeError,).toBe(false,);
        expect(error.name,).toBe('InvalidAuthenticationTagError',);
        expect(error.message,).toBe('Invalid authentication tag length.',);
      },
    },),
    it({
      name: 'MigrationFailedError extends Error embedding the failure reason after the restore notice',
      fn: async () => {
        /**
         Error under test carrying the aborting step's failure text.
         */
        const error = new MigrationFailedError({ reason: 'step exploded', },);
        expect(error,).toBeInstanceOf(Error,);
        expect(error instanceof TypeError,).toBe(false,);
        expect(error.name,).toBe('MigrationFailedError',);
        expect(error.message,).toBe(
          'Something went wrong during the migration! Changes applied to the store until this failed migration will be restored. step exploded',
        );
      },
    },),
  ],
},);
