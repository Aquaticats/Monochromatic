import { explicitLink, } from './source-ships-links.ts';
import type {
  TextEdit,
  TextRange,
} from './source-ships-ranges.ts';

//region Source ships addresses
// THE BARE ADDRESSES OF A TEXT, as GFM reads them, because the strict grammar turns one into a link whose text is
// read to the next space or less-than sign: a brace inside it is part of the address and a backslash written there
// would show. GFM reads an address only in text that is not the text of a link, starting with a protocol and a
// domain, or with `www.`. An address that a less-than sign ends is the one the encoder cannot leave alone: MDX reads
// the sign as the start of a tag, a backslash written before it becomes part of the address, and an entity written
// for it is read as part of the address too (GFM ends the address at an entity only when nothing follows), so the
// address is written as a
// link with the same text and target and the sign after it is escaped as any other.
//
// LIMITS. The domain is checked at its first character only (not white space, a control character, punctuation or a
// symbol, a hyphen excepted), not for its dots and underscores, and the trailing punctuation left out of the link is
// the set GFM names plus a closing parenthesis nothing opened, with no check for an entity at the end.

/**
 What GFM reads as the start of an address with a protocol.
 */
const PROTOCOLS: readonly string[] = [
  'http://',
  'https://',
];

/**
 What GFM reads as the start of an address without one.
 */
const WWW = 'www.';

/**
 Characters that may stand before `www.` for GFM to read an address.
 */
const BEFORE_WWW = ' \t\n\r(*_[]~';

/* oxlint-disable no-restricted-syntax/no-regex -- the input is one character, so the test is bounded and cannot
backtrack; the Unicode categories have no string API */
/**
 A character that cannot start a domain: white space, a control character, punctuation or a symbol.
 */
const DOMAIN_STOPS = /[\s\p{C}\p{P}\p{S}]/u;
/* oxlint-enable no-restricted-syntax/no-regex */

/**
 Characters GFM leaves out of an address when they end it.
 */
const TRAILING_PUNCTUATION = '!"\'*,.:;?_~';

/**
 A bare address found in a text.
 */
export type BareAddress = TextRange & {
  /**
   Whether a less-than sign ends the address, which MDX reads as the start of a tag when a backslash stands before it.
   */
  readonly endsAtTag: boolean;
};

/**
 Whether a character is an ASCII letter.

 @param character - one character, or none

 @returns Whether it is `a` to `z` or `A` to `Z`

 @example
 ```ts
 const letter = isAsciiLetter({ character: 'a', },);
 ```
 */
function isAsciiLetter({ character, }: { readonly character: string; },): boolean {
  /**
   Whether it is a lower-case letter.
   */
  const lower = (character >= 'a') && (character <= 'z');
  /**
   Whether it is an upper-case letter.
   */
  const upper = (character >= 'A') && (character <= 'Z');
  return lower || upper;
}

/**
 Whether a character can start the domain of an address, which GFM reads after the protocol.

 @param character - one character, or none

 @returns Whether a domain can start with it

 @example
 ```ts
 const starts = startsDomain({ character: 'e', },);
 ```
 */
function startsDomain({ character, }: { readonly character: string; },): boolean {
  if (character === '')
    return false;
  if (character === '-')
    return true;
  return !DOMAIN_STOPS.test(character,);
}

/**
 Whether GFM reads an address starting at an index.

 @param text - the text

 @param at - index to look at

 @returns Whether a protocol followed by a domain, or `www.`, starts there after a character GFM accepts before it

 @example
 ```ts
 const starts = startsAddress({ text: '见 https://example.net', at: 2, },);
 ```
 */
function startsAddress(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): boolean {
  /**
   The protocol starting here, when one does.
   */
  const protocol = PROTOCOLS.find(function startsHere(candidate,): boolean {
    return text.startsWith(
      candidate,
      at,
    );
  },);
  if (protocol !== undefined) {
    if (isAsciiLetter({ character: text.charAt(at - 1,), },))
      return false;
    return startsDomain({ character: text.charAt(at + protocol.length,), },);
  }
  if (!text.startsWith(
    WWW,
    at,
  ))
    return false;
  return (at === 0) || BEFORE_WWW.includes(text.charAt(at - 1,),);
}

/**
 Where an address ends: at the next space or less-than sign, as the GFM pattern reads it.

 @param text - the text

 @param from - index the address starts at

 @param limit - index the address cannot run past

 @returns Index of the first character that is not part of it, or the limit

 @example
 ```ts
 const stop = addressEnd({ text: 'https://example.net/a b', from: 0, limit: 23, },);
 ```
 */
function addressEnd(
  {
    text,
    from,
    limit,
  }: {
    readonly text: string;
    readonly from: number;
    readonly limit: number;
  },
): number {
  for (let at = from; at < limit; at += 1) {
    /**
     Character read.
     */
    const character = text.charAt(at,);
    if ((character === '<') || (character.trim() === ''))
      return at;
  }
  return limit;
}

