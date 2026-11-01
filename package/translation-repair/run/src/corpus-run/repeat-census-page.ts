import {
  type CensusGroup,
  classify,
  REFRAIN_SETTINGS,
} from './repeat-census-lines.ts';
import { maximalRepeats, } from './repeat-census-maximal.ts';
import {
  censusClausesOf,
  refrainGroups,
} from './repeat-census-refrain.ts';

//region Repeat census page
// The census of one page, and the blanking of its HTML comments before reading.

/**
 Marker that opens an HTML comment.
 */
const COMMENT_OPEN = '<!--';

/**
 Marker that closes an HTML comment.
 */
const COMMENT_CLOSE = '-->';

/**
 Replaces each character of a text with a space, keeping its line breaks.

 @param text - text to blank

 @returns Spaces of the same length, with each line break kept

 @example
 ```ts
 const blank = blankText({ text: 'a\nb', },);
 ```
 */
function blankText({ text, }: { readonly text: string; },): string {
  return text
    .split('',)
    .map(function blankOf(char,): string {
      return (char === '\n') ? char : ' ';
    },)
    .join('',);
}

/**
 Finds the HTML comments of a text, in order, each from its opening marker to
 the end of its first closing marker. A comment that never closes is no span.

 @param text - raw page text

 @returns Span of each comment, by offsets

 @example
 ```ts
 const spans = commentSpansOf({ text: '猫<!-- 注 -->咪', },);
 ```
 */
function commentSpansOf({ text, }: { readonly text: string; },): readonly {
  readonly open: number;
  readonly end: number
}[] {
  /**
   Spans found so far.
   */
  const spans: {
    readonly open: number;
    readonly end: number
  }[] = [];
  for (let from = 0; from < text.length;) {
    /**
     Offset of the next opening marker, or minus one.
     */
    const open = text.indexOf(
      COMMENT_OPEN,
      from,
    );
    if (open === (-1))
      break;
    /**
     Offset of the first closing marker after the opening one, or minus one.
     */
    const close = text.indexOf(
      COMMENT_CLOSE,
      open + COMMENT_OPEN.length,
    );
    if (close === (-1))
      break;
    spans.push({
      open,
      end: close + COMMENT_CLOSE.length,
    },);
    from = close + COMMENT_CLOSE.length;
  }
  return spans;
}

/**
 Blanks each HTML comment to spaces, keeping its line breaks, so that line
 numbers hold. A comment that never closes is left as it stands, as the census
 did.

 @param text - raw page text

 @returns Page text with comments blanked

 @example
 ```ts
 const masked = blankComments({ text: '猫<!-- 注 -->咪', },);
 ```
 */
function blankComments({ text, }: { readonly text: string; },): string {
  /**
   Comments of the page, in order.
   */
  const spans = commentSpansOf({ text, },);

  /**
   Text before each comment, then that comment blanked.
   */
  const pieces = spans.map(function carriedAndBlanked(
    span,
    index,
  ): string {
    /**
     Span of the comment that precedes the comment at this index, if any.
     */
    const previous = spans[index - 1];
    /**
     Offset where the text before this comment begins.
     */
    const from = (previous === undefined) ? 0 : previous.end;
    return text.slice(
      from,
      span.open,
    ) + blankText({ text: text.slice(
      span.open,
      span.end,
    ), },);
  },);

  /**
   Span of the last comment, if any.
   */
  const last = spans.at(-1);
  return [
    ...pieces,
    text.slice((last === undefined) ? 0 : last.end,),
  ].join('',);
}


/**
 Census of one page: its maximal repeats, and its refrain frames under each setting.

 @param text - page text, HTML comments included

 @returns Groups found, and the refrain counts per setting

 @example
 ```ts
 const census = censusOfPage({ text, },);
 ```
 */
export function censusOfPage({ text, }: { readonly text: string; },): {
  readonly maximal: readonly CensusGroup[];
  readonly refrain: readonly CensusGroup[];
  readonly refrainCounts: Readonly<Record<'main' | 'loose' | 'tight', number>>;
} {
  /**
   Lines of the page with HTML comments blanked, keeping line numbers.
   */
  const lines = blankComments({ text, },)
    .split('\n',);

  /**
   Role of each line.
   */
  const kinds = classify({ lines, },);

  /**
   Clauses of the usable lines.
   */
  const clauses = censusClausesOf({
    lines,
    kinds,
  },);
  /**
   Refrain groups under the main setting.
   */
  const main = refrainGroups({
    clauses,
    setting: REFRAIN_SETTINGS.main,
  },);

  /**
   Refrain groups under the loose setting.
   */
  const loose = refrainGroups({
    clauses,
    setting: REFRAIN_SETTINGS.loose,
  },);

  /**
   Refrain groups under the tight setting.
   */
  const tight = refrainGroups({
    clauses,
    setting: REFRAIN_SETTINGS.tight,
  },);
  return {
    maximal: maximalRepeats({
      lines,
      kinds,
    },),
    refrain: main,
    refrainCounts: {
      main: main.length,
      loose: loose.length,
      tight: tight.length,
    },
  };
}


//endregion Repeat census page
