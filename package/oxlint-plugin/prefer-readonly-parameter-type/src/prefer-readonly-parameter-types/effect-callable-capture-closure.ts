/**
 Caller origins a callable can reach, including through calls to callables beside it.
 
 `packagedCallableOrigins` is lexical: it names every binding the packaged body mentions and
 follows no call out of it. That answers the question for a closure that reads what it
 captured directly, and misses one that reaches caller state only by calling a sibling:
 
 ```ts
 const read = (): Row => config.row;
 holder.produce = (): Row => read();
 ```
 
 The stored arrow names only `read`, and a local bound to a function expression carries no
 parameter origin, so the lexical scan came back empty and the parameter was offered.
 Falsified: the annotation applied, type-checked clean beside a control whose direct write was
 rejected, and the driver changed the caller's row through the stored closure.
 
 Same cause as the over-reporting in the other direction, pointing the other way. A lexical
 scanner is answering a call-graph question, so it is wrong twice: it names bindings a
 read-only body merely mentions, and it misses captures that leave through a call.
 
 The bound is the source file, and it is a cost bound rather than a soundness one. An earlier
 version of this paragraph claimed the reverse; a mutant that deleted the bound survived the
 whole suite, which is what showed the claim was backwards.
 
 Why it cannot change an answer: `packagedCallableOrigins` resolves each named binding to a
 symbol and looks that symbol up in the origin map of the callable being summarised. A
 cross-file callee's body names its own symbols, which are absent from that map, so following
 it contributes nothing. It also loses nothing, because a callable able to capture those
 bindings must be written inside the callable that owns them and is therefore in the same
 file.
 
 So the bound exists to stop the walk crossing the workspace for answers it cannot change.
 No assertion can defend it, and none pretends to: the cost it avoids is unmeasured, and a
 mutation check over summaries is the wrong instrument for a claim about work rather than
 about results. *
 ## The activation premise, which decides what this walk does not have to answer for
 
 A callable handed as an argument is activated, and an activated closure's body is scanned inline as
 part of the enclosing callable, so every channel the enclosing callable has applies inside that
 closure too. Stated in full in `effect-unresolved-capture.ts` under what decides whether to withhold.
 
 It is not visible from this file and it has caused the same wrong conclusion three times: a reading
 of one module predicts a hole another module has already closed. Writes a reached callable performs
 are already charged for that reason, so this walk answers only for what invoking one hands back.
 
 @module
 */

import type {
  CallExpression,
  Node,
} from 'typescript/ast';
import {
  isCallExpression,
  isObjectLiteralExpression,
  isPropertyAccessExpression,
} from 'typescript/ast/is';
import type { Project, } from 'typescript/sync';

import { accessedCallables, } from './effect-accessor-reach.ts';
import { callableDeclaration, } from './effect-call-resolution.ts';
import { packagedCallableOrigins, } from './effect-packaged-callable-origins.ts';
import { assignedValuesInScope, } from './effect-assigned-values.ts';
import { packagedActualCallables, } from './effect-possible-values.ts';
import { reachableValueSources, } from './effect-result-reach.ts';
import type { EffectSlot, } from './effect-slot-identity.ts';
import {
  collectAstNodes,
  OWNED_CALLABLE_UNAVAILABLE,
  type SlotOrigins,
} from './effect-summary-model.ts';

/**
 Origins carried by nothing, shared so a call to no known callable allocates none.
 */
const NO_REACHED_ORIGINS: ReadonlySet<EffectSlot> = new Set<EffectSlot>();

/**
 Builds a stable identity for any node, callable declaration or not.
 
 `callableKey` is the usual identity here and it accepts only a callable declaration, while a
 packaged callable can also arrive as an accessor, a method or a class expression. The source
 span answers for all of them and is the same idiom `activateEscapedCallables` uses for its
 own cycle detection.
 
 @param node - Node being identified.
 
 @returns identity stable across one analysis.
 
 @example
 ```ts
 nodeKey({ node });
 ```
 */
function nodeKey({ node, }: { readonly node: Node; },): string {
  return `${node.getSourceFile()
    .fileName}:${String(node.pos,)}:${String(node.end,)}`;
}

/**
 Collects caller origins a callable reaches directly and through same-file callees.
 
 @param project - TypeScript project resolving call targets and binding symbols.
 
 @param bindingOriginBySymbolId - Origins of the callable being summarised.
 
 @param packaged - Callable whose reach is being computed.
 
 @returns parameter slots reachable through that callable and everything it calls nearby.
 
 @example
 ```ts
 transitiveCallableOrigins({ project, bindingOriginBySymbolId, packaged });
 ```
 */
