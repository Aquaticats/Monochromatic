import {
  certainFenceOf,
  fenceOf,
  indentationOf,
  NO_FENCE,
} from '@monochromatic-dev/translation-repair-markup/ts';

//region Source ships encoding
// THE TEXT OF A SLICE THAT SHIPS ITS ORIGINAL, AS LITERAL TEXT FOR THE STRICT MDX GRAMMAR. The page carries the
// original as the artifact records it, and writes it through here only when the page with the original as recorded
// does not parse (`source-ships-compose.ts` says when). Read off the grammar the package parses with
// (`document-structure/src/parse-mdx.ts`: `remark-mdx` over `remark-gfm` and `remark-math`, with no JavaScript parser
// behind it), MDX reads as syntax a `{` (an expression, closed by a `}` it counts), a `<` (a tag), and, in the
// plain markdown under it, a backslash before an ASCII punctuation character (an escape). So this writes a backslash
// before each `{`, `}` and `<`, which each of those reads as that character, and nothing else:
//
// - `>` stays. It is syntax in MDX only inside a tag, and a `<` written as `\<` opens none; at the start of a line it
//   is a block quote of the original, which encoding would turn into a line starting with a greater-than sign.
// - Footnote references `[^n]`, definitions `[^n]:`, links, headings and every Chinese character stay as they are.
// - A backslash that already escapes an ASCII punctuation character, `\{` or `\\` among them, is stepped over as the
//   pair, so an escape the original wrote is not doubled into a backslash and an open brace.
// - Fenced code, an inline code span closed on its own line, and an HTML comment are copied as they are: the grammar
//   reads none of them as syntax, and a backslash written into code would show. A fence is read by the lexer the
//   nesting bound uses (`markup/src/nesting-fence.ts`), so the two agree on what opens and closes one.
//
// KNOWN LIMITS, kept because the composition in `source-ships-compose.ts` ends in shipping the page without the
// original for a text this does not make parse. A code span that runs across lines is read as text, so a brace in it is
// escaped and the escape shows. Math between dollar signs is read as text too; a backslash before a brace is how TeX
// writes a literal one, so it shows as one. A line starting `import ` or `export ` is an ES module line to MDX, which
// no backslash can make plain; the parser here has no JavaScript parser behind it and reads such a line as text, so
// the case needs nothing.

/**
 Characters a backslash escapes in Markdown: the ASCII punctuation.
 */
const ESCAPABLE = '!"#$%&\'()*+,-./:;<=>?@[\\]^_`{|}~';

/**
 What may follow a closing fence on its line: spaces, tabs and the carriage return of a line split at its line feed.
 */
const TRAILING_WHITE_SPACE = ' \t\r';

/**
 What opens an HTML comment.
 */
const COMMENT_OPEN = '<!--';

/**
 What closes an HTML comment.
 */
const COMMENT_CLOSE = '-->';

/**
 How many spaces may stand before a fence and still open one.
 */
const MAX_FENCE_INDENT = 3;

/**
 How many characters a backslash and the character it escapes take.
 */
const ESCAPE_LENGTH = 2;

/**
 What a search for the end of a span answers when the span does not end.
 */
const NOT_FOUND = -1;

/**
 A run of fence characters, as the lexer of `markup` reads one.
 */
type FenceRun = ReturnType<typeof fenceOf>;

/**
 What the scan carries from one line to the next.
 */
type EncodingState = {
  /**
   The run of characters the open fence was opened with, `NO_FENCE` while none is open.
   */
  openFence: FenceRun;

  /**
   Whether the scan is inside an HTML comment that has not closed.
   */
  inComment: boolean;
};

/**
 What the scan carries inside one line.
 */
type Cursor = {
  /**
   Index of the character being read.
   */
  at: number;

  /**
   Index where the stretch not yet copied starts.
   */
  copiedTo: number;
};

/**
 Whether a character is one a backslash escapes.

 @param character - one character, or none at the end of a line, which no backslash escapes

 @returns Whether the character is ASCII punctuation

 @example
 ```ts
 const escapable = isEscapable({ character: '{', },);
 ```
 */
function isEscapable({ character, }: { readonly character: string; },): boolean {
  return (character.length === 1) && ESCAPABLE.includes(character,);
}

