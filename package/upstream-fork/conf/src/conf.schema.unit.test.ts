/**
 Schema validation behavior: defaults,
 violations,
 root keywords,
 formats,
 and ajv option pass-through.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createConf,
  InvalidSchemaError,
  RootSchemaPropertiesError,
  SchemaViolationError,
} from '../dist/final/neutral/index.mjs';

import {
  createTempDirectory,
  writeConfigFile,
} from './test-support.ts';

await describe({
  name: 'schema validation',
  children: [
    it({
      name: 'applies a top-level schema default for a missing key',
      fn: async () => {
        /**
         Store whose schema declares a top-level default.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
          schema: {
            foo: {
              type: 'string',
              default: 'bar',
            },
          },
        },);
        expect(conf.get('foo',),).toBe('bar',);
        expect(conf.has('foo',),).toBe(true,);
      },
    },),

    it({
      name: 'lets a Conf default overwrite a schema default',
      fn: async () => {
        /**
         Store carrying both a schema default and a Conf default for `foo`.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
          defaults: {
            foo: 'foo',
          },
          schema: {
            foo: {
              type: 'string',
              default: 'bar',
            },
          },
        },);
        expect(conf.get('foo',),).toBe('foo',);
      },
    },),

    it({
      name: 'leaves nested schema property defaults unapplied',
      fn: async () => {
        /**
         Mirrors upstream `failingTest('.get() - `schema` option - default')`,
         whose body must throw upstream: only top-level schema defaults are
         applied, so the nested `bar` default never reaches the store.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
          schema: {
            foo: {
              type: 'boolean',
              default: true,
            },
            nested: {
              type: 'object',
              properties: {
                bar: {
                  type: 'number',
                  default: 55,
                },
              },
            },
          },
        },);
        expect(conf.get('foo',),).toBe(true,);
        expect(conf.get('nested.bar',),).toBeUndefined();
      },
    },),

    it({
      name: 'replaces a nested schema default object with the whole Conf defaults object',
      fn: async () => {
        /**
         Store whose Conf default for `appearance` competes with its schema default.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
          defaults: {
            appearance: {
              layout: 'list',
            },
          },
          schema: {
            appearance: {
              type: 'object',
              default: {
                theme: 'light',
                layout: 'grid',
              },
            },
          },
        },);
        expect(conf.get('appearance',),).toEqual({
          layout: 'list',
        },);
      },
    },),

    it({
      name: 'throws SchemaViolationError at construction when a Conf default violates the schema',
      fn: async () => {
        /**
         Store options whose Conf default contradicts its schema.
         */
        const invalidDefaults = {
          cwd: createTempDirectory(),
          defaults: {
            foo: 1,
          },
          schema: {
            foo: {
              type: 'string',
            },
          },
        };
        expect(function constructWithInvalidDefault(): unknown {
          return createConf(invalidDefaults,);
        },).toThrow(SchemaViolationError,);
        expect(function constructWithInvalidDefault(): unknown {
          return createConf(invalidDefaults,);
        },).toThrow('Config schema violation: `foo` must be string',);
      },
    },),

    it({
      name: 'accepts a set that satisfies the schema',
      fn: async () => {
        /**
         Store whose schema constrains nested numeric properties.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
          schema: {
            foo: {
              type: 'object',
              properties: {
                bar: {
                  type: 'number',
                },
                foobar: {
                  type: 'number',
                  maximum: 100,
                },
              },
            },
          },
        },);
        expect(function setValidValue(): void {
          conf.set({
            key: 'foo',
            value: {
              bar: 1,
              foobar: 2,
            },
          },);
        },).not.toThrow();
      },
    },),

    it({
      name: 'throws SchemaViolationError when one property violates the schema',
      fn: async () => {
        /**
         Store whose schema types `foo` as a string.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
          schema: {
            foo: {
              type: 'string',
            },
          },
        },);
        expect(function setWrongType(): void {
          conf.set({
            key: 'foo',
            value: 1,
          },);
        },).toThrow(SchemaViolationError,);
        expect(function setWrongType(): void {
          conf.set({
            key: 'foo',
            value: 1,
          },);
        },).toThrow('Config schema violation: `foo` must be string',);
      },
    },),

    it({
      name: 'joins multiple schema violations into one message with "; "',
      fn: async () => {
        /**
         Store whose schema rejects the two properties of one nested object.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
          schema: {
            foo: {
              type: 'object',
              properties: {
                bar: {
                  type: 'number',
                },
                foobar: {
                  type: 'number',
                  maximum: 100,
                },
              },
            },
          },
        },);
        expect(function setTwoViolations(): void {
          conf.set({
            key: 'foo',
            value: {
              bar: '1',
              foobar: 101,
            },
          },);
        },).toThrow(
          'Config schema violation: `foo/bar` must be number; `foo/foobar` must be <= 100',
        );
      },
    },),

    it({
      name: 'reports every violated keyword of a complex schema',
      fn: async () => {
        /**
         Store whose schema combines length,
         pattern,
         array,
         and uniqueness keywords.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
          schema: {
            foo: {
              type: 'string',
              maxLength: 3,
              pattern: '[def]+',
            },
            bar: {
              type: 'array',
              uniqueItems: true,
              maxItems: 3,
              items: {
                type: 'integer',
              },
            },
          },
        },);
        expect(function setTooLongAndUnmatched(): void {
          conf.set({
            key: 'foo',
            value: 'abca',
          },);
        },).toThrow(
          'Config schema violation: `foo` must NOT have more than 3 characters; `foo` must match pattern "[def]+"',
        );
        expect(function setTooManyMixedItems(): void {
          conf.set({
            key: 'bar',
            value: [
              1,
              1,
              2,
              'a',
            ],
          },);
        },).toThrow(
          'Config schema violation: `bar` must NOT have more than 3 items; `bar/3` must be integer; `bar` must NOT have duplicate items (items ## 1 and 0 are identical)',
        );
      },
    },),

    it({
      name: 'validates declared string formats',
      fn: async () => {
        /**
         Store whose schema declares the `uri` format for `foo`.
         */
        const uriConf = createConf({
          cwd: createTempDirectory(),
          schema: {
            foo: {
              type: 'string',
              format: 'uri',
            },
          },
        },);
        expect(function setNonUri(): void {
          uriConf.set({
            key: 'foo',
            value: 'bar',
          },);
        },).toThrow('Config schema violation: `foo` must match format "uri"',);
        expect(function setUri(): void {
          uriConf.set({
            key: 'foo',
            value: 'https://example.com/',
          },);
        },).not.toThrow();

        /**
         Store whose schema declares the `url` format for `foo`.
         */
        const urlConf = createConf({
          cwd: createTempDirectory(),
          schema: {
            foo: {
              type: 'string',
              format: 'url',
            },
          },
        },);
        expect(function setNonUrl(): void {
          urlConf.set({
            key: 'foo',
            value: 'bar',
          },);
        },).toThrow('Config schema violation: `foo` must match format "url"',);
        expect(function setUrl(): void {
          urlConf.set({
            key: 'foo',
            value: 'https://example.com/',
          },);
        },).not.toThrow();
      },
    },),

    it({
      name: 'throws SchemaViolationError when the config file itself violates the schema',
      fn: async () => {
        /**
         Directory whose config file is replaced behind the store.
         */
        const directory = createTempDirectory();
        /**
         Store validating `foo` as a string.
         */
        const conf = createConf({
          cwd: directory,
          schema: {
            foo: {
              type: 'string',
            },
          },
        },);
        writeConfigFile({
          directory,
          data: {
            foo: 1,
          },
        },);
        expect(function readCorruptValue(): unknown {
          return conf.get('foo',);
        },).toThrow(SchemaViolationError,);
        expect(function readCorruptValue(): unknown {
          return conf.get('foo',);
        },).toThrow('Config schema violation: `foo` must be string',);
      },
    },),

    it({
      name: 'throws on schema-invalid config file data when clearInvalidConfig is off',
      fn: async () => {
        /**
         Directory whose config file holds a schema-invalid value.
         */
        const directory = createTempDirectory();
        writeConfigFile({
          directory,
          data: {
            myKey: 'invalid-type',
          },
        },);
        expect(function constructWithoutClearing(): unknown {
          return createConf({
            cwd: directory,
            clearInvalidConfig: false,
            schema: {
              myKey: {
                type: 'boolean',
              },
            },
          },);
        },).toThrow(SchemaViolationError,);
      },
    },),

    it({
      name: 'clears schema-invalid config file data when clearInvalidConfig is on',
      fn: async () => {
        /**
         Directory whose config file holds a schema-invalid value.
         */
        const directory = createTempDirectory();
        writeConfigFile({
          directory,
          data: {
            myKey: 'invalid-type',
          },
        },);
        /**
         Store that clears the invalid data instead of surfacing it.
         */
        const conf = createConf({
          cwd: directory,
          clearInvalidConfig: true,
          schema: {
            myKey: {
              type: 'boolean',
            },
          },
        },);
        expect(conf.store,).toEqual({},);
      },
    },),

    it({
      name: 'throws InvalidSchemaError when the schema option is not an object',
      fn: async () => {
        /**
         Store options carrying a string where the schema object belongs;
         cast because the runtime check under test is what rejects it.
         */
        const mistypedOptions = {
          cwd: createTempDirectory(),
          schema: 'object',
        } as unknown as Parameters<typeof createConf>[0];
        expect(function constructWithMistypedSchema(): unknown {
          return createConf(mistypedOptions,);
        },).toThrow(InvalidSchemaError,);
        expect(function constructWithMistypedSchema(): unknown {
          return createConf(mistypedOptions,);
        },).toThrow('The `schema` option must be an object.',);
      },
    },),

    it({
      name: 'accepts rootSchema keywords without a root properties key',
      fn: async () => {
        /**
         Store whose root schema constrains every property through a pattern.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
          rootSchema: {
            patternProperties: {
              '^.*$': {
                type: 'object',
                properties: {
                  schedule: {
                    type: 'string',
                  },
                  start: {
                    type: 'boolean',
                    default: true,
                  },
                },
              },
            },
          },
        },);
        conf.set({
          key: 'task',
          value: {
            schedule: 'daily',
          },
        },);
        expect(conf.get('task',),).toEqual({
          schedule: 'daily',
          start: true,
        },);
        expect(function setWrongNestedType(): void {
          conf.set({
            key: 'task',
            value: {
              schedule: 1,
            },
          },);
        },).toThrow('Config schema violation: `task/schedule` must be string',);
      },
    },),

    it({
      name: 'throws RootSchemaPropertiesError when rootSchema contains a properties key',
      fn: async () => {
        /**
         Store options whose root schema carries the forbidden `properties` key;
         cast because the runtime check under test is what rejects it.
         */
        const forbiddenRootSchema = {
          cwd: createTempDirectory(),
          rootSchema: {
            properties: {
              foo: {
                type: 'string',
              },
            },
          },
        } as unknown as Parameters<typeof createConf>[0];
        expect(function constructWithForbiddenRootSchema(): unknown {
          return createConf(forbiddenRootSchema,);
        },).toThrow(RootSchemaPropertiesError,);
        expect(function constructWithForbiddenRootSchema(): unknown {
          return createConf(forbiddenRootSchema,);
        },).toThrow(
          'The `rootSchema` option must not contain a `properties` key. Use the `schema` option for properties.',
        );
      },
    },),

    it({
      name: 'enforces rootSchema constraints on the whole store',
      fn: async () => {
        /**
         Store whose root schema forbids every undeclared property.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
          rootSchema: {
            additionalProperties: false,
          },
        },);
        expect(function setUndeclaredKey(): void {
          conf.set({
            key: 'foo',
            value: 'bar',
          },);
        },).toThrow('Config schema violation: `` must NOT have additional properties',);
      },
    },),

    it({
      name: 'passes ajvOptions through to ajv',
      fn: async () => {
        /**
         Store whose root schema forbids undeclared properties while ajv
         removes them instead of rejecting the write.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
          ajvOptions: {
            removeAdditional: true,
          },
          rootSchema: {
            additionalProperties: false,
          },
        },);
        conf.set({
          key: 'foo',
          value: 'bar',
        },);
        expect(conf.get('foo',),).toBeUndefined();
      },
    },),
  ],
},);
