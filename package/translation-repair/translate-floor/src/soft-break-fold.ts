import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type DocumentNode,
  requireMarkdownRefusal,
} from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Soft break fold
// SHOWS A PROSE PARAGRAPH THE WAY IT RENDERS, for a judge whose decision must
// not turn on where the text breaks its lines.
//
// Class one hundred fifty-three (XingZ6014, 2026-09-26): on a prose slice the
// consolidation polish is wrapped at its semantic boundaries before its gate,
// while the base may stand as the archive's own one-line paragraph, so the two
// candidates differed in layout the page never shows. Slices 36, 61 and 64
// carried a one-line base beside a wrapped polish, and three gate ballots
// weighed the polish's "added line breaks" although the sheet told them not
// to. Folding each paragraph onto one line removes the difference instead of
// asking the judge to ignore it.
//
// ONLY A TOP-LEVEL PARAGRAPH IS FOLDED. A blockquote, list, heading, table or
// tag keeps its lines, since its line starts carry syntax, and a hard break
// (two trailing spaces or a backslash) keeps its newline, since it renders.

/**
 Whether a paragraph line ends in a Markdown hard break.

 @param line - one paragraph line without its newline

 @returns Whether newline after line renders as line break

 @example
 ```ts
 const hard = endsInHardBreak({ line: 'The cat said:  ', },);
 // => true
 ```
 */
function endsInHardBreak({ line, }: { readonly line: string; },): boolean {
  return line.endsWith('  ',) || line.endsWith('\\',);
}

/**
 Folds one paragraph's soft breaks into spaces.

 @param text - exact paragraph source

 @returns Paragraph with soft breaks read as spaces and hard breaks kept

 @example
 ```ts
 const folded = foldParagraph({ text: 'The cat slept,\nand woke.', },);
 // => 'The cat slept, and woke.'
 ```
 */
function foldParagraph({ text, }: { readonly text: string; },): string {
  /**
   Paragraph lines in source order.
   */
  const lines = text.split('\n',);
  return lines
    .map(function joined(
      line,
      index,
    ): string {
      /**
       Whether line opens after rendered break.
       */
      const afterHard = (index > 0) && endsInHardBreak({ line: nonNullishOrThrow(lines[index - 1],), },);
      /**
       Whether line closes with rendered break.
       */
      const beforeHard = endsInHardBreak({ line, },);
      /**
       Whether this is paragraph's last line.
       */
      const last = index === (lines.length - 1);
      /**
       Line with indentation dropped where a soft break joins it.
       */
      const opened = ((index === 0) || afterHard) ? line : line.trimStart();
      /**
       Line with trailing space dropped where a soft break follows it.
       */
      const body = (last || beforeHard) ? opened : opened.trimEnd();
      if (index === 0)
        return body;
      return `${afterHard ? '\n' : ' '}${body}`;
    },)
    .join('',);
}

/**
 Folds every top-level paragraph's soft breaks into spaces, leaving every
 other block and every hard break exactly as written.

 A PASSAGE NESTED TOO DEEPLY TO READ is returned as written: its paragraphs
 cannot be located, and the text is one a floor refuses anyway.

 @internal

 @param text - Markdown passage

 @returns Same passage as its paragraphs render, one line each

 @example
 ```ts
 const folded = foldSoftBreaks({ text: 'The cat slept,\nand woke.\n\n> A poem,\n> a line.', },);
 // => 'The cat slept, and woke.\n\n> A poem,\n> a line.'
 ```
 */
export function foldSoftBreaks({ text, }: { readonly text: string; },): string {
  /**
   Body paragraphs in source order, none for a text nested too deeply to read.
   */
  const paragraphs = (function readParagraphs(): readonly DocumentNode[] {
    try {
      return parseDocument({ text, },)
        .nodes
        .filter(function isBodyParagraph(node,): boolean {
          return (node.zone === 'body') && (node.kind === 'paragraph');
        },);
    }
    catch (error) {
      // Only the plain grammar's refusal, a nesting too deep to read, shows
      // the text as written: no paragraph can be located in it. Anything
      // else keeps propagating.
      requireMarkdownRefusal({ error, },);
      return [];
    }
  })();
  /**
   Where each stretch between paragraphs begins.
   */
  const gapStarts = [
    0,
    ...paragraphs.map(function endOf(node,): number {
      return node.endOffset;
    },),
  ];
  /**
   Unchanged gaps interleaved with folded paragraphs.
   */
  const pieces = paragraphs.flatMap(function withGap(
    node,
    index,
  ): readonly string[] {
    return [
      text.slice(
        gapStarts[index],
        node.startOffset,
      ),
      foldParagraph({ text: node.text, },),
    ];
  },);
  return [
    ...pieces,
    text.slice(gapStarts.at(-1,),),
  ]
    .join('',);
}

//endregion Soft break fold