/**
 How many times a character repeats from the start of a text.

 @param text - text read from its start

 @param mark - character

 @returns Length of the run, zero when the text does not start with the character

 @example
 ```ts
 const length = runLength({ text: '``a', mark: '`', },);
 ```
 */
function runLength(
  {
    text,
    mark,
  }: {
    readonly text: string;
    readonly mark: string;
  },
): number {
  for (let at = 0; at < text.length; at += 1) {
    if (text.charAt(at,) !== mark)
      return at;
  }
  return text.length;
}

/**
 The fence a line outside any fence opens a block with.

 @param line - line outside any fence

 @returns Run (three backticks or tildes at least) or `NO_FENCE`; a backtick run followed by a backtick on the line
 is a code span and not a fence, and a line indented four columns is code the grammar does not have

 @example
 ```ts
 const run = fenceOpenedBy({ line: '```ts', },);
 ```
 */
function fenceOpenedBy({ line, }: { readonly line: string; },): FenceRun {
  /**
   Width of the indentation and where it ends.
   */
  const indentation = indentationOf({ line, },);
  if (indentation.columns > MAX_FENCE_INDENT)
    return NO_FENCE;
  return certainFenceOf({ line: line.slice(indentation.end,), },);
}

/**
 Whether a line closes the fence a block opened with.

 @param line - line inside the fence

 @param opened - run of characters the fence was opened with

 @returns Whether the line is the same character at least as many times, with nothing after it but spaces, tabs and
 the carriage return of a line split at its line feed

 @example
 ```ts
 const closes = closesFence({ line: '```', opened: fenceOpenedBy({ line: '```ts', },), },);
 ```
 */
function closesFence(
  {
    line,
    opened,
  }: {
    readonly line: string;
    readonly opened: FenceRun;
  },
): boolean {
  /**
   Width of the indentation and where it ends.
   */
  const indentation = indentationOf({ line, },);
  if (indentation.columns > MAX_FENCE_INDENT)
    return false;
  /**
   The line without its indentation.
   */
  const rest = line.slice(indentation.end,);
  /**
   The run of fence characters the line starts with.
   */
  const run = fenceOf({ line: rest, },);
  /**
   What follows the run, which may hold spaces and tabs only for the line to close the fence.
   */
  const after = rest.slice(run.length,);
  return (run.character === opened.character)
    && (run.length >= opened.length)
    && after
      .split('',)
      .every(function isSpaceTabOrReturn(character,): boolean {
        return TRAILING_WHITE_SPACE.includes(character,);
      },);
}

/**
 Where the backtick run that closes a code span opened by a run of the given length starts, on the same line.

 @param line - line holding the span

 @param from - index just past the opening run

 @param length - length of the opening run, which the closing run must equal exactly

 @returns Index of the closing run's first backtick, or {@link NOT_FOUND} for a span that does not close on this line

 @example
 ```ts
 const close = closingRunAt({ line: '`a` b', from: 1, length: 1, },);
 ```
 */
function closingRunAt(
  {
    line,
    from,
    length,
  }: {
    readonly line: string;
    readonly from: number;
    readonly length: number;
  },
): number {
  for (let at = from; at < line.length; at += 1) {
    if (line.charAt(at,) !== '`')
      continue;
    /**
     The run of backticks starting here.
     */
    const run = runLength({
      text: line.slice(at,),
      mark: '`',
    },);
    if (run === length)
      return at;
    // Past the whole run, so a longer run is not read as the shorter run it contains.
    at += run - 1;
  }
  return NOT_FOUND;
}

/**
 Copies the stretch of a line not yet copied, up to the scan position, then a backslash, and moves the copy mark to
 the scan position.

 @param pieces - pieces of the encoded line, to which the stretch and the backslash are appended

 @param line - line being encoded

 @param cursor - scan position; its copy mark is moved

 @example
 ```ts
 escapeHere({ pieces, line, cursor, },);
 ```
 */
function escapeHere(
  {
    pieces,
    line,
    cursor,
  }: {
    readonly pieces: string[];
    readonly line: string;
    readonly cursor: Cursor;
  },
): void {
  pieces.push(
    line.slice(
      cursor.copiedTo,
      cursor.at,
    ),
    '\\',
  );
  cursor.copiedTo = cursor.at;
}

