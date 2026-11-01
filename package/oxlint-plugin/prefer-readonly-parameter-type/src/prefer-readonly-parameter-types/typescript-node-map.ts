/**
 Position mapping between Oxlint ESTree ranges and TypeScript 7 AST nodes.
 
 @module
 */

import type {
  Node,
  SourceFile,
} from 'typescript/ast';

import { SemanticBridgeError, } from './semantic-bridge-error.ts';

/**
 Sentinel for traversal before any containing node is found.
 */
const NO_NODE: unique symbol = Symbol('no TypeScript node found at source offset',);

/**
 Converts Oxlint UTF-16 offset to TypeScript source offset.
 
 @param offset - Zero-based Oxlint range offset.
 
 @param hasBOM - Whether host stripped leading byte-order mark from source text.
 
 @returns Offset into TypeScript source text including restored byte-order mark.
 
 @example
 ```ts
 typescriptOffset({ offset: 4, hasBOM: true }); // 5
 ```
 */
export function typescriptOffset({
  offset,
  hasBOM,
}: {
  readonly offset: number;
  readonly hasBOM: boolean;
},): number {
  return hasBOM ? offset + 1 : offset;
}

/**
 Finds deepest TypeScript node containing UTF-16 source offset.
 
 Uses explicit work stack because source files are linear-input trees with
 unbounded practical depth.
 
 @param sourceFile - TypeScript source tree to search.
 
 @param offset - UTF-16 offset including any leading byte-order mark.
 
 @returns deepest containing node.
 
 @throws {@link SemanticBridgeError} when no node contains offset.
 
 @example
 ```ts
 const node = findNodeAtOffset({ sourceFile, offset: 10 });
 ```
 */
export function findNodeAtOffset({
  sourceFile,
  offset,
}: {
  readonly sourceFile: SourceFile;
  readonly offset: number;
},): Node {
  /**
   Mutable work stack for bounded structural traversal.
   */
  const stack: Node[] = [sourceFile,];
  /**
   Deepest matching node retained as traversal descends.
   */
  const found: { current: Node | typeof NO_NODE; } = { current: NO_NODE, };

  while (stack.length > 0) {
    /**
     Next candidate from depth-first work stack.
     */
    const candidate = stack.pop();
    if (candidate === undefined)
      continue;
    /**
     Candidate's trivia-free source start.
     */
    const start = candidate.getStart(sourceFile,);
    if ((offset < start) || (offset >= candidate.end))
      continue;
    found.current = candidate;
    candidate.forEachChild(function collectChild(child,): undefined {
      stack.push(child,);
      return undefined;
    },);
  }

  if (found.current === NO_NODE) {
    throw new SemanticBridgeError({
      reason: 'node-not-found',
      message: `No TypeScript node contains offset ${String(offset,)} in ${sourceFile.fileName}.`,
    },);
  }
  return found.current;
}
