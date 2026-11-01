import type { AdjudicatedIssue, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type ArchiveDispute,
  archiveDisputesOfRepair,
  claimLineOf,
  type DisputableChunk,
} from './archive-dispute.ts';

//region Accepted repair claims
// WHAT THE REPAIR PANEL ACCEPTED ABOUT A SLICE, for the sheets of the judges
// that decide a rewording of it (CuspariaKLSY21 slice 0, 2026-10-21).
//
// THE DISPUTE NOTE COVERS ONLY THE CLAIMS THAT DISPUTE THE ARCHIVE: an addition
// at any severity and another accuracy claim settled at major or worse
// (`archive-dispute.ts`). On that slice the panel accepted, 4 to 0 and twice,
// that "could not have a good meal" softens the source's hunger line, an
// `accuracy/mistranslation` claim settled at minor, so no dispute existed, no
// note was written and the polish gate chose "couldn't get a proper meal" 3 to
// 1 on a sheet that showed no claim. The producers had the claims (they wrote
// against them), so the judges that decide between a producer's text and its
// rewording read them too, in a block of the same shape the dispute note has.
//
// NO SEVERITY FILTER. The claims are shown whatever the panel settled them at;
// the block says they are claims to check against the original, not facts.

/**
 Accepted claims of the adjudicated issues, each as "category severity:
 summary" with the severity the panel settled, none twice.

 @param issues - adjudicated issues of one chunk

 @returns Claims of the accepted issues in issue order

 @example
 ```ts
 const claims = acceptedRepairClaimsOf({ issues: chunk.issues, },);
 ```
 */
export function acceptedRepairClaimsOf(
  { issues, }: { readonly issues: readonly AdjudicatedIssue[]; },
): readonly string[] {
  /**
   Every accepted issue's claims in the dispute note's wording.
   */
  const claims = issues
    .filter(function isAccepted(issue,): boolean {
      return issue.status === 'accepted';
    },)
    .flatMap(function claimsOf(issue,): readonly string[] {
      return issue.claims
        .map(function toLine(member,): string {
          return claimLineOf({
            member,
            severity: issue.severity,
          },);
        },);
    },);
  return [...new Set(claims,),];
}

/**
 Sheet block naming the claims the repair panel accepted for a slice.

 @param claims - accepted claims, each as "category severity: summary", never
 empty

 @returns Block headed for the sheets, saying what a judge is to make of them

 @example
 ```ts
 const note = acceptedRepairClaimsNote({ claims, },);
 ```
 */
export function acceptedRepairClaimsNote(
  { claims, }: { readonly claims: readonly string[]; },
): string {
  /**
   Claims numbered the way a ballot can cite them.
   */
  const numbered = claims
    .map(function toLine(
      claim,
      index,
    ): string {
      return `(${String(index + 1,)}) ${claim}`;
    },)
    .join('; ',);
  return `REPAIR PANEL CLAIMS ACCEPTED: the repair lane's adjudicators accepted ${
    String(claims.length,)
  } claim(s) of a defect in the archive rendering of this passage; the claims: ${numbered}. They are claims, not facts: `
    + 'check each against the ORIGINAL. Where the ORIGINAL supports a claim, a candidate that keeps the reading it names '
    + 'as wrong carries that error, and a candidate that corrects it is not departing from the ORIGINAL; a rewording '
    + 'that makes the correction softer, vaguer or less specific than the ORIGINAL puts the same fault back. Where the '
    + 'ORIGINAL does not support a claim, it counts for nothing.';
}

/**
 Sheet notes of the accepted repair claims per slice, for a stage that reads
 its slices by index.

 A claim the slice's dispute note already carries is left out, so no sheet
 names a claim twice, and a slice whose accepted claims are all in its dispute
 note has no note here.

 @param chunks - every chunk the repair lane settled

 @param disputes - the disputes read off the same chunks, by slice index

 @returns Notes keyed by slice index, in chunk order, none for a slice the
 panel accepted nothing on

 @example
 ```ts
 const notes = acceptedRepairClaimNotesOf({ chunks: repair.chunks, disputes, },);
 ```
 */
export function acceptedRepairClaimNotesOf(
  {
    chunks,
    disputes,
  }: {
    readonly chunks: readonly DisputableChunk[];
    readonly disputes: ReadonlyMap<number, ArchiveDispute>;
  },
): ReadonlyMap<number, string> {
  return new Map(chunks.flatMap(function toNote(chunk,): readonly (readonly [
    number,
    string,
  ])[] {
    /**
     This slice's dispute, absent on an undisputed slice.
     */
    const dispute = disputes.get(chunk.sliceIndex,);
    /**
     Claims the dispute note already names, none on an undisputed slice.
     */
    const shown = new Set((dispute === undefined) ? [] : dispute.acceptedClaims,);
    /**
     Accepted claims the dispute note does not carry.
     */
    const claims = acceptedRepairClaimsOf({ issues: chunk.issues, },)
      .filter(function notShown(claim,): boolean {
        return !shown.has(claim,);
      },);
    return (claims.length === 0)
      ? []
      : [[
        chunk.sliceIndex,
        acceptedRepairClaimsNote({ claims, },),
      ],];
  },),);
}

/**
 What the repair lane's panel accepted about the document, in the two forms
 the consolidation reads it: the disputes over archive renderings, and the
 notes of the accepted claims no dispute carries.

 @param repair - the repair lane's chunks and the slices its assembly withdrew

 @returns Disputes and claim notes keyed by slice index, read from the same
 chunks, so the notes leave out exactly the claims the disputes carry

 @example
 ```ts
 const { archiveDisputes, acceptedRepairClaimNotes, } = repairClaimEvidenceOf({ repair: lanes.repair, },);
 ```
 */
export function repairClaimEvidenceOf(
  { repair, }: {
    readonly repair: {
      readonly chunks: readonly DisputableChunk[];
      readonly withdrawnSliceIndices: readonly number[];
    };
  },
): {
  readonly archiveDisputes: ReadonlyMap<number, ArchiveDispute>;
  readonly acceptedRepairClaimNotes: ReadonlyMap<number, string>;
} {
  /**
   Disputes over the archive's renderings.
   */
  const archiveDisputes = archiveDisputesOfRepair({ repair, },);
  return {
    archiveDisputes,
    acceptedRepairClaimNotes: acceptedRepairClaimNotesOf({
      chunks: repair.chunks,
      disputes: archiveDisputes,
    },),
  };
}

//endregion Accepted repair claims
