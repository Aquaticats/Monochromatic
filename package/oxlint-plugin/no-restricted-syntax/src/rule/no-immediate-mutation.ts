import type {
  Context,
  CreateOnceRule,
  ESTree,
  VisitorWithHooks,
} from '@oxlint/plugins';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  NO_INIT_INFO,
  NO_PREVIOUS_STATEMENT,
  NO_STATIC_MEMBER_NAME,
  type InitInfo,
  previousInitInfo,
  previousSiblingStatement,
  referencesIdentifier,
  staticMemberName,
  unwrapExpression,
} from './no-immediate-mutation.syntax.ts';

//region Mutation classification

/**
 Reports diagnostic for matching immediate method mutation. Bails when the
 arguments reference the target via {@link referencesIdentifier}, then
 reads the method name via {@link staticMemberName} (against the
 {@link NO_STATIC_MEMBER_NAME} sentinel).
 
 @param context - Oxlint rule context.
 
 @param call - Call expression to inspect.
 
 @param info - Previous initializer info.
 
 @returns Whether call was classified, even when allowlisted.
 
 @example
 ```ts
 reportMethodMutation({ context, call, info });
 ```
 
 @mutates context - Emits Oxlint diagnostics through foreign rule context.
 */
function reportMethodMutation(
  {
    context,
    call,
    info,
  }: ForeignBorrowed<{
    readonly context: Context;
    readonly call: ESTree.CallExpression;
    readonly info: InitInfo;
  }>,
): boolean {
  if (call.optional)
    return false;
  if (call.callee
    .type
    !== 'MemberExpression')
    return false;
  /**
   Member expression callee after narrowing.
   */
  const { callee, } = call;
  if (callee.optional)
    return false;
  if (callee.object
    .type
    !== 'Identifier')
    return false;
  /**
   Method receiver after Identifier narrowing.
   */
  const { object, } = callee;
  if (object.name !== info.name)
    return false;
  if (referencesIdentifier({
    value: call.arguments,
    name: info.name,
    visitorKeys: context.sourceCode
      .visitorKeys,
  },))
    return true;
  /**
   Static method name being called on initialized target.
   */
  const methodName = staticMemberName({ member: callee, },);
  if (methodName === NO_STATIC_MEMBER_NAME)
    return false;
  if (((info.kind === 'array') && ((methodName === 'push') || (methodName === 'unshift')))
    || ((info.kind === 'setInline') && (methodName === 'add'))
    || ((info.kind === 'mapInline') && (methodName === 'set')))
    {
    context.report({
      node: call,
      messageId: 'forbidden',
    },);
    return true;
  }
  return ((info.kind === 'setNeedsSpreadTemp') && (methodName === 'add'))
    || ((info.kind === 'mapNeedsSpreadTemp') && (methodName === 'set'));
}

/**
 Reports diagnostic for matching `Object.assign(target, ...)` mutation:
 confirms the member via {@link staticMemberName} and bails when the
 source arguments reference the target via {@link referencesIdentifier}.
 
 @param context - Oxlint rule context.
 
 @param call - Call expression to inspect.
 
 @param info - Previous initializer info.
 
 @example
 ```ts
 reportObjectAssignMutation({ context, call, info });
 ```
 
 @mutates context - Emits Oxlint diagnostics through foreign rule context.
 */
function reportObjectAssignMutation(
  {
    context,
    call,
    info,
  }: ForeignBorrowed<{
    readonly context: Context;
    readonly call: ESTree.CallExpression;
    readonly info: InitInfo;
  }>,
): void {
  if (info.kind !== 'object')
    return;
  if (call.optional)
    return;
  if (call.callee
    .type
    !== 'MemberExpression')
    return;
  /**
   Member expression callee after narrowing.
   */
  const { callee, } = call;
  if (callee.optional)
    return;
  if (callee.object
    .type
    !== 'Identifier')
    return;
  /**
   Object.assign receiver after Identifier narrowing.
   */
  const { object, } = callee;
  if (object.name !== 'Object')
    return;
  if (staticMemberName({ member: callee, }) !== 'assign')
    return;
  /**
   Arguments supplied to Object.assign.
   */
  const assignArguments = call.arguments;
  if (assignArguments.length < 2)
    return;
  /**
   Target and first source argument passed to Object.assign.
   */
  const [target, firstSource,] = assignArguments;
  if ((target === undefined) || (target.type === 'SpreadElement'))
    return;
  if (target.type !== 'Identifier')
    return;
  if (target.name !== info.name)
    return;
  if ((firstSource !== undefined) && (firstSource.type === 'SpreadElement'))
    return;
  /**
   Source arguments that would move into object initializer.
   */
  const sourceArguments = assignArguments.slice(1,);
  if (referencesIdentifier({
    value: sourceArguments,
    name: info.name,
    visitorKeys: context.sourceCode
      .visitorKeys,
  },))
    return;
  context.report({
    node: call,
    messageId: 'forbidden',
  },);
}

