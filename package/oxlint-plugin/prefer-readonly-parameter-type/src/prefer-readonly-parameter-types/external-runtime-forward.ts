/**
 Runtime re-export forwarding that bypasses adjacent declaration substitution.
 
 @module
 */

import {
  dirname,
  extname,
  isAbsolute,
  relative,
  resolve,
  sep,
} from 'node:path';

import type {
  Node,
  SourceFile,
} from 'typescript/ast';
import {
  isExportDeclaration,
  isIdentifier,
  isImportDeclaration,
  isImportSpecifier,
  isNamedExports,
  isNamedImports,
  isSourceFile,
  isStringLiteral,
} from 'typescript/ast/is';
import type { Project, } from 'typescript/sync';

/**
 Sentinel when runtime source does not forward requested export through a relative module.
 */
export const RUNTIME_FORWARD_UNAVAILABLE: unique symbol = Symbol(
  'runtime package export does not have a resolvable relative forward',
);

/**
 Forwarded runtime source and binding name.
 */
export type RuntimeForward = {
  readonly source: SourceFile;
  readonly exportName: string;
};

/**
 Runtime JavaScript or TypeScript suffixes accepted for exact relative forwards.
 */
const RUNTIME_FORWARD_SUFFIXES: ReadonlySet<string> = new Set([
  '.js',
  '.mjs',
  '.cjs',
  '.ts',
  '.mts',
  '.cts',
  '.jsx',
  '.tsx',
]);

/**
 Resolves package-local runtime module without TypeScript declaration substitution.
 
 @param project - External implementation project containing shipped runtime files.
 
 @param source - Runtime module carrying relative import or re-export.
 
 @param moduleSpecifier - Authored relative runtime module specifier.
 
 @param packageRoot - Exact installed package boundary.
 
 @returns loaded runtime source or unavailable sentinel.
 */
function runtimeModule({
  project,
  source,
  moduleSpecifier,
  packageRoot,
}: {
  readonly project: Project;
  readonly source: SourceFile;
  readonly moduleSpecifier: string;
  readonly packageRoot: string;
}): SourceFile | typeof RUNTIME_FORWARD_UNAVAILABLE {
  if (!moduleSpecifier.startsWith('.',))
    return RUNTIME_FORWARD_UNAVAILABLE;
  /**
   Exact runtime path from authored module specifier.
   */
  const path = resolve(
    dirname(source.fileName,),
    moduleSpecifier,
  );
  /**
   Package-relative identity used for separator-aware containment check.
   */
  const packageRelativePath = relative(
    packageRoot,
    path,
  );
  if ((packageRelativePath === '..')
    || packageRelativePath.startsWith(`..${sep}`,)
    || isAbsolute(packageRelativePath,)
    || (!RUNTIME_FORWARD_SUFFIXES.has(extname(path,),)))
    return RUNTIME_FORWARD_UNAVAILABLE;
  /**
   Runtime source loaded under exact authored path.
   */
  const runtimeSource = project
    .program
    .getSourceFile(path,);
  return runtimeSource ?? RUNTIME_FORWARD_UNAVAILABLE;
}

/**
 Finds local binding imported by one runtime module.
 
 @param source - Runtime source containing imports.
 
 @param localName - Local binding re-exported by source.
 
 @returns import declaration and original export name or unavailable sentinel.
 */
function importedBinding({
  source,
  localName,
}: {
  readonly source: SourceFile;
  readonly localName: string;
}): {
  readonly declaration: Node;
  readonly exportName: string;
} | typeof RUNTIME_FORWARD_UNAVAILABLE {
  for (const statement of source.statements) {
    if (!isImportDeclaration(statement,))
      continue;
    /**
     Named import bindings that can be matched to local export.
     */
    const namedBindings = statement.importClause
      ?.namedBindings;
    if ((namedBindings === undefined)
      || (!isNamedImports(namedBindings,)))
      continue;
    for (const element of namedBindings.elements) {
      if (!isImportSpecifier(element,))
        continue;
      /**
       Local identifier introduced by named import.
       */
      const elementName = element
        .name
        .text;
      if (elementName !== localName)
        continue;
      /**
       Optional original package export before local rename.
       */
      const { propertyName, } = element;
      /**
       Original package export before optional local rename.
       */
      const importedName = propertyName === undefined
        ? elementName
        : propertyName.text;
      return {
        declaration: statement,
        exportName: importedName,
      };
    }
  }
  return RUNTIME_FORWARD_UNAVAILABLE;
}

