/**
 Structural and reachability claims for one default-library collection call.
 
 @module
 */

import type {
  CallExpression,
  Expression,
  Node,
} from 'typescript/ast';
import type {
  Checker,
  Project,
} from 'typescript/sync';

import {
  COLLECTION_STRUCTURE_MUTATED,
  COLLECTION_STRUCTURE_PRESERVED,
  collectionStructureClaim,
  memberChannelIsVerifiedNarrow,
} from './effect-default-library-readonly-view.ts';
import { rootParameterOrigins, } from './effect-call-resolution.ts';
import { receiverHoldsConstructedContainer, } from './effect-container-literal-holder.ts';
import { receiverElementsArePrimitiveHere, } from './effect-receiver-elements.ts';
import {
  callResultElementReceiver,
  callResultReceiver,
  RESULT_NOT_RECEIVER_STATE,
} from './effect-member-result-relation.ts';
import { resultEscapesCallable, } from './effect-result-escape.ts';
import { returnedResultDischargeable, } from './effect-returned-result-discharge.ts';
import {
  expressionCanCarryMutableState,
  resultExposesMutableState,
} from './effect-primitive-origin.ts';
import {
  addEffectSlots,
  type MutableEffectSummary,
  NO_SLOT_ORIGIN,
  type SlotOrigins,
} from './effect-summary-model.ts';
import { recordReadonlyViewApplications, } from './effect-readonly-view-application.ts';
import { recordResultApplication, } from './effect-result-substitution.ts';

/**
 Nothing about the call was answered, so both sides stay opaque.
 */
export const COLLECTION_CALL_UNDERIVED: unique symbol = Symbol(
  'collection call left both receiver and arguments unproven',
);

/**
 What the call does to its receiver is answered; its arguments are not.
 */
export const COLLECTION_CALL_RECEIVER_DERIVED: unique symbol = Symbol(
  'collection call answered for its receiver only',
);

/**
 The whole call is answered and needs no opaque boundary.
 */
export const COLLECTION_CALL_DERIVED: unique symbol = Symbol(
  'collection call fully answered',
);

/**
 How much of one collection call the derivation could answer.
 */
export type CollectionCallCoverage =
  | typeof COLLECTION_CALL_UNDERIVED
    | typeof COLLECTION_CALL_RECEIVER_DERIVED
    | typeof COLLECTION_CALL_DERIVED;

/**
 Tests whether the receiver claim alone is answerable for this member.
 
 Two conditions, both load-bearing. The member's user-code channel must be
 verified, because a result carrying nothing proves nothing on its own: `join`
 returns a `string` and still calls every element's `toString`, and
 `values.some(foreignPredicate)` returns a `boolean` and still runs the predicate.
 And the result must expose no caller-owned state, because a verified channel
 proves nothing about what comes back: `values.at(0)` reaches no user code and
 hands back the receiver's own element, which nothing then tracks as an alias, so
 `values.at(0).label = 'x'` would go unreported.
 
 @param project - TypeScript project proving default-library ownership.
 
 @param checker - TypeScript checker resolving the instantiated result type.
 
 @param call - Collection call whose result type decides exposure.
 
 @param declaration - Resolved member declaration.
 
 @returns whether receiver opacity is dischargeable for this call.
 
 @example
 ```ts
 receiverClaimAnswerable({ project, checker, call, declaration, });
 ```
 */
function receiverClaimAnswerable({
  project,
  checker,
  call,
  declaration,
  body,
}: {
  readonly project: Project;
  readonly checker: Checker;
  readonly call: CallExpression;
  readonly declaration: Node;
  readonly body?: Node;
},): boolean {
  /**
   How many arguments this call site passes, deciding the argument-conditional channel.
   */
  const passedArgumentCount = call.arguments
    .length;
  if (!memberChannelIsVerifiedNarrow({
    project,
    declaration,
    elementsArePrimitive: receiverElementsArePrimitiveHere({
      checker,
      call,
    },),
    /* Asked of this call rather than of the member, because that is where the answer
     * lives: the same locale member reaches an accessor when handed an options object and
     * reaches nothing when handed nothing. */
    argumentsAreAbsent: passedArgumentCount === 0,
  },))
    return false;
  /**
   Instantiated result type of this call.
   */
  const resultType = checker.getTypeAtLocation(call,);
  if (resultType === undefined)
    return false;
  if (!resultExposesMutableState({
    checker,
    type: resultType,
  },))
    return true;
  /* The result carries state, which used to end the matter. Provenance now tracks that
   * state, so the opacity report is redundant while every use of the result is one the
   * analysis attributes, and it is still required for any use that leaves. Without a
   * body there is nothing to scan, so nothing can be shown non-escaping. */
  if (body === undefined)
    return false;
  /* Asked once, before either relation, because the escape test is relation-agnostic: it
   * follows this call's result to whatever binding holds it and answers about every holder,
   * whichever relation the result satisfies. */
  /* Both gates ask this for the same discharge, so narrowing one alone changes nothing. */
  if (resultEscapesCallable({
    project,
    body,
    call,
    elementStepsAttributed: true,
  },)
    && (!returnedResultDischargeable({
      project,
      call,
      body,
    },)))
    return false;
  /* Either relation licenses the discharge on that same condition. A direct result is the
   * receiver's own value and a container result holds them, and once provenance tracks
   * either, the opacity report is redundant exactly while every use stays inside this
   * callable. The element step is what makes the container half true: a write through
   * `copy[0]`, a destructured element, an iterated one or a spread one is attributed to the
   * receiver's parameter, so this trades a report for an attribution rather than for
   * silence. Landing it while those attributions were empty would produce a false offer,
   * which is what `effect-summaries.unit.test.ts` pins. */
  return (callResultReceiver({
    project,
    checker,
    call,
  },) !== RESULT_NOT_RECEIVER_STATE)
    || (callResultElementReceiver({
      project,
      checker,
      call,
    },) !== RESULT_NOT_RECEIVER_STATE);
}

