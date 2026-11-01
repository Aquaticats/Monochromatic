/**
 Retention recorded for every argument a call receives.
 
 An argument that is a call result carries caller state the origin walk cannot see, because a callee's
 summary does not exist while its callers are walked. So `sink.push(firstRow(config,),)` handed the
 caller's row to a container and attributed nothing, and `keepRow(firstRow(config,),)` did the same
 through an owned callee. Both falsified.
 
 Recorded as a retention against the inner call site and resolved in the fixed point, which is the same
 deferral the write and store sites use. Retention rather than a mutation claim because handing a value
 to a call is a handoff and not a write, and it withholds silently, which is what a reader can do
 nothing about.
 
 Over-approximating: the receiving call may only read what it was given. The leaf gate takes the common
 half of that away, since an argument that cannot carry mutable state records nothing, and what remains
 withholds rather than offers.
 
 Its own module because two paths need it and one of them used to return before reaching it. A call to
 a callback parameter is classified, answered and returned early, so `rowCallee(passResultRow(cfg.row,),)`
 recorded nothing at all and was indistinguishable from a control handing over a freshly allocated row,
 while the same result handed to an unresolvable member recorded `opaque=[0]`. Falsified with a driver
 whose supplied callee retained the row and wrote through it.
 
 @module
 */

import type { CallExpression, } from 'typescript/ast';
import type {
  Checker,
  Project,
} from 'typescript/sync';

import { effectOriginLocation, } from './effect-origin-location.ts';
import { expressionCanCarryMutableState, } from './effect-primitive-origin.ts';
import { targetResultSites, } from './effect-result-binding.ts';
import { recordResultRetentionSites, } from './effect-result-substitution.ts';
import { handoffProvenance, } from './effect-retention-provenance.ts';
import type { MutableEffectSummary, } from './effect-summary-model.ts';

/**
 Records a retention for every argument of one call that can carry mutable state.
 
 @param project - TypeScript project resolving what each argument holds.
 
 @param checker - Checker deciding whether an argument can carry mutable state.
 
 @param resultSitesBySymbolId - Call sites each local binding can hold a result of.
 
 @param call - Call whose arguments are handed over.
 
 @param summary - Caller summary receiving retentions.
 
 @mutates summary - Adds a deferred retention per argument that can carry state.
 
 @example
 ```ts
 recordArgumentRetentions({ project, checker, resultSitesBySymbolId, call, summary });
 ```
 */
export function recordArgumentRetentions({
  project,
  checker,
  resultSitesBySymbolId,
  call,
  summary,
}: {
  readonly project: Project;
  readonly checker: Checker;
  readonly resultSitesBySymbolId: ReadonlyMap<number, ReadonlySet<string>>;
  readonly call: CallExpression;
  readonly summary: MutableEffectSummary;
},): void {
  call.arguments
    .forEach(function recordArgumentRetention(argument,): void {
      if (!expressionCanCarryMutableState({
        checker,
        node: argument,
      },))
        return;
      recordResultRetentionSites({
        summary,
        sites: targetResultSites({
          project,
          resultSitesBySymbolId,
          node: argument,
        },),
        provenance: handoffProvenance({
          handoff: `a call to ${call.expression
            .getText()}`,
          location: effectOriginLocation({ node: call, },),
        },),
      },);
    },);
}
