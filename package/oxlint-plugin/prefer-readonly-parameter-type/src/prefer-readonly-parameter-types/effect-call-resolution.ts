/**
 Owned-call resolution and parameter-origin helpers.
 
 @module
 */

import {
  type Checker,
  type Project,
  SymbolFlags,
} from 'typescript/sync';
import type { Node, } from 'typescript/ast';
import {
  isArrayLiteralExpression,
  isAssertionExpression,
  isClassExpression,
  isFunctionLikeDeclaration,
  isIdentifier,
  isNonNullExpression,
  isObjectLiteralExpression,
  isParenthesizedExpression,
  isPropertyAssignment,
  isSatisfiesExpression,
  isShorthandPropertyAssignment,
  isSpreadAssignment,
  isSpreadElement,
  isVariableDeclaration,
} from 'typescript/ast/is';

import { packagedCallableOrigins, } from './effect-packaged-callable-origins.ts';
import { expressionElementOrigins, } from './effect-element-origin.ts';
import { expressionValueOrigins, } from './effect-expression-provenance.ts';
import {
  expressionCanCarryMutableState,
  receiverElementsArePrimitive,
} from './effect-primitive-origin.ts';
import { isWorkspaceSourceFileName, } from './workspace-source-path.ts';
import type { EffectSlot, } from './effect-slot-identity.ts';
import {
  type EffectCallableDeclaration,
  expressionRoot,
  isEffectCallableDeclaration,
  type MutableEffectSummary,
  NO_SLOT_ORIGIN,
  OWNED_CALLABLE_UNAVAILABLE,
  EFFECT_SLOT_UNAVAILABLE,
  type SlotOrigins,
} from './effect-summary-model.ts';

/**
 Maps an expression to every callable parameter its value can be reached from.
 
 {@inheritDoc expressionValueOrigins}
 
 @param project - TypeScript project resolving symbols and signatures.
 
 @param bindingOriginBySymbolId - Local binding symbols mapped to source parameters.
 
 @param node - Expression whose value may carry parameter state.
 
 @returns parameter origins, empty when value is not parameter-derived.
 
 @example
 ```ts
 rootParameterOrigins({ project, bindingOriginBySymbolId, node });
 ```
 */
export function rootParameterOrigins({
  project,
  bindingOriginBySymbolId,
  node,
}: {
  readonly project: Project;
  readonly bindingOriginBySymbolId: ReadonlyMap<number, SlotOrigins>;
  readonly node: Node;
},): SlotOrigins {
  return expressionValueOrigins({
    project,
    bindingOriginBySymbolId,
    node,
  },);
}

/**
 Collects every caller parameter origin packaged inside one call argument.
 
 @param project - TypeScript project resolving root symbols and signatures.
 
 @param bindingOriginBySymbolId - Local binding symbols mapped to source parameters.
 
 @param node - Call argument whose object or array may package parameter values.
 
 @returns unique caller parameter indexes in authored order.
 
 @example
 ```ts
 parameterIndexes({
   project,
   bindingOriginBySymbolId,
   node,
 });
 ```
 */
