import type { ArtifactContestSlice, } from './corpus-run/artifact-two-lane-contest.ts';
import type { ArtifactComparisonRow, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  ARCHIVE_TENSE_LABEL,
  REPAIR_TENSE_LABEL,
  type TenseCandidateGroup,
  tenseCandidateGroups,
  TRANSLATE_TENSE_LABEL,
} from '@monochromatic-dev/translation-repair-translate-stage/ts';

//region Lane contest tense
// THE TENSE CANDIDATES OF THE THREE TEXTS A LANE CONTEST OFFERS, in one place
// so the block the judges read and the finding the artifact records are the
// same list under the same labels. A judge is shown them as candidates to check
// against the house tense rule; the artifact keeps them as a finding for the
// audit trail and counts them toward no verdict, no flawed archive and no run-off.

/**
 Candidates of the archive wording and the two lanes' wordings, each under the
 label the contest sheet gives that text.

 @param lifeEnded - whether the original's entry records a death

 @param subjectNames - names the translation declares for its subject, read as named subjects

 @param incumbentText - archive's own wording

 @param repairText - wording the repair lane left

 @param translateText - wording the translate lane left

 @returns One group per text with a candidate; empty when the entry records no death or no text has one

 @throws Whatever the detector raises other than the shared slice grammar's refusal, which lists none for that text

 @example
 ```ts
 const groups = laneContestTenseGroups({ lifeEnded: true, incumbentText: 'Mittens naps.', repairText: 'Mittens napped.', translateText: 'Mittens slept.', },);
 ```
 */
export function laneContestTenseGroups(
  {
    lifeEnded,
    subjectNames,
    incumbentText,
    repairText,
    translateText,
  }: {
    readonly lifeEnded: boolean;
    readonly subjectNames?: readonly string[];
    readonly incumbentText: string;
    readonly repairText: string;
    readonly translateText: string;
  },
): readonly TenseCandidateGroup[] {
  return tenseCandidateGroups({
    lifeEnded,
    ...((subjectNames === undefined) ? {} : { subjectNames, }),
    texts: [
      {
        label: ARCHIVE_TENSE_LABEL,
        text: incumbentText,
      },
      {
        label: REPAIR_TENSE_LABEL,
        text: repairText,
      },
      {
        label: TRANSLATE_TENSE_LABEL,
        text: translateText,
      },
    ],
  },);
}

/**
 Adds to each contest slice the tense candidates its judges were shown, only
 where there are any. A slice with none is returned as it stands, and so is a
 sole-lane slice, which asked no judge.

 @param slices - contest records the driver returned

 @param comparison - rows carrying the three texts of each slice

 @param lifeEnded - whether the original's entry records a death

 @param subjectNames - names the translation declares for its subject

 @returns The same records, in order, each with its finding where one exists

 @throws {@link Error} when a slice that went to the roster has no comparison row, which the contest never produces

 @example
 ```ts
 const recorded = withTenseCandidates({ slices, comparison: projected.comparison, lifeEnded: true, },);
 ```
 */
export function withTenseCandidates(
  {
    slices,
    comparison,
    lifeEnded,
    subjectNames,
  }: {
    readonly slices: readonly ArtifactContestSlice[];
    readonly comparison: readonly ArtifactComparisonRow[];
    readonly lifeEnded: boolean;
    readonly subjectNames?: readonly string[];
  },
): readonly ArtifactContestSlice[] {
  if (!lifeEnded)
    return slices;
  return slices.map(function withFinding(slice,): ArtifactContestSlice {
    /**
     Verdict the contest recorded for this slice.
     */
    const { verdict, } = slice;
    // A SOLE-LANE SLICE ASKED NO JUDGE, so no sheet showed its texts and there
    // is no list for the record to keep.
    if (verdict.kind === 'sole-lane')
      return slice;
    /**
     Row holding the texts the judges of this slice were shown.
     */
    const row = comparison.find(function namesSlice(candidate,): boolean {
      return candidate.sliceIndex === slice.sliceIndex;
    },);
    if (row === undefined) {
      throw new Error(
        'unreachable: a contest answers only slices the comparison holds, so every contest record has a comparison row',
      );
    }
    /**
     What the judges were shown for this slice.
     */
    const tenseCandidates = laneContestTenseGroups({
      lifeEnded,
      ...((subjectNames === undefined) ? {} : { subjectNames, }),
      incumbentText: row.incumbentText,
      repairText: row.repairText,
      translateText: row.translateText,
    },);
    return (tenseCandidates.length === 0)
      ? slice
      : {
        ...slice,
        tenseCandidates,
      };
  },);
}

//endregion Lane contest tense
