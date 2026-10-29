/**
 Pure parsing helpers for `mise registry` output lines.
 
 Kept separate from `mise.generate-index.ts` so the parsing logic is
 importable and unit-testable without triggering that script's
 top-level container pipeline.
 */

/**
 Reports whether `c` is ASCII whitespace as matched by regex `\s`
 (space, tab, newline, carriage return, form feed, vertical tab).
 
 @param c - single character to classify
 
 @returns whether `c` is whitespace
 
 @example
 ```ts
 isWhitespace(' ');  // true
 isWhitespace('\t'); // true
 isWhitespace('a');  // false
 ```
 */
function isWhitespace(c: string,): boolean {
  return (c === ' ')
    || (c === '\t')
    || (c === '\n')
    || (c === '\r')
    || (c === '\f')
    || (c === '\v');
}

/**
 Returns the leading non-whitespace token of `line`. Whitespace is
 defined the same way as regex `\s` (space, tab, newline, carriage
 return, form feed, vertical tab); empty or all-whitespace input returns
 an empty token.
 
 Used to read the tool name from each line of `mise registry` output.
 Whitespace is classified character by character with {@link isWhitespace}.
 
 Single left-to-right pass: leading whitespace is skipped, the first
 whitespace after the run ends it, and the run is joined once. The token
 is rebuilt from iterated characters rather than sliced by index, so
 code-point iteration reconstructs the substring identically to the
 original code-unit scan. O(n) time, O(1) stack, no recursion
 (stack-safe under engines without tail-call elimination).
 
 @param line - input line
 
 @returns first non-whitespace token (possibly empty)
 
 @example
 ```ts
 firstWhitespaceToken('  ripgrep  aqua:BurntSushi/ripgrep'); // 'ripgrep'
 firstWhitespaceToken('   ');                                // ''
 ```
 */
export function firstWhitespaceToken(line: string,): string {
  /**
   First maximal run of non-whitespace characters, accumulated in order and joined once.
   */
  const tokenChars: string[] = [];
  for (const c of line) {
    if (isWhitespace(c,)) {
      // whitespace after the token ends it; leading whitespace is skipped
      if (tokenChars.length
        > 0)
        break;
      continue;
    }
    tokenChars.push(c,);
  }
  return tokenChars.join('',);
}
