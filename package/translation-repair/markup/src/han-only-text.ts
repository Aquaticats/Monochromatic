import { isAsciiLetter, } from './ascii-letters.ts';

//region Han-only text
// Character tests shared by the floors that ask whether a run of text is
// written in Han alone, and by the tokenizer (`preservation-tokens.ts`),
// whose ideograph test is this one: every block of Han ideographs, and ASCII
// letters as the mark of a Latin form.
//
// ONE DEFINITION OF HAN (audit area six, 2026-09-28). The floors read the
// unified block alone and the tokenizer Extension A as well, so the two read
// one page two ways; neither read the compatibility block or astral Han. No
// pinned page carries a character from those blocks, so no page read
// differently; the next one to would have.

/**
 Blocks of Han ideographs, first and last code point of each: Extension A,
 the unified block, the compatibility block, and Extensions B to H.
 */
const HAN_BLOCKS: readonly {
  readonly first: number;
  readonly last: number;
}[] = [
  {
    first: 0x34_00,
    last: 0x4D_BF,
  },
  {
    first: 0x4E_00,
    last: 0x9F_FF,
  },
  {
    first: 0xF9_00,
    last: 0xFA_FF,
  },
  {
    first: 0x2_00_00,
    last: 0x3_13_4F,
  },
];

/**
 Whether a character is a Han ideograph.

 @param character - one code point of a text; a lone surrogate half is no
 ideograph

 @returns True inside any block of Han ideographs

 @example
 ```ts
 isHanCharacter({ character: '猫', },); // true
 ```
 */
export function isHanCharacter({ character, }: { readonly character: string; },): boolean {
  /**
   Code point of the character, none for an empty string.
   */
  const point = character.codePointAt(0,) ?? (-1);
  return HAN_BLOCKS.some(function holds({
    first,
    last,
  },): boolean {
    return (point >= first) && (point <= last);
  },);
}

/**
 Whether a text carries a Han ideograph and no ASCII letter, so its only
 form is Han.

 @param text - run of text under the question

 @returns True for a run whose only form is Han

 @example
 ```ts
 isHanOnly({ text: '猫猫摇篮曲', },); // true
 isHanOnly({ text: 'Nyan物语', },); // false
 ```
 */
export function isHanOnly({ text, }: { readonly text: string; },): boolean {
  /**
   Whether a Han ideograph has been read.
   */
  let han = false;
  for (const character of text) {
    if (isAsciiLetter({ character, },))
      return false;
    if (isHanCharacter({ character, },))
      han = true;
  }
  return han;
}

/**
 Whether a text carries a Han ideograph at all, whatever else it carries.

 @param text - run of text under the question

 @returns True for a run with some Han in it

 @example
 ```ts
 carriesHan({ text: 'Nyan物语', },); // true
 carriesHan({ text: 'Nyan', },); // false
 ```
 */
export function carriesHan({ text, }: { readonly text: string; },): boolean {
  for (const character of text) {
    if (isHanCharacter({ character, },))
      return true;
  }
  return false;
}

/**
 Whether a span of a text carries an ASCII letter.

 @param text - text read

 @param from - offset the span starts at

 @param to - offset just past the span

 @returns True when some character of the span is an ASCII letter

 @example
 ```ts
 carriesAsciiLetter({ text: '猫 (cat)', from: 3, to: 6, },); // true
 ```
 */
export function carriesAsciiLetter(
  {
    text,
    from,
    to,
  }: {
    readonly text: string;
    readonly from: number;
    readonly to: number;
  },
): boolean {
  for (let at = from; at < to; at += 1) {
    if (isAsciiLetter({ character: text.charAt(at,), },))
      return true;
  }
  return false;
}

//endregion Han-only text
