import {
  ancestorsOf,
  identifierName,
  isTreeNode,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import { namesBoundBy, } from './claude-schema-blind-positions.test-fixture.ts';
import {
  isFunctionNode,
  nodeAt,
  nodesAt,
  type Visit,
  visitsUnder,
} from './claude-schema-tree.test-fixture.ts';

//region Names that a scope around a node binds
// What the check behind the schema search (`claude-schema-blind-spots.test-fixture.ts`) asks to tell a name written
// in a function from the same name at the top level of the file: whether a scope around the node binds it again. A
// name the file imports or declares at its top level is read through the resolver of
// `claude-schema-resolve.test-fixture.ts`; a function, a block, a loop or a `catch` clause that binds the name makes
// it another thing there. The scopes are read as the language gives them: the parameters, the name of a function
// expression, the declarations of a function's own body for `var`, the declarations a block lists for `let`, `const`,
// functions and classes, the declarations the body of a TypeScript namespace lists (those, and an enum, a namespace, a
// declared function and `import x = y`, also under `export`), the head of a loop, the parameter of a `catch` clause and
// the name of a class expression. The top level of the file is not one of them.

/**
 The kinds of loop whose head declares names that only the loop sees.
 */
const LOOP_KINDS: ReadonlySet<string> = new Set<string>([
  'ForStatement',
  'ForInStatement',
  'ForOfStatement',
],);

/**
 The kinds of node whose `body` lists the statements of a scope: a block, a static block and a namespace's body.
 */
const STATEMENT_LIST_KINDS: ReadonlySet<string> = new Set([
  'BlockStatement',
  'StaticBlock',
  'TSModuleBlock',
],);

/**
 The kinds of declaration that bind the name in their `id` field: a function and a class, and the declarations of a
 TypeScript namespace's body that do the same (a declared function, an enum, a namespace, an import-equals declaration).
 */
const NAMING_DECLARATION_KINDS: ReadonlySet<string> = new Set([
  'ClassDeclaration',
  'FunctionDeclaration',
  'TSDeclareFunction',
  'TSEnumDeclaration',
  'TSImportEqualsDeclaration',
  'TSModuleDeclaration',
],);

/**
 The name a declaration's `id` starts with: the name itself, or the first name of a qualified one (`namespace A.B`
 declares `A`).

 @param id - `id` of a declaration, of any kind or none

 @returns Name, empty for any other node

 @example
 ```ts
 const name = firstNameOf({ id: declaration.id, },);
 ```
 */
function firstNameOf({ id, }: { readonly id: unknown; },): string {
  return (isTreeNode(id,) && (id.type === 'TSQualifiedName'))
    ? firstNameOf({ id: id.left, },)
    : identifierName({ node: id, },);
}

/**
 The names a list of statements declares in the scope that holds the list: a variable declaration, a function, a class,
 and, in a namespace, an enum, a namespace, a declared function and an import-equals declaration, also when an export
 statement holds the declaration.

 @param statements - statements of a block, of a static block, of a `switch` case or of a namespace

 @returns Names in order

 @example
 ```ts
 const names = declaredByStatements({ statements: nodesAt({ node: block, field: 'body', },), },);
 ```
 */
function declaredByStatements({ statements, }: { readonly statements: readonly TreeNode[]; },): readonly string[] {
  return statements.flatMap(function declaredByStatement(statement,): readonly string[] {
    /**
     The declaration the statement makes: the one an export statement holds, the statement itself otherwise.
     */
    const declaration = (statement.type === 'ExportNamedDeclaration')
      ? nodeAt({
        node: statement,
        field: 'declaration',
      },)
      : statement;
    if (declaration.type === 'VariableDeclaration') {
      return nodesAt({
        node: declaration,
        field: 'declarations',
      },)
        .flatMap(function boundByDeclarator(declarator,): readonly string[] {
          return namesBoundBy({ node: declarator.id, },);
        },);
    }
    return NAMING_DECLARATION_KINDS.has(declaration.type,) ? [firstNameOf({ id: declaration.id, },),] : [];
  },);
}

/**
 Whether a visit is a function, which the walk for `var` leaves out with everything under it.

 @param node - node visited

 @returns Whether it is a function of any form

 @example
 ```ts
 const isFunction = isNestedFunction({ node, },);
 ```
 */
function isNestedFunction({ node, }: Visit,): boolean {
  return isFunctionNode({ node, },);
}

/**
 The names a function's own body declares with `var`, wherever in the body the declaration sits, a nested function
 left out.

 @param body - body of the function

 @returns Names in order

 @example
 ```ts
 const names = declaredWithVar({ body: nodeAt({ node: fn, field: 'body', },), },);
 ```
 */
function declaredWithVar({ body, }: { readonly body: TreeNode; }): readonly string[] {
  return visitsUnder({
    root: body,
    skips: isNestedFunction,
  },)
    .flatMap(function declaredByVisit({ node, },): readonly string[] {
      return ((node.type === 'VariableDeclaration') && (node.kind === 'var'))
        ? declaredByStatements({ statements: [node,], },)
        : [];
    },);
}

/**
 The names one scope binds.

 @param scope - node that may be a scope, of any kind

 @returns Names in order, none for a node that opens no scope

 @example
 ```ts
 const names = boundByScope({ scope: fn, },);
 ```
 */
function boundByScope({ scope, }: { readonly scope: TreeNode; },): readonly string[] {
  if (isFunctionNode({ node: scope, },)) {
    return [
      ...nodesAt({
        node: scope,
        field: 'params',
      },)
        .flatMap(function boundByParameter(parameter,): readonly string[] {
          return namesBoundBy({ node: (parameter.type === 'TSParameterProperty') ? parameter.parameter : parameter, },);
        },),
      ...(scope.type === 'FunctionExpression' ? [identifierName({ node: scope.id, },),] : []),
      ...declaredWithVar({
        body: nodeAt({
          node: scope,
          field: 'body',
        },),
      },),
    ];
  }
  if (STATEMENT_LIST_KINDS.has(scope.type,)) {
    return declaredByStatements({
      statements: nodesAt({
        node: scope,
        field: 'body',
      },),
    },);
  }
  if (scope.type === 'SwitchStatement') {
    return declaredByStatements({
      statements: nodesAt({
        node: scope,
        field: 'cases',
      },)
        .flatMap(function statementsOfCase(switchCase,): readonly TreeNode[] {
          return nodesAt({
            node: switchCase,
            field: 'consequent',
          },);
        },),
    },);
  }
  if (LOOP_KINDS.has(scope.type,)) {
    return declaredByStatements({
      statements: [
        nodeAt({
          node: scope,
          field: (scope.type === 'ForStatement') ? 'init' : 'left',
        },),
      ],
    },);
  }
  if (scope.type === 'CatchClause') {
    return namesBoundBy({ node: scope.param, },);
  }
  return (scope.type === 'ClassExpression') ? [identifierName({ node: scope.id, },),] : [];
}

/**
 Whether a scope around a node binds a name, so the name written there is not the one the top level of the file holds.

 @param node - node read

 @param name - name written at the node

 @param parents - each node's parent

 @returns Whether some function, block, loop, `switch`, `catch` clause or class expression around the node binds it

 @example
 ```ts
 const isShadowed = isBoundAround({ node: call, name: 'gatherThroughCache', parents, },);
 ```
 */
export function isBoundAround(
  {
    node,
    name,
    parents,
  }: {
    readonly node: TreeNode;
    readonly name: string;
    readonly parents: ReadonlyMap<TreeNode, TreeNode>;
  },
): boolean {
  return ancestorsOf({
    node,
    parents,
  },)
    .some(function bindsName(scope,): boolean {
      return boundByScope({ scope, },)
        .includes(name,);
    },);
}

//endregion Names that a scope around a node binds
