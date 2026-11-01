import { decodedViewsOf, } from './credential-views.ts';

//region Credential decoded forms
// The forms a credential takes when a client sent it ENCODED and a provider
// echoes it DECODED: a `Basic` header's base64 body read back as `user:token`,
// and a token the client percent-encoded read back with its escapes resolved.
// The mask searches the header as sent; this lists what else the same header
// says once a mechanical decoder has read it.

/**
 Standard base64 alphabet in the order of its values.
 */
const BASE64_ALPHABET = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';

/**
 What the url-safe alphabet writes for the two units the standard one writes
 differently.
 */
const URL_SAFE_UNITS: ReadonlyMap<string, string> = new Map([
  [
    '-',
    '+',
  ],
  [
    '_',
    '/',
  ],
],);

/**
 Bits one base64 character holds.
 */
const BITS_PER_CHARACTER = 6;

/**
 Bits in a byte.
 */
const BITS_PER_BYTE = 8;

/**
 Most `=` units that pad a base64 text.
 */
const MOST_PADDING = 2;

/**
 Remainder a base64 text's length cannot leave modulo its group of four
 characters, since one character holds too few bits for a byte.
 */
const IMPOSSIBLE_REMAINDER = 1;

/**
 Characters in a base64 group.
 */
const GROUP_CHARACTERS = 4;

/**
 First printable ASCII unit, the space, and the last, the tilde.
 */
const PRINTABLE = {
  first: ' ',
  last: '~',
};

/**
 Reads the value of a base64 character in either alphabet.

 @param unit - one-unit text

 @returns Its six bits, or minus one when it is no base64 character

 @example
 ```ts
 sextetOf({ unit: 'A', },); // 0
 ```
 */
function sextetOf({ unit, }: { readonly unit: string; },): number {
  return BASE64_ALPHABET.indexOf(URL_SAFE_UNITS.get(unit,) ?? unit,);
}

/**
 Tells whether a text is printable ASCII throughout.

 @param text - text to check

 @returns True when every unit is between the space and the tilde

 @example
 ```ts
 isPrintable({ text: 'kit:whisker', },); // true
 ```
 */
function isPrintable({ text, }: { readonly text: string; },): boolean {
  return text
    .split('',)
    .every(function isBetween(unit,): boolean {
      return (unit >= PRINTABLE.first) && (unit <= PRINTABLE.last);
    },);
}

/**
 Largest value a byte holds, to cut the bits kept for the next one.
 */
const BYTE_MASK = 0xFF;

/**
 Reads a text as base64 of printable ASCII, in either alphabet, padded or not.

 @param text - text that may be base64

 @returns The text it spells, alone in a list, when every character is base64,
 the length is one a base64 text can have and every byte is printable ASCII;
 an empty list otherwise

 @example
 ```ts
 printableBase64Of({ text: 'a2l0OnNlY3JldC1rZXk=', },); // ['kit:secret-key']
 ```
 */
function printableBase64Of({ text, }: { readonly text: string; },): readonly string[] {
  /**
   The text without its padding.
   */
  const body = text.endsWith('==',)
    ? text.slice(
      0,
      -MOST_PADDING,
    )
    : (text.endsWith('=',)
      ? text.slice(
        0,
        -1,
      )
      : text);
  if ((body.length % GROUP_CHARACTERS) === IMPOSSIBLE_REMAINDER)
    return [];

  /**
   Values of the characters, with minus one for any that is no base64 character.
   */
  const sextets = body
    .split('',)
    .map(function valueOf(unit,): number {
      return sextetOf({ unit, },);
    },);
  if (sextets.includes(-1,))
    return [];

  /**
   Bytes read so far, and the bits of the characters not yet made into one.
   */
  const reading = {
    bytes: '',
    held: 0,
    count: 0,
  };
  for (const sextet of sextets) {
    reading.held = ((reading.held << BITS_PER_CHARACTER) | sextet) & ((BYTE_MASK << BITS_PER_CHARACTER) | BYTE_MASK);
    reading.count += BITS_PER_CHARACTER;
    if (reading.count >= BITS_PER_BYTE) {
      reading.count -= BITS_PER_BYTE;
      reading.bytes += String.fromCodePoint((reading.held >> reading.count) & BYTE_MASK,);
    }
  }
  return isPrintable({ text: reading.bytes, },) ? [reading.bytes,] : [];
}

/**
 Wants every decoded unit, since a header's secret has no needles to hold only some.

 @returns True

 @example
 ```ts
 wantsEveryUnit();
 ```
 */
function wantsEveryUnit(): boolean {
  return true;
}

/**
 Lists the other forms a credential header's secret takes once decoded.

 @param secret - the secret as the header carried it

 @returns Forms that differ from it: what a base64 secret spells when that is
 printable text, each side of its first colon (a `Basic` pair), and the secret
 with its percent or JSON escapes resolved

 @example
 ```ts
 decodedFormsOf({ secret: 'a2l0OnNlY3JldC1rZXk=', },); // ['kit:secret-key', 'kit', 'secret-key']
 ```
 */
export function decodedFormsOf({ secret, }: { readonly secret: string; },): readonly string[] {
  /**
   What the secret spells as base64, when it spells printable text.
   */
  const spelled = printableBase64Of({ text: secret, },);
  return [
    ...decodedViewsOf({
      text: secret,
      wanted: wantsEveryUnit,
    },)
      .map(function textOf(view,): string {
        return view.text;
      },),
    ...spelled,
    ...spelled.flatMap(function sidesOf(pair,): readonly string[] {
      /**
       Where the pair's two halves part.
       */
      const colon = pair.indexOf(':',);
      return (colon === (-1))
        ? []
        : [
          pair.slice(
            0,
            colon,
          ),
          pair.slice(colon + 1,),
        ];
    },),
  ].filter(function differs(form,): boolean {
    return form !== secret;
  },);
}

//endregion Credential decoded forms
