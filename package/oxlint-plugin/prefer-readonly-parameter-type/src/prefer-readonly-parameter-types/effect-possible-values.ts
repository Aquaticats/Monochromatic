/**
 Every expression one expression can evaluate to, without evaluating anything.
 
 The store path asks what a value handed outward can be, and it had been asking by testing
 syntax: is this written as a callable. That answered for the inline form and missed the two
 shapes ordinary source actually writes, both falsified:
 
 ```ts
 holder.produce = condition ? ((): Row => config.row) : fallback;
 
 const box = { produce: (): Row => config.row, };
 holder.box = box;
 ```
 
 A branch per form is what this exists to avoid. The question is one question, so it gets one
 walk, and the callers then ask their own questions of each answer: whether it is a callable,
 or what origins it packages.
 
 Additive rather than substitutive. Every node reports itself alongside whatever it can be
 followed to, so a caller that already handled the written form keeps handling it. An
 identifier aliasing `config.row` therefore still answers with the identifier, whose origins
 the binding map holds, as well as with the initializer it was bound to.
 
 What it refuses to enter is as load-bearing as what it follows. A call is not descended,
 because a call's result is a separate relation with its own deferral machinery, and claiming
 the callee expression is a possible result would make every call look like its own callee. A
 conditional's condition, an assignment's target, a computed key, a discarded operand and any
 arithmetic operand are not values the expression evaluates to. A nested callable's body is
 not entered either, because what a callable can be is the callable, and what it can reach is
 the question `effect-callable-capture-closure.ts` answers.
 
 @module
 */

import {
  type BindingElement,
  type Identifier,
  type Node,
  SyntaxKind,
} from 'typescript/ast';
import {
  isAssertionExpression,
  isBinaryExpression,
  isBindingElement,
  isConditionalExpression,
  isIdentifier,
  isNonNullExpression,
  isObjectLiteralExpression,
  isParenthesizedExpression,
  isParameterDeclaration,
  isPropertyAccessExpression,
  isPropertyAssignment,
  isSatisfiesExpression,
  isShorthandPropertyAssignment,
  isVariableDeclaration,
} from 'typescript/ast/is';
import type { Project, } from 'typescript/sync';

import { callableDeclaration, } from './effect-call-resolution.ts';
import {
  collectAstNodes,
  isEffectCallableDeclaration,
  OWNED_CALLABLE_UNAVAILABLE,
} from './effect-summary-model.ts';

/**
 Operators whose right operand alone is the value of the expression.
 
 `&&` yields its right operand only when it yields anything derived from the left at all, and
 its left operand is discarded whenever the right is produced. Assignment yields what was
 assigned. A comma yields its right operand and evaluates the left for effect.
 */
const RIGHT_OPERAND_VALUE_OPERATORS: ReadonlySet<SyntaxKind> = new Set([
  SyntaxKind.AmpersandAmpersandToken,
  SyntaxKind.EqualsToken,
  SyntaxKind.CommaToken,
],);

/**
 Operators where either operand can be the value of the expression.
 */
const EITHER_OPERAND_VALUE_OPERATORS: ReadonlySet<SyntaxKind> = new Set([
  SyntaxKind.QuestionQuestionToken,
  SyntaxKind.BarBarToken,
],);

/**
 Collects every expression one expression can evaluate to, following aliases and branches.
 
 @param project - TypeScript project resolving identifiers to their declarations.
 
 @param node - Expression whose possible values are wanted.
 
 @returns nodes the expression can evaluate to, including the expression itself.
 
 @example
 ```ts
 possibleValueNodes({ project, node });
 ```
 */
export function possibleValueNodes({
  project,
  node,
}: {
  readonly project: Project;
  readonly node: Node;
},): readonly Node[] {
  /**
   Answers found so far, keyed by source span so an alias cycle terminates.
   */
  const seen = new Map<string, Node>();
  /**
   Expressions still to expand.
   */
  const pending: Node[] = [node,];
  while (pending.length > 0) {
    /**
     Next expression being expanded.
     */
    const current = pending.pop();
    if (current === undefined)
      continue;
    /**
     Span identifying this expression across one analysis.
     */
    const key = `${current.getSourceFile()
      .fileName}:${String(current.pos,)}:${String(current.end,)}`;
    if (seen.has(key,))
      continue;
    seen.set(
      key,
      current,
    );
    followedValues({
      project,
      node: current,
    },)
      .forEach(function enqueueFollowed(followed,): void {
        pending.push(followed,);
      },);
  }
  return [...seen.values(),];
}

