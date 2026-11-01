//region Consistency markup
// WHETHER A QUOTE CAN BE MOVED WITHOUT TAKING MARKUP WITH IT. A judge quotes
// the exact words of a slice's English, and the English may hold emphasis,
// links, code or footnote marks around those words. A quote that opens a mark
// and does not close it (or closes one it did not open) cannot be replaced or
// carried without leaving the page with a stray delimiter, and a quote that
// holds a footnote mark would give a place a reference whose definition stays
// in another slice. The floors read links, destinations and code, but nothing
// there reads an emphasis delimiter left open or a footnote mark carried alone,
// so the quote is refused here, before anything is replaced.
//
// A CONSERVATIVE READING, NOT A PARSE: the quote is a fragment, and no grammar
// reads a fragment. It counts delimiter runs and brackets, and refuses a
// fragment whose counts cannot pair. Words that only look unbalanced (an
// underscore inside a word, a lone parenthesis) are refused too, which leaves
// a place as it stands and costs a replacement, never a page.

/**
 Characters whose runs delimit emphasis or strikethrough.
 */
const EMPHASIS_MARKS = '*_~';

/**
 Opening and closing characters that must pair inside a quote.
 */
const PAIRED_BRACKETS: readonly (readonly [
  string,
  string
])[] = [
  [
    '[',
    ']',
  ],
  [
    '(',
    ')',
  ],
];

/**
 How many delimiters of each kind a text opens or closes, reading a run of
 an emphasis character as pairs of the doubled mark and one single mark, and a
 run of backticks as one code delimiter.

 @param text - fragment to read

 @returns Count of single, doubled and code delimiters

 @example
 ```ts
 const counts = delimiterCounts({ text: '**cat** naps', },); // { single: 0, doubled: 2, code: 0 }
 ```
 */
function delimiterCounts(
  { text, }: { readonly text: string; },
): {
  readonly single: number;
  readonly doubled: number;
  readonly code: number
} {
  /**
   Totals so far, mutated as the runs are read.
   */
  const totals = {
    single: 0,
    doubled: 0,
    code: 0,
  };
  /**
   Run being read: its character and how many units it has so far.
   */
  const run = {
    character: '',
    length: 0,
  };
  /**
   Counts the run that just ended.
   */
  function endRun(): void {
    if (run.length === 0)
      return;
    if (run.character === '`')
      totals.code += 1;
    else if (EMPHASIS_MARKS.includes(run.character,)) {
      totals.doubled += Math.floor(run.length / 2,);
      totals.single += run.length % 2;
    }
    run.character = '';
    run.length = 0;
  }
  for (const character of text) {
    if ((run.length > 0) && (character === run.character)) {
      run.length += 1;
      continue;
    }
    endRun();
    if ((character === '`') || EMPHASIS_MARKS.includes(character,)) {
      run.character = character;
      run.length = 1;
    }
  }
  endRun();
  return totals;
}

/**
 How many times a character stands in a text.

 @param text - text to read

 @param character - one character to count

 @returns Number of occurrences

 @example
 ```ts
 const opened = countOf({ text: '(a (b)', character: '(', },); // 2
 ```
 */
function countOf(
  {
    text,
    character,
  }: {
    readonly text: string;
    readonly character: string;
  },
): number {
  return text.split(character,)
    .length
    - 1;
}

/**
 Whether a quote holds a footnote mark such as `[^1]`.

 @param text - fragment to read

 @returns Whether `[^` stands in it

 @example
 ```ts
 const marked = holdsFootnoteMark({ text: 'the cat naps[^1]', },); // true
 ```
 */
function holdsFootnoteMark({ text, }: { readonly text: string; },): boolean {
  return text.includes('[^',);
}

/**
 Whether a quote carries markup it cannot take along or leave behind: an
 emphasis or code delimiter without its pair, a bracket without its pair, or a
 footnote mark.

 @param text - quote or slot as a judge gave it

 @returns Whether moving it would strand a delimiter or a footnote reference

 @example
 ```ts
 const stray = carriesOrCutsMarkup({ text: 'on the sill** lulled the kitten', },); // true
 ```
 */
export function carriesOrCutsMarkup({ text, }: { readonly text: string; },): boolean {
  if (holdsFootnoteMark({ text, },))
    return true;
  /**
   Delimiters the fragment opens or closes.
   */
  const counts = delimiterCounts({ text, },);
  if (((counts.single % 2) !== 0) || ((counts.doubled % 2) !== 0)
    || ((counts.code % 2) !== 0))
    return true;
  return PAIRED_BRACKETS.some(function unpaired([open, close,],): boolean {
    return countOf({
      text,
      character: open,
    },) !== countOf({
      text,
      character: close,
    },);
  },);
}

//endregion Consistency markup
