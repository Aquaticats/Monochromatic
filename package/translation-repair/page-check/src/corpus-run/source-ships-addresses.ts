import { explicitLink, } from './source-ships-links.ts';
import {
  ESCAPABLE,
  type TextEdit,
  type TextRange,
} from './source-ships-ranges.ts';

//region Source ships addresses
// THE BARE ADDRESSES OF A TEXT, as GFM reads them, because the strict grammar turns one into a link whose text is
// read to the next space or less-than sign: a brace inside it is part of the address and a backslash written there
// would show. GFM reads an address only in text that is not the text of a link, starting with a protocol and a
// domain, or with `www.`. An address that a less-than sign ends is the one the encoder cannot leave alone: MDX reads
// the sign as the start of a tag, a backslash written before it becomes part of the address, and an entity written
// for it is read as part of the address too (GFM ends the address at an entity only when nothing follows), so the
// address is written as a link with the same text and target and the sign after it is escaped as any other.
//
// NO ADDRESS STANDS AFTER A LABEL NOTHING HAS CLOSED, TO THE TOKENIZER. The micromark extension that reads GFM autolink
// literals looks back through the events of the run of inline content for a `[` or `![` whose `]` has not come by
// the time the address starts, and reads no address then (`previousUnbalanced` in
// `micromark-extension-gfm-autolink-literal`), so the text after such a bracket is text to the strict grammar and a
// brace in it is an expression. The brackets of the text are counted from the start of each paragraph, heading or table
// cell, a backslash before a bracket taking it out of the count. The transform of `mdast-util-gfm-autolink-literal`
// reads the text nodes afterwards and links an address there whatever brackets stand before it, which the reader of
// the page sees as a link whose text holds an escaped brace and shows as written.
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
 A stretch of the text to read for addresses, and whether it is literal.
 */
type Stretch = {
  /**
   Where it stands.
   */
  readonly range: TextRange;

  /**
   Where it starts, for ordering.
   */
  readonly start: number;

  /**
   Whether it is a raw HTML block or indented code.
   */
  readonly isLiteral: boolean;
};

/**
 Where the run of inline content that holds an index starts.

 @param streamStarts - where each run starts, in any order

 @param at - index to look at

 @returns The last start at or before the index, or negative one when none is

 @example
 ```ts
 const stream = streamStartAt({ streamStarts: [0, 10,], at: 12, },);
 ```
 */
function streamStartAt(
  {
    streamStarts,
    at,
  }: {
    readonly streamStarts: readonly number[];
    readonly at: number;
  },
): number {
  return streamStarts.reduce(
    function latest(
      found,
      start,
    ): number {
      return ((start <= at) && (start > found)) ? start : found;
    },
    -1,
  );
}

/**
 Finds the bare addresses in the stretches a reader sees as text, apart from those in link text, where GFM reads none,
 and those after a label nothing has closed.

 @param text - the text

 @param ranges - stretches read as text

 @param insideLinks - stretches holding a link

 @param streamStarts - where each paragraph, heading or table cell starts, which a label opened before does not reach

 @param literal - stretches of raw HTML blocks and indented code, where a bracket is escaped in the page and so opens no
 label, and a backslash is a character of the address and no escape

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
    streamStarts = [],
    literal = [],
  }: {
    readonly text: string;
    readonly ranges: readonly TextRange[];
    readonly insideLinks: readonly TextRange[];
    readonly streamStarts?: readonly number[];
    readonly literal?: readonly TextRange[];
  },
): readonly BareAddress[] {
  /**
   Addresses found.
   */
  const found: BareAddress[] = [];
  /**
   The run of inline content being read, and how many labels it has opened that nothing has closed.
   */
  const labels = {
    stream: -1,
    open: 0,
    // Where the stretch read last ends, which a raw HTML block that follows it has no blank line from when it continues
    // that stretch's paragraph.
    readUpTo: 0,
  };
  /**
   Every stretch to read, with whether it is literal, in text order.
   */
  const stretches: readonly Stretch[] = [
    ...ranges.map(function inline(range,): Stretch {
      return {
        range,
        start: range.start,
        isLiteral: false,
      };
    },),
    ...literal.map(function raw(range,): Stretch {
      return {
        range,
        start: range.start,
        isLiteral: true,
      };
    },),
  ].toSorted(function byStart(
    left: Stretch,
    right: Stretch,
  ): number {
    return left.start - right.start;
  },);
  for (const stretch of stretches) {
    /**
     The stretch's range and whether it is literal.
     */
    const {
      range,
      isLiteral,
    } = stretch;
    /**
     The run of inline content the stretch belongs to.
     */
    const stream = streamStartAt({
      streamStarts,
      at: range.start,
    },);
    if (stream !== labels.stream) {
      labels.stream = stream;
      labels.open = 0;
    }
    // A RAW HTML BLOCK AFTER A BLANK LINE STARTS A PARAGRAPH OF ITS OWN, so no label is open in it.
    if (isLiteral && hasBlankLine({
      text: text.slice(
        labels.readUpTo,
        range.start,
      ),
    },))
      labels.open = 0;
    labels.readUpTo = range.end;
    for (let at = range.start; at < range.end; at += 1) {
      /**
       Character read.
       */
      const character = text.charAt(at,);
      if (!isLiteral) {
        if ((character === '\\') && ESCAPABLE.includes(text.charAt(at + 1,),)) {
          at += 1;
          continue;
        }
        if (character === '[')
          labels.open += 1;
        if (character === ']')
          labels.open = Math.max(
            0,
            labels.open - 1,
          );
      }
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
      // AFTER AN OPEN LABEL ONLY THE TRANSFORM READS AN ADDRESS, and it reads one up to a less-than sign that a tag
      // starts at, where the page, with that sign escaped, has it run on; so such an address is a link of its own.
      /**
       Whether a tag's less-than sign ends the address.
       */
      const endsAtTag = text.charAt(stop,) === '<';
      /**
       Whether the address stands clear of any label that is still open.
       */
      const isClear = endsAtTag
        || (labels.open === 0);
      if (!isClear)
        continue;
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
 Whether a blank line stands in a text: a line of indentation and block quote marks only between two line endings.

 @param text - the text

 @returns Whether the text holds a line ending, a line of nothing else, and a line ending

 @example
 ```ts
 const blank = hasBlankLine({ text: '\n\n', },);
 ```
 */
function hasBlankLine({ text, }: { readonly text: string; },): boolean {
  /**
   The lines of the text.
   */
  const lines = text.split('\n',);
  /**
   The lines between the first line ending and the last.
   */
  const between = lines.slice(
    1,
    -1,
  );
  return between.some(function isBlank(line,): boolean {
    for (const character of line) {
      if (!' \t>'.includes(character,))
        return false;
    }
    return true;
  },);
}

/**
 How many times a character stands in a text.

 @param text - the text

 @param character - one character

 @returns Count

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

 @returns Edit that replaces the address; the less-than sign after it is escaped as any other

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
