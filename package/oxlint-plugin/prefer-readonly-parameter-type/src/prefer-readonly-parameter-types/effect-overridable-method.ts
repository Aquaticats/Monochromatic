/**
 Whether a resolved callee is an instance method a subclass may override.
 
 A call resolves against the receiver's declared type, so an instance method resolves to the
 declaration that type names. The value at runtime may be a subclass whose override runs
 instead, and an override is free to write what the base only reads. Enumerating overrides
 cannot settle it either: a subclass may live in a consuming package this analysis never sees.
 
 @module
 */

import {
  type Node,
  SyntaxKind,
} from 'typescript/ast';
import {
  isClassDeclaration,
  isClassExpression,
  isMethodDeclaration,
} from 'typescript/ast/is';

/**
 Modifiers that stop a method from being reached through a subclass instance.
 
 `private` cannot be overridden at all. `static` is looked up on the constructor rather than
 through the prototype chain, so a call on an instance never dispatches to a derived one.
 */
const NON_OVERRIDABLE_MODIFIERS: ReadonlySet<SyntaxKind> = new Set([
  SyntaxKind.PrivateKeyword,
  SyntaxKind.StaticKeyword,
]);

/**
 Tests whether a declaration is a class instance method open to overriding.
 
 An object-literal method is not: nothing extends an object literal, so the body written there
 is the body that runs.
 
 @param declaration - Declaration a call resolved to.
 
 @returns whether a subclass could supply a different body.
 
 @example
 ```ts
 isOverridableMethod({ declaration });
 ```
 */
export function isOverridableMethod(
  { declaration, }: { readonly declaration: Node; },
): boolean {
  if (!isMethodDeclaration(declaration,))
    return false;
  /**
   Declaration holding this method, which decides whether anything can extend it.
   */
  const owner = declaration.parent;
  if ((owner === undefined)
    || ((!isClassDeclaration(owner,)) && (!isClassExpression(owner,))))
    return false;
  return !(declaration.modifiers ?? [])
    .some(function stopsOverriding(modifier,): boolean {
      return NON_OVERRIDABLE_MODIFIERS.has(modifier.kind,);
    },);
}
