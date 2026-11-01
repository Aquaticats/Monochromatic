import type { ArtifactConsolidateSlice, } from './corpus-run/artifact-two-lane-consolidate.ts';
import {
  shippedTenseLabelOf,
  tenseCandidateGroups,
} from '@monochromatic-dev/translation-repair-translate-stage/ts';

//region Consolidation shipped tense
// THE TENSE CANDIDATES OF THE TEXT A CONSOLIDATION SLICE SHIPPED, kept on the
// slice's record so a later read of the page sees the candidates of the wording
// that reached it, not only those of the wordings the contest judges were shown
// (`lane-contest-tense.ts` records those on the contest slice).
//
// ONLY WORDING THE SLICE ITSELF SHIPS. A consolidation, or an incumbent that
// stood in for an ineligible standing, carries its text on the record; a slice
// that kept the archive or changed nothing carries none, and the candidates of
// what it left standing are on the contest record already.
//
// A FINDING, NOT A VERDICT. Like the contest's, it counts toward no verdict, no
// flawed archive and no run-off.

/**
 Adds to a consolidation record the tense candidates of the wording it ships,
 only where there are any. A record that ships no wording of its own, and one
 whose wording has none, is returned as it stands.

 @param slice - consolidation record the driver made

 @param lifeEnded - whether the original's entry records a death

 @param subjectNames - names the translation declares for its subject, read as named subjects

 @returns The record, with its finding where the shipped wording has a candidate

 @throws Whatever the detector raises other than the shared slice grammar's refusal, which lists none for that text

 @example
 ```ts
 const recorded = withShippedTenseCandidates({ slice, lifeEnded: true, subjectNames: ['Mittens',], },);
 ```
 */
export function withShippedTenseCandidates(
  {
    slice,
    lifeEnded,
    subjectNames,
  }: {
    readonly slice: ArtifactConsolidateSlice;
    readonly lifeEnded: boolean;
    readonly subjectNames?: readonly string[];
  },
): ArtifactConsolidateSlice {
  if (!lifeEnded)
    return slice;
  /**
   Wording the slice ships.
   */
  const { shipped, } = slice;
  if ((shipped.kind === 'archive') || (shipped.kind === 'unchanged'))
    return slice;
  /**
   What the sheets called the shipped wording.
   */
  const label = shippedTenseLabelOf({ kind: shipped.kind, });
  /**
   The shipped wording's candidates, as one group or none.
   */
  const tenseCandidates = tenseCandidateGroups({
    lifeEnded,
    ...((subjectNames === undefined) ? {} : { subjectNames, }),
    texts: [
      {
        label,
        text: shipped.text,
      },
    ],
  },);
  return (tenseCandidates.length === 0)
    ? slice
    : {
      ...slice,
      tenseCandidates,
    };
}

//endregion Consolidation shipped tense
