import type {
  Root,
  RootContent,
} from 'mdast';

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  leadingMarkWidth,
  maskHtmlComments,
  maskLoneContainerTags,
  requireMarkdownRefusal,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { opensMdxTag, } from '@monochromatic-dev/translation-repair-markup/ts';
import { parseBodyTolerant, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Prose ranges
// CLASS ONE HUNDRED THIRTY-FOUR (hulicaijia19, 2026-09-25): a pass that
// rewrites the page's own English (dates, spellings) must leave everything
// that is not prose as it stands: a tag and its attributes (`text-align:
// center` is CSS, not a word), a JSX expression, a link destination, a bare
// URL, inline and fenced code, an HTML comment and the front matter. Each is
// found by one index scan and returned as a half-open range the rewrite may
// not touch. A tag opens where the MDX compiler reads one
// (`mdx-tag-start.ts`), a name in any script included.
//
// AN INLINE CODE SPAN'S END IS READ OFF THE REAL PARSE (ledger B68), not a
// scan for the next run of as many backticks before the next `'\n\n'`: a
// heading or any other block-starting construct interrupting a paragraph
// with no blank line extended the old scan's limit past the true end of the
// paragraph the opening backtick stood in, letting it match a closing
// backtick run belonging to a different, later span in a different block.
// The body is parsed ONCE per `protectedRanges` call, not once per
// backtick, since a parse per backtick would be a parse per character in the
// worst case.

/**
 One half-open range of a text the rewrite leaves alone.
 */
export type ProtectedRange = {
  readonly start: number;
  readonly end: number;
};

/**
 Opening of an HTML comment.
 */
const COMMENT_OPEN = '<!--';

/**
 Closing of an HTML comment.
 */
const COMMENT_CLOSE = '-->';

/**
 Fence a front matter block opens and closes with.
 */
const FRONT_MATTER_FENCE = '---\n';

/**
 Fence a code block opens and closes with.
 */
const CODE_FENCE = '```';

/**
 Characters that end a bare URL.
 */
const URL_END: ReadonlySet<string> = new Set([
  ' ',
  '\n',
  '\t',
  ')',
  '>',
  '"',
  '<',
],);

/**
 Schemes that open a bare URL.
 */
const URL_SCHEMES: readonly string[] = [
  'https://',
  'http://',
];

/**
 Where a closing marker ends, or the text's end when it never closes.

 @param text - text under scan

 @param marker - closing marker

 @param from - where to look from

 @returns Offset just past the marker, or the text's length

 @example
 ```ts
 pastMarker({ text: 'a --> b', marker: '-->', from: 0, },); // 5
 ```
 */
function pastMarker(
  {
    text,
    marker,
    from,
  }: {
    readonly text: string;
    readonly marker: string;
    readonly from: number;
  },
): number {
  /**
   Where the marker starts, or minus one.
   */
  const at = text.indexOf(
    marker,
    from,
  );
  return (at === (-1)) ? text.length : at + marker.length;
}

/**
 Where a text's front matter ends, zero when it opens with none. A byte order
 mark opening the text is no content: the front matter opens after it, and the
 range the end bounds holds the mark.

 @param text - text under scan

 @returns Offset just past the closing fence, or the text's length when the
 fence never closes

 @example
 ```ts
 frontMatterEnd({ text: '---\nname: Mittens\n---\nNaps.', },); // 22
 ```
 */
function frontMatterEnd({ text, }: { readonly text: string; },): number {
  /**
   Where the opening fence stands, after a leading byte order mark.
   */
  const start = leadingMarkWidth({ text, },);
  if (!text.startsWith(
    FRONT_MATTER_FENCE,
    start,
  ))
    return 0;
  /**
   Offset of the opening fence's line break, where the closing fence's search
   begins.
   */
  const fenceBreak = (start + FRONT_MATTER_FENCE.length) - 1;
  return pastMarker({
    text,
    marker: `\n${FRONT_MATTER_FENCE}`,
    from: fenceBreak,
  },);
}

/**
 The body past a text's front matter as the page grammar reads it, with
 where that body starts.

 COMMENTS AND LONE CONTAINER TAGS ARE MASKED FIRST to same-length whitespace,
 the preprocessing `parseSliceBody` gives the strict grammar, so no offset
 moves. An unmasked `<!--` refuses the strict grammar outright, and 17 of the
 pinned commit's 92 Chinese pages and 22 of its 92 English pages carry one;
 the body would then be read as plain markdown, where a tag opening an HTML
 block swallows everything to the next blank line and no inline code inside
 it is read at all.

 TOLERANT, as `parse-document.ts` reads a page: a body the strict grammar
 still refuses is read as plain markdown rather than thrown on, since the
 page passes and the translate floor call this on every page and slice they
 read, and a run ships every page. The downgrade's finding is not kept: the
 callers ask where blocks and code spans are, which the plain reading still
 answers.

 A BODY PLAIN MARKDOWN REFUSES TOO, nested past the parse bound or deep
 enough to exhaust its parser's stack, reads as having no blocks at all (ledger B100): no code span
 or sentence start is read off it, so its backticks are prose to the callers,
 and the Han residue floor, which reads every original, page and candidate
 through this, still answers rather than throwing. `parseBodyTolerant` reads the stack exhaustion of each of its two parses as
 that parse's refusal at the parse call; this function wraps nothing more, so an exhaustion raised by other code stays
 uncaught.

 @internal

 @param text - text under scan, front matter included when present

 @returns Parsed body and its offset in the text

 @example
 ```ts
 const { root, bodyOffset, } = proseBodyTree({ text, },);
 ```
 */
export function proseBodyTree({ text, }: { readonly text: string; },): {
  readonly root: Root;
  readonly bodyOffset: number;
} {
  /**
   Where the body starts.
   */
  const bodyOffset = frontMatterEnd({ text, },);
  /**
   Body with comments masked.
   */
  const { masked: withoutComments, } = maskHtmlComments({ text: text.slice(bodyOffset,), },);
  /**
   That body with lone container tags masked too, so a container half left
   open by chunking does not also defeat the strict grammar.
   */
  const { masked, } = maskLoneContainerTags({ text: withoutComments, },);
  try {
    /**
     That body as the strict grammar reads it, or as plain markdown where the
     strict grammar refuses it.
     */
    const { root, } = parseBodyTolerant({
      body: masked,
      bodyOffset,
    },);
    return {
      root,
      bodyOffset,
    };
  }
  catch (error) {
    // Only the plain grammar's own refusal reads as no structure; anything
    // else is an unexpected state that must keep propagating.
    requireMarkdownRefusal({ error, },);
    return {
      root: {
        type: 'root',
        children: [],
      },
      bodyOffset,
    };
  }
}

/**
 Every inline code span the real parse reads in one body, keyed by each
 span's own opening offset.

 READ OFF THE PARSE (ledger B68), replacing a scan that found a span's end
 at the next run of exactly as many backticks before the next `'\n\n'`: a
 heading, a blockquote, or any other block-starting construct interrupting a
 paragraph with no blank line extended that scan's limit past the true end
 of the paragraph the opening backtick stood in, letting it match a closing
 backtick run that belonged to a different, later span in a different
 block. `inlineCode` is an mdast LEAF node, so this walk never recurses into
 one; `'children' in node` is false for it.

 @param root - body parsed by the tolerant grammar

 @param bodyOffset - absolute offset of the parsed body within the full text

 @returns Each span's closing offset (exclusive), by its opening offset

 @example
 ```ts
 const spans = inlineCodeSpans({ root, bodyOffset: 0, },);
 ```
 */
function inlineCodeSpans(
  {
    root,
    bodyOffset,
  }: {
    readonly root: Root;
    readonly bodyOffset: number;
  },
): ReadonlyMap<number, number> {
  /**
   Spans found so far, by opening offset.
   */
  const spans = new Map<number, number>();
  /**
   Nodes still to visit, held as a stack so the walk stays iterative over a
   tree of unknown depth; order does not matter to a map keyed by offset.
   */
  const pending: RootContent[] = [...root.children,];
  // Next node, until the stack is empty.
  for (let node = pending.pop(); node !== undefined; node = pending.pop()) {
    if (node.type === 'inlineCode') {
      spans.set(
        bodyOffset + nonNullishOrThrow(node.position
          ?.start
          .offset,),
        bodyOffset + nonNullishOrThrow(node.position
          ?.end
          .offset,),
      );
    }
    if ('children' in node)
      pending.push(...node.children,);
  }
  return spans;
}

/**
 Whether a quote mark opened just before one offset closes on its line.

 @param text - text under scan

 @param at - offset just past the opening mark

 @param quote - opening mark

 @returns Whether the same mark stands again before the next line break

 @example
 ```ts
 closesOnLine({ text: "{cat's} nap", at: 5, quote: "'", },); // false
 ```
 */
function closesOnLine(
  {
    text,
    at,
    quote,
  }: {
    readonly text: string;
    readonly at: number;
    readonly quote: string;
  },
): boolean {
  /**
   Where the partner stands, or minus one.
   */
  const close = text.indexOf(
    quote,
    at,
  );
  /**
   Where the line ends, or minus one.
   */
  const newline = text.indexOf(
    '\n',
    at,
  );
  return (close !== (-1)) && ((newline === (-1)) || (close < newline));
}

/**
 Where a JavaScript comment opening at one offset ends.

 @param text - text under scan

 @param at - offset under the cursor

 @returns Offset just past a block comment's close or a line comment's line,
 or minus one where no comment opens here

 @example
 ```ts
 jsCommentEnd({ text: "/* cat's *\/ x", at: 0, },); // 11
 ```
 */
function jsCommentEnd(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): number {
  if (text.startsWith(
    '/*',
    at,
  )) {
    return pastMarker({
      text,
      marker: '*/',
      from: at + 2,
    },);
  }
  if (!text.startsWith(
    '//',
    at,
  ))
    return -1;
  /**
   Where the line comment's line ends.
   */
  const newline = text.indexOf(
    '\n',
    at,
  );
  return (newline === (-1)) ? text.length : newline;
}

/**
 Where a tag or JSX expression opened at one offset ends: the first `>` (for
 a tag) or the matching `}` (for an expression) outside quotes and nested
 braces.

 @param text - text under scan

 @param from - offset of the opening `<` or `{`

 @returns Offset just past the construct, or the text's length

 @example
 ```ts
 pastConstruct({ text: '<p a="x>y">', from: 0, },); // 11
 ```
 */
function pastConstruct(
  {
    text,
    from,
  }: {
    readonly text: string;
    readonly from: number;
  },
): number {
  /**
   Whether the construct is a tag, closed by `>`, or an expression, closed
   by its matching brace.
   */
  const isTag = text.charAt(from,) === '<';
  /**
   Brace depth, quote in force and position, carried through the scan.
   */
  const state = {
    depth: isTag ? 0 : 1,
    quote: '',
    at: from + 1,
  };
  while (state.at < text.length) {
    /**
     Where a comment opening here ends, inside an expression, where quote
     marks are prose (ledger K7: "{/* cat's note *\/}" opened a quote that
     never closed and shielded the rest of the text).
     */
    const commentEnd = ((state.quote === '') && ((!isTag) || (state.depth > 0)))
      ? jsCommentEnd({
        text,
        at: state.at,
      },)
      : -1;
    if (commentEnd !== (-1)) {
      state.at = commentEnd;
      continue;
    }
    /**
     Character under the cursor.
     */
    const character = text.charAt(state.at,);
    state.at += 1;
    if (state.quote !== '') {
      if (character === state.quote)
        state.quote = '';
      continue;
    }
    /**
     Whether an attribute value may run on across lines here, as a tag's
     does; a JavaScript string in an expression ends on its line, so a quote
     mark with no partner there is a stray apostrophe, not a string.
     */
    const multiline = isTag && (state.depth === 0);
    if (character === '`')
      state.quote = character;
    else if ((character === '"') || (character === '\'')) {
      if (multiline || closesOnLine({
        text,
        at: state.at,
        quote: character,
      },))
        state.quote = character;
    } else if (character === '{')
      state.depth += 1;
    else if (character === '}') {
      state.depth -= 1;
      if ((!isTag) && (state.depth === 0))
        return state.at;
    } else if (isTag && (character === '>')
      && (state.depth === 0))
      return state.at;
  }
  return text.length;
}

/**
 Where the construct starting at one offset ends, or minus one where no
 protected construct starts there.

 @param text - text under scan

 @param at - offset under the cursor

 @param codeSpans - every inline code span the real parse reads in this
 text, by its opening offset

 @returns Offset just past the construct, or minus one

 @example
 ```ts
 constructEnd({ text: '`x` y', at: 0, codeSpans: new Map([[0, 3,],]), },); // 3
 ```
 */
function constructEnd(
  {
    text,
    at,
    codeSpans,
  }: {
    readonly text: string;
    readonly at: number;
    readonly codeSpans: ReadonlyMap<number, number>;
  },
): number {
  if (text.startsWith(
    COMMENT_OPEN,
    at,
  ))
    return pastMarker({
      text,
      marker: COMMENT_CLOSE,
      from: at + COMMENT_OPEN.length,
    },);
  if (text.startsWith(
    CODE_FENCE,
    at,
  ))
    return pastMarker({
      text,
      marker: CODE_FENCE,
      from: at + CODE_FENCE.length,
    },);
  /**
   Character under the cursor.
   */
  const character = text.charAt(at,);
  // The parse alone says where a code span opens. A check that a backtick
  // after a backtick opens none stood here from before the spans came off the
  // parse, and hid the span whose opening backtick follows an escaped one
  // (ledger B130).
  if (character === '`')
    return codeSpans.get(at,) ?? (-1);
  if ((character === '<') && opensMdxTag({
    text,
    at,
  },))
    return pastConstruct({
      text,
      from: at,
    },);
  if (character === '{')
    return pastConstruct({
      text,
      from: at,
    },);
  if (text.startsWith(
    '](',
    at,
  ))
    return pastMarker({
      text,
      marker: ')',
      from: at + 2,
    },);
  if (URL_SCHEMES.some(function opens(scheme,): boolean {
    return text.startsWith(
      scheme,
      at,
    );
  },)) {
    /**
     Where the URL stops.
     */
    let end = at;
    while ((end < text.length) && (!URL_END.has(text.charAt(end,),)))
      end += 1;
    return end;
  }
  return -1;
}

/**
 Every range of a text that is not prose, in order.

 @param text - text under scan

 @returns Protected ranges, non-overlapping and in order

 @example
 ```ts
 protectedRanges({ text: 'a `b` c', },); // [{ start: 2, end: 5 }]
 ```
 */
export function protectedRanges(
  { text, }: { readonly text: string; },
): readonly ProtectedRange[] {
  /**
   Ranges found so far.
   */
  const ranges: ProtectedRange[] = [];
  /**
   Where the scan resumes: past the front matter when the text opens with it.
   */
  let at = frontMatterEnd({ text, },);
  if (at > 0) {
    ranges.push({
      start: 0,
      end: at,
    },);
  }
  /**
   Every inline code span the real parse reads past the front matter,
   computed ONCE here rather than per backtick.
   */
  const codeSpans = inlineCodeSpans(proseBodyTree({ text, },),);
  while (at < text.length) {
    /**
     End of a construct opening here, or minus one.
     */
    const end = constructEnd({
      text,
      at,
      codeSpans,
    },);
    if (end === (-1)) {
      at += 1;
      continue;
    }
    ranges.push({
      start: at,
      end,
    },);
    at = end;
  }
  return ranges;
}

/**
 Whether a span lies wholly outside every protected range.

 @param ranges - protected ranges of the text

 @param start - span's first offset

 @param end - span's exclusive end

 @returns Whether no range overlaps the span

 @example
 ```ts
 inProse({ ranges: [{ start: 2, end: 5 }], start: 6, end: 8, },); // true
 ```
 */
export function inProse(
  {
    ranges,
    start,
    end,
  }: {
    readonly ranges: readonly ProtectedRange[];
    readonly start: number;
    readonly end: number;
  },
): boolean {
  return ranges.every(function apart(range,): boolean {
    return (range.end <= start) || (range.start >= end);
  },);
}
//endregion Prose ranges
