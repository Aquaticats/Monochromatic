/**
 Captures charged per external formal, which the external branch reached before nothing.
 
 `applyExternalEffect` maps external formals onto caller origins taken from argument positions, and a
 closure argument has no origin of its own: captures are kept beside ordinary origins deliberately, so
 a channel reading only origins cannot see one. The external branch then returns before
 `recordOpaqueBoundary`, which is the only place captures were charged, so a capture handed to a
 package this analysis **did** inspect was charged by nothing at all.
 
 Falsified against a dependency written for the purpose, in
 `external-capture-channel.unit.test.ts`. The rule offered
 `Parameter "config" can be deeply readonly: property row is writable.` for a callable handing
 `(): Row => config.row` to a package export that pushes it into a module-level array, while a sibling
 callable in the same file reported that export by name and version, so the external path had provably
 run.
 
 ## Why per formal rather than per call
 
 Charging every argument's captures whenever an external effect resolved would be sound and would
 discard the precision the external branch exists for. An external summary is a proof about the shipped
 implementation, so a formal it reports no fact about is a formal that implementation demonstrably does
 not invoke, keep or write through, and a closure in that position exposes nothing.
 
 This is the design `exposedCaptureOrigins` was split out for, stated in its own documentation before
 any of it could be measured.
 
 ## Which facts expose a capture, and which do not
 
 Invocation and opacity do, and they are separate facts rather than one: an implementation that calls a
 handed callable records that formal as invoked, while one that keeps it records opacity, since a store
 into a module binding is what opacity records. Each has its own case in the test for that reason.
 
 A proven mutation does not. Writing through a formal reaches what that formal already refers to rather
 than anything invoking a callable in that position would hand back, and such a formal is charged
 directly by `applyExternalEffect` regardless. Excluding it keeps this channel about exposure.
 
 ## Where it stays conservative
 
 A formal this mapping cannot place charges every argument's captures, exactly as `applyExternalEffect`
 charges every argument's origins for one. The analysed implementation need not be the declaration the
 consumer resolved, so a summary can name a formal this declaration does not declare, and guessing
 which argument that was would be the one direction that loses a fact rather than an offer.
 
 @module
 */

import type {
  CallExpression,
  Node,
} from 'typescript/ast';
import type { Project, } from 'typescript/sync';

import { addOpaqueEffect, } from './effect-call-resolution.ts';
import { formalActualPositions, } from './effect-formal-actual-mapping.ts';
import {
  isEffectCallableDeclaration,
  type MutableEffectSummary,
  type SlotOrigins,
} from './effect-summary-model.ts';
import { exposedCaptureOrigins, } from './effect-unresolved-capture.ts';
import type { ExternalCallableEffect, } from './external-callable-effect.ts';

/**
 Records opacity for every capture an external formal's actual exposes.
 
 @param project - Consumer project resolving callables and binding symbols.
 
 @param bindingOriginBySymbolId - Parameter and alias origins of the callable being summarised.
 
 @param externalEffect - Proven external implementation effect naming which formals expose.
 
 @param declaration - Declaration the consumer resolved, whose formals order the arguments.
 
 @param call - Call whose arguments feed those formals.
 
 @param summary - Caller summary receiving opacity.
 
 @mutates summary - Adds an opaque slot per exposed capture, with the external export as its cause.
 
 @example
 ```ts
 recordExternalCaptureOpacity({
   project,
   bindingOriginBySymbolId,
   externalEffect,
   declaration,
   call,
   summary,
 });
 ```
 */
export function recordExternalCaptureOpacity({
  project,
  bindingOriginBySymbolId,
  externalEffect,
  declaration,
  call,
  summary,
}: {
  readonly project: Project;
  readonly bindingOriginBySymbolId: ReadonlyMap<number, SlotOrigins>;
  readonly externalEffect: ExternalCallableEffect;
  readonly declaration: Node;
  readonly call: CallExpression;
  readonly summary: MutableEffectSummary;
},): void {
  /**
   Actual expressions by external formal position, empty when no formal list could be read.
   */
  const actualsByFormal = formalActualExpressions({
    declaration,
    call,
  },);
  /**
   Actuals charged for a formal this mapping could not place, which is every argument.
   */
  const unplacedActuals = [...call.arguments,];
  exposingFormals(externalEffect,)
    .forEach(function chargeFormal(parameter,): void {
      (actualsByFormal[parameter] ?? unplacedActuals)
        .forEach(function chargeActual(actual,): void {
          exposedCaptureOrigins({
            project,
            bindingOriginBySymbolId,
            actual,
          },)
            .forEach(function markCaptured(origin,): void {
              addOpaqueEffect({
                summary,
                affectedSlot: origin,
                provenance: `${externalEffect.provenance} handed callable capture`,
              },);
            },);
        },);
    },);
}

/**
 Names the external formals whose handling can expose what a callable in that position hands back.
 
 Both sets are needed and each is pinned by its own case, because retention and invocation arrive as
 different facts. A store into a module binding is recorded as an opaque formal, and a formal the
 implementation calls is recorded as an invoked one. Measured by deleting each in turn: dropping
 invocation fails only the case whose export invokes, dropping opacity only the case whose export
 keeps.
 
 `callbackRelations` is deliberately not consulted, which is a correction rather than an omission. A
 relation names the formal the implementation feeds **into** a callback, never the formal **holding**
 one, so asking what a callable in that position exposes asks about the wrong argument: on a real
 shape the source actual is the value being passed along, which holds no callable, and the mutant
 removing this branch survived the whole suite.
 
 What that branch looked like it was for is covered already. An implementation that calls a handed
 callback records that formal as invoked, which the invoked case pins directly, and whatever the
 relation exposes about the caller's own values is charged by `applyExternalEffect` through their
 ordinary origins.
 
 @param externalEffect - Proven external implementation effect.
 
 @returns formal positions to charge, repeats included since charging twice adds nothing.
 
 @example
 ```ts
 exposingFormals(externalEffect);
 ```
 */
function exposingFormals(externalEffect: ExternalCallableEffect,): readonly number[] {
  return [
    ...externalEffect.summary
      .invokedParameterIndexes,
    ...externalEffect.summary
      .opaqueParameterIndexes,
  ];
}

/**
 Maps actual expressions from argument positions onto external formal positions.
 
 Shares `formalActualPositions` with the origin mapping rather than repeating it, so a rest formal, a
 spread and an explicit `this` formal are placed the same way for captures as for origins. Answering
 with expressions rather than origins is the whole difference: a capture lives in the argument's own
 syntax, not in any origin it carries.
 
 @param declaration - Declaration the consumer resolved.
 
 @param call - Call whose arguments feed the formals.
 
 @returns actual expressions by formal position.
 
 @example
 ```ts
 formalActualExpressions({ declaration, call });
 ```
 */
function formalActualExpressions({
  declaration,
  call,
}: {
  readonly declaration: Node;
  readonly call: CallExpression;
},): readonly (readonly Node[])[] {
  if (!isEffectCallableDeclaration(declaration,))
    return [];
  return formalActualPositions({
    callee: declaration,
    call,
  },)
    .map(function actualsForFormal(positions,): readonly Node[] {
      return positions.flatMap(function actualAtPosition(position,): readonly Node[] {
        /**
         Argument at this position, absent when the summary names more formals than the call fills.
         */
        const actual = call.arguments[position];
        return actual === undefined ? [] : [actual,];
      },);
    },);
}