/**
 Names the expressions one expression hands its value on from.
 
 @param project - TypeScript project resolving identifiers to their declarations.
 
 @param node - Expression being expanded by one step.
 
 @returns expressions whose value this expression can take.
 
 @example
 ```ts
 followedValues({ project, node });
 ```
 */
function followedValues({
  project,
  node,
}: {
  readonly project: Project;
  readonly node: Node;
},): readonly Node[] {
  if (isParenthesizedExpression(node,)
    || isNonNullExpression(node,)
    || isAssertionExpression(node,)
    || isSatisfiesExpression(node,))
    return [node.expression,];
  if (isConditionalExpression(node,))
    return [
      node.whenTrue,
      node.whenFalse,
    ];
  if (isBinaryExpression(node,)) {
    /**
     Operator deciding which operands carry the value.
     */
    const operator = node.operatorToken
      .kind;
    if (EITHER_OPERAND_VALUE_OPERATORS.has(operator,))
      return [
        node.left,
        node.right,
      ];
    if (RIGHT_OPERAND_VALUE_OPERATORS.has(operator,))
      return [node.right,];
    return [];
  }
  /* A property read names a value the same way an identifier does, and this walk followed only the
   * identifier. Measured: `registry.keep(holder.producer,)`, where `holder` is a literal carrying a
   * closure over the caller's row, recorded nothing at all and read identically to a control whose
   * literal allocates. The reach walk added elsewhere answers a different question, what a body can
   * get to, and cannot answer this one, which is what a given expression holds. */
  if (isPropertyAccessExpression(node,))
    return propertyAssignedValues({
      project,
      name: node.name,
    },);
  if (!isIdentifier(node,))
    return [];
  return aliasedInitializer({
    project,
    node,
  },);
}

/**
 Follows one property name to the values a property assignment gives it.
 
 Resolved through the name's own symbol rather than by walking the receiver, so an aliased or
 conditionally chosen holder needs no separate case: the checker already decided which declaration
 this name refers to.
 
 A shorthand assignment answers with its own name, which the worklist then expands, so
 `{ producer, }` reaches whatever `producer` holds without a second branch here.
 
 @param project - TypeScript project resolving the property name.
 
 @param name - Property name being read.
 
 @returns values the property assignments naming it hold.
 
 @example
 ```ts
 propertyAssignedValues({ project, name });
 ```
 */
function propertyAssignedValues({
  project,
  name,
}: {
  readonly project: Project;
  readonly name: Node;
},): readonly Node[] {
  return (project.checker
    .getSymbolAtLocation(name,)
    ?.declarations
    ?? [])
    .flatMap(function assignedValue(declared,): readonly Node[] {
      /**
       Declaration resolved into the project owning it.
       */
      const declaration = declared.resolve(project,);
      if (declaration === undefined)
        return [];
      if (isPropertyAssignment(declaration,))
        return [declaration.initializer,];
      if (isShorthandPropertyAssignment(declaration,))
        return [declaration.name,];
      return [];
    },);
}

/**
 Follows one identifier to the initializer it was bound to, when it has one.
 
 Only a declaration with an initializer answers. A parameter's default is one, and so is a
 binding element's: each is what the binding holds whenever nothing was supplied for it, which is
 the same relation a local declaration's initializer states. An import and a binding filled by
 assignment alone hand back nothing here, which is correct for this walk: what they can be is not
 written at their declaration, and guessing would claim a value the source never gave.
 
 A default is additive rather than substitutive, like everything else this walk reports. The
 identifier answers alongside the default, so a supplied value is never replaced by the default
 that was not used.
 
 @param project - TypeScript project resolving the identifier.
 
 @param node - Identifier being followed.
 
 @returns initializer the identifier was bound to, or nothing.
 
 @example
 ```ts
 aliasedInitializer({ project, node });
 ```
 */
