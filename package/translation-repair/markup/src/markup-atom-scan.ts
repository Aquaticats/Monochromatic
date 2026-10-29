import { footnoteLabelEnd, } from './footnote-label.ts';
import { opensMdxTag, } from './mdx-tag-start.ts';

//region Markup atom scan
// LEDGER L4, the owner's ruling of 2026-09-28 ("Markup atoms"): inside a quote
// an edit is licensed to change, footnote references, link destinations, MDX
// expressions, inline code and tags survive every edit except a removal an
// addition issue names. Envelopes and licensed quotes are the same quotes, so
// the preservation gate's bulk and distinctive rules measured nothing there
// (`residualTokens` was 0 on 536 of 540 regions), and a replay over 5,733
// recorded repair regions found footnotes lost on a quotation-mark claim and MDX
// braces lost on mistranslation claims.
//
// A SCAN OF THE TEXT, NOT A PARSE. An envelope is a fragment of one block,
// often cut mid-sentence, and a fragment parsed on its own can read a link's
// tail as prose; the literal syntax is what has to survive, so the literal
// syntax is what is read. An unclosed construct at the fragment's edge yields no
// atom rather than swallowing the rest.
//
// A MULTISET, NOT THE ORDERED SEQUENCE `protected-atom.ts` compares. That gate
// judges a whole paragraph a rewrite may only reword; an accuracy edit may
// legitimately move a footnote reference to the clause it belongs to, and what
// the ruling protects is that the atom survives, not where.
//
// One left-to-right pass: a code span is consumed whole before anything inside
// it is read, and a backslash escapes the character after it. A construct that
// never closes costs a scan to the fragment's end, so the worst case is
// quadratic in one envelope's length, which is bounded by its block.

/**
 What kind of markup an atom is, named in a refusal so a scorecard can tell
 them apart.

 @example
 ```ts
 const kind: MarkupAtomKind = 'footnote-reference';
 ```
 */
export type MarkupAtomKind =
  | 'footnote-reference'
    | 'link-destination'
    | 'mdx-expression'
    | 'inline-code'
    | 'tag';

/**
 How the editor sheet names each kind, in the order it lists them.

 A RECORD OVER EVERY KIND, so a kind the gate starts refusing cannot go
 unnamed on the sheet: a gate the editor is never told about refuses edits
 the editor had no way to avoid.

 @example
 ```ts
 const name = MARKUP_ATOM_SHEET_NAMES['inline-code'];
 ```
 */
export const MARKUP_ATOM_SHEET_NAMES: Readonly<Record<MarkupAtomKind, string>> = {
  'footnote-reference': 'footnote markers like [^1]',
  'link-destination': 'link destinations',
  'mdx-expression': 'expressions in braces',
  'inline-code': 'inline code in backticks',
  tag: 'tags',
};

/**
 Kinds whose atom is an identifier: a label or an address no checker judges,
 which an edit may move but never re-mark (ledger L4). The gate reads it and
 so does the editor sheet, so the two cannot disagree on which kinds these
 are.

 A DELIBERATE SUBSET of `MarkupAtomKind`. LEFT OUT are `mdx-expression`,
 `inline-code` and `tag`, which are neither labels nor addresses. A kind gained
 later is therefore not an identifier until it is added here.

 @example
 ```ts
 const isIdentifier = MARKUP_IDENTIFIER_KINDS.has('footnote-reference',);
 ```
 */
export const MARKUP_IDENTIFIER_KINDS: ReadonlySet<MarkupAtomKind> = new Set([
  'footnote-reference',
  'link-destination',
],);

/**
 One markup atom, as the exact bytes that must survive.

 @example
 ```ts
 const atom: MarkupAtom = { kind: 'footnote-reference', value: '[^1]', };
 ```
 */
export type MarkupAtom = {
  /**
   What the atom is.
   */
  readonly kind: MarkupAtomKind;

  /**
   Exact source text, delimiters included.
   */
  readonly value: string;
};

/**
 What a reader found at one position.
 */
type Found =
  | {
    /**
     A construct closed here.
     */
    readonly found: true;

    /**
     The atom it is.
     */
    readonly atom: MarkupAtom;

    /**
     Offset just past it.
     */
    readonly end: number;
  }
    | {
    /**
     Nothing closed; the scan moves on.
     */
      readonly found: false;

    /**
     Offset the scan resumes at.
     */
      readonly resume: number;
    };

