/**
 Human-readable diagnostics for unresolved caller-observable effects.
 
 @module
 */

import type { Context, } from '@oxlint/plugins';

import type { ParameterIndex, } from './effect-slot-identity.ts';
import type { CallableEffectSummary, } from './effect-summaries.ts';
import {
  boundariesAreReportable,
  splitRetentionBoundaries,
} from './effect-retention-provenance.ts';
import { COLLECTION_MEMBER_NAMES, } from './effect-member-channel-authority.ts';
import {
  everyBoundaryIsInputMethod,
  inputMethodUsageSubject,
} from './input-diagnostic-description.ts';

/**
 Sorted uncertainty provenance and display text for one parameter.
 
 `facts` carries only the causes this report can address, which are unresolved calls.
 A store is recorded as opacity too, because an escaped reference is exactly a value the
 analysis cannot prove stays unwritten, but it is not a call and no remedy this report
 offers applies to it. Keeping it out of `facts` also keeps it out of
 `everyBoundaryIsInputMethod`, which is an `every` over the same list and would otherwise
 lose the method-specific message for any parameter that is both called and stored.
 
 @example
 ```ts
 const uncertainty: UncertaintyBoundaries = {
   facts: ['JSON.stringify'],
   names: 'JSON.stringify',
   reportable: true,
   retained: false,
 };
 ```
 */
export type UncertaintyBoundaries = {
  readonly facts: readonly string[];
  readonly names: string;
  readonly reportable: boolean;
  readonly retained: boolean;
};

/**
 Reads stable uncertainty provenance for one parameter.
 
 @param effectSummary - Callable effects carrying upstream provenance.
 
 @param parameterIndex - Parameter whose uncertainty is described.
 
 @returns sorted facts and human-readable fallback.
 
 @example
 ```ts
 uncertaintyBoundaries({ effectSummary, parameterIndex: 0 });
 ```
 */
export function uncertaintyBoundaries({
  effectSummary,
  parameterIndex,
}: {
  readonly effectSummary: CallableEffectSummary;
  readonly parameterIndex: ParameterIndex;
},): UncertaintyBoundaries {
  /**
   Sorted upstream boundary names retained by effect propagation, whatever caused them.
   */
  const boundaries = [
    ...effectSummary.opaqueProvenanceByParameter
      .get(parameterIndex,)
      ?? [],
  ].toSorted();
  /**
   Boundaries split into causes this report can address and stores it cannot.
   */
  const {
    callBoundaries,
    retentionBoundaries,
  } = splitRetentionBoundaries({ boundaries, },);
  return {
    facts: callBoundaries,
    /* The fallback belongs to the call half alone. Opacity with no provenance at all is a
     * genuine unknown and has to speak, but a parameter whose every recorded cause is a
     * store must not borrow that sentence: it would claim the rule could not determine a
     * name for something it named exactly. Whether that case reports is decided by
     * `reportableOpacity`, which is why this is safe to leave as the call half's text. */
    names: callBoundaries.length === 0
      ? 'a call whose name this rule could not determine'
      : callBoundaries.join(', ',),
    reportable: boundariesAreReportable({ boundaries, },),
    retained: retentionBoundaries.length > 0,
  };
}

/**
 Reads the member name out of one recorded boundary.
 
 A boundary is authored text plus a location, `slices.filter [path:line]`, so the member is
 the last dotted segment of the expression half. Parsed rather than carried alongside
 because the boundary set is what crosses into the summary and the cache, and widening that
 to a structured record would change the serialized shape for one message's benefit.
 
 @param boundary - Recorded boundary text.
 
 @returns member name, empty when the boundary names no member.
 
 @example
 ```ts
 collectionMemberOf({ boundary: 'slices.filter [src/a.ts:7]' });
 ```
 */
function collectionMemberOf({ boundary, }: { readonly boundary: string; },): string {
  /**
   Expression half, before the location this boundary was recorded at.
   */
  const [expression = '',] = boundary.split(' [',);
  return expression.split('.',)
    .at(-1,)
    ?? '';
}

/**
 Builds unresolved external effect report for one input.
 
 @param loc - Parameter source location.
 
 @param inputSubject - Plain-language input description.
 
 @param affectedNames - Bindings whose own slot carries the opacity, absent to name them all.
 
 @param targetIndexes - Authored targets mapped to parameter indexes.
 
 @param parameterIndex - Parameter carrying unresolved effects.
 
 @param uncertainty - Sorted upstream boundary description.
 
 @returns unresolved-effect report descriptor.
 
 @example
 ```ts
 opaqueEffectReport({ loc, inputSubject, targetIndexes, parameterIndex, uncertainty });
 ```
 */
export function opaqueEffectReport({
  loc,
  inputSubject,
  targetIndexes,
  parameterIndex,
  uncertainty,
  affectedNames,
}: {
  readonly loc: {
    readonly start: {
      readonly line: number;
      readonly column: number;
    };
    readonly end: {
      readonly line: number;
      readonly column: number;
    };
  };
  readonly inputSubject: string;
  readonly targetIndexes: ReadonlyMap<string, number>;
  readonly parameterIndex: number;
  readonly affectedNames?: ReadonlySet<string>;
  readonly uncertainty: UncertaintyBoundaries;
},): Parameters<Context['report']>[0] {
  /**
   Whether every unknown call is a method on one current input binding.
   */
  const onlyInputMethods = everyBoundaryIsInputMethod({
    boundaries: uncertainty.facts,
    targetIndexes,
    parameterIndex,
  },);
  /* A finding whose every cause is a collection member gets a message about collection
   * members. The general one offers to add a repository-owned implementation to a tsconfig,
   * which no engine intrinsic has, and to mark the input as a runtime-owned host capability,
   * which ordinary array data is not. Issue #414 reports exactly that: the message names no
   * change that resolves the finding it is attached to. */
  /**
   How many calls this finding names as causes.
   */
  const namedCallCount = uncertainty.facts
    .length;
  /**
   Whether every named call is a recognised collection member.
   
   Vacuously true for an empty list, which is why the count is asked separately: a finding
   with no named cause speaks through the general message rather than claiming to be about
   collections.
   */
  const everyCauseIsCollection = uncertainty.facts
    .every(function namesCollectionMember(boundary,): boolean {
      return COLLECTION_MEMBER_NAMES.has(collectionMemberOf({ boundary, },),);
    },);
  /**
   Whether this finding is entirely about collection members on one input.
   */
  const onlyCollectionMembers = onlyInputMethods
    && (namedCallCount > 0)
    && everyCauseIsCollection;
  return {
    loc,
    messageId: onlyCollectionMembers
      ? 'opaqueCollectionEffect'
      : onlyInputMethods
        ? 'opaqueMethodEffect'
        : 'opaqueEffect',
    data: {
      inputSubject: onlyInputMethods
        ? inputMethodUsageSubject({
          boundaries: uncertainty.facts,
          targetIndexes,
          parameterIndex,
          ...(affectedNames === undefined) ? {} : { affectedNames, },
        },)
        : inputSubject,
      boundaries: uncertainty.names,
    },
  };
}
