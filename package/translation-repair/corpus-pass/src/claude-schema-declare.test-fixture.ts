import {
  identifierName,
  isTreeNode,
  type TreeNode,
  unwrapped,
} from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import {
  type Binding,
  type DeclaredFunction,
  type Gathered,
  joinedAll,
  NOTHING,
} from './claude-schema-facts.test-fixture.ts';
import {
  functionIn,
  isFunctionNode,
  isNoNode,
  keyNameOf,
  NO_NODE,
  nodeAt,
  nodesAt,
  pathOf,
  stringValueOf,
} from './claude-schema-tree.test-fixture.ts';

//region What the declarations of a module add
// What a top-level declaration adds to the facts of its module: a function declaration, and each constant a variable
// declaration names, whether the constant holds a function, an object, a loaded module, a name or a chain of members, or
// is taken apart into members.

/**
 The module a dynamic `import()` or a `require` loads, when its path is a literal.

 @param node - expression read, of any kind or none

 @returns The path as the call writes it, empty for any other expression and for a path that is no literal

 @example
 ```ts
 const specifier = loadedModuleOf({ node: declarator.init, },);
 ```
 */
function loadedModuleOf({ node, }: { readonly node: unknown; },): string {
  /**
   The expression without the type marks and parentheses around it.
   */
  const { inner, } = unwrapped({ node, },);
  if (!isTreeNode(inner,))
    return '';
  if (inner.type === 'AwaitExpression')
    return loadedModuleOf({ node: inner.argument, },);
  if (inner.type === 'ImportExpression')
    return stringValueOf({ node: inner.source, },);
  if ((inner.type !== 'CallExpression') || (identifierName({ node: inner.callee, },) !== 'require'))
    return '';
  /**
   The first argument of the call.
   */
  const [first = NO_NODE,] = nodesAt({
    node: inner,
    field: 'arguments',
  },);
  return stringValueOf({ node: first, },);
}

/**
 The object literal an expression is, once the type marks around it are taken off.

 @param node - expression read, of any kind or none

 @returns The object literal, `NO_NODE` for any other expression

 @example
 ```ts
 const object = objectIn({ node: declarator.init, },);
 ```
 */
function objectIn({ node, }: { readonly node: unknown; },): TreeNode {
  /**
   The expression without the type marks and parentheses around it.
   */
  const { inner, } = unwrapped({ node, },);
  return (isTreeNode(inner,) && (inner.type === 'ObjectExpression')) ? inner : NO_NODE;
}

/**
 The functions among the members of an object literal, one level in, as `object.member`.

 @param owner - name of the constant that holds the object

 @param object - the object literal

 @returns One entry per member that is a method or a property holding a function; a getter, a setter and a member with
 a computed name are not

 @example
 ```ts
 const members = membersOf({ owner: 'kit', object, },);
 ```
 */
function membersOf(
  {
    owner,
    object,
  }: {
    readonly owner: string;
    readonly object: TreeNode;
  },
): readonly DeclaredFunction[] {
  return nodesAt({
    node: object,
    field: 'properties',
  },)
    .flatMap(function memberFunction(property,): readonly DeclaredFunction[] {
      /**
       The member's name, empty when it is computed, an accessor or a spread.
       */
      const name = ((property.type === 'Property') && (property.kind === 'init')
        && (property.computed !== true))
        ? keyNameOf({ key: nodeAt({
          node: property,
          field: 'key',
        },), },)
        : '';
      /**
       The function the member holds, `NO_NODE` when it holds none.
       */
      const fn = functionIn({ node: property.value, },);
      return ((name === '') || (!isFunctionNode({ node: fn, },)))
        ? []
        : [
          {
            name: `${owner}.${name}`,
            holder: property,
            fn,
          },
        ];
    },);
}

/**
 What a declarator that names a constant adds: a function, an object and its members, a module, or an alias.

 @param declarator - the declarator, whose id is a plain name

 @param name - the constant's name

 @returns The declared functions and the binding

 @example
 ```ts
 const added = gatherConstant({ declarator, name: 'build', },);
 ```
 */
