/**
 Tests for what the Claude schema refuses that no keyword allowlist can see (`claude-schema-refusals.ts`): the
 structured outputs page lists an `allOf` with a `$ref`, a complex `enum` member, an external `$ref` and a recursive
 schema as not supported, and caps one request at 24 optional and 16 union-typed parameters. Each case reads the
 refusal `structuralRefusalOf` returns, as a list of zero or one, and its control arm reads a schema the page accepts.
 `claude-schema.unit.test.ts` reads the same refusals through `claudeSchemaOf`, which throws them.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { structuralRefusalOf, } from '../dist/final/node/index.mjs';

/**
 An object schema with the given members, `required` listing the first of them.

 @param names - member names, each a plain string member
 @param required - how many of the names `required` lists
 @param member - schema every member has

 @returns Schema of that shape

 @example
 ```ts
 const schema = membersOf({ names: ['lap', 'sunbeam',], required: 1, member: { type: 'string', }, },);
 ```
 */
function membersOf(
  {
    names,
    required,
    member,
  }: {
    readonly names: readonly string[];
    readonly required: number;
    readonly member: Readonly<Record<string, unknown>>;
  },
): Readonly<Record<string, unknown>> {
  return {
    type: 'object',
    properties: Object.fromEntries(names.map(function entryOf(name,): readonly [string, unknown] {
      return [name, member,];
    },),),
    required: names.slice(0, required,),
  };
}

/**
 Names `cat0` to `cat` and one less than the count.

 @param count - how many names

 @returns Names in order

 @example
 ```ts
 const names = catNames({ count: 2, },);
 // => ['cat0', 'cat1',]
 ```
 */
function catNames({ count, }: { readonly count: number; },): readonly string[] {
  return Array.from({ length: count, }, function nameOf(_unused, index,): string {
    return `cat${String(index,)}`;
  },);
}