/**
 Length of the run of one character starting at an offset.

 @param text - text scanned

 @param at - first offset of the run

 @param character - character the run repeats

 @returns Run length, zero when the offset holds another character

 @example
 ```ts
 const width = runLength({ text: '``a``', at: 0, character: '`', },);
 ```
 */
function runLength(
  {
    text,
    at,
    character,
  }: {
    readonly text: string;
    readonly at: number;
    readonly character: string;
  },
): number {
  /**
   Cursor past the run.
   */
  const cursor = { end: at, };
  while (text[cursor.end] === character)
    cursor.end += 1;
  return cursor.end - at;
}

/**
 Reads a code span opened by a backtick run, closed only by a run of the same
 length, as CommonMark closes one.

 @param text - text scanned

 @param at - offset of the opening run

 @returns Span, or where to resume past an opening run nothing closes

 @example
 ```ts
 const found = codeSpanAt({ text: 'type `meow` now', at: 5, },);
 ```
 */
function codeSpanAt({
  text,
  at,
}: {
  readonly text: string;
  readonly at: number
},): Found {
  /**
   Width of the opening run.
   */
  const width = runLength({
    text,
    at,
    character: '`',
  },);

  /**
   Cursor over candidate closing runs.
   */
  const cursor = { at: text.indexOf(
    '`',
    at + width,
  ), };
  while (cursor.at !== (-1)) {
    /**
     Width of this candidate closing run.
     */
    const closing = runLength({
      text,
      at: cursor.at,
      character: '`',
    },);
    if (closing === width)
      return {
        found: true,
        atom: {
          kind: 'inline-code',
          value: text.slice(
            at,
            cursor.at + width,
          ),
        },
        end: cursor.at + width,
      };
    cursor.at = text.indexOf(
      '`',
      cursor.at + closing,
    );
  }
  // An opening run nothing closes is literal backticks, all of them.
  return {
    found: false,
    resume: at + width,
  };
}

/**
 Reads a delimited construct whose opener nests, such as a brace or a
 parenthesis, from its opener to the matching closer on one depth count.

 @param text - text scanned

 @param at - offset of the opener

 @param open - character raising the depth

 @param close - character lowering it

 @param stopAtNewline - whether a newline ends the construct unclosed

 @returns Offset just past the matching closer, or -1 when none closes it

 @example
 ```ts
 const end = matchingClose({ text: '{a{b}}', at: 0, open: '{', close: '}', stopAtNewline: false, },);
 ```
 */
function matchingClose(
  {
    text,
    at,
    open,
    close,
    stopAtNewline,
  }: {
    readonly text: string;
    readonly at: number;
    readonly open: string;
    readonly close: string;
    readonly stopAtNewline: boolean;
  },
): number {
  /**
   Depth and cursor of the walk.
   */
  const walk = {
    depth: 0,
    at,
  };
  while (walk.at < text.length) {
    /**
     Character at the cursor.
     */
    const character = text[walk.at];
    if (character === '\\') {
      walk.at += 2;
      continue;
    }
    if (stopAtNewline && (character === '\n'))
      return -1;
    if (character === open)
      walk.depth += 1;
    else if (character === close) {
      walk.depth -= 1;
      if (walk.depth === 0)
        return walk.at + 1;
    }
    walk.at += 1;
  }
  return -1;
}

/**
 Opener of an HTML comment, which the corpus's build rewrites into a JSX
 comment before compiling, so it reads as a tag here; any other `<!` fails
 the compile and is prose.
 */
const COMMENT_OPENER = '<!--';

/**
 Closer of an HTML comment.
 */
const COMMENT_CLOSER = '-->';

/**
 Reads a tag: a comment from `<!--` to `-->`, or an element from `<` to the
 `>` that closes it, stepping over attribute expressions and quoted values.

 @param text - text scanned

 @param at - offset of the angle bracket, already known to open a tag

 @returns Offset just past the tag, or -1 when it never closes

 @example
 ```ts
 const end = tagEnd({ text: '<Paw side="left" />', at: 0, },);
 ```
 */
