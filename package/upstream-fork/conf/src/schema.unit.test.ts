/**
 Unit tests for the JSON Schema validator factory and schema-default capture:
 no-op validation, default collection, ajv default application, violation
 messages, option pass-through, and the schema-option error paths.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  captureSchemaDefaults,
  createConf,
  createSchemaValidator,
  InvalidSchemaError,
  RootSchemaPropertiesError,
  SchemaViolationError,
  type PreparedOptions,
  type ValueSchema,
} from '../dist/final/neutral/index.mjs';

import {
  createTempDirectory,
} from './test-support.ts';

/**
 Captures whatever a call throws so assertions can inspect the error.
 
 @param call - Invocation expected to throw.
 
 @returns Captured failure, or `undefined` when the call returns normally.
 */
function captureFailure(call: () => void,): unknown {
  try {
    call();
  }
  catch (error) {
    return error;
  }
  return undefined;
}

/**
 Builds prepared-shaped options for direct `createSchemaValidator` calls.
 
 The full `prepareOptions` pipeline is bypassed on purpose:
 the validator reads only `schema`,
 `rootSchema`,
 and `ajvOptions`,
 so tests hand it a prepared-shaped object carrying a cast.
 
 @param overrides - Option fields the test varies.
 
 @returns Prepared-shaped options for `createSchemaValidator`.
 */
function preparedOptionsWith(overrides: Record<string, unknown>,): PreparedOptions<Record<string, unknown>> {
  return {
    cwd: '/unused',
    configName: 'config',
    fileExtension: 'json',
    projectSuffix: 'nodejs',
    clearInvalidConfig: false,
    accessPropertiesByDotNotation: true,
    configFileMode: 0o666,
    encryptionAlgorithm: 'aes-256-cbc',
    ...overrides,
  } as PreparedOptions<Record<string, unknown>>;
}

