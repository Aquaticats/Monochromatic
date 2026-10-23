/**
 Tests for the schema the Claude body sends (`claude-schema.ts`). Anthropic's structured outputs take a narrower
 JSON Schema than the stages write: `additionalProperties: false` is required on every object, and the numeric,
 string and array bounds are not supported (the bundled API skill, `shared/tool-use-concepts.md`, "JSON Schema
 Limitations"; the official SDKs strip the bounds and check them client-side, and this client is raw HTTP). The
 sanitiser is the one place the narrowing happens, so every case here reads what it returns and what it leaves alone.
 A stripped bound is told to the model in the field's description, in the form the live Structured outputs page gives
 ("How SDK transformation works", step 2: `{minimum: 100}`), and a keyword the feature refuses that no SDK strips
 stops the build of the body with a named error instead of answering HTTP 400 at the first live call.
 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { claudeSchemaOf, } from '../dist/final/node/index.mjs';

/**
 What a schema is refused with, as text.

 @param schema - schema as a stage would write it

 @returns Error name and message `claudeSchemaOf` throws for it, or empty text where it returns

 @example
 ```ts
 const refusal = refusalTextOf({ schema: { type: 'object', allOf: [{ $ref: '#/$defs/cat', },], }, },);
 ```
 */
function refusalTextOf({ schema, }: { readonly schema: Readonly<Record<string, unknown>>; },): string {
  try {
    claudeSchemaOf({ schema, },);
    return '';
  }
  catch (error) {
    return `${(error as Error).name}: ${(error as Error).message}`;
  }
}

/**
 Builds an object schema with the given number of members, of which a count are required.

 @param members - how many members the object declares

 @param required - how many of them are listed in `required`

 @param union - whether each member is a union typed by a type array

 @returns Schema of that shape

 @example
 ```ts
 const schema = membersSchema({ members: 25, required: 0, union: false, },);
 ```
 */
function membersSchema(
  {
    members,
    required,
    union,
  }: {
    readonly members: number;
    readonly required: number;
    readonly union: boolean;
  },
): Readonly<Record<string, unknown>> {
  /**
   Member names, one per cat.
   */
  const names = Array.from({ length: members, }, function nameOf(_unused, index,): string {
    return `cat${String(index,)}`;
  },);
  return {
    type: 'object',
    properties: Object.fromEntries(names.map(function memberOf(name,): readonly [string, unknown] {
      return [name, union ? { type: ['string', 'null',], } : { type: 'string', },];
    },),),
    required: names.slice(0, required,),
  };
}

