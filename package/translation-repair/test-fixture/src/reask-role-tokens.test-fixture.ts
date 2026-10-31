//region Re-ask role tokens
// THE CUTTING OF SOURCE TEXT THE READER OF THE ROLE A CALL NAMES WORKS ON (`reask-role-call.test-fixture.ts`): tokens
// with comments dropped, the end of a bracket and the properties of an object literal.

/**
 One piece of source text.
 */
export type Token = {
  /**
   What kind of piece it is: a word or number, a quoted string, or one punctuation character.
   */
  readonly kind: 'word' | 'string' | 'punct';

  /**
   Its text; a string without its quotes.
   */
  readonly text: string;
};

/**
 ASCII letters, digits, `_` and `$`; a letter of another script is a word character by its case, in
 {@link isWordCharacter}.
 */
const WORD_CHARACTERS = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_$';

/**
 Whether a character can be part of a word.

 @param character - one character

 @returns True for a letter, a digit, `_` or `$`

 @example
 ```ts
 const inWord = isWordCharacter({ character: 'a', },);
 ```
 */
function isWordCharacter({ character, }: { readonly character: string; },): boolean {
  return WORD_CHARACTERS.includes(character,) || (character.toLowerCase() !== character.toUpperCase());
}

/**
 Reads the end of a quoted string or template, past escapes. A template ends at its next unescaped backtick, so a
 backtick inside a `${}` hole ends it early; the call sites this reads hold none.

 @param source - whole text

 @param start - index of the opening quote

 @returns Index one past the closing quote

 @example
 ```ts
 const end = endOfQuoted({ source: "'a'", start: 0, },);
 ```
 */
function endOfQuoted(
  {
    source,
    start,
  }: {
    readonly source: string;
    readonly start: number;
  },
): number {
  /**
   The quote that closes it.
   */
  const quote = source[start] ?? '';
  for (let at = start + 1; at < source.length; at += 1) {
    /**
     Character read.
     */
    const character = source[at] ?? '';
    if (character === '\\')
      at += 1;
    else if (character === quote)
      return at + 1;
  }
  return source.length;
}

/**
 Cuts source text into tokens, dropping comments.

 @param source - text of a TypeScript file

 @returns Tokens in source order

 @example
 ```ts
 const tokens = tokensOf({ source: 'call({ role: 1 },);', },);
 ```
 */
export function tokensOf({ source, }: { readonly source: string; },): readonly Token[] {
  /**
   Tokens found so far.
   */
  const tokens: Token[] = [];
  for (let at = 0; at < source.length;) {
    /**
     Character read.
     */
    const character = source[at] ?? '';
    if ((character === ' ') || (character === '\n')
      || (character === '\t')
      || (character === '\r')) {
      at += 1;
    }
    else if (source.startsWith(
      '//',
      at,
    )) {
      /**
       End of the line comment.
       */
      const lineEnd = source.indexOf(
        '\n',
        at,
      );
      at = (lineEnd === (-1)) ? source.length : lineEnd;
    }
    else if (source.startsWith(
      '/*',
      at,
    )) {
      /**
       End of the block comment.
       */
      const blockEnd = source.indexOf(
        '*/',
        at + 2,
      );
      at = (blockEnd === (-1)) ? source.length : blockEnd + 2;
    }
    else if ((character === '\'') || (character === '"')
      || (character === '`')) {
      /**
       Index past the string.
       */
      const end = endOfQuoted({
        source,
        start: at,
      },);
      tokens.push({
        kind: 'string',
        text: source.slice(
          at + 1,
          end - 1,
        ),
      },);
      at = end;
    }
    else if (isWordCharacter({ character, },)) {
      /**
       Index past the word.
       */
      let end = at;
      while ((end < source.length) && isWordCharacter({ character: source[end] ?? '', },))
        end += 1;
      tokens.push({
        kind: 'word',
        text: source.slice(
          at,
          end,
        ),
      },);
      at = end;
    }
    else {
      tokens.push({
        kind: 'punct',
        text: character,
      },);
      at += 1;
    }
  }
  return tokens;
}

/**
 Index one past the bracket that closes the one at `open`.

 @param tokens - all tokens
 
 @param open - index of an opening `(`, `[`, `{` or `<`
 
 @param angled - whether `<` and `>` count as brackets, which a type argument list needs and an object does not
 
 @returns Index one past the matching closer, or the end

 @example
 ```ts
 const after = afterBracket({ tokens, open: 3, angled: false, },);
 ```
 */
export function afterBracket(
  {
    tokens,
    open,
    angled,
  }: {
    readonly tokens: readonly Token[];
    readonly open: number;
    readonly angled: boolean;
  },
): number {
  return (function scan(): number {
    /**
     Nesting depth, counting every kind of bracket as one.
     */
    let depth = 0;
    for (let at = open; at < tokens.length; at += 1) {
      /**
       Text of the token, empty for a token that is no punctuation.
       */
      const text = (tokens[at]
        ?.kind
        === 'punct') ? tokens[at]
          ?.text
          ?? '' : '';
      if ((text === '(') || (text === '[')
        || (text === '{')
        || (angled && (text === '<')))
        depth += 1;
      if ((text === ')') || (text === ']')
        || (text === '}')
        || (angled && (text === '>')))
        depth -= 1;
      if (depth === 0)
        return at + 1;
    }
    return tokens.length;
  })();
}

/**
 Splits the tokens of an object literal's body into its properties at the commas outside every bracket.

 @param body - tokens between the braces
 
 @returns One list of tokens per property

 @example
 ```ts
 const properties = propertiesOf({ body, },);
 ```
 */
export function propertiesOf({ body, }: { readonly body: readonly Token[]; },): readonly (readonly Token[])[] {
  return (function split(): readonly (readonly Token[])[] {
    /**
     Properties closed so far.
     */
    const properties: (readonly Token[])[] = [];

    /**
     Tokens of the property being read.
     */
    let current: Token[] = [];

    /**
     Bracket depth.
     */
    let depth = 0;
    for (const token of body) {
      if (token.kind === 'punct') {
        if ((token.text === '(') || (token.text === '[')
          || (token.text === '{'))
          depth += 1;
        if ((token.text === ')') || (token.text === ']')
          || (token.text === '}'))
          depth -= 1;
        if ((token.text === ',') && (depth === 0)) {
          properties.push(current,);
          current = [];
          continue;
        }
      }
      current.push(token,);
    }
    if (current.length > 0)
      properties.push(current,);
    return properties;
  })();
}

//endregion Re-ask role tokens
