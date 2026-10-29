import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

//region Credential views
// Reads a reply text the way a mechanical decoder would, keeping for every unit
// of the decoded text the stretch of the original it came from, so a credential
// found in a decoded reading is masked where it stands in the original.
//
// ONLY ESCAPES THAT COULD MATTER ARE READ. A reply full of `\n` and `\"` holds
// no credential's spelling in them, so a text whose escapes all decode to a
// unit no needle holds (and that opens no further escape) is not decoded at all,
// which keeps the cost of an ordinary JSON reply at that of one search.
//
// TWO DECODERS, CHAINED. A JSON string's escapes (`\u002D`, `\/`, `\"`, `\\`)
// and percent escapes (`%2D`, either hex case). A reply can carry a credential
// under both, one inside the other: a URL inside a JSON string, a JSON text
// inside a JSON string. Every chain of up to three decoders is read, and a
// decoder that finds nothing to decode ends its chain, so the work is one pass
// per decoder that found something, never one per possible chain.

/**
 A text read through decoders, with where each of its units came from.
 */
export type TextView = {
  /**
   The decoded text.
   */
  readonly text: string;

  /**
   For each unit of the decoded text, the index in the original where the
   stretch it was decoded from begins.
   */
  readonly starts: Int32Array;

  /**
   For each unit of the decoded text, the index in the original one past the
   stretch it was decoded from.
   */
  readonly ends: Int32Array;
};

/**
 Test of whether a decoded unit is one a needle holds.
 */
export type WantedUnit = (parameters: { readonly unit: string; },) => boolean;

/**
 Most decoders one chain applies.
 */
const MOST_DECODERS_IN_A_CHAIN = 3;

/**
 Digits a hex number is written with, lower case.
 */
const HEX_DIGITS = '0123456789abcdef';

/**
 Digits of a JSON unicode escape after its `\u`.
 */
const UNICODE_ESCAPE_DIGITS = 4;

/**
 Digits of a percent escape after its `%`.
 */
const PERCENT_ESCAPE_DIGITS = 2;

/**
 Units of a whole JSON unicode escape, `\u` and its digits.
 */
const UNICODE_ESCAPE_WIDTH = 2 + UNICODE_ESCAPE_DIGITS;

/**
 Units of a whole percent escape, `%` and its digits.
 */
const PERCENT_ESCAPE_WIDTH = 1 + PERCENT_ESCAPE_DIGITS;

/**
 Units of a JSON single-character escape, the backslash and its unit.
 */
const SIMPLE_ESCAPE_WIDTH = 2;

/**
 What each hex digit is worth against the one after it.
 */
const HEX_BASE = 16;

/**
 Unit that opens a JSON escape.
 */
const BACKSLASH = '\\';

/**
 Unit that opens a percent escape.
 */
const PERCENT = '%';

/**
 What each JSON single-character escape stands for.
 */
const JSON_SIMPLE_ESCAPES: ReadonlyMap<string, string> = new Map([
  [
    '"',
    '"',
  ],
  [
    '\\',
    '\\',
  ],
  [
    '/',
    '/',
  ],
  [
    'b',
    '\b',
  ],
  [
    'f',
    '\f',
  ],
  [
    'n',
    '\n',
  ],
  [
    'r',
    '\r',
  ],
  [
    't',
    '\t',
  ],
],);

/**
 What the position of an opener holds: an escape and what it stands for, or
 nothing that is one.
 */
type Escape = {
  /**
   Whether an escape begins here.
   */
  readonly found: false;
} | {
  /**
   Whether an escape begins here.
   */
  readonly found: true;

  /**
   The unit it stands for.
   */
  readonly unit: string;

  /**
   Units of text it takes up.
   */
  readonly width: number;
};

/**
 What a position that opens no escape reads as.
 */
const NO_ESCAPE: Escape = { found: false, };

/**
 Tells whether every unit of a text is a hex digit.

 @param digits - text to check

 @returns True when every unit is a hex digit of either case

 @example
 ```ts
 areHexDigits({ digits: '2D', },); // true
 ```
 */
function areHexDigits({ digits, }: { readonly digits: string; },): boolean {
  return digits
    .toLowerCase()
    .split('',)
    .every(function isHex(digit,): boolean {
      return HEX_DIGITS.includes(digit,);
    },);
}

/**
 Reads the number hex digits write.

 @param digits - hex digits of either case

 @returns Number they write

 @example
 ```ts
 numberOfHex({ digits: '2d', },); // 45
 ```
 */
function numberOfHex({ digits, }: { readonly digits: string; },): number {
  return digits
    .toLowerCase()
    .split('',)
    .reduce(
    function append(
      total,
      digit,
    ): number {
      return (total * HEX_BASE) + HEX_DIGITS.indexOf(digit,);
    },
    0,
    );
}

