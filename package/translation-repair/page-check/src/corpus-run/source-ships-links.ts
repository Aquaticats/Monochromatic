import type { TextRange, } from './source-ships-ranges.ts';

//region Source ships links
// LINKS WRITTEN IN MARKDOWN'S OWN SYNTAX where the original had a form MDX reads as a tag: an angle autolink, or a bare
// address that a less-than sign ends. The text of the link and its target are what the original showed and went to.
// An angle autolink inside the text of a link, which cannot hold a link, is written as its address alone.

/**
 Characters other than control characters and spaces that a destination holds as a percent escape, since Markdown reads
 them as the end of it or as its syntax.
 */
const DESTINATION_SYNTAX = '<>(){}\\';

/**
 Characters Markdown reads in the text of a link, and the dollar sign, which the site's math reads as the edge of an
 expression and drops from the text.
 */
const LABEL_SYNTAX = '\\[]{}<>*_`~&$';

/**
 The highest code point of a character that one percent escape stands for.
 */
const ASCII_LIMIT = 0x7F;

/**
 The code point of a space, below which every character is a control character.
 */
const SPACE_CODE = 0x20;

/**
 The base percent escapes write their digits in.
 */
const HEX = 16;

/**
 Writes a character of a destination as the percent escape of its UTF-8 bytes when Markdown reads it as syntax.

 @param character - one character of the destination

 @returns Character, or its escape

 @example
 ```ts
 const written = destinationCharacter({ character: ' ', },);
 ```
 */
function destinationCharacter({ character, }: { readonly character: string; },): string {
  /**
   The character's code point.
   */
  const codePoint = character.codePointAt(0,) ?? 0;
  /**
   Whether Markdown reads the character as the end of a destination or as its syntax.
   */
  const unsafe = (codePoint <= SPACE_CODE)
    || (codePoint === ASCII_LIMIT)
    || DESTINATION_SYNTAX.includes(character,)
    || (character.trim() === '');
  if (!unsafe)
    return character;
  if (codePoint > ASCII_LIMIT)
    return encodeURIComponent(character,);
  /**
   The code point in upper-case hexadecimal.
   */
  const digits = codePoint
    .toString(HEX,)
    .toUpperCase();
  return `%${
    digits.padStart(
      2,
      '0',
    )
  }`;
}

/**
 Writes text with each character Markdown reads as syntax written with a backslash, so that it shows as typed.

 @param label - what the text shows

 @returns Text, a backslash before each character Markdown reads

 @example
 ```ts
 const written = escapedLabel({ label: 'a_b*c', },);
 ```
 */
export function escapedLabel({ label, }: { readonly label: string; },): string {
  return Array.from(label,)
    .map(function writtenCharacter(character,): string {
      return LABEL_SYNTAX.includes(character,) ? `\\${character}` : character;
    },)
    .join('',);
}

/**
 Writes a link in Markdown's own link syntax, so that the strict MDX grammar reads a link with this text and this target
 where the original had an angle autolink or a bare address.

 @param label - what the link shows

 @param target - where the link goes

 @returns Link, the characters Markdown reads written with a backslash in the text and as a percent escape in the target

 @example
 ```ts
 const link = explicitLink({ label: 'https://example.net/a', target: 'https://example.net/a', },);
 ```
 */
export function explicitLink(
  {
    label,
    target,
  }: {
    readonly label: string;
    readonly target: string;
  },
): string {
  /**
   The target with each character Markdown reads as syntax written as a percent escape.
   */
  const destination = Array.from(target,)
    .map(function destinationOf(character,): string {
      return destinationCharacter({ character, },);
    },)
    .join('',);
  return `[${escapedLabel({ label, },)}](${destination})`;
}

/**
 Writes the text of an angle autolink as a link with the same text and target, without the angle brackets MDX reads
 as a tag.

 @param source - text

 @param url - target the parse gives the autolink

 @param range - where the autolink stands

 @returns Link in Markdown's own link syntax

 @example
 ```ts
 const link = linkInsteadOfAutolink({ source, url: node.url, range, },);
 ```
 */
export function linkInsteadOfAutolink(
  {
    source,
    url,
    range,
  }: {
    readonly source: string;
    readonly url: string;
    readonly range: TextRange;
  },
): string {
  return explicitLink({
    label: source.slice(
      range.start + 1,
      range.end - 1,
    ),
    target: url,
  },);
}

/**
 Writes the address of an angle autolink that stands inside the text of a link as text, without the angle brackets MDX
 reads as a tag, since a link cannot hold a link.

 @param source - text

 @param range - where the autolink stands, brackets included

 @returns Address, a backslash before each character Markdown reads

 @example
 ```ts
 const written = autolinkAsText({ source, range, },);
 ```
 */
export function autolinkAsText(
  {
    source,
    range,
  }: {
    readonly source: string;
    readonly range: TextRange;
  },
): string {
  return escapedLabel({
    label: source.slice(
      range.start + 1,
      range.end - 1,
    ),
  },);
}

/**
 Writes an angle autolink as MDX can read it: a link with the same text and target, or, inside the text of a link, which
 holds none, its address alone.

 @param source - text

 @param url - target the parse gives the autolink

 @param range - where the autolink stands, brackets included

 @param insideLink - whether the autolink stands inside the text of a link

 @returns What takes the place of the autolink

 @example
 ```ts
 const written = autolinkWritten({ source, url: node.url, range, insideLink: false, },);
 ```
 */
export function autolinkWritten(
  {
    source,
    url,
    range,
    insideLink,
  }: {
    readonly source: string;
    readonly url: string;
    readonly range: TextRange;
    readonly insideLink: boolean;
  },
): string {
  if (insideLink) {
    return autolinkAsText({
      source,
      range,
    },);
  }
  return linkInsteadOfAutolink({
    source,
    url,
    range,
  },);
}

//endregion Source ships links
