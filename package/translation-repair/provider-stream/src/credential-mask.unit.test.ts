/**
 Tests for masking the credentials a request carried out of a reply body, and
 for listing which header values count as credentials.
 Fixtures are cat-themed invention; the key is the stand-in the quoting-failure
 fixture holds.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  CREDENTIAL_MARKER,
  credentialsOfHeaders,
  maskCredentials,
  maskCredentialsInCutText,
  MINIMUM_CREDENTIAL_UNITS,
  needlesOf,
} from '../dist/final/node/index.mjs';
import { WHISKER_KEY, } from '@monochromatic-dev/translation-repair-test-fixture/ts/quoting-failure.test-fixture.ts';

/**
 Header value carrying the stand-in key behind its scheme word.
 */
const BEARER = `Bearer ${WHISKER_KEY}`;

/**
 Invented key whose characters a percent encoder treats as reserved: a slash, a
 plus, an equals sign and a colon.
 */
const RESERVED_KEY = 'cat/key+odd=7421:meow';

/**
 Writes a text as base64 or base64url, the way a proxy or an HTTP library does.

 @param text - text whose UTF-8 bytes are written

 @param url - whether the url-safe alphabet is used

 @param padded - whether the `=` padding is kept

 @returns Encoded text

 @example
 ```ts
 base64Of({ text: 'cat', url: false, padded: true, },); // 'Y2F0'
 ```
 */
function base64Of(
  {
    text,
    url,
    padded,
  }: {
    readonly text: string;
    readonly url: boolean;
    readonly padded: boolean;
  },
): string {
  /**
   The encoding in the alphabet asked for.
   */
  const encoded = Buffer.from(
    text,
    'utf8',
  ).toString(url ? 'base64url' : 'base64',);
  if (url)
    return padded ? `${encoded}${'='.repeat((4 - (encoded.length % 4)) % 4,)}` : encoded;

  return padded ? encoded : encoded.replaceAll(
    '=',
    '',
  );
}

/**
 Writes every unit of a text as a percent escape.

 @param text - text of ASCII units

 @param lower - whether the hex digits are lower case

 @returns Text with every unit escaped

 @example
 ```ts
 percentAll({ text: 'cat', lower: false, },); // '%63%61%74'
 ```
 */
function percentAll(
  {
    text,
    lower,
  }: {
    readonly text: string;
    readonly lower: boolean;
  },
): string {
  return text.split('',)
    .map(function escapeUnit(unit,): string {
      /**
       Two hex digits of the unit.
       */
      const hex = (unit.codePointAt(0,) ?? 0)
        .toString(16,)
        .padStart(
        2,
        '0',
        );
      return `%${lower ? hex : hex.toUpperCase()}`;
    },)
    .join('',);
}

/**
 Writes every unit of a text as a JSON unicode escape.

 @param text - text of ASCII units

 @param lower - whether the hex digits are lower case

 @returns Text with every unit escaped

 @example
 ```ts
 unicodeAll({ text: 'c', lower: true, },); // '\\u0063'
 ```
 */
function unicodeAll(
  {
    text,
    lower,
  }: {
    readonly text: string;
    readonly lower: boolean;
  },
): string {
  return text.split('',)
    .map(function escapeUnit(unit,): string {
      /**
       Four hex digits of the unit.
       */
      const hex = (unit.codePointAt(0,) ?? 0)
        .toString(16,)
        .padStart(
        4,
        '0',
        );
      return String.raw`\u${lower ? hex : hex.toUpperCase()}`;
    },)
    .join('',);
}

/**
 Masks one text for the stand-in key the way the transport does for a bearer header.

 @param text - reply text

 @returns Reply with every spelling of the key the mask reads masked

 @example
 ```ts
 maskedFor({ text: 'bad key', },); // 'bad key'
 ```
 */
function maskedFor({ text, }: { readonly text: string; },): string {
  return maskCredentials({
    text,
    credentials: credentialsOfHeaders({ headers: { Authorization: BEARER, }, },),
  },);
}

