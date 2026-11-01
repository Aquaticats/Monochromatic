/**
 Cross-signature mutation contract consistency checks.
 
 @module
 */

import type {
  Context,
  LineColumn,
} from '@oxlint/plugins';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type { Project, } from 'typescript/sync';

import {
  collectAstNodes,
  type EffectCallableDeclaration,
  isEffectCallableDeclaration,
} from './effect-summary-model.ts';
import {
  type EffectSummaryIndex,
  NO_EFFECT_SUMMARY,
} from './effect-summaries.ts';
import { SemanticBridgeError, } from './semantic-bridge-error.ts';

/**
 Converts TypeScript offset to Oxlint offset after BOM stripping.
 
 @param offset - TypeScript source offset.
 
 @param hasBOM - Whether Oxlint stripped leading BOM.
 
 @returns Oxlint source offset.
 */
function oxlintOffset({
  offset,
  hasBOM,
}: {
  readonly offset: number;
  readonly hasBOM: boolean;
},): number {
  return hasBOM ? Math.max(
    0,
    offset - 1,
  ) : offset;
}

/**
 Compares finite parameter-index sets.
 
 @param left - First index set.
 
 @param right - Second index set.
 
 @returns whether sets contain same indexes.
 */
function equalIndexes({
  left,
  right,
}: {
  readonly left: ReadonlySet<number>;
  readonly right: ReadonlySet<number>;
},): boolean {
  return (left.size === right.size)
    && [...left,].every(function inRight(index,): boolean {
      return right.has(index,);
    },);
}

/**
 Builds report location for callable name or declaration span.
 
 @param context - Rule context mapping offsets.
 
 @param declaration - Callable declaration to locate.
 
 @returns copied Oxlint location.
 */
function declarationLocation({
  context,
  declaration,
}: ForeignBorrowed<{
  readonly context: Context;
  readonly declaration: EffectCallableDeclaration;
}>,): {
  start: LineColumn;
  end: LineColumn
} {
  /**
   Source file owning declaration.
   */
  const sourceFile = declaration.getSourceFile();
  /**
   Named declaration node when callable exposes one.
   */
  const locationNode = ('name' in declaration) && (declaration.name !== undefined)
    ? declaration.name
    : declaration;
  /**
   Oxlint start offset after BOM normalization.
   */
  const start = oxlintOffset({
    offset: locationNode.getStart(sourceFile,),
    hasBOM: context.sourceCode
      .hasBOM,
  },);
  /**
   Oxlint end offset after BOM normalization.
   */
  const end = oxlintOffset({
    offset: locationNode.end,
    hasBOM: context.sourceCode
      .hasBOM,
  },);
  return {
    start: { ...context.sourceCode
      .getLocFromIndex(start,), },
    end: { ...context.sourceCode
      .getLocFromIndex(end,), },
  };
}

/**
 Reports implementation whose actual effects differ from union of overload contracts.
 
 @param context - Rule context receiving diagnostics.
 
 @param project - TypeScript project resolving overload symbols.
 
 @param sourceFile - Current source file whose overloads are checked.
 
 @param effectIndex - Whole-project callable summaries.
 
 @example
 ```ts
 verifyOverloadConsistency({ context, project, sourceFile, effectIndex });
 ```
 
 @mutates context - Emits Oxlint diagnostics through foreign rule context.
 */
export function verifyOverloadConsistency({
  context,
  project,
  sourceFile,
  effectIndex,
}: ForeignBorrowed<{
  readonly context: Context;
  readonly project: Project;
  readonly sourceFile: Parameters<typeof collectAstNodes>[0];
  readonly effectIndex: EffectSummaryIndex;
}>,): void {
  /**
   Callable declarations grouped by resolved name symbol.
   */
  const declarationsBySymbol = new Map<number, EffectCallableDeclaration[]>();
  collectAstNodes(sourceFile,)
    .forEach(function collect(node,): void {
      if ((!isEffectCallableDeclaration(node,))
        || (!('name' in node))
        || (node.name === undefined))
        return;
    /**
     Resolved callable symbol shared by overload declarations.
     */
      const symbol = project.checker
        .getSymbolAtLocation(node.name,);
      if (symbol === undefined)
        return;
      declarationsBySymbol.set(
        symbol.id,
        [
          ...declarationsBySymbol.get(symbol.id,) ?? [],
          node,
        ],
      );
    },);
  declarationsBySymbol.forEach(function verifyGroup(
    declarations: readonly EffectCallableDeclaration[],
  ): void {
    /**
     Implementation declarations carrying bodies.
     */
    const implementations = declarations.filter(function hasBody(declaration,): boolean {
      return ('body' in declaration) && (declaration.body !== undefined);
    },);
    /**
     Bodyless overload declarations carrying authored contracts.
     */
    const overloads = declarations.filter(function lacksBody(declaration,): boolean {
      return (!('body' in declaration)) || (declaration.body === undefined);
    },);
    if ((implementations.length !== 1) || (overloads.length === 0))
      return;
    /**
     Sole implementation declaration after cardinality guard.
     */
    const [implementation,] = implementations;
    if (implementation === undefined)
      return;
    /**
     Actual implementation effect summary.
     */
    const implementationSummary = effectIndex.get(implementation,);
    if (implementationSummary === NO_EFFECT_SUMMARY)
      throw new SemanticBridgeError({
        reason: 'node-not-found',
        message: 'Effect summary index omitted overload implementation.',
      },);
    /**
     Union of bodyless overload mutation contracts by parameter position.
     */
    const overloadEffects = overloads.reduce(
      /**
       Unions one overload contract into accumulated parameter positions.
       
       @param effects - Mutable accumulator of affected positions.
       
       @param overload - Current bodyless overload declaration.
       
       @returns same accumulator after adding current overload effects.
       
       @mutates effects - Adds mutation contract positions from current overload.
       */
      function union(
        effects,
        overload,
      ): Set<number> {
      /**
       Bodyless overload summary seeded from authored contracts.
       */
        const summary = effectIndex.get(overload,);
        if (summary === NO_EFFECT_SUMMARY)
          throw new SemanticBridgeError({
            reason: 'node-not-found',
            message: 'Effect summary index omitted bodyless overload.',
          },);
        summary.referentMutatedParameterIndexes
          .forEach(function add(index,): void {
            effects.add(index,);
          },);
        return effects;
      },
      new Set<number>(),
    );
    /* Referent mutations on both sides, because an authored contract can express
     * nothing else. `recordBodylessEffects` seeds an overload's `mutated` from
     * `@mutates` blocks and never touches `invoked`, while an implementation seeds
     * `invoked` from every callable it calls through a parameter. Comparing the union
     * against contracts therefore reported a category difference as a disagreement:
     * any callable that invokes something reached through a parameter differed from
     * its own overloads permanently, and the only way to silence it was to author a
     * `@mutates` claiming a write in order to acknowledge a call, which is the trade
     * `JCH` forbids.
     *
     * Measured on `invokedStepOverload` and its `invokedStepPlain` control in
     * `readonly-overload-invalid.ts`: identical bodies invoking a supplied step,
     * mutating nothing, and only the overloaded one was reported. The invoked
     * capability is still recorded and still propagates to callers; it is only no
     * longer read as a contract a signature failed to state. */
    if (equalIndexes({
      left: implementationSummary.referentMutatedParameterIndexes,
      right: overloadEffects,
    },))
      return;
    context.report({
      node: context.sourceCode
        .ast,
      loc: declarationLocation({
        context,
        declaration: implementation,
      },),
      messageId: 'inconsistentMutatesContract',
    },);
  },);
}
