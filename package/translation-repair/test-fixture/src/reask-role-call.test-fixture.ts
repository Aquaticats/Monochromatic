//region Re-ask role at a call
// WHICH ROLE A CALL NAMES, READ FROM THE SOURCE TEXT (ledger B1079, B1100). The re-ask of a reasoning runaway leaves out
// the models its role may not take, so every call of a function that asks a bench names the role of that bench. A
// caller that cannot be driven to its gather with a scripted client in one case is held to its role by what its source
// names: this reads the argument object of each real call and returns the value of its role key.
//
// IT READS TOKENS AND NOT LINES. The source is cut into words, strings and punctuation with comments dropped, so the
// formatter's layout decides nothing: a call with `await`, a returned call, a call kept in a variable and a call on one
// line are all read, an example in a comment is none, and a declaration (`function callee`) or a method of another
// object (`.callee(`) is no call. The argument object is split into its properties at the commas that are not inside a
// bracket, so the role of a message in a prompt, which sits inside brackets, is no property of the call.

/**
 One piece of source text.
 */
type Token = {
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
 The three dots that open a spread property.
 */
const SPREAD_MARK = '...';

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
function tokensOf({ source, }: { readonly source: string; },): readonly Token[] {
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
function afterBracket(
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
function propertiesOf({ body, }: { readonly body: readonly Token[]; },): readonly (readonly Token[])[] {
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

/**
 Reads what one call's argument names under a key.

 @param properties - properties of the call's argument object
 
 @param key - key whose values are read
 
 @returns The value named, `none` where the key is absent, `<value> then a spread` where a spread follows the key,
 and `duplicate <key>: a, b` where the key is written twice

 @example
 ```ts
 const named = namedUnderKey({ properties, key: 'role', },);
 ```
 */
function namedUnderKey(
  {
    properties,
    key,
  }: {
    readonly properties: readonly (readonly Token[])[];
    readonly key: string;
  },
): string {
  /**
   Whether a property is a spread, `...x`.

   @param property - tokens of one property

   @returns True when it opens with three dots
   */
  function isSpread(property: readonly Token[],): boolean {
    return property
      .slice(
      0,
      SPREAD_MARK.length,
      )
      .map(function textOf(token,): string {
        return token.text;
      },)
      .join('',)
      === SPREAD_MARK;
  }

  /**
   Whether a property is written under the key.

   @param property - tokens of one property

   @returns True when its first token is the key
   */
  function isKey(property: readonly Token[],): boolean {
    return property[0]
      ?.text
      === key;
  }

  /**
   Values the key is given, in source order.
   */
  const values = properties
    .filter(isKey,)
    .map(function valueOf(property,): string {
      return (property[1]
        ?.text
        === ':')
        ? property
          .slice(2,)
          .map(function textOf(token,): string {
            return token.text;
          },)
          .join('',)
        : key;
    },);

  /**
   Whether a spread follows the first property under the key, which can replace the value it names.
   */
  const spreadAfter = properties
    .slice(properties.findIndex(isKey,) + 1,)
    .some(isSpread,)
    && (values.length > 0);
  if (values.length === 0)
    return 'none';
  if (values.length > 1)
    return `duplicate ${key}: ${values.join(', ',)}`;
  return spreadAfter ? `${values[0] ?? ''} then a spread` : values[0] ?? '';
}

/**
 Reads the value each real call of a function names under a key.

 @param source - text of the file that calls the function

 @param callee - function called

 @param key - key of the argument object that names the role; `role` unless a stage takes it under another name

 @returns One entry per real call, in source order: the value named, the name of the variable passed, `none` where the
 call names none, `<value> then a spread` where a spread after the key can replace it, `duplicate <key>: a, b` where the
 key is written twice, or `not an object literal` where the argument is not an object literal

 @example
 ```ts
 expect(rolesNamedAtCalls({ source, callee: 'gatherStageVoices', },),).toEqual(['wide-judge',],);
 ```
 */
export function rolesNamedAtCalls(
  {
    source,
    callee,
    key = 'role',
  }: {
    readonly source: string;
    readonly callee: string;
    readonly key?: string;
  },
): readonly string[] {
  /**
   Tokens of the file.
   */
  const tokens = tokensOf({ source, },);

  /**
   What each call names.
   */
  const named: string[] = [];
  for (let at = 0; at < tokens.length; at += 1) {
    if ((tokens[at]
      ?.kind
      !== 'word') || (tokens[at]
        ?.text
        !== callee))
      continue;

    /**
     The token before the name, which tells a call from a declaration or a method of another object.
     */
    const before = tokens[at - 1]
      ?.text;
    if ((before === 'function') || (before === '.'))
      continue;

    /**
     Index of the token after the name and its type arguments.
     */
    const afterName = (tokens[at + 1]
      ?.text
      === '<') ? afterBracket({
        tokens,
        open: at + 1,
        angled: true,
      },) : at + 1;
    if (tokens[afterName]
      ?.text
      !== '(')
      continue;
    if (tokens[afterName + 1]
      ?.text
      !== '{') {
      named.push('not an object literal',);
      continue;
    }

    /**
     Index one past the closing brace of the argument object.
     */
    const afterObject = afterBracket({
      tokens,
      open: afterName + 1,
      angled: false,
    },);
    named.push(namedUnderKey({
      properties: propertiesOf({ body: tokens.slice(
        afterName + 2,
        afterObject - 1,
      ), },),
      key,
    },),);
  }
  return named;
}

//endregion Re-ask role at a call
