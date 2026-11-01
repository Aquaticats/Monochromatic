import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { isLatinCapital, } from '@monochromatic-dev/translation-repair-markup/ts';
import { emphasisSpans, } from './emphasis-spans.ts';

//region Archive italic spans
// The words the archive sets in italics, read for class one hundred
// seventy-three's title restore, off the parse (ledger B72). This split each
// line at its stars: words inside an HTML comment or an MDX expression read as
// italics, a span across two lines read as its first line's words, and a
// linked title or one set with underscores was never read. Bold is a strong
// node, not emphasis, so its words are not read as italics either way.

/**
 Whether a span reads as a title: no space at either edge and a capital Latin
 letter first, accented or not, as a work's name opens (`Émile`); the test
 took A to Z only until ledger B18.

 @param span - words a span sets in italics

 @returns Whether the span can name a work

 @example
 ```ts
 titleLike({ span: 'Long Nap', },); // true
 ```
 */
function titleLike({ span, }: { readonly span: string; },): boolean {
  /**
   First character of the span.
   */
  const first = span.charAt(0,);
  return (span.length > 0)
    && (span.trim() === span)
    && isLatinCapital({ character: first, },);
}

/**
 Every span the archive's body sets in italics that can name a work.

 @param slices - prepared pairs, whose target text is the archive's

 @returns Italic spans, once each, every run of spaces and line feeds read as
 one space

 @example
 ```ts
 archiveItalicSpans({ slices, },); // Set { 'Long Nap' }
 ```
 */
export function archiveItalicSpans(
  { slices, }: { readonly slices: readonly ChunkPair[]; },
): ReadonlySet<string> {
  return new Set(slices
    .filter(function isBody(slice,): boolean {
      return slice.syntax !== 'front-matter';
    },)
    .flatMap(function readSlice(slice,): readonly string[] {
      return emphasisSpans({
        text: slice.target
          .text,
      },)
        .map(function wordsOf(span,): string {
          return span.words;
        },);
    },)
    .filter(function isTitle(span,): boolean {
      return titleLike({ span, },);
    },),);
}

//endregion Archive italic spans
