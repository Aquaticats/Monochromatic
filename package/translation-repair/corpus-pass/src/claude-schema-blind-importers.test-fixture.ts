import {
  nodesUnder,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import type { ModuleFacts, } from './claude-schema-facts.test-fixture.ts';
import type { Family, } from './claude-schema-resolve.test-fixture.ts';
import { targetOf, } from './claude-schema-target.test-fixture.ts';
import { moduleLoadOf, } from './claude-schema-tree.test-fixture.ts';

//region The files that import from a file or export from it
// What `claude-schema-blind-constants.test-fixture.ts` follows a constant through: for each file of the sources, the
// modules that import a name from it, import it as a namespace, export a name of it, export it as a namespace or export
// everything of it, and the modules that load it with `import()` or `require`.

/**
 A module that imports from a file or exports from it, as that module sees it.
 */
export type Importer =
  | {
    /**
     A name bound by an import of the file.
     */
    readonly kind: 'import';

    /**
     Path of the module that imports.
     */
    readonly file: string;

    /**
     The local name.
     */
    readonly local: string;

    /**
     The name the file exports it by, `*` for the file as a whole.
     */
    readonly imported: string;
  }
    | {
    /**
     A name the module exports from the file.
     */
      readonly kind: 'from';

    /**
     Path of the module that exports.
     */
      readonly file: string;

    /**
     The name the module exports it by.
     */
      readonly exported: string;

    /**
     The name the file exports it by.
     */
      readonly imported: string;
    }
    | {
    /**
     The file as a whole, exported by the module as a namespace.
     */
      readonly kind: 'namespace';

    /**
     Path of the module that exports.
     */
      readonly file: string;
    }
    | {
    /**
     Everything the file exports, exported by the module with `export *`.
     */
      readonly kind: 'star';

    /**
     Path of the module that exports.
     */
      readonly file: string;
    };

/**
 A module that imports from a file or exports from it, with the specifier it writes for the file.
 */
type Written = {
  /**
   The specifier as written.
   */
  readonly specifier: string;

  /**
   The module.
   */
  readonly importer: Importer;
};

/**
 What a module imports and exports from other files, with the specifier each is written by.

 @param file - path of the module

 @param facts - what the module binds, exports and exports everything of

 @returns One entry for each import, export from, namespace export and star

 @example
 ```ts
 const written = writtenBy({ file: 'a/src/a.ts', facts, });
 ```
 */
function writtenBy(
  {
    file,
    facts,
  }: {
    readonly file: string;
    readonly facts: ModuleFacts;
  },
): readonly Written[] {
  /**
   Entries found so far.
   */
  const written: Written[] = [];
  for (const [local, binding,] of facts.bindings) {
    if (binding.kind === 'import') {
      written.push({
        specifier: binding.specifier,
        importer: {
          kind: 'import',
          file,
          local,
          imported: binding.imported,
        },
      },);
    }
  }
  for (const [exported, entry,] of facts.exports) {
    if (entry.kind === 'from') {
      written.push({
        specifier: entry.specifier,
        importer: {
          kind: 'from',
          file,
          exported,
          imported: entry.imported,
        },
      },);
    }
    if (entry.kind === 'namespace') {
      written.push({
        specifier: entry.specifier,
        importer: {
          kind: 'namespace',
          file,
        },
      },);
    }
  }
  for (const specifier of facts.stars) {
    written.push({
      specifier,
      importer: {
        kind: 'star',
        file,
      },
    },);
  }
  return written;
}

/**
 The modules that import from each file of the sources or export from it, by the path of that file.

 @param family - modules of the sources

 @returns Importers, by the path of the file they read

 @example
 ```ts
 const importers = importersOf({ family, });
 ```
 */
export function importersOf({ family, }: { readonly family: Family; },): ReadonlyMap<string, readonly Importer[]> {
  /**
   Importers found so far.
   */
  const found = new Map<string, Importer[]>();
  for (const [file, facts,] of family) {
    for (const written of writtenBy({
      file,
      facts,
    },)) {
      /**
       The specifier and the module that writes it.
       */
      const {
        specifier,
        importer,
      } = written;
      /**
       The file the specifier names, empty when it names none of the sources.
       */
      const target = targetOf({
        family,
        from: file,
        specifier,
      },);
      if (target !== '') {
        found.set(
          target,
          [
            ...found.get(target,) ?? [],
            importer,
          ],
        );
      }
    }
  }
  return found;
}

/**
 The path a node loads, when it is an `import()` or a `require` of a file of the sources.

 @param family - modules of the sources

 @param file - path of the module that holds the node

 @param node - node read

 @returns Path of the file loaded, empty when the node loads none of the sources

 @example
 ```ts
 const loaded = loadedBy({ family, file, node, });
 ```
 */
function loadedBy(
  {
    family,
    file,
    node,
  }: {
    readonly family: Family;
    readonly file: string;
    readonly node: TreeNode;
  },
): string {
  if ((node.type !== 'ImportExpression') && (node.type !== 'CallExpression'))
    return '';
  /**
   The load the node is, if it is one.
   */
  const load = moduleLoadOf({ node, },);
  return load.isLoad
    ? targetOf({
      family,
      from: file,
      specifier: load.specifier,
    },)
    : '';
}

/**
 The files that load each file of the sources with `import()` or `require`, by the path of the file loaded.

 @param family - modules of the sources

 @returns Paths of the files that load, by the path of the file loaded

 @example
 ```ts
 const loaders = loadersOf({ family, });
 ```
 */
export function loadersOf({ family, }: { readonly family: Family; },): ReadonlyMap<string, readonly string[]> {
  /**
   Loaders found so far.
   */
  const found = new Map<string, string[]>();
  for (const [file, facts,] of family) {
    for (const node of nodesUnder({ root: facts.program, },)) {
      /**
       The file the node loads, empty when it loads none of the sources.
       */
      const target = loadedBy({
        family,
        file,
        node,
      },);
      if (target !== '') {
        found.set(
          target,
          [
            ...found.get(target,) ?? [],
            file,
          ],
        );
      }
    }
  }
  return found;
}

//endregion The files that import from a file or export from it
