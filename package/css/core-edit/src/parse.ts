import { tokenize, } from '@csstools/css-tokenizer';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { StringCss, } from './brand.ts';
import { CssParseError, } from './errors.ts';
import type { CssEditState, } from './node.ts';
import { consumeContents, } from './parse-contents.ts';

/**
 Module logger for the parse entry point.
 */
const l = tagged({ tag: 'parse', },);

/**
 Parses a CSS document into an immutable, byte-preserving CST.
 
 Tokenization uses the `\@csstools/css-tokenizer` spec tokenizer; structure
 follows the CSS Syntax section 5 unified block-contents model, so
 declarations, nested rules (with or without `&`), and at-rules mix freely in
 any block, and unknown at-rules such as `\@mixin` parse like known ones.
 Preludes and declaration values stay uninterpreted token slices;
 stringifying an unedited state reproduces the source byte-exactly.
 
 Strict on malformed input: tokenizer-level errors (unterminated strings or
 comments, bad urls) and structure-level errors (unclosed or stray braces,
 rules without blocks) all throw.
 
 @param source - Branded CSS source string.
 
 @returns Immutable edit state holding the parsed stylesheet.
 
 @throws CssParseError on malformed input.
 
 @example
 ```ts
 const state = parseCss({ source: '.btn { \@apply --card; }' as StringCss });
 state.root.children[0]?.kind; // => 'rule'
 ```
 */
export function parseCss({
  source,
}: {
  readonly source: StringCss;
},): CssEditState {
  tagged({
    tag: parseCss.name,
    l,
  },)
    .trace(`parsing ${String(source.length,)} characters`,);

  /**
   Tokenizer-reported failures reduced to the fields strict mode throws with.
   */
  const tokenizerErrors: {
    readonly message: string;
    readonly offset: number;
  }[] = [];
  /**
   Full token array, EOF token included.
   */
  const tokens = tokenize(
    { css: source, },
    {
      /**
       Records one tokenizer failure for the strict-mode throw.
       
       @param error - Tokenizer diagnostic, read-only view.
       */
      onParseError: function collectTokenizerError(error: {
        readonly message: string;
        readonly sourceStart: number;
      },): void {
        tokenizerErrors.push({
          message: error.message,
          offset: error.sourceStart,
        },);
      },
    },
  );

  /**
   First tokenizer error, when any; parsing is strict, so one is fatal.
   */
  const [firstError,] = tokenizerErrors;
  if (firstError !== undefined)
    throw new CssParseError({
      message: `tokenizer error: ${firstError.message}`,
      offset: firstError.offset,
    },);

  /**
   Structured top-level contents of the document.
   */
  const contents = consumeContents({
    tokens,
    start: 0,
    insideBlock: false,
  },);

  return {
    root: {
      kind: 'stylesheet',
      children: contents.children,
    },
  };
}
