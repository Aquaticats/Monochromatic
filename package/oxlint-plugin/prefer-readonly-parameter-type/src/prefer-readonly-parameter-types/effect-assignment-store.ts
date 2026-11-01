/**
 Assignments that hand parameter-reachable state to a binding the callable does not own.
 
 Split from `direct-effect-summary.ts` rather than added to its body scan, because the
 question is not the one `inspectDirectWrite` answers. That asks which parameter a write
 lands on, and returns early for an identifier target on purpose: `held = row` changes
 no object the caller can see. This asks the opposite question about the same syntax,
 which is what the assignment hands outward, and an identifier target is exactly where
 that matters.
 
 @module
 */

import {
  type Node,
  SyntaxKind,
} from 'typescript/ast';
import {
  isBinaryExpression,
  isForOfStatement,
  isVariableDeclarationList,
} from 'typescript/ast/is';
import type { Project, } from 'typescript/sync';

import {
  addOpaqueEffect,
  callableDeclaration,
  parameterIndexes,
} from './effect-call-resolution.ts';
import { effectOriginLocation, } from './effect-origin-location.ts';
import {
  calledCallableOrigins,
  transitiveCallableOrigins,
} from './effect-callable-capture-closure.ts';
import { expressionCanCarryMutableState, } from './effect-primitive-origin.ts';
import { targetResultSites, } from './effect-result-binding.ts';
import { possibleValueNodes, } from './effect-possible-values.ts';
import { recordResultRetentionSites, } from './effect-result-substitution.ts';
import { retentionProvenance, } from './effect-retention-provenance.ts';
import {
  type MutableEffectSummary,
  OWNED_CALLABLE_UNAVAILABLE,
  type SlotOrigins,
} from './effect-summary-model.ts';
import { targetIsCallableLocal, } from './effect-value-consumer.ts';

/**
 Assignment operators that store the right operand's own reference.
 
 The logical forms belong here and the arithmetic forms do not. `held ||= config.row`
 stores the row itself whenever the target is absent, so the caller's object outlives
 the call exactly as plain assignment leaves it. `total += config.rows.length` coerces
 its operand to a primitive and retains nothing, so treating the compound operators as
 one class would report a callable that kept only a number.
 */
const RETAINING_ASSIGNMENT_OPERATORS: ReadonlySet<SyntaxKind> = new Set([
  SyntaxKind.EqualsToken,
  SyntaxKind.BarBarEqualsToken,
  SyntaxKind.AmpersandAmpersandEqualsToken,
  SyntaxKind.QuestionQuestionEqualsToken,
],);

/**
 Records opacity for every parameter an escaping assignment can hand outward.
 
 Opacity rather than a dimension of its own. An escaped reference is precisely a value
 this analysis cannot prove stays unwritten, which is what an opaque slot already
 asserts, and it already propagates through owned calls and withholds the offer. A
 separate escape bit would answer the same question until something exists to read it
 more finely.
 
 `parameterIndexes` decides what the assignment hands over, and choosing it over
 `expressionOrigins` is the whole precision of this. The origin resolver descends an
 object literal and answers with the parameter reached through a property read that
 filled it, so `held = { label: config.row.label, }` would attribute a store of a string
 to `config`. `parameterIndexes` gates every leaf on whether that leaf can carry mutable
 state, so the same literal contributes nothing.
 
 @param project - TypeScript project resolving origins and targets.
 
 @param bindingOriginBySymbolId - Local binding origins by symbol identity.
 
 @param resultSitesBySymbolId - Call sites each local binding can hold a result of.
 
 @param summary - Summary receiving opacity.
 
 @param node - Candidate assignment from the body scan.
 
 @param body - Body of callable being summarised.
 
 @mutates summary - Adds an opaque slot and store provenance per escaping origin.
 
 @example
 ```ts
 recordAssignmentStore({ project, bindingOriginBySymbolId, summary, node, body, });
 ```
 */
