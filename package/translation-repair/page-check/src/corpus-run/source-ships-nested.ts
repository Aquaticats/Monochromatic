import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import {
  parseMarkdownMathBody,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  originalStretches,
  type SourceSpans,
  type TextEdit,
  type TextRange,
} from './source-ships-ranges.ts';

//region Source ships nested
// A BLOCK NESTED IN ANOTHER, READ AS A TEXT OF ITS OWN: the body of a footnote definition and the blocks indented under
// one are parsed from the characters that make them up, and what the parse says is mapped back to the offsets of the
// whole text (`source-ships-spans.ts` says why).

/**
 What reads the spans of a text, handed to the nested reading so that the two modules do not import each other.
 */
export type SpansReader = (
  arguments_: {
    readonly source: string;
    readonly readable: string;
    readonly root: TreeNode;
  },
) => SourceSpans;

/**
 The characters of a text at the indices.

 @param text - text

 @param indices - indices of the characters, in order

 @returns Characters joined

 @example
 ```ts
 const body = charactersAt({ text: 'abc', indices: [0, 2,], },);
 ```
 */
function charactersAt(
  {
    text,
    indices,
  }: {
    readonly text: string;
    readonly indices: readonly number[];
  },
): string {
  return indices
    .map(function characterAt(at,): string {
      return text.charAt(at,);
    },)
    .join('',);
}

/**
 Reads a block nested in another as a text of its own: the body of a footnote definition, or a block indented under one.

 @param source - whole text

 @param readable - whole text with its comments blanked, the parse's own input

 @param indices - indices of the whole text that make up the block, in order

 @param readSpans - what reads a block's spans, handed in so this module and the reader do not import each other

 @returns Spans of the block at the offsets of the whole text

 @throws {@link Error} when a node of the block has no position, which the reading used never builds

 @example
 ```ts
 const spans = nestedSpans({ source, readable, indices: [6, 7, 8,], readSpans: spansOf, });
 ```
 */
export function nestedSpans(
  {
    source,
    readable,
    indices,
    readSpans,
  }: {
    readonly source: string;
    readonly readable: string;
    readonly indices: readonly number[];
    readonly readSpans: SpansReader;
  },
): SourceSpans {
  /**
   The block as a text of its own.
   */
  const body = charactersAt({
    text: source,
    indices,
  },);
  /**
   The same with its comments blanked.
   */
  const bodyReadable = charactersAt({
    text: readable,
    indices,
  },);
  /**
   What the parse of the block says of it.
   */
  const inner = readSpans({
    source: body,
    readable: bodyReadable,
    root: parseMarkdownMathBody({ body: bodyReadable, },),
  },);
  return {
    text: inner
      .text
      .flatMap(function original(range,): readonly TextRange[] {
        return originalStretches({
          range,
          indices,
        },);
      },),
    literal: inner
      .literal
      .flatMap(function original(range,): readonly TextRange[] {
        return originalStretches({
          range,
          indices,
        },);
      },),
    edits: inner
      .edits
      .map(function original(edit,): TextEdit {
        return {
          ...edit,
          start: nonNullishOrThrow(indices[edit.start],),
          end: nonNullishOrThrow(indices[edit.end - 1],) + 1,
        };
      },),
    insideLinks: inner
      .insideLinks
      .flatMap(function original(range,): readonly TextRange[] {
        return originalStretches({
          range,
          indices,
        },);
      },),
    streamStarts: inner
      .streamStarts
      .map(function original(start,): number {
        return nonNullishOrThrow(indices[start],);
      },),
  };
}

//endregion Source ships nested
