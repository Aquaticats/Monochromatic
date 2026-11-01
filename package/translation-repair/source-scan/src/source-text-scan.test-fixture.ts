import { isAsciiAlphanumeric, } from '@monochromatic-dev/translation-repair-markup';

//region Source text scan
// Index scans over this package's own formatted source: matching brackets,
// splitting at depth zero, dropping comments, finding a whole identifier.
// Split from `tsdoc-example-scan.test-fixture.ts` at its line budget; strings and comments
// are skipped so a bracket inside either is text, not structure.

/**
 Value `indexOf` returns for text that is not there.
 */
export const NOT_FOUND = -1;

/**
 Bracket pairs the scan balances.
 */
const CLOSER: Readonly<Record<string, string>> = {
  '(': ')',
  '{': '}',
  '[': ']',
};

/**
 Whether a character can sit inside an identifier.

 @param character - one character, or empty past either end

 @returns Whether it is a letter, digit, `_` or `$`

 @example
 ```ts
 const inside = isIdentifierCharacter({ character: 'a', },);
 ```
 */
export function isIdentifierCharacter({ character, }: { readonly character: string; },): boolean {
  return isAsciiAlphanumeric({ character, },)
    || (character === '_')
    || (character === '$');
}

/**
 Finds the bracket closing the one at `open`, skipping strings and comments.

 @param text - text holding both brackets

 @param open - index of the opening bracket

 @returns Index of its closing bracket, or `NOT_FOUND`

 @example
 ```ts
 const close = matchingClose({ text: '(a, (b))', open: 0, },);
 ```
 */
export function matchingClose({
  text,
  open,
}: {
  readonly text: string;
  readonly open: number
},): number {
  /**
   Closers still owed, innermost last.
   */
  const owed: string[] = [];
  for (let index = open; index < text.length; index += 1) {
    /**
     Character at the cursor.
     */
    const character = text[index] ?? '';
    if ((character === '\'') || (character === '"')
      || (character === '`')) {
      // A STRING'S BRACKETS ARE TEXT: skip to its closing quote.
      /**
       Where the string closes.
       */
      const end = text.indexOf(
        character,
        index + 1,
      );
      if (end === NOT_FOUND)
        return NOT_FOUND;
      index = end;
    }
    else if (text.startsWith(
      '/*',
      index,
    )) {
      index = text.indexOf(
        '*/',
        index + 2,
      ) + 1;
      if (index === 0)
        return NOT_FOUND;
    }
    else if (text.startsWith(
      '//',
      index,
    )) {
      index = text.indexOf(
        '\n',
        index,
      );
      if (index === NOT_FOUND)
        return NOT_FOUND;
    }
    else if (CLOSER[character] !== undefined)
      owed.push(CLOSER[character] ?? '',);
    else if (character === owed.at(-1,)) {
      owed.pop();
      if (owed.length === 0)
        return index;
    }
  }
  return NOT_FOUND;
}

/**
 Splits text at a separator where no bracket is open.

 @param text - text to split

 @param separator - one character

 @returns Pieces, trimmed, empty ones dropped

 @example
 ```ts
 const pieces = topLevelPieces({ text: 'a, { b, c }', separator: ',', },);
 ```
 */
export function topLevelPieces({
  text,
  separator,
}: {
  readonly text: string;
  readonly separator: string
},): readonly string[] {
  /**
   Where the separator stands at depth zero.
   */
  const cuts: number[] = [];
  for (let index = 0; index < text.length; index += 1) {
    /**
     Character at the cursor.
     */
    const character = text[index] ?? '';
    if (CLOSER[character] !== undefined) {
      /**
       Where this bracket closes.
       */
      const close = matchingClose({
        text,
        open: index,
      },);
      index = (close === NOT_FOUND) ? text.length : close;
    }
    else if (character === separator)
      cuts.push(index,);
  }

  /**
   Piece boundaries: before the text, each cut, and its end.
   */
  const bounds = [
    NOT_FOUND,
    ...cuts,
    text.length,
  ];

  /**
   The pieces between them.
   */
  const pieces = bounds.slice(1,)
    .map(function pieceEndingAt(
      end,
      at,
    ): string {
      return text.slice(
        (bounds[at] ?? NOT_FOUND) + 1,
        end,
      );
    },);
  return pieces
    .map(function trimmed(piece,): string {
      return piece.trim();
    },)
    .filter(function nonEmpty(piece,): boolean {
      return piece !== '';
    },);
}

/**
 Removes block and line comments from type text.

 @param text - type text

 @returns The text without comments

 @example
 ```ts
 const bare = withoutBlockAndLineComments({ text: '{ /** doc *\/ readonly a: T; }', },);
 ```
 */
export function withoutBlockAndLineComments({ text, }: { readonly text: string; },): string {
  /**
   Text kept so far.
   */
  const kept: string[] = [];
  for (let index = 0; index < text.length; index += 1) {
    if (text.startsWith(
      '/*',
      index,
    )) {
      /**
       Where the comment ends.
       */
      const end = text.indexOf(
        '*/',
        index + 2,
      );
      index = (end === NOT_FOUND) ? text.length : (end + 1);
    }
    else if (text.startsWith(
      '//',
      index,
    )) {
      /**
       Where the line ends.
       */
      const end = text.indexOf(
        '\n',
        index,
      );
      index = (end === NOT_FOUND) ? text.length : (end - 1);
    }
    else
      kept.push(text[index] ?? '',);
  }
  return kept.join('',);
}

/**
 Where an identifier occurs whole in text, or `NOT_FOUND`.

 @param text - text to search

 @param word - identifier to find

 @param after - literal text that must follow it, such as `(`

 @returns Index of the first whole occurrence followed by `after`

 @example
 ```ts
 const at = wholeWordAt({ text: 'meow(purr(',  word: 'purr', after: '(', },);
 ```
 */
export function wholeWordAt(
  {
    text,
    word,
    after,
  }: {
    readonly text: string;
    readonly word: string;
    readonly after: string
  },
): number {
  for (let index = text.indexOf(word,); index !== NOT_FOUND; index = text.indexOf(
    word,
    index + 1,
  )) {
    /**
     Index just past the word.
     */
    const end = index + word.length;
    if ((!isIdentifierCharacter({ character: text[index - 1] ?? '', },))
      && (!isIdentifierCharacter({ character: text[end] ?? '', },))
      && text.startsWith(
        after,
        end,
      ))
      return index;
  }
  return NOT_FOUND;
}

//endregion Source text scan
