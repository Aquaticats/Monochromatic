/**
 Whether a returned result's callers can all be seen to substitute for it.
 
 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type {
  CallExpression,
  Node,
} from 'typescript/ast';
import { isCallExpression, } from 'typescript/ast/is';
import type {
  Checker,
  Project,
} from 'typescript/sync';

import {
  callResultElementReceiver,
  callResultReceiver,
  RESULT_NOT_RECEIVER_STATE,
} from './effect-member-result-relation.ts';

import {
  bindingDeclarationInitializer,
  bindingIsReassignable,
  NO_BINDING_INITIALIZER,
} from './effect-binding-initializer.ts';
import { bindingAssignedWithin, } from './effect-binding-assignment.ts';
import { resultEscapesCallable, } from './effect-result-escape.ts';
import { callersAreEnumerable, } from './effect-caller-enumeration.ts';
import { writtenDirectlyInBody, } from './effect-enclosing-callable.ts';
import {
  NOTHING_WRAPPED,
  transparentOperand,
} from './effect-expression-provenance.ts';
import { expressionContainsForeignBorrowed, } from './foreign-borrowed-classifier.ts';
import {
  memberCallReceiver,
  NO_MEMBER_RECEIVER,
} from './effect-member-call-receiver.ts';
import { isEffectCallableDeclaration, } from './effect-summary-model.ts';

/**
 Returned-result discharge logger.
 */
const l = tagged({ tag: 'effect-returned-result-discharge', },);

/**
 Tests whether a call's result is a verified piece of its own receiver's state.
 
 Either relation answers it, and the two are the same claim at different arity: a container
 result holds the receiver's elements and a direct result is one of them. The receiver-chain
 descent needs only that the receiver governs what comes back, so both qualify and neither
 alone does.
 
 @param project - TypeScript project proving default-library ownership.
 
 @param checker - TypeScript checker resolving signature and types.
 
 @param call - Call whose result provenance is in question.
 
 @returns whether a verified relation names the receiver as the result's source.
 
 @example
 ```ts
 callResultIsReceiverState({ project, checker, call });
 ```
 */
function callResultIsReceiverState({
  project,
  checker,
  call,
}: {
  readonly project: Project;
  readonly checker: Checker;
  readonly call: CallExpression;
},): boolean {
  return (callResultElementReceiver({
    project,
    checker,
    call,
  },) !== RESULT_NOT_RECEIVER_STATE)
    || (callResultReceiver({
      project,
      checker,
      call,
    },) !== RESULT_NOT_RECEIVER_STATE);
}

/**
 Tests whether every caller of the callable owning a body is one this analysis can see.
 
 The precondition `effect-result-escape.ts` names. Returning parameter-reachable state is
 benign by accepted policy, but the escape charge stands "until a caller substitutes
 through `directReturned`", so discharging it requires knowing every caller is one the
 fixed point will reach.
 
 Asked here rather than through `completeForeignBorrowedGraph`, which answers the same
 question while building an ownership fixed point this decision does not need. The
 underlying query is one checker call, measured at roughly one to two milliseconds per
 declaration on workspace source, so asking it directly costs a fraction of that graph and
 needs none of its state threaded into this layer.
 
 Completeness means what it means for that graph, and what that is changed. Both used to
 read "every usage TypeScript can enumerate resolves" as completeness, which is true of an
 exported callable with one in-program caller while consumers outside the program go
 unenumerated. The argument for keeping it was that the two mechanisms must agree about
 identical callables.
 
 That argument was wrong in a way worth recording, because it is nearly right. The two do
 have to share a notion, but the one they shared was sound for only one of them: the
 ownership graph over-approximates and adds charges, while this under-approximates and
 removes one, and an enumeration that may be missing callers is safe to trust only in the
 first direction. So the shared predicate was strengthened rather than forked, and
 `callersAreEnumerable` is now asked by both.
 
 @param project - TypeScript project enumerating signature usage.
 
 @param body - Body of the callable whose callers are in question.
 
 @returns whether every enumerable usage resolves.
 
 @example
 ```ts
 callersAllResolve({ project, body });
 ```
 */
