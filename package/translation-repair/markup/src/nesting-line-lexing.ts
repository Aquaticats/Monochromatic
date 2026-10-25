import { NESTING_BOUND, } from './nesting-vocabulary.ts';

//region Nesting line lexing
// READS THE PARTS OF ONE LINE the nesting bound counts: the spaces opening it, the
// container markers after them, a fence it may open, and whether it is a rule.
// Every function here walks the line once, by index, and none recurses.

/**
 Columns a tab counts for where indentation is measured.
 */
const TAB_COLUMNS = 4;

/**
 Fewest rule characters in a thematic break.
 */
const RULE_MINIMUM = 3;

/**
 ASCII digits.
 */
const DIGITS = '0123456789';

/**
 Characters that open a list item when a blank follows.
 */
const BULLETS = '-*+';

/**
 What the container markers opening a line came to.
 */
type ContainerPrefix = {
  /**
   Index after the last marker.
   */
  end: number;

  /**
   How many markers the line opens with.
   */
  count: number;

  /**
   One-based column of the first marker past the bound, zero when none is.
   */
  beyondColumn: number;
};

/**
 What a line's leading spaces and tabs came to.
 */
type Indentation = {
  /**
   Columns of leading spaces and tabs, a tab counting four.
   */
  readonly columns: number;

  /**
   Index of the first character that is neither.
   */
  readonly end: number;
};

/**
 Whether a character is one of a set.

 @param character - one UTF-16 unit, the empty string past a line's end

 @param set - characters it may be

 @returns True when it is exactly one character and that character is in the set

 @example
 ```ts
 isOneOf({ character: '7', set: '0123456789', },);
 // => true
 ```
 */
export function isOneOf(
  {
    character,
    set,
  }: {
    readonly character: string;
    readonly set: string;
  },
): boolean {
  return (character.length === 1) && set.includes(character,);
}

/**
 Whether a character is a space or a tab.

 @param character - one UTF-16 unit, the empty string past a line's end

 @returns True for a space or a tab

 @example
 ```ts
 isBlank({ character: ' ', },);
 // => true
 ```
 */
function isBlank({ character, }: { readonly character: string; },): boolean {
  return (character === ' ') || (character === '\t');
}

/**
 Index of the first character from a position that is not a space or a tab.

 @param line - one line, without its newline

 @param from - index to start at

 @returns Index of that character, the line's length when none follows

 @example
 ```ts
 blanksEnd({ line: '  cat', from: 0, },);
 // => 2
 ```
 */
export function blanksEnd(
  {
    line,
    from,
  }: {
    readonly line: string;
    readonly from: number;
  },
): number {
  for (let index = from; index < line.length; index++) {
    if (!isBlank({ character: line.charAt(index,), },))
      return index;
  }
  return line.length;
}

/**
 Index of the first character from a position that is not in a set.

 @param line - one line, without its newline

 @param from - index to start at

 @param set - characters the run is made of

 @returns Index after the run, the line's length when it runs to the end

 @example
 ```ts
 characterRunEnd({ line: '123.', from: 0, set: '0123456789', },);
 // => 3
 ```
 */
export function characterRunEnd(
  {
    line,
    from,
    set,
  }: {
    readonly line: string;
    readonly from: number;
    readonly set: string;
  },
): number {
  for (let index = from; index < line.length; index++) {
    if (!isOneOf({
      character: line.charAt(index,),
      set,
    },))
      return index;
  }
  return line.length;
}

/**
 Reads a line's leading spaces and tabs.

 @param line - one line, without its newline

 @returns Their width in columns, a tab counting four, and where they end

 @example
 ```ts
 indentationOf({ line: '    cat', },);
 // => { columns: 4, end: 4, }
 ```
 */
export function indentationOf({ line, }: { readonly line: string; },): Indentation {
  /**
   Index of the first character after the indentation.
   */
  const end = blanksEnd({
    line,
    from: 0,
  },);
  return {
    columns: Array.from(line.slice(
      0,
      end,
    ),)
      .reduce(
      function widened(
        columns,
        character,
      ): number {
        return columns + ((character === '\t') ? TAB_COLUMNS : 1);
      },
      0,
      ),
    end,
  };
}

