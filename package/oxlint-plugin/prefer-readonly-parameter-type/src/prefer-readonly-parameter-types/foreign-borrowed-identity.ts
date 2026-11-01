/**
 Exact semantic identity check for foreign ownership marker.
 
 @module
 */

import type {
  Project,
  Type,
} from 'typescript/sync';

/**
 Detects exact project-owned foreign ownership marker.
 
 @param project - TypeScript project resolving alias declarations.
 
 @param type - TypeScript semantic type.
 
 @returns whether type uses marker declared by this plugin.
 
 @example
 ```ts
 isForeignBorrowedType({ project, type });
 ```
 */
export function isForeignBorrowedType({
  project,
  type,
}: {
  readonly project: Project;
  readonly type: Type;
},): boolean {
  /**
   Authored alias symbol retained by generic marker instantiation.
   */
  const alias = type.getAliasSymbol();
  if ((alias === undefined)
    || ((alias.name !== 'ForeignBorrowed')
      && (alias.name !== 'ForeignHostCapability')))
    return false;
  return alias.declarations
    .some(function markerDeclaration(handle,): boolean {
      /**
       Resolved marker declaration for exact source identity.
       */
      const declaration = handle.resolve(project,);
      return (declaration !== undefined)
        && declaration.getSourceFile()
          .fileName
          .replaceAll(
          '\\',
          '/',
          )
          .endsWith('/ownership-marker/foreign-borrowed/src/index.ts',);
    },);
}