await describe({
  name: 'schema validation and default capture',
  children: [
    it({
      name: 'validates arbitrary data without error and captures no defaults when no schema options are configured',
      fn: async () => {
        /**
         Validator built without `schema`,
         `rootSchema`,
         or `ajvOptions`.
         */
        const validator = createSchemaValidator(preparedOptionsWith({},),);
        expect(validator.validate({
          nested: {
            list: [
              1,
              'two',
              null,
            ],
          },
          text: 'anything goes',
        },),).toBeUndefined();
        expect(validator.schemaDefaults,).toEqual({},);
      },
    },),

    it({
      name: 'captures top-level schema defaults into schemaDefaults',
      fn: async () => {
        expect(captureSchemaDefaults({
          port: {
            type: 'number',
            default: 8_080,
          },
          name: {
            type: 'string',
            default: 'conf',
          },
          plain: {
            type: 'string',
          },
        },),).toEqual({
          port: 8_080,
          name: 'conf',
        },);
      },
    },),

    it({
      name: 'skips default entries whose value is undefined and non-object property schemas',
      fn: async () => {
        /**
         Schema map mixing capturable,
         undefined-default,
         and non-object property entries.
         */
        const schema = {
          kept: {
            type: 'number',
            default: 0,
          },
          undefinedDefault: {
            type: 'number',
            default: undefined,
          },
          booleanSchema: true,
          nullSchema: null,
          emptySchema: {},
        } as unknown as Record<string, ValueSchema>;
        expect(captureSchemaDefaults(schema,),).toEqual({
          kept: 0,
        },);
      },
    },),

    it({
      name: 'applies schema defaults into the validated object',
      fn: async () => {
        /**
         Validator declaring one default the caller omits.
         */
        const validator = createSchemaValidator(preparedOptionsWith({
          schema: {
            theme: {
              type: 'string',
              default: 'light',
            },
            port: {
              type: 'number',
              default: 8_080,
            },
          },
        },),);
        /**
         Store object validated in place.
         */
        const data: Record<string, unknown> = {
          theme: 'dark',
        };
        validator.validate(data,);
        expect(data,).toEqual({
          theme: 'dark',
          port: 8_080,
        },);
      },
    },),

    it({
      name: 'throws SchemaViolationError with upstream message for a type violation',
      fn: async () => {
        /**
         Validator requiring a string theme.
         */
        const validator = createSchemaValidator(preparedOptionsWith({
          schema: {
            theme: {
              type: 'string',
            },
          },
        },),);
        /**
         Failure captured from the invalid validation.
         */
        const failure = captureFailure(function validateWrongType(): void {
          validator.validate({
            theme: 42,
          },);
        },);
        expect(failure,).toBeInstanceOf(SchemaViolationError,);
        expect((failure as SchemaViolationError).message,)
          .toBe('Config schema violation: `theme` must be string',);
      },
    },),

    it({
      name: 'throws SchemaViolationError with upstream message for a maximum violation',
      fn: async () => {
        /**
         Validator capping the port value.
         */
        const validator = createSchemaValidator(preparedOptionsWith({
          schema: {
            port: {
              type: 'number',
              maximum: 10,
            },
          },
        },),);
        /**
         Failure captured from the invalid validation.
         */
        const failure = captureFailure(function validateOverMaximum(): void {
          validator.validate({
            port: 42,
          },);
        },);
        expect(failure,).toBeInstanceOf(SchemaViolationError,);
        expect((failure as SchemaViolationError).message,)
          .toBe('Config schema violation: `port` must be <= 10',);
      },
    },),

    it({
      name: 'joins multiple violations with a semicolon in one message',
      fn: async () => {
        /**
         Validator carrying one violation per property.
         */
        const validator = createSchemaValidator(preparedOptionsWith({
          schema: {
            theme: {
              type: 'string',
            },
            port: {
              type: 'number',
              maximum: 10,
            },
          },
        },),);
        /**
         Failure captured from the doubly-invalid validation.
         */
        const failure = captureFailure(function validateBothWrong(): void {
          validator.validate({
            theme: 42,
            port: 42,
          },);
        },);
        expect(failure,).toBeInstanceOf(SchemaViolationError,);
        expect((failure as SchemaViolationError).message,)
          .toBe('Config schema violation: `theme` must be string; `port` must be <= 10',);
      },
    },),

    it({
      name: 'throws InvalidSchemaError when the schema option is not an object',
      fn: async () => {
        expect(function buildWithNonObjectSchema(): void {
          createSchemaValidator(preparedOptionsWith({
            schema: 'not-an-object',
          },),);
        },).toThrow(InvalidSchemaError,);
      },
    },),

    it({
      name: 'throws RootSchemaPropertiesError when rootSchema carries a properties key',
      fn: async () => {
        expect(function buildWithRootProperties(): void {
          createSchemaValidator(preparedOptionsWith({
            rootSchema: {
              properties: {},
            },
          },),);
        },).toThrow(RootSchemaPropertiesError,);
      },
    },),

    it({
      name: 'throws SchemaViolationError when rootSchema forbids additional keys',
      fn: async () => {
        /**
         Validator wrapping the property map with `additionalProperties: false`.
         */
        const validator = createSchemaValidator(preparedOptionsWith({
          rootSchema: {
            additionalProperties: false,
          },
          schema: {
            known: {
              type: 'number',
            },
          },
        },),);
        /**
         Failure captured from the extra-key validation.
         */
        const failure = captureFailure(function validateExtraKey(): void {
          validator.validate({
            known: 1,
            xtra: 2,
          },);
        },);
        expect(failure,).toBeInstanceOf(SchemaViolationError,);
        expect((failure as SchemaViolationError).message,)
          .toBe('Config schema violation: `` must NOT have additional properties',);
      },
    },),

    it({
      name: 'passes ajvOptions through so removeAdditional drops forbidden keys during validation',
      fn: async () => {
        /**
         Validator configured with ajv's `removeAdditional: true`,
         which drops exactly the keys `additionalProperties: false` forbids.
         */
        const validator = createSchemaValidator(preparedOptionsWith({
          rootSchema: {
            additionalProperties: false,
          },
          ajvOptions: {
            removeAdditional: true,
          },
          schema: {
            known: {
              type: 'number',
            },
          },
        },),);
        /**
         Store object carrying one forbidden key.
         */
        const data: Record<string, unknown> = {
          known: 1,
          xtra: 2,
        };
        validator.validate(data,);
        expect(data,).toEqual({
          known: 1,
        },);
      },
    },),

    it({
      name: 'throws SchemaViolationError for values failing a declared format',
      fn: async () => {
        /**
         Validator requiring the url format.
         */
        const validator = createSchemaValidator(preparedOptionsWith({
          schema: {
            homepage: {
              type: 'string',
              format: 'url',
            },
          },
        },),);
        /**
         Failure captured from the malformed-url validation.
         */
        const failure = captureFailure(function validateMalformedUrl(): void {
          validator.validate({
            homepage: 'not-a-url',
          },);
        },);
        expect(failure,).toBeInstanceOf(SchemaViolationError,);
        expect((failure as SchemaViolationError).message,)
          .toBe('Config schema violation: `homepage` must match format "url"',);
      },
    },),

    it({
      name: 'applies schema defaults and surfaces violations end-to-end through createConf',
      fn: async () => {
        /**
         Store whose schema declares defaults and one type constraint.
         */
        const config = createConf({
          cwd: createTempDirectory(),
          schema: {
            theme: {
              type: 'string',
              default: 'light',
            },
            port: {
              type: 'number',
              default: 8_080,
            },
          },
        },);
        expect(config.get('theme',),).toBe('light',);
        expect(config.get('port',),).toBe(8_080,);

        config.set({
          key: 'theme',
          value: 'dark',
        },);
        expect(config.get('theme',),).toBe('dark',);
        expect(config.get('port',),).toBe(8_080,);

        /**
         Failure captured from an invalid store write.
         */
        const failure = captureFailure(function setInvalidTheme(): void {
          config.set({
            key: 'theme',
            value: 42,
          },);
        },);
        expect(failure,).toBeInstanceOf(SchemaViolationError,);
        expect((failure as SchemaViolationError).message,)
          .toBe('Config schema violation: `theme` must be string',);
      },
    },),
  ],
},);
