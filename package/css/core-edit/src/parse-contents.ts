import {
  isTokenAtKeyword,
  isTokenCloseCurly,
  isTokenEOF,
  isTokenOpenCurly,
  isTokenSemicolon,
  type TokenCloseCurly,
} from '@csstools/css-tokenizer';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import { CssParseError, } from './errors.ts';
import type {
  CssAtRule,
  CssBlock,
  CssNode,
  CssRule,
} from './node.ts';
import { classifyRun, } from './parse-classify.ts';
import {
  type CSSToken,
  isClosingToken,
  isOpeningToken,
  isTriviaToken,
  tokenData,
} from './token.ts';

//region Step results

/**
 One consumed node plus the index where consumption stopped.
 */
type ConsumedNode<NodeKind extends CssNode,> = {
  readonly node: NodeKind;
  readonly nextIndex: number;
};

/**
 Contents of a stylesheet or block plus where consumption stopped; the
 closing token is present exactly when consuming inside a block.
 */
export type ConsumedContents = {
  readonly children: readonly CssNode[];
  readonly nextIndex: number;
  readonly closeToken?: TokenCloseCurly;
};

//endregion Step results

//region Block consumer

/**
 Consumes a braced block starting at its `{` token by delegating the interior
 to {@link consumeContents} in block mode.
 
 @param tokens - Full token array of the document.
 
 @param openIndex - Index of the block's `{` token.
 
 @returns Block node and the index after its `}`.
 
 @throws CssParseError when the block never closes.
 */
function consumeBlock({
  tokens,
  openIndex,
}: {
  readonly tokens: readonly CSSToken[];
  readonly openIndex: number;
},): {
  readonly block: CssBlock;
  readonly nextIndex: number;
} {
  /**
   Opening `{` token of the block.
   */
  const openToken = tokens[openIndex];
  if ((openToken === undefined) || (!isTokenOpenCurly(openToken,)))
    throw new CssParseError({
      message: 'block consumer called off an opening curly token',
      offset: openToken === undefined ? 0 : openToken[2],
    },);

  /**
   Structured interior of the block.
   */
  const contents = consumeContents({
    tokens,
    start: openIndex + 1,
    insideBlock: true,
  },);

  return {
    block: {
      kind: 'block',
      openToken,
      children: contents.children,
      closeToken: nonNullishOrThrow(contents.closeToken,),
    },
    nextIndex: contents.nextIndex,
  };
}

//endregion Block consumer

//region At-rule consumer

/**
 Consumes an at-rule starting at its at-keyword token: prelude tokens up to a
 depth-zero `;` (statement form), `{` (block form via {@link consumeBlock}),
 or an enclosing block's `}` / end of input (statement form without a
 terminator).
 
 @param tokens - Full token array of the document.
 
 @param start - Index of the at-keyword token.
 
 @returns At-rule node and the index where consumption stopped.
 
 @throws CssParseError on unbalanced closing tokens inside the prelude.
 */
function consumeAtRule({
  tokens,
  start,
}: {
  readonly tokens: readonly CSSToken[];
  readonly start: number;
},): ConsumedNode<CssAtRule> {
  /**
   At-keyword token opening the rule.
   */
  const atToken = tokens[start];
  if ((atToken === undefined) || (!isTokenAtKeyword(atToken,)))
    throw new CssParseError({
      message: 'at-rule consumer called off an at-keyword token',
      offset: atToken === undefined ? 0 : atToken[2],
    },);

  /**
   Parsed data of the at-keyword token, holding the unescaped name.
   */
  const atData = tokenData(atToken,);

  return (function scanAtRule(): ConsumedNode<CssAtRule> {
    /**
     Nesting depth relative to the prelude's own level.
     */
    let depth = 0;
    /**
     Scan cursor over prelude tokens.
     */
    let index = start + 1;

    while (index < tokens.length) {
    /**
     Token at the scan cursor.
     */
      const token = tokens[index];
      if ((token === undefined) || isTokenEOF(token,)
        || ((depth === 0) && isTokenCloseCurly(token,)))
        break;

      if (depth === 0) {
        if (isTokenSemicolon(token,)) {
          return {
            node: {
              kind: 'atRule',
              atToken,
              name: atData.value,
              preludeTokens: tokens.slice(
                start + 1,
                index,
              ),
              semicolonToken: token,
            },
            nextIndex: index + 1,
          };
        }
        if (isTokenOpenCurly(token,)) {
        /**
         Braced body of the at-rule.
         */
          const consumed = consumeBlock({
            tokens,
            openIndex: index,
          },);
          return {
            node: {
              kind: 'atRule',
              atToken,
              name: atData.value,
              preludeTokens: tokens.slice(
                start + 1,
                index,
              ),
              block: consumed.block,
            },
            nextIndex: consumed.nextIndex,
          };
        }
      }

    // Depth bookkeeping mirrors classifyRun: function/paren/square tokens
    // nest; their closers un-nest. Curly handling above only fires at depth 0.
      if (isOpeningToken(token,))
        depth += 1;
      else if (isClosingToken(token,)) {
        depth -= 1;
        if (depth < 0)
          throw new CssParseError({
            message: 'unbalanced closing token in at-rule prelude',
            offset: token[2],
          },);
      }
      index += 1;
    }
    return {
      node: {
        kind: 'atRule',
        atToken,
        name: atData.value,
        preludeTokens: tokens.slice(
          start + 1,
          index,
        ),
      },
      nextIndex: index,
    };
  })();
}

