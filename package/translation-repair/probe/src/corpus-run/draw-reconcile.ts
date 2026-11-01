//region Draw reconcile refusal
// Why a final draw refuses an artifact it cannot reconcile with its parsed
// accepted population, as one marked class.
//
// THE ENTRY ID IS PRINTED ON PURPOSE. `run-json-read.ts` leaves that decision
// to the callers that hold an id, and this one holds it: an operator whose
// draw was refused needs to know which artifact to look at, and the pass
// already names entries on its `INCOMPLETE` lines. The id is a corpus path
// segment and carries no wording. WHAT SAT UNDER `acceptedCount` IS NOT
// PRINTED: the earlier bare `Error` copied it through `JSON.stringify`, and a
// stray value there could be anything, so only its `typeof` name travels.

/**
 Name `typeof` gives a value.

 @example
 ```ts
 const found: TypeofName = typeof 'one';
 ```
 */
export type TypeofName = 'bigint' | 'boolean' | 'function' | 'number' | 'object' | 'string' | 'symbol' | 'undefined';

/**
 Why an artifact could not be reconciled with the population parsed from it.

 @example
 ```ts
 const fault: DrawReconcileFault = { kind: 'count-disagrees', declared: 4, parsed: 1, };
 ```
 */
export type DrawReconcileFault = {
  /**
   Artifact records no number under `acceptedCount`.
   */
  readonly kind: 'no-numeric-count';

  /**
   `typeof` name of what sat there instead.
   */
  readonly foundType: TypeofName;
} | {
  /**
   Recorded count and parsed accepted population disagree.
   */
  readonly kind: 'count-disagrees';

  /**
   Count the artifact recorded.
   */
  readonly declared: number;

  /**
   Accepted issues parsing found.
   */
  readonly parsed: number;
};

/**
 Sentence for each fault, keyed by kind so every kind has exactly one.
 */
const RECONCILE_SENTENCES: {
  readonly [K in DrawReconcileFault['kind']]: (fault: Extract<DrawReconcileFault, { kind: K; }>) => string;
} = {
  'no-numeric-count': function noNumericCount(fault,): string {
    return `artifact records no numeric acceptedCount (found ${
      fault.foundType
    } there). Every artifact this pipeline writes carries one, so its absence means the file came `
      + 'from somewhere else and nothing can confirm the accepted population is complete';
  },
  'count-disagrees': function countDisagrees(fault,): string {
    return `artifact acceptedCount ${String(fault.declared,)} != parsed ${
      String(fault.parsed,)
    }; the accepted population would be silently short`;
  },
};

/**
 Words a reconcile fault, from its kind and numbers alone.

 @param fault - why the artifact could not be reconciled

 @returns Sentence naming the fault, without the entry

 @example
 ```ts
 const sentence = reconcileSentence({ fault: { kind: 'no-numeric-count', foundType: 'string', }, },);
 ```
 */
export function reconcileSentence({ fault, }: { readonly fault: DrawReconcileFault; },): string {
  if (fault.kind === 'no-numeric-count')
    return RECONCILE_SENTENCES['no-numeric-count'](fault,);
  return RECONCILE_SENTENCES['count-disagrees'](fault,);
}

/**
 Refusal of a final draw whose artifact cannot be reconciled with the
 accepted population parsed from it.

 MARKED, because its message is composed of the entry id, which these tools
 report by design, and sentences written here from a kind and numbers.

 @example
 ```ts
 throw new DrawReconcileError({ entryId: 'Toka_ls', fault: { kind: 'count-disagrees', declared: 4, parsed: 1, }, },);
 ```
 */
export class DrawReconcileError extends Error {
  /**
   Declares this message safe to forward: an entry id and a sentence composed
   here from a kind and numbers.
   */
  readonly messageNamesOnly: true = true;

  /**
   Entry whose artifact was refused.
   */
  readonly entryId: string;

  /**
   Why it was refused.
   */
  readonly fault: DrawReconcileFault;

  /**
   @param entryId - entry whose artifact was refused

   @param fault - why it could not be reconciled
   */
  constructor(
    {
      entryId,
      fault,
    }: {
      readonly entryId: string;
      readonly fault: DrawReconcileFault;
    },
  ) {
    super(`reconcile failed for ${entryId}: ${reconcileSentence({ fault, },)}`,);
    this.name = 'DrawReconcileError';
    this.entryId = entryId;
    this.fault = fault;
  }
}

//endregion Draw reconcile refusal
