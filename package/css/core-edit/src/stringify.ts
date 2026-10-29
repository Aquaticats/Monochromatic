import type {
  CssBlock,
  CssEditState,
  CssNode,
} from './node.ts';
import type { CSSToken, } from './token.ts';

//region Accumulation

/**
 Appends the source representation of each token in a slice to the
 accumulator.
 
 @param tokens - Token slice to render.
 
 @param out - Shared raw-string accumulator.
 */
function pushTokenRaws({
  tokens,
  out,
}: {
  readonly tokens: readonly CSSToken[];
  readonly out: string[];
},): void {
  for (const token of tokens) {
    /**
     Source representation slot of the token tuple.
     */
    const [, raw,] = token;
    out.push(raw,);
  }
}

/**
 Appends one block's source text: opening brace, children, closing brace.
 
 @param block - Block to render.
 
 @param out - Shared raw-string accumulator.
 */
function pushBlock({
  block,
  out,
}: {
  readonly block: CssBlock;
  readonly out: string[];
},): void {
  /**
   Opening brace representation.
   */
  const [, openRaw,] = block.openToken;
  out.push(openRaw,);
  pushNodes({
    nodes: block.children,
    out,
  },);
  /**
   Closing brace representation.
   */
  const [, closeRaw,] = block.closeToken;
  out.push(closeRaw,);
}

/**
 Appends one node's source text.
 
 @param node - Node to render.
 
 @param out - Shared raw-string accumulator.
 */
function pushNode({
  node,
  out,
}: {
  readonly node: CssNode;
  readonly out: string[];
},): void {
  if ((node.kind === 'trivia') || (node.kind === 'declaration')) {
    pushTokenRaws({
      tokens: node.tokens,
      out,
    },);
    return;
  }

  if (node.kind === 'rule') {
    pushTokenRaws({
      tokens: node.preludeTokens,
      out,
    },);
    pushBlock({
      block: node.block,
      out,
    },);
    return;
  }

  /**
   At-keyword representation, byte-exact including the `@`.
   */
  const [, atRaw,] = node.atToken;
  out.push(atRaw,);
  pushTokenRaws({
    tokens: node.preludeTokens,
    out,
  },);
  if (node.block !== undefined) {
    pushBlock({
      block: node.block,
      out,
    },);
    return;
  }
  if (node.semicolonToken !== undefined) {
    /**
     Statement terminator representation.
     */
    const [, semicolonRaw,] = node.semicolonToken;
    out.push(semicolonRaw,);
  }
}

/**
 Appends a node list's source text in order.
 
 @param nodes - Nodes to render.
 
 @param out - Shared raw-string accumulator.
 */
function pushNodes({
  nodes,
  out,
}: {
  readonly nodes: readonly CssNode[];
  readonly out: string[];
},): void {
  for (const node of nodes) {
    pushNode({
      node,
      out,
    },);
  }
}

//endregion Accumulation

//region Entry points

/**
 Renders a node list back to source text in order. One flat accumulator and
 a single join keep large documents allocation-light.
 
 @param nodes - Nodes to render.
 
 @returns Concatenated source text.
 
 @example
 ```ts
 stringifyNodes({ nodes: state.root.children });
 ```
 */
export function stringifyNodes({
  nodes,
}: {
  readonly nodes: readonly CssNode[];
},): string {
  /**
   Raw-string accumulator shared by the whole render.
   */
  const out: string[] = [];
  pushNodes({
    nodes,
    out,
  },);
  return out.join('',);
}

/**
 Renders a full edit state back to CSS source. For a state that has not been
 edited since {@link parseCss}, output is byte-identical to the input; edited
 regions render from their new nodes while untouched regions keep their
 original bytes.
 
 @param state - Edit state to render.
 
 @returns CSS source text.
 
 @example
 ```ts
 stringifyCss({ state: parseCss({ source, }) }) === source; // => true
 ```
 */
export function stringifyCss({
  state,
}: {
  readonly state: CssEditState;
},): string {
  return stringifyNodes({
    nodes: state.root
      .children,
  },);
}

//endregion Entry points