function callersAllResolve({
  project,
  body,
}: {
  readonly project: Project;
  readonly body: Node;
},): boolean {
  /**
   Callable owning this body, which is what usage is enumerated for.
   */
  const declaration = body.parent;
  if (!isEffectCallableDeclaration(declaration,))
    return false;
  /* Whether the enumeration can be complete at all, asked before what it contains. A callable
   * other files may import has callers `getSignatureUsage` never sees, and requiring a
   * non-empty result does not reach that: one in-program call satisfies it while a consumer
   * outside the program writes through the returned container unattributed. */
  if (!callersAreEnumerable({
    project,
    declaration,
  },))
    return false;
  try {
    /**
     Every usage of this callable's signature the project can enumerate.
     */
    const usages = project.languageService
      .getSignatureUsage(declaration,);
    /* An empty enumeration is refused rather than accepted, and the distinction is the
     * whole soundness of this test. `every` over nothing is true, so a callable with no
     * caller in the program would discharge vacuously, and a callable with no caller in the
     * program is exactly the one whose callers this analysis cannot see: an exported
     * `returnedLookupEffect` handing back a `Set` the caller owns has consumers outside
     * this repository that no enumeration reaches. Requiring a caller means the discharge
     * rests on substitution that demonstrably happens rather than on the absence of
     * evidence. */
    if (usages.length === 0)
      return false;
    return usages
      .every(function usageResolves(usage,): boolean {
        /**
         Call this usage stands for, absent when it is not one this project resolves.
         */
        const resolved = usage.call
          ?.resolve(project,);
        return resolved !== undefined;
      },);
  }
  catch (error) {
    /* Failing closed, and the only correct direction here. An enumeration this cannot
     * perform is exactly the case where callers may exist unseen, so the charge stands. */
    l.error(`signature usage unavailable, keeping the escape charge: ${String(error,)}`,);
    return false;
  }
}

/**
 Tests whether a returned result's escape charge is answered by its callers.
 
 Returning is the one escape whose destination this analysis can follow, and only where it
 can see everyone who receives it. Three conditions, each added for a measured case rather
 than for symmetry.
 
 Returning must be the result's only escape, which is what the position condition became.
 It began as a syntactic test for `return rows.slice(0,);` exactly, and that is the narrowest
 case of the question rather than the question: a call bound to a `const` and then returned,
 or wrapped in an assertion, hands the caller the same value by the same route. Asking the
 escape test with this body's returns attributed answers all of them at once, and refuses a
 result that also leaves by a route no caller substitutes for, such as a store into module
 state.
 
 Every caller must be enumerable and resolvable, because a caller the fixed point never
 visits never substitutes, and the returned fact would then be recorded and read by nobody.
 
 And the receiver must not carry foreign-borrowed state, which is the condition
 enumeration found rather than reasoning. Discharging without it silenced
 `filterForeignFixtureTree`, `filterAliasedForeignFixtureTree` and `sortForeignFixtureTree`,
 and `effect-opaque-boundary.ts` records the first two as having "lost their finding
 entirely" once already, diagnosed as a defect and fixed. `ForeignBorrowed` marks an
 ownership boundary, and a container returned out of foreign-owned state is not something
 an in-program caller accounts for however completely it enumerates: what the caller
 substitutes is provenance, and the effects on that side of the boundary are what this
 analysis does not model.
 
 @param project - TypeScript project resolving usage and ownership.
 
 @param call - Call whose result carries the escape charge.
 
 @param body - Body of the callable returning it.
 
 @returns whether the charge is answered and may be discharged.
 
 @example
 ```ts
 returnedResultDischargeable({ project, call, body });
 ```
 */
