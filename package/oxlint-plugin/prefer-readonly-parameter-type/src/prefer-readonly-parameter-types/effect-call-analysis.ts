/**
 Semantic call classification for callable effect summaries.
 
 @module
 */

import type {
  Checker,
  Project,
} from 'typescript/sync';
import type {
  CallExpression,
  Node,
} from 'typescript/ast';
import { isIdentifier, } from 'typescript/ast/is';

import { recordArgumentRetentions, } from './effect-argument-retention.ts';
import { applyExternalEffect, } from './effect-external-application.ts';
import { recordExternalCaptureOpacity, } from './effect-external-capture.ts';
import {
  type ExternalCallableEffectResolver,
  EXTERNAL_CALLABLE_EFFECT_UNAVAILABLE,
} from './external-callable-effect.ts';
import { addOwnedCallEdge, } from './effect-owned-call-edge.ts';
import { isDefaultLibraryArrayBrandDeclaration, } from './effect-default-library-array-brand.ts';
import {
  COLLECTION_CALL_DERIVED,
  COLLECTION_CALL_RECEIVER_DERIVED,
  COLLECTION_CALL_UNDERIVED,
  recordCollectionMemberEffect,
} from './effect-collection-member-effect.ts';
import {
  addEffectSlots,
  isEffectCallableDeclaration,
  type MutableEffectSummary,
  NO_SLOT_ORIGIN,
  OWNED_CALLABLE_UNAVAILABLE,
  type SlotOrigins,
} from './effect-summary-model.ts';
import { isOverridableMethod, } from './effect-overridable-method.ts';
import {
  callableDeclaration,
  parameterIndexes,
  rootParameterOrigins,
} from './effect-call-resolution.ts';
import {
  memberCallReceiver,
  NO_MEMBER_RECEIVER,
} from './effect-member-call-receiver.ts';
import {
  NOT_A_VERIFIED_READER,
  READER_RESULT_FRESH,
  verifiedReaderCall,
} from './effect-default-library-reader-authority.ts';
import { effectCallName, } from './effect-call-name.ts';
import { recordOpaqueBoundary, } from './effect-opaque-boundary.ts';
import { recordUnresolvedCaptureOpacity, } from './effect-unresolved-capture.ts';
import {
  packagedActualCallables,
  possibleValueNodes,
} from './effect-possible-values.ts';
import type { EffectSlot, } from './effect-slot-identity.ts';
import { resultEscapesCallable, } from './effect-result-escape.ts';
import { effectOriginLocation, } from './effect-origin-location.ts';
import { expressionCanCarryMutableState, } from './effect-primitive-origin.ts';
import { targetResultSites, } from './effect-result-binding.ts';
import { recordResultRetentionSites, } from './effect-result-substitution.ts';
import { handoffProvenance, } from './effect-retention-provenance.ts';

/**
 Classifies one call as callback invocation, owned source edge, derived package edge, or opaque boundary.
 
 @param project - TypeScript project resolving symbols.
 
 @param checker - TypeScript checker resolving call receiver.
 
 @param resultSitesBySymbolId - Call sites each local binding can hold a result of.
 
 @param containerLiteralHolders - Bindings holding a container this callable built.
 
 @param bindingOriginBySymbolId - Current callable parameter and alias origins.
 
 @param call - Call expression to classify.
 
 @param summary - Current callable summary receiving facts.
 
 @param foreignInbound - Whether call belongs directly to summary callable.
 
 @param analysisRoot - Optional external implementation root accepted for owned call edges.
 
 @param externalEffectResolver - Demand-driven package implementation analyzer.
 
 @mutates summary - Adds call, mutation, callback, or opaque effect facts.
 
 @example
 ```ts
 inspectEffectCall({ project, checker, bindingOriginBySymbolId, call, summary });
 ```
 */
