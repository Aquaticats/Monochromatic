import { readTagName, } from './mdx-tag-name.ts';
import type { FenceRun, } from './nesting-fence.ts';
import type { HtmlBlock, } from './nesting-html-block.ts';
import { isOneOf, } from './nesting-line-lexing.ts';
import {
  DELIMITER_BOUND,
  NESTING_BOUND,
  type NestingGrammar,
  type NestingMeasure,
  type NestingReading,
} from './nesting-vocabulary.ts';

//region Nesting inline count
// COUNTS WHAT A LINE OPENS AND CLOSES: brackets, emphasis delimiters, and under the
// strict grammar open tags and open braces, each carried from line to line in one
// mutable state the scan owns, and read against its bound as it grows.
//
// A TAG OPENS WHERE THE STRICT GRAMMAR READS ITS NAME, through the package's one
// reader of a tag name (`mdx-tag-name.ts`). This count once read a name as ASCII
// letters and digits, so a tag named in another script or opening with `$` or `_`,
// which the grammar nests, opened nothing here, and a name holding a character the
// grammar refuses opened one.

/**
 What the scan reads past a line's end when a tag's name or the whitespace after
 it reaches that end: a line ending, which the strict grammar steps over inside a
 tag as it steps over a space, then a name's letter and the tag's bracket. The
 next line is out of the scan's sight, and the grammar reads `<Paw`, `<Paw.` and
 `</` at a line's end as a tag the next line goes on with; where the next line
 does not go on with it, the page fails its compile, so a tag counted for it is
 one the bound can afford.
 */
const LINE_GOES_ON = '\nb>';

/**
 HTML elements that never hold content, so an open tag of one nests nothing.
 */
const VOID_ELEMENTS: ReadonlySet<string> = new Set([
  'area',
  'base',
  'br',
  'col',
  'embed',
  'hr',
  'img',
  'input',
  'link',
  'meta',
  'param',
  'source',
  'track',
  'wbr',
],);

/**
 The counts one scan carries from line to line.
 */
type Counted = 'brackets' | 'delimiters' | 'tags' | 'braces';

/**
 Whether a tag was begun whose closing angle bracket is still to come.
 */
type PendingTag = 'none' | 'opening' | 'closing';

/**
 Mutable state one scan carries from line to line.
 */
export type ScanState = {
  /**
   Open brackets of the paragraph being read.
   */
  brackets: number;

  /**
   Emphasis delimiters of the block being read.
   */
  delimiters: number;

  /**
   Open tags not yet closed.
   */
  tags: number;

  /**
   Open braces not yet closed.
   */
  braces: number;

  /**
   Whether a tag was begun whose closing angle bracket is still to come.
   */
  pendingTag: PendingTag;

  /**
   The fence run the body is inside, none when it is inside no fence. Set
   only for a fence the parser certainly reads (`nesting-bound.ts`).
   */
  fence: FenceRun;

  /**
   A fence the parser may be inside that the scan cannot be certain of, in
   which the lines are counted and a fence line may close it, none when the
   body is inside no such fence.
   */
  loose: FenceRun;

  /**
   Raw html the parser may be reading, inside which no line opens a fence.
   */
  html: HtmlBlock;

  /**
   Whether a line opening a math block has been read, after which the strict
   grammar may read raw math where a line looks like a fence.
   */
  mathSeen: boolean;
};

/**
 What the measure names for each count.
 */
const COUNTED_MEASURE: Readonly<Record<Counted, NestingMeasure>> = {
  brackets: 'brackets',
  delimiters: 'emphasis delimiters',
  tags: 'open tags',
  braces: 'open braces',
};

/**
 The bound each count is read against.
 */
const COUNTED_BOUND: Readonly<Record<Counted, number>> = {
  brackets: NESTING_BOUND,
  delimiters: DELIMITER_BOUND,
  tags: NESTING_BOUND,
  braces: NESTING_BOUND,
};

/**
 Which count a character opens.

 @param character - one UTF-16 unit

 @param grammar - grammar that will read the body

 @returns Count it opens, or that it opens none

 @example
 ```ts
 opensOf({ character: '[', grammar: 'markdown', },);
 // => 'brackets'
 ```
 */
function opensOf(
  {
    character,
    grammar,
  }: {
    readonly character: string;
    readonly grammar: NestingGrammar;
  },
): Counted | 'none' {
  if (character === '[')
    return 'brackets';
  if (isOneOf({
    character,
    set: '*_~',
  },))
    return 'delimiters';
  if ((grammar === 'mdx') && (character === '{'))
    return 'braces';
  return 'none';
}

