/**
 Regex-free separator tokenizer shared by model-id and speed-signal scans.

 Both call sites accumulate past every separator; only the separator set
 differs, so the walk lives here and each caller supplies its own predicate.

 @module
 */

//region Separator predicates

/**
 Check whether a character is ASCII whitespace.

 Covers the six characters that the model-id and speed-signal separator
 predicates share. Non-ASCII spaces such as the no-break space stay inside
 tokens because model registries never use them as separators.

 @param character - character to check

 @returns whether character is ASCII whitespace

 @example
 ```typescript
 isAsciiWhitespace('\t'); // true
 isAsciiWhitespace('\u00A0'); // false
 ```

 @internal
 */
export function isAsciiWhitespace(
  character: string,
): boolean {
  return (character === ' ')
    || (character === '\t')
    || (character === '\n')
    || (character === '\r')
    || (character === '\f')
    || (character === '\v');
}

//endregion Separator predicates

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