function aliasedInitializer({
  project,
  node,
}: {
  readonly project: Project;
  readonly node: Identifier;
},): readonly Node[] {
  /**
   Symbol the identifier resolves to.
   */
  const symbol = project.checker
    .getResolvedSymbol(node,);
  /**
   Declaration owning that symbol.
   */
  const first = symbol
    ?.declarations
    .at(0,);
  /**
   Declaration the symbol names, preferring the one carrying its value.
   */
  const declared = symbol
    ?.valueDeclaration
    ?? first;
  /**
   Declaration resolved into the project owning it.
   */
  const declaration = declared
    ?.resolve(project,);
  if (declaration === undefined)
    return [];
  /* A parameter's default is what the binding holds whenever the argument is omitted, which is the
   * same relation a local declaration's initializer states. Parameters were excluded, so a call to
   * a parameter with a callable default resolved to the declared function type and activated
   * nothing, and gating an initializer's callables through activation would have lost a write the
   * callable genuinely performs. */
  if (isVariableDeclaration(declaration,))
    return declaration.initializer === undefined ? [] : [declaration.initializer,];
  /* A binding element's default states the same relation one step further in. `{ writer = ... } = {}`
   * declares the default on the element rather than on the parameter, and excluding it left a call
   * to that binding resolving to the declared type and reaching nothing, so a default closure
   * writing through what it receives had that write attributed to nobody. */
  /* Without a default, a binding element still names a value: whatever the pattern was filled from
   * holds at that property. Measured: `const { producer, } = { producer: (): Row => config.row, },`
   * handed to a retaining callee recorded nothing, because the element declares no initializer and
   * nothing followed the pattern back to its source. */
  if (isBindingElement(declaration,))
    return declaration.initializer === undefined
      ? destructuredPropertyValues({
        project,
        element: declaration,
      },)
      : [declaration.initializer,];
  if (!isParameterDeclaration(declaration,))
    return [];
  return declaration.initializer === undefined ? [] : [declaration.initializer,];
}

/**
 Names every callable one actual can hold.
 
 The two ways to answer see different things, and this composes them rather than choosing.
 `callableDeclaration` follows a local's initializer and stops at a parameter, so a callable
 arriving as a parameter default was named by nothing: `retain(callback,)`, where `callback`
 defaults to a closure over the caller's configuration, offered that configuration while the
 closure `retain` kept wrote through it. Falsified.
 
 Filtering the value walk to values already written as callable declarations was the first repair,
 and it answered for a default written inline and for nothing named. `possibleValueNodes` follows a
 parameter to the identifier its default names and stops there, and an identifier is not a callable
 declaration, so a default naming an ordinary function resolved to no callable at all. Measured:
 storing what a block-bodied named default handed back left the configuration offered, while the
 same callee reached directly or through a local alias charged it.
 
 So every value is now resolved rather than tested, and the results are keyed by source span so one
 declaration reached by several values answers once. That also picks up a conditional and an alias,
 which the filter missed for the same reason it missed the named default.
 
 Kept out of the callback identity beside it, which stays with the narrow resolver: naming a
 default as the callable a callee invokes would claim the default's effects for a call where the
 caller supplied something else, and that claim can be wrong in the offering direction. Whereas
 every consumer of a capture asks what the callee stated about its own formal first, so widening
 what fills a formal can only ever add an effect the callee already declared.
 
 Lives beside the value walk it is one filter over, because both the unresolved boundary and the
 reach walk need the same answer and the reach walk cannot import it from a module that imports
 the reach walk.
 
 Exported because the unresolved boundary needs the same answer. An earlier note here said a
 capture only ever adds opacity, which stopped being true once captures began feeding the mutation
 and returned-origin channels as well.
 
 @param project - TypeScript project resolving values an expression can hold.
 
 @param actual - Argument expression whose callables are wanted.
 
 @returns callables the actual can hold.
 
 @example
 ```ts
 packagedActualCallables({ project, actual });
 ```
 */
