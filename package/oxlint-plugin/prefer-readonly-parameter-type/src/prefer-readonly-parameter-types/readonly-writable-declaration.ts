/**
 Writable property and index declaration ownership extraction.
 
 @module
 */

import type { Node, } from 'typescript/ast';
import type {
  IndexInfo,
  Project,
  Symbol as TypeScriptSymbol,
} from 'typescript/sync';

import type { WritableDeclarationOwner, } from './readonly-classification-model.ts';
import { declarationIsReadonly, } from './readonly-declaration.ts';
import { writableDeclarationOwners, } from './readonly-declaration-ownership.ts';

/**
 Resolves writable declaration nodes from one property symbol.
 
 @param property - Semantic property classified as writable.
 
 @param project - Project resolving declaration handles.
 
 @returns writable declarations and whether any declaration was unavailable.
 */
function writablePropertyDeclarations({
  property,
  project,
}: {
  readonly property: TypeScriptSymbol;
  readonly project: Project;
}): {
  readonly declarations: readonly Node[];
  readonly unresolved: boolean;
} {
  /**
   Semantic declarations narrowed once for resolution and traversal.
   */
  const { declarations: propertyDeclarations, } = property;
  /**
   Mutable resolution state carried across declaration handles.
   */
  const resolution = { unresolved: propertyDeclarations.length === 0, };
  /**
   Resolved declarations whose authored slot is not readonly.
   */
  const declarations = propertyDeclarations
    .flatMap(function resolveWritable(handle,): readonly Node[] {
      /**
       Declaration resolved through current semantic snapshot.
       */
      const declaration = handle.resolve(project,);
      if (declaration === undefined) {
        resolution.unresolved = true;
        return [];
      }
      return declarationIsReadonly(declaration,) ? [] : [declaration,];
    },);
  return {
    declarations,
    unresolved: resolution.unresolved,
  };
}

/**
 Collects ownership for declarations making one property writable.
 
 @param property - Semantic property classified as writable.
 
 @param project - Project resolving and classifying declarations.
 
 @returns sorted distinct writable declaration owners.
 
 @example
 ```ts
 writablePropertyOwners({ property, project });
 ```
 */
export function writablePropertyOwners({
  property,
  project,
}: {
  readonly property: TypeScriptSymbol;
  readonly project: Project;
}): readonly WritableDeclarationOwner[] {
  /**
   Writable declaration resolution for current property.
   */
  const resolution = writablePropertyDeclarations({
    property,
    project,
  },);
  return writableDeclarationOwners({
    ...resolution,
    project,
  },);
}

/**
 Collects ownership for one writable index signature.
 
 @param indexInfo - Semantic index classified as writable.
 
 @param project - Project resolving and classifying declaration.
 
 @returns writable declaration ownership or unresolved evidence.
 
 @example
 ```ts
 writableIndexOwners({ indexInfo, project });
 ```
 */
export function writableIndexOwners({
  indexInfo,
  project,
}: {
  readonly indexInfo: IndexInfo;
  readonly project: Project;
}): readonly WritableDeclarationOwner[] {
  /**
   Optional authored index declaration resolved in current snapshot.
   */
  const declaration = indexInfo.declaration
    ?.resolve(project,);
  return writableDeclarationOwners({
    declarations: declaration === undefined ? [] : [declaration,],
    unresolved: declaration === undefined,
    project,
  },);
}