/**
 Reports diagnostic for matching object property assignment mutation,
 bailing when the property or right-hand side reference the target via
 {@link referencesIdentifier}.
 
 @param context - Oxlint rule context.
 
 @param assignment - Assignment expression to inspect.
 
 @param info - Previous initializer info.
 
 @example
 ```ts
 reportPropertyAssignmentMutation({ context, assignment, info });
 ```
 
 @mutates context - Emits Oxlint diagnostics through foreign rule context.
 */
function reportPropertyAssignmentMutation(
  {
    context,
    assignment,
    info,
  }: ForeignBorrowed<{
    readonly context: Context;
    readonly assignment: ESTree.AssignmentExpression;
    readonly info: InitInfo;
  }>,
): void {
  if (info.kind !== 'object')
    return;
  if (assignment.operator !== '=')
    return;
  if (assignment.left
    .type
    !== 'MemberExpression')
    return;
  /**
   Assignment target after member-expression narrowing.
   */
  const { left, } = assignment;
  if (left.object
    .type
    !== 'Identifier')
    return;
  /**
   Assigned object after Identifier narrowing.
   */
  const { object, } = left;
  if (object.name !== info.name)
    return;
  if (left.computed
    && referencesIdentifier({
      value: left.property,
      name: info.name,
      visitorKeys: context.sourceCode
        .visitorKeys,
    },))
    {
    return;
  }
  if (referencesIdentifier({
    value: assignment.right,
    name: info.name,
    visitorKeys: context.sourceCode
      .visitorKeys,
  },))
    return;
  context.report({
    node: assignment,
    messageId: 'forbidden',
  },);
}

/**
 Checks one expression statement for immediate mutation: locates the
 previous statement via {@link previousSiblingStatement} (against
 {@link NO_PREVIOUS_STATEMENT}), classifies it via {@link previousInitInfo}
 (against {@link NO_INIT_INFO}), unwraps the current expression via
 {@link unwrapExpression}, and delegates to {@link reportMethodMutation},
 {@link reportObjectAssignMutation}, or
 {@link reportPropertyAssignmentMutation}.
 
 @param context - Oxlint rule context.
 
 @param node - Expression statement being visited.
 
 @mutates context - Emits immediate-mutation diagnostics through foreign context.
 
 @example
 ```ts
 checkExpressionStatement({ context, node });
 ```
 */
function checkExpressionStatement(
  {
    context,
    node,
  }: ForeignBorrowed<{
    readonly context: Context;
    readonly node: ESTree.ExpressionStatement;
  }>,
): void {
  /**
   Previous sibling statement that may initialize this mutation target.
   */
  const previous = previousSiblingStatement({ node, },);
  if (previous === NO_PREVIOUS_STATEMENT)
    return;
  /**
   Initializer information from previous statement.
   */
  const info = previousInitInfo({
    context,
    statement: previous,
  },);
  if (info === NO_INIT_INFO)
    return;
  /**
   Expression being checked for immediate mutation.
   */
  const expression = unwrapExpression({ expression: node.expression, },);
  if (expression.type === 'CallExpression') {
    if (!reportMethodMutation({
      context,
      call: expression,
      info,
    },))
      reportObjectAssignMutation({
        context,
        call: expression,
        info,
      },);
    return;
  }
  if (expression.type !== 'AssignmentExpression')
    return;
  reportPropertyAssignmentMutation({
    context,
    assignment: expression,
    info,
  },);
}

//endregion Mutation classification

/**
 Bans immediate mutation after initialization, with Set/Map clone
 exceptions. Each expression statement is checked via
 {@link checkExpressionStatement}.
 
 This mirrors `unicorn/no-immediate-mutation` for arrays, objects, Set/Map
 constructors backed by array literals, and `Object.assign`. Unlike upstream
 oxlint 1.71.0, it allowlists `new Set(existing); set.add(value)` and
 `new Map(existing); map.set(key, value)` because folding those mutations into
 the initializer requires materializing a temporary spread array.
 
 @example
 ```ts
 const set = new Set([a]);
 set.add(b); // reported
 
 const cloned = new Set(existingSet);
 cloned.add(b); // allowed
 ```
 */
export const noImmediateMutation: CreateOnceRule = {
  meta: {
    type: 'suggestion',
    docs: {
      description:
        'Disallow immediate mutation after initialization, except Set/Map clone-plus-mutate patterns that would require a temporary spread array.',
      recommended: true,
    },
    messages: {
      forbidden: 'Move this immediate mutation into the initializer. Clone-plus-mutate Set/Map patterns are allowed when avoiding mutation would require a temporary spread array.',
    },
  },
  /**
   Creates immediate-mutation visitor.
   
   @param context - Foreign rule context receiving diagnostics.
   
   @mutates context - Emits immediate-mutation diagnostics through foreign context.
   
   @example
   ```ts
   noImmediateMutation.createOnce(context);
   ```
   */
  createOnce(context: ForeignBorrowed<Context>,): VisitorWithHooks {
    return {
      ExpressionStatement(node: ForeignBorrowed<ESTree.ExpressionStatement>,): void {
        checkExpressionStatement({
          context,
          node,
        },);
      },
    };
  },
};
