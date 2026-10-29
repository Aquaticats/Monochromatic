/**
 Progressive type evidence for spread and copy rules.

 Rules first decide what syntax alone proves. When syntax cannot tell an array
 from a string, typed array, or iterator, they ask this module, which consults
 the TypeScript 7 semantic bridge owned by the readonly plugin. A failure that is
 about one source, such as a file outside every configured project, degrades to
 `untyped`, so the rule falls back to its type-free report instead of guessing.
 A bridge that cannot start is not about one source: every later answer would be
 degraded the same way and nothing would say so, so it stops the lint instead. The
 bridge is started when the plugin loads, before Oxlint reserves memory for its
 workers, so a host with many cores does not take that path for want of a fork.

 @module
 */

import type {
  Context,
  ESTree,
} from '@oxlint/plugins';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { SemanticBridgeError, } from '@monochromatic-dev/oxlint-plugin-prefer-readonly-parameter-type/ts/prefer-readonly-parameter-types/semantic-bridge-error.ts';
import {
  openSemanticFile,
  type SemanticFileSession,
} from '@monochromatic-dev/oxlint-plugin-prefer-readonly-parameter-type/ts/prefer-readonly-parameter-types/typescript-sync-adapter.ts';
import type { Node, } from 'typescript/ast';

import {
  bridgeStartFailure,
  bridgeUnavailableError,
  NO_START_FAILURE,
} from './bridge-start.ts';
import {
  valueKindOfType,
  type ValueKind,
} from './type-kind.ts';

/**
 Module logger for semantic evidence decisions.
 */
const l = tagged({ tag: 'no-restricted-syntax/spread-evidence', },);

/**
 Sentinel for an ESTree range with no TypeScript node of identical extent.
 */
const NO_EXACT_NODE: unique symbol = Symbol('semantic tree lacks a node covering exactly the linted source range',);

/**
 Walks up from the deepest TypeScript node at a range start to the node whose
 extent matches the range exactly.

 @param session - Semantic session of the linted file.

 @param node - ESTree expression whose TypeScript counterpart is wanted.

 @param hasBOM - Whether Oxlint stripped a byte-order mark, shifting TypeScript offsets by one.

 @returns TypeScript node with the same extent, or {@link NO_EXACT_NODE}.

 @throws {@link SemanticBridgeError} when no TypeScript node covers the start offset.

 @example
 ```ts
 exactTypeScriptNode({ session, node: call, hasBOM: false });
 ```
 */
function exactTypeScriptNode(
  {
    session,
    node,
    hasBOM,
  }: ForeignBorrowed<{
    readonly session: SemanticFileSession;
    readonly node: ESTree.Expression;
    readonly hasBOM: boolean;
  }>,
): Node | typeof NO_EXACT_NODE {
  /**
   Byte-order-mark shift between Oxlint and TypeScript offsets.
   */
  const shift = hasBOM ? 1 : 0;
  /**
   Expected TypeScript start and end of the expression.
   */
  const wanted = {
    start: node.start + shift,
    end: node.end + shift,
  };
  /**
   Ancestor cursor starting at the deepest node covering the start offset.
   */
  const cursor: { current: Node; } = { current: session.nodeAtOffset(node.start,), };
  while (cursor.current !== session.sourceFile) {
    /**
     Trivia-free start of the current ancestor.
     */
    const start = cursor.current
      .getStart(session.sourceFile,);
    if ((start === wanted.start) && (cursor.current
      .end
      === wanted.end))
      return cursor.current;
    if ((start < wanted.start) || (cursor.current
      .end
      > wanted.end))
      return NO_EXACT_NODE;
    cursor.current = cursor.current
      .parent;
  }
  return NO_EXACT_NODE;
}

