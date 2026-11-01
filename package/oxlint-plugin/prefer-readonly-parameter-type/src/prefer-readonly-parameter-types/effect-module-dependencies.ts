/**
 Resolved module dependencies for persistent-cache invalidation.
 
 @module
 */

import {
  type Node,
  type SourceFile,
  SyntaxKind,
} from 'typescript/ast';
import {
  isCallExpression,
  isExportDeclaration,
  isImportDeclaration,
  isImportEqualsDeclaration,
  isImportTypeNode,
  isLiteralTypeNode,
  isStringLiteral,
} from 'typescript/ast/is';
import type { Project, } from 'typescript/sync';

import { collectAstNodes, } from './effect-summary-model.ts';

/**
 Sentinel when one module reference cannot be resolved to a program file.
 
 Unresolved references make a file's dependency set unknowable, so its
 persistent cache entry must fail closed instead of validating a partial set.
 */
export const MODULE_DEPENDENCIES_UNRESOLVED: unique symbol = Symbol(
  'module dependency references could not be fully resolved',
);

/**
 Collects module reference specifier nodes authored in one source.
 
 Static `import`/`export ... from` declarations, `import =` external
 references, literal dynamic `import()` arguments, and literal
 `import('...')` type queries name every channel TypeScript name
 resolution can enter another module from this source.
 A runtime-variable dynamic `import()` argument is skipped instead of
 failing closed: the checker types its result independently of any
 workspace file's content, so no other file can change this source's
 summaries through that call, and value flow through it stays fail-closed
 inside effect analysis itself.
 A non-literal `import()` type-query argument is statically unknowable
 yet does shape checker semantics, so it fails the collection closed
 through the unresolved sentinel.
 
 @param sourceFile - Source whose module references are collected.
 
 @returns specifier nodes,
 or unresolved sentinel when any semantic reference is statically unknowable.
 */
function moduleReferenceSpecifiers(
  sourceFile: SourceFile,
): readonly Node[] | typeof MODULE_DEPENDENCIES_UNRESOLVED {
  /**
   Collected statically resolvable specifier nodes.
   */
  const specifiers: Node[] = [];
  for (const node of collectAstNodes(sourceFile,)) {
    if (isImportDeclaration(node,)) {
      specifiers.push(node.moduleSpecifier,);
      continue;
    }
    if (isExportDeclaration(node,)) {
      if (node.moduleSpecifier !== undefined)
        specifiers.push(node.moduleSpecifier,);
      continue;
    }
    if (isImportEqualsDeclaration(node,)) {
      /**
       External module reference expression for `import name = require(...)`.
       */
      const reference = node.moduleReference;
      if (reference.kind !== SyntaxKind.ExternalModuleReference)
        continue;
      if (!('expression' in reference))
        return MODULE_DEPENDENCIES_UNRESOLVED;
      specifiers.push(reference.expression,);
      continue;
    }
    if (isImportTypeNode(node,)) {
      /**
       Type-query module argument shaping checker semantics.
       */
      const { argument, } = node;
      if ((!isLiteralTypeNode(argument,))
        || (!isStringLiteral(argument.literal,)))
        return MODULE_DEPENDENCIES_UNRESOLVED;
      specifiers.push(argument.literal,);
      continue;
    }
    if (isCallExpression(node,)
      && (node.expression
        .kind
        === SyntaxKind.ImportKeyword)) {
      /**
       First dynamic import argument, statically typed only as string literal.
       */
      const argument = node.arguments
        .at(0,);
      if ((argument !== undefined) && isStringLiteral(argument,))
        specifiers.push(argument,);
    }
  }
  return specifiers;
}

/**
 Resolves every authored module reference in one source to program files.
 
 @param project - Configured project resolving module symbols.
 
 @param sourceFile - Source whose dependencies are resolved.
 
 @returns sorted unique resolved file paths,
 or unresolved sentinel when any reference cannot be proven.
 
 @example
 ```ts
 const dependencies = directModuleDependencies({ project, sourceFile });
 ```
 */
export function directModuleDependencies({
  project,
  sourceFile,
}: {
  readonly project: Project;
  readonly sourceFile: SourceFile;
},): readonly string[] | typeof MODULE_DEPENDENCIES_UNRESOLVED {
  /**
   Authored module reference specifiers, or unknowable-reference sentinel.
   */
  const specifiers = moduleReferenceSpecifiers(sourceFile,);
  if ((typeof specifiers) === 'symbol')
    return MODULE_DEPENDENCIES_UNRESOLVED;
  /**
   Unique resolved dependency paths.
   */
  const resolved = new Set<string>();
  for (const specifier of specifiers) {
    /**
     Module symbol for specifier, undefined for unresolvable references.
     */
    const symbol = project.checker
      .getSymbolAtLocation(specifier,);
    /**
     Preferred declaration handle carrying module source identity.
     */
    const handle = symbol?.valueDeclaration
      ?? symbol?.declarations
        .at(0,);
    /**
     Resolved declaration whose owning file names the dependency.
     */
    const declaration = handle?.resolve(project,);
    if (declaration === undefined)
      return MODULE_DEPENDENCIES_UNRESOLVED;
    resolved.add(
      declaration.getSourceFile()
        .fileName,
    );
  }
  return [...resolved,].toSorted();
}