/**
 Encodes one line outside any fence, carrying whether an HTML comment is open.

 @param line - line of the text, without its newline

 @param state - what the scan carries; its comment flag is read and written

 @returns Line with a backslash before each brace and each less-than sign that is syntax

 @example
 ```ts
 const encoded = encodeLine({ line: '她说 {x}', state: { openFence: NO_FENCE, inComment: false, }, },);
 ```
 */
function encodeLine(
  {
    line,
    state,
  }: {
    readonly line: string;
    readonly state: EncodingState;
  },
): string {
  /**
   Pieces of the encoded line, joined at the end.
   */
  const pieces: string[] = [];
  /**
   Where the scan stands and what it has copied.
   */
  const cursor: Cursor = {
    at: 0,
    copiedTo: 0,
  };
  if (state.inComment) {
    /**
     Where the comment closes on this line.
     */
    const close = line.indexOf(COMMENT_CLOSE,);
    if (close === NOT_FOUND)
      return line;
    state.inComment = false;
    cursor.at = close + COMMENT_CLOSE.length;
  }
  while (cursor.at < line.length) {
    /**
     Character being read.
     */
    const character = line.charAt(cursor.at,);
    if (character === '\\') {
      // A BACKSLASH BEFORE PUNCTUATION IS AN ESCAPE ALREADY, and is stepped over as the pair it is.
      cursor.at += isEscapable({ character: line.charAt(cursor.at + 1,), },) ? ESCAPE_LENGTH : 1;
      continue;
    }
    if (character === '`') {
      /**
       Length of the run of backticks.
       */
      const length = runLength({
        text: line.slice(cursor.at,),
        mark: '`',
      },);
      /**
       Where the span this run opens closes, if it closes on this line.
       */
      const close = closingRunAt({
        line,
        from: cursor.at + length,
        length,
      },);
      cursor.at = (close === NOT_FOUND) ? (cursor.at + length) : (close + length);
      continue;
    }
    if ((character === '{') || (character === '}')) {
      escapeHere({
        pieces,
        line,
        cursor,
      },);
      cursor.at += 1;
      continue;
    }
    if (character === '<') {
      if (line.startsWith(
        COMMENT_OPEN,
        cursor.at,
      )) {
        /**
         Where the comment closes on this line.
         */
        const close = line.indexOf(
          COMMENT_CLOSE,
          cursor.at + COMMENT_OPEN.length,
        );
        if (close === NOT_FOUND) {
          state.inComment = true;
          break;
        }
        cursor.at = close + COMMENT_CLOSE.length;
        continue;
      }
      escapeHere({
        pieces,
        line,
        cursor,
      },);
    }
    cursor.at += 1;
  }
  pieces.push(line.slice(cursor.copiedTo,),);
  return pieces.join('',);
}

/**
 Writes the text of a slice so that the strict MDX grammar reads it as literal text.

 A backslash goes before each `{`, `}` and `<` that is syntax, and nothing else changes: the module comment says what
 stays and why.

 @param text - slice text as the artifact records it

 @returns Text with those characters escaped, equal to the text when it holds none

 @example
 ```ts
 const literal = encodeAsLiteralMdx({ text: '她说 { 然后走了。', },); // '她说 \\{ 然后走了。'
 ```
 */
export function encodeAsLiteralMdx({ text, }: { readonly text: string; },): string {
  /**
   What the scan carries from line to line.
   */
  const state: EncodingState = {
    openFence: NO_FENCE,
    inComment: false,
  };
  return text
    .split('\n',)
    .map(function encodeIt(line,): string {
      /**
       The run the open fence was opened with, `NO_FENCE` while none is open.
       */
      const { openFence, } = state;
      if (openFence.length > 0) {
        if (closesFence({
          line,
          opened: openFence,
        },))
          state.openFence = NO_FENCE;
        return line;
      }
      if (!state.inComment) {
        /**
         The run this line opens a fence with, when it opens one.
         */
        const opening = fenceOpenedBy({ line, },);
        if (opening.length > 0) {
          state.openFence = opening;
          return line;
        }
      }
      return encodeLine({
        line,
        state,
      },);
    },)
    .join('\n',);
}

//endregion Source ships encoding
