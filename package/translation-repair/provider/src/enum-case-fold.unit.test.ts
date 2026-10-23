/**
 Tests for the rule that takes a reply value differing from a listed `enum` or `const` value only in capitalization back
 to the schema's own spelling, on the value alone: the readings that carry it into a log line and an outcome
 are in `chat-json-outcome-case.unit.test.ts`.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { foldEnumCase, } from '../dist/final/node/index.mjs';

/**
 Schema of a reply with a list of findings, each with a severity the schema lists.
 */
const FINDINGS_SCHEMA = {
  type: 'object',
  properties: {
    mood: { type: 'string', enum: ['calm', 'not calm',], },
    findings: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          severity: { type: 'string', enum: ['minor', 'major',], },
          note: { type: 'string', },
        },
      },
    },
  },
};

await describe({
  name: foldEnumCase.name,
  concurrency: 1,
  children: [
    it({
      name: 'TAKES THE LISTED SPELLING of a string that differs only in capitalization, and names where it sat',
      fn: async () => {
        expect(foldEnumCase({
          schema: FINDINGS_SCHEMA,
          value: { mood: 'Calm', findings: [], },
        },),).toEqual({
          value: {
            mood: 'calm',
            findings: [],
          },
          folds: [
            {
              path: 'reply.mood',
              received: 'Calm',
              taken: 'calm',
            },
          ],
        },);
        expect(foldEnumCase({
          schema: FINDINGS_SCHEMA,
          value: { mood: 'not Calm', },
        },).value,).toEqual({ mood: 'not calm', },);
      },
    },),

    it({
      name: 'NAMES THE POSITION OF A LIST MEMBER in the path, and leaves the members the schema lists exactly',
      fn: async () => {
        expect(foldEnumCase({
          schema: FINDINGS_SCHEMA,
          value: {
            findings: [
              {
                severity: 'minor',
                note: 'Purr',
              },
              {
                severity: 'MAJOR',
                note: 'Hiss',
              },
            ],
          },
        },),).toEqual({
          value: {
            findings: [
              {
                severity: 'minor',
                note: 'Purr',
              },
              {
                severity: 'major',
                note: 'Hiss',
              },
            ],
          },
          folds: [
            {
              path: 'reply.findings[1].severity',
              received: 'MAJOR',
              taken: 'major',
            },
          ],
        },);
      },
    },),

    it({
      name: 'HANDS BACK THE SAME OBJECT where nothing was taken back, and never changes the value handed in',
      fn: async () => {
        /**
         A reply the schema lists exactly.
         */
        const exact = {
          mood: 'calm',
          findings: [{ severity: 'minor', },],
        };
        expect(foldEnumCase({
          schema: FINDINGS_SCHEMA,
          value: exact,
        },).value,).toBe(exact,);

        /**
         A reply with one capital to take back.
         */
        const capitalised = {
          mood: 'Calm',
          findings: [{ severity: 'minor', },],
        };
        const { value, } = foldEnumCase({
          schema: FINDINGS_SCHEMA,
          value: capitalised,
        },);
        expect(value,).not.toBe(capitalised,);
        expect(capitalised.mood,).toBe('Calm',);
      },
    },),

    it({
      name: 'KEEPS A STRING NO LISTED VALUE MATCHES, and one that SEVERAL LISTED VALUES MATCH',
      fn: async () => {
        expect(foldEnumCase({
          schema: FINDINGS_SCHEMA,
          value: { mood: 'sleepy', },
        },),).toEqual({
          value: { mood: 'sleepy', },
          folds: [],
        },);
        expect(foldEnumCase({
          schema: {
            type: 'object',
            properties: { mood: { enum: ['calm', 'Calm', 'not calm',], }, },
          },
          value: { mood: 'CALM', },
        },),).toEqual({
          value: { mood: 'CALM', },
          folds: [],
        },);
      },
    },),

    it({
      name: 'TAKES THE SPELLING OF A LISTED VALUE ONCE where the schema lists it twice',
      fn: async () => {
        expect(foldEnumCase({
          schema: {
            type: 'object',
            properties: { mood: { enum: ['calm', 'calm',], }, },
          },
          value: { mood: 'Calm', },
        },).value,).toEqual({ mood: 'calm', },);
      },
    },),

    it({
      name: 'READS A `const` AS A LIST OF ONE, and ignores a member of an enum that is not a string',
      fn: async () => {
        expect(foldEnumCase({
          schema: {
            type: 'object',
            properties: { kind: { const: 'ballot', }, size: { enum: [1, 2,], }, },
          },
          value: { kind: 'Ballot', size: 1, },
        },).value,).toEqual({ kind: 'ballot', size: 1, },);
      },
    },),

    it({
      name: 'LEAVES A PROPERTY THE SCHEMA DOES NOT DESCRIBE, a node that lists nothing, and a value of another kind',
      fn: async () => {
        expect(foldEnumCase({
          schema: FINDINGS_SCHEMA,
          value: {
            other: 'Calm',
            findings: 'Calm',
          },
        },),).toEqual({
          value: {
            other: 'Calm',
            findings: 'Calm',
          },
          folds: [],
        },);
        expect(foldEnumCase({
          schema: { type: 'string', },
          value: 'Calm',
        },),).toEqual({
          value: 'Calm',
          folds: [],
        },);
        expect(foldEnumCase({
          schema: { enum: ['calm',], },
          value: 3,
        },),).toEqual({
          value: 3,
          folds: [],
        },);
      },
    },),

    it({
      name: 'READS THE REPLY ITSELF when the schema lists its values, at the path reply',
      fn: async () => {
        expect(foldEnumCase({
          schema: { enum: ['calm', 'not calm',], },
          value: 'Not calm',
        },),).toEqual({
          value: 'not calm',
          folds: [
            {
              path: 'reply',
              received: 'Not calm',
              taken: 'not calm',
            },
          ],
        },);
      },
    },),

    it({
      name: 'LEAVES A VALUE WITH SPACE AROUND IT, since the rule is capitalization alone and a leading or trailing '
        + 'space, or a no-break space, is a different value that the stage guard decides',
      fn: async () => {
        /**
         Schema listing one mood, to read each spelling against.
         */
        const schema = {
          type: 'object',
          properties: { mood: { enum: ['major', 'not calm',], }, },
        };
        for (const mood of ['major ', ' Major', 'not\u00A0Calm', 'Major\n',]) {
          expect(foldEnumCase({
            schema,
            value: { mood, },
          },),).toEqual({
            value: { mood, },
            folds: [],
          },);
        }
      },
    },),

    it({
      name: 'COMPARES WITHOUT A LOCALE, so the dotted capital I (U+0130) and the dotless small i (U+0131) match '
        + 'only what the language-neutral lowercase says: the dotted capital lowers to i with a combining dot '
        + '(U+0307), and the dotless i lowers to itself',
      fn: async () => {
        /**
         Reads a reply value against a list of one listed spelling.

         @param listed - spelling the schema lists

         @param received - what the reply gave

         @returns Whatever the fold made of the value

         @example
         ```ts
         const read = readAgainst({ listed: 'id', received: 'Id', },);
         ```
         */
        function readAgainst(
          {
            listed,
            received,
          }: {
            readonly listed: string;
            readonly received: string;
          },
        ): ReturnType<typeof foldEnumCase> {
          return foldEnumCase({
            schema: {
              type: 'object',
              properties: { id: { enum: [listed,], }, },
            },
            value: { id: received, },
          },);
        }

        // The dotted capital I does not fold to a plain i.
        expect(readAgainst({ listed: 'id', received: '\u0130d', },).folds,).toEqual([],);

        // It does fold to the spelling its lowercase is: a plain i and a combining dot (U+0307).
        expect(readAgainst({ listed: 'i\u0307d', received: '\u0130d', },).folds,).toEqual([
          {
            path: 'reply.id',
            received: '\u0130d',
            taken: 'i\u0307d',
          },
        ],);

        // The dotless small i matches neither I nor i, and a capital I does not match a listed dotless i.
        expect(readAgainst({ listed: 'I', received: '\u0131', },).folds,).toEqual([],);
        expect(readAgainst({ listed: 'i', received: '\u0131', },).folds,).toEqual([],);
        expect(readAgainst({ listed: '\u0131d', received: 'Id', },).folds,).toEqual([],);
      },
    },),

    it({
      name: 'FOLDS CAPITALS OF THE GERMAN SHARP S ONLY WHERE THE LOWERCASE ALLOWS: STRASSE matches strasse, '
        + 'does not match a listed strasse with the sharp s, and the capital sharp s (U+1E9E) matches it',
      fn: async () => {
        /**
         Reads a reply value against a list of one listed spelling.

         @param listed - spelling the schema lists

         @param received - what the reply gave

         @returns Values taken back

         @example
         ```ts
         const folds = foldsAgainst({ listed: 'strasse', received: 'STRASSE', },);
         ```
         */
        function foldsAgainst(
          {
            listed,
            received,
          }: {
            readonly listed: string;
            readonly received: string;
          },
        ): ReturnType<typeof foldEnumCase>['folds'] {
          return foldEnumCase({
            schema: { enum: [listed,], },
            value: received,
          },).folds;
        }

        expect(foldsAgainst({ listed: 'strasse', received: 'STRASSE', },),).toEqual([
          {
            path: 'reply',
            received: 'STRASSE',
            taken: 'strasse',
          },
        ],);
        expect(foldsAgainst({ listed: 'stra\u00DFe', received: 'STRASSE', },),).toEqual([],);
        expect(foldsAgainst({ listed: 'stra\u00DFe', received: 'STRA\u1E9EE', },),).toEqual([
          {
            path: 'reply',
            received: 'STRA\u1E9EE',
            taken: 'stra\u00DFe',
          },
        ],);
      },
    },),

    it({
      name: 'LEAVES A NUMBER, A NULL, A BOOLEAN AND A LIST WHERE THE ENUM LISTS STRINGS, and reads only the strings '
        + 'of an enum that mixes kinds',
      fn: async () => {
        for (const mood of [1, null, true, ['Calm',], { mood: 'Calm', },]) {
          expect(foldEnumCase({
            schema: {
              type: 'object',
              properties: { mood: { enum: ['calm', 'not calm',], }, },
            },
            value: { mood, },
          },),).toEqual({
            value: { mood, },
            folds: [],
          },);
        }
        expect(foldEnumCase({
          schema: { enum: ['calm', null, 3,], },
          value: 'Calm',
        },).value,).toBe('calm',);
        expect(foldEnumCase({
          schema: { enum: ['calm', null, 3,], },
          value: null,
        },).value,).toBeNull();
      },
    },),

    it({
      name: 'LEAVES A VALUE UNDER anyOf, oneOf, allOf, $ref, A SCHEMA-VALUED additionalProperties AND A LIST OF '
        + 'ITEMS SCHEMAS AS IT CAME, which is the stated limit of the rule',
      fn: async () => {
        /**
         Node listing a spelling that a reply of `Calm` differs from only in its capital.
         */
        const listing = { enum: ['calm',], };

        /**
         Schemas that put the listing under a form the rule does not follow, each with the reply that reaches it.
         */
        const unfollowed = [
          {
            schema: { type: 'object', properties: { mood: { anyOf: [listing, { type: 'null', },], }, }, },
            value: { mood: 'Calm', },
          },
          {
            schema: { type: 'object', properties: { mood: { oneOf: [listing,], }, }, },
            value: { mood: 'Calm', },
          },
          {
            schema: { type: 'object', properties: { mood: { allOf: [listing,], }, }, },
            value: { mood: 'Calm', },
          },
          {
            schema: {
              type: 'object',
              properties: { mood: { $ref: '#/$defs/mood', }, },
              $defs: { mood: listing, },
            },
            value: { mood: 'Calm', },
          },
          {
            schema: { type: 'object', additionalProperties: listing, },
            value: { mood: 'Calm', },
          },
          {
            schema: { type: 'array', items: [listing,], },
            value: ['Calm',],
          },
        ];
        for (const { schema, value, } of unfollowed) {
          expect(foldEnumCase({ schema, value, },),).toEqual({
            value,
            folds: [],
          },);
        }
      },
    },),
  ],
},);
