import type {
  TextEdit,
  TextRange,
} from './source-ships-ranges.ts';

//region Source ships opener text
// HOW A STRETCH THAT HELD AN OPENER OF A COMMENT THAT NEVER CLOSES IS WRITTEN AS TEXT, once it is taken out of the code or
// math it stood in (`source-ships-opener-in-code.ts` says why and which): the opener as a character reference, a comment that
// closes as it stands, every character that starts syntax with a backslash, and the marks of the containers at the start of
// a line as they are.

/**
 What the code and math that hold an opener come to.
 */
export type OpenerInCode = {
  /**
   Edits that write each fence line, code span and inline math holding an opener, in text order and not overlapping.
   */
  readonly edits: readonly TextEdit[];

  /**
   Index of the `<` of each opener the edits write, which no edit of its own is to write again.
   */
  readonly handled: readonly number[];
};

/**
 Characters a backslash goes before in what is written as text: those that start inline syntax or a block, and the braces
 and the less-than sign the strict grammar reads as syntax.
 */
export const TEXT_SYNTAX = '\\`*_{}[]#+-.!<>&|~$=)';

/**
 What an opener is written as.
 */
export const OPENER_WRITTEN = '&lt;';

/**
 The characters of a line before a block's content that stand the same on every line of the block: the marks of a block
 quote and white space.
 */
export const CARRIED = ' \t>';

/**
 The line ending that ends a line.

 @param source - text

 @param feed - index of the line feed that ends the line

 @returns A carriage return and a line feed when a carriage return stands before the line feed, a line feed otherwise

 @example
 ```ts
 const ending = endingAt({ source: 'a\r\nb', feed: 2, },);
 ```
 */
export function endingAt({
  source,
  feed,
}: {
  readonly source: string;
  readonly feed: number
},): string {
  return ((feed > 0) && (source.charAt(feed - 1,) === '\r')) ? '\r\n' : '\n';
}

/**
 Writes a stretch of the text as text: an opener as the character reference, a comment that closes as it stands, every
 character that starts syntax with a backslash, and the marks of the containers at the start of a line as they are.

 @param source - text

 @param range - where the stretch stands

 @param openers - index of the `<` of each opener of a comment that never closes

 @param closed - comments that close, which the page masks

 @returns The stretch written for the strict grammar

 @example
 ```ts
 const written = writtenAsText({ source, range: { start: 0, end: 9, }, openers: [4,], closed: [], },);
 ```
 */
export function writtenAsText(
  {
    source,
    range,
    openers,
    closed,
  }: {
    readonly source: string;
    readonly range: TextRange;
    readonly openers: readonly number[];
    readonly closed: readonly TextRange[];
  },
): string {
  /**
   Pieces of the stretch, joined at the end.
   */
  const pieces: string[] = [];
  for (let at = range.start; at < range.end; at += 1) {
    /**
     Character read.
     */
    const character = source.charAt(at,);
    /**
     The comment that closes and holds this character.
     */
    const comment = closed.find(function holds({
      start,
      end,
    },): boolean {
      return (start <= at) && (at < end);
    },);
    if (comment !== undefined) {
      /**
       Where the comment ends here.
       */
      const until = Math.min(
        comment.end,
        range.end,
      );
      pieces.push(source.slice(
        at,
        until,
      ),);
      at = until - 1;
      continue;
    }
    if (openers.includes(at,)) {
      pieces.push(OPENER_WRITTEN,);
      continue;
    }
    pieces.push(TEXT_SYNTAX.includes(character,) ? `\\${character}` : character,);
    if (character === '\n') {
      /**
       Where the marks of the containers end on the next line.
       */
      const marks = { end: at + 1, };
      while ((marks.end < range.end) && CARRIED.includes(source.charAt(marks.end,),))
        marks.end += 1;
      pieces.push(source.slice(
        at + 1,
        marks.end,
      ),);
      at = marks.end - 1;
    }
  }
  return pieces.join('',);
}

//endregion Source ships opener text
