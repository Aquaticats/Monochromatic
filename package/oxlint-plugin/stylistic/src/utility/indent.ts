import { isWhitespaceChar, } from '@monochromatic-dev/oxlint-plugin-shared/ts';

/**
 Returns the leading run of ASCII whitespace from `text`.
 
 Linear scan: walks character by character until the first non-whitespace
 position, then slices once. Replaces the prior `/^(\s*)/.exec` capture
 with a strictly O(n) implementation suitable for hot lint paths.
 
 @param text - text whose leading whitespace run is wanted
 
 @returns prefix of `text` consisting solely of whitespace characters
 
 @example
 ```ts
 leadingWhitespace('  foo'); // '  '
 leadingWhitespace('bar'); // ''
 ```
 
 @internal
 */
export function leadingWhitespace(text: string,): string {
  /**
   Linear scan returning the first non-whitespace index, or `text.length` when
   `text` is empty or entirely whitespace. Single forward pass: O(n) time, O(1)
   stack, no recursion.
   
   @returns first non-whitespace position (or `text.length`)
   */
  function scan(): number {
    for (let cursorIndex = 0; cursorIndex < text
      .length; cursorIndex += 1) {
      if (!isWhitespaceChar(text.charAt(cursorIndex,),))
        return cursorIndex;
    }
    return text.length;
  }
  return text.slice(
    0,
    scan(),
  );
}

/**
 Parameters for {@link baseIndentAt}.
 */
export type BaseIndentAtParams = {
  /**
   Full file source text.
   */
  readonly sourceText: string;
  /**
   Byte offset on the line whose indentation is wanted.
   */
  readonly offset: number;
};

/**
 Returns the whitespace prefix of the line containing `offset`.
 
 Used to derive the indentation level when emitting multi-line autofix
 replacements so the inserted content lines up with the surrounding code.
 
 @returns leading whitespace of the line, or `''` if the line has none
 
 @example
 ```ts
 baseIndentAt({ sourceText: '  foo(a, b);', offset: 6 }) // -> '  '
 ```
 
 @internal
 */
export function baseIndentAt({
  sourceText,
  offset,
}: BaseIndentAtParams,): string {
  /**
   Byte offset of the first character on the line containing `offset`.
   */
  const lineStart = sourceText.lastIndexOf(
    '\n',
    offset - 1,
  )
    + 1;
  /**
   Substring from line start to `offset`; the helper returns just its leading whitespace.
   */
  const linePrefix = sourceText.slice(
    lineStart,
    offset,
  );
  return leadingWhitespace(linePrefix,);
}