/**
 Records both claims for one default-library collection call.
 
 The two claims stay independent, because a member can restructure its receiver
 and run user code over it in the same call. `Map.getOrInsertComputed` inserts
 and invokes a caller-supplied factory; `Array.sort(comparator)` reorders and
 invokes the comparator. Each records its mutation and then, separately, has its
 observers analyzed.
 
 Only a fully answered call is discharged. A restructuring member whose
 reachable user code cannot be derived reports its mutation and still falls
 through to the opaque boundary, so a bare `Array.sort()`, which reorders and
 runs the default comparator's string coercion, ends up both mutated and
 opaque rather than silently accepted.
 
 @param project - TypeScript project resolving observer declarations.
 
 @param checker - TypeScript checker resolving receiver and parameter types.
 
 @param bindingOriginBySymbolId - Current callable parameter and alias origins.
 
 @param call - Collection call expression.
 
 @param receiver - Receiver expression whose parameter root is required.
 
 @param declaration - Resolved member declaration.
 
 @param summary - Caller summary receiving facts.
 
 @param analysisRoot - Optional external implementation root.
 
 @returns how much of the call was answered.
 
 @mutates summary - Adds receiver mutation and derived element-flow relations.
 
 @example
 ```ts
 recordCollectionMemberEffect({ project, checker, bindingOriginBySymbolId, containerLiteralHolders, call, receiver, declaration, summary });
 ```
 */
export function recordCollectionMemberEffect({
  project,
  checker,
  bindingOriginBySymbolId,
  containerLiteralHolders,
  call,
  receiver,
  declaration,
  summary,
  analysisRoot,
  body,
}: {
  readonly project: Project;
  readonly checker: Checker;
  readonly bindingOriginBySymbolId: ReadonlyMap<number, SlotOrigins>;
  readonly containerLiteralHolders: ReadonlySet<number>;
  readonly call: CallExpression;
  readonly receiver: Expression;
  readonly declaration: Node;
  readonly summary: MutableEffectSummary;
  readonly analysisRoot?: string;
  readonly body?: Node;
},): CollectionCallCoverage {
  /**
   What this member does to the receiver's own structure.
   */
  const structure = collectionStructureClaim({
    project,
    declaration,
  },);
  if ((structure !== COLLECTION_STRUCTURE_PRESERVED)
    && (structure !== COLLECTION_STRUCTURE_MUTATED))
    return COLLECTION_CALL_UNDERIVED;
  if (structure === COLLECTION_STRUCTURE_MUTATED) {
    /**
     Caller parameters owning receiver, when receiver can carry mutable state.
     */
    const mutatedParameterOrigins = expressionCanCarryMutableState({
      checker,
      node: receiver,
    },)
        /* A container this callable built is not the caller's, however much of the
         * caller's state it holds. `const stack = [root,]; stack.pop();` restructures the
         * fresh array and leaves `root` alone, and charging the receiver's origins there
         * reported a write nothing performs, on the work-stack shape `AGENTS.md` requires
         * over recursion. Reachability runs the other way: the container reaches the
         * parameter, and the parameter does not reach the container.
         *
         * Only this charge consults the record. Origins are untouched, so a write made
         * *through* the container, `stack[0].label = x`, keeps its attribution through the
         * element path, which is what makes suppressing this one safe rather than a hole.
         * Recorded in `doc/planning/prefer-readonly-container-value-provenance.md`. */
      && (!receiverHoldsConstructedContainer({
        project,
        containerLiteralHolders,
        node: receiver,
      },))
      ? rootParameterOrigins({
        project,
        bindingOriginBySymbolId,
        node: receiver,
      },)
      : NO_SLOT_ORIGIN;
    addEffectSlots({
      target: summary.directMutated,
      values: mutatedParameterOrigins,
    },);
    /* A receiver that is itself a call carries whatever that call returns, and no walk
     * here can know what that is: the callee's summary does not exist while its callers
     * are being scanned. `launderMutable(rows,).push(...)` therefore recorded a mutation
     * of nothing and offered `readonly Row[]` for an array it grows, measured in
     * `doc/planning/prefer-readonly-return-substitution.md`. Recording the use defers the
     * origins to `propagateResultApplications`, which has the callee summary. */
    recordResultApplication({
      summary,
      node: receiver,
      kind: 'mutated',
    },);
  }
  // The mutation above is recorded before any discharge below, because a member can
  // be both a verified mutator and narrow: `push` restructures its receiver and
  // reaches nothing but an own-index write. Returning early past the mutation record
  // would trade one silent gap for another.
  if (recordReadonlyViewApplications({
    project,
    checker,
    bindingOriginBySymbolId,
    call,
    receiver,
    summary,
    ...(analysisRoot === undefined) ? {} : { analysisRoot, },
    ...(body === undefined) ? {} : { body, },
  },))
    return COLLECTION_CALL_DERIVED;
  return receiverClaimAnswerable({
    project,
    checker,
    call,
    declaration,
    ...(body === undefined) ? {} : { body, },
  },)
    ? COLLECTION_CALL_RECEIVER_DERIVED
    : COLLECTION_CALL_UNDERIVED;
}
