import { textsInCodePointOrder, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  BARE_URL,
  BRACE_EXPRESSION,
  CJK_IDEOGRAPH,
  FENCE_LINE,
  FOOTNOTE_LINE,
  FRONT_KEY_LINE,
  HEADING_LINE,
  HTML_LINE,
  INLINE_TAG,
  JSX_KEYWORD_LINE,
  JSX_MARK_LINE,
  JSX_OPEN_LINE,
  LINK_DESTINATION,
  LIST_LINE,
  QUOTE_LINE,
  TEMPLATE_EXPRESSION,
} from './repeat-census-patterns.ts';

//region Repeat census lines
// The design's census of repeated source wording, ported line by line: each
// line is classified by its Markdown role, sanitised, and compared by its
// non-space characters. The command (`repeat-census.ts`) runs it beside the
// package's own detection (`repeat-census-package.ts`), which reads the parse,
// and prints both in one summary. Its thresholds are the design's main setting,
// with its loose and tight settings counted for the sensitivity table.



/**
 Roles of the lines that take part in the comparison; fences, code, blanks,
 JSX and the like are left out.
 */
export const USABLE_KINDS: ReadonlySet<string> = new Set([
  'prose',
  'heading',
  'list',
  'quote',
  'frontmatter',
  'html',
  'footnote',
],);

/**
 Opening quotation marks the census counts.
 */
const OPENERS = '「“『';

/**
 Closing quotation marks the census counts.
 */
const CLOSERS = '」”』';

/**
 Fewest characters of a line's wording below which a line is JSX rather than prose, as the census measured it.
 */
const JSX_CJK_FLOOR = 4;

/**
 Lines at the start of a page in which a front-matter key may stand, as the census measured it.
 */
const FRONT_MATTER_LINES = 12;

/**
 Fence markers that make one fenced block: an opening and a closing marker.
 */
const FENCE_PAIR = 2;

/**
 Thresholds of one refrain setting.
 */
export type RefrainSetting = {
  /**
   Shortest shared prefix.
   */
  readonly prefixMinimum: number;

  /**
   Shortest shared suffix.
   */
  readonly suffixMinimum: number;

  /**
   Longest varying slot.
   */
  readonly maximumSlot: number;

  /**
   Fewest characters a clause holds to be compared.
   */
  readonly minimumClause: number;
};

/**
 The census's three refrain settings: the main one the design recommends, and
 the loose and tight ones it measured for sensitivity.
 */
export const REFRAIN_SETTINGS: Readonly<Record<'main' | 'loose' | 'tight', RefrainSetting>> = {
  main: {
    prefixMinimum: 4,
    suffixMinimum: 3,
    maximumSlot: 8,
    minimumClause: 6,
  },
  loose: {
    prefixMinimum: 3,
    suffixMinimum: 3,
    maximumSlot: 10,
    minimumClause: 5,
  },
  tight: {
    prefixMinimum: 5,
    suffixMinimum: 4,
    maximumSlot: 6,
    minimumClause: 7,
  },
};

/**
 One place a repeated wording stands, by line.
 */
export type CensusOccurrence = {
  /**
   One-based line the occurrence stands on.
   */
  readonly line: number;

  /**
   Role of that line.
   */
  readonly kind: string;

  /**
   Whether the occurrence sits inside an open quotation.
   */
  readonly inQuote: boolean;
};

/**
 One repeat the census found, with the flags its occurrences carry.
 */
export type CensusGroup = {
  /**
   Kind of repeat: a maximal run or a refrain frame.
   */
  readonly kind: 'maximal' | 'refrain';

  /**
   Non-space characters in a maximal run, zero for a refrain.
   */
  readonly length: number;

  /**
   Where the repeat stands.
   */
  readonly occurrences: readonly CensusOccurrence[];

  /**
   Flags holding for at least one occurrence, sorted.
   */
  readonly flags: readonly string[];
};

/**
 Counts CJK ideographs in a string.

 @param text - any text

 @returns Number of ideographs

 @example
 ```ts
 const count = cjkCount({ text: '猫咪', },);
 ```
 */
function cjkCount({ text, }: { readonly text: string; },): number {
  return [...text.matchAll(CJK_IDEOGRAPH,),].length;
}

/**
 Removes the markup the census does not count as wording: link destinations
 (keeping the bracket), bare URLs, inline tags and expressions.

 @param line - raw line

 @returns The line with that markup blanked

 @example
 ```ts
 const wording = sanitize({ line: '[猫](https://cat.example.invalid) 好', },);
 ```
 */
