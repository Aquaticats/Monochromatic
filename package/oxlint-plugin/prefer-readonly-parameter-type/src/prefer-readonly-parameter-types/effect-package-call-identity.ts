/**
 Authored package import identity for invoked call expression.
 
 @module
 */

import type {
  CallExpression,
  ImportClause,
  ImportDeclaration,
  ImportSpecifier,
  NamespaceImport,
  Node,
} from 'typescript/ast';
import {
  isIdentifier,
  isImportClause,
  isImportDeclaration,
  isImportSpecifier,
  isNamespaceImport,
  isPropertyAccessExpression,
  isSourceFile,
  isStringLiteral,
} from 'typescript/ast/is';
import type {
  Checker,
  Project,
} from 'typescript/sync';

/**
 Sentinel when call lacks exact package import identity.
 */
export const PACKAGE_CALL_IDENTITY_UNAVAILABLE: unique symbol = Symbol(
  'package call identity unavailable',
);

/**
 Exact package module export invoked by caller.
 */
export type PackageCallIdentity = {
  readonly moduleSpecifier: string;
  readonly exportName: string;
  readonly memberPath: readonly string[];
};

/**
 Import declaration and binding declaration for one local symbol.
 */
type ImportBinding = {
  readonly declaration: ImportDeclaration;
  readonly binding: ImportClause | ImportSpecifier | NamespaceImport;
};

/**
 Finds enclosing import declaration for one binding node.
 
 @param node - Import binding declaration candidate.
 
 @returns import declaration and binding or unavailable sentinel.
 */
function enclosingImport(
  node: Node,
): ImportBinding | typeof PACKAGE_CALL_IDENTITY_UNAVAILABLE {
  if ((!isImportClause(node,))
    && (!isImportSpecifier(node,))
    && (!isNamespaceImport(node,)))
    return PACKAGE_CALL_IDENTITY_UNAVAILABLE;
  /**
   Import binding retained while parent cursor ascends.
   */
  const binding = node;
  /**
   Parent cursor bounded by source file.
   */
  const cursor: { current: Node; } = { current: node.parent, };
  while (!isSourceFile(cursor.current,)) {
    if (isImportDeclaration(cursor.current,)) {
      return {
        declaration: cursor.current,
        binding,
      };
    }
    cursor.current = cursor.current
      .parent;
  }
  return PACKAGE_CALL_IDENTITY_UNAVAILABLE;
}

/**
 Resolves local symbol to authored import binding.
 
 @param project - TypeScript project resolving symbol handles.
 
 @param checker - TypeScript checker resolving local symbol.
 
 @param node - Local imported identifier.
 
 @returns import binding or unavailable sentinel.
 */
function importBinding({
  project,
  checker,
  node,
}: {
  readonly project: Project;
  readonly checker: Checker;
  readonly node: Node;
}): ImportBinding | typeof PACKAGE_CALL_IDENTITY_UNAVAILABLE {
  /**
   Local import alias symbol.
   */
  const symbol = checker.getSymbolAtLocation(node,);
  if (symbol === undefined)
    return PACKAGE_CALL_IDENTITY_UNAVAILABLE;
  for (const handle of symbol.declarations) {
    /**
     Local import declaration node resolved in caller project.
     */
    const declaration = handle.resolve(project,);
    if (declaration === undefined)
      continue;
    /**
     Enclosing import for supported binding declaration.
     */
    const imported = enclosingImport(declaration,);
    if (imported !== PACKAGE_CALL_IDENTITY_UNAVAILABLE)
      return imported;
  }
  return PACKAGE_CALL_IDENTITY_UNAVAILABLE;
}

/**
 Extracts nonrelative string module specifier.
 
 @param declaration - Authored import declaration.
 
 @returns package module specifier or unavailable sentinel.
 */
function packageModuleSpecifier(
  declaration: ImportDeclaration,
): string | typeof PACKAGE_CALL_IDENTITY_UNAVAILABLE {
  if (!isStringLiteral(declaration.moduleSpecifier,))
    return PACKAGE_CALL_IDENTITY_UNAVAILABLE;
  /**
   Authored import module text.
   */
  const { text, } = declaration.moduleSpecifier;
  return text.startsWith('.',) || text.startsWith('/',)
    ? PACKAGE_CALL_IDENTITY_UNAVAILABLE
    : text;
}

/**
 Resolves exact package module export invoked by call.
 
 Supports direct default or named imports and namespace member calls.
 Other value-flow shapes fail closed.
 
 @param project - Caller TypeScript project.
 
 @param checker - Caller checker.
 
 @param call - Invoked call expression.
 
 @returns package call identity or unavailable sentinel.
 
 @example
 ```ts
 packageCallIdentity({ project, checker, call });
 ```
 */
export function packageCallIdentity({
  project,
  checker,
  call,
}: {
  readonly project: Project;
  readonly checker: Checker;
  readonly call: CallExpression;
}): PackageCallIdentity | typeof PACKAGE_CALL_IDENTITY_UNAVAILABLE {
  if (isIdentifier(call.expression,)) {
    /**
     Import binding for direct callable identifier.
     */
    const imported = importBinding({
      project,
      checker,
      node: call.expression,
    },);
    if (imported === PACKAGE_CALL_IDENTITY_UNAVAILABLE)
      return PACKAGE_CALL_IDENTITY_UNAVAILABLE;
    /**
     Exact package module specifier.
     */
    const moduleSpecifier = packageModuleSpecifier(imported.declaration,);
    if (moduleSpecifier === PACKAGE_CALL_IDENTITY_UNAVAILABLE)
      return PACKAGE_CALL_IDENTITY_UNAVAILABLE;
    if (isImportSpecifier(imported.binding,)) {
      return {
        moduleSpecifier,
        exportName: imported.binding
          .propertyName
          ?.text
          ?? imported.binding
            .name
            .text,
        memberPath: [],
      };
    }
    if (isImportClause(imported.binding,)) {
      return {
        moduleSpecifier,
        exportName: 'default',
        memberPath: [],
      };
    }
    return PACKAGE_CALL_IDENTITY_UNAVAILABLE;
  }
  if ((!isPropertyAccessExpression(call.expression,))
    || (!isIdentifier(call.expression
      .expression,)))
    return PACKAGE_CALL_IDENTITY_UNAVAILABLE;
  /**
   Namespace import binding for property call receiver.
   */
  const imported = importBinding({
    project,
    checker,
    node: call.expression
      .expression,
  },);
  if (imported === PACKAGE_CALL_IDENTITY_UNAVAILABLE)
    return PACKAGE_CALL_IDENTITY_UNAVAILABLE;
  /**
   Exact package module specifier.
   */
  const moduleSpecifier = packageModuleSpecifier(imported.declaration,);
  if (moduleSpecifier === PACKAGE_CALL_IDENTITY_UNAVAILABLE)
    return PACKAGE_CALL_IDENTITY_UNAVAILABLE;
  if (isNamespaceImport(imported.binding,)) {
    return {
      moduleSpecifier,
      exportName: call.expression
        .name
        .text,
      memberPath: [],
    };
  }
  if (isImportSpecifier(imported.binding,)) {
    return {
      moduleSpecifier,
      exportName: imported.binding
        .propertyName
        ?.text
        ?? imported.binding
          .name
          .text,
      memberPath: [call.expression
        .name
        .text,],
    };
  }
  if (isImportClause(imported.binding,)) {
    return {
      moduleSpecifier,
      exportName: 'default',
      memberPath: [call.expression
        .name
        .text,],
    };
  }
  return PACKAGE_CALL_IDENTITY_UNAVAILABLE;
}
