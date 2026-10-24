import {
  identifierName,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import { stringValueOf, } from './claude-schema-tree.test-fixture.ts';

//region What one module declares, imports and exports
// The facts `claude-schema-resolve.test-fixture.ts` resolves names through, and the shapes the reading of one file adds
// them up from: the functions a file declares at the top level, the names its top level binds (a function, an object, an
// import, an alias, a destructured member, a dynamic import), what it exports and the names its code refers to.

/**
 A function a module declares at its top level, or a member of an object a top-level constant holds.
 */
export type DeclaredFunction = {
  /**
   The function's name, `object.member` for the member of an object.
   */
  readonly name: string;

  /**
   The part of the declaration that holds the function: the declaration, the value a constant holds, or the member.
   */
  readonly holder: TreeNode;

  /**
   The function itself.
   */
  readonly fn: TreeNode;
};

/**
 What a name bound at the top level of a module stands for, before any other module is read.
 */
export type Binding =
  | {
    /**
     The name is a function the module declares.
     */
    readonly kind: 'function';
  }
    | {
    /**
     The name is an object a constant holds, whose members are read as `name.member`.
     */
      readonly kind: 'object';
    }
    | {
    /**
     The name is brought in by an import.
     */
      readonly kind: 'import';

    /**
     The module the name comes from, as the import writes it.
     */
      readonly specifier: string;

    /**
     The name the module exports it by: a name, `default`, or `*` for the module as a whole.
     */
      readonly imported: string;
    }
    | {
    /**
     The name is another name or a chain of members, as in `const make = kit.build`.
     */
      readonly kind: 'path';

    /**
     The names the value is read through.
     */
      readonly path: readonly string[];
    }
    | {
    /**
     The name is a module loaded by a dynamic `import()` or a `require` of a literal path, or a member of it.
     */
      readonly kind: 'module';

    /**
     The module's path as the call writes it.
     */
      readonly specifier: string;

    /**
     The members read off the module before the name: none for the module itself, one for a destructured member.
     */
      readonly members: readonly string[];
    };

/**
 What a module exports under one name.
 */
export type ExportEntry =
  | {
    /**
     The name is bound at the top level of this module.
     */
    readonly kind: 'local';

    /**
     The local name.
     */
    readonly name: string;
  }
    | {
    /**
     The name is another module's export, as in `export { a as b } from`.
     */
      readonly kind: 'from';

    /**
     The module as the export writes it.
     */
      readonly specifier: string;

    /**
     The name the other module exports it by.
     */
      readonly imported: string;
    }
    | {
    /**
     The name is another module as a whole, as in `export * as ns from`.
     */
      readonly kind: 'namespace';

    /**
     The module as the export writes it.
     */
      readonly specifier: string;
    };

/**
 One parsed file, read for the names it binds and exports.
 */
export type ModuleFacts = {
  /**
   Path of the file under `package/translation-repair`.
   */
  readonly file: string;

  /**
   Whether the file writes a format or names the type of one.
   */
  readonly holdsFormat: boolean;

  /**
   The functions the file declares, in order.
   */
  readonly declared: readonly DeclaredFunction[];

  /**
   What each name bound at the top level stands for.
   */
  readonly bindings: ReadonlyMap<string, Binding>;

  /**
   What the module exports, by the name it exports it under.
   */
  readonly exports: ReadonlyMap<string, ExportEntry>;

  /**
   The modules the file exports everything of, as in `export * from`, in order.
   */
  readonly stars: readonly string[];

  /**
   The names the file's code refers to, each as a path.
   */
  readonly references: readonly (readonly string[])[];
};

/**
 What one top-level statement adds to the facts of its module.
 */
export type Gathered = {
  /**
   The functions it declares.
   */
  readonly declared: readonly DeclaredFunction[];

  /**
   The names it binds, in order.
   */
  readonly bindings: readonly (readonly [
    string,
    Binding,
  ])[];

  /**
   The names it exports, in order.
   */
  readonly exports: readonly (readonly [
    string,
    ExportEntry,
  ])[];

  /**
   The modules it exports everything of.
   */
  readonly stars: readonly string[];
};

/**
 What a statement that adds nothing adds.
 */
export const NOTHING: Gathered = {
  declared: [],
  bindings: [],
  exports: [],
  stars: [],
};

/**
 Two sets of facts as one.

 @param first - facts so far

 @param second - facts to add

 @returns The first, then the second

 @example
 ```ts
 const both = joined({ first: NOTHING, second: NOTHING, },);
 ```
 */
function joined(
  {
    first,
    second,
  }: {
    readonly first: Gathered;
    readonly second: Gathered;
  },
): Gathered {
  return {
    declared: [
      ...first.declared,
      ...second.declared,
    ],
    bindings: [
      ...first.bindings,
      ...second.bindings,
    ],
    exports: [
      ...first.exports,
      ...second.exports,
    ],
    stars: [
      ...first.stars,
      ...second.stars,
    ],
  };
}

/**
 Many sets of facts as one.

 @param parts - facts to join, in order

 @returns The parts, one after the other

 @example
 ```ts
 const all = joinedAll({ parts: [NOTHING, NOTHING,], },);
 ```
 */
export function joinedAll({ parts, }: { readonly parts: readonly Gathered[]; },): Gathered {
  return parts.reduce(
    function add(
      first,
      second,
    ): Gathered {
      return joined({
        first,
        second,
      },);
    },
    NOTHING,
  );
}

/**
 The name an import or an export writes, as a plain name or a string in quotes.

 @param node - the node that holds the name, of any kind or none

 @returns The name, empty for any other node

 @example
 ```ts
 const name = nameWrittenBy({ node: specifier.imported, },);
 ```
 */
export function nameWrittenBy({ node, }: { readonly node: unknown; },): string {
  return identifierName({ node, },) || stringValueOf({ node, },);
}

//endregion What one module declares, imports and exports
