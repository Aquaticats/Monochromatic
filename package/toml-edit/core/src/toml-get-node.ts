/**
 {@link tomlGetNode}: read the parse-time AST node at a path.
 
 @module
 */

import type { AST, } from 'toml-eslint-parser';

import { TomlPathNotFoundError, } from './errors.ts';
import { formatPath, } from './path.ts';
import {
  locateValueNode,
  NOT_LOCATED,
} from './resolve-document.ts';
import type {
  TomlEditState,
  TomlPath,
} from './types.ts';

/**
 Return the parse-time AST node at `path`.
 
 Power-user escape hatch: it returns the retained AST node of a clean
 (unmutated) node. A path created or edited by a mutation has no parse-time
 node, so {@link TomlPathNotFoundError} is thrown; {@link tomlGetValue} returns
 the current value instead.
 
 @returns Computed result (`AST.TOMLContentNode | AST.TOMLTable | readonly AST.TOMLTable[]`).
 
 @throws {@link TomlPathNotFoundError} when `path` has no clean parse-time node.
 
 @example
 ```ts
 const node = tomlGetNode({ edit, path: ['tools', 'bun',], },);
 ```
 */
export function tomlGetNode(
  {
    edit,
    path,
  }: {
    readonly edit: TomlEditState;
    readonly path: TomlPath;
  },
): AST.TOMLContentNode | AST.TOMLTable | readonly AST.TOMLTable[] {
  /**
   Structural location so a clean node's retained AST node can surface.
   */
  const located = locateValueNode({
    blocks: edit.blocks,
    path,
  },);
  if (located === NOT_LOCATED)
    throw notFound({ path, },);
  if (located.kind
    === 'value') {
    if (located.value
      .origin
      .kind
      === 'clean')
      return asContentNode({
        node: located.value
          .origin
          .astNode,
        path,
      },);
    throw notFound({ path, },);
  }
  if (located.kind
    === 'table') {
    if (located.table
      .headerOrigin
      .kind
      === 'clean')
      return asTable({
        node: located.table
          .headerOrigin
          .astNode,
        path,
      },);
    throw notFound({ path, },);
  }
  return located.tables
    .map(function each(t,) {
      if (t.headerOrigin
        .kind
        !== 'clean')
        throw notFound({ path, },);
      return asTable({
        node: t.headerOrigin
          .astNode,
        path,
      },);
    },);
}

/**
 Narrow a retained parse-time node to a {@link AST.TOMLContentNode}, throwing
 when the node kind does not match (rather than asserting the type).
 
 @param node - Retained parse-time node whose `.type` is checked at runtime.
 
 @param path - Requested path, used only to build the not-found error.
 
 @returns Content node (value, array, or inline table).
 
 @throws {@link TomlPathNotFoundError} when `node` is not a content node.
 */
function asContentNode(
  {
    node,
    path,
  }: {
    readonly node: AST.TOMLNode;
    readonly path: TomlPath;
  },
): AST.TOMLContentNode {
  if ((node.type
    === 'TOMLValue')
    || (node.type
      === 'TOMLArray')
    || (node.type
      === 'TOMLInlineTable'))
    return node;
  throw notFound({ path, },);
}

/**
 Narrow a retained parse-time node to a {@link AST.TOMLTable}, throwing when the
 node kind does not match (rather than asserting the type).
 
 @param node - Retained parse-time node whose `.type` is checked at runtime.
 
 @param path - Requested path, used only to build the not-found error.
 
 @returns Table node.
 
 @throws {@link TomlPathNotFoundError} when `node` is not a table.
 */
function asTable(
  {
    node,
    path,
  }: {
    readonly node: AST.TOMLNode;
    readonly path: TomlPath;
  },
): AST.TOMLTable {
  if (node.type
    === 'TOMLTable')
    return node;
  throw notFound({ path, },);
}

/**
 Build the not-found error for `path`.
 
 @returns Error to throw.
 */
function notFound({ path, }: { readonly path: TomlPath; },): TomlPathNotFoundError {
  return new TomlPathNotFoundError(
    `Path ${formatPath({ path, },)} has no parse-time AST node`,
  );
}
