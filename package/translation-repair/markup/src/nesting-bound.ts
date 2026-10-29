import {
  certainFenceOf,
  fenceOf,
  type FenceRun,
  NO_FENCE,
} from './nesting-fence.ts';
import {
  htmlBlockAfter,
  htmlBlockAfterBlank,
} from './nesting-html-block.ts';
import {
  type ScanState,
  scanInline,
} from './nesting-inline-count.ts';
import {
  blanksEnd,
  containerPrefixOf,
  indentationOf,
  isDefinitionLine,
  isRuleLine,
  linesAtEveryEnding,
} from './nesting-line-lexing.ts';
import {
  NESTING_BOUND,
  type NestingExcess,
  type NestingGrammar,
  type NestingReading,
  STACK_OVERFLOW_ACCOUNT,
  STACK_OVERFLOW_MESSAGE,
} from './nesting-vocabulary.ts';

//region Nesting bound
// A TEXT NESTED DEEPLY ENOUGH IS REFUSED BEFORE ANY PARSER READS IT, in one linear pass.
//
// WHY A BOUND AT ALL. The Markdown parser descends once per nesting level, and
// so does the tree transform it runs on its own output, so a reply of thousands
// of nested quotation markers exhausts the stack, and whether it does depends on
// how deep the caller already stands: 5,989 quotation markers parsed from a
// shallow caller and 5,957 from the translate stage's call chain (measured on
// the built package, one child process per depth, default stack). The same reply
// could pass in one place and fail in another. Other shapes never overflow and
// cost minutes instead: a link nested in a link grows faster than the text
// (2.3 s at 6,400 levels, over 40 s at 25,600), and so do nested images and
// unmatched emphasis delimiters. A bound checked first makes the outcome a
// property of the text.
//
// WHAT IS MEASURED, lexically, because the parser is the thing being protected:
// the container markers opening a line (quotation marks, list markers, footnote
// definition labels), the indentation of a line, the open brackets of a
// paragraph, the open tags and open braces of an MDX body, and the emphasis
// delimiters of a block. Each is read against one bound. The deepest nesting of
// any page of the pinned corpus is far below it (`package/translation-repair/doc/audit-ledger.md`, the entry
// that records the bound).
//
// THE MEASURE ERRS TOWARD REFUSING, never toward reading: it counts a bracket
// inside a code span and an open tag it cannot match, which only a text no
// reader writes would pile up. It SKIPS A STRETCH ONLY WHERE THE PARSER
// CERTAINLY READS NO NESTING IN IT: the inline counts of a fenced block
// that opens at the start of a line outside every container and every raw
// html block (and, under the strict grammar, every math block, expression and
// open tag), and the delimiters of a thematic break. Anywhere else the scan
// counts, so a fence in a quotation or a list item, an indented one, one with
// a backtick in its backtick info string, and one the parser takes for raw
// text all count the lines after them. Measured on the built package before
// this rule: each of those passed a text that the parser then nested.

/**
 Columns of indentation that make one level of nesting.
 */
const INDENT_COLUMNS_PER_LEVEL = 2;

/**
 Whether a line that opens like a fence opens one the parser certainly reads.

 A FENCE LINE IS NO FENCE where the parser reads raw text: inside an html
 block, inside a math block under the strict grammar, and inside an
 expression or a tag still open there. The scan counts the lines after a
 fence it cannot be certain of, which is the direction that only refuses more.

 @param state - counts carried from line to line

 @param grammar - grammar that will read the body

 @returns True when no raw text the parser reads could hold the line

 @example
 ```ts
 const certain = fenceIsCertain({ state, grammar: 'markdown', },);
 ```
 */
function fenceIsCertain(
  {
    state,
    grammar,
  }: {
    readonly state: Readonly<ScanState>;
    readonly grammar: NestingGrammar;
  },
): boolean {
  if (state.html !== 'none')
    return false;
  if (grammar !== 'mdx')
    return true;
  if (state.mathSeen)
    return false;
  if (state.braces > 0)
    return false;
  return state.pendingTag === 'none';
}

/**
 Whether a line closes a fence.

 A closing fence is the same character, at least as long, with nothing but
 blanks after it, wherever it is indented: closing a fence the parser still
 reads only makes the lines after it count.

 @param open - fence that is open

 @param run - fence run the line begins with after its markers and blanks

 @param line - whole line

 @param restStart - index where the line's text begins after its markers and blanks

 @returns True when the line closes the fence

 @example
 ```ts
 const closes = closesFence({ open: { character: '`', length: 3, }, run: { character: '`', length: 3, }, line: '```', restStart: 0, },);
 ```
 */
function closesFence(
  {
    open,
    run,
    line,
    restStart,
  }: {
    readonly open: FenceRun;
    readonly run: FenceRun;
    readonly line: string;
    readonly restStart: number;
  },
): boolean {
  return (run.character === open.character)
    && (run.length >= open.length)
    && (blanksEnd({
      line,
      from: restStart + run.length,
    },) === line.length);
}