export function returnedResultDischargeable({
  project,
  call,
  body,
}: {
  readonly project: Project;
  readonly call: CallExpression;
  readonly body: Node;
},): boolean {
  /* Returning has to be the *only* escape, which is a different question from the call being
   * the returned expression. Asked of the escape test with returns attributed: if the result
   * still escapes with this body's own returns discounted, some other route carries it out and
   * no caller substitution accounts for that one. `localBoundAndStoredElements` stores its copy
   * in module state and returns it, and must keep its report for the store alone.
   *
   * Replaces a syntactic test that required `return rows.slice(0,);` exactly. That form is the
   * narrowest case of this one, and the wider question is the one the discharge always meant:
   * a call bound to a `const` and then returned, or wrapped in an assertion, hands the caller
   * the same value by the same route, and the escape test already follows a result through its
   * holders. */
  if (resultEscapesCallable({
    project,
    body,
    call,
    elementStepsAttributed: true,
    returnsAttributed: true,
  },))
    return false;
  /* Returned by *this* callable, not merely by some callable. `callIsReturnedOutright`
   * accepts a `ReturnStatement` wherever it is written, and the callers enumerated below are
   * enumerated for the body handed in, so a call returned from a nested declaration decides
   * its discharge on a different callable's callers.
   *
   * MASKED, measured 2026-08-07: removing this changes no diagnostic. `resultEscapesCallable`
   * treats any reference inside a nested callable as escaping outright, on the ground that a
   * captured use outlives its reasoning about statement order, so a parameter reached from
   * inside the inner declaration is charged whether or not this refuses. Kept because that
   * masking lives in another module and is not a property of this decision. */
  if (!writtenDirectlyInBody({
    node: call,
    body,
  },))
    return false;
  /* Asked at the base of the receiver chain rather than at the immediate receiver, because
   * the shapes this exists for compose members. `tree.children.slice().filter(observer,)`
   * has a call for its receiver, and a call carries no ownership of its own, so asking the
   * immediate receiver answered no and discharged all three foreign cases anyway. Measured
   * before and after adding this descent. */
  /**
   Base of the receiver chain, with composed member calls descended.
   */
  const base: { current: Node | typeof NO_MEMBER_RECEIVER; } = {
    current: memberCallReceiver({ call, },),
  };
  /**
   Expressions already descended through, so an alias cycle cannot spin this.
   */
  const visited = new Set<Node>();
  while (base.current !== NO_MEMBER_RECEIVER) {
    /* A repeat ends the walk by refusing it rather than by falling out of it. Reaching a
     * node twice means the descent produced no base, and a walk proving absence has to read
     * "no answer" as "not proven": ending the loop and classifying whatever the cursor last
     * held would report a cycle clean. Ending it at all is still required, since nothing
     * else bounds the alias hops.
     *
     * Believed unreachable, and this is reasoning rather than a measurement, which the
     * annotations beside it earn the right to distinguish. Only `const` bindings are followed
     * here, so a cycle needs each to read the next before it is initialised, and TypeScript
     * rejects that as use before declaration. No program was written to confirm it.
     *
     * Kept regardless, because it is the loop's termination answer and has to be given
     * whether or not anything reaches it. The alternative is not "no check" but "fall out of
     * the loop and classify whatever the cursor holds", which is the wrong answer. */
    if (visited.has(base.current,))
      return false;
    visited.add(base.current,);
    /* Runtime-transparent wrappers are stripped before any structural test, because every
     * test below asks what kind of expression this is and a wrapper answers for itself.
     * `(owned.map(lift,) as Row[]).slice(0,)` is an assertion rather than a call, so the
     * relation requirement never ran and the walk stopped at the wrapper and classified it,
     * which is the same provenance hole that requirement closes wearing a cast. `as`,
     * parentheses, `!` and `satisfies` all do it. Shared with the provenance walk rather
     * than restated, so the two cannot disagree about which forms erase.
     *
     * NOT masked, and the first guess that it was is worth recording as wrong. It looked
     * masked because the relation requirement it protects is masked, but that is not the only
     * thing a wrapper hides: `bindingAssignedWithin` can answer only about an `Identifier`, so
     * an assertion around the base hides the name from it and the written-endpoint check
     * passes on a parameter that was pointed elsewhere. Removing this offers four parameters
     * that must not be offered, pinned by `localAssertedRepointedElements`. A structural test
     * is only as good as its ability to see what it is testing. */
    /**
     Inner expression, when this base is a wrapper whose value is exactly its operand's.
     */
    const unwrapped = transparentOperand({ node: base.current, },);
    if (unwrapped !== NOTHING_WRAPPED) {
      base.current = unwrapped;
      continue;
    }
    if (isCallExpression(base.current,)) {
      /* Descended only where a verified relation says the result is the receiver's own
       * state. Following every member call syntactically assumes what the relation exists
       * to prove: `local.map(function lift() { return foreign; },).slice(0,)` reaches
       * `local` and reports a clean base, while every element of the returned container
       * came from the observer instead. `map` and `flatMap` carry no receiver relation for
       * exactly that reason, so asking for one turns the assumption into a test.
       *
       * MASKED, measured 2026-08-07: removing this changes no diagnostic. Every member
       * without a receiver relation takes an observer, and the observer path marks the
       * receiver opaque on its own, so the shapes this refuses are charged anyway. The two
       * facts are independent, and a member gaining a relation-free result without an
       * observer would separate them. */
      if (!callResultIsReceiverState({
        project,
        checker: project.checker,
        call: base.current,
      },))
        return false;
      base.current = memberCallReceiver({ call: base.current, },);
      continue;
    }
    /* Names are descended too, and nearly the same hop the element walk uses.
     * `const copied = tree.children.slice(); return copied.filter(observer,);` puts a name
     * where the composed form puts a call, and stopping at the name left
     * `filterAliasedForeignFixtureTree` discharged while its two siblings were restored.
     * Its own doc calls it the aliased spelling of the case beside it, so the two agreeing
     * is the point of it existing.
     *
     * A reassignable name is refused before the hop rather than stopped at, and the two
     * differ. That hop ignores later assignment, which its own doc calls "the
     * over-attributing direction and deliberate": a reassigned local keeps answering for
     * the container it was declared with, costing precision and never an offer. Read
     * backwards for a *negative* ownership proof the same property is unsound, since
     * `let held = owned; held = foreign; return held.filter(keep,);` proves clean from an
     * initializer the receiver no longer holds. Stopping at the name instead proves it
     * clean a second way, because the name carries the type the initializer gave it, so
     * only refusing outright answers correctly. `declaredConst` already carried this
     * argument for the container record, against the same `let` shape. */
    if (bindingIsReassignable({
      project,
      checker: project.checker,
      node: base.current,
    },))
      return false;
    /**
     Value this name was declared with, when it names one local declaration.
     */
    const declared = bindingDeclarationInitializer({
      project,
      checker: project.checker,
      node: base.current,
    },);
    if (declared === NO_BINDING_INITIALIZER)
      break;
    base.current = declared;
  }
  /* UNREACHABLE by construction rather than masked, which is a different claim and the one
   * the code actually supports. Both entry points are closed. The gates reach this only for a
   * member call: `effect-call-analysis.ts` runs `recordCollectionMemberEffect` only when the
   * receiver is not the sentinel, and `recordReadonlyViewApplications` is handed a receiver
   * expression. And inside the loop a call is descended only after passing the relation test,
   * which `effect-member-result-relation.ts` answers by calling `memberCallReceiver` and
   * refusing the sentinel, so a call that passes has a receiver by definition.
   *
   * Kept because it states an invariant the two entry points happen to satisfy rather than one
   * this function enforces, and a third caller would not inherit it.
   *
   * An unresolved base is refused rather than skipped, and the two are opposite answers.
   * The sentinel means the descent ran out of receiver before it reached anything ownership
   * can be asked of: `read().slice(0,)` has a call for its receiver whose own callee names
   * no member, so `memberCallReceiver` answers the sentinel and there is no node left to
   * classify. Treating that as "no foreign state found" reads an absent answer as a clean
   * one, which is the guarded failure exactly. `returnsSliceOfOpaqueCallResult` is the
   * program, and it is offered read-only without this. */
  if (base.current === NO_MEMBER_RECEIVER)
    return false;
  /* The endpoint has to still hold what its declaration says, and a declaration cannot
   * answer that for a parameter. `bindingIsReassignable` settles a `let` because the
   * declaration carries the answer; a parameter is declared once and may be pointed
   * elsewhere by any statement. The ownership marker does not prevent it, since
   * `ForeignBorrowed<Value>` intersects an optional property and so assigns to a plain
   * `Value` with no error, which makes `owned = foreign;` legal and leaves the classifier
   * reading a declared type the binding no longer holds. */
  if (bindingAssignedWithin({
    project,
    body,
    node: base.current,
  },))
    return false;
  if (expressionContainsForeignBorrowed({
    project,
    node: base.current,
  },))
    return false;
  return callersAllResolve({
    project,
    body,
  },);
}
