/**
 SVG parsing utilities for extracting glyph cells and path data from
 the Aquaticat master glyph strip SVG exported from Figma.
 */

//region Types: data extracted from the master SVG

/**
 Raw path data extracted from a single SVG `<path>` element.
 */
export type CellPath = {
  /**
   SVG path `d` attribute string.
   */
  readonly d: string;
  /**
   Whether this path uses stroke rather than fill.
   */
  readonly isStroked: boolean;
  /**
   Stroke width when `isStroked` is true, otherwise 0.
   */
  readonly strokeWidth: number;
};

/**
 One glyph cell from the horizontal strip.
 */
export type Cell = {
  /**
   Absolute X offset of this cell in the master SVG.
   */
  readonly xOffset: number;
  /**
   All path elements belonging to this cell.
   */
  readonly paths: readonly CellPath[];
};

/**
 Parser-owned cell accumulator before immutable publication.
 */
type MutableCell = {
  /**
   Absolute X offset of this cell in the master SVG.
   */
  xOffset: number;
  /**
   Path elements accumulated until next cell begins.
   */
  paths: CellPath[];
};

//endregion Types

//region Whitespace

/**
 Every character JavaScript's `\s` class matches: the five ASCII whitespace controls, the space,
 the line terminators, and the space separators and byte-order mark beyond ASCII.
 
 Written as one string rather than as code ranges because the class is not contiguous.
 */
const WHITESPACE_CHARACTERS = '\t\n\v\f\r \u00A0\u1680'
  + '\u2000\u2001\u2002\u2003\u2004\u2005\u2006\u2007\u2008\u2009\u200A'
  + '\u2028\u2029\u202F\u205F\u3000\uFEFF';

/**
 Reports one index as absent, which is what {@link String.prototype.indexOf} answers.
 
 The name exists because the linter reads a bare `-1` inside a comparison as a nested binary
 expression, and because every site here is asking the same question: where did this literal
 first appear, or is it not in the text at all.
 */
const NOT_FOUND = -1;

/**
 Checks whether one character is whitespace, counting the empty string as not.
 
 The empty string is what {@link String.charAt} reports where an index falls outside the text, and
 treating it as non-whitespace is what ends a run at either edge without a bounds check.
 
 @param character - one character, or the empty string where an index falls outside the text
 
 @returns whether the character is one JavaScript's `\s` class matches
 
 @example
 ```ts
 isWhitespaceCharacter('\n'); // true
 isWhitespaceCharacter(''); // false
 ```
 */
function isWhitespaceCharacter(character: string,): boolean {
  return (character !== '')
    && WHITESPACE_CHARACTERS.includes(character,);
}

/**
 Finds where the whitespace run starting at one index ends.
 
 @param text - text to scan
 
 @param start - index the run begins at, whose character the caller already found to be whitespace
 
 @returns index one past the last whitespace character of the run
 
 @example
 ```ts
 whitespaceRunEnd({ text: '<rect  width', start: 5 }); // 7
 ```
 */
function whitespaceRunEnd({
  text,
  start,
}: {
  readonly text: string;
  readonly start: number;
},): number {
  /**
   Index the scan has reached.
   */
  let cursor = start;
  while (isWhitespaceCharacter(text.charAt(cursor,),))
    cursor += 1;
  return cursor;
}

//endregion Whitespace

//region Numbers

/**
 Checks whether one character is an ASCII decimal digit, the only digit class the strip uses.
 
 @param character - one character, or the empty string where an index falls outside the text
 
 @returns whether the character sits in `0` through `9`
 
 @example
 ```ts
 isAsciiDigit('7'); // true
 ```
 */
function isAsciiDigit(character: string,): boolean {
  return (character >= '0')
    && (character <= '9');
}

