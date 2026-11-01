import { fencedLineFlags, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  maskHtmlComments,
  splitFrontMatter,
} from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Page visible text
// LEDGER E10 (2026-09-28): the page-name glossary read raw documents, so a
// link inside an editor's comment or the front matter paired as a name the
// page renders, a note line starting with "#" inside a comment counted as a
// heading, and on one pinned entry that extra heading broke the heading-count
// alignment and dropped three real heading pairs from every sheet. Measured
// over the 92 pinned entries: 5 read differently. The glossary reads only what
// the page shows: the body after the front matter, with HTML comments, JSX
// comments and fenced code blanked.

/**
 Opening of a JSX comment.
 */
const JSX_COMMENT_OPEN = '{/*';

/**
 Closing of a JSX comment.
 */
const JSX_COMMENT_CLOSE = '*/}';

/**
 Replaces every character of a text but its line breaks with a space.

 @param text - text to blank

 @returns Blanked text, as long as the input and with its lines intact

 @example
 ```ts
 blankKeepingLines({ text: 'a\nbc', },); // ' \n  '
 ```
 */
function blankKeepingLines(
  { text, }: { readonly text: string; },
): string {
  return text
    .split('\n',)
    .map(function blank(line,): string {
      return ' '.repeat(line.length,);
    },)
    .join('\n',);
}

/**
 Blanks every JSX comment, keeping line breaks; an unclosed one runs to the
 end, as MDX reads it.

 @param text - body text

 @returns Text with each comment blanked

 @example
 ```ts
 maskJsxComments({ text: 'a {/* b *\/} c', },); // 'a           c'
 ```
 */
function maskJsxComments(
  { text, }: { readonly text: string; },
): string {
  /**
   Kept and blanked pieces in order.
   */
  const parts: string[] = [];
  for (let cursor = 0; cursor < text.length;) {
    /**
     Next comment opening at or past the cursor.
     */
    const open = text.indexOf(
      JSX_COMMENT_OPEN,
      cursor,
    );
    if (open === (-1)) {
      parts.push(text.slice(cursor,),);
      break;
    }
    /**
     Where that comment closes, or minus one.
     */
    const close = text.indexOf(
      JSX_COMMENT_CLOSE,
      open + JSX_COMMENT_OPEN.length,
    );
    /**
     Offset past the comment.
     */
    const end = (close === (-1)) ? text.length : close + JSX_COMMENT_CLOSE.length;
    parts.push(
      text.slice(
        cursor,
        open,
      ),
      blankKeepingLines({
        text: text.slice(
          open,
          end,
        ),
      },),
    );
    cursor = end;
  }
  return parts.join('',);
}

/**
 What a page shows a reader: its body after the front matter, with HTML
 comments, JSX comments and fenced code blanked and every line kept.

 @param text - whole document

 @returns Visible body text

 @example
 ```ts
 visibleText({ text: '---\ntitle: x\n---\n<!-- note -->\n## Nap', },);
 ```
 */
export function visibleText(
  { text, }: { readonly text: string; },
): string {
  /**
   The body after any front matter.
   */
  const { body, } = splitFrontMatter({ text, },);
  /**
   That body with its HTML comments blanked.
   */
  const { masked, } = maskHtmlComments({ text: body, },);
  /**
   And with its JSX comments blanked.
   */
  const uncommented = maskJsxComments({ text: masked, },);
  /**
   The body's lines.
   */
  const lines = uncommented.split('\n',);
  /**
   Which lines are fenced code.
   */
  const fenced = fencedLineFlags({ lines, },);
  return lines
    .map(function shown(
      line,
      at,
    ): string {
      return (fenced[at] === true) ? '' : line;
    },)
    .join('\n',);
}

//endregion Page visible text
