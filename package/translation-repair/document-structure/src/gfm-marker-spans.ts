//region Positioned GFM marker lexemes
// Grammar follows micromark-extension-gfm-footnote 2.1.0 callData/labelInside and escape states.
// Lexical hits still need AST context before they can authorize edits.

/**
 Maximum raw label length accepted by the installed footnote tokenizer.
 */
const MAX_GFM_LABEL_LENGTH = 999;

/**
 This position does not contain a GFM marker, distinct from an invalid caller state.

 @example
 ```ts
 if (typeof gfmMarkerAt({ text, offset }) === 'symbol') inspectOrdinaryText();
 ```
 */
export const NO_GFM_MARKER: unique symbol = Symbol('no GFM marker at this position',);

/**
 Exact marker lexeme, with offsets relative to the supplied text.

 @example
 ```ts
 const marker: GfmMarkerSpan = { rawLabel: 'Note', startOffset: 0, endOffset: 7 };
 ```
 */
export type GfmMarkerSpan = {
  /**
   Encoded Markdown spelling, not a decoded mdast label.
   */
  readonly rawLabel: string;
  /**
   Opening bracket offset.
   */
  readonly startOffset: number;
  /**
   Offset immediately after the unescaped closing bracket.
   */
  readonly endOffset: number;
};

/**
 Checks backslash parity immediately before a possible marker.
 Runs inspected for distinct openings cannot overlap.

 @param text - original syntax-bearing text

 @param offset - opening bracket position

 @returns Whether Markdown escapes this opening

 @example
 ```ts
 const escaped = escapedMarkerOpening({ text, offset });
 ```
 */
function escapedMarkerOpening({
  text,
  offset,
}: {
  readonly text: string;
  readonly offset: number
},): boolean {
  for (let cursor = offset - 1; cursor >= 0; cursor -= 1) {
    if (text[cursor] !== '\\')
      return ((offset - 1
        - cursor) % 2) === 1;
  }
  return (offset % 2) === 1;
}

/**
 Reads one bounded marker, respecting bracket escapes and the tokenizer's whitespace rule.
 A definition and a reference share this lexeme; AST context distinguishes their roles.

 @param text - syntax-bearing text, with non-content regions already masked when appropriate

 @param offset - possible opening bracket

 @returns Exact lexeme when present, otherwise no marker

 @example
 ```ts
 const marker = gfmMarkerAt({ text: '[^a\\]b]', offset: 0 });
 ```
 */
export function gfmMarkerAt({
  text,
  offset,
}: {
  readonly text: string;
  readonly offset: number
},): GfmMarkerSpan | typeof NO_GFM_MARKER {
  if ((!text.startsWith(
    '[^',
    offset,
  )) || escapedMarkerOpening({
    text,
    offset,
  },))
    return NO_GFM_MARKER;
  /**
   First identifier character after the fixed opener.
   */
  const start = offset + 2;
  /**
   Inclusive closing-bracket bound after the longest valid identifier.
   */
  const limit = Math.min(
    text.length - 1,
    start + MAX_GFM_LABEL_LENGTH,
  );
  for (let cursor = start; cursor <= limit; cursor += 1) {
    /**
     Current raw code unit; bounds prevent an absent value.
     */
    const character = text[cursor];
    if ((character === '[') || (character === ' ')
      || (character === '\t')
      || (character === '\n')
      || (character === '\r'))
      return NO_GFM_MARKER;
    if (character === ']')
      return cursor === start ? NO_GFM_MARKER : {
        rawLabel: text.slice(
          start,
          cursor,
        ),
        startOffset: offset,
        endOffset: cursor + 1,
      };
    if ((character === '\\') && ((text[cursor + 1] === '[') || (text[cursor + 1] === ']')
      || (text[cursor + 1] === '\\')))
      cursor += 1;
  }
  return NO_GFM_MARKER;
}

/**
 Scans positioned lexemes without treating escaped openings as unresolved references.
 Each attempted label has a fixed tokenizer bound, so malformed overlapping openings remain linear.

 @param text - one syntax-bearing text-node slice

 @returns Lexemes in source order

 @example
 ```ts
 const markers = gfmMarkerSpans({ text: 'Missing[^9].' });
 ```
 */
export function gfmMarkerSpans({ text, }: { readonly text: string; },): readonly GfmMarkerSpan[] {
  /**
   Owned result list.
   */
  const markers: GfmMarkerSpan[] = [];
  for (let cursor = text.indexOf('[^',); cursor !== (-1); cursor = text.indexOf(
    '[^',
    cursor + 1,
  )) {
    /**
     Bounded read at this possible opening.
     */
    const marker = gfmMarkerAt({
      text,
      offset: cursor,
    },);
    if ((typeof marker) !== 'symbol') {
      markers.push(marker,);
      cursor = marker.endOffset - 1;
    }
  }
  return markers;
}

//endregion Positioned GFM marker lexemes