export function inspectEffectCall({
  project,
  checker,
  bindingOriginBySymbolId,
  resultSitesBySymbolId,
  containerLiteralHolders,
  call,
  summary,
  foreignInbound,
  analysisRoot,
  externalEffectResolver,
  body,
}: {
  readonly project: Project;
  readonly checker: Checker;
  readonly bindingOriginBySymbolId: ReadonlyMap<number, SlotOrigins>;
  readonly resultSitesBySymbolId: ReadonlyMap<number, ReadonlySet<string>>;
  readonly containerLiteralHolders: ReadonlySet<number>;
  readonly call: CallExpression;
  readonly summary: MutableEffectSummary;
  readonly foreignInbound: boolean;
  readonly analysisRoot?: string;
  readonly externalEffectResolver: ExternalCallableEffectResolver;
  readonly body?: Node;
},): void {
  /**
   Caller parameter roots corresponding to call arguments.
   */
  const allArgumentIndexes = call.arguments
    .map(function argumentIndex(argument,): readonly EffectSlot[] {
      return parameterIndexes({
        project,
        bindingOriginBySymbolId,
        node: argument,
      },);
    },);
  /**
   Parameters the direct callee identifier can hold, when it is a callback.
   */
  const callbackParameterOrigins = isIdentifier(call.expression,)
    ? rootParameterOrigins({
      project,
      bindingOriginBySymbolId,
      node: call.expression,
    },)
    : NO_SLOT_ORIGIN;
  if (callbackParameterOrigins.size > 0) {
    addEffectSlots({
      target: summary.directInvoked,
      values: callbackParameterOrigins,
    },);
    call.arguments
      .forEach(function callbackArgument(
        argument,
        callbackArgumentPosition,
      ): void {
        /**
         Source parameter passed to callback argument, when direct.
         */
        const sourceSlots = parameterIndexes({
          project,
          bindingOriginBySymbolId,
          node: argument,
        },);
        sourceSlots.forEach(function callbackSource(
          sourceSlot,
        ): void {
          /* One relation per callback origin. A reassigned callback local may hold
           * either parameter, and the argument reaches whichever one runs, so
           * recording a single origin would under-report the other. */
          callbackParameterOrigins.forEach(function relateOrigin(
            callbackSlot,
          ): void {
            summary.relations
              .push({
                callbackSlot,
                callbackArgumentPosition,
                sourceSlot,
              },);
          },);
        },);
      },);
    /* And the case the relation has nobody to defer to. A relation says the caller supplied this
     * callback and the caller knows what it does, which is why deferring is right and why task #75
     * closed the question. A parameter default is supplied by the callee, so deferring loses
     * whatever the default does. Measured: `directWriter(directTarget.row,)` where `directWriter`
     * defaults to a closure writing through its own parameter recorded `mutated=[1]` alone and left
     * `directTarget` offered, and applying the offer type-checks because a `ReadonlyDeep<Row>` is
     * accepted where `Row` is expected. Falsified with a driver that omits the argument.
     *
     * An edge is built in addition to the relation rather than instead of it, because a caller that
     * does supply a callback still needs the relation, and because the default's effects can only
     * add to what this call already claims. Claiming the default's write when the caller supplied
     * something else withholds an offer that might have stood, which is the safe direction. */
    /* Asked through the shared resolver rather than by filtering the value walk here. The walk hands
     * back the identifier a default names, and an identifier is not a callable declaration, so a
     * default naming an ordinary function built no edge at all while one written inline built one.
     * Measured: storing what a block-bodied named default handed back left the configuration offered,
     * where the same callee reached directly or through a local alias charged it. */
    packagedActualCallables({
      project,
      actual: call.expression,
    },)
      .forEach(function edgeToDefault(value,): void {
        if (!isEffectCallableDeclaration(value,))
          return;
        addOwnedCallEdge({
          project,
          call,
          callee: value,
          allArgumentIndexes,
          summary,
          foreignInbound,
          ...(analysisRoot === undefined) ? {} : { analysisRoot, },
        },);
      },);
    /* And the captures the relation above cannot carry. A relation names which caller-owned value
     * reached which callback argument position, and the caller can reconstruct that because the
     * caller chose the value. A closure written here is not the caller's choice, and what it captures
     * is visible only inside this callable, so `parameterIndexes` answered empty and the relation held
     * nothing at all.
     *
     * Measured: `invoke((): Row => handedThrough.row,)` recorded no opacity and was indistinguishable
     * from a control handing over a closure that allocates, while the same closure handed to
     * `registry.keep` recorded `opaque=[0]`. Two paths, one relation, disagreeing. Falsified with a
     * driver whose supplied callee kept the producer, invoked it, and wrote through the row.
     *
     * So the deferral #75 settled is incomplete rather than wrong, and this restores the missing half
     * without touching the half that works. The capture gate alone rather than the whole boundary,
     * because the boundary would also mark ordinary direct arguments opaque and that is exactly what
     * the relation exists to defer. */
    recordUnresolvedCaptureOpacity({
      project,
      bindingOriginBySymbolId,
      summary,
      actuals: call.arguments,
      provenance: `${effectCallName(call.expression,)} [${effectOriginLocation({
        node: call,
      },)}]`,
    },);
    /* And the retention every argument carries, which this branch used to return before recording. A
     * relation cannot see through an inner call result, because a callee's summary does not exist while
     * its callers are walked, so `rowCallee(passResultRow(cfg.row,),)` recorded nothing at all and was
     * indistinguishable from a control handing over a freshly allocated row, while the same result
     * handed to an unresolvable member recorded `opaque=[0]`. Falsified with a driver whose supplied
     * callee retained the row and wrote through it.
     *
     * Third instance of one pattern, after the capture gate above: a branch answers its own question
     * and returns before something every call needs. */
    recordArgumentRetentions({
      project,
      checker,
      resultSitesBySymbolId,
      call,
      summary,
    },);
    return;
  }

  /**
   Selected call signature for overload-aware declaration resolution.
   */
  const resolvedSignature = checker.getResolvedSignature(call,);
  /**
   Function-like declaration selected by resolved signature.
   */
  const resolvedDeclaration = resolvedSignature
    ?.declaration
    ?.resolve(project,);
  if ((resolvedDeclaration !== undefined)
    && isDefaultLibraryArrayBrandDeclaration({
      project,
      declaration: resolvedDeclaration,
    },))
    return;
  /* A default-library reader takes its caller-owned value as an argument rather than as a
   * receiver, so the collection path below never reaches it and every value handed to one
   * took an opaque boundary. A reader neither writes nor is handed a callback, so the only
   * question left is what its result shares with what it read. */
  if (resolvedDeclaration !== undefined) {
    /**
     Verified reader and the value it reads, when this call is one.
     */
    const reader = verifiedReaderCall({
      project,
      checker,
      call,
      declaration: resolvedDeclaration,
    },);
    if (reader !== NOT_A_VERIFIED_READER) {
      if (reader.resultRelation === READER_RESULT_FRESH)
        /* Nothing of the operand comes back, so no later use can reach it. */
        return;
      if ((body !== undefined)
        && (!resultEscapesCallable({
          project,
          body,
          call,
          /* Element steps are not attributed on this path: nothing here walks the
           * elements of a call result reaching an argument. */
          elementStepsAttributed: false,
        },)))
        /* The result carries the operand's values, and every use of it is one this
         * analysis attributes, so tracking replaces the boundary exactly as it does for a
         * verified collection member. A use that leaves keeps the boundary. */
        return;
    }
  }
  /**
   Expression the call was made on, however the member was named.
   */
  const collectionReceiver = memberCallReceiver({ call, },);
  /**
   How much of a default-library collection call the derivation answered.
   */
  const collectionCoverage = (resolvedDeclaration !== undefined)
    && (collectionReceiver !== NO_MEMBER_RECEIVER)
    ? recordCollectionMemberEffect({
      project,
      checker,
      bindingOriginBySymbolId,
      containerLiteralHolders,
      call,
      receiver: collectionReceiver,
      declaration: resolvedDeclaration,
      summary,
      ...(analysisRoot === undefined) ? {} : { analysisRoot, },
      ...(body === undefined) ? {} : { body, },
    },)
    : COLLECTION_CALL_UNDERIVED;
  if (collectionCoverage === COLLECTION_CALL_DERIVED)
    return;
  /* A call resolves against the receiver's declared type, so an instance method resolves to
   * the declaration that type names. The value at runtime may be a subclass whose override runs
   * instead, and an override is free to write what the base only reads.
   * `callOverridableMethod` in the slot-narrowing fixture measures exactly that: it reported no
   * effect at all while `WritingDerived.inspect` assigns the row its base reads, which offers a
   * row a subclass mutates.
   *
   * The subclass need not exist in this project either, so enumerating overrides cannot settle
   * it. Treating the call as unresolved is what the rule already does everywhere it cannot see
   * the body that runs. */
  /**
   Whether the resolved declaration is an instance method a subclass may override.
   */
  const overridable = (resolvedDeclaration !== undefined)
    && isOverridableMethod({ declaration: resolvedDeclaration, },);
  /**
   Whether the signature resolved to a callable this analysis can inspect.
   */
  const resolvedIsOwnedCallable = (resolvedDeclaration !== undefined)
    && isEffectCallableDeclaration(resolvedDeclaration,);
  /**
   Owned callee declaration selected by signature, when one body certainly runs.
   */
  const signatureCallee = (resolvedIsOwnedCallable && (!overridable))
    ? callableDeclaration({
      project,
      node: resolvedDeclaration,
      ...(analysisRoot === undefined) ? {} : { analysisRoot, },
    },)
    : OWNED_CALLABLE_UNAVAILABLE;
  /**
   Owned callee selected by signature or expression symbol fallback.
   */
  const callee = ((signatureCallee === OWNED_CALLABLE_UNAVAILABLE) && (!overridable))
    ? callableDeclaration({
      project,
      node: call.expression,
      ...(analysisRoot === undefined) ? {} : { analysisRoot, },
    },)
    : signatureCallee;
  recordArgumentRetentions({
    project,
    checker,
    resultSitesBySymbolId,
    call,
    summary,
  },);
  if ((callee === OWNED_CALLABLE_UNAVAILABLE)
    && (resolvedDeclaration !== undefined)) {
    /**
     Demand-driven effect inferred from exact shipped package implementation.
     */
    const externalEffect = externalEffectResolver({
      consumerProject: project,
      call,
      declaration: resolvedDeclaration,
    },);
    if (externalEffect !== EXTERNAL_CALLABLE_EFFECT_UNAVAILABLE) {
      /* Before the application, and the early return below is why it has to be here rather than left to
       * `recordOpaqueBoundary`. Captures are kept beside ordinary origins, so the origin mapping
       * `applyExternalEffect` performs cannot see one, and returning on a proven external effect skipped
       * the only channel that could. Charged per formal, so a formal the implementation provably ignores
       * still keeps its offer. */
      recordExternalCaptureOpacity({
        project,
        bindingOriginBySymbolId,
        externalEffect,
        declaration: resolvedDeclaration,
        call,
        summary,
      },);
      applyExternalEffect({
        externalEffect,
        declaration: resolvedDeclaration,
        call,
        allArgumentIndexes,
        summary,
      },);
      return;
    }
  }
  if (callee !== OWNED_CALLABLE_UNAVAILABLE) {
    addOwnedCallEdge({
      project,
      call,
      callee,
      allArgumentIndexes,
      summary,
      foreignInbound,
      ...(analysisRoot === undefined) ? {} : { analysisRoot, },
    },);
    return;
  }

  recordOpaqueBoundary({
    project,
    bindingOriginBySymbolId,
    call,
    allArgumentIndexes,
    summary,
    receiverDerived: collectionCoverage === COLLECTION_CALL_RECEIVER_DERIVED,
  },);
}
