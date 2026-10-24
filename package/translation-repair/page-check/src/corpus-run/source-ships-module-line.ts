import { moduleLineStartsOf, } from '@monochromatic-dev/translation-repair-document-structure/ts';

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
// was the last. The reading here is made from the page as it will be written, after every other rewrite, so the lines it
// names are the lines the page holds.

/**
 Writes the first letter of each line the strict grammar reads as an ES module as a character reference, which shows
 the same and starts no module.

 @param text - text with every other rewrite made, as the page will hold it

 @returns Text with `&#105;` or `&#101;` in place of the first letter of each such line, equal to the text when it
 holds none

 @example
 ```ts
 const written = withModuleLinesWritten({ text: '她说\n\nimport data\n', },);
 ```
 */
export function withModuleLinesWritten({ text, }: { readonly text: string; },): string {
  /**
   Pieces of the result, joined at the end.
   */
  const pieces: string[] = [];
  /**
   Where the text not yet copied starts.
   */
  const copied = { upTo: 0, };
  for (const at of moduleLineStartsOf({ body: text, },)) {
    pieces.push(
      text.slice(
        copied.upTo,
        at,
      ),
      `&#${String(text.codePointAt(at,),)};`,
    );
    copied.upTo = at + 1;
  }
  pieces.push(text.slice(copied.upTo,),);
  return pieces.join('',);
}

//endregion Source ships module line
