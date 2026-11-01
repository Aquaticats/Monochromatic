import {
  type CSSToken,
  isTokenCDC,
  isTokenCDO,
  isTokenCloseCurly,
  isTokenCloseParen,
  isTokenCloseSquare,
  isTokenComment,
  isTokenFunction,
  isTokenOpenCurly,
  isTokenOpenParen,
  isTokenOpenSquare,
  isTokenWhitespace,
} from '@csstools/css-tokenizer';

export type { CSSToken, } from '@csstools/css-tokenizer';
// Token-kind guards consumers need to interpret prelude and value slices
// without depending on the tokenizer package directly.
export {
  isTokenIdent,
  isTokenString,
  isTokenURL,
} from '@csstools/css-tokenizer';

//region Nesting

/**
 Reports whether a token opens a nested component-value scope: `{`, `(`, `[`,
 or a function token (which owns its closing `)`).
 
 @param token - Token under test.
 
 @returns Whether nesting depth increases at this token.
 
 @example
 ```ts
 isOpeningToken(tokenize({ css: '(' })[0]); // => true
 ```
 */
export function isOpeningToken(token: CSSToken,): boolean {
  return isTokenOpenCurly(token,)
    || isTokenOpenParen(token,)
    || isTokenOpenSquare(token,)
    || isTokenFunction(token,);
}

/**
 Reports whether a token closes a nested component-value scope: `}`, `)`, `]`.
 
 @param token - Token under test.
 
 @returns Whether nesting depth decreases at this token.
 
 @example
 ```ts
 isClosingToken(tokenize({ css: ')' })[0]); // => true
 ```
 */
export function isClosingToken(token: CSSToken,): boolean {
  return isTokenCloseCurly(token,)
    || isTokenCloseParen(token,)
    || isTokenCloseSquare(token,);
}

//endregion Nesting

//region Trivia

/**
 Reports whether a token carries no structural meaning for the CST layer:
 whitespace, comments, and the HTML-era CDO/CDC guards. Runs of such tokens
 collapse into a single `trivia` node so they survive stringification
 byte-exactly without participating in structure.
 
 @param token - Token under classification.
 
 @returns Whether the token belongs in a trivia run.
 
 @example
 ```ts
 isTriviaToken(tokenize({ css: '/* note *\/' })[0]); // => true
 ```
 */
export function isTriviaToken(token: CSSToken,): boolean {
  return isTokenWhitespace(token,)
    || isTokenComment(token,)
    || isTokenCDO(token,)
    || isTokenCDC(token,);
}

//endregion Trivia

//region Parsed data

/**
 Position of the parsed-data slot in the token tuple.
 */
const TOKEN_DATA_SLOT = 4;

/**
 Reads a token's parsed-data slot (index 4 of the tuple): the unescaped,
 unquoted, or numeric interpretation the tokenizer stores beside the raw
 representation.
 
 @param token - Token tuple to read.
 
 @returns Parsed data typed per the token kind.
 
 @example
 ```ts
 tokenData(atKeywordToken).value; // => 'mixin'
 ```
 */
export function tokenData<const TokenKind extends CSSToken,>(
  token: TokenKind,
): TokenKind[typeof TOKEN_DATA_SLOT] {
  return token[TOKEN_DATA_SLOT];
}

//endregion Parsed data

//region Raw text

/**
 Joins the source representation (index 1 of each token tuple) of a token
 slice back into CSS text. Representation strings are byte-exact copies of
 the input, so joining a parse's full token sequence reproduces the source.
 
 Avoids the upstream `stringify(...tokens)` spread, which risks argument-count
 limits on large stylesheets.
 
 @param tokens - Token slice to render.
 
 @returns Concatenated source text of the slice.
 
 @example
 ```ts
 rawTextOfTokens({ tokens: tokenize({ css: '.a{}' }) }); // => '.a{}'
 ```
 */
export function rawTextOfTokens({
  tokens,
}: {
  readonly tokens: readonly CSSToken[];
},): string {
  return tokens
    .map(
      /**
       Extracts one token's source representation.
       
       @param token - Token tuple.
       
       @returns Byte-exact source text of that token.
       */
    function tokenRaw(token: CSSToken,): string {
        /**
         Source representation slot of the token tuple.
         */
      const [, raw,] = token;
      return raw;
    },
    )
    .join('',);
}

//endregion Raw text
