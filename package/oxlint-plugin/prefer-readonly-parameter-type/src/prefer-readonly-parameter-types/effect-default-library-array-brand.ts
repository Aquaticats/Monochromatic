/**
 Exact semantic recognition for TypeScript's default-library array brand check.
 
 @module
 */

import type { Node, } from 'typescript/ast';
import {
  isIdentifier,
  isInterfaceDeclaration,
  isMethodSignatureDeclaration,
} from 'typescript/ast/is';
import type { Project, } from 'typescript/sync';

/**
 TypeScript default-library interface owning `Array.isArray`.
 */
const ARRAY_CONSTRUCTOR_INTERFACE_NAME = 'ArrayConstructor';

/**
 Array brand-check method name.
 */
const ARRAY_BRAND_METHOD_NAME = 'isArray';

/**
 Prove selected declaration is built-in `Array.isArray`.
 
 Recognition depends on semantic declaration ownership, not authored call
 text. Shadowed globals, project declarations, and package declarations stay
 unresolved unless their implementations can be inspected normally.
 
 @param project - TypeScript project proving default-library ownership.
 
 @param declaration - Selected callable declaration.
 
 @returns Whether declaration is exact non-dispatching array brand check.
 
 @example
 ```typescript
 isDefaultLibraryArrayBrandDeclaration({ project, declaration });
 ```
 */
export function isDefaultLibraryArrayBrandDeclaration({
  project,
  declaration,
}: {
  readonly project: Project;
  readonly declaration: Node;
}): boolean {
  if ((!isMethodSignatureDeclaration(declaration,))
    || (!isIdentifier(declaration.name,)))
    return false;
  /**
   Selected method name after declaration-shape narrowing.
   */
  const declarationName = declaration
    .name
    .text;
  if ((declarationName !== ARRAY_BRAND_METHOD_NAME)
    || (!project
      .program
      .isSourceFileDefaultLibrary(declaration.getSourceFile(),)))
    return false;
  /**
   Default-library interface selected as method owner.
   */
  const owner = declaration.parent;
  if ((!isInterfaceDeclaration(owner,)) || (!isIdentifier(owner.name,)))
    return false;
  /**
   Selected owner name after interface-shape narrowing.
   */
  const ownerName = owner
    .name
    .text;
  return ownerName === ARRAY_CONSTRUCTOR_INTERFACE_NAME;
}
