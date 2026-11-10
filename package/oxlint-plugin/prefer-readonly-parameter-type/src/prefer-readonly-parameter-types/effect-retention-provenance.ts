/**
 Shared vocabulary for opacity a store caused, as opposed to opacity a call caused.
 
 Both causes travel in the same provenance strings, because that is what already crosses
 a call edge: `propagateUncertaintyProvenance` copies a callee's facts onto the caller
 verbatim, so a cause recorded in the text reaches every caller with no change to the
 summary model, the fixed point, the in-process cache or the persistent format. Reading a
 cause back out of that text is the idiom `everyBoundaryIsInputMethod` already uses to
 decide which message a boundary set deserves.
 
 The writer and the reader therefore share one constant instead of spelling the same
 prefix twice, which is the whole reason this module exists rather than a literal at each
 end.
 
 @module
 */

//region Provenance prefixes
// The three prefixes that mark a provenance fact as a store, another handoff,
// or a callable handed back.

/**
 Prefix marking a provenance fact as a store rather than an unresolved call.
 */
const RETENTION_PROVENANCE_PREFIX = 'stored into ';

/**
 Prefix marking a provenance fact as a handoff that is neither a store nor a return.
 
 A construction and a yield both give caller state to something outliving the call, and
 neither stores it anywhere a target could be named. Reusing the store prefix produced facts
 like `stored into a construction of RowKeeper`, which reads as a store into the construction
 rather than as a handoff to it, so the vocabulary gained a third entry instead.
 */
const HANDOFF_PROVENANCE_PREFIX = 'handed outward by ';

/**
 Prefix marking a provenance fact as a callable the callable handed back.
 
 A second prefix rather than a second target text, because nothing is stored when a callable
 is returned and a fact reading `stored into a callable returned to its caller` would be read
 as a store by the next person looking at a summary dump. The silence is what both prefixes
 share and it is what `isRetentionProvenance` decides, so the reader recognises both while
 the text stays true to what happened.
 */
const RETURNED_CALLABLE_PROVENANCE_PREFIX = 'handed back as a callable capturing it at ';

//endregion Provenance prefixes

//region Building a fact
// One provenance fact for a value handed to a binding the callable does not own.

/**
 Builds one provenance fact for a value the callable handed to a binding it does not own.
 
 @param target - Authored text of what the value was handed to.
 
 @param location - Where the store sits, so a report can point at it.
 
 @returns provenance fact carrying both cause and place.
 
 @example
 ```ts
 retentionProvenance({ target: 'this.#task', location: 'src/card.ts:60', });
 ```
 */
export function retentionProvenance({
  target,
  location,
}: {
  readonly target: string;
  readonly location: string;
},): string {
  return `${RETENTION_PROVENANCE_PREFIX}${target} [${location}]`;
}

//endregion Building a fact

//region Reading the facts back
// Whether a fact came from the retention channel, and the one pass that
// separates facts a call caused from facts a store caused.

/**
 Tests whether one provenance fact was written by the retention channel.
 
 Exported for the persistent-cache validator, which has to reject a retained result use
 whose provenance would not be read back as a retention. A payload carrying an empty or
 unprefixed string there passes every structural check and then reaches the diagnostic as
 call-caused opacity, which reports an unresolved effect naming a call that does not
 exist. The prefix is the whole of the retention contract, so the validator has to test
 it here rather than restate it.
 
 @param provenance - One recorded provenance fact.
 
 @returns whether the fact reads back as a store.
 
 @example
 ```ts
 isRetentionProvenance('stored into held [src/card.ts:60]',);
 ```
 */
export function isRetentionProvenance(provenance: string,): boolean {
  return provenance.startsWith(RETENTION_PROVENANCE_PREFIX,)
    || provenance.startsWith(HANDOFF_PROVENANCE_PREFIX,)
    || provenance.startsWith(RETURNED_CALLABLE_PROVENANCE_PREFIX,);
}

