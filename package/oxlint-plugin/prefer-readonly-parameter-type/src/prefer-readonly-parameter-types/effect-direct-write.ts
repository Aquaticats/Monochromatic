/**
 Attribution for a write whose target is rooted at a parameter or one of its aliases.
 
 Split from `direct-effect-summary.ts` for the line budget, and the split falls on a
 seam rather than an arbitrary cut. This answers which parameter a write lands on, while
 `effect-assignment-store.ts` answers where an assigned value went, and the two run over
 the same assignment nodes without sharing anything else.
 
 @module
 */

import type { Node, } from 'typescript/ast';
import { isIdentifier, } from 'typescript/ast/is';
import type { Project, } from 'typescript/sync';

import { expressionOrigins, } from './effect-binding-origins.ts';
import { targetResultSites, } from './effect-result-binding.ts';
import { recordResultApplicationSites, } from './effect-result-substitution.ts';
import {
  addEffectSlots,
  type MutableEffectSummary,
  type SlotOrigins,
} from './effect-summary-model.ts';

/**
 Records direct write rooted at callable parameter or alias.
 
 @param project - TypeScript project resolving root symbol.
 
 @param bindingOriginBySymbolId - Local binding origins by symbol identity.
 
 @param resultSitesBySymbolId - Call sites each local binding can hold a result of.
 
 @param summary - Summary receiving direct mutation.
 
 @param node - Write target expression.
 
 @mutates summary - Adds direct caller-observable write target.
 
 @example
 ```ts
 inspectDirectWrite({ project, bindingOriginBySymbolId, resultSitesBySymbolId, summary, node: assignment.left });
 ```
 */
export function inspectDirectWrite({
  project,
  bindingOriginBySymbolId,
  resultSitesBySymbolId,
  summary,
  node,
}: {
  readonly project: Project;
  readonly bindingOriginBySymbolId: ReadonlyMap<number, SlotOrigins>;
  readonly resultSitesBySymbolId: ReadonlyMap<number, ReadonlySet<string>>;
  readonly summary: MutableEffectSummary;
  readonly node: Node;
},): void {
  if (isIdentifier(node,))
    return;
  addEffectSlots({
    target: summary.directMutated,
    values: expressionOrigins({
      project,
      bindingOriginBySymbolId,
      node,
    },),
  },);
  /* The origin walk above stops dead at a call. `expressionValueOrigins` strips the
   * access layers, finds a `CallExpression` at the root, asks `provenanceSuccessors` what
   * feeds it and is told nothing, so `firstRow(config,).label = 'written'` recorded no
   * write at all.
   *
   * That was a false offer rather than a precision gap, and the falsification is in
   * `doc/planning/prefer-readonly-return-substitution.md`, section "The write path never
   * asks what a call returned". `firstRow` and `writeThroughOwnedCall` were both offered
   * `ReadonlyDeep`, applying both type-checked under TypeScript 7.0.2, and running the
   * pair printed the caller's row with the written label. It compiles because assignability
   * ignores `readonly` property modifiers, so a callee declaring `Row` hands a
   * `ReadonlyDeep<Row>` back as mutable with no diagnostic.
   *
   * A local between the call and the write hides it just as completely, and
   * `targetResultSites` answers both cases with one question: it strips the access layers,
   * names the call when one is underneath, and otherwise follows the identifier through
   * the binding map that records which call filled it. More than one site can come back,
   * because an alias chain can be fed from several places.
   *
   * A callee that allocates needs no exception here. Its returned set is empty, so the
   * deferred use resolves to nothing and the offer stands. */
  recordResultApplicationSites({
    summary,
    sites: targetResultSites({
      project,
      resultSitesBySymbolId,
      node,
    },),
    kind: 'mutated',
  },);
}
