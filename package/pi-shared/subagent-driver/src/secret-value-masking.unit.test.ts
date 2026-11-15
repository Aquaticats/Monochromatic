/**
 What the masking replaces, and what it leaves alone: the value itself, the shapes a command prints it in, the marker that names
 it, and the parts of a message it must not disturb.

 Every value here is a stand-in this file makes up. Nothing in this file reads a store, a credential file or an environment.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  createSecretValueMasking,
  MASKED_VALUE_CLOSING,
  MASKED_VALUE_OPENING,
  type MaskedDeeply,
  maskedValueMarker,
  MINIMUM_MASKED_VALUE_LENGTH,
} from '../dist/final/node/index.mjs';

/**
 A stand-in value, long enough to be masked.
 */
const TOKEN_VALUE = 'stand-in-token-value-0123456789';

/**
 The name the marker says in its place.
 */
const TOKEN_NAME = 'STAND_IN_API_KEY';

/**
 A second stand-in value, so a call that masks two shows it masks both.
 */
const OTHER_VALUE = 'stand-in-password-value-abcdefghij';

/**
 The name of the second stand-in.
 */
const OTHER_NAME = 'STAND_IN_PASSWORD';

/**
 The masking of one stand-in value.
 */
const oneValue = createSecretValueMasking({
  values: [{ name: TOKEN_NAME, value: TOKEN_VALUE, },],
},);

/**
 What the text says where the value was.
 */
const MARKER = maskedValueMarker(TOKEN_NAME,);

/**
 The base64 form of the stand-in value, which a command that encodes it prints.
 */
const TOKEN_AS_BASE64 = Buffer.from(
  TOKEN_VALUE,
  'utf8',
).toString('base64',);

/**
 The percent-encoded form of the stand-in value, which a URL prints.
 */
const TOKEN_AS_URL = encodeURIComponent(TOKEN_VALUE,);