function tagEnd({
  text,
  at,
}: {
  readonly text: string;
  readonly at: number
},): number {
  if (text.startsWith(
    COMMENT_OPENER,
    at,
  )) {
    /**
     Where the comment's closer starts.
     */
    const closer = text.indexOf(
      COMMENT_CLOSER,
      at + COMMENT_OPENER.length,
    );
    return (closer === (-1)) ? -1 : closer + COMMENT_CLOSER.length;
  }
  /**
   Cursor and the quote character open at it, empty outside a value.
   */
  const walk = {
    at: at + 1,
    quote: '',
  };
  while (walk.at < text.length) {
    /**
     Character at the cursor.
     */
    const character = text[walk.at];
    if (walk.quote !== '') {
      if (character === walk.quote)
        walk.quote = '';
      walk.at += 1;
    }
    else if ((character === '"') || (character === '\'')) {
      walk.quote = character;
      walk.at += 1;
    }
    else if (character === '{') {
      /**
       End of the attribute expression.
       */
      const end = matchingClose({
        text,
        at: walk.at,
        open: '{',
        close: '}',
        stopAtNewline: false,
      },);
      if (end === (-1))
        return -1;
      walk.at = end;
    }
    else if (character === '>')
      return walk.at + 1;
    else
      walk.at += 1;
  }
  return -1;
}

/**
 Reads whatever construct opens at one offset.

 @param text - text scanned

 @param at - offset under the cursor

 @returns Atom there, or where to resume

 @example
 ```ts
 const found = atomAt({ text: 'a[^1]', at: 1, },);
 ```
 */
function atomAt({
  text,
  at,
}: {
  readonly text: string;
  readonly at: number
},): Found {
  /**
   Where to resume when nothing closes here.
   */
  const missed: Found = {
    found: false,
    resume: at + 1,
  };

  /**
   Closes a construct into an atom, or misses when it never closed.

   @param kind - kind of construct opened here

   @param valueStart - where the atom's bytes start

   @param end - offset past its closer, -1 when none

   @returns Atom, or the miss

   @example
   ```ts
   closeAs({ kind: 'tag', valueStart: at, end, },);
   ```
   */
  function closeAs(
    {
      kind,
      valueStart,
      end,
    }: {
      readonly kind: MarkupAtomKind;
      readonly valueStart: number;
      readonly end: number
    },
  ): Found {
    return (end === (-1))
      ? missed
      : {
        found: true,
        atom: {
          kind,
          value: text.slice(
            valueStart,
            end,
          ),
        },
        end,
      };
  }

  /**
   Character at the cursor.
   */
  const character = text[at];
  if (character === '`')
    return codeSpanAt({
      text,
      at,
    },);
  if (text.startsWith(
    '[^',
    at,
  )) {
    /**
     Where the label closes, absent when the text after the opener is no footnote label (`footnote-label.ts`).
     */
    const close = footnoteLabelEnd({
      text,
      from: at + 2,
    },);
    return (close === (-1))
      ? missed
      : closeAs({
        kind: 'footnote-reference',
        valueStart: at,
        end: close + 1,
      },);
  }
  if (text.startsWith(
    '](',
    at,
  ))
    return closeAs({
      kind: 'link-destination',
      valueStart: at + 1,
      end: matchingClose({
        text,
        at: at + 1,
        open: '(',
        close: ')',
        stopAtNewline: true,
      },),
    },);
  if (character === '{')
    return closeAs({
      kind: 'mdx-expression',
      valueStart: at,
      end: matchingClose({
        text,
        at,
        open: '{',
        close: '}',
        stopAtNewline: false,
      },),
    },);
  if ((character === '<') && (opensMdxTag({
    text,
    at,
  },) || text.startsWith(
    COMMENT_OPENER,
    at,
  )))
    return closeAs({
      kind: 'tag',
      valueStart: at,
      end: tagEnd({
        text,
        at,
      },),
    },);
  return missed;
}

/**
 Every markup atom in a text, in the order they appear.

 @param text - envelope text or quote, any fragment of a block

 @returns Atoms found

 @example
 ```ts
 const atoms = scanMarkupAtoms({ text: 'The cat napped[^1].', },);
 ```
 */
export function scanMarkupAtoms({ text, }: { readonly text: string; },): readonly MarkupAtom[] {
  /**
   Atoms in appearance order.
   */
  const atoms: MarkupAtom[] = [];

  /**
   Cursor over the text.
   */
  const cursor = { at: 0, };
  while (cursor.at < text.length) {
    if (text[cursor.at] === '\\') {
      // An escaped character is literal, whatever it is.
      cursor.at += 2;
      continue;
    }
    /**
     What opens here, if anything.
     */
    const found = atomAt({
      text,
      at: cursor.at,
    },);
    if (found.found) {
      atoms.push(found.atom,);
      cursor.at = found.end;
    }
    else
      cursor.at = found.resume;
  }
  return atoms;
}

//endregion Markup atom scan