/**
 Resolves one named export through authored relative runtime forwarding.
 
 TypeScript normally substitutes adjacent `.d.mts` files while resolving `.mjs` imports.
 This resolver follows only explicit runtime syntax and exact package-local paths.
 
 @param project - External implementation project containing shipped runtime files.
 
 @param sourceNode - Runtime module source.
 
 @param exportName - Public export requested by consumer.
 
 @param packageRoot - Exact installed package boundary.
 
 @returns forwarded runtime source and binding or unavailable sentinel.
 
 @example
 ```ts
 runtimeForwardedExport({ project, sourceNode, exportName: 'parse', packageRoot });
 ```
 */
export function runtimeForwardedExport({
  project,
  sourceNode,
  exportName,
  packageRoot,
}: {
  readonly project: Project;
  readonly sourceNode: Node;
  readonly exportName: string;
  readonly packageRoot: string;
}): RuntimeForward | typeof RUNTIME_FORWARD_UNAVAILABLE {
  if (!isSourceFile(sourceNode,))
    return RUNTIME_FORWARD_UNAVAILABLE;
  for (const statement of sourceNode.statements) {
    if (!isExportDeclaration(statement,))
      continue;
    /**
     Named exports that can identify one forwarded binding.
     */
    const { exportClause, } = statement;
    if ((exportClause === undefined)
      || (!isNamedExports(exportClause,)))
      continue;
    for (const element of exportClause.elements) {
      if (!isIdentifier(element.name,))
        continue;
      /**
       Public identifier introduced by named export.
       */
      const elementName = element
        .name
        .text;
      if (elementName !== exportName)
        continue;
      /**
       Optional original identifier before export rename.
       */
      const { propertyName, } = element;
      /**
       Original binding name before optional export rename.
       */
      const localName = (propertyName !== undefined)
        && isIdentifier(propertyName,)
        ? propertyName.text
        : elementName;
      /**
       Optional authored source on direct re-export declaration.
       */
      const { moduleSpecifier, } = statement;
      if ((moduleSpecifier !== undefined)
        && isStringLiteral(moduleSpecifier,)) {
        /**
         Runtime source selected by direct re-export.
         */
        const forwardedSource = runtimeModule({
          project,
          source: sourceNode,
          moduleSpecifier: moduleSpecifier.text,
          packageRoot,
        },);
        return forwardedSource === RUNTIME_FORWARD_UNAVAILABLE
          ? forwardedSource
          : {
            source: forwardedSource,
            exportName: localName,
          };
      }
      /**
       Imported local binding selected for local export declaration.
       */
      const imported = importedBinding({
        source: sourceNode,
        localName,
      },);
      if (imported === RUNTIME_FORWARD_UNAVAILABLE)
        return RUNTIME_FORWARD_UNAVAILABLE;
      /**
       Declaration carrying selected local import.
       */
      const importedDeclaration = imported.declaration;
      if (!isImportDeclaration(importedDeclaration,))
        return RUNTIME_FORWARD_UNAVAILABLE;
      /**
       Authored source from selected import declaration.
       */
      const importedSpecifier = importedDeclaration.moduleSpecifier;
      if (!isStringLiteral(importedSpecifier,))
        return RUNTIME_FORWARD_UNAVAILABLE;
      /**
       Runtime source selected through imported local binding.
       */
      const importedSource = runtimeModule({
        project,
        source: sourceNode,
        moduleSpecifier: importedSpecifier.text,
        packageRoot,
      },);
      return importedSource === RUNTIME_FORWARD_UNAVAILABLE
        ? importedSource
        : {
          source: importedSource,
          exportName: imported.exportName,
        };
    }
  }
  return RUNTIME_FORWARD_UNAVAILABLE;
}