await describe({
  name: 'the masking of a secret value',
  children: [
    describe({
      name: createSecretValueMasking.name,
      children: [
        it({
          name: 'replaces the value where it stands, with a marker that names the variable',
          fn: async () => {
            const masked = oneValue.maskText(`MISE_GITHUB_TOKEN=${TOKEN_VALUE}`,);
            expect(masked,).toBe(`MISE_GITHUB_TOKEN=${MARKER}`,);
            expect(masked,).not.toContain(TOKEN_VALUE,);
          },
        },),
        it({
          name: 'replaces a value inside a longer line, keeping the rest of it',
          fn: async () => {
            const masked = oneValue.maskText(
              `export STAND_IN_API_KEY=${TOKEN_VALUE} # a comment about it`,
            );
            expect(masked,).toBe(`export STAND_IN_API_KEY=${MARKER} # a comment about it`,);
          },
        },),
        it({
          name: 'replaces every occurrence of the value, not only the first',
          fn: async () => {
            const masked = oneValue.maskText(`${TOKEN_VALUE} and ${TOKEN_VALUE}`,);
            expect(masked,).toBe(`${MARKER} and ${MARKER}`,);
          },
        },),
        it({
          name: 'replaces the value as JSON escapes it inside a string',
          fn: async () => {
            /**
             A stand-in value holding the characters JSON escapes, so its escaped form differs from the value.
             */
            const escapedValue = 'stand-in-"quoted"-value-0123456789';
            /**
             The masking of that value.
             */
            const escaping = createSecretValueMasking({
              values: [{ name: TOKEN_NAME, value: escapedValue, },],
            },);
            /**
             The value as a JSON file a command printed holds it.
             */
            const printed = JSON.stringify({ STAND_IN_API_KEY: escapedValue, },);
            const masked = escaping.maskText(printed,);
            expect(masked,).toBe(JSON.stringify({ STAND_IN_API_KEY: MARKER, },),);
            expect(masked,).not.toContain('quoted',);
          },
        },),
        it({
          name: 'replaces the value as base64 encodes it',
          fn: async () => {
            const masked = oneValue.maskText(`the encoder printed ${TOKEN_AS_BASE64}`,);
            expect(masked,).toBe(`the encoder printed ${MARKER}`,);
            expect(masked,).not.toContain(TOKEN_AS_BASE64,);
          },
        },),
        it({
          name: 'replaces the value as a URL percent-encodes it',
          fn: async () => {
            /**
             A stand-in value with a character a URL encodes, so the encoded form differs from the value.
             */
            const urlValue = 'stand-in/value+with spaces';
            /**
             The masking of that value.
             */
            const urlMasking = createSecretValueMasking({
              values: [{ name: TOKEN_NAME, value: urlValue, },],
            },);
            const masked = urlMasking.maskText(`https://example.test/?key=${encodeURIComponent(urlValue,)}`,);
            expect(masked,).toBe(`https://example.test/?key=${MARKER}`,);
          },
        },),
        it({
          name: 'masks two values of one text, each with its own marker',
          fn: async () => {
            /**
             The masking of both stand-in values.
             */
            const both = createSecretValueMasking({
              values: [
                { name: TOKEN_NAME, value: TOKEN_VALUE, },
                { name: OTHER_NAME, value: OTHER_VALUE, },
              ],
            },);
            const masked = both.maskText(`${TOKEN_NAME}=${TOKEN_VALUE}\n${OTHER_NAME}=${OTHER_VALUE}`,);
            expect(masked,).toContain(maskedValueMarker(TOKEN_NAME,),);
            expect(masked,).toContain(maskedValueMarker(OTHER_NAME,),);
            expect(masked,).not.toContain(TOKEN_VALUE,);
            expect(masked,).not.toContain(OTHER_VALUE,);
          },
        },),
        it({
          name: 'leaves ordinary text exactly as it was, and answers the same string',
          fn: async () => {
            /**
             Text holding no value, which is what most tool results are.
             */
            const ordinary = 'a build finished in 3.4 s with 12 warnings';
            expect(oneValue.maskText(ordinary,),).toBe(ordinary,);
          },
        },),
        it({
          name: 'does not mask a value shorter than the shortest one it will mask',
          fn: async () => {
            /**
             A value one character shorter than the floor.
             */
            const tooShort = 'x'.repeat(MINIMUM_MASKED_VALUE_LENGTH - 1,);
            /**
             The masking of it.
             */
            const shortMasking = createSecretValueMasking({
              values: [{ name: TOKEN_NAME, value: tooShort, },],
            },);
            expect(shortMasking.patternCount,).toBe(0,);
            expect(shortMasking.maskText(`STAND_IN_API_KEY=${tooShort}`,),).toBe(`STAND_IN_API_KEY=${tooShort}`,);
          },
        },),
        it({
          name: 'keeps the markers it put in place when it runs over its own answer again',
          fn: async () => {
            const once = oneValue.maskText(`STAND_IN_API_KEY=${TOKEN_VALUE}`,);
            expect(oneValue.maskText(once,),).toBe(once,);
          },
        },),
        it({
          name: 'masks the nested strings of a message and keeps what nothing was masked in',
          fn: async () => {
            /**
             A message shaped as Pi hands one to its masking: a role, a content part, and the tool call identifiers around them.
             */
            const message = {
              role: 'toolResult',
              toolCallId: 'call-1',
              content: [{
                type: 'text',
                text: `STAND_IN_API_KEY=${TOKEN_VALUE}`,
              },],
              isError: false,
            };
            /**
             What masking made of it.
             */
            const masked: MaskedDeeply<typeof message> = oneValue.maskDeeply(message,);
            expect(masked.changed,).toBe(true,);
            expect(masked.value.content[0]?.text,).toBe(`STAND_IN_API_KEY=${MARKER}`,);
            expect(masked.value.role,).toBe('toolResult',);
            expect(masked.value.toolCallId,).toBe('call-1',);
            // The message handed in is untouched, so a caller that masks a transcript keeps the one it holds.
            expect(message.content[0]?.text,).toBe(`STAND_IN_API_KEY=${TOKEN_VALUE}`,);
          },
        },),
        it({
          name: 'answers the very value it was given when nothing was masked in it',
          fn: async () => {
            /**
             A message holding no value.
             */
            const message = {
              role: 'assistant',
              content: [{ type: 'text', text: 'done', },],
            };
            /**
             What masking made of it.
             */
            const masked = oneValue.maskDeeply(message,);
            expect(masked.changed,).toBe(false,);
            expect(masked.value,).toBe(message,);
          },
        },),
        it({
          name: 'counts the values and the forms it looks for, which is what the driver compares with Pi',
          fn: async () => {
            expect(oneValue.valueCount,).toBe(1,);
            expect(oneValue.patternCount,).toBeGreaterThan(1,);
          },
        },),
        it({
          name: 'answers every text unchanged when it was armed with no value at all',
          fn: async () => {
            /**
             A masking armed with nothing, which a machine with no source is.
             */
            const none = createSecretValueMasking({ values: [], },);
            expect(none.patternCount,).toBe(0,);
            expect(none.maskText(`STAND_IN_API_KEY=${TOKEN_VALUE}`,),).toBe(`STAND_IN_API_KEY=${TOKEN_VALUE}`,);
          },
        },),
        it({
          name: 'never puts an armed value, or a form of one, inside a marker it writes',
          fn: async () => {
            /**
             A name that is another value's own text, which is the case a marker could disclose a value through.
             */
            const collidingName = 'STAND_IN_SHARED_KEY';
            /**
             The masking of a value whose name is another armed value, and of the value that name holds.
             */
            const colliding = createSecretValueMasking({
              values: [
                { name: collidingName, value: TOKEN_VALUE, },
                { name: 'STAND_IN_SECOND_KEY', value: collidingName, },
              ],
            },);
            /**
             What masking one of the values wrote.
             */
            const masked = colliding.maskText(`${TOKEN_VALUE} and ${collidingName}`,);
            for (const value of [TOKEN_VALUE, collidingName,]) {
              expect(masked,).not.toContain(value,);
            }
            expect(masked,).toContain('[masked secret value of',);
          },
        },),
        it({
          name: 'changes nothing when it runs over its own answer a second time',
          fn: async () => {
            /**
             Every stand-in value in one text, which is what a tool result holding a store's output looks like.
             */
            const text = `${TOKEN_NAME}=${TOKEN_VALUE}\n${OTHER_NAME}=${OTHER_VALUE}\n${TOKEN_AS_BASE64}`;
            /**
             The masking of both values.
             */
            const both = createSecretValueMasking({
              values: [
                { name: TOKEN_NAME, value: TOKEN_VALUE, },
                { name: OTHER_NAME, value: OTHER_VALUE, },
              ],
            },);
            /**
             What one pass wrote.
             */
            const once = both.maskText(text,);
            expect(both.maskText(once,),).toBe(once,);
          },
        },),
        it({
          name: 'looks for more than the value itself, which is what covers an encoder and a URL',
          fn: async () => {
            // This stand-in holds nothing a JSON escape or a URL escapes, so its forms are the value and its base64 forms.
            expect(oneValue.patternCount,).toBeGreaterThan(1,);
            expect(oneValue.maskText(TOKEN_AS_URL,),).toBe(MARKER,);
            expect(oneValue.maskText(TOKEN_AS_BASE64,),).toBe(MARKER,);
          },
        },),
      ],
    },),
  ],
},);
