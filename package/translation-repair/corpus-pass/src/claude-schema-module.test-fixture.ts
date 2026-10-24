import {
  identifierName,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import { gatherDeclaration, } from './claude-schema-declare.test-fixture.ts';
import {
  type Binding,
  type ExportEntry,
  type Gathered,
  joinedAll,
  type ModuleFacts,
  nameWrittenBy,
  NOTHING,
} from './claude-schema-facts.test-fixture.ts';
import {
  holdsFormatMention,
  isNoNode,
  nodeAt,
  nodesAt,
  referencesIn,
  stringValueOf,
} from './claude-schema-tree.test-fixture.ts';

//region What one module imports and exports
// Reads one parsed file for the facts `claude-schema-resolve.test-fixture.ts` resolves names through: what its import
// declarations bind, what its export declarations expose, and, through the declarations, what it declares. Nothing here
// leaves the file; which function a name stands for in another module is the resolver's question.

/**
 The name a module exports an imported name by.

 @param entry - the import specifier

 @returns `default` for a default import, `*` for a namespace import, the written name for a named import, empty for
 any other node

 @example
 ```ts
 const imported = importedNameOf({ entry, },);
 ```
 */
function importedNameOf({ entry, }: { readonly entry: TreeNode; },): string {
  if (entry.type === 'ImportDefaultSpecifier')
    return 'default';
  if (entry.type === 'ImportNamespaceSpecifier')
    return '*';
  if (entry.type !== 'ImportSpecifier')
    return '';
  return nameWrittenBy({ node: nodeAt({
    node: entry,
    field: 'imported',
  },), },);
}

/**
 What an import declaration adds: a binding for each name it brings in, unless it brings in types only.

 @param statement - the import declaration

 @returns The bindings

 @example
 ```ts
 const added = gatherImport({ statement, },);
 ```
 */
function gatherImport({ statement, }: { readonly statement: TreeNode; },): Gathered {
  /**
   The module the import comes from.
   */
  const specifier = stringValueOf({ node: statement.source, },);
  if ((statement.importKind === 'type') || (specifier === ''))
    return NOTHING;
  return {
    ...NOTHING,
    bindings: nodesAt({
      node: statement,
      field: 'specifiers',
    },)
      .flatMap(function bindingOf(entry,): readonly (readonly [
        string,
        Binding,
      ])[] {
        /**
         The local name.
         */
        const local = identifierName({ node: nodeAt({
          node: entry,
          field: 'local',
        },), },);
        /**
         The name the module exports it by.
         */
        const imported = importedNameOf({ entry, },);
        return ((entry.importKind === 'type') || (imported === '')
          || (local === ''))
          ? []
          : [
            [
              local,
              {
                kind: 'import',
                specifier,
                imported,
              },
            ],
          ];
      },),
  };
}

/**
 What a named export adds: the declaration it exports, or the entries of its list.

 @param statement - the export declaration

 @returns The declared functions, the bindings and the exports

 @example
 ```ts
 const added = gatherNamedExport({ statement, },);
 ```
 */
function gatherNamedExport({ statement, }: { readonly statement: TreeNode; },): Gathered {
  if (statement.exportKind === 'type')
    return NOTHING;
  /**
   The declaration the statement exports, `NO_NODE` for a list.
   */
  const declaration = nodeAt({
    node: statement,
    field: 'declaration',
  },);
  if (!isNoNode({ node: declaration, },)) {
    /**
     What the declaration adds.
     */
    const gathered = gatherDeclaration({ declaration, },);
    return {
      ...gathered,
      exports: gathered.bindings
        .map(function exportOf([name,],): readonly [
          string,
          ExportEntry,
        ] {
          return [
            name,
            {
              kind: 'local',
              name,
            },
          ];
        },),
    };
  }
  /**
   The module the list comes from, empty for a list of local names.
   */
  const specifier = stringValueOf({ node: statement.source, },);
  return {
    ...NOTHING,
    exports: nodesAt({
      node: statement,
      field: 'specifiers',
    },)
      .flatMap(function entryOf(entry,): readonly (readonly [
        string,
        ExportEntry,
      ])[] {
        /**
         The name the entry exports from the module or from this file.
         */
        const local = nameWrittenBy({ node: nodeAt({
          node: entry,
          field: 'local',
        },), },);
        /**
         The name the entry exports it under.
         */
        const exported = nameWrittenBy({ node: nodeAt({
          node: entry,
          field: 'exported',
        },), },);
        if ((entry.exportKind === 'type') || (local === '')
          || (exported === ''))
          return [];
        return [
          [
            exported,
            (specifier === '')
              ? {
                kind: 'local',
                name: local,
              }
              : {
                kind: 'from',
                specifier,
                imported: local,
              },
          ],
        ];
      },),
  };
}

/**
 What a default export adds: the function it declares, and the default entry for a declared function or a name.

 @param statement - the default export declaration

 @returns The declared functions, the bindings and the exports

 @example
 ```ts
 const added = gatherDefaultExport({ statement, },);
 ```
 */
function gatherDefaultExport({ statement, }: { readonly statement: TreeNode; },): Gathered {
  /**
   What the statement exports.
   */
  const declaration = nodeAt({
    node: statement,
    field: 'declaration',
  },);
  if (isNoNode({ node: declaration, },))
    return NOTHING;
  /**
   The name the default export points at: the declared function's, or the name written after `export default`.
   */
  const name = identifierName({ node: (declaration.type === 'Identifier') ? declaration : declaration.id, },);
  /**
   What the declaration adds.
   */
  const gathered = gatherDeclaration({ declaration, },);
  return (name === '')
    ? gathered
    : {
      ...gathered,
      exports: [[
        'default',
        {
          kind: 'local',
          name,
        },
      ],],
    };
}

/**
 What an `export * from` adds: the module as a star source, or, with a name, a namespace entry.

 @param statement - the export declaration

 @returns The exports and the star sources

 @example
 ```ts
 const added = gatherExportAll({ statement, },);
 ```
 */
function gatherExportAll({ statement, }: { readonly statement: TreeNode; },): Gathered {
  /**
   The module the statement re-exports.
   */
  const specifier = stringValueOf({ node: statement.source, },);
  /**
   The name of the namespace, empty for a plain `export *`.
   */
  const name = nameWrittenBy({ node: nodeAt({
    node: statement,
    field: 'exported',
  },), },);
  if ((statement.exportKind === 'type') || (specifier === ''))
    return NOTHING;
  return (name === '')
    ? {
      ...NOTHING,
      stars: [specifier,],
    }
    : {
      ...NOTHING,
      exports: [[
        name,
        {
          kind: 'namespace',
          specifier,
        },
      ],],
    };
}

/**
 What one top-level statement adds.

 @param statement - the statement

 @returns The declared functions, the bindings, the exports and the star sources

 @example
 ```ts
 const added = gatherStatement({ statement, },);
 ```
 */
function gatherStatement({ statement, }: { readonly statement: TreeNode; },): Gathered {
  if (statement.type === 'ImportDeclaration')
    return gatherImport({ statement, },);
  if (statement.type === 'ExportNamedDeclaration')
    return gatherNamedExport({ statement, },);
  if (statement.type === 'ExportDefaultDeclaration')
    return gatherDefaultExport({ statement, },);
  if (statement.type === 'ExportAllDeclaration')
    return gatherExportAll({ statement, },);
  return gatherDeclaration({ declaration: statement, },);
}

/**
 Reads one parsed file for what it declares, binds and exports.

 @param file - path of the file under `package/translation-repair`

 @param program - the file's program node

 @returns The facts of the module

 @example
 ```ts
 const facts = moduleFactsOf({ file: 'a/src/a.ts', program, },);
 ```
 */
export function moduleFactsOf(
  {
    file,
    program,
  }: {
    readonly file: string;
    readonly program: TreeNode;
  },
): ModuleFacts {
  /**
   What the top-level statements add together.
   */
  const gathered = joinedAll({
    parts: nodesAt({
      node: program,
      field: 'body',
    },)
      .map(function gatherOne(statement,): Gathered {
        return gatherStatement({ statement, },);
      },),
  },);
  return {
    file,
    holdsFormat: holdsFormatMention({ root: program, },),
    declared: gathered.declared,
    bindings: new Map(gathered.bindings,),
    exports: new Map(gathered.exports,),
    stars: gathered.stars,
    references: referencesIn({ root: program, },),
  };
}

//endregion What one module imports and exports
