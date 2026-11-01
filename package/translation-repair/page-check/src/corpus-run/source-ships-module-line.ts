import {
  moduleLineStartsOf,
  parseMarkdownMathBody,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { TextRange, } from './source-ships-ranges.ts';

//region Source ships module line
// THE LINES MDX READS AS AN ES MODULE: a flow block at the first column of the document's own level that starts with
// `import ` or `export ` is read by acorn as JavaScript, and prose that starts so is refused. A backslash cannot
// make it plain; the first letter written as a character reference does.
//
// WHICH LINES START A MODULE IS READ FROM THE STRICT GRAMMAR'S TOKENIZER, NOT FROM A LIST OF SHAPES
// (`document-structure/src/module-line-starts.ts` says how). Whether a line opens a block depends on the state the
// tokenizer is in when it reaches the line: the containers the line continues, a setext underline before it, a line of
// invisible characters the package's mask blanks, a footnote definition that stays open, a list marker alone. A list of
// the shapes that change that state (an earlier version of this module held one) is right for the shapes it lists and
// silent about the rest, and the rest kept turning up: the line after a setext underline that follows a restarted block
// was the last. The reading here is made from the page as it will be written, after every other rewrite, so the lines
// it names are the lines the page holds.
//
// A LINE INSIDE A CODE SPAN OR INLINE MATH IS JOINED TO THE LINE BEFORE IT, NOT WRITTEN AS A REFERENCE. The plain
// reading keeps a span across a line of invisible characters, which the mask of `parseDocument` blanks before the
// strict parse and so ends the paragraph at; a character reference written inside the span shows as its characters. A
// line ending inside a span reads as a space, so replacing the line ending before the module word with a space shows
// the same, and the line before it is no longer blank to the strict grammar, which then reads the span as the plain
// reading does. Joining and writing can change what the next reading finds, so the reading is repeated until it finds
// no line; each pass removes the lines it found and makes none, so it ends.

/**
 Finds the code spans and the inline math of a text, as the plain reading with the site's math reads them.

 @param text - text to read

 @returns Start and end of each span, in the order the walk reaches them

 @example
 ```ts
 const spans = spanRangesOf({ text: '`a` b', },);
 ```
 */
function spanRangesOf({ text, }: { readonly text: string; },): readonly TextRange[] {
  /**
   Spans found.
   */
  const spans: TextRange[] = [];
  /**
   Nodes still to walk, the next one on top.
   */
  const pending: TreeNode[] = [parseMarkdownMathBody({ body: text, },),];
  for (let node = pending.pop(); node !== undefined; node = pending.pop()) {
    if (((node.type === 'inlineCode') || (node.type === 'inlineMath')) && (node.position !== undefined)) {
      /**
       Where the span starts and ends.
       */
      const {
        start,
        end,
      } = node.position;
      spans.push({
        start: start.offset ?? 0,
        end: end.offset ?? 0,
      },);
    }
    if ('children' in node) {
      /**
       The node's children, the last first so they come out in order.
       */
      const { children, } = node;
      pending.push(...children.toReversed(),);
    }
  }
  return spans;
}

/**
 Rewrites each module line a text holds once: a line inside a span is joined to the line before it, any other has its
 first letter written as a character reference.

 @param text - text to rewrite

 @param starts - index of the first letter of each module line, ascending

 @returns Rewritten text

 @example
 ```ts
 const once = rewrittenOnce({ text: 'x\n\nimport a\n', starts: [3,], },);
 ```
 */
function rewrittenOnce(
  {
    text,
    starts,
  }: {
    readonly text: string;
    readonly starts: readonly number[];
  },
): string {
  /**
   Where the code spans and the inline math stand.
   */
  const spans = spanRangesOf({ text, },);
  /**
   Pieces of the result, joined at the end.
   */
  const pieces: string[] = [];
  /**
   Where the text not yet copied starts.
   */
  const copied = { upTo: 0, };
  for (const at of starts) {
    if (spans.some(function holds(span,): boolean {
      return (span.start < at) && (at < span.end);
    },)) {
      /**
       Where the line ending before the line starts, a carriage return and a line feed together.
       */
      const breakStart = text.startsWith(
        '\r\n',
        at - 2,
      )
        ? at - 2
        : at - 1;
      pieces.push(
        text.slice(
          copied.upTo,
          breakStart,
        ),
        ' ',
      );
      copied.upTo = at;
    }
    else {
      pieces.push(
        text.slice(
          copied.upTo,
          at,
        ),
        `&#${String(text.codePointAt(at,),)};`,
      );
      copied.upTo = at + 1;
    }
  }
  pieces.push(text.slice(copied.upTo,),);
  return pieces.join('',);
}

/**
 Writes each line the strict grammar reads as an ES module so that it starts no module and shows the same.

 @param text - text with every other rewrite made, as the page will hold it

 @returns Text with `&#105;` or `&#101;` in place of the first letter of each such line, or the line joined to the
 one before it where it stands in a code span or inline math; equal to the text when it holds none

 @example
 ```ts
 const written = withModuleLinesWritten({ text: '她说\n\nimport data\n', },);
 ```
 */
export function withModuleLinesWritten({ text, }: { readonly text: string; },): string {
  /**
   The text so far.
   */
  const state = { current: text, };
  for (
    let starts = moduleLineStartsOf({ body: text, },);
    starts.length > 0;
    starts = moduleLineStartsOf({ body: state.current, },)
  ) {
    state.current = rewrittenOnce({
      text: state.current,
      starts,
    },);
  }
  return state.current;
}

//endregion Source ships module line
