/**
 Recording opacity for a value whose iterator the rule cannot vouch for.
 
 @module
 */

import type { Node, } from 'typescript/ast';
import {
  isForOfStatement,
  isSpreadElement,
} from 'typescript/ast/is';
import type { Project, } from 'typescript/sync';

import { recordIterationStore, } from './effect-assignment-store.ts';
import { addOpaqueEffect, } from './effect-call-resolution.ts';
import { expressionOrigins, } from './effect-binding-origins.ts';
import { expressionValueOrigins, } from './effect-expression-provenance.ts';
import { iterationOpensUserCode, } from './effect-iteration-channel.ts';
import type { EffectSlot, } from './effect-slot-identity.ts';
import {
  addEffectSlots,
  type MutableEffectSummary,
  type SlotOrigins,
} from './effect-summary-model.ts';

/**
 Characters of a drained expression kept in a provenance string.
 */
const DRAIN_PROVENANCE_LIMIT = 60;

/**
 Charges every parameter a drained value can hold, when draining may run user code.
 
 Both spellings that drain reach this: a `for...of` expression and a spread element. They
 arrive through different branches of the walk and ask one question, so the question lives
 here rather than twice there, which is also what keeps `direct-effect-summary.ts` inside
 its line budget.
 
 Opacity rather than mutation, because what the iterator does is exactly what cannot be seen
 from the loop. A member that only reads is charged too, which costs precision and keeps the
 claim sound: this says the channel is open, not that it was used.
 
 @param project - TypeScript project resolving types and declarations.
 
 @param bindingOriginBySymbolId - Current callable parameter and alias origins.
 
 @param summary - Caller summary receiving the charge.
 
 @param node - Walked node, charged when it is one of the draining spellings.
 
 @mutates summary - Adds opacity for every origin the drained value can hold.
 
 @example
 ```ts
 chargeDrainedIterator({ project, bindingOriginBySymbolId, summary, node, spelling: 'iterated' });
 ```
 */
export function chargeDrainedIterator({
  project,
  bindingOriginBySymbolId,
  summary,
  node,
}: {
  readonly project: Project;
  readonly bindingOriginBySymbolId: ReadonlyMap<number, SlotOrigins>;
  readonly summary: MutableEffectSummary;
  readonly node: Node;
},): void {
  if (isSpreadElement(node,)) {
    chargeDrained({
      project,
      bindingOriginBySymbolId,
      summary,
      drained: node.expression,
      spelling: 'spread',
    },);
    return;
  }
  if (isForOfStatement(node,))
    chargeDrained({
      project,
      bindingOriginBySymbolId,
      summary,
      drained: node.expression,
      spelling: 'iterated',
    },);
}

/**
 Charges one drained expression's origins.
 
 @param project - TypeScript project resolving types and declarations.
 
 @param bindingOriginBySymbolId - Current callable parameter and alias origins.
 
 @param summary - Caller summary receiving the charge.
 
 @param drained - Expression whose iterator runs.
 
 @param spelling - Word naming how it is drained.
 
 @mutates summary - Adds opacity for every origin the drained value can hold.
 
 @example
 ```ts
 chargeDrained({ project, bindingOriginBySymbolId, summary, drained, spelling: 'spread' });
 ```
 */
function chargeDrained({
  project,
  bindingOriginBySymbolId,
  summary,
  drained,
  spelling,
}: {
  readonly project: Project;
  readonly bindingOriginBySymbolId: ReadonlyMap<number, SlotOrigins>;
  readonly summary: MutableEffectSummary;
  readonly drained: Node;
  readonly spelling: string;
},): void {
  if (!iterationOpensUserCode({
    project,
    node: drained,
  },))
    return;
  expressionValueOrigins({
    project,
    bindingOriginBySymbolId,
    node: drained,
  },)
    .forEach(function chargeOrigin(origin: EffectSlot,): void {
      /**
       Drained expression's text, trimmed to keep a diagnostic readable.
       */
      const named = drained.getText()
        .slice(
        0,
        DRAIN_PROVENANCE_LIMIT,
        );
      addOpaqueEffect({
        summary,
        affectedSlot: origin,
        provenance: `${spelling} ${named}`,
      },);
    },);
}

/**
 Records what one iteration statement retains and, for the awaiting form, what it mutates.
 
 Moved here from `direct-effect-summary.ts` so that file stays inside its line budget while
 the drain charge lives beside it. The two belong together anyway: both answer what an
 iteration does to the value it drains, one about the target and one about the iterator.
 
 @param project - TypeScript project resolving types and declarations.
 
 @param bindingOriginBySymbolId - Current callable parameter and alias origins.
 
 @param resultSitesBySymbolId - Call-result sites reachable per binding.
 
 @param summary - Caller summary receiving retention and mutation.
 
 @param node - Iteration statement being summarized.
 
 @param body - Callable body owning this statement.
 
 @mutates summary - Records the iteration target and any awaited drain.
 
 @example
 ```ts
 recordIterationEffects({ project, bindingOriginBySymbolId, resultSitesBySymbolId, summary, node, body });
 ```
 */
export function recordIterationEffects({
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
  /* Charged here rather than at the generic walk site, because the iteration branch there
   * returns before reaching it, so a `for...of` never arrived. Measured: the pinned case read
   * `[]` until this moved. */
  chargeDrainedIterator({
    project,
    bindingOriginBySymbolId,
    summary,
    node,
  },);
  /* Asked of every iteration statement, including the awaiting form: what the target retains
   * does not depend on how the iterator was drained. */
  recordIterationStore({
    project,
    bindingOriginBySymbolId,
    resultSitesBySymbolId,
    summary,
    node,
    body,
  },);
  if (!isForOfStatement(node,))
    return;
  if (node.awaitModifier === undefined)
    return;
  addEffectSlots({
    target: summary.directMutated,
    values: expressionOrigins({
      project,
      bindingOriginBySymbolId,
      node: node.expression,
    },),
  },);
}
