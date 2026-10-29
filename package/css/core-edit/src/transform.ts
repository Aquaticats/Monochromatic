import { isTokenWhitespace, } from '@csstools/css-tokenizer';
import {
  type CssNode,
  type CssStylesheet,
  type CssTrivia,
  isCssTrivia,
} from './node.ts';

//region Visitor contract

/**
 Replacement decision for one visited node: the same node (keep), a different
 node (replace), or a node array to splice in place; an empty array removes
 the node.
 */
export type CssVisitResult = CssNode | readonly CssNode[];

/**
 Callback deciding each node's fate. Called bottom-up: a container's children
 are transformed before the container itself is visited, so the visitor sees
 containers with already-rewritten bodies.
 */
export type CssVisitor = (node: CssNode,) => CssVisitResult;

//endregion Visitor contract

//region Accumulation

/**
 Fold state for one child-list rebuild: nodes emitted so far plus whether any
 differ from the input.
 */
type RebuildAccumulator = {
  readonly out: CssNode[];
  changed: boolean;
};

/**
 Narrows a visitor result to the splice (array) form; `Array.isArray` alone
 narrows a readonly-array union member to `any[]`.
 
 @param result - Visitor decision under test.
 
 @returns Whether the result is a node array.
 */
function isNodeArray(result: CssVisitResult,): result is readonly CssNode[] {
  return Array.isArray(result,);
}

/**
 Drops the whitespace tokens trailing a trivia run's last comment, so node
 removal eats the blank space that led into the node without eating the
 comments before it.
 
 @param trivia - Trivia run preceding a removed node.
 
 @returns Zero-or-one trimmed trivia nodes; empty when the run was whitespace-only.
 */
function trimTrailingWhitespace(trivia: CssTrivia,): readonly CssTrivia[] {
  /**
   Index one past the run's last non-whitespace token.
   */
  const keepEnd = trivia.tokens
    .findLastIndex(
      /**
       Reports whether one token must survive the trim.
       
       @param token - Trivia token.
       
       @returns Whether the token is not plain whitespace.
       */
    function isNotWhitespace(token,) {
      return !isTokenWhitespace(token,);
    },
    )
    + 1;
  if (keepEnd === 0)
    return [];
  if (keepEnd
    === trivia.tokens
      .length)
    return [trivia,];
  return [{
    kind: 'trivia',
    tokens: trivia.tokens
      .slice(
      0,
      keepEnd,
      ),
  },];
}

//endregion Accumulation

//region Transform

/**
 Rebuilds a node list immutably under a visitor, bottom-up.
 
 Containers (rules and block-bearing at-rules) get their block children
 transformed first; the visitor then sees the rebuilt container. Untouched
 subtrees keep reference identity, and when nothing changes at all the
 original array returns unchanged, giving cheap structural sharing.
 
 @param nodes - Nodes to transform.
 
 @param visit - Per-node replacement decision.
 
 @param pruneTriviaBeforeRemoved - Whether removing a node (empty-array
 result) also drops the whitespace immediately before it, so deletions do
 not leave doubled blank space; comments in that trivia survive (mirrors how
 postcss removal ate only the node's leading whitespace).
 
 @returns Transformed node list; reference-equal to the input when no node changed.
 
 @example
 ```ts
 transformNodes({
   nodes: state.root.children,
   visit: (node) => node.kind === 'atRule' && node.name === 'mixin' ? [] : node,
   pruneTriviaBeforeRemoved: true,
 });
 ```
 */
export function transformNodes({
  nodes,
  visit,
  pruneTriviaBeforeRemoved = false,
}: {
  readonly nodes: readonly CssNode[];
  readonly visit: CssVisitor;
  readonly pruneTriviaBeforeRemoved?: boolean;
},): readonly CssNode[] {
  /**
   Fold outcome across all input nodes.
   */
  const rebuilt = nodes.reduce<RebuildAccumulator>(
    /**
     Folds one input node into the rebuild.
     
     @param accumulator - Nodes emitted so far plus the change flag.
     
     @param node - Current input node.
     
     @returns Updated accumulator.
     */
    function rebuildNode(
      accumulator,
      node,
    ) {
      /**
       Node with its block children already transformed, when it has any.
       */
      const inner = rebuildContainer({
        node,
        visit,
        pruneTriviaBeforeRemoved,
      },);

      /**
       Visitor's decision for the rebuilt node.
       */
      const result = visit(inner,);

      if (isNodeArray(result,)) {
        accumulator.changed = true;
        if ((result.length === 0) && pruneTriviaBeforeRemoved) {
          /**
           Node currently last in the rebuilt list, candidate leading trivia.
           */
          const last = accumulator.out
            .at(-1,);
          if ((last !== undefined) && isCssTrivia(last,)) {
            accumulator.out
              .pop();
            accumulator.out
              .push(...trimTrailingWhitespace(last,),);
          }
        }
        accumulator.out
          .push(...result,);
        return accumulator;
      }

      if ((result !== node) || (inner !== node))
        accumulator.changed = true;
      accumulator.out
        .push(result,);
      return accumulator;
    },
    {
      out: [],
      changed: false,
    },
  );

  return rebuilt.changed ? rebuilt.out : nodes;
}

/**
 Transforms the children of a container node, returning a rebuilt container
 when they changed and the original node otherwise. Non-containers pass
 through untouched.
 
 @param node - Candidate container.
 
 @param visit - Per-node replacement decision, forwarded to the recursion.
 
 @param pruneTriviaBeforeRemoved - Removal policy, forwarded to the recursion.
 
 @returns Node with transformed block children, or the input node.
 */
function rebuildContainer({
  node,
  visit,
  pruneTriviaBeforeRemoved,
}: {
  readonly node: CssNode;
  readonly visit: CssVisitor;
  readonly pruneTriviaBeforeRemoved: boolean;
},): CssNode {
  if ((node.kind !== 'rule') && (node.kind !== 'atRule'))
    return node;
  /**
   Block owned by the container, absent on statement at-rules.
   */
  const { block, } = node;
  if (block === undefined)
    return node;

  /**
   Transformed block children.
   */
  const children = transformNodes({
    nodes: block.children,
    visit,
    pruneTriviaBeforeRemoved,
  },);
  if (children === block.children)
    return node;

  return {
    ...node,
    block: {
      ...block,
      children,
    },
  };
}

//endregion Transform

/**
 Convenience wrapper transforming a whole stylesheet, returning a fresh root
 when anything changed.
 
 @param root - Stylesheet to transform.
 
 @param visit - Per-node replacement decision.
 
 @param pruneTriviaBeforeRemoved - Removal policy for leading trivia.
 
 @returns Transformed stylesheet; reference-equal to the input when unchanged.
 
 @example
 ```ts
 transformStylesheet({ root: state.root, visit: dropMixins });
 ```
 */
export function transformStylesheet({
  root,
  visit,
  pruneTriviaBeforeRemoved = false,
}: {
  readonly root: CssStylesheet;
  readonly visit: CssVisitor;
  readonly pruneTriviaBeforeRemoved?: boolean;
},): CssStylesheet {
  /**
   Transformed top-level children.
   */
  const children = transformNodes({
    nodes: root.children,
    visit,
    pruneTriviaBeforeRemoved,
  },);
  if (children === root.children)
    return root;
  return {
    kind: 'stylesheet',
    children,
  };
}