//endregion At-rule consumer

//region Contents consumer

/**
 Consumes stylesheet or block contents per the CSS Syntax section 5 unified
 model: trivia runs, at-rules, declarations, and qualified rules may all
 appear; {@link classifyRun} decides between the latter two.
 
 @param tokens - Full token array of the document.
 
 @param start - Index of the first content token.
 
 @param insideBlock - Whether a `}` legitimately ends these contents.
 
 @returns Children in source order, the stop index, and the closing token
 when inside a block.
 
 @throws CssParseError on a stray `}` at top level or an unclosed block.
 
 @example
 ```ts
 consumeContents({ tokens, start: 0, insideBlock: false });
 // => { children: [...], nextIndex: 12 }
 ```
 */
export function consumeContents({
  tokens,
  start,
  insideBlock,
}: {
  readonly tokens: readonly CSSToken[];
  readonly start: number;
  readonly insideBlock: boolean;
},): ConsumedContents {
  return (function scanContents(): ConsumedContents {
  /**
   Accumulated child nodes in source order.
   */
    const children: CssNode[] = [];
  /**
   Scan cursor over the token array.
   */
    let index = start;

    while (index < tokens.length) {
    /**
     Token at the scan cursor.
     */
      const token = tokens[index];

      if ((token === undefined) || isTokenEOF(token,))
        break;

      if (isTokenCloseCurly(token,)) {
        if (!insideBlock)
          throw new CssParseError({
            message: 'stray closing brace at top level',
            offset: token[2],
          },);
        return {
          children,
          nextIndex: index + 1,
          closeToken: token,
        };
      }

      if (isTriviaToken(token,)) {
      /**
       Exclusive end of the trivia run.
       */
        let runEnd = index + 1;
      /**
       Token at the trivia scan cursor, refreshed as the run advances.
       */
        let runToken = tokens[runEnd];
        while ((runToken !== undefined) && isTriviaToken(runToken,)) {
          runEnd += 1;
          runToken = tokens[runEnd];
        }
        children.push({
          kind: 'trivia',
          tokens: tokens.slice(
            index,
            runEnd,
          ),
        },);
        index = runEnd;
        continue;
      }

      if (isTokenAtKeyword(token,)) {
      /**
       Consumed at-rule step.
       */
        const consumed = consumeAtRule({
          tokens,
          start: index,
        },);
        children.push(consumed.node,);
        index = consumed.nextIndex;
        continue;
      }

    /**
     Declaration-versus-rule decision for this run.
     */
      const classified = classifyRun({
        tokens,
        start: index,
      },);

      if (classified.outcome === 'declaration') {
        children.push({
          kind: 'declaration',
          tokens: tokens.slice(
            index,
            classified.endExclusive,
          ),
        },);
        index = classified.endExclusive;
        continue;
      }

    /**
     Consumed block of the qualified rule.
     */
      const consumed = consumeBlock({
        tokens,
        openIndex: classified.blockOpenIndex,
      },);
    /**
     Qualified rule assembled from prelude slice and block.
     */
      const rule: CssRule = {
        kind: 'rule',
        preludeTokens: tokens.slice(
          index,
          classified.blockOpenIndex,
        ),
        block: consumed.block,
      };
      children.push(rule,);
      index = consumed.nextIndex;
    }
  /**
   EOF token at the terminal cursor, when tokenizer supplied one.
   */
    const terminalToken = tokens[index];
    if (insideBlock)
      throw new CssParseError({
        message: 'block reached end of input without its closing brace',
        offset: terminalToken === undefined ? 0 : terminalToken[2],
      },);
    return {
      children,
      nextIndex: index,
    };
  })();
}

//endregion Contents consumer
