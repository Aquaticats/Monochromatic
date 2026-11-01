import type { TreeNode, } from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import {
  type Importer,
  importersOf,
  loadersOf,
} from './claude-schema-blind-importers.test-fixture.ts';
import {
  type BlindContext,
  type ConstantReader,
  type ConstantReaders,
  namesBoundBy,
} from './claude-schema-blind-positions.test-fixture.ts';
import { readsOfName, } from './claude-schema-blind-reads.test-fixture.ts';
import {
  EXPORT_ASSIGNMENT,
  REQUIRE_FORM,
} from './claude-schema-equals.test-fixture.ts';
import { drained, } from './claude-schema-drain.test-fixture.ts';
import type { Family, } from './claude-schema-resolve.test-fixture.ts';
import {
  isNoNode,
  keyNameOf,
  NO_NODE,
  nodeAt,
  nodesAt,
} from './claude-schema-tree.test-fixture.ts';

//region The files that read a constant another file exports
// The third total follows a constant through its references (`claude-schema-blind-flow.test-fixture.ts`). A constant
// that a file exports is read in other files too, and the walk goes there: by the name an import gives it, by an
// alias, through a module that exports it again by name or by a star, through a member of a namespace import, and
// through an `import()` or a `require` of the file. A namespace import that is read as a whole, a namespace that is
// exported, and a load of the file are reported at once, since what they read of the file cannot be told. An export
// list or an `export from` is no read: it only gives the name to other files, which the walk then follows.

/**
 A name a file exports.
 */
type Exported = {
  /**
   Path of the file.
   */
  readonly file: string;

  /**
   The name it exports under.
   */
  readonly exported: string;
};

/**
 What one importer adds to the walk of an export: the places that read it and the exports it gives the name to.
 */
type Step = {
  /**
   The places the importer reads the export.
   */
  readonly readers: readonly ConstantReader[];

  /**
   The exports of the importer that carry the export on.
   */
  readonly exports: readonly Exported[];
};

/**
 The names under which a module exports a name bound at its top level: by `export const` and its kin, by an export
 list, and by `export default`.

 @param context - what the module's names are read against

 @param name - name bound at the top level

 @returns Names it is exported by, none when the module does not export it

 @example
 ```ts
 const exported = exportedAs({ context, name: 'FORMAT', });
 // => ['FORMAT']
 ```
 */
function exportedAs(
  {
    context,
    name,
  }: {
    readonly context: BlindContext;
    readonly name: string;
  },
): readonly string[] {
  /**
   Names found so far.
   */
  const found: string[] = [];
  /**
   The program of the module and what it exports.
   */
  const {
    program,
    exports,
  } = context.facts;
  for (const statement of nodesAt({
    node: program,
    field: 'body',
  },)) {
    /**
     The declaration the statement exports, if it exports one.
     */
    const declaration = nodeAt({
      node: statement,
      field: 'declaration',
    },);
    if (declaration.type === 'VariableDeclaration') {
      for (const declarator of nodesAt({
        node: declaration,
        field: 'declarations',
      },)) {
        if (namesBoundBy({ node: declarator.id, },)
          .includes(name,))
          found.push(name,);
      }
    }
  }
  for (const [exported, entry,] of exports) {
    if ((entry.kind === 'local') && (entry.name === name))
      found.push(exported,);
  }
  return found;
}

/**
 The member a name is the object of, when the member has a plain name.

 @param context - what the file's names are read against

 @param head - identifier that reads the name

 @returns Member, `NO_NODE` when the name is no object of a member or the member is computed

 @example
 ```ts
 const member = plainMemberOf({ context, head, });
 ```
 */
function plainMemberOf(
  {
    context,
    head,
  }: {
    readonly context: BlindContext;
    readonly head: TreeNode;
  },
): TreeNode {
  /**
   The node that holds the identifier.
   */
  const holder = context.parents
    .get(head,);
  if (holder?.type !== 'MemberExpression')
    return NO_NODE;
  // A reference that sits in a member without being its computed key is its object, so the object needs no test.
  return (holder.computed !== true) ? holder : NO_NODE;
}

/**
 The places a file reads a namespace import of a file of the sources, for one export.

 @param context - what the file's names are read against

 @param local - name of the namespace import

 @param exported - export read

 @returns Readers: the member of that name where it is read as one, the name alone where it is read as a whole or by a
 computed member, and none where another member is read

 @example
 ```ts
 const readers = namespaceReadersOf({ context, local: 'formats', exported: 'FORMAT', });
 ```
 */