/**
 Reads a JSON escape at a position.

 @param text - text read

 @param at - index of a backslash

 @returns Unit and width, or no unit for a backslash that begins no escape

 @example
 ```ts
 jsonEscapeAt({ text: String.raw`\u002D`, at: 0, },); // { found: true, unit: '-', width: 6, }
 ```
 */
function jsonEscapeAt({
  text,
  at,
}: {
  readonly text: string;
  readonly at: number
},): Escape {
  /**
   Unit after the backslash.
   */
  const marker = text.charAt(at + 1,);
  if (marker === 'u') {
    /**
     The four digits after the `u`.
     */
    const digits = text.slice(
      at + SIMPLE_ESCAPE_WIDTH,
      at + UNICODE_ESCAPE_WIDTH,
    );
    return ((digits.length === UNICODE_ESCAPE_DIGITS) && areHexDigits({ digits, },))
      ? {
        found: true,
        unit: String.fromCodePoint(numberOfHex({ digits, },),),
        width: UNICODE_ESCAPE_WIDTH,
      }
      : NO_ESCAPE;
  }
  /**
   What the single-character escape stands for.
   */
  const unit = JSON_SIMPLE_ESCAPES.get(marker,);
  return (unit === undefined)
    ? NO_ESCAPE
    : {
      found: true,
      unit,
      width: SIMPLE_ESCAPE_WIDTH,
    };
}

/**
 Reads a percent escape at a position.

 @param text - text read

 @param at - index of a percent sign

 @returns Byte as one unit and width, or no unit for a percent sign
 that begins no escape

 @example
 ```ts
 percentEscapeAt({ text: '%2d', at: 0, },); // { found: true, unit: '-', width: 3, }
 ```
 */
function percentEscapeAt({
  text,
  at,
}: {
  readonly text: string;
  readonly at: number
},): Escape {
  /**
   The two digits after the percent sign.
   */
  const digits = text.slice(
    at + 1,
    at + PERCENT_ESCAPE_WIDTH,
  );
  return ((digits.length === PERCENT_ESCAPE_DIGITS) && areHexDigits({ digits, },))
    ? {
      found: true,
      unit: String.fromCodePoint(numberOfHex({ digits, },),),
      width: PERCENT_ESCAPE_WIDTH,
    }
    : NO_ESCAPE;
}

/**
 One decoder: the unit that opens an escape in its writing, and how it reads one.
 */
type Decoder = {
  /**
   The one unit that opens an escape.
   */
  readonly opener: string;

  /**
   Reads the escape at a position.
   */
  readonly escapeAt: (parameters: {
    readonly text: string;
    readonly at: number
  },) => Escape;
};

/**
 The two decoders, JSON string escapes and percent escapes.
 */
const DECODERS: readonly Decoder[] = [
  {
    opener: BACKSLASH,
    escapeAt: jsonEscapeAt,
  },
  {
    opener: PERCENT,
    escapeAt: percentEscapeAt,
  },
];

/**
 Widens a wanted-unit test to the units that open an escape, since decoding
 one is how a chain reaches a unit a needle holds.

 @param wanted - whether a decoded unit is one a needle holds

 @returns Test, true also for a backslash and a percent sign

 @example
 ```ts
 orOpeners({ wanted, },)({ unit: '%', },); // true
 ```
 */
function orOpeners({ wanted, }: { readonly wanted: WantedUnit; },): WantedUnit {
  return function wantedOrOpener({ unit, }: { readonly unit: string; },): boolean {
    return (unit === BACKSLASH) || (unit === PERCENT)
      || wanted({ unit, },);
  };
}

/**
 Tells whether a text holds an escape of one decoder whose decoded unit is wanted.

 @param text - text to look in

 @param decoder - reading of escapes to look for

 @param wanted - whether a decoded unit is wanted

 @returns True when at least one escape decodes to a wanted unit

 @example
 ```ts
 hasWantedEscape({ text, decoder, wanted, },);
 ```
 */
function hasWantedEscape(
  {
    text,
    decoder,
    wanted,
  }: {
    readonly text: string;
    readonly decoder: Decoder;
    readonly wanted: WantedUnit;
  },
): boolean {
  for (
    let at = text.indexOf(decoder.opener,);
    at !== (-1);
    at = text.indexOf(
      decoder.opener,
      at + 1,
    )
  ) {
    /**
     The escape at this opener.
     */
    const escape = decoder.escapeAt({
      text,
      at,
    },);
    if (escape.found && wanted({ unit: escape.unit, },))
      return true;
  }
  return false;
}

/**
 Decodes the wanted escapes of one decoder out of a view.

 @param view - text and where its units came from

 @param decoder - reading of escapes to apply

 @param wanted - whether a decoded unit is one a needle holds or one that opens
 a further escape, so an escape that is not is left alone and a view with no
 escape that is gets no decoding

 @returns Decoded view in a list, or an empty list when no wanted escape was found

 @example
 ```ts
 decodeView({ view, decoder: DECODERS[0], wanted, },);
 ```
 */
