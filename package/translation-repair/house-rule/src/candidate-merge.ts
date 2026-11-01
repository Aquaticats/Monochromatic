import {
  type Candidate,
  mergeProducers,
} from './candidate-select-model.ts';

//region Candidate merge
// Collapsing candidates that render the same text, keeping every author.
//
// Two things break when identical proposals stand as separate candidates. The
// ballot SPLITS, so text every producer agreed on can lose to a lone dissenter;
// and the pair-based self-vote discount stops applying, because each copy is
// credited to one model and the others look disinterested in it.
//
// The translate lane does this inline while assembling its slate, since it also
// has an incumbent to fold in. This is the plain version for lanes that only
// have model proposals.

/**
 Merges candidates rendering identical text, crediting every producer.

 Order is preserved and the FIRST copy holds the position, so a lane that
 assembles candidates in roster order keeps that order.

 @param candidates - proposals as their producers made them

 @returns Distinct proposals, each carrying every model that wrote it

 @example
 ```ts
 const distinct = mergeIdenticalCandidates({ candidates, },);
 ```
 */
export function mergeIdenticalCandidates<ValueT,>(
  { candidates, }: { readonly candidates: readonly Candidate<ValueT>[]; },
): readonly Candidate<ValueT>[] {
  /**
   Kept candidates by rendered text, in first-seen order.
   */
  const byText = new Map<string, Candidate<ValueT>>();
  for (const candidate of candidates) {
    /**
     Earlier candidate rendering the same text, when one exists.
     */
    const kept = byText.get(candidate.rendered,);
    if (kept === undefined) {
      byText.set(
        candidate.rendered,
        candidate,
      );
      continue;
    }
    byText.set(
      candidate.rendered,
      {
        ...kept,
        producer: mergeProducers({
          left: kept.producer,
          right: candidate.producer,
        },),
      },
    );
  }

  return [...byText.values(),];
}

//endregion Candidate merge