function namespaceReadersOf(
  {
    context,
    local,
    exported,
  }: {
    readonly context: BlindContext;
    readonly local: string;
    readonly exported: string;
  },
): readonly ConstantReader[] {
  /**
   The program of the file.
   */
  const { program, } = context.facts;
  return readsOfName({
    context,
    name: local,
    scope: program,
  },)
    .flatMap(function readerOf(head,): readonly ConstantReader[] {
      /**
       The member the identifier is the object of, `NO_NODE` when it is none or the member is computed.
       */
      const member = plainMemberOf({
        context,
        head,
      },);
      /**
       The name of that member, empty when there is none.
       */
      const read = isNoNode({ node: member, },)
        ? ''
        : keyNameOf({
          key: nodeAt({
            node: member,
            field: 'property',
          },),
        },);
      if ((read !== '') && (read !== exported))
        return [];
      return [
        {
          context,
          node: (read === '') ? head : member,
          stray: '',
        },
      ];
    },);
}

/**
 What one importer adds to the walk of an export.

 @param context - what the importing module's names are read against

 @param importer - importer

 @param item - export followed

 @returns Places the importer reads it and the exports that carry it on

 @example
 ```ts
 const step = stepOfImporter({ context, importer, item, });
 ```
 */
function stepOfImporter(
  {
    context,
    importer,
    item,
  }: {
    readonly context: BlindContext;
    readonly importer: Importer;
    readonly item: Exported;
  },
): Step {
  if (importer.kind === 'from') {
    return {
      readers: [],
      exports: (importer.imported === item.exported)
        ? [
          {
            file: importer.file,
            exported: importer.exported,
          },
        ]
        : [],
    };
  }
  if (importer.kind === 'star') {
    return {
      readers: [],
      exports: (item.exported === 'default')
        ? []
        : [
          {
            file: importer.file,
            exported: item.exported,
          },
        ],
    };
  }
  if (importer.kind === 'namespace') {
    return {
      readers: [
        {
          context,
          node: NO_NODE,
          stray: `namespace export of ${item.file} in ${importer.file}`,
        },
      ],
      exports: [],
    };
  }
  /**
   Whether the import is the module as a whole: a namespace import, or the require form of an import-equals declaration.
   */
  const isWhole = (importer.imported === '*') || (importer.imported === REQUIRE_FORM);
  if ((importer.imported !== item.exported) && (!isWhole)) {
    return {
      readers: [],
      exports: [],
    };
  }
  /**
   The program of the importing module.
   */
  const { program, } = context.facts;
  return {
    readers: (isWhole && (item.exported !== EXPORT_ASSIGNMENT))
      ? namespaceReadersOf({
        context,
        local: importer.local,
        exported: item.exported,
      },)
      : readsOfName({
        context,
        name: importer.local,
        scope: program,
      },)
        .map(function readerOf(node,): ConstantReader {
          return {
            context,
            node,
            stray: '',
          };
        },),
    exports: exportedAs({
      context,
      name: importer.local,
    },)
      .map(function exportOf(exported,): Exported {
        return {
          file: importer.file,
          exported,
        };
      },),
  };
}

/**
 Where the other files of the sources read a constant that a file exports.

 @param family - modules of the sources

 @param contextOf - what the names of a file are read against

 @returns Places the constant is read, each with the context of its file and the node that holds the value there

 @example
 ```ts
 const readers = constantReadersOf({ family, contextOf, });
 const places = readers({ file: 'a/src/a.ts', name: 'FORMAT', });
 ```
 */
export function constantReadersOf(
  {
    family,
    contextOf,
  }: {
    readonly family: Family;
    readonly contextOf: (input: { readonly file: string; },) => BlindContext;
  },
): ConstantReaders {
  /**
   What imports from each file.
   */
  const importers = importersOf({ family, });
  /**
   What loads each file, read when the first constant asks.
   */
  const loaders = new Map<'all', ReadonlyMap<string, readonly string[]>>();
  return function readersOf(
    {
      file,
      name,
    },
  ): readonly ConstantReader[] {
    if (!loaders.has('all',)) {
      loaders.set(
        'all',
        loadersOf({ family, }),
      );
    }
    /**
     What loads each file.
     */
    const loading = loaders.get('all',) ?? new Map<string, readonly string[]>();
    /**
     The exports followed, so an export that leads back to itself ends.
     */
    const followed = new Set<string>();
    /**
     The places found.
     */
    const readers: ConstantReader[] = [];
    /**
     The exports still to follow.
     */
    const pending: Exported[] = exportedAs({
      context: contextOf({ file, },),
      name,
    },)
      .map(function exportOf(exported,): Exported {
        return {
          file,
          exported,
        };
      },);
    for (const item of drained({ stack: pending, },)) {
      if (!followed.has(`${item.file}#${item.exported}`,)) {
        followed.add(`${item.file}#${item.exported}`,);
        for (const importer of importers.get(item.file,) ?? []) {
          /**
           What the importer adds.
           */
          const step = stepOfImporter({
            context: contextOf({ file: importer.file, },),
            importer,
            item,
          },);
          readers.push(...step.readers,);
          pending.push(...step.exports,);
        }
        for (const loader of loading.get(item.file,) ?? []) {
          readers.push({
            context: contextOf({ file: loader, },),
            node: NO_NODE,
            stray: `import() or require of ${item.file} in ${loader}`,
          },);
        }
      }
    }
    return readers;
  };
}

//endregion The files that read a constant another file exports