await describe({
  name: 'credential mask',
  children: [
    it({
      name: 'WRITES THE MARKER A READER FINDS, free of every character a JSON string escapes',
      fn: async () => {
        expect(CREDENTIAL_MARKER,).toBe('[masked: credential sent with this request]',);
        expect(MINIMUM_CREDENTIAL_UNITS,).toBe(12,);
        expect(JSON.stringify(CREDENTIAL_MARKER,),).toBe(`"${CREDENTIAL_MARKER}"`,);
      },
    },),
    it({
      name: 'MASKS THE WHOLE HEADER VALUE as one marker, the token inside it not leaving a second',
      fn: async () => {
        expect(maskCredentials({
          text: `rejected ${BEARER} today`,
          credentials: [BEARER, WHISKER_KEY,],
        },),).toBe(`rejected ${CREDENTIAL_MARKER} today`,);
      },
    },),
    it({
      name: 'MASKS THE TOKEN WITHOUT ITS SCHEME WORD and leaves the scheme word standing',
      fn: async () => {
        expect(maskCredentials({
          text: `rejected Bearer ${WHISKER_KEY}`,
          credentials: [WHISKER_KEY,],
        },),).toBe(`rejected Bearer ${CREDENTIAL_MARKER}`,);
      },
    },),
    it({
      name: 'MASKS EVERY OCCURRENCE, overlapping and touching ones included',
      fn: async () => {
        expect(maskCredentials({
          text: `${WHISKER_KEY} and ${WHISKER_KEY}${WHISKER_KEY}!`,
          credentials: [WHISKER_KEY,],
        },),).toBe(`${CREDENTIAL_MARKER} and ${CREDENTIAL_MARKER}!`,);
        expect(maskCredentials({
          text: 'aaaaaaaaaaaaaaaa',
          credentials: ['aaaaaaaaaaaa',],
        },),).toBe(CREDENTIAL_MARKER,);
      },
    },),
    it({
      name: 'RETURNS THE VERY SAME TEXT when no credential occurs in it',
      fn: async () => {
        /**
         Reply that never mentions the key.
         */
        const text = 'the cat napped on the mat';
        expect(maskCredentials({
          text,
          credentials: [WHISKER_KEY,],
        },),).toBe(text,);
        expect(maskCredentials({
          text,
          credentials: [],
        },),).toBe(text,);
      },
    },),
    it({
      name: 'LEAVES A VALUE UNDER THE MINIMUM ALONE and masks one at the minimum',
      fn: async () => {
        /**
         Value one unit under the minimum.
         */
        const short = 'a'.repeat(MINIMUM_CREDENTIAL_UNITS - 1,);
        /**
         Value exactly at the minimum.
         */
        const exact = 'b'.repeat(MINIMUM_CREDENTIAL_UNITS,);
        expect(maskCredentials({
          text: `${short} ${exact}`,
          credentials: [short, exact,],
        },),).toBe(`${short} ${CREDENTIAL_MARKER}`,);
      },
    },),
    it({
      name: 'LEAVES THE TEXT ALONE FOR AN EMPTY CREDENTIAL, which would otherwise match at every position',
      fn: async () => {
        expect(maskCredentials({
          text: 'the cat napped',
          credentials: ['',],
        },),).toBe('the cat napped',);
      },
    },),
    it({
      name: 'KEEPS A JSON STRING VALID where the credential stands against its delimiters and escapes',
      fn: async () => {
        /**
         Key holding a quote, a backslash and a slash, as a header may.
         */
        const odd = String.raw`cat/key"with\odd-7421`;
        /**
         Reply writing the key as a JSON string does, and with the slash escaped.
         */
        const text = `{"a":"${JSON.stringify(odd,).slice(1, -1,)}","b":"${
          JSON.stringify(odd,).slice(1, -1,)
            .split('/',)
            .join(String.raw`\/`,)
        }","c":"${WHISKER_KEY}"}`;
        /**
         What the reply parses to once masked.
         */
        const parsed: unknown = JSON.parse(maskCredentials({
          text,
          credentials: [odd, WHISKER_KEY,],
        },),);
        expect(parsed,).toEqual({
          a: CREDENTIAL_MARKER,
          b: CREDENTIAL_MARKER,
          c: CREDENTIAL_MARKER,
        },);
      },
    },),
    it({
      name: 'MASKS IN A BODY THAT IS NOT JSON, an event frame or a line of prose',
      fn: async () => {
        expect(maskCredentials({
          text: `data: bad key ${WHISKER_KEY}\n\nplain ${WHISKER_KEY}`,
          credentials: [WHISKER_KEY,],
        },),).toBe(`data: bad key ${CREDENTIAL_MARKER}\n\nplain ${CREDENTIAL_MARKER}`,);
      },
    },),
    it({
      name: 'MASKS BETWEEN ASTRAL CHARACTERS without splitting a surrogate pair',
      fn: async () => {
        expect(maskCredentials({
          text: `\u{1F431}${WHISKER_KEY}\u{1F431}\u{1F431}${WHISKER_KEY}`,
          credentials: [WHISKER_KEY,],
        },),).toBe(`\u{1F431}${CREDENTIAL_MARKER}\u{1F431}\u{1F431}${CREDENTIAL_MARKER}`,);
      },
    },),
    it({
      name: 'NEVER SEARCHES THE MARKER it wrote, whatever a credential spells',
      fn: async () => {
        expect(maskCredentials({
          text: `${WHISKER_KEY} masked`,
          credentials: [WHISKER_KEY, 'credential sent',],
        },),).toBe(`${CREDENTIAL_MARKER} masked`,);
      },
    },),

    it({
      name: 'MASKS A KEY A PROVIDER ECHOES PERCENT-ENCODED, whole or in its reserved characters only, in either hex case',
      fn: async () => {
        /**
         The reserved-character key, in each way an encoder may write it.
         */
        const spellings = [
          'cat%2Fkey%2Bodd%3D7421%3Ameow',
          'cat%2fkey%2bodd%3d7421%3ameow',
          'cat%2Fkey+odd%3D7421%3ameow',
          percentAll({ text: RESERVED_KEY, lower: false, },),
          percentAll({ text: RESERVED_KEY, lower: true, },),
        ];
        expect(spellings.map(function maskedSpelling(spelling,): string {
          return maskCredentials({
            text: `rejected ?key=${spelling}&x=1`,
            credentials: [RESERVED_KEY,],
          },);
        },),).toEqual(spellings.map(function expected(): string {
          return `rejected ?key=${CREDENTIAL_MARKER}&x=1`;
        },),);
        expect([
          percentAll({ text: WHISKER_KEY, lower: false, },),
          percentAll({ text: WHISKER_KEY, lower: true, },),
          'whisker%2Dkey%2d7421',
        ].map(function maskedSpelling(spelling,): string {
          return maskedFor({ text: `bad ${spelling} here`, },);
        },),).toEqual([
          `bad ${CREDENTIAL_MARKER} here`,
          `bad ${CREDENTIAL_MARKER} here`,
          `bad ${CREDENTIAL_MARKER} here`,
        ],);
      },
    },),
    it({
      name: 'MASKS A KEY WRITTEN AS A FORM VALUE, the space of its header value a plus',
      fn: async () => {
        expect(maskCredentials({
          text: 'auth=Bearer+whisker-key-7421&x=1',
          credentials: [BEARER,],
        },),).toBe(`auth=${CREDENTIAL_MARKER}&x=1`,);
      },
    },),
    it({
      name: 'MASKS A KEY A PROVIDER ECHOES IN BASE64 AND BASE64URL, padded or not, standing alone',
      fn: async () => {
        expect([false, true,].flatMap(function alphabetOf(url,): readonly string[] {
          return [false, true,].map(function paddingOf(padded,): string {
            return maskedFor({
              text: `bad ${base64Of({ text: WHISKER_KEY, url, padded, },)} here`,
            },);
          },);
        },),).toEqual([
          `bad ${CREDENTIAL_MARKER} here`,
          `bad ${CREDENTIAL_MARKER} here`,
          `bad ${CREDENTIAL_MARKER} here`,
          `bad ${CREDENTIAL_MARKER} here`,
        ],);
      },
    },),
    it({
      name: 'MASKS A KEY INSIDE A LONGER BASE64 RUN at each of the three alignments a prefix of unknown length gives, leaving only the neighbours its bits do not touch',
      fn: async () => {
        /**
         Prefix and suffix lengths in bytes, every residue of three twice over.
         */
        const lengths = [0, 1, 2, 3, 4, 5,];
        /**
         One case per prefix, suffix, alphabet and padding: the text, and what
         the mask must leave, which is every character the key's bits do not touch.
         */
        const cases = lengths.flatMap(function prefixOf(prefix,): readonly {
          readonly text: string;
          readonly expected: string;
        }[] {
          return lengths.flatMap(function suffixOf(suffix,): readonly {
            readonly text: string;
            readonly expected: string;
          }[] {
            return [false, true,].map(function alphabetOf(url,): {
              readonly text: string;
              readonly expected: string;
            } {
              /**
               Bytes around the key, none of them a key's.
               */
              const data = `${'p'.repeat(prefix,)}${WHISKER_KEY}${'s'.repeat(suffix,)}`;
              /**
               Unpadded encoding of the run.
               */
              const encoded = base64Of({ text: data, url, padded: false, },);
              /**
               First character the key's bits touch.
               */
              const first = Math.floor((8 * prefix) / 6);
              /**
               Last character the key's bits touch.
               */
              const last = Math.floor(((8 * (prefix + WHISKER_KEY.length)) - 1) / 6);
              return {
                text: `x ${encoded} y`,
                expected: `x ${encoded.slice(0, first,)}${CREDENTIAL_MARKER}${encoded.slice(last + 1,)} y`,
              };
            },);
          },);
        },);
        expect(cases.map(function maskedCase(each,): string {
          return maskedFor({ text: each.text, },);
        },),).toEqual(cases.map(function expectedCase(each,): string {
          return each.expected;
        },),);
      },
    },),
    it({
      name: 'LEAVES AN EQUALS SIGN BEFORE A BASE64 FIND, which holds no bit of the key, where the find\'s alignment '
        + 'widens it over the character before',
      fn: async () => {
        /**
         Needles of the key whose find widens over one character before it.
         */
        const widening = needlesOf({ credential: WHISKER_KEY, },)
          .filter(function widensBack(needle,): boolean {
            return needle.before === 1;
          },);
        expect(widening.length > 0,).toBe(true,);
        expect(widening.map(function maskedAfterEquals(needle,): string {
          return maskedFor({ text: `key=${needle.text}`, },);
        },),).toEqual(widening.map(function expected(): string {
          return `key=${CREDENTIAL_MARKER}`;
        },),);
      },
    },),
    it({
      name: 'LEAVES EQUALS SIGNS AFTER A BASE64 FIND WHOSE KEY ENDS ON A WHOLE GROUP, which pad no run the key ends, '
        + 'and takes the padding after a find whose key ends inside a group',
      fn: async () => {
        /**
         Needles of the key whose last character holds no bit beyond it.
         */
        const whole = needlesOf({ credential: WHISKER_KEY, },)
          .filter(function endsOnGroup(needle,): boolean {
            return needle.after === 0;
          },);
        expect(whole.length > 0,).toBe(true,);
        expect(whole.map(function maskedBeforeEquals(needle,): string {
          return maskedFor({ text: ` ${needle.text}== `, },);
        },),).toEqual(whole.map(function expected(): string {
          return ` ${CREDENTIAL_MARKER}== `;
        },),);
        /**
         The key alone in padded base64, whose last group it ends inside.
         */
        const padded = base64Of({ text: WHISKER_KEY, url: false, padded: true, },);
        expect(padded.endsWith('=',),).toBe(true,);
        expect(maskedFor({ text: ` ${padded} `, },),).toBe(` ${CREDENTIAL_MARKER} `,);
      },
    },),
    it({
      name: 'MASKS THE DECODED FORM OF A BASIC HEADER, the whole pair and the secret half alike, and its base64 body as sent',
      fn: async () => {
        /**
         Basic header carrying the stand-in key as the password.
         */
        const credentials = credentialsOfHeaders({
          headers: {
            Authorization: `Basic ${base64Of({ text: `kit:${WHISKER_KEY}`, url: false, padded: true, },)}`,
          },
        },);
        expect([
          `echo kit:${WHISKER_KEY}!`,
          `echo ${WHISKER_KEY}!`,
          `echo Basic ${base64Of({ text: `kit:${WHISKER_KEY}`, url: false, padded: true, },)}!`,
        ].map(function maskedEcho(text,): string {
          return maskCredentials({
            text,
            credentials,
          },);
        },),).toEqual([
          `echo ${CREDENTIAL_MARKER}!`,
          `echo ${CREDENTIAL_MARKER}!`,
          `echo ${CREDENTIAL_MARKER}!`,
        ],);
      },
    },),
    it({
      name: 'MASKS A KEY SENT PERCENT-ENCODED AND ECHOED DECODED, and as it was sent',
      fn: async () => {
        /**
         Header whose token the client wrote percent-encoded.
         */
        const credentials = credentialsOfHeaders({ headers: { 'x-api-key': 'cat%2Fkey%2Bodd%3D7421', }, },);
        expect(maskCredentials({
          text: 'echo cat/key+odd=7421 and cat%2Fkey%2Bodd%3D7421',
          credentials,
        },),).toBe(`echo ${CREDENTIAL_MARKER} and ${CREDENTIAL_MARKER}`,);
      },
    },),
    it({
      name: 'MASKS A KEY WHOSE CHARACTERS A JSON WRITER ESCAPED AS UNICODE, in either hex case and with the solidus escaped',
      fn: async () => {
        expect([
          unicodeAll({ text: WHISKER_KEY, lower: true, },),
          unicodeAll({ text: WHISKER_KEY, lower: false, },),
          String.raw`whisker\u002dkey\u002D7421`,
        ].map(function maskedSpelling(spelling,): string {
          return maskedFor({ text: `{"error":"bad ${spelling} here"}`, },);
        },),).toEqual([
          `{"error":"bad ${CREDENTIAL_MARKER} here"}`,
          `{"error":"bad ${CREDENTIAL_MARKER} here"}`,
          `{"error":"bad ${CREDENTIAL_MARKER} here"}`,
        ],);
        expect(maskCredentials({
          text: String.raw`{"error":"bad cat\u002fkey+odd=7421:meow and cat\/key+odd=7421:meow"}`,
          credentials: [RESERVED_KEY,],
        },),).toBe(`{"error":"bad ${CREDENTIAL_MARKER} and ${CREDENTIAL_MARKER}"}`,);
      },
    },),
    it({
      name: 'MASKS A KEY INSIDE A JSON STRING THAT IS ITSELF INSIDE A JSON STRING, escaped once for each',
      fn: async () => {
        /**
         Reply whose error field holds another reply's JSON text.
         */
        const inner = JSON.stringify({ detail: `bad ${unicodeAll({ text: WHISKER_KEY, lower: true, },)} here`, },);
        /**
         The outer reply, which escapes the inner text once more.
         */
        const outer = JSON.stringify({ error: inner, },);
        /**
         What the outer reply parses to, and what the inner text parses to.
         */
        const parsed: unknown = JSON.parse(maskedFor({ text: outer, },),);
        expect(parsed,).toEqual({ error: `{"detail":"bad ${CREDENTIAL_MARKER} here"}`, },);
      },
    },),
    it({
      name: 'MASKS A KEY PERCENT-ENCODED INSIDE A JSON STRING and one in base64 whose slash the JSON writer escaped',
      fn: async () => {
        /**
         Invented key whose base64, one byte in, holds a slash.
         */
        const slashKey = 'meow-???-purr-7421';
        /**
         The key's base64 after a one-byte prefix, unpadded.
         */
        const encoded = base64Of({ text: `k${slashKey}`, url: false, padded: false, },);
        /**
         Reply written by a JSON writer that escapes the solidus.
         */
        const text = JSON.stringify({
          url: 'https://example.test/?key=whisker%2Dkey%2D7421',
          run: encoded,
        },).replaceAll(
          '/',
          String.raw`\/`,
        );
        expect(encoded.includes('/',),).toBe(true,);
        expect(
          JSON.parse(maskCredentials({
            text,
            credentials: [slashKey, WHISKER_KEY,],
          },),),
        ).toEqual({
          url: `https://example.test/?key=${CREDENTIAL_MARKER}`,
          run: `${encoded.slice(0, 1,)}${CREDENTIAL_MARKER}`,
        },);
      },
    },),
    it({
      name: 'STOPS WIDENING A BASE64 FIND AT A UNIT NO BASE64 RUN IS WRITTEN WITH, so a find against a quote or '
        + 'the text\'s edge leaves what lies beyond it',
      fn: async () => {
        /**
         Degenerate key whose base64 reads the same at every alignment.
         */
        const repeated = 'a'.repeat(MINIMUM_CREDENTIAL_UNITS,);
        /**
         Needle of the alignment that widens a character each way.
         */
        const widening = needlesOf({ credential: repeated, },)
          .filter(function widensBothWays(needle,): boolean {
            return (needle.before === 1) && (needle.after === 1);
          },)
          .map(function textOf(needle,): string {
            return needle.text;
          },);
        expect(widening.length,).toBe(2,);
        expect(widening.map(function maskedAgainstQuotes(text,): string {
          return maskCredentials({
            text: `{"k":"${text}"}`,
            credentials: [repeated,],
          },);
        },),).toEqual(widening.map(function expected(): string {
          return `{"k":"${CREDENTIAL_MARKER}"}`;
        },),);
        expect(widening.map(function maskedAtTheEdges(text,): string {
          return maskCredentials({
            text,
            credentials: [repeated,],
          },);
        },),).toEqual(widening.map(function expected(): string {
          return CREDENTIAL_MARKER;
        },),);
      },
    },),
    it({
      name: 'LISTS THE PAIR AND THE PASSWORD OF A BASIC HEADER beside its whole value and token, the user name '
        + 'under the minimum left out, and the resolved form of a percent-escaped token',
      fn: async () => {
        /**
         Base64 body of the pair the stand-in key is the password of.
         */
        const pair = base64Of({ text: `kit:${WHISKER_KEY}`, url: false, padded: true, },);
        expect(credentialsOfHeaders({ headers: { Authorization: `Basic ${pair}`, }, },),).toEqual([
          `Basic ${pair}`,
          pair,
          `kit:${WHISKER_KEY}`,
          WHISKER_KEY,
        ],);
        expect(credentialsOfHeaders({ headers: { 'x-api-key': 'cat%2Fkey%2Bodd%3D7421', }, },),).toEqual([
          'cat%2Fkey%2Bodd%3D7421',
          'cat/key+odd=7421',
        ],);
      },
    },),
    it({
      name: 'MASKS THE OPENING OF A BASE64 SPELLING A CUT TEXT ENDS INSIDE, and the whole spelling before it',
      fn: async () => {
        /**
         The key alone in base64, whose first fifteen characters a cut leaves.
         */
        const run = base64Of({ text: WHISKER_KEY, url: false, padded: false, },);
        expect(maskCredentialsInCutText({
          text: `says ${run} and ${run.slice(0, 15,)}`,
          credentials: [WHISKER_KEY,],
        },),).toBe(`says ${CREDENTIAL_MARKER} and ${CREDENTIAL_MARKER}`,);
      },
    },),
    it({
      name: 'LISTS THE WHOLE VALUE AND THE TOKEN of an authorization header, whatever the name\'s case',
      fn: async () => {
        expect(credentialsOfHeaders({ headers: { authorization: BEARER, }, },),).toEqual([BEARER, WHISKER_KEY,],);
        expect(credentialsOfHeaders({ headers: { Authorization: BEARER, }, },),).toEqual([BEARER, WHISKER_KEY,],);
      },
    },),
    it({
      name: 'LISTS THE VALUE ALONE of an API key header that has no scheme word',
      fn: async () => {
        expect(credentialsOfHeaders({ headers: { 'x-api-key': WHISKER_KEY, }, },),).toEqual([WHISKER_KEY,],);
      },
    },),
    it({
      name: 'LISTS NOTHING for headers no credential travels in, and for a secret under the minimum',
      fn: async () => {
        expect(credentialsOfHeaders({
          headers: {
            'content-type': 'application/json',
            'anthropic-version': '2023-06-01',
            'authorization': 'Bearer ',
            'x-api-key': 'short',
          },
        },),).toEqual([],);
      },
    },),
    it({
      name: 'MASKS THE OPENING OF A CREDENTIAL A CUT TEXT ENDS INSIDE from four units on, and leaves three alone',
      fn: async () => {
        /**
         Endings of the key, from three units to all but the last one, and what
         the cut text becomes for each.
         */
        const lengths = Array.from(
          { length: WHISKER_KEY.length - 3, },
          function unitsAt(_unused, at,): number {
            return at + 3;
          },
        );
        expect(lengths.map(function maskedAt(units,): string {
          return maskCredentialsInCutText({
            text: `says ${WHISKER_KEY.slice(0, units,)}`,
            credentials: [WHISKER_KEY,],
          },);
        },),).toEqual(lengths.map(function expected(units,): string {
          return (units < 4) ? `says ${WHISKER_KEY.slice(0, units,)}` : `says ${CREDENTIAL_MARKER}`;
        },),);
      },
    },),
    it({
      name: 'MASKS WHOLE COPIES AND THE CUT OPENING TOGETHER, and the opening of the header value as written',
      fn: async () => {
        expect(maskCredentialsInCutText({
          text: `${WHISKER_KEY} then ${BEARER.slice(0, 11,)}`,
          credentials: [BEARER, WHISKER_KEY,],
        },),).toBe(`${CREDENTIAL_MARKER} then ${CREDENTIAL_MARKER}`,);
      },
    },),
    it({
      name: 'RETURNS THE VERY SAME CUT TEXT when it holds no copy and ends in no opening of a credential',
      fn: async () => {
        /**
         Text ending on three units of the key, and a word that is no key's.
         */
        const text = 'the cat napped on the mat, whi';
        expect(maskCredentialsInCutText({
          text,
          credentials: [WHISKER_KEY,],
        },),).toBe(text,);
        expect(maskCredentialsInCutText({
          text,
          credentials: [],
        },),).toBe(text,);
      },
    },),
  ],
},);