export function sanitize({ line, }: { readonly line: string; },): string {
  return line
    .replace(
    LINK_DESTINATION,
    ']',
    )
    .replace(
    BARE_URL,
    ' ',
    )
    .replace(
    INLINE_TAG,
    ' ',
    )
    .replace(
    TEMPLATE_EXPRESSION,
    ' ',
    )
    .replace(
    BRACE_EXPRESSION,
    ' ',
    );
}

/**
 Classifies one line's role, in the census's precedence order.

 @param line - raw line

 @param index - zero-based line index

 @param inFence - whether the line lies inside a fenced code block

 @returns Role of the line

 @example
 ```ts
 const role = lineRole({ line: '## 标题', index: 0, inFence: false, },);
 ```
 */
function lineRole(
  {
    line,
    index,
    inFence,
  }: {
    readonly line: string;
    readonly index: number;
    readonly inFence: boolean;
  },
): string {
  if (inFence)
    return 'code';
  if (line.trim() === '')
    return 'blank';
  if (HEADING_LINE.test(line,))
    return 'heading';
  if (LIST_LINE.test(line,))
    return 'list';
  if (QUOTE_LINE.test(line,))
    return 'quote';
  if (HTML_LINE.test(line,))
    return 'html';
  if (FOOTNOTE_LINE.test(line,))
    return 'footnote';
  if (JSX_KEYWORD_LINE.test(line,))
    return 'jsx';
  if (JSX_MARK_LINE.test(line,))
    return 'jsx';
  if (JSX_OPEN_LINE.test(line,) && (cjkCount({ text: sanitize({ line, },), },) < JSX_CJK_FLOOR))
    return 'jsx';
  if ((index < FRONT_MATTER_LINES) && FRONT_KEY_LINE.test(line,))
    return 'frontmatter';
  return 'prose';
}

/**
 Classifies each line by its Markdown role, tracking fenced code blocks.

 @param lines - page lines

 @returns Role of each line

 @example
 ```ts
 const kinds = classify({ lines: ['## 标题',], },);
 ```
 */
export function classify({ lines, }: { readonly lines: readonly string[]; },): readonly string[] {
  /**
   Whether each line is a fence marker, which opens or closes a fenced block.
   */
  const fences = lines.map(function isFence(line,): boolean {
    return FENCE_LINE.test(line,);
  },);
  /**
   Fence markers seen so far in the walk over the lines, counted once per line so the walk is linear in the lines.
   */
  const seen = { markers: 0, };
  return lines.map(function roleOf(
    line,
    index,
  ): string {
    if (fences[index] === true) {
      seen.markers++;
      return 'fence';
    }
    // An odd count of markers before the line leaves it inside a fenced block.
    return lineRole({
      line,
      index,
      inFence: (seen.markers % FENCE_PAIR) === 1,
    },);
  },);
}

/**
 Counts the open quotation marks before a column of a normalised line, and
 reports whether any stays open there.

 @param norm - line with whitespace removed

 @param column - column of the occurrence

 @returns Whether the occurrence sits inside an open quotation

 @example
 ```ts
 const quoted = insideQuote({ norm: '「猫咪', column: 2, },);
 ```
 */
export function insideQuote({
  norm,
  column,
}: {
  readonly norm: string;
  readonly column: number;
},): boolean {
  /**
   Characters before the column.
   */
  const prefix = norm
    .slice(
    0,
    column,
    )
    .split('',);

  /**
   Opening marks before the column.
   */
  const opens = prefix
    .filter(function isOpener(char,): boolean {
      return OPENERS.includes(char,);
    },)
    .length;

  /**
   Closing marks before the column.
   */
  const closes = prefix
    .filter(function isCloser(char,): boolean {
      return CLOSERS.includes(char,);
    },)
    .length;
  return opens > closes;
}

/**
 Flags of a group: its occurrences' non-prose roles, quotation, and a single line.

 @param occurrences - places the repeat stands

 @returns Sorted flag names holding for at least one occurrence

 @example
 ```ts
 const flags = flagsOf({ occurrences, },);
 ```
 */
export function flagsOf({ occurrences, }: { readonly occurrences: readonly CensusOccurrence[]; },): readonly string[] {
  /**
   Flags collected so far.
   */
  const flags = new Set<string>();
  for (const occurrence of occurrences) {
    if (occurrence.kind !== 'prose')
      flags.add(occurrence.kind,);
    if (occurrence.inQuote)
      flags.add('quoted',);
  }
  /**
   Distinct lines the occurrences stand on.
   */
  const lines = new Set(occurrences.map(function lineOfOccurrence(occurrence,): number {
    return occurrence.line;
  },),);
  if (lines.size === 1)
    flags.add('same-line',);
  return textsInCodePointOrder({ texts: [...flags,], },);
}
//endregion Repeat census lines