function decodeView(
  {
    view,
    decoder,
    wanted,
  }: {
    readonly view: TextView;
    readonly decoder: Decoder;
    readonly wanted: WantedUnit;
  },
): readonly TextView[] {
  if (!hasWantedEscape({
    text: view.text,
    decoder,
    wanted,
  },))
    return [];

  /**
   Pieces of the decoded text, plain runs and decoded units in order.
   */
  const pieces: string[] = [];
  /**
   Where each decoded unit came from, sized for a text that decodes nothing.
   */
  const starts = new Int32Array(view.text
    .length,);
  /**
   One past where each decoded unit came from.
   */
  const ends = new Int32Array(view.text
    .length,);
  /**
   Read position and write position.
   */
  const cursor = {
    read: 0,
    write: 0,
  };
  while (cursor.read
    < view.text
      .length) {
    /**
     Next position that opens an escape, or the text's end.
     */
    const found = view.text
      .indexOf(
      decoder.opener,
      cursor.read,
      );
    /**
     End of the plain run before it.
     */
    const stop = (found === (-1)) ? view.text
      .length : found;
    pieces.push(view.text
      .slice(
      cursor.read,
      stop,
      ),);
    // The plain run's origins, copied in one move each: a typed array's own
    // copy from an offset, where the write never runs past the read.
    starts.set(
      view.starts
        .subarray(
        cursor.read,
        stop,
        ),
      cursor.write,
    );
    ends.set(
      view.ends
        .subarray(
        cursor.read,
        stop,
        ),
      cursor.write,
    );
    cursor.write += stop - cursor.read;
    cursor.read = stop;
    if (found === (-1))
      break;

    /**
     The escape at the opener.
     */
    const escape = decoder.escapeAt({
      text: view.text,
      at: found,
    },);
    /**
     What the opener stands for once the unwanted escapes are left as they are.
     */
    const decoded = (escape.found && wanted({ unit: escape.unit, },)) ? escape : NO_ESCAPE;
    /**
     Units of the read text this write stands for.
     */
    const width = decoded.found ? decoded.width : 1;
    pieces.push(decoded.found ? decoded.unit : decoder.opener,);
    starts[cursor.write] = nonNullishOrThrow(view.starts[found],);
    ends[cursor.write] = nonNullishOrThrow(view.ends[(found + width) - 1],);
    cursor.write += 1;
    cursor.read = found + width;
  }
  return [
    {
      text: pieces.join('',),
      starts: starts.subarray(
        0,
        cursor.write,
      ),
      ends: ends.subarray(
        0,
        cursor.write,
      ),
    },
  ];
}

/**
 Reads a text through every chain of up to three decoders that finds a wanted
 escape.

 @param text - reply text, of any shape

 @param wanted - whether a decoded unit is one a needle holds; the units that open
 a further escape are wanted always

 @returns One view per chain that decoded something wanted and read a text no
 other chain read; none when the text holds no such escape

 @example
 ```ts
 decodedViewsOf({ text: String.raw`whisker\u002Dkey`, wanted, },); // one view, 'whisker-key'
 ```
 */
export function decodedViewsOf(
  {
    text,
    wanted,
  }: {
    readonly text: string;
    readonly wanted: WantedUnit;
  },
): readonly TextView[] {
  /**
   The test every chain applies, which also wants the units that open an escape.
   */
  const wantedOrOpening = orOpeners({ wanted, },);
  if (!DECODERS.some(function findsInText(decoder,): boolean {
    return hasWantedEscape({
      text,
      decoder,
      wanted: wantedOrOpening,
    },);
  },))
    return [];

  /**
   The text as its own first view, each unit from itself.
   */
  const identity: TextView = {
    text,
    starts: new Int32Array(text.length,),
    ends: new Int32Array(text.length,),
  };
  for (let at = 0; at < text.length; at += 1) {
    identity.starts[at] = at;
    identity.ends[at] = at + 1;
  }

  /**
   Views found.
   */
  const found: TextView[] = [];
  /**
   The views of the chain length now being read.
   */
  const frontier: { views: readonly TextView[]; } = { views: [identity,], };
  for (let depth = 0; depth < MOST_DECODERS_IN_A_CHAIN; depth += 1) {
    /**
     Views one decoder further.
     */
    const next = frontier.views
      .flatMap(function decodeEach(view,): readonly TextView[] {
        return DECODERS.flatMap(function decodeWith(decoder,): readonly TextView[] {
          return decodeView({
            view,
            decoder,
            wanted: wantedOrOpening,
          },);
        },);
      },);
    /**
     Views whose text no earlier view has, since two chains that decode
     independent escapes in either order read the same text.
     */
    const fresh = next.filter(function isNew(
      view,
      at,
    ): boolean {
      return ![
        ...found,
        ...next.slice(
          0,
          at,
        ),
      ].some(function sameText(other,): boolean {
        return other.text === view.text;
      },);
    },);
    found.push(...fresh,);
    frontier.views = fresh;
  }
  return found;
}

//endregion Credential views
