/**
 Regex-free separator tokenizer shared by model-id and speed-signal scans.

 Both call sites accumulate past every separator; only the separator set
 differs, so the walk lives here and each caller supplies its own predicate.

 @module
 */

//region Token splitting

/**
 Split text into non-empty tokens at separator characters.

 Separator runs at either end or between tokens never emit an empty token, so
 the result carries exactly the pieces a caller scores or parses. Characters
 the predicate rejects, including those outside the BMP, stay inside their
 token.

 @param value - text to tokenize

 @param isSeparator - predicate naming the characters that end a token

 @returns non-empty tokens in input order

 @example
 ```typescript
 splitSeparatedTokens({
   value: 'claude-3.5',
   isSeparator: (character) => character === '-',
 }); // ['claude', '3.5']
 ```

 @internal
 */
export function splitSeparatedTokens(
  {
    value,
    isSeparator,
  }: {
    readonly value: string;
    readonly isSeparator: (character: string) => boolean;
  },
): string[] {
  /**
   Completed tokens.
   */
  const tokens: string[] = [];
  /**
   Characters collected for the current token.
   */
  let currentTokenCharacters: string[] = [];
  for (const character of value) {
    if (isSeparator(character,)) {
      if (currentTokenCharacters.length
        > 0) {
        tokens.push(currentTokenCharacters.join('',),);
        currentTokenCharacters = [];
      }
      continue;
    }
    currentTokenCharacters.push(character,);
  }
  if (currentTokenCharacters.length
    > 0)
    tokens.push(currentTokenCharacters.join('',),);
  return tokens;
}

//endregion Token splitting