export function parameterIndexes({
  project,
  bindingOriginBySymbolId,
  node,
}: {
  readonly project: Project;
  readonly bindingOriginBySymbolId: ReadonlyMap<number, SlotOrigins>;
  readonly node: Node;
},): readonly EffectSlot[] {
  /**
   Checker for the project resolving this argument structure.
   */
  const { checker, } = project;
  /**
   Unique origins discovered through bounded argument structure.
   */
  const origins = new Set<EffectSlot>();
  /**
   Adds every origin a callable packaged inside this argument can hand over.
   
   @param packaged - Callable, accessor or method found inside the packaging structure.
   
   */
  function collectPackagedCallable(packaged: Node,): void {
    packagedCallableOrigins({
      project,
      bindingOriginBySymbolId,
      packaged,
    },)
      .forEach(function collectPackagedOrigin(origin,): void {
        origins.add(origin,);
      },);
  }
  /**
   Visits only authored object and array packaging structure.
   
   @param current - Current packaged expression or direct parameter root.
   
   */
  function collect(current: Node,): void {
    /* A spread argument, and any wrapper that changes nothing about the value it holds.
     * `call.arguments` contains the spread element itself, and the structural checks below
     * test for literal kinds, so a spread of an array literal reached none of them and
     * packaged nothing. `spreadEdgeEffect` in the call-edge fixture measured that: the
     * formal mapping pointed at the spread position and found an empty origin set there. */
    if (isSpreadElement(current,)) {
      collect(current.expression,);
      return;
    }
    if (isParenthesizedExpression(current,)
      || isNonNullExpression(current,)
      || isAssertionExpression(current,)
      || isSatisfiesExpression(current,)) {
      collect(current.expression,);
      return;
    }
    if (isClassExpression(current,)) {
      /* A class expression is neither a callable this routes to a body scan nor a literal it
       * descends, so a row held by a static member was reachable by the callee and invisible
       * here: `callee({ holder: class { static row = first; }, },)` attributed the callee's
       * write through `holder.row` to nothing. `classMemberPackaging` in the slot-narrowing
       * fixture measures it. Scanned for named bindings exactly as a method or accessor is,
       * which reaches a static initializer, a field initializer and anything a body closes
       * over. */
      collectPackagedCallable(current,);
      return;
    }
    /* Authored aggregate structure is walked here before the root resolver is consulted, and the
     * order is load-bearing. `provenanceSuccessors` also descends a literal now, so the resolver
     * answers for one, and answering first would end this walk before the branches below run.
     * Those branches are what reach a value held only by an accessor or a method, whose origin is
     * inside a body rather than in a property value. Measured: `accessorKeyBroadcast` in the
     * slot-narrowing fixture, whose row is reachable only through its getter, dropped from both
     * parameters to one, which is a lost write rather than lost precision. */
    if (isObjectLiteralExpression(current,)) {
      current.properties
        .forEach(function collectProperty(property,): void {
          if (isPropertyAssignment(property,)) {
          /**
           Authored property name used to select callee-mutated target.
           */
            const propertyName = property.name
              .getText();
          /* A property holding a callable does have a readable value, and reading it
           * still finds nothing: what reaches a parameter is the body the callee runs. */
            if (isFunctionLikeDeclaration(property.initializer,)) {
              collectPackagedCallable(property.initializer,);
              return;
            }
            collect(property.initializer,);
            return;
          }
          if (isShorthandPropertyAssignment(property,)) {
          /**
           Value symbol hidden behind object shorthand property symbol.
           */
            const valueSymbol = checker.getShorthandAssignmentValueSymbol(property,);
            if (valueSymbol !== undefined) {
            /**
             Caller parameter origins represented by shorthand value.
             */
              const shorthandOrigins = bindingOriginBySymbolId.get(valueSymbol.id,)
                ?? NO_SLOT_ORIGIN;
              if (expressionCanCarryMutableState({
                checker,
                node: property.name,
              },))
                shorthandOrigins.forEach(function collectShorthand(origin,): void {
                  origins.add(origin,);
                },);
            }
            return;
          }
          if (isSpreadAssignment(property,)) {
            collect(property.expression,);
            return;
          }
        /* Whatever is left has no property value this walk can read. An accessor
         * computes one by running its body when the callee reads the property; a method
         * is a body the callee calls. Both reach a parameter only through that body, so
         * both are scanned for named bindings. */
          collectPackagedCallable(property,);
        },);
      return;
    }
    /**
     Direct parameter origins at current expression root.
     */
    const direct = rootParameterOrigins({
      project,
      bindingOriginBySymbolId,
      node: current,
    },);
    if (direct.size > 0) {
      if (expressionCanCarryMutableState({
        checker,
        node: current,
      },))
        direct.forEach(function collectDirect(origin,): void {
          origins.add(origin,);
        },);
      return;
    }
    /* Handing over a fresh container hands over what it holds. `unresolvedSink(rows.slice())`
     * gives the callee every row the caller owns while the container's own value carries no
     * origin, so the value question answers empty and the call looked like it received
     * nothing. Measured with the container discharge in place and this branch absent: that
     * call reported no effect on `rows` at all, where it reported the receiver's opacity
     * before the discharge. A report traded for silence rather than for an attribution is the
     * one outcome the discharge may not produce. */
    /**
     Origins this argument hands over through its elements, for a verified container.
     */
    const throughElements = expressionElementOrigins({
      project,
      bindingOriginBySymbolId,
      node: current,
    },);
    if (throughElements.size > 0) {
      throughElements.forEach(function collectThroughElements(origin,): void {
        origins.add(origin,);
      },);
      return;
    }
    if (isArrayLiteralExpression(current,)) {
      current.elements
        .forEach(function collectElement(element,): void {
          if (!isSpreadElement(element,)) {
            if (isFunctionLikeDeclaration(element,)) {
              collectPackagedCallable(element,);
              return;
            }
            collect(element,);
            return;
          }
        /**
         Spread source type used to prove newly allocated arrays retain no
         caller-reachable object when every copied element is primitive.
         */
          const spreadType = checker.getTypeAtLocation(element.expression,);
          if ((spreadType !== undefined)
            && receiverElementsArePrimitive({
              checker,
              type: spreadType,
            },))
            return;
          collect(element.expression,);
        },);
    }
  }
  collect(node,);
  return [...origins,];
}

