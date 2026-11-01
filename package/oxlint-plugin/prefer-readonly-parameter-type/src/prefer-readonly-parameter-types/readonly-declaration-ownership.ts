/**
 Eager source ownership for writable semantic declarations.
 
 @module
 */

import type { Node, } from 'typescript/ast';
import type { Project, } from 'typescript/sync';

import type { WritableDeclarationOwner, } from './readonly-classification-model.ts';
import { isWorkspaceSourceFileName, } from './workspace-source-path.ts';

/**
 Classifies one resolved declaration by source ownership.
 
 A path outside every installed-module directory is workspace source whatever the program's
 library flag says. A workspace package reached through a `node_modules` symbolic link has such
 a path and is flagged as an external-library module, and the effect analysis already treats it
 as owned source it can inspect (`isOwnedSource` in `shared-analysis-key.ts`,
 `indexedSourceFileMap`), so the owner here agrees with it. A path inside an installed-module
 directory stays external-library even when the program does not flag it.
 
 @param declaration - Semantic declaration resolved in current snapshot.
 
 @param project - Project owning declaration source metadata.
 
 @returns workspace,
 external-library,
 or default-library ownership.
 
 @example
 ```ts
 writableDeclarationOwner({ declaration, project });
 ```
 */
function writableDeclarationOwner({
  declaration,
  project,
}: {
  readonly declaration: Node;
  readonly project: Project;
}): Exclude<WritableDeclarationOwner, 'unresolved'> {
  /**
   Source containing exact writable declaration.
   */
  const sourceFile = declaration.getSourceFile();
  /**
   Program classifying library membership for current source.
   */
  const { program, } = project;
  if (program
    .isSourceFileDefaultLibrary(sourceFile,))
    return 'default-library';
  if (!isWorkspaceSourceFileName(sourceFile.fileName,))
    return 'external-library';
  return 'workspace';
}

/**
 Collects deterministic ownership categories for writable declarations.
 
 @param declarations - Resolved writable declarations.
 
 @param unresolved - Whether at least one writable declaration did not resolve.
 
 @param project - Project owning source classification.
 
 @returns sorted distinct ownership categories.
 
 @example
 ```ts
 writableDeclarationOwners({ declarations, unresolved: false, project });
 ```
 */
export function writableDeclarationOwners({
  declarations,
  unresolved,
  project,
}: {
  readonly declarations: readonly Node[];
  readonly unresolved: boolean;
  readonly project: Project;
}): readonly WritableDeclarationOwner[] {
  /**
   Ownership categories accumulated without declaration duplication.
   */
  const owners = new Set<WritableDeclarationOwner>(
    declarations.map(function declarationOwner(declaration,): WritableDeclarationOwner {
      return writableDeclarationOwner({
        declaration,
        project,
      },);
    },),
  );
  if (unresolved)
    owners.add('unresolved',);
  return [...owners,].toSorted();
}