/**
 Reads a line for the fence it opens or closes, under two readings of what a
 fence is.

 THE CERTAIN READING SKIPS, THE LOOSE READING ONLY WATCHES. A fence the parser
 certainly reads (`certainFenceOf`) makes every line of it uncounted, up to a
 line that closes it. A fence line the scan cannot be certain of (indented,
 in a container, with a backtick in its info string, in raw html) is kept
 open in the loose reading, in which the lines after it are counted, and
 which only a closing line ends: the parser may be inside that fence, so a
 fence line after it may close it where this scan would otherwise take it for
 the opening of a fence of its own and skip what follows.

 @param line - one line without its newline, not blank

 @param rest - line after its container markers and blanks

 @param restStart - index where `rest` begins in the line

 @param state - counts carried from line to line, updated in place

 @param grammar - grammar that will read the body

 @returns True when the line belongs to a fence the parser certainly reads, so
 nothing on it is counted

 @example
 ```ts
 const skipped = readFence({ line: '```ts', rest: '```ts', restStart: 0, state, grammar: 'markdown', },);
 ```
 */
function readFence(
  {
    line,
    rest,
    restStart,
    state,
    grammar,
  }: {
    readonly line: string;
    readonly rest: string;
    readonly restStart: number;
    readonly state: ScanState;
    readonly grammar: NestingGrammar;
  },
): boolean {
  /**
   Fence run this line begins with after its markers and blanks.
   */
  const run = fenceOf({ line: rest, },);
  /**
   The fence the parser certainly reads and the one it may be inside, each
   none when the body is inside no such fence.
   */
  const {
    fence: certain,
    loose,
  } = state;
  if (certain.length > 0) {
    /**
     Whether this line closes the fence the parser certainly reads.
     */
    const closed = closesFence({
      open: certain,
      run,
      line,
      restStart,
    },);
    if (closed)
      state.fence = NO_FENCE;
    return true;
  }
  if (loose.length > 0) {
    /**
     Whether this line closes the fence the parser may be inside.
     */
    const closedLoose = closesFence({
      open: loose,
      run,
      line,
      restStart,
    },);
    if (closedLoose)
      state.loose = NO_FENCE;
    return closedLoose;
  }
  if (run.length === 0)
    return false;
  /**
   Fence this line opens, when the parser certainly reads one here.
   */
  const opener = fenceIsCertain({
    state,
    grammar,
  },)
    ? certainFenceOf({ line, },)
    : NO_FENCE;
  if (opener.length > 0) {
    state.fence = opener;
    return true;
  }
  state.loose = run;
  return false;
}

/**
 Reads one line against every measure, updating the counts it carries.

 @param line - one line without its newline, not blank

 @param lineNumber - one-based number of the line

 @param grammar - grammar that will read the body

 @param state - counts carried from line to line, updated in place

 @returns First excess on the line, or that it stays within

 @example
 ```ts
 const reading = readLine({ line: '> cat', lineNumber: 1, grammar: 'markdown', state, },);
 ```
 */
function readLine(
  {
    line,
    lineNumber,
    grammar,
    state,
  }: {
    readonly line: string;
    readonly lineNumber: number;
    readonly grammar: NestingGrammar;
    readonly state: ScanState;
  },
): NestingReading {
  /**
   The line's leading spaces and tabs.
   */
  const indentation = indentationOf({ line, },);
  if (Math.floor(indentation.columns / INDENT_COLUMNS_PER_LEVEL,) > NESTING_BOUND)
    return {
      kind: 'beyond',
      measure: 'indentation',
      bound: NESTING_BOUND,
      line: lineNumber,
      column: indentation.end + 1,
    };
  /**
   Container markers that open the line.
   */
  const prefix = containerPrefixOf({
    line,
    start: indentation.end,
  },);
  if (prefix.beyondColumn > 0)
    return {
      kind: 'beyond',
      measure: 'container markers',
      bound: NESTING_BOUND,
      line: lineNumber,
      column: prefix.beyondColumn,
    };
  /**
   Index where the line's text begins, after its markers and the blanks that follow.
   */
  const restStart = blanksEnd({
    line,
    from: prefix.end,
  },);
  /**
   The line's text after its markers and indentation.
   */
  const rest = line.slice(restStart,);
  if (readFence({
    line,
    rest,
    restStart,
    state,
    grammar,
  },))
    return { kind: 'within', };
  // A line that is no fence may open raw html or, under the strict grammar, a
  // math block, inside which a line that looks like a fence opens none.
  state.html = htmlBlockAfter({
    before: state.html,
    rest,
  },);
  if ((grammar === 'mdx') && rest.startsWith('$$',))
    state.mathSeen = true;
  /**
   How many braces were open before the line.
   */
  const bracesBefore = state.braces;
  /**
   What the line did to the counts.
   */
  const reading = scanInline({
    line,
    start: prefix.end,
    lineNumber,
    grammar,
    state,
    skipDelimiters: isRuleLine({ line: rest, },),
  },);
  // A definition takes a brace for a character of its destination or title, so none of its braces stays open past it,
  // though they all counted against the bound on the line itself.
  if (isDefinitionLine({ line: rest, }))
    state.braces = bracesBefore;
  return reading;
}

