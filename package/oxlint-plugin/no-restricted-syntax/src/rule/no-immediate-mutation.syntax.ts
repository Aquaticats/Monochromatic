import type {
  Context,
  ESTree,
} from '@oxlint/plugins';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { NO_VARIABLE, } from './no-sync.constants.ts';
import { findVariable, } from './no-sync.syntax.ts';

//region Sentinels and types

/**
 Sentinel used when previous statement cannot seed an immediate-mutation report.
 */
export const NO_INIT_INFO: unique symbol = Symbol('no.immediate-mutation init info found',);

/**
 Sentinel used when member access has no static identifier property.
 */
export const NO_STATIC_MEMBER_NAME: unique symbol = Symbol('no.immediate-mutation static member name found',);

/**
 Sentinel used when expression has no supported initializer category.
 */
const NO_INITIALIZER_KIND = Symbol('no.immediate-mutation initializer kind found',);

/**
 Sentinel used when parent node has no sibling statement list.
 */
const NO_STATEMENT_LIST = Symbol('no.immediate-mutation sibling statement list found',);

/**
 Sentinel used when current statement has no previous sibling statement.
 */
export const NO_PREVIOUS_STATEMENT: unique symbol = Symbol('no.immediate-mutation previous sibling statement found',);

/**
 Initializer categories this rule can fold a following mutation into safely.
 */
type InitializerKind =
  | 'array'
    | 'object'
    | 'setInline'
    | 'mapInline'
    | 'setNeedsSpreadTemp'
    | 'mapNeedsSpreadTemp';

/**
 Previous statement initializer details needed to compare with mutation target.
 */
export type InitInfo = {
  /**
   Binding or assignment target initialized by previous statement.
   */
  readonly name: string;
  /**
   Whether following mutation can move into initializer without extra spread temp.
   */
  readonly kind: InitializerKind;
};

/**
 Result of trying to classify previous statement as immediate-mutation seed.
 */
type InitInfoResult = InitInfo | typeof NO_INIT_INFO;

/**
 Result of trying to classify an initializer expression.
 */
type InitializerKindResult = InitializerKind | typeof NO_INITIALIZER_KIND;

/**
 Parent body list that may contain current expression statement.
 */
type StatementList = readonly ESTree.Node[];

/**
 Result of looking for a statement list on the current parent.
 */
type StatementListResult = StatementList | typeof NO_STATEMENT_LIST;

/**
 Result of looking for a static member property name.
 */
type StaticMemberNameResult = string | typeof NO_STATIC_MEMBER_NAME;

/**
 Result of looking for a previous sibling statement.
 */
type PreviousStatementResult = ESTree.Node | typeof NO_PREVIOUS_STATEMENT;

//endregion Sentinels and types

//region Generic AST helpers

/**
 Returns expression with transparent wrappers removed so initializer checks see real syntax.
 
 @param expression - Expression node that may be wrapped by parentheses or TS-only casts.
 
 @returns Inner runtime expression.
 
 @example
 ```ts
 unwrapExpression({ expression });
 ```
 */
export function unwrapExpression(
  { expression, }: ForeignBorrowed<{ readonly expression: ESTree.Expression; }>,
): ESTree.Expression {
  if (expression.type === 'ParenthesizedExpression')
    return unwrapExpression({ expression: expression.expression, },);
  if (expression.type === 'TSAsExpression')
    return unwrapExpression({ expression: expression.expression, },);
  if (expression.type === 'TSSatisfiesExpression')
    return unwrapExpression({ expression: expression.expression, },);
  if (expression.type === 'TSTypeAssertion')
    return unwrapExpression({ expression: expression.expression, },);
  if (expression.type === 'TSNonNullExpression')
    return unwrapExpression({ expression: expression.expression, },);
  if (expression.type === 'TSInstantiationExpression')
    return unwrapExpression({ expression: expression.expression, },);
  return expression;
}

/**
 Returns static member property name when member access is not computed.
 
 @param member - Member expression being inspected.
 
 @returns Identifier property name, or {@link NO_STATIC_MEMBER_NAME} when
 member is computed/private.
 
 @example
 ```ts
 staticMemberName({ member });
 ```
 */
