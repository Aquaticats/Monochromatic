import { isAsciiAlphanumeric, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Channel marker tails
// A provider emits special tokens of the shape `<|word|>` and filters them out
// of streamed content. The filter is not atomic across event boundaries, so
// when a token straddles two deltas a TAIL of it survives and lands in front of
// otherwise correct JSON.
//
// Observed on `hf:moonshotai/Kimi-K3`: `|>` on 2026-08-12, then `p|>` and
// `ep|>` on 2026-08-13, all three being suffixes of the same `<|...|>` shape.
// The first fix matched the exact string `|>`, which is what survived on the
// first day, and therefore stopped working the moment the surviving tail grew
// by one character, which is how 21 of the 23 voices lost in the most recent
// run window were lost.
//
// SO THE RULE IS THE SHAPE, NOT A VOCABULARY. Matching a list of known tokens
// would require knowing the provider's special-token set, which we do not and
// cannot read from here; every tail of every marker is covered by checking that
// what sits in front of the JSON could only be the end of a `<|word|>`. The
// cost of the shape rule is that it also strips a fragment no real token ever
// produced, such as `xep|>`; that costs nothing, because the alternative to
// stripping it is failing to parse.
//
// The property the previous implementation protected is kept: this refuses to
// be a general "skip junk until the first opening brace" rule, which would
// swallow a model prefixing an apology or partial refusal and turn content the
// refusal detector must classify into a silent parse success. An apology
// carries spaces and does not close with `|>` inside a dozen characters, so it
// still fails to parse, loudly.
//
// DELIBERATELY OUT OF SCOPE: a leak of `>` alone, with the `|` also consumed.
// Every observed tail closes with `|>`, and accepting a bare `>` would also
// strip a Markdown blockquote marker ahead of JSON. Should it ever happen it
// arrives diagnosable rather than silent, because the caller logs the opening
// characters of anything that fails to parse.

/**
 Longest single marker tail considered.

 `<|im_start|>` is twelve characters, so a whole marker of that family fits
 and anything longer is prose that happens to contain the closing characters.
 */
const MARKER_TAIL_LIMIT = 12;

/**
 Most consecutive markers consumed before the run is treated as prose.

 More than one can leak when two tokens straddle the same delta boundary. The
 bound exists so a pathological input cannot turn this into an unbounded scan.
 */
const MARKER_RUN_LIMIT = 4;

/**
 Characters every marker of this family ends with.
 */
const MARKER_CLOSE = '|>';

/**
 Characters a whole, untruncated marker begins with.

 Optional when matching, precisely because the interesting case is the one
 where the opening did not survive.
 */
const MARKER_OPEN = '<|';

/**
 Openings that mean the marker run has ended and real content has started.

 A fence is included because the fence stripper runs before this and cannot
 see a fence hidden behind a marker; the caller unwraps it afterwards.
 */
const CONTENT_OPENINGS: readonly string[] = [
  '{',
  '[',
  '```',
];

/**
 Reports whether a character can appear in a marker's name.

 Compared by code point rather than by pattern, since the alphabet is three
 contiguous ranges plus one character and an index scan states that directly.

 @param character - single character from the candidate marker

 @returns True for ASCII letters, digits and underscore

 @example
 ```ts
 const ok = isMarkerBodyCharacter({ character: 'p', },);
 ```
 */
function isMarkerBodyCharacter(
  {
    character,
  }: {
    readonly character: string;
  },
): boolean {
  return isAsciiAlphanumeric({ character, },) || (character === '_');
}

/**
 Measures one marker sitting at the start of the text.

 @param text - candidate text, already trimmed at its start

 @returns Length of the marker, zero when the text does not open with one

 @example
 ```ts
 const width = markerWidth({ text: 'ep|>{"count":2}', },);
 ```
 */
function markerWidth(
  {
    text,
  }: {
    readonly text: string;
  },
): number {
  /**
   Where the marker would close, taking the FIRST closing characters so a
   `|>` occurring later inside the JSON cannot extend the candidate.
   */
  const closeAt = text.indexOf(MARKER_CLOSE,);
  if (closeAt === (-1))
    return 0;

  /**
   Length of the whole candidate marker, closing characters included.
   */
  const width = closeAt + MARKER_CLOSE.length;
  if (width > MARKER_TAIL_LIMIT)
    return 0;

  /**
   Candidate marker with its closing characters removed.
   */
  const head = text.slice(
    0,
    closeAt,
  );

  /**
   Marker name, with the opening characters removed when they survived.
   */
  const body = head.startsWith(MARKER_OPEN,)
    ? head.slice(MARKER_OPEN.length,)
    : head;

  for (const character of body) {
    if (!isMarkerBodyCharacter({ character, },))
      return 0;
  }

  return width;
}

/**
 Content with any marker run removed, and the run that was removed.

 `marker` is empty when nothing was stripped, rather than absent, because the
 caller logs it and an empty string reads as "nothing to report" at the call
 site without a nullish check.

 @example
 ```ts
 const { content, marker, } = stripChannelMarker({ text: 'p|>{"count":2}', },);
 ```
 */
export type ChannelMarkerStrip = {
  /**
   Content the parser should see.
   */
  readonly content: string;

  /**
   Exact fragment removed, empty when the input was left untouched.
   */
  readonly marker: string;
};

/**
 Removes truncated provider channel markers sitting in front of content.

 Strips only when every leading fragment is shaped like the end of a `<|word|>`
 token AND what follows opens an object, an array or a code fence, so a reply
 that begins with those characters and then says something else still fails to
 parse. The decision is transactional: a run that does not reach real content
 leaves the input untouched rather than partially repaired.

 @param text - model content, before or after fence removal

 @returns Content without the markers, plus what was removed for logging

 @example
 ```ts
 const { content, marker, } = stripChannelMarker({ text: 'ep|>{"count":2}', },);
 ```
 */
export function stripChannelMarker(
  {
    text,
  }: {
    readonly text: string;
  },
): ChannelMarkerStrip {
  /**
   Nothing removed, which every rejection in this function returns.
   */
  const untouched: ChannelMarkerStrip = {
    content: text,
    marker: '',
  };

  /**
   Input without leading whitespace so the first marker sits at column zero.
   */
  const trimmed = text.trimStart();

  /**
   Characters of leading marker consumed.

   Folded over a fixed number of slots rather than accumulated in a mutable
   cursor: once the run ends, every remaining slot returns the width
   unchanged, so the bound is expressed by the array length instead of by a
   break.
   */
  const consumed = Array.from({ length: MARKER_RUN_LIMIT, },)
    .reduce(
    function extend(width: number,): number {
        /**
         Width of the next marker, zero once the run has ended.
         */
      const next = markerWidth({ text: trimmed.slice(width,), },);
      return (next === 0) ? width : (width + next);
    },
    0,
    );

  if (consumed === 0)
    return untouched;

  /**
   What follows the run, which must itself open real content.
   */
  const rest = trimmed
    .slice(consumed,)
    .trimStart();

  if (!CONTENT_OPENINGS.some(function opensContent(opening,): boolean {
    return rest.startsWith(opening,);
  },))
    return untouched;

  return {
    content: rest,
    marker: trimmed.slice(
      0,
      consumed,
    ),
  };
}

//endregion Channel marker tails