/**
 Resolves call target or callback expression to owned function-like declaration.
 
 @param project - TypeScript project resolving declaration handles.
 
 @param node - Callee or callback expression.
 
 @param analysisRoot - Optional external implementation root accepted as inspectable source.
 
 @returns owned function-like declaration or sentinel.
 
 @example
 ```ts
 callableDeclaration({ project, node });
 ```
 */
export function callableDeclaration({
  project,
  node,
  analysisRoot,
}: {
  readonly project: Project;
  readonly node: Node;
  readonly analysisRoot?: string;
},): EffectCallableDeclaration | typeof OWNED_CALLABLE_UNAVAILABLE {
  /**
   Cursor follows callable variable aliases iteratively.
   */
  const cursor: { current: Node; } = { current: node, };
  /**
   Stable node keys prevent cyclic callable alias traversal.
   */
  const visited = new Set<string>();
  while (!isEffectCallableDeclaration(cursor.current,)) {
    /**
     Stable source span for alias-cycle detection.
     */
    const cursorKey = `${cursor.current
      .getSourceFile()
      .fileName}:${String(cursor.current
        .pos,)}:${String(cursor.current
          .end,)}`;
    if (visited.has(cursorKey,))
      return OWNED_CALLABLE_UNAVAILABLE;
    visited.add(cursorKey,);
    /**
     Resolved symbol for identifier or expression.
     */
    const symbol = isIdentifier(cursor.current,)
      ? project.checker
        .getResolvedSymbol(cursor.current,)
      : project.checker
        .getSymbolAtLocation(cursor.current,);
    /**
     Import and re-export aliases followed to exact value declaration.
     */
    const aliasedSymbol = (symbol !== undefined)
      && ((symbol.flags & SymbolFlags.Alias) !== 0)
      ? project
        .checker
        .getAliasedSymbol(symbol,)
      : symbol;
    /**
     Resolved symbol or absent source symbol.
     */
    const declarationSymbol = aliasedSymbol ?? symbol;
    /**
     Preferred value declaration handle, with first declaration fallback.
     */
    const handle = declarationSymbol?.valueDeclaration
      ?? declarationSymbol?.declarations
        .at(0,);
    /**
     Resolved declaration in current project.
     */
    const declaration = handle?.resolve(project,);
    if (declaration === undefined)
      return OWNED_CALLABLE_UNAVAILABLE;
    if (isVariableDeclaration(declaration,)
      && (declaration.initializer !== undefined)) {
      cursor.current = declaration.initializer;
      continue;
    }
    cursor.current = declaration;
  }
  /**
   Owned callable declaration reached after alias traversal.
   */
  const declaration = cursor.current;
  /**
   Source file used to reject declaration and external-library boundaries.
   */
  const sourceFile = declaration.getSourceFile();
  if (sourceFile.isDeclarationFile)
    return OWNED_CALLABLE_UNAVAILABLE;
  if ((!project.program
    .isSourceFileFromExternalLibrary(sourceFile,))
    /* Symlink-resolved workspace dependencies classify as external while
     * living at repository paths; their source stays inspectable. */
    || isWorkspaceSourceFileName(sourceFile.fileName,)
    || ((analysisRoot !== undefined)
      && sourceFile.fileName
        .startsWith(analysisRoot,)))
    return declaration;
  return OWNED_CALLABLE_UNAVAILABLE;
}

/**
 Records unresolved external effect and callable provenance.
 
 @param summary - Callable summary receiving opaque effect.
 
 @param affectedSlot - Affected source parameter index.
 
 @param provenance - Authored external call expression text.
 
 @mutates summary - Adds opaque index and provenance evidence.
 
 @example
 ```ts
 addOpaqueEffect({ summary, affectedSlot, provenance });
 ```
 */
export function addOpaqueEffect({
  summary,
  affectedSlot,
  provenance,
}: {
  readonly summary: MutableEffectSummary;
  readonly affectedSlot: EffectSlot | typeof EFFECT_SLOT_UNAVAILABLE;
  readonly provenance: string;
},): void {
  if (affectedSlot === EFFECT_SLOT_UNAVAILABLE)
    return;
  summary.directOpaque
    .add(affectedSlot,);
  /**
   Existing provenance facts for parameter, or new accumulator.
   */
  const provenanceFacts = summary.opaqueProvenanceBySlot
    .get(affectedSlot,)
    ?? new Set<string>();
  provenanceFacts.add(provenance,);
  summary.opaqueProvenanceBySlot
    .set(
    affectedSlot,
    provenanceFacts,
    );
}
