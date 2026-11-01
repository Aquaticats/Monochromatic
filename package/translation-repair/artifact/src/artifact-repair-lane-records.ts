import { requireArray, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import { parseSettledTwoLaneArtifact, } from './corpus-run/artifact-two-lane-read.ts';

//region Repair lane records
// Where the repair lane's own records live, said once, for every reader that
// wants them.
//
// THE PATH MOVED AND NOTHING NOTICED. Every reader of these records keyed on a
// top-level `artifact.issues` and `artifact.findings` that version 1 wrote and
// version 2 does not. The records did not go away. Across the 47 settled
// artifacts they sit at `lanes.repair.result`: 1546 issue records, 827 of them
// carrying a `shipped` disposition, and 2809 findings. At the keys the readers
// used there are none of either.
//
// What made the move invisible is that the readers walked RAW parsed JSON, so
// a wrong key was never a type error, only an `undefined`. One reader answered
// that `undefined` with an empty list and reported zero shipped records over a
// whole corpus, which reads exactly like a run that shipped nothing. Another
// refused, which at least said something was wrong. Neither could say the KEY
// was wrong.
//
// The findings case is the sharper one: that list exists, by its own reader's
// comment, to notice a stage going quiet. It has reported every stage quiet on
// every artifact since the path moved.
//
// The cure is not a corrected string. It is going through the version 2 parser,
// so the walk to the lane is type-checked and a later move breaks the build
// instead of returning a confident zero.

/**
 Repair lane's two record lists, as one artifact carries them.

 @example
 ```ts
 const { issues, findings, } = repairLaneRecordsOf({ value, path: 'Kitten', },);
 ```
 */
export type RepairLaneRecords = {
  /**
   One record per issue the lane adjudicated, each still unread.
   */
  readonly issues: readonly unknown[];

  /**
   What the lane's stages reported, each still unread.
   */
  readonly findings: readonly unknown[];
};

/**
 Reads the repair lane's records out of one settled artifact.

 VERSION 2 OWNS THE ENVELOPE AND THE LANE OWNS ITS CONTENTS, which is why this
 is split across a parse and two guards. `parseSettledTwoLaneArtifact` proves the
 walk as far as the lane's `result` record and hands it back unread, because
 version 2 deliberately says nothing about what a result holds. The two arrays
 inside it are the repair lane's own schema, so they are checked here.

 BOTH LISTS ARE READ IN ONE PARSE. Version 2 recomputes the comparison and
 checks it against the copy on file, so asking twice pays for that twice and
 invites the two answers to come from different reads of the same artifact.

 ABSENCE REFUSES rather than reading as an empty corpus. On every settled
 artifact both paths hold an array, an empty one included: an entry that files
 no issue still writes `issues: []`. So a missing array does not mean a quiet
 run, it means this reader and the writer disagree, and answering that with an
 empty list is what hid the moved path in the first place.

 @param value - parsed artifact JSON, unread

 @param path - dotted path naming this artifact in error messages

 @returns Both record lists as written, each element still unread

 @throws {@link ArtifactParseError} when version 2 refuses the artifact, or
 when the repair lane's result carries either name as something else

 @example
 ```ts
 const { issues, } = repairLaneRecordsOf({ value, path: 'Kitten', },);
 ```
 */
export function repairLaneRecordsOf(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): RepairLaneRecords {
  /**
   Artifact with its envelope proven, so reading `lanes.repair.raw` is type-checked.
   */
  const artifact = parseSettledTwoLaneArtifact({ value, },);

  /**
   Repair lane's result exactly as the file holds it.

   SPELLED `raw` HERE AND `result` ON DISK. The error paths this function
   builds use the
   on-disk spelling, because they are read by someone holding the file rather
   than this type.
   */
  const result = artifact
    .lanes
    .repair
    .raw;

  return {
    issues: requireArray({
      value: result.issues,
      path: `${path}.lanes.repair.result.issues`,
    },),
    findings: requireArray({
      value: result.findings,
      path: `${path}.lanes.repair.result.findings`,
    },),
  };
}

//endregion Repair lane records
