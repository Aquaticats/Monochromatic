/**
 Public immutable effect-summary lookup model.
 
 Everything here is keyed by parameter, deliberately, even though effects are recorded
 against slots internally. A parameter position is the only thing that means the same on
 both sides of a project boundary, in an overload comparison, and in a diagnostic, and a
 branded `ParameterIndex` is what stops a slot arriving where one of those is expected.
 `effect-public-summary.ts` performs the projection.
 
 The brand earns its keep here more than anywhere else. A branded number is still assignable
 to `number`, so declaring these sets as `ReadonlySet<number>` would have let slot-keyed sets
 through untouched and unprojected, which reads as a parameter that carries no effect.
 
 @module
 */

import type { ExternalOmissionRecord, } from './effect-external-omission.ts';
import type { ParameterIndex, } from './effect-slot-identity.ts';
import type { EffectCallableDeclaration, } from './effect-summary-model.ts';

/**
 One callback relation as a consumer outside the analyzer sees it.
 
 Both positions are parameter positions of the declaring callable, projected from the slots
 the relation was inferred against.
 */
export type PublicCallbackRelation = {
  readonly callbackParameterIndex: ParameterIndex;
  readonly callbackArgumentPosition: number;
  readonly sourceParameterIndex: ParameterIndex;
};

/**
 Readonly effect summary exposed to rule verification.
 
 @example
 ```ts
 if (summary.mutatedParameterIndexes.has(asParameterIndex(0,),)) {
   // first parameter may be affected
 }
 ```
 */
export type CallableEffectSummary = {
  readonly mutatedParameterIndexes: ReadonlySet<ParameterIndex>;
  readonly referentMutatedParameterIndexes: ReadonlySet<ParameterIndex>;
  readonly returnedParameterIndexes: ReadonlySet<ParameterIndex>;
  readonly invokedParameterIndexes: ReadonlySet<ParameterIndex>;
  readonly opaqueParameterIndexes: ReadonlySet<ParameterIndex>;
  readonly opaqueProvenanceByParameter: ReadonlyMap<ParameterIndex, ReadonlySet<string>>;
  readonly callbackRelations: readonly PublicCallbackRelation[];
  /**
   Authored binding names whose own slot carries the opacity, per parameter.
   
   The only slot-derived thing crossing this boundary, and it crosses as names rather than
   slots because a slot number means nothing outside the declaration that allocated it. A
   parameter is absent when nothing beneath it is opaque, which is not the same as an empty
   set: an effect on the whole-parameter slot lists every binding under it.
   */
  readonly opaqueBindingsByParameter: ReadonlyMap<ParameterIndex, ReadonlySet<string>>;
};

/**
 Whole-project effect lookup tied to one TypeScript snapshot project.
 */
export type EffectSummaryIndex = {
  /**
   Looks up summary for exact callable declaration node.
   */
  readonly get: (
    declaration: EffectCallableDeclaration,
  ) => CallableEffectSummary | typeof NO_EFFECT_SUMMARY;
  /**
   Proves which parameters a marker holds under foreign ownership.
   
   Separate from the summary because it is priced differently. Every other fact above is read
   off the callable's own fixed point, while this one walks the complete backwards caller
   closure of the whole configured scope, once per callable asked about, and that walk is the
   single largest cost the rule carries. A consumer that cannot act on the answer should not
   pay for it, and the only way to express that is to make asking a separate act.
   */
  readonly proveForeignBorrowed: (
    declaration: EffectCallableDeclaration,
  ) => ReadonlySet<ParameterIndex>;
  /**
   Lists the sources with omitted callables that the callable's summary was built from.
   
   Separate from the summary because only a consumer applying this index's effects to its own
   summaries needs it: those summaries outlive the index in a cache, and a restore that never
   builds the index again can say what a fresh scan said only if the entry recorded it.
   Covers the declaring source and every source reached from it, each once, sorted by name.
   */
  readonly reachedOmissions: (
    declaration: EffectCallableDeclaration,
  ) => readonly ExternalOmissionRecord[];
};

/**
 Sentinel when declaration is outside indexed owned source.
 */
export const NO_EFFECT_SUMMARY: unique symbol = Symbol(
  'declaration lacks indexed CallableEffectSummary',
);