await describe({
  name: structuralRefusalOf.name,
  children: [
    it({
      name: 'NAMES AN allOf MEMBER THAT HOLDS A $ref BY ITS INDEX, and leaves an allOf of plain schemas',
      fn: async () => {
        expect(structuralRefusalOf({
          schema: {
            type: 'object',
            properties: { cat: { allOf: [{ type: 'string', }, { $ref: '#/$defs/cat', },], }, },
            $defs: { cat: { type: 'string', }, },
          },
        },),).toEqual([{ refusedKeyword: 'allOf with a $ref', schemaPath: 'root.cat.allOf[1]', },],);
        expect(structuralRefusalOf({
          schema: { type: 'object', properties: { cat: { allOf: [{ type: 'string', },], }, }, },
        },),).toEqual([],);
      },
    },),

    it({
      name: 'REFUSES AN enum MEMBER THAT IS AN OBJECT OR A LIST, and leaves strings, numbers, Booleans and null',
      fn: async () => {
        expect(structuralRefusalOf({
          schema: { type: 'object', properties: { place: { enum: ['lap', { spot: 'sunbeam', },], }, }, },
        },),).toEqual([{
          refusedKeyword: 'enum with a member that is not a string, a number, a Boolean or null',
          schemaPath: 'root.place',
        },],);
        expect(structuralRefusalOf({
          schema: { type: 'object', properties: { place: { enum: [['lap',],], }, }, },
        },).length,).toBe(1,);
        expect(structuralRefusalOf({
          schema: { type: 'object', properties: { place: { enum: ['lap', 2, true, null,], }, }, },
        },),).toEqual([],);
      },
    },),

    it({
      name: 'REFUSES A $ref THAT LEAVES THE DOCUMENT, and leaves one that starts with #',
      fn: async () => {
        expect(structuralRefusalOf({
          schema: { type: 'object', properties: { cat: { $ref: 'http://example.test/cat.json', }, }, },
        },),).toEqual([{ refusedKeyword: '$ref to another document', schemaPath: 'root.cat', },],);
        expect(structuralRefusalOf({
          schema: {
            type: 'object',
            properties: { cat: { $ref: '#/$defs/cat', }, },
            $defs: { cat: { type: 'string', }, },
          },
        },),).toEqual([],);
      },
    },),

    it({
      name: 'REFUSES A $ref BACK TO THE ROOT AND A CYCLE BETWEEN TWO DEFINITIONS, and follows a definition used twice '
        + 'once and a pointer with an escaped segment, none of which is a cycle',
      fn: async () => {
        expect(structuralRefusalOf({
          schema: { type: 'object', properties: { child: { $ref: '#', }, }, },
        },),).toEqual([{ refusedKeyword: 'a recursive $ref', schemaPath: 'root.child', },],);
        expect(structuralRefusalOf({
          schema: {
            type: 'object',
            properties: { cat: { $ref: '#/$defs/cat', }, },
            $defs: {
              cat: { type: 'object', properties: { kitten: { $ref: '#/$defs/kitten', }, }, },
              kitten: { type: 'object', properties: { cat: { $ref: '#/$defs/cat', }, }, },
            },
          },
        },),).toEqual([{ refusedKeyword: 'a recursive $ref', schemaPath: 'root.cat', },],);
        expect(structuralRefusalOf({
          schema: {
            type: 'object',
            properties: {
              mother: { $ref: '#/$defs/cat', },
              father: { $ref: '#/$defs/cat', },
              uncle: { $ref: '#/$defs/a~1b', },
            },
            $defs: { cat: { type: 'string', }, 'a/b': { type: 'string', }, },
          },
        },),).toEqual([],);
      },
    },),

    it({
      name: 'REFUSES A $ref THAT NAMES NOTHING IN THE DOCUMENT: a pointer into a definition nobody wrote is a '
        + 'reference no grammar can be built from, and was passed here because the walk stopped at a target it could '
        + 'not find. The control arms are a pointer that resolves and a pointer to a Boolean schema',
      fn: async () => {
        expect(structuralRefusalOf({
          schema: {
            type: 'object',
            properties: { ghost: { $ref: '#/$defs/nobody', }, },
            $defs: { cat: { type: 'string', }, },
          },
        },),).toEqual([{ refusedKeyword: 'a $ref that names no schema', schemaPath: 'root.ghost', },],);
        expect(structuralRefusalOf({
          schema: { type: 'object', properties: { ghost: { $ref: '#/nope', }, }, },
        },),).toEqual([{ refusedKeyword: 'a $ref that names no schema', schemaPath: 'root.ghost', },],);
        expect(structuralRefusalOf({
          schema: {
            type: 'object',
            properties: { cat: { $ref: '#/$defs/cat', }, },
            $defs: { cat: { type: 'object', properties: { kitten: { $ref: '#/$defs/missing', }, }, }, },
          },
        },),).toEqual([{ refusedKeyword: 'a $ref that names no schema', schemaPath: 'root.cat', },],);
        expect(structuralRefusalOf({
          schema: {
            type: 'object',
            properties: { cat: { $ref: '#/$defs/cat', }, any: { $ref: '#/$defs/anything', }, },
            $defs: { cat: { type: 'string', }, anything: true, },
          },
        },),).toEqual([],);
      },
    },),

    it({
      name: 'COUNTS EVERY OBJECT\'S MEMBERS NOT LISTED IN required, and refuses past 24 where 24 stands',
      fn: async () => {
        expect(structuralRefusalOf({
          schema: membersOf({ names: catNames({ count: 25, },), required: 0, member: { type: 'string', }, },),
        },),).toEqual([{
          refusedKeyword: '25 optional parameters, where the page allows 24 in one request',
          schemaPath: 'root',
        },],);
        expect(structuralRefusalOf({
          schema: membersOf({ names: catNames({ count: 24, },), required: 0, member: { type: 'string', }, },),
        },),).toEqual([],);
        expect(structuralRefusalOf({
          schema: membersOf({ names: catNames({ count: 30, },), required: 6, member: { type: 'string', }, },),
        },),).toEqual([],);
        expect(structuralRefusalOf({
          schema: {
            type: 'object',
            properties: {
              lap: membersOf({ names: catNames({ count: 13, },), required: 0, member: { type: 'string', }, },),
              bowl: membersOf({ names: catNames({ count: 12, },), required: 0, member: { type: 'string', }, },),
            },
            required: ['lap', 'bowl',],
          },
        },).length,).toBe(1,);
      },
    },),

    it({
      name: 'COUNTS A MEMBER THAT USES anyOf OR A TYPE ARRAY AS A UNION, and refuses past 16 where 16 stands',
      fn: async () => {
        expect(structuralRefusalOf({
          schema: membersOf({
            names: catNames({ count: 17, },),
            required: 17,
            member: { type: ['string', 'null',], },
          },),
        },),).toEqual([{
          refusedKeyword: '17 parameters that use a union type, where the page allows 16 in one request',
          schemaPath: 'root',
        },],);
        expect(structuralRefusalOf({
          schema: membersOf({
            names: catNames({ count: 17, },),
            required: 17,
            member: { anyOf: [{ type: 'string', }, { type: 'null', },], },
          },),
        },).length,).toBe(1,);
        expect(structuralRefusalOf({
          schema: membersOf({
            names: catNames({ count: 16, },),
            required: 16,
            member: { type: ['string', 'null',], },
          },),
        },),).toEqual([],);
        expect(structuralRefusalOf({
          schema: { type: 'object', properties: { spot: true, }, required: ['spot',], },
        },),).toEqual([],);
      },
    },),

    it({
      name: 'REPORTS ONE REFUSAL, A NODE\'S OWN BEFORE A CYCLE BEFORE A COUNT, so the message names the first thing to '
        + 'mend',
      fn: async () => {
        expect(structuralRefusalOf({
          schema: {
            ...membersOf({ names: catNames({ count: 25, },), required: 0, member: { type: 'string', }, },),
            $defs: { loop: { $ref: '#/$defs/loop', }, },
            properties: {
              ...(membersOf({
                names: catNames({ count: 25, },),
                required: 0,
                member: { type: 'string', },
              },).properties as object),
              away: { $ref: 'http://example.test/away.json', },
            },
          },
        },),).toEqual([{ refusedKeyword: '$ref to another document', schemaPath: 'root.away', },],);
      },
    },),
  ],
},);
