/**
 Comment-aware operator token location between two operand spans.
 
 @example
 ```ts
 findOperatorToken({ source: 'a + b', from: 1, to: 4, token: '+' });
 ```
 */

/**
 Locates one operator token between operand spans, skipping comments.
 
 Between a binary/logical expression's `left.end` and `right.start` only
 whitespace, parentheses, and comments can occur, so a linear scan that
 skips `//` and `/*` comments finds the real operator token; a plain
 `indexOf` would match operator characters inside comments.
 
 @param options - Source text, scan bounds, and operator token text.
 
 @returns Start offset of the token.
 
 @throws Error when the token does not occur outside comments in range.
 
 @example
 ```ts
 findOperatorToken({ source: 'a + b', from: 1, to: 4, token: '+' });
 // 2
 ```
 */
export function findOperatorToken(options: {
  readonly source: string;
  readonly from: number;
  readonly to: number;
  readonly token: string;
},): number {
  return (function scanTrivia(): number {
  /**
   Scan cursor advancing through trivia between operands.
   */
    let cursor = options.from;

    while (cursor < options.to) {
    /**
     Two-character lookahead classifying comment openers.
     */
      const pair = options.source
        .slice(
        cursor,
        cursor + 2,
        );

      if (pair === '//') {
      /**
       End of line comment, bounded by scan range.
       */
        const lineEnd = options.source
          .indexOf(
          '\n',
          cursor,
          );
        cursor = lineEnd === (-1) ? options.to : lineEnd + 1;
        continue;
      }

      if (pair === '/*') {
      /**
       End of block comment, bounded by scan range.
       */
        const blockEnd = options.source
          .indexOf(
          '*/',
          cursor + 2,
          );

        if (blockEnd === (-1))
          throw new Error(`unterminated block comment at offset ${String(cursor,)}`,);

        cursor = blockEnd + 2;
        continue;
      }

      if (options.source
        .startsWith(
        options.token,
        cursor,
        ))
        return cursor;

      cursor += 1;
    }

    throw new Error(
      `operator token ${options.token} not found between offsets ${String(options.from,)} and ${String(options.to,)}`,
    );
  })();
}
