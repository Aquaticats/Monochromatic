/**
 Package call identity recovered from exported declaration owner.
 
 @module
 */

import type {
  CallExpression,
  Node,
} from 'typescript/ast';
import {
  isClassLikeDeclaration,
  isIdentifier,
  isInterfaceDeclaration,
  isPropertyAccessExpression,
  isSourceFile,
  isTypeAliasDeclaration,
  isVariableDeclaration,
} from 'typescript/ast/is';

import {
  type PackageCallIdentity,
  PACKAGE_CALL_IDENTITY_UNAVAILABLE,
} from './effect-package-call-identity.ts';
import type { InstalledPackageIdentity, } from './installed-package-identity.ts';
import {
  PACKAGE_IMPLEMENTATION_UNAVAILABLE,
  packageModuleSpecifierForDeclaration,
} from './package-implementation-resolution.ts';

/**
 Finds nearest named exported owner around declaration.
 
 @param declaration - Selected package method declaration.
 
 @returns owner name or package-call unavailable sentinel.
 */
function declarationOwnerName(
  declaration: Node,
): string | typeof PACKAGE_CALL_IDENTITY_UNAVAILABLE {
  /** Parent cursor seeking class,
   interface,
   type alias,
   or exported variable owner.
   */
  const cursor: { current: Node; } = { current: declaration.parent, };
  while (!isSourceFile(cursor.current,)) {
    if (isClassLikeDeclaration(cursor.current,)) {
      /**
       Optional class or class-expression name.
       */
      const { name, } = cursor.current;
      if ((name !== undefined) && isIdentifier(name,))
        return name.text;
    }
    if (isInterfaceDeclaration(cursor.current,)
      || isTypeAliasDeclaration(cursor.current,)
      || isVariableDeclaration(cursor.current,)) {
      /**
       Required named declaration binding.
       */
      const { name, } = cursor.current;
      if (isIdentifier(name,))
        return name.text;
    }
    cursor.current = cursor.current
      .parent;
  }
  return PACKAGE_CALL_IDENTITY_UNAVAILABLE;
}

/**
 Recovers package export and member identity from declaration owner.
 
 Used for instance and retained-object methods whose call receiver is not an
 import binding.
 
 @param identity - Exact package owning declaration.
 
 @param call - Invoked package method call.
 
 @param declaration - Selected package method declaration.
 
 @returns exact package call identity or unavailable sentinel.
 
 @example
 ```ts
 packageDeclarationCallIdentity({ identity, call, declaration });
 ```
 */
export function packageDeclarationCallIdentity({
  identity,
  call,
  declaration,
}: {
  readonly identity: InstalledPackageIdentity;
  readonly call: CallExpression;
  readonly declaration: Node;
}): PackageCallIdentity | typeof PACKAGE_CALL_IDENTITY_UNAVAILABLE {
  if (!isPropertyAccessExpression(call.expression,))
    return PACKAGE_CALL_IDENTITY_UNAVAILABLE;
  /**
   Module specifier whose types entry exactly owns declaration source.
   */
  const moduleSpecifier = packageModuleSpecifierForDeclaration({
    identity,
    declarationFileName: declaration.getSourceFile()
      .fileName,
  },);
  if (moduleSpecifier === PACKAGE_IMPLEMENTATION_UNAVAILABLE)
    return PACKAGE_CALL_IDENTITY_UNAVAILABLE;
  /**
   Runtime export owner matching declaration container.
   */
  const exportName = declarationOwnerName(declaration,);
  return exportName === PACKAGE_CALL_IDENTITY_UNAVAILABLE
    ? PACKAGE_CALL_IDENTITY_UNAVAILABLE
    : {
      moduleSpecifier,
      exportName,
      memberPath: [call.expression
        .name
        .text,],
    };
}
