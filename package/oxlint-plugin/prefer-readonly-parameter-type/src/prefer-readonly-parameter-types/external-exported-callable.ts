/**
 Runtime package export callable resolution.
 
 @module
 */

import type { Node, } from 'typescript/ast';
import {
  isClassLikeDeclaration,
  isGetAccessorDeclaration,
  isIdentifier,
  isMethodDeclaration,
  isObjectLiteralExpression,
  isPropertyAssignment,
  isPropertyDeclaration,
  isSetAccessorDeclaration,
  isStringLiteral,
  isVariableDeclaration,
} from 'typescript/ast/is';
import {
  type Project,
  SymbolFlags,
} from 'typescript/sync';

import { callableDeclaration, } from './effect-call-resolution.ts';
import {
  RUNTIME_FORWARD_UNAVAILABLE,
  runtimeForwardedExport,
} from './external-runtime-forward.ts';

/**
 Sentinel when runtime export cannot resolve to callable source.
 */
export const EXPORTED_CALLABLE_UNAVAILABLE: unique symbol = Symbol(
  'runtime package export callable source could not be resolved',
);

/**
 Reads simple authored member name.
 
 @param node - Class or object member candidate.
 
 @returns member name or unavailable sentinel.
 */
function memberName(
  node: Node,
): string | typeof EXPORTED_CALLABLE_UNAVAILABLE {
  if ((!isMethodDeclaration(node,))
    && (!isPropertyAssignment(node,))
    && (!isPropertyDeclaration(node,))
    && (!isGetAccessorDeclaration(node,))
    && (!isSetAccessorDeclaration(node,)))
    return EXPORTED_CALLABLE_UNAVAILABLE;
  return isIdentifier(node.name,) || isStringLiteral(node.name,)
    ? node.name
      .text
    : EXPORTED_CALLABLE_UNAVAILABLE;
}

/**
 Resolves callable nested under exported class or object value.
 
 @param project - External implementation project.
 
 @param declaration - Exported runtime value declaration.
 
 @param path - Authored member path from imported export.
 
 @param packageRoot - External source root accepted by callable resolver.
 
 @returns callable declaration or effect-unavailable sentinel.
 */
function exportedMemberCallable({
  project,
  declaration,
  path,
  packageRoot,
}: {
  readonly project: Project;
  readonly declaration: Node;
  readonly path: readonly string[];
  readonly packageRoot: string;
}): ReturnType<typeof callableDeclaration> | typeof EXPORTED_CALLABLE_UNAVAILABLE {
  /**
   Mutable syntax cursor following authored object members.
   */
  const cursor: { current: Node; } = { current: declaration, };
  for (const segment of path) {
    if (isVariableDeclaration(cursor.current,)
      && (cursor.current
        .initializer
        !== undefined))
      cursor.current = cursor.current
        .initializer;
    /**
     Direct class or object members at current path segment.
     */
    const members = isClassLikeDeclaration(cursor.current,)
      ? cursor.current
        .members
      : isObjectLiteralExpression(cursor.current,)
        ? cursor.current
          .properties
        : [];
    /**
     Exact authored member declaration.
     */
    const member = members.find(function matchingMember(candidate,): boolean {
      return memberName(candidate,) === segment;
    },);
    if (member === undefined)
      return EXPORTED_CALLABLE_UNAVAILABLE;
    cursor.current = isPropertyAssignment(member,)
      ? member.initializer
      : member;
  }
  return callableDeclaration({
    project,
    node: cursor.current,
    analysisRoot: packageRoot,
  },);
}

/**
 Resolves exported callable declaration from implementation module.
 
 @param project - External implementation project.
 
 @param sourceNode - Implementation source file node.
 
 @param exportName - Exact package export member.
 
 @param memberPath - Nested class or object member path.
 
 @param packageRoot - External source root accepted by callable resolver.
 
 @returns callable declaration or effect-unavailable sentinel.
 
 @mutates project - `project.checker.getExportsOfModule` may populate TypeScript checker caches.
 
 @example
 ```ts
 exportedCallable({ project, sourceNode, exportName, memberPath, packageRoot });
 ```
 */
export function exportedCallable({
  project,
  sourceNode,
  exportName,
  memberPath,
  packageRoot,
  visitedSourceNames = new Set(),
}: {
  readonly project: Project;
  readonly sourceNode: Node;
  readonly exportName: string;
  readonly memberPath: readonly string[];
  readonly packageRoot: string;
  readonly visitedSourceNames?: ReadonlySet<string>;
}): ReturnType<typeof callableDeclaration> | typeof EXPORTED_CALLABLE_UNAVAILABLE {
  /**
   Exact runtime forwarding available independently of TypeScript alias substitution.
   */
  const forwarded = runtimeForwardedExport({
    project,
    sourceNode,
    exportName,
    packageRoot,
  },);
  /**
   Resolves precomputed runtime forwarding with cycle protection.
   
   @returns forwarded callable or unavailable sentinel.
   */
  function resolveForwardedCallable(): ReturnType<typeof callableDeclaration>
    | typeof EXPORTED_CALLABLE_UNAVAILABLE {
    if (forwarded === RUNTIME_FORWARD_UNAVAILABLE)
      return EXPORTED_CALLABLE_UNAVAILABLE;
    /**
     Runtime source selected by authored forwarding syntax.
     */
    const forwardedSource = forwarded.source;
    /**
     Canonical forwarded source identity used for cycle protection.
     */
    const forwardedSourceName = forwardedSource.fileName;
    if (visitedSourceNames.has(forwardedSourceName,))
      return EXPORTED_CALLABLE_UNAVAILABLE;
    return exportedCallable({
      project,
      sourceNode: forwardedSource,
      exportName: forwarded.exportName,
      memberPath,
      packageRoot,
      visitedSourceNames: new Set([
        ...visitedSourceNames,
        forwardedSourceName,
      ],),
    },);
  }
  /**
   Module symbol exposing runtime exports.
   */
  const moduleSymbol = project.checker
    .getSymbolAtLocation(sourceNode,);
  if (moduleSymbol === undefined)
    return resolveForwardedCallable();
  /**
   Exact exported symbol by authored package binding name.
   */
  const exported = project.checker
    .getExportsOfModule(moduleSymbol,)
    .find(function namedExport(symbol,): boolean {
      return symbol.name === exportName;
    },);
  if (exported === undefined)
    return resolveForwardedCallable();
  /**
   Export alias followed to implementation declaration.
   */
  const resolved = (exported.flags & SymbolFlags.Alias) !== 0
    ? project.checker
      .getAliasedSymbol(exported,)
    : exported;
  /**
   Preferred implementation declaration handle.
   */
  const handle = resolved.valueDeclaration
    ?? resolved.declarations
      .at(0,);
  /**
   Resolved implementation declaration node.
   */
  const declaration = handle?.resolve(project,);
  if (declaration === undefined)
    return resolveForwardedCallable();
  /**
   Callable resolved through TypeScript's ordinary alias graph.
   */
  const callable = memberPath.length === 0
    ? callableDeclaration({
      project,
      node: declaration,
      analysisRoot: packageRoot,
    },)
    : exportedMemberCallable({
      project,
      declaration,
      path: memberPath,
      packageRoot,
    },);
  if ((typeof callable) !== 'symbol')
    return callable;
  /**
   Runtime forwarding result when declaration substitution hid implementation.
   */
  const forwardedResult = resolveForwardedCallable();
  return forwardedResult === EXPORTED_CALLABLE_UNAVAILABLE
    ? callable
    : forwardedResult;
}
