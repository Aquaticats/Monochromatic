/**
 Deterministic coverage driver: exercises every exported function and its error
 paths with fixed inputs, so the V8 coverage it produces is reproducible. Run
 under `NODE_V8_COVERAGE` by the `fuzz:coverage` task, then summarized by
 `coverage-report.ts`.
 
 @module
 */

import {
  asCssSource,
  type CssAtRule,
  type CssNode,
  type CssVisitResult,
  isClosingToken,
  isCssAtRule,
  isCssDeclaration,
  isCssRule,
  isCssTrivia,
  isOpeningToken,
  isTokenIdent,
  isTokenString,
  isTokenURL,
  isTriviaToken,
  parseCss,
  rawTextOfTokens,
  stringifyCss,
  stringifyNodes,
  tokenData,
  transformNodes,
  transformStylesheet,
} from '@monochromatic-dev/css-core-edit/ts';

//region Helpers

/**
 Runs a thunk that is expected to throw, swallowing the error so the driver
 keeps exercising remaining paths. Re-throws anything that is not an `Error`.
 
 @param thunk - Operation expected to throw.
 
 @example
 ```ts
 swallow(function bad() { parseCss({ source: asCssSource('}') }); });
 ```
 */
function swallow(thunk: () => void,): void {
  try {
    thunk();
  }
  catch (error: unknown) {
    if (!(Error.isError(error,)))
      throw error;
  }
}

//endregion Helpers

//region Visitors

/**
 Keeps every node, proving the identity transform preserves reference
 identity.
 
 @param node - Visited node.
 
 @returns Same node.
 
 @example
 ```ts
 transformStylesheet({ root, visit: keep }) === root; // => true
 ```
 */
function keep(node: CssNode,): CssVisitResult {
  return node;
}

/**
 Replaces each declaration with a fresh equal node, driving the changed
 single-node path.
 
 @param node - Visited node.
 
 @returns Fresh declaration, or the node untouched.
 
 @example
 ```ts
 transformStylesheet({ root, visit: replaceDeclarations });
 ```
 */
function replaceDeclarations(node: CssNode,): CssVisitResult {
  return isCssDeclaration(node,)
    ? {
      kind: 'declaration' as const,
      tokens: node.tokens,
    }
    : node;
}

/**
 Doubles each declaration, driving the non-empty splice path.
 
 @param node - Visited node.
 
 @returns Two-node splice, or the node untouched.
 
 @example
 ```ts
 transformNodes({ nodes, visit: spliceDeclarations });
 ```
 */
function spliceDeclarations(node: CssNode,): CssVisitResult {
  return isCssDeclaration(node,)
    ? [
      node,
      node,
    ]
    : node;
}

/**
 Removes every declaration, driving prune-removal over the leading trivia.
 
 @param node - Visited node.
 
 @returns Empty splice for declarations; the node untouched otherwise.
 
 @example
 ```ts
 transformStylesheet({ root, visit: dropDeclarations, pruneTriviaBeforeRemoved: true });
 ```
 */
function dropDeclarations(node: CssNode,): CssVisitResult {
  return isCssDeclaration(node,) ? [] : node;
}

/**
 Removes declarations whose raw text starts with `b`, driving removal where
 the previously emitted node is a declaration rather than trivia.
 
 @param node - Visited node.
 
 @returns Empty splice for matching declarations; the node untouched otherwise.
 
 @example
 ```ts
 transformStylesheet({ root, visit: dropSecondDeclaration, pruneTriviaBeforeRemoved: true });
 ```
 */
function dropSecondDeclaration(node: CssNode,): CssVisitResult {
  if (!isCssDeclaration(node,))
    return node;
  /**
   Raw source text of the declaration run.
   */
  const raw = rawTextOfTokens({ tokens: node.tokens, },);
  return raw.startsWith('b',) ? [] : node;
}

/**
 Removes every qualified rule, driving removal without pruning.
 
 @param node - Visited node.
 
 @returns Empty splice for rules; the node untouched otherwise.
 
 @example
 ```ts
 transformNodes({ nodes, visit: dropRules });
 ```
 */
function dropRules(node: CssNode,): CssVisitResult {
  return isCssRule(node,) ? [] : node;
}

//endregion Visitors

//region Exercise

/**
 Document exercising every structural shape: trivia (comments, CDO/CDC),
 statement and block at-rules, an unknown at-rule, relaxed nesting with the
 restart-as-rule reclassification, a custom property keeping a `{}` value,
 function and url tokens, and a statement at-rule ended by its block.
 */
