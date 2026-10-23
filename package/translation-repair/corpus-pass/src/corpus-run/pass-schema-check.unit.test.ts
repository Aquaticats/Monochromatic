/**
 Tests for the check a pass makes of its stage schemas before it asks any provider (`pass-schema-check.ts`): every
 schema goes through the narrowing the Claude body applies, a schema the route cannot send is refused with the file,
 the schema's own name, the keyword and the path, and one run names every refused schema. The producers are the
 family's own list in the control arm and invented ones in the others. Nothing here reaches a provider.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type { JsonSchemaResponseFormat, } from '@monochromatic-dev/translation-repair-provider';
import {
  assertSchemasSendable,
  CLAUDE_SCHEMA_PRODUCERS,
  type SchemaProducer,
  UnsendableSchemasError,
} from '../../dist/final/node/index.mjs';

/**
 A response format over the given object schema.

 @param name - schema's own name
 @param schema - schema under test

 @returns Format a stage would send

 @example
 ```ts
 const format = formatOf({ name: 'pick', schema: { type: 'object', }, },);
 ```
 */
function formatOf(
  {
    name,
    schema,
  }: {
    readonly name: string;
    readonly schema: Readonly<Record<string, unknown>>;
  },
): JsonSchemaResponseFormat {
  return {
    type: 'json_schema',
    json_schema: {
      name,
      strict: true,
      schema,
    },
  };
}

/**
 What the check threw for these producers, or nothing where it returned.

 @param producers - producers handed to the check

 @returns Thrown value as a list of zero or one

 @example
 ```ts
 const [thrown,] = thrownFor({ producers: [], },);
 ```
 */
function thrownFor({ producers, }: { readonly producers: readonly SchemaProducer[]; },): readonly unknown[] {
  try {
    assertSchemasSendable({ producers, },);
    return [];
  }
  catch (error) {
    return [error,];
  }
}

await describe({
  name: assertSchemasSendable.name,
  children: [
    it({
      name: 'LETS THE FAMILY\'S OWN PRODUCERS THROUGH, every answer schema a stage sends (the control arm)',
      fn: async () => {
        expect(CLAUDE_SCHEMA_PRODUCERS.length > 0,).toBe(true,);
        expect(thrownFor({ producers: CLAUDE_SCHEMA_PRODUCERS, },),).toEqual([],);
      },
    },),

    it({
      name: 'NAMES EVERY REFUSED SCHEMA WITH ITS FILE, NAME, KEYWORD AND PATH, in the order of the producers, and not '
        + 'a schema the route can send',
      fn: async () => {
        /**
         Producers: two the route refuses, one it sends, and one file holding a refused and a sent schema.
         */
        const producers: readonly SchemaProducer[] = [
          {
            file: 'cat/src/pick-wire.ts',
            formats: [formatOf({
              name: 'pick',
              schema: { type: 'object', properties: { pick: { oneOf: [{ type: 'string', },], }, }, },
            },),],
          },
          {
            file: 'cat/src/nap-wire.ts',
            formats: [formatOf({
              name: 'nap',
              schema: { type: 'object', properties: { spot: { type: 'string', }, }, },
            },),],
          },
          {
            file: 'cat/src/bowl-wire.ts',
            formats: [
              formatOf({
                name: 'bowl_plain',
                schema: { type: 'object', properties: { fish: { type: 'string', }, }, },
              },),
              formatOf({ name: 'bowl_loop', schema: { type: 'object', properties: { next: { $ref: '#', }, }, }, },),
            ],
          },
        ];
        const [thrown,] = thrownFor({ producers, },);

        expect(thrown,).toBeInstanceOf(UnsendableSchemasError,);
        expect((thrown as Error).message,).toContain('2 stage schema(s)',);
        expect((thrown as Error).message,).toContain('cat/src/pick-wire.ts (pick): "oneOf" at root.pick',);
        expect((thrown as Error).message,).toContain(
          'cat/src/bowl-wire.ts (bowl_loop): "a recursive $ref" at root.next',
        );
        expect((thrown as Error).message,).not.toContain('nap-wire',);
        expect((thrown as Error).message,).not.toContain('bowl_plain',);
        expect((thrown as Error).name,).toBe('UnsendableSchemasError',);
      },
    },),

    it({
      name: 'RETHROWS A FAILURE THAT IS NOT A REFUSED KEYWORD, so a broken producer is not read as a clean list',
      fn: async () => {
        /**
         Producer whose schema cannot be read.
         */
        const broken: SchemaProducer = {
          file: 'cat/src/broken-wire.ts',
          formats: [{
            type: 'json_schema',
            json_schema: {
              name: 'broken',
              strict: true,
              get schema(): Record<string, unknown> {
                throw new Error('the schema could not be built',);
              },
            },
          },],
        };
        const [thrown,] = thrownFor({ producers: [broken,], },);

        expect(thrown,).toBeInstanceOf(Error,);
        expect(thrown,).not.toBeInstanceOf(UnsendableSchemasError,);
        expect((thrown as Error).message,).toBe('the schema could not be built',);
      },
    },),
  ],
},);