/**
 Whether an index lies inside one of the stretches.

 @param at - index to look at

 @param ranges - stretches, in any order

 @returns Whether the index is in one of them

 @example
 ```ts
 const inside = within({ at: 3, ranges: [{ start: 0, end: 10, },], },);
 ```
 */
function within(
  {
    at,
    ranges,
  }: {
    readonly at: number;
    readonly ranges: readonly TextRange[];
  },
): boolean {
  return ranges.some(function holds(
    {
      start,
      end,
    },
  ): boolean {
    return (at >= start) && (at < end);
  },);
}

/**
 Finds the bare addresses inside the stretches a reader sees as text, apart from those in the text of a link, where GFM
 reads none.

 @param text - the text

 @param ranges - stretches read as text

 @param insideLinks - stretches holding a link

 @returns Where each address stands, from its protocol to the next space or less-than sign

 @example
 ```ts
 const addresses = bareAddresses({ text, ranges: [{ start: 0, end: text.length, },], insideLinks: [], },);
 ```
 */
export function bareAddresses(
  {
    text,
    ranges,
    insideLinks,
  }: {
    readonly text: string;
    readonly ranges: readonly TextRange[];
    readonly insideLinks: readonly TextRange[];
  },
): readonly BareAddress[] {
  /**
   Addresses found.
   */
  const found: BareAddress[] = [];
  for (const range of ranges) {
    for (let at = range.start; at < range.end; at += 1) {
      if (!startsAddress({
        text,
        at,
      },))
        continue;
      if (within({
        at,
        ranges: insideLinks,
      },))
        continue;
      /**
       Index where the address ends.
       */
      const stop = addressEnd({
        text,
        from: at,
        limit: range.end,
      },);
      found.push({
        start: at,
        end: stop,
        endsAtTag: text.charAt(stop,) === '<',
      },);
      at = stop - 1;
    }
  }
  return found;
}

/**
 How many times a character stands in a text.

 @param text - the text

 @param character - one character

 @returns The count

 @example
 ```ts
 const opened = countOf({ text: 'a(b(c)', character: '(', },);
 ```
 */
function countOf(
  {
    text,
    character,
  }: {
    readonly text: string;
    readonly character: string;
  },
): number {
  /**
   The pieces between the occurrences.
   */
  const pieces = text.split(character,);
  return pieces.length - 1;
}

/**
 Whether the last character of an address is left out of the link when it ends the address: punctuation GFM names, or a
 closing parenthesis nothing opened.

 @param kept - the address as far as it is kept

 @returns Whether the last character is left out

 @example
 ```ts
 const left = isLeftOut({ kept: 'https://example.net/a.', },);
 ```
 */
function isLeftOut({ kept, }: { readonly kept: string; },): boolean {
  /**
   The last character of the address.
   */
  const last = kept.at(-1,) ?? '';
  if (TRAILING_PUNCTUATION.includes(last,))
    return true;
  if (last !== ')')
    return false;
  return countOf({
    text: kept,
    character: ')',
  },) > countOf({
    text: kept,
    character: '(',
  },);
}

/**
 Where the link an address becomes ends: the address less the punctuation GFM leaves out of one when it ends it, and a
 closing parenthesis nothing opened.

 @param text - the text

 @param address - where the address stands

 @returns Index after the last character of the link

 @example
 ```ts
 const end = linkEnd({ text: 'https://example.net/a.', address: { start: 0, end: 22, }, },);
 ```
 */
function linkEnd(
  {
    text,
    address,
  }: {
    readonly text: string;
    readonly address: TextRange;
  },
): number {
  /**
   Where the link ends so far.
   */
  const end = { at: address.end, };
  while ((end.at > address.start) && isLeftOut({
    kept: text.slice(
      address.start,
      end.at,
    ),
  },))
    end.at -= 1;
  return end.at;
}

/**
 Writes a bare address a less-than sign ends as a link with the same text and target.

 @param text - the text

 @param address - where the address stands

 @returns The edit that replaces the address; the less-than sign after it is escaped as any other

 @example
 ```ts
 const edit = addressAsLink({ text: 'https://example.net/a<b>', address: { start: 0, end: 21, }, },);
 ```
 */
export function addressAsLink(
  {
    text,
    address,
  }: {
    readonly text: string;
    readonly address: TextRange;
  },
): TextEdit {
  /**
   Where the link ends.
   */
  const end = linkEnd({
    text,
    address,
  },);
  /**
   What the address reads.
   */
  const shown = text.slice(
    address.start,
    end,
  );
  return {
    start: address.start,
    end,
    replacement: explicitLink({
      label: shown,
      target: shown.startsWith(WWW,) ? `http://${shown}` : shown,
    },),
  };
}

//endregion Source ships addresses
