/**
 Exact semantic identity check for opaque foreign host capability marker.
 
 @module
 */

import type {
  Project,
  Type,
} from 'typescript/sync';

/**
 Detects exact project-owned foreign host capability marker.
 
 @param project - TypeScript project resolving alias declarations.
 
 @param type - TypeScript semantic type.
 
 @returns whether type uses exact host capability marker declaration.
 
 @example
 ```ts
 isForeignHostCapabilityType({ project, type });
 ```
 */
export function isForeignHostCapabilityType({
  project,
  type,
}: {
  readonly project: Project;
  readonly type: Type;
},): boolean {
  /**
   Type constituents awaiting exact alias inspection.
   */
  const pending: Type[] = [type,];
  /**
   Semantic type identities already inspected through unions or intersections.
   */
  const visited = new Set<number>();
  while (pending.length > 0) {
    /**
     Next semantic type constituent.
     */
    const current = pending.pop();
    if ((current === undefined) || visited.has(current.id,))
      continue;
    visited.add(current.id,);
    /**
     Authored alias symbol retained by generic marker instantiation.
     */
    const alias = current.getAliasSymbol();
    if ((alias !== undefined)
      && (alias.name === 'ForeignHostCapability')
      && alias.declarations
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
        },))
      return true;
    if (current.isUnionType() || current.isIntersectionType()) {
      current.getTypes()
        .forEach(function enqueueConstituent(constituent,): void {
          pending[pending.length] = constituent;
        },);
    }
  }
  return false;
}