/**
 Opens the linted file in the semantic bridge and finds the expression's node.

 @param context - Rule context of the linted file.

 @param node - Expression to locate.

 @returns Session plus matching node, {@link NO_EXACT_NODE}, or the bridge failure.

 @throws Rethrows failures other than {@link SemanticBridgeError}, which mean a bridge defect.

 @example
 ```ts
 const located = locateSemanticNode({ context, node: call });
 ```
 */
function locateSemanticNode(
  {
    context,
    node,
  }: ForeignBorrowed<{
    readonly context: Context;
    readonly node: ESTree.Expression;
  }>,
):
  | {
    readonly session: SemanticFileSession;
    readonly semanticNode: Node | typeof NO_EXACT_NODE
  }
    | SemanticBridgeError
  {
  try {
    /**
     Semantic session over the current source text.
     */
    const session = openSemanticFile({
      fileName: context.physicalFilename,
      sourceText: context.sourceCode
        .text,
      hasBOM: context.sourceCode
        .hasBOM,
    },);
    return {
      session,
      semanticNode: exactTypeScriptNode({
        session,
        node,
        hasBOM: context.sourceCode
          .hasBOM,
      },),
    };
  }
  catch (error) {
    if (error instanceof SemanticBridgeError)
      return error;
    tagged({
      tag: locateSemanticNode.name,
      l,
    },)
      .error(`semantic bridge failed unexpectedly for ${context.physicalFilename}: ${String(error,)}`,);
    throw error;
  }
}

/**
 Classifies the value an expression evaluates to, using the semantic bridge
 when it can answer and `untyped` whenever it cannot.

 @param context - Rule context of the linted file.

 @param node - Expression whose value kind decides the report.

 @returns Value kind, `untyped` when no semantic type is available for this source.

 @throws {@link SemanticBridgeError} with reason `api-unavailable` when the native TypeScript child cannot start,
 and rethrows failures other than {@link SemanticBridgeError}, which mean a bridge defect.

 @example
 ```ts
 if (expressionValueKind({ context, node: call }) === 'array') report();
 ```
 */
export function expressionValueKind(
  {
    context,
    node,
  }: ForeignBorrowed<{
    readonly context: Context;
    readonly node: ESTree.Expression;
  }>,
): ValueKind {
  /**
   Function-tagged evidence logger.
   */
  const rl = tagged({
    tag: expressionValueKind.name,
    l,
  },);
  /**
   Why the bridge could not start when the plugin loaded. It is not asked again, because a second spawn
   would only repeat the failure.
   */
  const startFailure = bridgeStartFailure();
  /**
   Located semantic node, or the bridge failure explaining why none exists.
   */
  const located = (startFailure === NO_START_FAILURE)
    ? locateSemanticNode({
      context,
      node,
    },)
    : startFailure;
  if (located instanceof SemanticBridgeError) {
    if (located.reason === 'api-unavailable') {
      rl.error(`semantic bridge unavailable for ${context.physicalFilename}: ${located.message}`,);
      throw bridgeUnavailableError({
        reason: located.reason,
        message: located.message,
        fileName: context.physicalFilename,
      },);
    }
    rl.debug(`no semantic evidence for ${context.physicalFilename} (${located.reason}); using type-free report`,);
    return 'untyped';
  }
  if (located.semanticNode === NO_EXACT_NODE) {
    rl.warn(`no TypeScript node spans ${String(node.start,)}..${String(node.end,)} in ${context.physicalFilename}; using type-free report`,);
    return 'untyped';
  }
  /**
   Checker type of the expression.
   */
  const type = located.session
    .checker
    .getTypeAtLocation(located.semanticNode,);
  if (type === undefined) {
    rl.debug(`checker has no type at ${String(node.start,)} in ${context.physicalFilename}`,);
    return 'untyped';
  }
  /**
   Value kind derived from the checker type.
   */
  const kind = valueKindOfType({
    checker: located.session
      .checker,
    type,
    depth: 0,
  },);
  rl.debug(`${context.physicalFilename}:${String(node.start,)} classified as ${kind}`,);
  return kind;
}