/**
 Where an ordered list marker beginning at an index ends.

 @param line - one line, without its newline

 @param index - where the marker may begin, at a digit, the one place
 `markerEndOf` reads an ordered marker

 @returns Index after the digits, the delimiter and its blank, zero when no
 ordered marker begins there

 @throws {@link Error} when no digit stands at the index, which its one
 caller has already read

 @example
 ```ts
 orderedMarkerEnd({ line: '1. cat', index: 0, },);
 // => 2
 ```
 */
function orderedMarkerEnd(
  {
    line,
    index,
  }: {
    readonly line: string;
    readonly index: number;
  },
): number {
  /**
   Index after the digits.
   */
  const digitsEnd = characterRunEnd({
    line,
    from: index,
    set: DIGITS,
  },);
  if (digitsEnd === index)
    throw new Error('unreachable: an ordered list marker was read where no digit stands, though markerEndOf reads one only at a digit',);
  if (!isOneOf({
    character: line[digitsEnd] ?? '',
    set: '.)',
  },))
    return 0;
  /**
   Index of the character after the delimiter.
   */
  const afterDelimiter = digitsEnd + 1;
  if (afterDelimiter === line.length)
    return afterDelimiter;
  return isBlank({ character: line.charAt(afterDelimiter,), },) ? afterDelimiter : 0;
}

/**
 Where a footnote definition's label beginning at an index ends.

 @param line - one line, without its newline

 @param index - where the label may begin

 @returns Index after the label and its colon, zero when none begins there

 @example
 ```ts
 labelMarkerEnd({ line: '[^1]: cat', index: 0, },);
 // => 5
 ```
 */
function labelMarkerEnd(
  {
    line,
    index,
  }: {
    readonly line: string;
    readonly index: number;
  },
): number {
  if ((line[index] !== '[') || (line[index + 1] !== '^'))
    return 0;
  for (let cursor = index + 2; cursor < line.length; cursor++) {
    /**
     Character read inside the label.
     */
    const character = line.charAt(cursor,);
    if (character === ']')
      return (line[cursor + 1] === ':') ? (cursor + 2) : 0;
    if (isBlank({ character, },))
      return 0;
  }
  return 0;
}

/**
 Where one container marker beginning at an index ends.

 @param line - one line, without its newline

 @param index - where a marker may begin

 @returns Index after a quotation mark, a list marker or a footnote
 definition label beginning there, zero when none does

 @example
 ```ts
 markerEndOf({ line: '- cat', index: 0, },);
 // => 1
 ```
 */
function markerEndOf(
  {
    line,
    index,
  }: {
    readonly line: string;
    readonly index: number;
  },
): number {
  /**
   Character the marker would begin with.
   */
  const character = line[index] ?? '';
  if (character === '>')
    return index + 1;
  if (isOneOf({
    character,
    set: BULLETS,
  },)) {
    /**
     Whether the bullet ends the line or a blank follows it.
     */
    const endsItem = ((index + 1) >= line.length) || isBlank({ character: line.charAt(index + 1,), },);
    return endsItem ? (index + 1) : 0;
  }
  if (isOneOf({
    character,
    set: DIGITS,
  },))
    return orderedMarkerEnd({
      line,
      index,
    },);
  return labelMarkerEnd({
    line,
    index,
  },);
}

/**
 Reads the container markers opening a line.

 @param line - one line, without its newline

 @param start - index of the first character after the line's indentation

 @returns Where the markers end, how many there are, and where the first one
 past the bound sits

 @example
 ```ts
 const prefix = containerPrefixOf({ line: '> > cat', start: 0, },);
 // => { end: 3, count: 2, beyondColumn: 0, }
 ```
 */
