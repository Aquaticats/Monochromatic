import type { TextRange, } from './source-ships-ranges.ts';

//region Source ships display math
// DISPLAY MATH THAT THE PLAIN READING TAKES FOR INLINE MATH. A line of `$$` alone indented four columns or more, after a
// line of a paragraph, is a line of the paragraph with math that runs over the lines to the plain reading, which has the
// indented code that would cap the indentation. The strict grammar has none: a line of `$$` alone opens display math there
// whatever its indentation, and the next such line closes it, and a brace in it is no expression. Its lines are copied as
// they stand. Joining them as the lines of inline math that a brace cuts are joined (`source-ships-lines.ts`) would leave
// the page an unclosed display math that takes the rest of the text.

/**
 Characters that stand before the content of a line inside a container: the indentation and the mark of a block quote.
 */
const CONTAINER_PREFIX = ' \t>';

/**
 The dollar signs that open and close display math.
 */
const FENCE = '$$';

/**
 Whether a stretch holds nothing but the characters that stand before the content of a line.

 @param text - stretch

 @returns Whether every character is a space, a tab or a greater-than sign

 @example
 ```ts
 const before = isContainerPrefix({ text: '>     ', },);
 ```
 */
function isContainerPrefix({ text, }: { readonly text: string; },): boolean {
  return Array.from(text,)
    .every(function isMark(character,): boolean {
      return CONTAINER_PREFIX.includes(character,);
    },);
}

/**
 Whether the rest of a line from an index holds nothing but white space.

 @param source - text

 @param from - index to read from

 @returns Whether the line is blank from there to its end

 @example
 ```ts
 const blank = isBlankFrom({ source: '$$  \nx', from: 2, },);
 ```
 */
function isBlankFrom(
  {
    source,
    from,
  }: {
    readonly source: string;
    readonly from: number;
  },
): boolean {
  /**
   Index of the line ending, none at the end of the text.
   */
  const ending = source.indexOf(
    '\n',
    from,
  );
  /**
   The rest of the line.
   */
  const rest = source.slice(
    from,
    (ending === (-1)) ? source.length : ending,
  );
  return rest.trim() === '';
}

/**
 Whether inline math of the plain reading is display math to the strict grammar: it opens with a line of `$$` alone that
 stands first on its line, and closes with a later line of `$$` alone.

 @param source - text

 @param range - where the inline math stands

 @returns Whether the lines of the math are copied as they stand

 @example
 ```ts
 const display = readsAsDisplayMath({ source: 'a\n    $$\n    {a}\n    $$\n', range: { start: 6, end: 24, }, },);
 ```
 */
export function readsAsDisplayMath(
  {
    source,
    range,
  }: {
    readonly source: string;
    readonly range: TextRange;
  },
): boolean {
  if (!source.startsWith(
    FENCE,
    range.start,
  ))
    return false;
  if (!source.startsWith(
    FENCE,
    range.end - FENCE.length,
  ))
    return false;
  /**
   Index of the line ending that ends the opening line.
   */
  const opening = source.indexOf(
    '\n',
    range.start,
  );
  if ((opening === (-1)) || (opening >= range.end))
    return false;
  if (!isBlankFrom({
    source,
    from: range.start + FENCE.length,
  },))
    return false;
  /**
   Index of the line ending before the opening line.
   */
  const before = source.lastIndexOf(
    '\n',
    range.start - 1,
  );
  if ((before === (-1)) || (!isContainerPrefix({
    text: source.slice(
      before + 1,
      range.start,
    ),
  },)))
    return false;
  /**
   Index of the first character of the closing line.
   */
  const closing = source.lastIndexOf(
    '\n',
    range.end - 1,
  ) + 1;
  if ((closing <= opening) || (!isContainerPrefix({
    text: source.slice(
      closing,
      range.end - FENCE.length,
    ),
  },)))
    return false;
  return isBlankFrom({
    source,
    from: range.end,
  },);
}

//endregion Source ships display math