/**
 Reads a body against the nesting bound, in one pass over its characters.

 @param body - text about to be parsed

 @param grammar - grammar that will read it

 @returns Where the body first passes the bound, or that it stays within

 @example
 ```ts
 const reading = firstNestingExcess({ body: `${'>'.repeat(300,)} cat`, grammar: 'markdown', },);
 // => { kind: 'beyond', measure: 'container markers', bound: 256, line: 1, column: 257, }
 ```
 */
export function firstNestingExcess(
  {
    body,
    grammar,
  }: {
    readonly body: string;
    readonly grammar: NestingGrammar;
  },
): NestingReading {
  /**
   Counts carried from line to line.
   */
  const state: ScanState = {
    brackets: 0,
    delimiters: 0,
    tags: 0,
    braces: 0,
    pendingTag: 'none',
    fence: NO_FENCE,
    loose: NO_FENCE,
    html: 'none',
    mathSeen: false,
  };
  /**
   Lines of the body as the parser reads them, a leading byte order mark
   dropped.
   */
  const lines = linesAtEveryEnding({ body: body.startsWith('\uFEFF',) ? body.slice(1,) : body, },);
  for (const [position, line,] of lines.entries()) {
    /**
     Index after the line's leading blanks, the line's length when it is all blank.
     */
    const blankEnd = blanksEnd({
      line,
      from: 0,
    },);
    if (blankEnd === line.length) {
      // A blank line ends the paragraph and the block, so neither count
      // carries past it.
      state.brackets = 0;
      state.delimiters = 0;
      state.html = htmlBlockAfterBlank({ before: state.html, },);
      continue;
    }
    /**
     What this line did to the counts.
     */
    const reading = readLine({
      line,
      lineNumber: position + 1,
      grammar,
      state,
    },);
    if (reading.kind === 'beyond')
      return reading;
  }
  return { kind: 'within', };
}

/**
 Whether a caught value is the engine's stack exhaustion.

 @param value - what a catch caught, of unknown type by construction

 @returns True for a `RangeError` whose message starts as the engine's stack
 exhaustion does, false for every other value including another `RangeError`

 @example
 ```ts
 if (isStackOverflow(caughtValue,))
   return refusal;
 ```
 */
export function isStackOverflow(value: unknown,): value is RangeError {
  if (!(value instanceof RangeError))
    return false;
  return value.message
    .startsWith(STACK_OVERFLOW_MESSAGE,);
}

/**
 Says in words where a body passes the bound, quoting nothing it read.

 @param excess - where the body passed the bound

 @returns Phrase naming the measure, the bound and the place

 @example
 ```ts
 nestingAccount({ excess: { measure: 'brackets', bound: 256, line: 3, column: 9, }, },);
 // => 'nested too deeply to read: its brackets pass the bound of 256 at line 3, column 9'
 ```
 */
export function nestingAccount({ excess, }: { readonly excess: NestingExcess; },): string {
  return `nested too deeply to read: its ${excess.measure} pass the bound of ${excess.bound} at line ${excess.line}, `
    + `column ${excess.column}`;
}

/**
 Says in words why a body is refused, quoting nothing it read: where it passed
 the bound, or that the parser exhausted its stack.

 @param reading - where the body passed the bound, or `'stack-exhausted'`

 @returns Account {@link nestingAccount} gives for a place, or
 {@link STACK_OVERFLOW_ACCOUNT} for an exhaustion

 @example
 ```ts
 const account = nestingRefusalAccount({ reading: 'stack-exhausted', },);
 // => 'nested too deeply to read: the parser exhausted its stack'
 ```
 */
export function nestingRefusalAccount(
  { reading, }: { readonly reading: NestingExcess | 'stack-exhausted'; },
): string {
  if (reading === 'stack-exhausted')
    return STACK_OVERFLOW_ACCOUNT;
  return nestingAccount({ excess: reading, },);
}

/**
 Carries where a body passed the bound, for the strict grammar's refusal to
 read position and account from.

 Its own message is the account alone, so a reader of the cause sees no text
 of the body. `place` is the point the grammar's refusals name theirs by.

 @example
 ```ts
 throw new NestingBoundError({ excess, },);
 ```
 */
export class NestingBoundError extends Error {
  /**
   Declares this message safe to forward: it states which measure passed the
   bound and where, and repeats no document text.
   */
  readonly messageNamesOnly: true = true;

  /**
   Where the body passed the bound.
   */
  readonly excess: NestingExcess;

  /**
   Line and column of the item that passed the bound, as a parser message
   names its stop.
   */
  readonly place: {
    readonly line: number;
    readonly column: number;
  };

  /**
   Builds the error from where the body passed the bound.

   @param excess - where the body passed the bound

   @example
   ```ts
   new NestingBoundError({ excess, },);
   ```
   */
  public constructor({ excess, }: { readonly excess: NestingExcess; },) {
    super(nestingAccount({ excess, },),);
    this.name = 'NestingBoundError';
    this.excess = excess;
    this.place = {
      line: excess.line,
      column: excess.column,
    };
  }
}

//endregion Nesting bound