export function staticMemberName(
  { member, }: ForeignBorrowed<{ readonly member: ESTree.MemberExpression; }>,
): StaticMemberNameResult {
  if (member.computed)
    return NO_STATIC_MEMBER_NAME;
  /**
   Static member property node.
   */
  const { property, } = member;
  if (property.type !== 'Identifier')
    return NO_STATIC_MEMBER_NAME;
  return property.name;
}

/**
 Returns true when value structurally looks like requested ESTree identifier.
 
 @param value - Record-like AST node candidate.
 
 @param name - Identifier name being searched for.
 
 @returns Whether value is an Identifier with requested name.
 
 @example
 ```ts
 isIdentifierNamed({ value, name: 'set' });
 ```
 */
function isIdentifierNamed(
  {
    value,
    name,
  }: {
    readonly value: Readonly<Record<string, unknown>>;
    readonly name: string;
  },
): boolean {
  return (value.type === 'Identifier') && (value.name === name);
}

/**
 Narrows unknown foreign AST value to property-readable record.
 
 @param value - Unknown candidate supplied by AST traversal.
 
 @returns whether value is non-null object.
 */
function isAstRecord(value: unknown,): value is Readonly<Record<string, unknown>> {
  return (value !== null) && ((typeof value) === 'object');
}

/**
 Conservatively checks whether AST-ish value references an identifier
 name, matching nodes via {@link isIdentifierNamed} and recursing into
 object/array children.
 
 @param value - AST node, array, or primitive value to inspect.
 
 @param name - Identifier name that would make a rewrite unsafe.
 
 @param visitorKeys - Foreign parser child keys for each node type.
 
 @returns Whether name appears in value.
 
 @example
 ```ts
 referencesIdentifier({
   value: call.arguments,
   name: 'set',
   visitorKeys: context.sourceCode.visitorKeys,
 });
 ```
 */
export function referencesIdentifier(
  {
    value,
    name,
    visitorKeys,
  }: {
    readonly value: unknown;
    readonly name: string;
    readonly visitorKeys: Readonly<Record<string, readonly string[]>>;
  },
): boolean {
  if (Array.isArray(value,))
    return value.some(function itemReferencesIdentifier(item,): boolean {
      return referencesIdentifier({
        value: item,
        name,
        visitorKeys,
      },);
    },);
  if (!isAstRecord(value,))
    return false;
  if (isIdentifierNamed({
    value,
    name,
  },))
    return true;
  /**
   Parser-declared child fields for current node type.
   */
  const childKeys = (typeof value.type) === 'string'
    ? visitorKeys[value.type] ?? []
    : [];
  return childKeys.some(function childReferencesIdentifier(key,): boolean {
    return referencesIdentifier({
      value: value[key],
      name,
      visitorKeys,
    },);
  },);
}

//endregion Generic AST helpers

//region Initializer classification

/**
 Returns whether constructor argument would need a spread array to absorb
 mutation, after unwrapping it via {@link unwrapExpression}.
 
 @param expression - NewExpression for Set, WeakSet, Map, or WeakMap.
 
 @returns Whether folding later mutation requires extra spread-temp allocation.
 
 @example
 ```ts
 collectionNeedsSpreadTemp({ expression: newSetExpression });
 ```
 */
function collectionNeedsSpreadTemp(
  { expression, }: ForeignBorrowed<{ readonly expression: ESTree.NewExpression; }>,
): boolean {
  /**
   Constructor arguments supplied to Set or Map.
   */
  const constructorArguments = expression.arguments;
  if (constructorArguments.length !== 1)
    return false;
  /**
   Sole constructor argument; spread args cannot be extended in place.
   */
  const [argument,] = constructorArguments;
  if (argument === undefined)
    return false;
  if (argument.type === 'SpreadElement')
    return true;
  /**
   Unwrapped constructor argument expression.
   */
  const unwrappedArgument = unwrapExpression({ expression: argument, },);
  return unwrappedArgument.type !== 'ArrayExpression';
}