export function packagedActualCallables({
  project,
  actual,
}: {
  readonly project: Project;
  readonly actual: Node;
},): readonly Node[] {
  /**
   Callables found so far, keyed by source span so one declaration answers once however many values
   reached it.
   */
  const found = new Map<string, Node>();
  possibleValueNodes({
    project,
    node: actual,
  },)
    .forEach(function candidateCallable(value,): void {
      /**
       Callable this value is, whether written here or named.
       */
      const callable = isEffectCallableDeclaration(value,)
        ? value
        : callableDeclaration({
          project,
          node: value,
        },);
      if (callable === OWNED_CALLABLE_UNAVAILABLE)
        return;
      found.set(
        `${callable.getSourceFile()
          .fileName}:${String(callable.pos,)}:${String(callable.end,)}`,
        callable,
      );
    },);
  return [...found.values(),];
}

/**
 Follows one binding element back to the property its pattern was filled from.
 
 Answers only for a named property in an object pattern whose declaration carries an initializer.
 An array pattern, a rest element and a pattern filled by assignment all hand back nothing, which is
 correct for this walk: what they hold is not written where the binding is declared, and guessing
 would claim a value the source never gave.
 
 @param project - TypeScript project resolving the pattern's source.
 
 @param element - Binding element declaring no default of its own.
 
 @returns values the matching property holds.
 
 @example
 ```ts
 destructuredPropertyValues({ project, element });
 ```
 */
function destructuredPropertyValues({
  project,
  element,
}: {
  readonly project: Project;
  readonly element: BindingElement;
},): readonly Node[] {
  /**
   Name the element binds under, which is the property name unless it was renamed.
   */
  const bound = element.propertyName ?? element.name;
  if ((bound === undefined) || (!isIdentifier(bound,)))
    return [];
  /**
   Pattern this element belongs to.
   */
  const pattern = element.parent;
  /**
   Declaration the pattern containing this element belongs to.
   */
  const owner = pattern.parent;
  /**
   Whether the owning declaration is one that can carry an initializer.
   */
  const declaresInitializer = isVariableDeclaration(owner,)
    || isParameterDeclaration(owner,)
    || isBindingElement(owner,);
  /**
   Expression the pattern was filled from, absent when nothing was written there.
   */
  const filled = declaresInitializer ? owner.initializer : undefined;
  if (filled === undefined)
    return [];
  return propertyValuesOfLiteral({
    project,
    filled,
    propertyName: bound.text,
  },);
}

/**
 Names the values one literal gives a named property.
 
 The literal is reached through this walk rather than taken as written, so an alias, a conditional
 and a parameter default all answer, and each candidate that is not a literal contributes nothing.
 
 @param project - TypeScript project resolving what the expression holds.
 
 @param filled - Expression the pattern was filled from.
 
 @param propertyName - Property the binding takes its value from.
 
 @returns values that property holds across every literal the expression can be.
 
 @example
 ```ts
 propertyValuesOfLiteral({ project, filled, propertyName });
 ```
 */
function propertyValuesOfLiteral({
  project,
  filled,
  propertyName,
}: {
  readonly project: Project;
  readonly filled: Node;
  readonly propertyName: string;
},): readonly Node[] {
  return possibleValueNodes({
    project,
    node: filled,
  },)
    .filter(function isLiteral(value,): boolean {
      return isObjectLiteralExpression(value,);
    },)
    .flatMap(function matchingProperties(literal,): readonly Node[] {
      return collectAstNodes(literal,)
        .flatMap(function matchingProperty(member,): readonly Node[] {
          if (isPropertyAssignment(member,) && isIdentifier(member.name,)) {
            /**
             Name this assignment gives its value.
             */
            const { text, } = member.name;
            return text === propertyName ? [member.initializer,] : [];
          }
          if (isShorthandPropertyAssignment(member,) && isIdentifier(member.name,)) {
            /**
             Name this shorthand both reads and gives.
             */
            const { text, } = member.name;
            return text === propertyName ? [member.name,] : [];
          }
          return [];
        },);
    },);
}
