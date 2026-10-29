import type {
  TokenAtKeyword,
  TokenCloseCurly,
  TokenOpenCurly,
  TokenSemicolon,
} from '@csstools/css-tokenizer';
import type { CSSToken, } from './token.ts';

//region Node kinds

/**
 Run of whitespace, comment, CDO, and CDC tokens between structural nodes.
 Kept as a first-class node so stringification reproduces the source
 byte-exactly and edits can decide what happens to surrounding blank space.
 */
export type CssTrivia = {
  readonly kind: 'trivia';
  /**
   Tokens of the run, in source order.
   */
  readonly tokens: readonly CSSToken[];
};

/**
 Declaration-shaped raw token run inside a block: an ident, a colon, and the
 value tokens, including the trailing semicolon when present. Values stay
 unparsed token slices; this layer never interprets them.
 */
export type CssDeclaration = {
  readonly kind: 'declaration';
  /**
   Tokens of the run, in source order, trailing semicolon included when present.
   */
  readonly tokens: readonly CSSToken[];
};

/**
 Braced block owned by a rule or an at-rule. Children follow the CSS Syntax
 section 5 unified block-contents model: declarations, nested rules, at-rules,
 and trivia may all appear in any block.
 */
export type CssBlock = {
  readonly kind: 'block';
  /**
   Opening `{` token; kept so escapes and positions round-trip.
   */
  readonly openToken: TokenOpenCurly;
  /**
   Structured contents between the braces.
   */
  readonly children: readonly CssNode[];
  /**
   Closing `}` token; strict parsing guarantees its presence.
   */
  readonly closeToken: TokenCloseCurly;
};

/**
 At-rule such as `\@import`, `\@media`, `\@mixin`, or `\@apply`. Either
 `block` (braced form) or `semicolonToken` (statement form) is present;
 a statement at-rule terminated by end-of-block or end-of-file has neither.
 */
export type CssAtRule = {
  readonly kind: 'atRule';
  /**
   At-keyword token, byte-exact including the `\@` and any escapes.
   */
  readonly atToken: TokenAtKeyword;
  /**
   Unescaped at-rule name without the leading `\@`, from tokenizer data.
   */
  readonly name: string;
  /**
   Prelude tokens between the at-keyword and the block or semicolon,
   surrounding trivia included.
   */
  readonly preludeTokens: readonly CSSToken[];
  /**
   Braced body when the at-rule has one.
   */
  readonly block?: CssBlock;
  /**
   Terminating `;` when the statement form carries one.
   */
  readonly semicolonToken?: TokenSemicolon;
};

/**
 Qualified (style) rule: selector prelude tokens plus a braced block.
 Selector text stays an uninterpreted token slice, so `&` nesting and
 relaxed nesting both pass through untouched.
 */
export type CssRule = {
  readonly kind: 'rule';
  /**
   Selector prelude tokens, surrounding trivia included.
   */
  readonly preludeTokens: readonly CSSToken[];
  /**
   Braced body of the rule.
   */
  readonly block: CssBlock;
};

/**
 Any node that may appear in stylesheet or block children.
 */
export type CssNode = CssTrivia | CssDeclaration | CssAtRule | CssRule;

/**
 Root of a parsed document.
 */
export type CssStylesheet = {
  readonly kind: 'stylesheet';
  /**
   Top-level nodes in source order.
   */
  readonly children: readonly CssNode[];
};

/**
 Immutable handle on a parsed CSS document. Every edit function returns a
 fresh state; the underlying node tree is shared by reference where unchanged.
 */
export type CssEditState = {
  readonly root: CssStylesheet;
};

//endregion Node kinds

//region Guards

/**
 Narrows a node to {@link CssAtRule}.
 
 @param node - Node under test.
 
 @returns Whether the node is an at-rule.
 
 @example
 ```ts
 isCssAtRule({ kind: 'trivia', tokens: [] }); // => false
 ```
 */
export function isCssAtRule(node: CssNode,): node is CssAtRule {
  return node.kind === 'atRule';
}

/**
 Narrows a node to {@link CssRule}.
 
 @param node - Node under test.
 
 @returns Whether the node is a qualified rule.
 
 @example
 ```ts
 isCssRule({ kind: 'trivia', tokens: [] }); // => false
 ```
 */
export function isCssRule(node: CssNode,): node is CssRule {
  return node.kind === 'rule';
}

/**
 Narrows a node to {@link CssTrivia}.
 
 @param node - Node under test.
 
 @returns Whether the node is a trivia run.
 
 @example
 ```ts
 isCssTrivia({ kind: 'trivia', tokens: [] }); // => true
 ```
 */
export function isCssTrivia(node: CssNode,): node is CssTrivia {
  return node.kind === 'trivia';
}

/**
 Narrows a node to {@link CssDeclaration}.
 
 @param node - Node under test.
 
 @returns Whether the node is a declaration run.
 
 @example
 ```ts
 isCssDeclaration({ kind: 'declaration', tokens: [] }); // => true
 ```
 */
export function isCssDeclaration(node: CssNode,): node is CssDeclaration {
  return node.kind === 'declaration';
}

//endregion Guards