/**
 Classifies expression initializer for immediate-mutation rule by
 unwrapping it via {@link unwrapExpression}, checking for a shadowing
 binding via {@link findVariable} against the {@link NO_VARIABLE}
 sentinel, and sizing Set/Map constructor arguments via
 {@link collectionNeedsSpreadTemp}.
 
 @param expression - Runtime expression assigned to previous statement's target.
 
 @returns Initializer category, or the {@link NO_INITIALIZER_KIND} sentinel
 when expression is outside rule scope.
 
 @example
 ```ts
 initializerKind({ expression: declarator.init });
 ```
 */
function initializerKind(
  {
    context,
    expression,
  }: ForeignBorrowed<{
    readonly context: Context;
    readonly expression: ESTree.Expression;
  }>,
): InitializerKindResult {
  /**
   Transparent wrappers removed before syntax classification.
   */
  const unwrapped = unwrapExpression({ expression, },);
  if (unwrapped.type === 'ArrayExpression')
    return 'array';
  if (unwrapped.type === 'ObjectExpression')
    return 'object';
  if (unwrapped.type !== 'NewExpression')
    return NO_INITIALIZER_KIND;
  if (unwrapped.callee
    .type
    !== 'Identifier')
    return NO_INITIALIZER_KIND;
  /**
   NewExpression callee after Identifier narrowing.
   */
  const { callee, } = unwrapped;
  /**
   Local variable shadowing the global Set or Map constructor.
   */
  const shadowingVariable = findVariable({
    context,
    node: callee,
    name: callee.name,
  },);
  if ((shadowingVariable !== NO_VARIABLE) && (shadowingVariable.defs
    .length
    > 0))
    return NO_INITIALIZER_KIND;
  /**
   Whether Set/Map constructor consumes a non-array iterable.
   */
  const needsSpreadTemp = collectionNeedsSpreadTemp({ expression: unwrapped, },);
  if ((callee.name === 'Set') || (callee.name === 'WeakSet'))
    return needsSpreadTemp ? 'setNeedsSpreadTemp' : 'setInline';
  if ((callee.name === 'Map') || (callee.name === 'WeakMap'))
    return needsSpreadTemp ? 'mapNeedsSpreadTemp' : 'mapInline';
  return NO_INITIALIZER_KIND;
}

/**
 Classifies a variable declaration as previous immediate-mutation initializer.
 
 @param declaration - Previous variable declaration statement.
 
 @returns Init info for last declarator, classified via
 {@link initializerKind}, or {@link NO_INIT_INFO} when the declarator is
 unsupported, matching upstream rule behavior.
 
 @example
 ```ts
 initInfoFromDeclaration({ declaration });
 ```
 */
function initInfoFromDeclaration(
  {
    context,
    declaration,
  }: ForeignBorrowed<{
    readonly context: Context;
    readonly declaration: ESTree.VariableDeclaration;
  }>,
): InitInfoResult {
  /**
   Last declarator only, matching upstream unicorn/no-immediate-mutation behavior.
   */
  const declarator = declaration.declarations
    .at(-1,);
  if (declarator === undefined)
    return NO_INIT_INFO;
  if (declarator.id
    .type
    !== 'Identifier')
    return NO_INIT_INFO;
  if (declarator.init === null)
    return NO_INIT_INFO;
  /**
   Initializer syntax category for last declarator.
   */
  const kind = initializerKind({
    context,
    expression: declarator.init,
  },);
  if (kind === NO_INITIALIZER_KIND)
    return NO_INIT_INFO;
  return {
    name: declarator.id
      .name,
    kind,
  };
}

/**
 Classifies an assignment expression as previous immediate-mutation
 initializer, unwrapping the right-hand side via {@link unwrapExpression}
 and classifying it via {@link initializerKind}.
 
 @param assignment - Previous assignment expression statement.
 
 @returns Init info when assignment is `name = <initializer>`, otherwise
 {@link NO_INIT_INFO}.
 
 @example
 ```ts
 initInfoFromAssignment({ assignment });
 ```
 */