/**
 Finds where the decimal run starting at one index ends.
 
 The shape is `\d+(?:\.\d+)?`: an integer run, then a fraction only when a point is followed by
 at least one digit. A point with no digit before or after it is left for the caller, which is
 what keeps `translate(.5)` and `translate(1.)` from reading as canonical transforms.
 
 @param text - text to scan
 
 @param start - index the integer run begins at
 
 @returns index one past the last digit of the run
 
 @example
 ```ts
 decimalRunEnd({ text: '645.5)', start: 0 }); // 5
 decimalRunEnd({ text: '645.)', start: 0 }); // 3
 decimalRunEnd({ text: '.5)', start: 0 }); // 0
 ```
 */
function decimalRunEnd({
  text,
  start,
}: {
  readonly text: string;
  readonly start: number;
},): number {
  /**
   Index the scan has reached.
   */
  let cursor = start;
  while (isAsciiDigit(text.charAt(cursor,),))
    cursor += 1;
  // `\d+` needs one integer digit before any point belongs to the number.
  if (cursor === start)
    return start;
  if (text.charAt(cursor,) !== '.')
    return cursor;
  /**
   Index one past the last fraction digit, or one past the point when the fraction is empty.
   */
  let fractionEnd = cursor + 1;
  while (isAsciiDigit(text.charAt(fractionEnd,),))
    fractionEnd += 1;
  if (fractionEnd === (cursor + 1))
    return cursor;
  return fractionEnd;
}

//endregion Numbers

/**
 Sentinel returned by {@link attr} when the named attribute is genuinely
 absent. A `unique symbol` because an empty string is a valid attribute
 value, so no falsy default can stand in for "not found"; callers narrow
 identity with `=== ATTRIBUTE_ABSENT`.
 */
const ATTRIBUTE_ABSENT: unique symbol = Symbol('aquaticat/svg-attribute-not-present-in-element',);

/**
 Matches one attribute of a raw attribute string: its name, the quote that
 opens its value, and that value. Matching a whole attribute at a time keeps a
 `name=<quote>` sequence inside another attribute's value, or a name that
 merely ends with the sought one, from being read as the attribute itself.
 */
