/**
 Semantic source-session construction.
 
 @module
 */

import type {
  Checker,
  Project,
} from 'typescript/sync';
import type {
  Node,
  SourceFile,
} from 'typescript/ast';

import {
  findNodeAtOffset,
  typescriptOffset,
} from './typescript-node-map.ts';

/**
 Semantic handles for one current source snapshot.
 
 @example
 ```ts
 const session = openSemanticFile({ fileName, sourceText, hasBOM: false });
 const type = session.checker.getTypeAtLocation(session.nodeAtOffset(10));
 ```
 */
export type SemanticFileSession = {
  /**
   Canonical absolute source path used by TypeScript project service.
   */
  readonly fileName: string;
  /**
   Configured or inferred project selected for source.
   */
  readonly project: Project;
  /**
   Project checker tied to active snapshot.
   */
  readonly checker: Checker;
  /**
   Source tree including current virtual overlay.
   */
  readonly sourceFile: SourceFile;
  /**
   Maps Oxlint range offset to deepest TypeScript node.
   */
  readonly nodeAtOffset: (offset: number) => Node;
};

/**
 Creates semantic handles over one project source.
 
 @param fileName - Canonical source path.
 
 @param project - Configured project containing source.
 
 @param sourceFile - Source tree from current immutable snapshot.
 
 @param hasBOM - Whether Oxlint stripped leading byte-order mark.
 
 @returns semantic project, checker, source tree, and offset mapper.
 
 @example
 ```ts
 semanticFileSession({ fileName, project, sourceFile, hasBOM: false });
 ```
 */
export function semanticFileSession({
  fileName,
  project,
  sourceFile,
  hasBOM,
}: {
  readonly fileName: string;
  readonly project: Project;
  readonly sourceFile: SourceFile;
  readonly hasBOM: boolean;
}): SemanticFileSession {
  return {
    fileName,
    project,
    checker: project.checker,
    sourceFile,
    nodeAtOffset(offset: number,): Node {
      return findNodeAtOffset({
        sourceFile,
        offset: typescriptOffset({
          offset,
          hasBOM,
        },),
      },);
    },
  };
}