function initInfoFromAssignment(
  {
    assignment,
    context,
  }: ForeignBorrowed<{
    readonly assignment: ESTree.AssignmentExpression;
    readonly context: Context;
  }>,
): InitInfoResult {
  if (assignment.operator !== '=')
    return NO_INIT_INFO;
  if (assignment.left
    .type
    !== 'Identifier')
    return NO_INIT_INFO;
  /**
   Assignment target after Identifier narrowing.
   */
  const { left, } = assignment;
  /**
   Chained assignment cannot be rewritten into one initializer safely.
   */
  const right = unwrapExpression({ expression: assignment.right, },);
  if (right.type === 'AssignmentExpression')
    return NO_INIT_INFO;
  /**
   Initializer syntax category for assigned expression.
   */
  const kind = initializerKind({
    context,
    expression: right,
  },);
  if (kind === NO_INITIALIZER_KIND)
    return NO_INIT_INFO;
  return {
    name: left.name,
    kind,
  };
}

/**
 Classifies previous statement as initializer immediately before a
 mutation, delegating to {@link initInfoFromDeclaration} for variable
 declarations and, after unwrapping the expression via
 {@link unwrapExpression}, to {@link initInfoFromAssignment} for
 assignment expressions.
 
 @param statement - Statement immediately before current expression statement.
 
 @returns Init info for supported declaration or assignment shapes.
 
 @example
 ```ts
 previousInitInfo({ statement });
 ```
 */
export function previousInitInfo(
  {
    context,
    statement,
  }: ForeignBorrowed<{
    readonly context: Context;
    readonly statement: ESTree.Node;
  }>,
): InitInfoResult {
  if (statement.type === 'VariableDeclaration')
    return initInfoFromDeclaration({
      context,
      declaration: statement,
    },);
  if (statement.type !== 'ExpressionStatement')
    return NO_INIT_INFO;
  /**
   Previous expression after transparent wrappers.
   */
  const expression = unwrapExpression({ expression: statement.expression, },);
  if (expression.type !== 'AssignmentExpression')
    return NO_INIT_INFO;
  return initInfoFromAssignment({
    assignment: expression,
    context,
  },);
}

//endregion Initializer classification

//region Sibling statement lookup

/**
 Returns statement list containing a candidate expression statement.
 
 @param parent - Parent node of current expression statement.
 
 @returns Statement-bearing list, or {@link NO_STATEMENT_LIST} when parent
 cannot be inspected.
 
 @example
 ```ts
 siblingStatements({ parent: node.parent });
 ```
 */
function siblingStatements(
  { parent, }: ForeignBorrowed<{ readonly parent: ESTree.Node; }>,
): StatementListResult {
  if (parent.type === 'Program')
    return parent.body;
  if (parent.type === 'BlockStatement')
    return parent.body;
  if (parent.type === 'StaticBlock')
    return parent.body;
  if (parent.type === 'SwitchCase')
    return parent.consequent;
  return NO_STATEMENT_LIST;
}

/**
 Returns statement immediately before current expression statement, looked
 up via {@link siblingStatements}.
 
 @param node - Expression statement being linted.
 
 @returns Previous sibling statement, or {@link NO_PREVIOUS_STATEMENT} when
 none exists.
 
 @example
 ```ts
 previousSiblingStatement({ node });
 ```
 */
export function previousSiblingStatement(
  { node, }: ForeignBorrowed<{ readonly node: ESTree.ExpressionStatement; }>,
): PreviousStatementResult {
  /**
   Sibling statement list from nearest statement-bearing parent.
   */
  const statements = siblingStatements({ parent: node.parent, },);
  if (statements === NO_STATEMENT_LIST)
    return NO_PREVIOUS_STATEMENT;
  /**
   Current statement index by object identity.
   */
  const currentIndex = statements.findIndex(function hasSameSpan(
    statement: ForeignBorrowed<ESTree.Node>,
  ): boolean {
    return (statement.start === node.start) && (statement.end === node.end);
  },);
  if (currentIndex <= 0)
    return NO_PREVIOUS_STATEMENT;
  /**
   Previous statement from same parent body.
   */
  const previous = statements.at(currentIndex - 1,);
  if (previous === undefined)
    return NO_PREVIOUS_STATEMENT;
  return previous;
}

//endregion Sibling statement lookup