const MAIN_SOURCE = asCssSource([
  '<!-- --> /* head */',
  "@import url('reset.css') layer(base);",
  '@media (min-width: 640px) { .card { color: rgb(0 0 0 / 50%); } }',
  '@mixin --chip { padding: 1px; span:hover { top: 0; } }',
  ':root { --raw: { nested: token }; }',
  '.btn[data-x="y"] { background: url(a.png); margin: calc((1px + 2px) * 3) }',
  '.z { @apply --chip }',
].join('\n',),);

/**
 Exercises parsing, stringification, token and node helpers, and the
 transform layer across keep, replace, splice, and prune-removal paths.
 
 @throws Error when the untouched round-trip is not byte-identical, which
 would make every downstream coverage number meaningless.
 
 @example
 ```ts
 exercise();
 ```
 */
function exercise(): void {
  /**
   Parsed main document.
   */
  const state = parseCss({ source: MAIN_SOURCE, },);
  if (stringifyCss({ state, },) !== MAIN_SOURCE)
    throw new Error('coverage driver round-trip diverged; driver inputs are stale',);
  /**
   Top-level children of the main document.
   */
  const topChildren = state.root
    .children;
  stringifyNodes({ nodes: topChildren, },);

  // Node guards over the top-level children.
  for (const child of topChildren) {
    isCssAtRule(child,);
    isCssRule(child,);
    isCssTrivia(child,);
    isCssDeclaration(child,);
  }

  // Token helpers over the leading trivia run and the import prelude.
  /**
   Leading trivia run: CDO, CDC, whitespace, and the head comment.
   */
  const [lead,] = topChildren;
  if ((lead !== undefined) && isCssTrivia(lead,)) {
    rawTextOfTokens({ tokens: lead.tokens, },);
    for (const token of lead.tokens) {
      isTriviaToken(token,);
      isOpeningToken(token,);
      isClosingToken(token,);
    }
  }
  /**
   Import at-rule, first structural child.
   */
  const importRule = topChildren.find(function firstAtRule(child: CssNode,): child is CssAtRule {
    return isCssAtRule(child,);
  },);
  if (importRule !== undefined) {
    tokenData(importRule.atToken,);
    for (const token of importRule.preludeTokens) {
      isTokenIdent(token,);
      isTokenString(token,);
      isTokenURL(token,);
    }
  }

  // Transform layer: keep (reference identity), replace, splice, and removals.
  /**
   Identity-transformed root, expected reference-equal to the input.
   */
  const kept = transformStylesheet({
    root: state.root,
    visit: keep,
  },);
  if (kept !== state.root)
    throw new Error('identity transform lost structural sharing',);

  transformStylesheet({
    root: state.root,
    visit: replaceDeclarations,
  },);

  transformNodes({
    nodes: topChildren,
    visit: spliceDeclarations,
  },);

  /**
   Pruning fixture: whitespace-only, comment-then-whitespace, and
   comment-only runs before removed declarations.
   */
  const pruneState = parseCss({
    source: asCssSource(
      '.a { color: red; } .b { /* keep */ color: red; } .c {/* c */color: red;}',
    ),
  },);
  transformStylesheet({
    root: pruneState.root,
    visit: dropDeclarations,
    pruneTriviaBeforeRemoved: true,
  },);
  /**
   Adjacent-declaration fixture: removal where the preceding emitted node is
   a declaration, not trivia.
   */
  const adjacentState = parseCss({ source: asCssSource('.d { a:1;b:2 }',), },);
  transformStylesheet({
    root: adjacentState.root,
    visit: dropSecondDeclaration,
    pruneTriviaBeforeRemoved: true,
  },);
  // Removal without pruning.
  transformNodes({
    nodes: pruneState.root
      .children,
    visit: dropRules,
  },);

  // Edge shapes: top-level declaration ending at EOF, statement at-rule
  // without a terminator at EOF.
  parseCss({ source: asCssSource('color: red',), },);
  parseCss({ source: asCssSource('@apply --chip',), },);

  // Error paths.
  swallow(function tokenizerError() {
    parseCss({ source: asCssSource('/* open',), },);
  },);
  swallow(function strayClose() {
    parseCss({ source: asCssSource('}',), },);
  },);
  swallow(function unclosedBlock() {
    parseCss({ source: asCssSource('.a {',), },);
  },);
  swallow(function unbalancedRun() {
    parseCss({ source: asCssSource(') {}',), },);
  },);
  swallow(function unbalancedPrelude() {
    parseCss({ source: asCssSource('@media ) {}',), },);
  },);
  swallow(function semicolonPrelude() {
    parseCss({ source: asCssSource('.a ; {}',), },);
  },);
  swallow(function preludeHitsBlockEnd() {
    parseCss({ source: asCssSource('.b { .c }',), },);
  },);
  swallow(function preludeHitsEof() {
    parseCss({ source: asCssSource('.a',), },);
  },);
}

//endregion Exercise

exercise();
