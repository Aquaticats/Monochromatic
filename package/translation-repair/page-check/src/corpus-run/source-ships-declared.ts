import {
  isJsonArray,
  isJsonRecord,
  parseMdxBody,
  requireMdxRefusal,
  withMdxExhaustionRefused,
} from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Source ships declared
// THE NAMES A TEXT DECLARES AS AN ES MODULE. A page is one MDX module, and acorn refuses it when two of its `import`
// or `export` lines declare one name. Two runs of the original that each declare a name parse alone, and the page
// holding both does not. The names are read from the `estree` the strict parse gives each module block, since a line
// that merely starts with the word is no module and declares nothing.

/**
 A member of an object read from the tree.

 @param node - object, or anything else the tree holds

 @param key - name of the member

 @returns The member, nothing known of it; `undefined` for a value that is no object or lacks the member

 @example
 ```ts
 const type = memberOf({ node: statement, key: 'type', },);
 ```
 */
function memberOf(
  {
    node,
    key,
  }: {
    readonly node: unknown;
    readonly key: string;
  },
): unknown {
  return isJsonRecord(node,) ? node[key] : undefined;
}

/**
 The items of a member that is a list.

 @param value - member read from the tree

 @returns The items, none for a value that is no list

 @example
 ```ts
 const items = itemsOf({ value: [1, 2,], },);
 ```
 */
function itemsOf({ value, }: { readonly value: unknown; },): readonly unknown[] {
  return isJsonArray(value,) ? value : [];
}

/**
 The name of an identifier node.

 @param node - identifier, or anything else the tree holds

 @returns The name in a list of one, none for a value that holds no name

 @example
 ```ts
 const names = nameIn({ node: { type: 'Identifier', name: 'a', }, },);
 ```
 */
function nameIn({ node, }: { readonly node: unknown; },): readonly string[] {
  /**
   The member holding the name.
   */
  const name = memberOf({
    node,
    key: 'name',
  },);
  return (typeof name) === 'string' ? [name,] : [];
}

/**
 Names an `import` or `export` statement declares for the page.

 @param statement - statement of a module block

 @returns Names, none for a statement that declares nothing

 @example
 ```ts
 const names = namesOfStatement({ statement: { type: 'ExportDefaultDeclaration', }, },);
 ```
 */
function namesOfStatement({ statement, }: { readonly statement: unknown; },): readonly string[] {
  /**
   Kind of the statement.
   */
  const type = memberOf({
    node: statement,
    key: 'type',
  },);
  if (type === 'ImportDeclaration') {
    return itemsOf({
      value: memberOf({
        node: statement,
        key: 'specifiers',
      },),
    },)
      .flatMap(function local(specifier,): readonly string[] {
        return nameIn({
          node: memberOf({
            node: specifier,
            key: 'local',
          },),
        },);
      },);
  }
  if (type === 'ExportDefaultDeclaration')
    return ['default',];
  if (type !== 'ExportNamedDeclaration')
    return [];
  /**
   What the export declares, when it declares anything.
   */
  const declaration = memberOf({
    node: statement,
    key: 'declaration',
  },);
  if (isJsonRecord(declaration,)) {
    /**
     The declarations of a variable list, none for a function or a class.
     */
    const listed = itemsOf({
      value: memberOf({
        node: declaration,
        key: 'declarations',
      },),
    },);
    return (listed.length === 0 ? [declaration,] : listed).flatMap(function named(declared,): readonly string[] {
      return nameIn({
        node: memberOf({
          node: declared,
          key: 'id',
        },),
      },);
    },);
  }
  return itemsOf({
    value: memberOf({
      node: statement,
      key: 'specifiers',
    },),
  },)
    .flatMap(function exported(specifier,): readonly string[] {
      return nameIn({
        node: memberOf({
          node: specifier,
          key: 'exported',
        },),
      },);
    },);
}

/**
 Names a text declares as modules under the strict grammar.

 @param text - text of a run, or a page

 @returns Names in order, none for a text that declares nothing and none for one the strict grammar refuses, which has
 no module of its own to declare them

 @example
 ```ts
 const names = declaredNamesOf({ text: 'import a from "b"\n', },);
 ```
 */
export function declaredNamesOf({ text, }: { readonly text: string; },): readonly string[] {
  try {
    /**
     Tree of the strict grammar.
     */
    const root = withMdxExhaustionRefused({
      parse: parseMdxBody,
      input: { body: text, },
    },);
    /**
     Blocks directly under the root.
     */
    const { children, } = root;
    return children.flatMap(function modules(block,): readonly string[] {
      if (block.type !== 'mdxjsEsm')
        return [];
      /**
       The estree the parser gave the module block.
       */
      const estree = memberOf({
        node: memberOf({
          node: block,
          key: 'data',
        },),
        key: 'estree',
      },);
      return itemsOf({
        value: memberOf({
          node: estree,
          key: 'body',
        },),
      },)
        .flatMap(function declared(statement,): readonly string[] {
          return namesOfStatement({ statement, },);
        },);
    },);
  }
  catch (error) {
    requireMdxRefusal({ error, },);
    return [];
  }
}

//endregion Source ships declared