export function containerPrefixOf(
  {
    line,
    start,
  }: {
    readonly line: string;
    readonly start: number;
  },
): ContainerPrefix {
  /**
   What the markers read so far came to, updated as each is read.
   */
  const found: ContainerPrefix = {
    end: start,
    count: 0,
    beyondColumn: 0,
  };
  for (;;) {
    /**
     Index where the next marker would begin.
     */
    const index = blanksEnd({
      line,
      from: found.end,
    },);
    /**
     Index after the marker found there, zero when none begins there.
     */
    const after = markerEndOf({
      line,
      index,
    },);
    if (after === 0)
      break;
    found.count++;
    if (found.count === (NESTING_BOUND + 1))
      found.beyondColumn = index + 1;
    found.end = after;
  }
  return found;
}

/**
 Whether a line is a thematic break, so its delimiters nest nothing.

 EXACTLY THE GRAMMAR'S BREAK: three or more of one of `*`, `_` and `-`, with
 only spaces and tabs between them. A line mixing those characters is a
 paragraph of emphasis delimiters and costs the parser as one (`*-` repeated
 24,000 times took 3.7 to 6.6 s to parse in four runs on the built package),
 so it is no rule here.

 @param line - one line without its newline

 @returns True when the line is a thematic break

 @example
 ```ts
 isRuleLine({ line: '* * *', },);
 // => true
 isRuleLine({ line: '*-*-*-', },);
 // => false
 ```
 */
export function isRuleLine({ line, }: { readonly line: string; },): boolean {
  /**
   What the line holds besides spaces and tabs, in order.
   */
  const marks = Array.from(line,)
    .filter(function isMark(character,): boolean {
      return !isBlank({ character, },);
    },);
  /**
   The character the break would be made of.
   */
  const [first = '',] = marks;
  return (marks.length >= RULE_MINIMUM)
    && isOneOf({
      character: first,
      set: '*_-',
    },)
    && marks.every(function isFirst(character,): boolean {
      return character === first;
    },);
}

/**
 The lines of a body as the parser reads them: a line feed, a carriage return and a carriage return with a line feed after
 it each end a line.

 @param body - text about to be parsed

 @returns Lines without their endings, one more than the endings the body holds

 @example
 ```ts
 const lines = linesOf({ body: 'a\rb\r\nc', },); // ['a', 'b', 'c']
 ```
 */
export function linesOf({ body, }: { readonly body: string; },): readonly string[] {
  /**
   Lines found so far.
   */
  const lines: string[] = [];
  /**
   Index where the line being read starts.
   */
  const line = { start: 0, };
  for (let at = 0; at < body.length; at += 1) {
    /**
     The character at the index.
     */
    const character = body.charAt(at,);
    if ((character !== '\n') && (character !== '\r'))
      continue;
    lines.push(body.slice(
      line.start,
      at,
    ),);
    if ((character === '\r') && (body.charAt(at + 1,) === '\n'))
      at += 1;
    line.start = at + 1;
  }
  lines.push(body.slice(line.start,),);
  return lines;
}

/**
 Whether a line is a link reference definition, whose destination and title read a brace as a character.

 A footnote definition is none: the text after its colon is read as inline text, braces and all.

 @param line - the line's text after its container markers and indentation

 @returns Whether the line opens with a label and a colon

 @example
 ```ts
 const definition = isDefinitionLine({ line: '[r]: u{b', },); // true
 ```
 */
export function isDefinitionLine({ line, }: { readonly line: string; },): boolean {
  if ((!line.startsWith('[',)) || line.startsWith('[^',))
    return false;
  for (let index = 1; index < line.length; index++) {
    /**
     Character read.
     */
    const character = line.charAt(index,);
    if (character === '\\') {
      // A backslash takes the character after it into the label.
      index++;
      continue;
    }
    // A label holds no bracket that is not escaped.
    if (character === '[')
      return false;
    if (character === ']')
      return line.charAt(index + 1,) === ':';
  }
  return false;
}

//endregion Nesting line lexing