/* oxlint-disable no-restricted-syntax/no-regex -- SVG attribute tokenizer: one name of non-space, non-`=` characters, then one quoted value. The lazy value stops at the next matching quote, so every character is visited a bounded number of times; the input is one element's attribute string from a bounded master strip. */
const ATTRIBUTE_REGEX = /(?<name>[^\s=]+)\s*=\s*(?<quote>['"])(?<value>.*?)\k<quote>/gsu;
/* oxlint-enable no-restricted-syntax/no-regex */

/**
 Extracts an XML attribute value by name from a raw attribute string.
 Accepts both double- and single-quoted values: the Figma-exported master
 strip SVG this parser reads uses single quotes throughout.
 
 @param attrs - raw XML attribute string to search
 
 @param name - attribute name to extract
 
 @returns attribute value, or {@link ATTRIBUTE_ABSENT} if not found
 
 @example
 ```ts
 attr({ attrs: 'fill="red" stroke="black"', name: 'fill' }); // 'red'
 attr({ attrs: "fill='red' stroke='black'", name: 'fill' }); // 'red'
 ```
 */
function attr({
  attrs,
  name,
}: {
  readonly attrs: string;
  readonly name: string;
},): string | typeof ATTRIBUTE_ABSENT {
  for (const match of attrs.matchAll(ATTRIBUTE_REGEX,)) {
    /**
     Name and value of the attribute the current match spans, if it captured both.
     */
    const { groups, } = match;
    if (groups?.name === name)
      return groups.value ?? '';
  }
  return ATTRIBUTE_ABSENT;
}

//region Element scanning

/**
 One self-closing element the master strip draws.
 */
type SelfClosingElement = {
  /**
   Element name, one of the two names the master strip uses.
   */
  readonly tag: 'rect' | 'path';
  /**
   Raw attribute text between the element name and the closing `/>`.
   */
  readonly attrs: string;
};

/**
 Element names the master strip draws, in the order the pattern this scan replaces listed them.
 */
const RECT_ELEMENT_NAME = 'rect';

/**
 Name of the other element the master strip draws; both names are the same length.
 */
const PATH_ELEMENT_NAME = 'path';

/**
 Reports the element name one opening holds, or the empty string when it holds neither name.
 
 The empty string stands in for absence here because no element name is empty, so the two cases
 cannot be confused; that is the one thing an attribute value could not promise at
 {@link ATTRIBUTE_ABSENT}.
 
 @param svgContent - full SVG file content
 
 @param start - index one past the opening `<`, where the name belongs
 
 @returns element name at that index, or the empty string when no name starts there
 
 @example
 ```ts
 elementNameAt({ svgContent: '<rect width="1" />', start: 1 }); // 'rect'
 ```
 */
function elementNameAt({
  svgContent,
  start,
}: {
  readonly svgContent: string;
  readonly start: number;
},): string {
  if (svgContent.startsWith(
    RECT_ELEMENT_NAME,
    start,
  ))
    return RECT_ELEMENT_NAME;
  if (svgContent.startsWith(
    PATH_ELEMENT_NAME,
    start,
  ))
    return PATH_ELEMENT_NAME;
  return '';
}

/**
 Reads every self-closing `<rect .../>` and `<path .../>` element in strip order.
 
 One cursor replaces the global pattern this scan used to run through `matchAll`. An element needs
 an opening `<`, a name, one whitespace character, and a `/>` terminator with no `>` between the
 name and it, so the scan resumes at the next `<` unless all of those hold. The first `>` at or
 after the name decides each attempt and that search only moves forward, so an export whose last
 opener never terminates costs one pass of the document rather than a rescan per opener.
 
 @param svgContent - full SVG file content
 
 @returns self-closing elements in the order they appear
 
 @example
 ```ts
 selfClosingElements('<rect width="1" /><path d="M0 0" />').length; // 2
 ```
 */
function selfClosingElements(svgContent: string,): SelfClosingElement[] {
  /**
   Elements found so far, in the order they appear.
   */
  const elements: SelfClosingElement[] = [];
  /**
   Index of the next `<` the scan has not ruled out.
   */
  let cursor = 0;
  /**
   Index of the first `>` at or after the element name under examination, or `NOT_FOUND` when the
   document holds no further `>`.
   */
  let closingAngle = svgContent.indexOf('>',);
  while (cursor < svgContent.length) {
    /**
     Index of the opening `<` this attempt reads from.
     */
    const opening = svgContent.indexOf(
      '<',
      cursor,
    );
    if (opening === NOT_FOUND)
      break;
    /**
     Index one past the opening `<`, where the element name belongs.
     */
    const nameStart = opening + 1;
    /**
     Element name at the opening, or the empty string when the `<` starts something else.
     */
    const tag = elementNameAt({
      svgContent,
      start: nameStart,
    },);
    if (tag === '') {
      cursor = nameStart;
      continue;
    }
    /**
     Index one past the name, where the separator whitespace belongs.
     */
    const attrsStart = nameStart + tag.length;
    if (!isWhitespaceCharacter(svgContent.charAt(attrsStart,),)) {
      cursor = nameStart;
      continue;
    }
    if (closingAngle < attrsStart) {
      closingAngle = svgContent.indexOf(
        '>',
        attrsStart,
      );
    }
    // Without a `>` ahead no later opener can terminate either, so the scan is finished.
    if (closingAngle === NOT_FOUND)
      break;
    if (svgContent.charAt(closingAngle - 1,) !== '/') {
      cursor = nameStart;
      continue;
    }
    elements.push({
      tag: tag === RECT_ELEMENT_NAME ? 'rect' : 'path',
      attrs: svgContent.slice(
        whitespaceRunEnd({
          text: svgContent,
          start: attrsStart,
        },),
        closingAngle - 1,
      ),
    },);
    cursor = closingAngle + 1;
  }
  return elements;
}

//endregion Element scanning

//region Transforms

/**
 Literal that opens a canonical `translate(N)` transform.
 */
const TRANSLATE_OPENING = 'translate(';

/**
 Reads the X offset of the first canonical `translate(N)` transform in one attribute value.
 
 The value is searched rather than tested from its start because a transform sequence such as
 `rotate(45) translate(645)` still names the cell offset. A candidate that opens `translate(`
 without a decimal argument and a closing parenthesis is skipped, so a malformed transform
 earlier in the value does not hide a canonical one later.
 
 @param transform - raw `transform` attribute value
 
 @returns numeric X offset, or 0 when no canonical transform is present
 
 @example
 ```ts
 translateOffset('translate(645)'); // 645
 translateOffset('rotate(45) translate(12.5)'); // 12.5
 ```
 */
function translateOffset(transform: string,): number {
  for (let cursor = 0; cursor < transform.length;) {
    /**
     Index of the next candidate opener.
     */
    const opening = transform.indexOf(
      TRANSLATE_OPENING,
      cursor,
    );
    if (opening === NOT_FOUND)
      return 0;
    /**
     Index the decimal argument begins at.
     */
    const argumentStart = opening + TRANSLATE_OPENING.length;
    /**
     Index one past the decimal argument.
     */
    const argumentEnd = decimalRunEnd({
      text: transform,
      start: argumentStart,
    },);
    if ((argumentEnd > argumentStart) && (transform.charAt(argumentEnd,) === ')'))
      return Number(transform.slice(
        argumentStart,
        argumentEnd,
      ),);
    cursor = opening + 1;
  }
  return 0;
}

//endregion Transforms

/**
 Parse the master glyph strip SVG and return one {@link Cell} per glyph position.
 Cells are delimited by `<rect>` elements; all `<path>` elements between two
 consecutive rects belong to the preceding cell. Missing attributes read
 back as {@link ATTRIBUTE_ABSENT} and fall back to defaults or skip the
 path entirely.
 
 @param svgContent - full SVG file content
 
 @returns array of cells in strip order (left to right)
 
 @example
 ```ts
 const cells = parseSvg(svgContent);
 cells[0].xOffset; // X position of first glyph cell
 ```
 */
export function parseSvg(svgContent: string,): Cell[] {
  /**
   Accumulator of parsed cells, filled in strip order as `<rect>` tags are encountered.
   */
  const cells: MutableCell[] = [];

  for (const element of selfClosingElements(svgContent,)) {
    /**
     Element name (`rect` or `path`) of the element under examination.
     */
    const { tag, } = element;
    /**
     Raw attribute substring between the element name and the self-closing `/>`.
     */
    const { attrs, } = element;

    if (tag === 'rect') {
      /**
       Raw `transform` attribute value, expected to be `translate(<x>)` for cell rects.
       */
      const transform = attr({
        attrs,
        name: 'transform',
      },);
      /**
       Parsed X offset of this cell rect; falls back to 0 when no translate is present.
       */
      const xOffset = transform === ATTRIBUTE_ABSENT
        ? 0
        : translateOffset(transform,);
      cells.push({
        xOffset,
        paths: [],
      },);
      continue;
    }

    // tag === "path"
    /**
     Raw SVG path `d` attribute string for the current `<path>` element.
     */
    const d = attr({
      attrs,
      name: 'd',
    },);
    if (d === ATTRIBUTE_ABSENT)
      continue;

    /**
     Raw `stroke` attribute value used to discriminate stroked paths from filled ones.
     */
    const strokeAttr = attr({
      attrs,
      name: 'stroke',
    },);
    /**
     True when the path has a stroke but no fill, indicating it must be expanded into an outline.
     */
    const isStroked = (strokeAttr !== ATTRIBUTE_ABSENT) && (attr({
      attrs,
      name: 'fill',
    },)
      === ATTRIBUTE_ABSENT);
    /**
     Raw `stroke-width` attribute string; parsed only when the path is actually stroked.
     */
    const strokeWidthStr = attr({
      attrs,
      name: 'stroke-width',
    },);
    /**
     Numeric stroke width in SVG units; 0 for filled paths so downstream code skips expansion.
     */
    const strokeWidth = isStroked && (strokeWidthStr !== ATTRIBUTE_ABSENT)
      ? Number(strokeWidthStr,)
      : 0;

    /**
     Most recently pushed cell, which owns every `<path>` until the next `<rect>` appears.
     */
    const currentCell = cells.at(-1,);
    if (currentCell === undefined)
      throw new Error(`path data before the first cell rect: ${d}`,);
    currentCell.paths
      .push({
        d,
        isStroked,
        strokeWidth,
      },);
  }

  return cells;
}

//region SVG path tokenization

/**
 Parsed absolute SVG path command (M/L/H/V/Z only).
 */
export type SVGPathCommand =
  | {
    readonly type: 'M';
    readonly x: number;
    readonly y: number;
  }
    | {
      readonly type: 'L';
      readonly x: number;
      readonly y: number;
    }
    | {
      readonly type: 'H';
      readonly x: number;
    }
    | {
      readonly type: 'V';
      readonly y: number;
    }
    | { readonly type: 'Z'; };

/**
 Tokenize an SVG path `d` attribute into absolute commands.
 Only handles M, L, H, V, Z (the commands used in the Aquaticat glyph SVG).
 
 @param d - SVG path data string
 
 @returns ordered list of path commands
 
 @example
 ```ts
 const commands = parseSvgPathD('M0 0 L10 10 Z');
 // [{ type: 'M', x: 0, y: 0 }, { type: 'L', x: 10, y: 10 }, { type: 'Z' }]
 ```
 */
export function parseSvgPathD(d: string,): SVGPathCommand[] {
  /**
   Accumulator of parsed commands, returned to the caller in path order.
   */
  const commands: SVGPathCommand[] = [];
  /* oxlint-disable no-restricted-syntax/no-regex -- SVG path tokenizer; the alternation either matches one command letter or one signed decimal in linear time, and the input is a bounded `d=` attribute value (Aquaticat master strip). No backtracking risk on either branch. */
  /**
   Matches either a command letter (M/L/H/V/Z) or a signed decimal number.
   */
  const tokenRegex = /(?<command>[MLHVZ])|(?<number>-?\d+(?:\.\d+)?)/gu;
  /* oxlint-enable no-restricted-syntax/no-regex */

  /**
   Last command letter seen, which determines how subsequent number tokens are consumed.
   
   Declared as `let` because the token loop reassigns it whenever a new command
   letter is matched; coordinate tokens between letters apply to whatever letter
   was set most recently.
   */
  let currentCmd = '';
  /**
   One ordered token stream over the path data. A paired Y coordinate consumes
   the next token from this same iterator, so iteration state lives here and
   never in the regex's `lastIndex`.
   */
  const tokens = d.matchAll(tokenRegex,);

  for (const tok of tokens) {
    /**
     Named capture groups from the current token: `command` for a letter, `number` for a value.
     */
    const { groups, } = tok;
    /**
     Captured command letter (`command` group) from the current token, undefined when the token is a number.
     */
    const commandLetter = groups?.command;
    if (commandLetter !== undefined) {
      currentCmd = commandLetter;
      if (currentCmd === 'Z')
        commands.push({ type: 'Z', },);
      continue;
    }

    /**
     Numeric value of the current number token (`number` group of `tokenRegex`).
     */
    const num = Number(groups?.number,);

    if ((currentCmd === 'M') || (currentCmd === 'L')) {
      /**
       Y coordinate token paired with the just-consumed X for M/L commands.
       */
      const yTok = tokens.next();
      if (yTok.done === true)
        throw new Error(
          `path data ends where a Y coordinate belongs, after ${num}: ${d}`,
        );
      /**
       Named capture groups of the paired Y token; `number` holds the coordinate value.
       */
      const { groups: yGroups, } = yTok.value;
      /**
       Numeric text of the paired Y coordinate, absent when the token is a command letter.
       */
      const yNumber = yGroups?.number;
      if (yNumber === undefined) {
        throw new Error(
          `expected a Y coordinate after ${num} in path data: ${d}`,
        );
      }
      commands.push({
        type: currentCmd,
        x: num,
        y: Number(yNumber,),
      },);
    }
    else if (currentCmd === 'H') {
      commands.push({
        type: 'H',
        x: num,
      },);
    }
    else if (currentCmd === 'V') {
      commands.push({
        type: 'V',
        y: num,
      },);
    }
  }

  return commands;
}

//endregion SVG path tokenization
