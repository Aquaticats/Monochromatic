import {
  ancestorsOf,
  identifierName,
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
// functions and classes, the head of a loop, the parameter of a `catch` clause and the name of a class expression.
// The top level of the file is not one of them.

/**
 The kinds of loop whose head declares names that only the loop sees.
 */
const LOOP_KINDS: ReadonlySet<string> = new Set<string>([
  'ForStatement',
  'ForInStatement',
  'ForOfStatement',
],);

/**
 The names a list of statements declares in the scope that holds the list: a variable declaration, a function and a
 class.

 @param statements - statements of a block, of a static block or of a `switch` case

 @returns Names in order

 @example
 ```ts
 const names = declaredByStatements({ statements: nodesAt({ node: block, field: 'body', },), },);
 ```
 */
function declaredByStatements({ statements, }: { readonly statements: readonly TreeNode[]; },): readonly string[] {
  return statements.flatMap(function declaredByStatement(statement,): readonly string[] {
    if (statement.type === 'VariableDeclaration') {
      return nodesAt({
        node: statement,
        field: 'declarations',
      },)
        .flatMap(function boundByDeclarator(declarator,): readonly string[] {
          return namesBoundBy({ node: declarator.id, },);
        },);
    }
    if ((statement.type === 'FunctionDeclaration') || (statement.type === 'ClassDeclaration'))
      return [identifierName({ node: statement.id, },),];
    return [];
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
  if ((scope.type === 'BlockStatement') || (scope.type === 'StaticBlock')) {
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
