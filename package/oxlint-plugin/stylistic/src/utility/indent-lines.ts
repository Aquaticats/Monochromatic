import {
  leadingWhitespace,
} from './indent.ts';
import {
  at,
} from './range.ts';

//region Line table
// Per-line leading-whitespace facts every checked line compares against.

/**
 Leading-whitespace facts for every source line, aligned by line index.
 */
export type IndentLineTable = {
  readonly lineStarts: readonly number[];
  readonly widths: readonly number[];
  readonly contentStarts: readonly number[];
  readonly blank: readonly boolean[];
};

/**
 Builds the per-line leading-whitespace table for `sourceText`.
 
 Widths count leading ASCII whitespace characters, matching how the other
 layout fixers measure emitted indentation.
 
 @param sourceText - full file text
 
 @returns aligned line-start, width, content-start, and blank flags
 
 @example
 ```ts
 const { widths, } = lineTableOf('  foo\n');
 ```
 */
export function lineTableOf(sourceText: string,): IndentLineTable {
  /**
   Offset where each line begins; `0` plus one past every newline.
   */
  const lineStarts: number[] = [0,];
  for (
    let charIndex = 0;
    charIndex < sourceText.length;
    charIndex += 1
  ) {
    if (sourceText.charAt(charIndex,) === '\n')
      lineStarts.push(charIndex + 1,);
  }
  /**
   Line texts between newline characters, aligned with `lineStarts`.
   */
  const pieces = sourceText.split('\n',);
  const widths: number[] = [];
  const contentStarts: number[] = [];
  const blank: boolean[] = [];
  for (
    let lineIndex = 0;
    lineIndex < pieces.length;
    lineIndex += 1
  ) {
    const piece = at({
      arr: pieces,
      index: lineIndex,
    },);
    const width = leadingWhitespace(piece,)
      .length;
    widths.push(width,);
    contentStarts.push(at({
      arr: lineStarts,
      index: lineIndex,
    },) + width,);
    blank.push(width === piece.length,);
  }
  return {
    lineStarts,
    widths,
    contentStarts,
    blank,
  };
}
//endregion Line table

//region Template-line flags
// One sweep marks every line whose content starts inside a template literal.

/**
 Parameters for {@link inTemplateFlagsOf}.
 */
export type InTemplateFlagsOfParams = {
  readonly templateSpans: readonly (readonly [
    number,
    number
  ])[];
  readonly contentStarts: readonly number[];
};

/**
 Marks every line whose content starts strictly inside a template span.
 
 Template spans nest, so an interval stack tracks the innermost open span per
 line in one forward sweep; proper nesting leaves the containing span on top,
 making membership O(lines + spans).
 
 @param params - template spans and per-line content offsets
 
 @returns `true` per line whose content is template interior
 
 @example
 ```ts
 inTemplateFlagsOf({ templateSpans: [[0, 10]], contentStarts: [1] }); // [true]
 ```
 */
export function inTemplateFlagsOf({
  templateSpans,
  contentStarts,
}: InTemplateFlagsOfParams,): readonly boolean[] {
  const flags: boolean[] = [];
  /**
   Span indexes currently covering the sweep cursor, innermost last.
   */
  const openSpanIndexes: number[] = [];
  let spanCursor = 0;
  for (const contentStart of contentStarts) {
    while (
      (spanCursor < templateSpans.length)
        && (at({
          arr: templateSpans,
          index: spanCursor,
        },)[0] < contentStart)
    ) {
      openSpanIndexes.push(spanCursor,);
      spanCursor += 1;
    }
    while (openSpanIndexes.length > 0) {
      /**
       Innermost open span; once its end passes the cursor the sweep moves on.
       */
      const topSpan = at({
        arr: templateSpans,
        index: at({
          arr: openSpanIndexes,
          index: openSpanIndexes.length - 1,
        },),
      },);
      if (topSpan[1] > contentStart)
        break;
      openSpanIndexes.pop();
    }
    flags.push(openSpanIndexes.length > 0,);
  }
  return flags;
}
//endregion Template-line flags