export function recordAssignmentStore({
  project,
  bindingOriginBySymbolId,
  resultSitesBySymbolId,
  summary,
  node,
  body,
}: {
  readonly project: Project;
  readonly bindingOriginBySymbolId: ReadonlyMap<number, SlotOrigins>;
  readonly resultSitesBySymbolId: ReadonlyMap<number, ReadonlySet<string>>;
  readonly summary: MutableEffectSummary;
  readonly node: Node;
  readonly body: Node;
},): void {
  if (!isBinaryExpression(node,))
    return;
  if (!RETAINING_ASSIGNMENT_OPERATORS.has(node.operatorToken
    .kind,))
    return;
  /* The target policy follows `assignmentStoreEscapes` rather than restating it, so a
   * property, an element and a destructuring pattern all count. Narrowing this to a plain
   * identifier would miss `sink.value = config.row` entirely, and a target the callable
   * does own is exactly what `targetIsCallableLocal` answers about. */
  if (targetIsCallableLocal({
    project,
    target: node.left,
    body,
  },))
    return;
  /**
   Authored target text, naming what the value was handed to.
   */
  const targetText = node.left
    .getText();
  /**
   Where the store sits, so the report can point at it.
   */
  const location = effectOriginLocation({ node, },);
  /**
   Retention provenance naming where the stored value went.
   */
  const provenance = retentionProvenance({
    target: targetText,
    location,
  },);
  /**
   Every expression the stored value can evaluate to, itself included.
   
   Asked once and used twice below, because the two questions this path puts to a stored value
   are questions about the same set: what origins it packages, and whether it is a callable
   handing its captures over. Testing the written syntax answered both only for the inline
   form, and missed the conditional and the container held in a local, both falsified.
   */
  const storedValues = possibleValueNodes({
    project,
    node: node.right,
  },);
  storedValues.forEach(function recordPackagedOrigins(value,): void {
    parameterIndexes({
      project,
      bindingOriginBySymbolId,
      node: value,
    },)
      .forEach(function recordEscapingSlot(affectedSlot,): void {
        addOpaqueEffect({
          summary,
          affectedSlot,
          provenance,
        },);
      },);
  },);

  /* A stored closure hands over everything it captured, and nothing above sees that.
   * `effect-expression-provenance.ts` gives a function expression no provenance successors,
   * so `parameterIndexes` comes back empty for `holder.callback = (): Row => config.row`,
   * and the body is never scanned either: `closure-activity.ts` calls a stored closure
   * inactive, correct for one that never runs and wrong here, because whoever holds it
   * decides whether it runs.
   *
   * Falsified rather than argued. The rule offered `ReadonlyDeep`, the applied annotation
   * type-checked, and a holder invoking the stored closure changed the caller's row.
   * Recorded in `doc/planning/prefer-readonly-return-substitution.md`, section "The
   * escaping closure is a false offer, falsified".
   *
   * `packagedCallableOrigins` answers this because it is the same question the argument
   * path already asks of a method or accessor authored inside a call-argument literal: what
   * can a callable handed to something else reach. Handing it over by storing it differs
   * from handing it over as an argument in who holds it, not in what it captured.
   *
   * It over-approximates in the direction of withholding, naming every binding the body
   * mentions rather than only those a write could travel through, so a closure that merely
   * reads its capture withholds too. Measured, not assumed: `storeReadingClosure` in the
   * structural-store fixture stores `(): number => config.row.label.length` and records
   * `opaque=[0]`. Task #64 holds the finer question. */
  /**
   Whether any possible value of the store was a callable handing its captures over.
   */
  const handedOver: { any: boolean; } = { any: false, };
  storedValues.forEach(function recordCapturedCallable(value,): void {
    /**
     Callable this possible value resolves to, absent when it is not one.
     */
    const callable = callableDeclaration({
      project,
      node: value,
    },);
    if (callable === OWNED_CALLABLE_UNAVAILABLE)
      return;
    handedOver.any = true;
    transitiveCallableOrigins({
      project,
      bindingOriginBySymbolId,
      packaged: callable,
    },)
      .forEach(function recordCapturedSlot(affectedSlot,): void {
        addOpaqueEffect({
          summary,
          affectedSlot,
          provenance,
        },);
      },);
  },);
  /* A call to a locally defined callable has no summary to defer against, since its body is
   * scanned inline, so the retention recorded below substitutes nothing. Falsified: storing what
   * a local function or a method on a local literal hands back left the parameter offered while
   * the caller's row escaped. */
  storedValues.forEach(function recordCalledCallable(value,): void {
    calledCallableOrigins({
      project,
      bindingOriginBySymbolId,
      node: value,
    },)
      .forEach(function recordReachedSlot(affectedSlot,): void {
        addOpaqueEffect({
          summary,
          affectedSlot,
          provenance,
        },);
      },);
  },);
  if (handedOver.any)
    /* Nothing below can add to a callable: a function expression is not a call, so it holds no
     * deferred result for `targetResultSites` to substitute into. */
    return;
  /* `parameterIndexes` stops at a call exactly as the write-side walk did, so
   * `held = firstRow(config,)` recorded nothing while `held = config.row` recorded
   * opacity for the same retention. Falsified rather than inferred: the rule offered
   * `ReadonlyDeep` to both `firstRow` and the storing callable, applying both
   * type-checked, and a later write through the stored value changed the caller's row.
   * Recorded in `doc/planning/prefer-readonly-return-substitution.md`, section "Stage
   * two, a store the analysis could not see".
   *
   * `targetResultSites` rather than the stored expression alone, so a local between the
   * call and the store is followed the same way the write side follows one.
   *
   * Gated on what the stored value can carry, because this path does not go through
   * `parameterIndexes` and so does not inherit its leaf test. Measured without the gate:
   * `heldLabel = firstRow(config,).label` recorded `opaque=[0]` with store provenance for
   * retaining a `string`, which is precisely the claim the leaf test exists to refuse.
   * `storeHeldFresh` stayed empty only because its callee returns nothing, not because
   * anything here recognised a primitive. */
  if (!expressionCanCarryMutableState({
    checker: project.checker,
    node: node.right,
  },))
    return;
  recordResultRetentionSites({
    summary,
    sites: targetResultSites({
      project,
      resultSitesBySymbolId,
      node: node.right,
    },),
    provenance,
  },);
}