/**
 Which count a character closes.

 @param character - one UTF-16 unit

 @param grammar - grammar that will read the body

 @returns Count it closes, or that it closes none

 @example
 ```ts
 closesOf({ character: ']', grammar: 'markdown', },);
 // => 'brackets'
 ```
 */
function closesOf(
  {
    character,
    grammar,
  }: {
    readonly character: string;
    readonly grammar: NestingGrammar;
  },
): Counted | 'none' {
  if (character === ']')
    return 'brackets';
  if ((grammar === 'mdx') && (character === '}'))
    return 'braces';
  return 'none';
}

/**
 Counts one more of something against its bound.

 @param counted - which count grows

 @param lineNumber - one-based number of the line

 @param index - zero-based index of the character that opens it

 @param state - counts carried from line to line, updated in place

 @returns That the count passed its bound at this place, or that it stays within

 @example
 ```ts
 const reading = opened({ counted: 'brackets', lineNumber: 1, index: 0, state, },);
 ```
 */
function opened(
  {
    counted,
    lineNumber,
    index,
    state,
  }: {
    readonly counted: Counted;
    readonly lineNumber: number;
    readonly index: number;
    readonly state: ScanState;
  },
): NestingReading {
  state[counted]++;
  if (state[counted] <= COUNTED_BOUND[counted])
    return { kind: 'within', };
  return {
    kind: 'beyond',
    measure: COUNTED_MEASURE[counted],
    bound: COUNTED_BOUND[counted],
    line: lineNumber,
    column: index + 1,
  };
}

/**
 Reads the angle brackets of a line against the open tags, which only the
 strict grammar nests over.

 @param line - one line without its newline

 @param lineGoingOn - same line with `LINE_GOES_ON` after it, which a tag's
 name is read in

 @param index - index of the character, an angle bracket

 @param lineNumber - one-based number of the line

 @param state - counts carried from line to line, updated in place

 @returns That a tag opened past the bound at this place, or that it stays within

 @example
 ```ts
 const reading = readAngle({ line: '<div>', lineGoingOn: `<div>${LINE_GOES_ON}`, index: 0, lineNumber: 1, state, },);
 ```
 */
function readAngle(
  {
    line,
    lineGoingOn,
    index,
    lineNumber,
    state,
  }: {
    readonly line: string;
    readonly lineGoingOn: string;
    readonly index: number;
    readonly lineNumber: number;
    readonly state: ScanState;
  },
): NestingReading {
  if (line[index] === '<') {
    /**
     The tag the strict grammar reads at this bracket, none where it reads the
     bracket as text or refuses the name after it.
     */
    const [reading,] = readTagName({
      text: lineGoingOn,
      at: index,
    },);
    if (reading === undefined)
      return { kind: 'within', };
    /**
     Name the tag opens with, lowercased as elements are compared.
     */
    const name = reading.name
      .toLowerCase();
    if (reading.closes)
      state.pendingTag = 'closing';
    else
      state.pendingTag = VOID_ELEMENTS.has(name,) ? 'none' : 'opening';
    return { kind: 'within', };
  }
  /**
   Which kind of tag this angle bracket ends, if any is begun.
   */
  const { pendingTag, } = state;
  state.pendingTag = 'none';
  if (pendingTag === 'closing')
    state.tags = Math.max(
      0,
      state.tags - 1,
    );
  if ((pendingTag === 'opening') && (line[index - 1] !== '/'))
    return opened({
      counted: 'tags',
      lineNumber,
      index,
      state,
    },);
  return { kind: 'within', };
}

/**
 The quotes a JavaScript string opens with that this measure skips, the template literal left out since it goes on to
 the next line.
 */
const STRING_QUOTES = '"\'';

/**
 What `stringEnd` answers when the quote never closes on its line.
 */
const NO_STRING_END = -1;

/**
 Where a quoted string inside an expression ends on its line.

 @param line - one line without its newline

 @param quote - quote that opened the string

 @param from - index of the first character after the opening quote

 @returns Index of the closing quote, `NO_STRING_END` when the line holds none: the quote is then no string, as a
 JavaScript string does not go on to the next line

 @example
 ```ts
 const end = stringEnd({ line: '{a "{"}', quote: '"', from: 4, },);
 // => 5
 ```
 */
