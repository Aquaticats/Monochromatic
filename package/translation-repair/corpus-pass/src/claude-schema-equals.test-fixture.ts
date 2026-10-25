import {
  identifierName,
  isTreeNode,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import {
  type Gathered,
  NOTHING,
} from './claude-schema-facts.test-fixture.ts';
import {
  nodeAt,
  stringValueOf,
} from './claude-schema-tree.test-fixture.ts';

//region What an import-equals declaration and an export assignment add
// `import a = require('./a.ts')` binds the module as a whole, as `import * as a from './a.ts'` does, unless the module
// has an `export =`, and then it binds the value that `export =` gives. `import a = b.c` binds a chain of names, as
// `const a = b.c` does. The declaration is the TypeScript form of both, and the files read it as the forms they stand
// for. `export = name` is an export named `export=`, which `import a = require()` binds.

/**
 The name `export = name` gives its export, which no `export` statement can write.
 */
export const EXPORT_ASSIGNMENT = 'export=';

/**
 What an import-equals declaration of a required module binds, in place of an imported name: the whole module, or the
 value of its `export =` when it has one.
 */
export const REQUIRE_FORM = '=';

/**
 The names an entity name of an import-equals declaration reads through.

 @param node - entity name: a name, or a qualified name

 @returns Names in order, none for any other node

 @example
 ```ts
 const path = entityPathOf({ node: declaration.moduleReference, },);
 ```
 */
function entityPathOf({ node, }: { readonly node: unknown; },): readonly string[] {
  if (!isTreeNode(node,))
    return [];
  if (node.type === 'TSQualifiedName') {
    return [
      ...entityPathOf({ node: node.left, },),
      identifierName({ node: node.right, },),
    ];
  }
  return (node.type === 'Identifier') ? [identifierName({ node, },),] : [];
}

/**
 What an import-equals declaration adds: the module as a whole for a require of a literal path, the chain of names for
 an entity name.

 @param declaration - import-equals declaration

 @returns Bindings, none for a declaration of types only

 @example
 ```ts
 const added = gatherImportEquals({ declaration, },);
 ```
 */
export function gatherImportEquals({ declaration, }: { readonly declaration: TreeNode; },): Gathered {
  /**
   The local name.
   */
  const name = identifierName({ node: declaration.id, },);
  /**
   What the name is bound to: a required module or an entity name.
   */
  const reference = nodeAt({
    node: declaration,
    field: 'moduleReference',
  },);
  if ((name === '') || (declaration.importKind === 'type'))
    return NOTHING;
  if (reference.type === 'TSExternalModuleReference') {
    /**
     The module path the require writes, which the grammar of the declaration admits as a string literal only.
     */
    const specifier = stringValueOf({ node: reference.expression, },);
    return {
      ...NOTHING,
      bindings: [[
        name,
        {
          kind: 'import',
          specifier,
          imported: REQUIRE_FORM,
        },
      ],],
    };
  }
  /**
   The names the entity name reads through.
   */
  const path = entityPathOf({ node: reference, },);
  return {
    ...NOTHING,
    bindings: [[
      name,
      {
        kind: 'path',
        path,
      },
    ],],
  };
}

/**
 What an export assignment adds: an export of the name it writes.

 @param declaration - export assignment

 @returns The export, none when the assignment writes something other than a name

 @example
 ```ts
 const added = gatherExportAssignment({ declaration, },);
 ```
 */
export function gatherExportAssignment({ declaration, }: { readonly declaration: TreeNode; },): Gathered {
  /**
   The name the assignment writes, empty for any other expression.
   */
  const name = identifierName({ node: nodeAt({
    node: declaration,
    field: 'expression',
  },), },);
  return (name === '')
    ? NOTHING
    : {
      ...NOTHING,
      exports: [[
        EXPORT_ASSIGNMENT,
        {
          kind: 'local',
          name,
        },
      ],],
    };
}

//endregion What an import-equals declaration and an export assignment add