/**
 Records opacity for a parameter an iteration statement hands to a binding it does not own.
 
 `for (held of config.rows)` retains a caller-owned row past the call exactly as
 `held = config.rows[0]` does, and no assignment expression appears anywhere in it, so the
 classification that reads assignments cannot see it. Measured before this existed:
 `opaque=[]` and the parameter offered, beside `held = config.rows.at(0,)` recording
 `opaque=[0]` for the same retention.
 
 A declaration initializer is not a store. `for (const row of config.rows)` binds a fresh
 local per iteration, which dies with the iteration, and reporting it would take every
 ordinary read loop with it.
 
 @param project - TypeScript project resolving origins and targets.
 
 @param bindingOriginBySymbolId - Local binding origins by symbol identity.
 
 @param resultSitesBySymbolId - Call sites each local binding can hold a result of.
 
 @param summary - Summary receiving opacity.
 
 @param node - Candidate iteration statement from the body scan.
 
 @param body - Body of callable being summarised.
 
 @mutates summary - Adds an opaque slot and store provenance per retained origin.
 
 @example
 ```ts
 recordIterationStore({ project, bindingOriginBySymbolId, summary, node, body, });
 ```
 */
export function recordIterationStore({
  project,
  bindingOriginBySymbolId,
  resultSitesBySymbolId,
  summary,
  node,
  body,
}: {
  readonly project: Project;
  readonly bindingOriginBySymbolId: ReadonlyMap<number, SlotOrigins>;
  readonly resultSitesBySymbolId: ReadonlyMap<number, ReadonlySet<string>>;
  readonly summary: MutableEffectSummary;
  readonly node: Node;
  readonly body: Node;
},): void {
  if (!isForOfStatement(node,))
    return;
  /**
   What each element is assigned to, either a fresh declaration or an existing binding.
   */
  const { initializer, } = node;
  if (isVariableDeclarationList(initializer,))
    return;
  if (targetIsCallableLocal({
    project,
    target: initializer,
    body,
  },))
    return;
  /* The element decides, not the iterable. `parameterIndexes` gates the leaves of the
   * iterable expression, and an array of strings is itself an object, so asking only that
   * question would report a loop that retains nothing but a `string`. The target's type is
   * the element type here, which is what the binding actually holds after the loop. */
  if (!expressionCanCarryMutableState({
    checker: project.checker,
    node: initializer,
  },))
    return;
  /**
   Authored target text, naming what each element was handed to.
   */
  const targetText = initializer.getText();
  /**
   Where the iteration sits, so the report can point at it.
   */
  const location = effectOriginLocation({ node, },);
  /**
   Retention provenance naming where each element went.
   */
  const provenance = retentionProvenance({
    target: targetText,
    location,
  },);
  parameterIndexes({
    project,
    bindingOriginBySymbolId,
    node: node.expression,
  },)
    .forEach(function recordEscapingSlot(affectedSlot,): void {
      addOpaqueEffect({
        summary,
        affectedSlot,
        provenance,
      },);
    },);
  /* `for (held of rowsOf(config,))` retains through a call the origin walk cannot see,
   * for the same reason the assignment form could not. The iterable is the expression
   * whose result is consumed here, so that is what the retention defers against. */
  recordResultRetentionSites({
    summary,
    sites: targetResultSites({
      project,
      resultSitesBySymbolId,
      node: node.expression,
    },),
    provenance,
  },);
}