await describe({
  name: claudeSchemaOf.name,
  children: [
    it({
      name: 'CLOSES EVERY OBJECT, whether it sits under properties, items, anyOf or a definition, and whether it '
        + 'declares properties or only a type, and replaces an additionalProperties that allows more with false',
      fn: async () => {
        expect(claudeSchemaOf({
          schema: {
            type: 'object',
            properties: {
              lap: { properties: { warm: { type: 'boolean', }, }, },
              naps: { type: 'array', items: { type: 'object', additionalProperties: true, }, },
              either: { anyOf: [{ type: 'object', }, { type: 'string', },], },
            },
            $defs: { bowl: { type: 'object', properties: {}, }, },
          },
        },),).toEqual({
          type: 'object',
          properties: {
            lap: { properties: { warm: { type: 'boolean', }, }, additionalProperties: false, },
            naps: { type: 'array', items: { type: 'object', additionalProperties: false, }, },
            either: { anyOf: [{ type: 'object', additionalProperties: false, }, { type: 'string', },], },
          },
          $defs: { bowl: { type: 'object', properties: {}, additionalProperties: false, }, },
          additionalProperties: false,
        },);
      },
    },),

    it({
      name: 'DROPS EACH BOUND THE FEATURE DOES NOT SUPPORT, numeric, string and array, and keeps the keywords it does '
        + 'support: type, enum, const, required, description, format and a minItems of 1',
      fn: async () => {
        expect(claudeSchemaOf({
          schema: {
            type: 'object',
            properties: {
              age: { type: 'integer', minimum: 1, maximum: 30, exclusiveMinimum: 0, exclusiveMaximum: 31, multipleOf: 1, },
              name: { type: 'string', minLength: 1, maxLength: 20, description: 'what the cat answers to', },
              toys: { type: 'array', minItems: 1, maxItems: 5, uniqueItems: true, items: { type: 'string', enum: ['yarn', 'mouse',], }, },
              kind: { const: 'tabby', },
            },
            required: ['age', 'name',],
          },
        },),).toEqual({
          type: 'object',
          properties: {
            age: {
              type: 'integer',
              description: '{minimum: 1, maximum: 30, exclusiveMinimum: 0, exclusiveMaximum: 31, multipleOf: 1}',
            },
            name: { type: 'string', description: 'what the cat answers to\n\n{minLength: 1, maxLength: 20}', },
            toys: {
              type: 'array',
              items: { type: 'string', enum: ['yarn', 'mouse',], },
              minItems: 1,
              description: '{maxItems: 5, uniqueItems: true}',
            },
            kind: { const: 'tabby', },
          },
          required: ['age', 'name',],
          additionalProperties: false,
        },);
      },
    },),

    it({
      name: 'LEAVES A MEMBER NAMED LIKE A KEYWORD ALONE: a property called minimum or maxLength is a name under '
        + 'properties and not a bound, and an enum or const value that spells a keyword is data',
      fn: async () => {
        expect(claudeSchemaOf({
          schema: {
            type: 'object',
            properties: {
              minimum: { type: 'string', },
              maxLength: { type: 'integer', minimum: 0, },
              pick: { type: 'string', enum: ['minimum', 'uniqueItems',], },
            },
            required: ['minimum',],
          },
        },),).toEqual({
          type: 'object',
          properties: {
            minimum: { type: 'string', },
            maxLength: { type: 'integer', description: '{minimum: 0}', },
            pick: { type: 'string', enum: ['minimum', 'uniqueItems',], },
          },
          required: ['minimum',],
          additionalProperties: false,
        },);
      },
    },),

    it({
      name: 'TELLS THE MODEL A STRIPPED BOUND IN THE FIELD\'S DESCRIPTION (R6), as the SDKs do: minimum 1 on an integer '
        + 'becomes a plain integer described as {minimum: 1}, a bound beside a description is appended to it after a '
        + 'blank line, and a field with no bound gains no description',
      fn: async () => {
        expect(claudeSchemaOf({
          schema: {
            type: 'object',
            properties: {
              paragraph: { type: 'integer', minimum: 1, },
              spot: { type: 'string', description: 'where the cat sleeps', maxLength: 40, },
              mood: { type: 'string', },
            },
          },
        },),).toEqual({
          type: 'object',
          properties: {
            paragraph: { type: 'integer', description: '{minimum: 1}', },
            spot: { type: 'string', description: 'where the cat sleeps\n\n{maxLength: 40}', },
            mood: { type: 'string', },
          },
          additionalProperties: false,
        },);
      },
    },),

    it({
      name: 'REFUSES A KEYWORD THE FEATURE DOES NOT SUPPORT AND NO SDK STRIPS (R10), by a named error that names the '
        + 'keyword and where it sits, so a stage schema that writes oneOf, not, if, '
        + 'patternProperties, contains or prefixItems fails when its body is built and not with HTTP 400 '
        + 'at the first live call',
      fn: async () => {
        /**
         Each refused keyword with a schema that carries it somewhere below the root, and the path it must be named at.
         */
        const cases: readonly {
          readonly keyword: string;
          readonly schema: Readonly<Record<string, unknown>>;
          readonly path: string;
        }[] = [
          {
            keyword: 'oneOf',
            schema: { type: 'object', properties: { pick: { oneOf: [{ type: 'string', }, { type: 'integer', },], }, }, },
            path: 'root.pick',
          },
          {
            keyword: 'not',
            schema: { type: 'object', properties: { naps: { type: 'array', items: { not: { type: 'null', }, }, }, }, },
            path: 'root.naps.items',
          },
          {
            keyword: 'if',
            schema: { type: 'object', properties: { bowl: { if: { type: 'string', }, }, }, },
            path: 'root.bowl',
          },
          {
            keyword: 'prefixItems',
            schema: { type: 'object', properties: { pair: { type: 'array', prefixItems: [{ type: 'string', },], }, }, },
            path: 'root.pair',
          },
          {
            keyword: 'patternProperties',
            schema: { type: 'object', patternProperties: { '^x': { type: 'string', }, }, },
            path: 'root',
          },
          {
            keyword: 'contains',
            schema: { type: 'object', properties: { toys: { type: 'array', contains: { type: 'string', }, }, }, },
            path: 'root.toys',
          },
          {
            keyword: 'oneOf',
            schema: { type: 'object', properties: { either: { anyOf: [{ type: 'string', }, { oneOf: [{ type: 'null', },], },], }, }, },
            path: 'root.either.anyOf[1]',
          },
        ];
        for (const { keyword, schema, path, } of cases) {
          /**
           The failure the schema is refused with.
           */
          let caught: unknown;
          try {
            claudeSchemaOf({ schema, },);
          }
          catch (error) {
            caught = error;
          }
          expect(caught,).toBeInstanceOf(Error,);
          expect((caught as Error).name,).toBe('ClaudeSchemaKeywordError',);
          expect((caught as Error & { readonly messageNamesOnly?: boolean; }).messageNamesOnly,).toBe(true,);
          expect((caught as Error).message,).toContain(`"${keyword}"`,);
          expect((caught as Error).message,).toContain(path,);
        }
      },
    },),

    it({
      name: 'REFUSES NOTHING THE FEATURE SUPPORTS OR THE SDKS STRIP (R10): every keyword on the allowlist passes, and '
        + 'a member named oneOf, or an enum value that spells a refused keyword, is data and not a keyword',
      fn: async () => {
        expect(() =>
          claudeSchemaOf({
            schema: {
              type: 'object',
              title: 'a cat',
              description: 'one cat',
              properties: {
                oneOf: { type: 'string', enum: ['not', 'pattern',], default: 'not', },
                at: { type: 'string', format: 'date-time', },
                kind: { const: 'tabby', },
                both: { allOf: [{ type: 'string', },], },
                nap: { anyOf: [{ type: 'string', }, { type: 'null', },], },
                bowl: { $ref: '#/$defs/bowl', },
              },
              required: ['oneOf',],
              $defs: { bowl: { type: 'string', }, },
              definitions: { dish: { type: 'string', }, },
            },
          },)).not.toThrow();
      },
    },),

    it({
      name: 'SENDS A PATTERN, WHICH THE STRUCTURED OUTPUTS PAGE LISTS AS SUPPORTED (F7), and a keyed call of '
        + '2026-10-22 accepted: it stays on the node beside its type and gains no description',
      fn: async () => {
        expect(claudeSchemaOf({
          schema: {
            type: 'object',
            properties: { name: { type: 'string', pattern: String.raw`^[a-z]+(\d{1,3})?$`, }, },
          },
        },),).toEqual({
          type: 'object',
          properties: { name: { type: 'string', pattern: String.raw`^[a-z]+(\d{1,3})?$`, }, },
          additionalProperties: false,
        },);
      },
    },),

    it({
      name: 'REFUSES A PATTERN WRITTEN WITH A CONSTRUCT THE PAGE LISTS AS NOT SUPPORTED (F7): a backreference, a '
        + 'lookahead, a lookbehind and a word boundary, named by a ClaudeSchemaKeywordError at the node, and leaves a '
        + 'pattern that spells them as data (an escaped backslash, a character class)',
      fn: async () => {
        /**
         Patterns the page refuses, each with the text the error must carry.
         */
        const refused: readonly (readonly [string, string])[] = [
          [String.raw`^(a)\1$`, 'backreference',],
          ['^a(?=b)', 'lookahead',],
          ['^a(?!b)', 'lookahead',],
          ['(?<=a)b', 'lookbehind',],
          ['(?<!a)b', 'lookbehind',],
          [String.raw`\bcat\b`, 'word boundary',],
          [String.raw`\Bcat`, 'word boundary',],
        ];
        refused.forEach(function refuses([pattern, construct,],): void {
          /**
           Failure the schema is refused with.
           */
          const failure = (function refusal(): unknown {
            try {
              claudeSchemaOf({ schema: { type: 'object', properties: { name: { type: 'string', pattern, }, }, }, },);
              return undefined;
            }
            catch (error) {
              return error;
            }
          })();
          expect(failure,).toBeInstanceOf(Error,);
          expect((failure as Error).name,).toBe('ClaudeSchemaKeywordError',);
          expect((failure as Error).message,).toContain(construct,);
          expect((failure as Error).message,).toContain('root.name',);
        },);
        expect(() =>
          claudeSchemaOf({
            schema: {
              type: 'object',
              properties: { name: { type: 'string', pattern: String.raw`^[\b]\\1(?:a|b)(?<kind>c)$`, }, },
            },
          },)).not.toThrow();
      },
    },),

    it({
      name: 'KEEPS minItems OF 0 OR 1, WHICH THE PAGE LISTS AS SUPPORTED, AND TELLS THE MODEL ANY OTHER (F7)',
      fn: async () => {
        expect(claudeSchemaOf({
          schema: {
            type: 'object',
            properties: {
              some: { type: 'array', items: { type: 'string', }, minItems: 1, },
              any: { type: 'array', items: { type: 'string', }, minItems: 0, },
              pair: { type: 'array', items: { type: 'string', }, minItems: 2, },
            },
          },
        },).properties,).toEqual({
          some: { type: 'array', items: { type: 'string', }, minItems: 1, },
          any: { type: 'array', items: { type: 'string', }, minItems: 0, },
          pair: { type: 'array', items: { type: 'string', }, description: '{minItems: 2}', },
        },);
      },
    },),

    it({
      name: 'KEEPS A format ON THE PAGE\'S LIST OF TEN AND TELLS THE MODEL ANY OTHER (F7), as the SDKs filter it',
      fn: async () => {
        expect(claudeSchemaOf({
          schema: {
            type: 'object',
            properties: {
              at: { type: 'string', format: 'date-time', },
              id: { type: 'string', format: 'uuid', },
              tint: { type: 'string', format: 'color', },
            },
          },
        },).properties,).toEqual({
          at: { type: 'string', format: 'date-time', },
          id: { type: 'string', format: 'uuid', },
          tint: { type: 'string', description: '{format: "color"}', },
        },);
      },
    },),

    it({
      name: 'REFUSES A NAMED BACKREFERENCE OUTSIDE A CLASS, which is a backreference as much as a numbered one is'
        + ', and leaves the same two characters inside a class',
      fn: async () => {
        /**
         Refusal for a pattern that repeats a named group.
         */
        const refusal = refusalTextOf({
          schema: { type: 'object', properties: { name: { type: 'string', pattern: String.raw`^(?<n>a)\k<n>$`, }, }, },
        },);
        expect(refusal,).toContain('ClaudeSchemaKeywordError',);
        expect(refusal,).toContain('backreference',);
        expect(refusal,).toContain('root.name',);
        expect(refusalTextOf({
          schema: { type: 'object', properties: { name: { type: 'string', pattern: String.raw`^[\k]$`, }, }, },
        },),).toBe('',);
      },
    },),

    it({
      name: 'REFUSES AN allOf MEMBER THAT HOLDS A $ref, which the page lists as not supported, and sends an allOf '
        + 'of plain schemas',
      fn: async () => {
        /**
         Refusal for an allOf that points at a definition.
         */
        const refusal = refusalTextOf({
          schema: {
            type: 'object',
            properties: { cat: { allOf: [{ $ref: '#/$defs/cat', },], }, },
            $defs: { cat: { type: 'string', }, },
          },
        },);
        expect(refusal,).toContain('ClaudeSchemaKeywordError',);
        expect(refusal,).toContain('allOf',);
        expect(refusal,).toContain('root.cat.allOf[0]',);
        expect(refusalTextOf({
          schema: { type: 'object', properties: { cat: { allOf: [{ type: 'string', },], }, }, },
        },),).toBe('',);
      },
    },),

    it({
      name: 'REFUSES A $ref THAT RESOLVES TO NOTHING, naming the node that holds it, '
        + 'and sends one that resolves',
      fn: async () => {
        /**
         Refusal for a pointer into a definition nobody wrote.
         */
        const refusal = refusalTextOf({
          schema: {
            type: 'object',
            properties: { ghost: { $ref: '#/$defs/nobody', }, },
            $defs: { cat: { type: 'string', }, },
          },
        },);
        expect(refusal,).toContain('ClaudeSchemaKeywordError',);
        expect(refusal,).toContain('names no schema',);
        expect(refusal,).toContain('root.ghost',);
        expect(refusalTextOf({
          schema: {
            type: 'object',
            properties: { cat: { $ref: '#/$defs/cat', }, },
            $defs: { cat: { type: 'string', }, },
          },
        },),).toBe('',);
      },
    },),

    it({
      name: 'REFUSES AN enum MEMBER THAT IS NOT A STRING, A NUMBER, A BOOLEAN OR NULL: the page allows no complex '
        + 'type in an enum',
      fn: async () => {
        for (const member of [{ spot: 'sunbeam', }, ['sunbeam',],]) {
          /**
           Refusal for an enum holding a complex member.
           */
          const refusal = refusalTextOf({
            schema: { type: 'object', properties: { place: { enum: ['lap', member,], }, }, },
          },);
          expect(refusal,).toContain('ClaudeSchemaKeywordError',);
          expect(refusal,).toContain('enum',);
          expect(refusal,).toContain('root.place',);
        }
        expect(refusalTextOf({
          schema: { type: 'object', properties: { place: { enum: ['lap', 2, true, null,], }, }, },
        },),).toBe('',);
      },
    },),

    it({
      name: 'REFUSES A $ref THAT DOES NOT START WITH #, an external reference the page lists as not supported'
        + ', and sends one into the same document',
      fn: async () => {
        /**
         Refusal for a reference to another document.
         */
        const refusal = refusalTextOf({
          schema: { type: 'object', properties: { cat: { $ref: 'http://example.test/cat.json', }, }, },
        },);
        expect(refusal,).toContain('ClaudeSchemaKeywordError',);
        expect(refusal,).toContain('$ref',);
        expect(refusal,).toContain('root.cat',);
        expect(refusalTextOf({
          schema: {
            type: 'object',
            properties: { cat: { $ref: '#/$defs/cat', }, },
            $defs: { cat: { type: 'string', }, },
          },
        },),).toBe('',);
      },
    },),

    it({
      name: 'REFUSES A RECURSIVE $ref CYCLE, whether the reference points at the root or two definitions point at each '
        + 'other, and sends a definition used twice',
      fn: async () => {
        /**
         Schemas that refer back into themselves: to the root, and between two definitions.
         */
        const recursive: readonly Readonly<Record<string, unknown>>[] = [
          { type: 'object', properties: { child: { $ref: '#', }, }, },
          {
            type: 'object',
            properties: { cat: { $ref: '#/$defs/cat', }, },
            $defs: {
              cat: { type: 'object', properties: { kitten: { $ref: '#/$defs/kitten', }, }, },
              kitten: { type: 'object', properties: { cat: { $ref: '#/$defs/cat', }, }, },
            },
          },
        ];
        for (const schema of recursive) {
          /**
           Refusal for the recursive schema.
           */
          const refusal = refusalTextOf({ schema, },);
          expect(refusal,).toContain('ClaudeSchemaKeywordError',);
          expect(refusal,).toContain('recursive',);
        }
        expect(refusalTextOf({
          schema: {
            type: 'object',
            properties: { mother: { $ref: '#/$defs/cat', }, father: { $ref: '#/$defs/cat', }, },
            $defs: { cat: { type: 'string', }, },
          },
        },),).toBe('',);
      },
    },),

    it({
      name: 'REFUSES MORE THAN 24 OPTIONAL PARAMETERS, the page\'s limit for one request, and sends 24',
      fn: async () => {
        /**
         Refusal for 25 members none of which is required.
         */
        const refusal = refusalTextOf({ schema: membersSchema({ members: 25, required: 0, union: false, },), },);
        expect(refusal,).toContain('ClaudeSchemaKeywordError',);
        expect(refusal,).toContain('optional parameters',);
        expect(refusal,).toContain('25',);
        expect(refusalTextOf({ schema: membersSchema({ members: 24, required: 0, union: false, },), },),).toBe('',);
        expect(refusalTextOf({ schema: membersSchema({ members: 40, required: 40, union: false, },), },),).toBe('',);
      },
    },),

    it({
      name: 'REFUSES MORE THAN 16 PARAMETERS THAT USE A UNION TYPE, anyOf or a type array, and sends 16',
      fn: async () => {
        /**
         Refusal for 17 members typed by a type array.
         */
        const refusal = refusalTextOf({ schema: membersSchema({ members: 17, required: 17, union: true, },), },);
        expect(refusal,).toContain('ClaudeSchemaKeywordError',);
        expect(refusal,).toContain('union',);
        expect(refusal,).toContain('17',);
        expect(refusalTextOf({ schema: membersSchema({ members: 16, required: 16, union: true, },), },),).toBe('',);
        /**
         Refusal for 17 members typed by anyOf.
         */
        const anyOfRefusal = refusalTextOf({
          schema: {
            type: 'object',
            properties: Object.fromEntries(Array.from({ length: 17, }, function memberOf(_unused, index,) {
              return [`cat${String(index,)}`, { anyOf: [{ type: 'string', }, { type: 'null', },], },];
            },),),
          },
        },);
        expect(anyOfRefusal,).toContain('union',);
      },
    },),

    it({
      name: 'LEAVES THE CALLER\'S SCHEMA UNTOUCHED, returns a schema that shares no object with it, and passes a '
        + 'boolean schema through',
      fn: async () => {
        /**
         Schema the caller wrote, with a bound and an open object.
         */
        const written = { type: 'object', properties: { age: { type: 'integer', minimum: 1, }, }, };
        /**
         The same schema, to compare after the call.
         */
        const copy = structuredClone(written,);
        const sent = claudeSchemaOf({ schema: written, },);
        expect(written,).toEqual(copy,);
        expect(sent,).not.toBe(written,);
        expect((sent.properties as Record<string, unknown>).age,).not.toBe(written.properties.age,);
        expect(claudeSchemaOf({ schema: { type: 'object', properties: { any: true, }, }, },).properties,).toEqual({ any: true, },);
      },
    },),
  ],
},);