function gatherConstant(
  {
    declarator,
    name,
  }: {
    readonly declarator: TreeNode;
    readonly name: string;
  },
): Gathered {
  /**
   The value the constant is assigned.
   */
  const init = nodeAt({
    node: declarator,
    field: 'init',
  },);
  /**
   The function the value is, `NO_NODE` when it is none.
   */
  const fn = functionIn({ node: init, },);
  /**
   The object literal the value is, `NO_NODE` when it is none.
   */
  const object = objectIn({ node: init, },);
  /**
   The module a dynamic import or a require of a literal path loads.
   */
  const specifier = loadedModuleOf({ node: init, },);
  /**
   The names the value is read through when it is a name or a chain of members.
   */
  const path = pathOf({ node: init, },);
  if (isFunctionNode({ node: fn, },)) {
    return {
      ...NOTHING,
      declared: [
        {
          name,
          holder: init,
          fn,
        },
      ],
      bindings: [[
        name,
        { kind: 'function', },
      ],],
    };
  }
  if (!isNoNode({ node: object, },)) {
    return {
      ...NOTHING,
      declared: membersOf({
        owner: name,
        object,
      },),
      bindings: [[
        name,
        { kind: 'object', },
      ],],
    };
  }
  if (specifier !== '') {
    return {
      ...NOTHING,
      bindings: [
        [
          name,
          {
            kind: 'module',
            specifier,
            members: [],
          },
        ],
      ],
    };
  }
  return (path.length === 0)
    ? NOTHING
    : {
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
 What a declarator that takes a value apart adds: for each plain member it names, the name it binds to and where the
 value comes from.

 @param declarator - the declarator, whose id is an object pattern

 @returns The bindings

 @example
 ```ts
 const added = gatherPattern({ declarator, },);
 ```
 */
function gatherPattern({ declarator, }: { readonly declarator: TreeNode; },): Gathered {
  /**
   The pattern the declarator takes the value apart with.
   */
  const pattern = nodeAt({
    node: declarator,
    field: 'id',
  },);
  /**
   The value that is taken apart.
   */
  const init = nodeAt({
    node: declarator,
    field: 'init',
  },);
  /**
   The module a dynamic import or a require of a literal path loads.
   */
  const specifier = loadedModuleOf({ node: init, },);
  /**
   The names the value is read through when it is a name or a chain of members.
   */
  const path = pathOf({ node: init, },);
  if (isNoNode({ node: pattern, },) || ((specifier === '') && (path.length === 0)))
    return NOTHING;
  return {
    ...NOTHING,
    bindings: nodesAt({
      node: pattern,
      field: 'properties',
    },)
      .flatMap(function bindingOf(property,): readonly (readonly [
        string,
        Binding,
      ])[] {
        /**
         The member the property takes out, empty when it is computed.
         */
        const member = (property.computed === true)
          ? ''
          : keyNameOf({ key: nodeAt({
            node: property,
            field: 'key',
          },), },);
        /**
         The node that holds the local name, or its default value.
         */
        const target = nodeAt({
          node: property,
          field: 'value',
        },);
        /**
         The local name, also when the property gives it a default value.
         */
        const local = identifierName({
          node: (target.type === 'AssignmentPattern') ? nodeAt({
            node: target,
            field: 'left',
          },) : target,
        },);
        if ((property.type !== 'Property') || (member === '')
          || (local === ''))
          return [];
        return [
          [
            local,
            (specifier === '')
              ? {
                kind: 'path',
                path: [
                  ...path,
                  member,
                ],
              }
              : {
                kind: 'module',
                specifier,
                members: [member,],
              },
          ],
        ];
      },),
  };
}

/**
 What a declaration adds: a function, or each constant a variable declaration names.

 @param declaration - the function declaration or the variable declaration

 @returns The declared functions and the bindings

 @example
 ```ts
 const added = gatherDeclaration({ declaration: statement, },);
 ```
 */
export function gatherDeclaration({ declaration, }: { readonly declaration: TreeNode; },): Gathered {
  if (declaration.type === 'FunctionDeclaration') {
    /**
     The function's name, empty for an anonymous default export.
     */
    const name = identifierName({ node: declaration.id, },);
    return (name === '')
      ? NOTHING
      : {
        ...NOTHING,
        declared: [
          {
            name,
            holder: declaration,
            fn: declaration,
          },
        ],
        bindings: [[
          name,
          { kind: 'function', },
        ],],
      };
  }
  if (declaration.type !== 'VariableDeclaration')
    return NOTHING;
  return joinedAll({
    parts: nodesAt({
      node: declaration,
      field: 'declarations',
    },)
      .map(function gatherDeclarator(declarator,): Gathered {
        /**
         The declarator's id.
         */
        const id = nodeAt({
          node: declarator,
          field: 'id',
        },);
        if (id.type === 'ObjectPattern')
          return gatherPattern({ declarator, },);
        /**
         The constant's name, empty when the id is no plain name.
         */
        const name = identifierName({ node: id, },);
        return (name === '')
          ? NOTHING
          : gatherConstant({
            declarator,
            name,
          },);
      },),
  },);
}

//endregion What the declarations of a module add