export function transitiveCallableOrigins({
  project,
  bindingOriginBySymbolId,
  packaged,
}: {
  readonly project: Project;
  readonly bindingOriginBySymbolId: ReadonlyMap<number, SlotOrigins>;
  readonly packaged: Node;
},): ReadonlySet<EffectSlot> {
  /**
   File every followed callee must share, bounding work rather than deciding an answer.
   
   A callable able to capture these bindings is written inside the one that owns them and so
   shares its file, and a cross-file callee names symbols this origin map does not hold. The
   bound therefore skips only work, which is why deleting it changes no assertion.
   */
  const { fileName, } = packaged.getSourceFile();
  /**
   Origins reachable through the callable and everything it calls nearby.
   */
  const origins = new Set<EffectSlot>();
  /**
   Callables already folded in, keyed so a recursive or mutually recursive pair terminates.
   */
  const visited = new Set<string>([nodeKey({ node: packaged, },),],);
  /**
   Callables still to fold in.
   */
  const pending: Node[] = [packaged,];
  while (pending.length > 0) {
    /**
     Next callable whose reach is being folded in.
     */
    const current = pending.pop();
    if (current === undefined)
      continue;
    packagedCallableOrigins({
      project,
      bindingOriginBySymbolId,
      packaged: current,
    },)
      .forEach(function collectOrigin(origin,): void {
        origins.add(origin,);
      },);
    [
      ...calledCallables({
        project,
        within: current,
        fileName,
      },),
      /* And the callables a property read reaches, which the call walk cannot see because a read is
       * not a call. A getter over caller state is the sharp case: reading it runs a body the walk
       * never entered. */
      ...accessedCallables({
        project,
        within: current,
        fileName,
      },),
    ]
      .forEach(function enqueueCallee(callee,): void {
        /**
         Stable key for the callee, so each is folded in once.
         */
        const key = nodeKey({ node: callee, },);
        if (visited.has(key,))
          return;
        visited.add(key,);
        pending.push(callee,);
      },);
  }
  return origins;
}

/**
 Resolves every call inside one callable to a same-file callable declaration.
 
 The callee expression is resolved rather than the call, because what is wanted is what the value
 before the parentheses can be. That is what follows `read()` to the arrow `read` was bound to.
 
 Asked through the shared resolver, which is the same relation the argument and edge paths ask. The
 narrow resolver alone answers for one declaration and nothing for a conditional, so a handed closure
 invoking `(pick ? revealRow : freshRow)()` reached nothing and offered the configuration `revealRow`
 reads, while the same closure invoking one named callee charged it. Measured before and after.
 
 @param project - TypeScript project resolving call targets.
 
 @param within - Callable whose body is scanned for calls.
 
 @param fileName - File a callee must share to be followed.
 
 @returns callable declarations this body calls in the same file.
 
 @example
 ```ts
 calledCallables({ project, within, fileName });
 ```
 */
function calledCallables({
  project,
  within,
  fileName,
}: {
  readonly project: Project;
  readonly within: Node;
  readonly fileName: string;
},): readonly Node[] {
  return collectAstNodes(within,)
    .filter(function isCall(node,): node is CallExpression {
      return isCallExpression(node,);
    },)
    .flatMap(function resolveCallee(call,): readonly Node[] {
      /* A callee binding filled by assignment names a callable too, and only its initializer was followed.
       * Asked here rather than in the value walk, which feeds owned call edges and broke a completeness
       * invariant when widened, and rather than in the completion gate, where it was a no-op because that
       * gate answers what a completion carries and not which origins are reachable. */
      return [
        ...packagedActualCallables({
          project,
          actual: call.expression,
        },),
        ...assignedValuesInScope({
          project,
          node: call.expression,
        },)
          .flatMap(function assignedCallables(assigned,): readonly Node[] {
            return packagedActualCallables({
              project,
              actual: assigned,
            },);
          },),
      ]
        .filter(function sameFileCallee(callee,): boolean {
          /**
           File the resolved callee is written in.
           */
          const { fileName: calleeFileName, } = callee.getSourceFile();
          return calleeFileName === fileName;
        },);
    },);
}

