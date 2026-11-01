import {
  characterRunEnd,
  isOneOf,
} from './nesting-line-lexing.ts';

//region Nesting html block
// WHERE THE PARSER MAY BE READING RAW HTML, so that a line which looks like a fence
// there opens no fenced block. CommonMark reads a line opening with an angle
// bracket as an html block that runs to the next blank line, and five kinds
// run to an end marker instead, blank lines and fence-like lines included
// (`micromark-core-commonmark@2.0.4` `lib/html-flow.js`). The scan skips what
// a fence holds, so it must not take a fence line in raw html for one.
//
// THE READING ERRS TOWARD STAYING IN HTML: it never leaves a block before
// the marker the grammar ends it by, and it enters one for any line opening
// with an angle bracket, whether or not the grammar would. A text it keeps
// in html longer than the parser does is counted rather than skipped.

/**
 What raw html the parser may be reading: none, a block that ends at a blank
 line, or one of the kinds that end at a marker (`pre`: the closing tag of
 `pre`, `script`, `style` or `textarea`; `comment`: `-->`; `instruction`:
 `?>`; `declaration`: `>`; `cdata`: `]]>`).
 */
export type HtmlBlock = 'none' | 'blank' | 'pre' | 'comment' | 'instruction' | 'declaration' | 'cdata';

/**
 Tag names whose opening tag begins a block that ends at its closing tag.
 */
const RAW_TEXT_NAMES: ReadonlySet<string> = new Set([
  'pre',
  'script',
  'style',
  'textarea',
],);

/**
 What opens a comment, which its end marker is looked for after.
 */
const COMMENT_OPENER = '<!--';

/**
 What opens a character data section, which its end marker is looked for after.
 */
const CDATA_OPENER = '<![cdata[';

/**
 ASCII letters, which a declaration's name begins with.
 */
const LETTERS = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ';

/**
 Which kind of html block a line opens, when it opens one.

 @param rest - the line after its container markers and blanks

 @returns The kind of block the line opens, `none` for a line that does not
 begin with an angle bracket, `blank` for any other line that does

 @example
 ```ts
 htmlBlockOpenedBy({ rest: '<!-- note', },);
 // => 'comment'
 ```
 */
function htmlBlockOpenedBy({ rest, }: { readonly rest: string; },): HtmlBlock {
  if (!rest.startsWith('<',))
    return 'none';
  /**
   The line as the grammar compares tag names, without case.
   */
  const lowered = rest.toLowerCase();
  if (lowered.startsWith(COMMENT_OPENER,))
    return 'comment';
  if (lowered.startsWith('<?',))
    return 'instruction';
  if (lowered.startsWith(CDATA_OPENER,))
    return 'cdata';
  if ((rest[1] === '!') && isOneOf({
    character: rest[2] ?? '',
    set: LETTERS,
  },))
    return 'declaration';
  /**
   Index after the tag name an opening tag begins with.
   */
  const nameEnd = characterRunEnd({
    line: lowered,
    from: 1,
    set: LETTERS,
  },);
  /**
   Whether the name runs to the end of the line or is followed by the end of
   the tag or a blank.
   */
  const nameEnds = (nameEnd === lowered.length)
    || isOneOf({
      character: lowered.charAt(nameEnd,),
      set: '> \t',
    },);
  if (RAW_TEXT_NAMES.has(lowered.slice(
    1,
    nameEnd,
  ),) && nameEnds)
    return 'pre';
  return 'blank';
}

/**
 How many characters of a line the opening of a kind of block takes, which
 its end marker is looked for after.

 @param kind - the kind of block the line opens

 @returns Characters the opener takes

 @example
 ```ts
 openerWidth({ kind: 'comment', },);
 // => 4
 ```
 */
function openerWidth({ kind, }: { readonly kind: HtmlBlock; },): number {
  if (kind === 'comment')
    return COMMENT_OPENER.length;
  if (kind === 'cdata')
    return CDATA_OPENER.length;
  if ((kind === 'instruction') || (kind === 'declaration'))
    return 2;
  return 1;
}

/**
 Whether a line carries the marker that ends a kind of block.

 @param kind - the kind of block that is open

 @param line - the line, without case folding

 @param from - index to look from, after the opener when the line opened the block

 @returns True when the end marker of that kind stands in the line from there

 @example
 ```ts
 endsBlock({ kind: 'comment', line: 'note -->', from: 0, },);
 // => true
 ```
 */
function endsBlock(
  {
    kind,
    line,
    from,
  }: {
    readonly kind: HtmlBlock;
    readonly line: string;
    readonly from: number;
  },
): boolean {
  if (kind === 'comment')
    return line.includes(
      '-->',
      from,
    );
  if (kind === 'instruction')
    return line.includes(
      '?>',
      from,
    );
  if (kind === 'cdata')
    return line.includes(
      ']]>',
      from,
    );
  if (kind === 'declaration')
    return line.includes(
      '>',
      from,
    );
  /**
   The line as the grammar compares tag names, without case.
   */
  const lowered = line.toLowerCase();
  return [...RAW_TEXT_NAMES,].some(function closes(name,): boolean {
    return lowered.includes(
      `</${name}>`,
      from,
    );
  },);
}

/**
 Where one non-blank line leaves the reading of raw html.

 @param before - what the lines before it left open

 @param rest - the line after its container markers and blanks

 @returns What stays open after the line: a block that ends at a blank line
 stays until one comes, a block that ends at a marker stays until the line
 that carries it, and a line outside any block opens one when it begins with
 an angle bracket

 @example
 ```ts
 htmlBlockAfter({ before: 'none', rest: '<div>', },);
 // => 'blank'
 ```
 */
export function htmlBlockAfter(
  {
    before,
    rest,
  }: {
    readonly before: HtmlBlock;
    readonly rest: string;
  },
): HtmlBlock {
  if (before === 'blank')
    return 'blank';
  if (before !== 'none')
    return endsBlock({
      kind: before,
      line: rest,
      from: 0,
    },)
      ? 'none'
      : before;
  /**
   What this line opens, when it opens a block.
   */
  const opened = htmlBlockOpenedBy({ rest, },);
  if ((opened === 'none') || (opened === 'blank'))
    return opened;
  return endsBlock({
    kind: opened,
    line: rest,
    from: openerWidth({ kind: opened, },),
  },)
    ? 'none'
    : opened;
}

/**
 Where a blank line leaves the reading of raw html: a block that ends at a
 blank line ends, one that ends at a marker does not.

 @param before - what the lines before it left open

 @returns What stays open after the blank line

 @example
 ```ts
 htmlBlockAfterBlank({ before: 'blank', },);
 // => 'none'
 ```
 */
export function htmlBlockAfterBlank({ before, }: { readonly before: HtmlBlock; },): HtmlBlock {
  return (before === 'blank') ? 'none' : before;
}

//endregion Nesting html block