function stringEnd(
  {
    line,
    quote,
    from,
  }: {
    readonly line: string;
    readonly quote: string;
    readonly from: number;
  },
): number {
  for (let index = from; index < line.length; index++) {
    if (line[index] === '\\') {
      // A backslash takes the character after it into the string.
      index++;
      continue;
    }
    if (line[index] === quote)
      return index;
  }
  return NO_STRING_END;
}

/**
 Reads one character of a line against the counts it opens and closes.

 @param line - one line without its newline

 @param lineGoingOn - same line with `LINE_GOES_ON` after it, which a tag's
 name is read in

 @param index - index of the character

 @param lineNumber - one-based number of the line

 @param grammar - grammar that will read the body

 @param state - counts carried from line to line, updated in place

 @param skipDelimiters - whether this line's emphasis delimiters nest nothing

 @returns That the character passed a bound at this place, or that it stays within

 @example
 ```ts
 const reading = readCharacter({ line: '[a]', lineGoingOn: `[a]${LINE_GOES_ON}`, index: 0, lineNumber: 1, grammar: 'markdown', state, skipDelimiters: false, },);
 ```
 */
function readCharacter(
  {
    line,
    lineGoingOn,
    index,
    lineNumber,
    grammar,
    state,
    skipDelimiters,
  }: {
    readonly line: string;
    readonly lineGoingOn: string;
    readonly index: number;
    readonly lineNumber: number;
    readonly grammar: NestingGrammar;
    readonly state: ScanState;
    readonly skipDelimiters: boolean;
  },
): NestingReading {
  /**
   Character read here.
   */
  const character = line.charAt(index,);
  /**
   Count this character opens, if any, under the grammar.
   */
  const opens = opensOf({
    character,
    grammar,
  },);
  if (opens !== 'none') {
    if ((opens === 'delimiters') && skipDelimiters)
      return { kind: 'within', };
    return opened({
      counted: opens,
      lineNumber,
      index,
      state,
    },);
  }
  /**
   Count this character closes, if any, under the grammar.
   */
  const closes = closesOf({
    character,
    grammar,
  },);
  if (closes !== 'none') {
    state[closes] = Math.max(
      0,
      state[closes] - 1,
    );
    return { kind: 'within', };
  }
  if ((grammar === 'mdx') && ((character === '<') || (character === '>')))
    return readAngle({
      line,
      lineGoingOn,
      index,
      lineNumber,
      state,
    },);
  return { kind: 'within', };
}

/**
 Reads the inline characters of one line against the bound.

 @param line - one line without its newline

 @param start - index where the inline text begins

 @param lineNumber - one-based number of the line

 @param grammar - grammar that will read the body

 @param state - counts carried from line to line, updated in place

 @param skipDelimiters - whether this line's emphasis delimiters nest nothing

 @returns First excess on the line, or that it stays within

 @example
 ```ts
 const reading = scanInline({ line, start: 0, lineNumber: 1, grammar: 'markdown', state, skipDelimiters: false, },);
 ```
 */
export function scanInline(
  {
    line,
    start,
    lineNumber,
    grammar,
    state,
    skipDelimiters,
  }: {
    readonly line: string;
    readonly start: number;
    readonly lineNumber: number;
    readonly grammar: NestingGrammar;
    readonly state: ScanState;
    readonly skipDelimiters: boolean;
  },
): NestingReading {
  /**
   The line as a tag's name is read in, joined once so each bracket reads it
   without building it again.
   */
  const lineGoingOn = `${line}${LINE_GOES_ON}`;
  for (let index = start; index < line.length; index++) {
    if (line[index] === '\\') {
      // An escaped character opens and closes nothing.
      index++;
      continue;
    }
    /**
     Whether the character is a quote that may open a string.
     */
    const isQuote = isOneOf({
      character: line.charAt(index,),
      set: STRING_QUOTES,
    },);
    /**
     Whether the character stands inside an expression the strict grammar has open.
     */
    const inExpression = (grammar === 'mdx') && (state.braces > 0);
    if (inExpression && isQuote) {
      // A brace in a string of an open expression opens and closes nothing.
      /**
       Where the string that opens here ends.
       */
      const end = stringEnd({
        line,
        quote: line.charAt(index,),
        from: index + 1,
      },);
      if (end !== NO_STRING_END) {
        index = end;
        continue;
      }
    }
    /**
     What this character did to the counts.
     */
    const reading = readCharacter({
      line,
      lineGoingOn,
      index,
      lineNumber,
      grammar,
      state,
      skipDelimiters,
    },);
    if (reading.kind === 'beyond')
      return reading;
  }
  return { kind: 'within', };
}

//endregion Nesting inline count