/**
 Collects caller origins the result of calling a locally defined callable can carry.
 
 The deferred result relation had nothing to substitute against a locally defined callee's call
 site. Measured before this existed:
 
 ```ts
 function read(): Row {
   return config.row;
 }
 bag.row = read();
 ```
 
 recorded `opaque=[]` and was offered, while the same store through a top-level callee recorded
 retention correctly. Falsified, driver printing the caller's row changed. The inline scan even
 put the nested return into the enclosing callable's returned set, so a callable returning `void`
 claimed a returned origin.
 
 Answered with what the callable can reach rather than with what it returns. Over-approximating,
 since the callable may return something freshly allocated, and it is the direction that
 withholds.
 
 An earlier version of this paragraph said a callable written inside the one being summarised has no
 summary of its own, and that a precise answer would need nested callables to carry one. Both halves
 are false and the claim was load-bearing enough that two separate reviews reasoned from it, one of
 them concluding a defect had a cause it did not have.
 
 A nested callable does have a summary, and consumers do read it. Measured twice. A write through a
 defaulted closure's own formal reaches the caller, `invokeWritingDefault` recording
 `mutated=[0,1]`, which can only travel through that callable's summary. And once #98 gave a concise
 body its returned fact, a store of what a nested default handed back began charging through
 `propagateResultApplications`, which reads `summaries.get(edge.calleeKey)` and skips when it finds
 nothing.
 
 What was true is narrower: the old result path did not consult that summary. This walk stays as the
 over-approximating answer for what a callee reaches, which is a different question from what it
 hands back.
 
 @param project - TypeScript project resolving the callee.
 
 @param bindingOriginBySymbolId - Origins of the callable being summarised.
 
 @param node - Expression that may be a call to a locally defined callable.
 
 @returns origins the result can carry, empty when the callee is not one.
 
 @example
 ```ts
 calledCallableOrigins({ project, bindingOriginBySymbolId, node });
 ```
 */
export function calledCallableOrigins({
  project,
  bindingOriginBySymbolId,
  node,
}: {
  readonly project: Project;
  readonly bindingOriginBySymbolId: ReadonlyMap<number, SlotOrigins>;
  readonly node: Node;
},): ReadonlySet<EffectSlot> {
  if (!isCallExpression(node,))
    return NO_REACHED_ORIGINS;
  /**
   Callable the callee expression resolves to, absent when nothing owned answers.
   */
  const callee = callableDeclaration({
    project,
    node: node.expression,
  },);
  /**
   Origins reachable through the callee and through whatever it was called on.
   */
  const reached = new Set<EffectSlot>();
  if (callee !== OWNED_CALLABLE_UNAVAILABLE)
    transitiveCallableOrigins({
      project,
      bindingOriginBySymbolId,
      packaged: callee,
    },)
      .forEach(function collectCalleeOrigin(origin,): void {
        reached.add(origin,);
      },);
  /* And every other callable the callee expression can hold, because the resolver stops at a
   * parameter and a parameter can carry a default. Measured: a store of what invoking a defaulted
   * producer handed back recorded nothing against the configuration the default reads, while the
   * same store through a named callee recorded it, so the offer stood on the resolver's silence
   * rather than on anything about the store. Falsified.
   *
   * Asked alongside the resolver rather than instead of it, on the same grounds the capture channel
   * asks it: an origin the result can carry only ever withholds. */
  packagedActualCallables({
    project,
    actual: node.expression,
  },)
    .forEach(function collectCandidate(candidate,): void {
      transitiveCallableOrigins({
        project,
        bindingOriginBySymbolId,
        packaged: candidate,
      },)
        .forEach(function collectCandidateOrigin(origin,): void {
          reached.add(origin,);
        },);
    },);
  if (!isPropertyAccessExpression(node.expression,))
    return reached;
  /* The receiver is asked as well as the callee, never instead of it, and unioning rather than
   * choosing is the whole fix. A method reading `this.row` names no binding at all, because `this`
   * is a keyword, so scanning the method body answers empty while the state it reaches sits in the
   * literal the method was written in. Resolving the callee succeeds for such a method, so an
   * early return on that success scanned exactly the body that cannot see the capture.
   *
   * Measured, which is how the three passing shapes hid this one: a method naming the parameter
   * directly, an arrow property naming it, and a plain property read all answered correctly, and
   * only the `this` form did not. Falsified.
   *
   * Every binding the receiver's literal mentions is collected, without asking which property the
   * call selected, exactly as the aggregate descent elsewhere declines to track keys. */
  reachableValueSources({
    project,
    node: node.expression
      .expression,
  },)
    .forEach(function collectSource(source,): void {
      if (!isObjectLiteralExpression(source,))
        return;
      transitiveCallableOrigins({
        project,
        bindingOriginBySymbolId,
        packaged: source,
      },)
        .forEach(function collectOrigin(origin,): void {
          reached.add(origin,);
        },);
    },);
  return reached;
}