/**
 Separates provenance facts a call caused from facts a store caused.
 
 Kept as one pass returning both halves rather than two predicates over the same list,
 because every caller needs both: one half decides whether to speak and the other decides
 what to name.
 
 @param boundaries - Provenance facts recorded against one parameter.
 
 @returns facts split by cause, each keeping its original order.
 
 @example
 ```ts
 const { callBoundaries, } = splitRetentionBoundaries({ boundaries, },);
 ```
 */
export function splitRetentionBoundaries({
  boundaries,
}: {
  readonly boundaries: readonly string[];
},): {
  readonly callBoundaries: readonly string[];
  readonly retentionBoundaries: readonly string[];
} {
  /* Both halves ask `isRetentionProvenance` rather than spelling its test again, which is
   * what this module's own opening says it exists to prevent and what it was nonetheless
   * doing here. Adding a second retention prefix for a returned callable moved the predicate
   * and left these two behind, so a returned capture was classified a call, joined the
   * boundary list, and both spoke where it should have stayed silent and changed the leading
   * boundary of an unrelated parameter's message. Two symptoms, one cause, and the cause was
   * a duplicated test rather than the new prefix. */
  return {
    callBoundaries: boundaries.filter(function boundaryIsCall(boundary,): boolean {
      return !isRetentionProvenance(boundary,);
    },),
    retentionBoundaries: boundaries.filter(function boundaryIsRetention(boundary,): boolean {
      return isRetentionProvenance(boundary,);
    },),
  };
}

//endregion Reading the facts back

//region Reportable opacity
// Whether the opacity these facts describe has a cause a report can name.

/**
 Tests whether opacity described by these facts has a cause a report can ask about.
 
 Absence of provenance is reportable rather than silent, and getting that backwards is
 the mistake this exists to prevent. Opacity with nothing recorded against it is the
 genuine unknown the fallback wording was written for, while opacity whose every recorded
 cause is a store is understood completely. Testing an empty call list alone would make
 those two indistinguishable and silence the one that has to speak.
 
 @param boundaries - Provenance facts recorded against one parameter or one slot.
 
 @returns whether a report drawn from these facts would name a cause it can ask about.
 
 @example
 ```ts
 boundariesAreReportable({ boundaries: ['JSON.stringify'], },);
 ```
 */
export function boundariesAreReportable({
  boundaries,
}: {
  readonly boundaries: readonly string[];
},): boolean {
  /**
   Facts split by whether a report can address them.
   */
  const {
    callBoundaries,
    retentionBoundaries,
  } = splitRetentionBoundaries({ boundaries, },);
  return (callBoundaries.length > 0)
    || (retentionBoundaries.length === 0);
}

//endregion Reportable opacity

//region Handoff and return facts
// The fact for a callable handed back to the caller, and the fact for caller
// state handed to something outliving the call.

/**
 Builds one provenance fact for a callable the callable handed back to its caller.
 
 @param location - Where the return sits, so a summary dump can point at it.
 
 @returns provenance fact naming the return as the escape.
 
 @example
 ```ts
 returnedCallableProvenance({ location: 'src/card.ts:60', });
 ```
 */
export function returnedCallableProvenance({
  location,
}: {
  readonly location: string;
},): string {
  return `${RETURNED_CALLABLE_PROVENANCE_PREFIX}${location}`;
}

/**
 Builds one provenance fact for caller state handed to something outliving the call.
 
 @param handoff - What received the value, phrased to follow "handed outward by".
 
 @param location - Where the handoff sits, so a summary dump can point at it.
 
 @returns provenance fact naming the handoff as the escape.
 
 @example
 ```ts
 handoffProvenance({ handoff: 'a yield', location: 'src/card.ts:60', });
 ```
 */
export function handoffProvenance({
  handoff,
  location,
}: {
  readonly handoff: string;
  readonly location: string;
},): string {
  return `${HANDOFF_PROVENANCE_PREFIX}${handoff} [${location}]`;
}
//endregion Handoff and return facts
