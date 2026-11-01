import type { ScreenedFinding, } from '@monochromatic-dev/translation-repair-rendering-audit/ts';
import type { AuditRelocationPair, } from './rendering-audit-settled-read.ts';
import {
  type SettledAuditRow,
  SLOT_SEPARATOR,
} from './rendering-audit-settled-row.ts';

//region Relocation pairing
// The relocation reading rule, in code.
//
// WHAT THE RULE SAYS: per-slice judging cannot tell a RELOCATION from a
// fabrication. A passage the archive carried across a slice boundary is missing
// from the slice it belonged to and present in a slice that does not account
// for it, so one move is reported twice: as an omission on one slice and as an
// unsupported addition on its neighbour. Counting both condemns the archive for
// doing something reasonable.
//
// WHAT THIS DOES ABOUT IT: names the pairs. It does NOT subtract them from any
// tally. Nothing here can tell a real relocation from a genuine omission that
// happens to sit beside a genuine addition, and a rule that silently removed
// both would be the same defect in the other direction: an instrument that
// cannot see a class of damage.
//
// FIXED BEFORE THE NUMBERS EXISTED, while the full run was still buying its
// subjects. Adjacency is ONE slice, and it is one because a moved passage lands
// next door; widening it after seeing a tally would let the rule absorb
// whatever it needed to absorb.

/**
 Category naming a passage the candidate does not carry.
 */
const OMISSION: string = 'omission';

/**
 Category naming a passage the original does not carry.
 */
const UNSUPPORTED_ADDITION: string = 'unsupported-addition';

/**
 How far apart two slices may be and still be read as one relocation.
 */
const ADJACENT: number = 1;

/**
 One document as this rule reads it: a run set, an entry, and its slices.

 @example
 ```ts
 const key = documentKey({ row, },);
 ```
 */
type DocumentKey = string;

/**
 Names the document a row belongs to.

 BOTH the run set and the entry, because two runs of one entry write rows with
 the same entry id and the same slice indices. Pairing across them would
 report a relocation nobody's document contains.

 @param row - one audited slice

 @returns Key unique to one document of one run

 @example
 ```ts
 const key = documentKey({ row, },);
 ```
 */
function documentKey({ row, }: { readonly row: SettledAuditRow; },): DocumentKey {
  return `${row.runSet}${SLOT_SEPARATOR}${row.entryId}`;
}

/**
 Every claim on one subject that anchored, in one category.

 @param row - one audited slice

 @param category - category to keep

 @returns Matching findings across all voices

 @example
 ```ts
 const missing = findingsOf({ row, category: 'omission', },);
 ```
 */
function findingsOf(
  {
    row,
    category,
  }: {
    readonly row: SettledAuditRow;
    readonly category: string;
  },
): readonly ScreenedFinding[] {
  /**
   Every voice's screened answer on this subject.
   */
  const voices = row.report
    .rows;

  return voices
    .flatMap(function claimsOf(voice,): readonly ScreenedFinding[] {
      return voice.findings;
    },)
    .filter(function isCategory(finding,): boolean {
      return finding.category === category;
    },);
}

/**
 Pairs an omission on one slice with an addition on its neighbour.

 @param rows - every audited slice, from any number of documents

 @returns Candidates, in row order, empty when nothing pairs

 @example
 ```ts
 const candidates = auditRelocationPairs({ rows, },);
 ```
 */
export function auditRelocationPairs(
  { rows, }: { readonly rows: readonly SettledAuditRow[]; },
): readonly AuditRelocationPair[] {
  /**
   Rows keyed by the document they belong to, so no pairing crosses documents
   or crosses two runs of one document.
   */
  const byDocument = new Map<DocumentKey, readonly SettledAuditRow[]>();
  rows.forEach(function place(row,): void {
    /**
     Document this row belongs to.
     */
    const key = documentKey({ row, },);
    byDocument.set(
      key,
      [
        ...byDocument.get(key,) ?? [],
        row,
      ],
    );
  },);

  return [...byDocument.values(),].flatMap(function pairWithin(
    inDocument,
  ): readonly AuditRelocationPair[] {
    return inDocument.flatMap(function fromSlice(row,): readonly AuditRelocationPair[] {
      /**
       Passages this slice was told it dropped.
       */
      const missing = findingsOf({
        row,
        category: OMISSION,
      },);
      if (missing.length === 0)
        return [];

      /**
       Slices next door, either side.
       */
      const neighbours = inDocument.filter(function isAdjacent(other,): boolean {
        return Math.abs(other.sliceIndex - row.sliceIndex,) === ADJACENT;
      },);

      return neighbours.flatMap(function against(neighbour,): readonly AuditRelocationPair[] {
        /**
         Passages the neighbour was told it invented.
         */
        const added = findingsOf({
          row: neighbour,
          category: UNSUPPORTED_ADDITION,
        },);

        return missing.flatMap(function pair(gone,): readonly AuditRelocationPair[] {
          return added.map(function with_(extra,): AuditRelocationPair {
            return {
              runSet: row.runSet,
              entryId: row.entryId,
              omissionAt: row.sliceIndex,
              additionAt: neighbour.sliceIndex,
              omissionReason: gone.reason,
              additionReason: extra.reason,
            };
          },);
        },);
      },);
    },);
  },);
}

/**
 Counts the distinct pairs of slices the candidates sit on.

 THE NUMBER A READER MEANS BY "relocations". `auditRelocationPairs` pairs
 claims, one per omission finding and addition finding across every voice, so
 one moved passage that three voices noticed on each side is nine candidates.
 Keyed on the run set as well as the entry, for the reason the pairing is: two
 runs of one entry write the same indices.

 @param pairs - candidates as paired

 @returns How many distinct (run set, entry, omission slice, addition slice)
 tuples they cover

 @example
 ```ts
 const slicePairs = distinctSlicePairs({ pairs, },);
 ```
 */
export function distinctSlicePairs(
  { pairs, }: { readonly pairs: readonly AuditRelocationPair[]; },
): number {
  return new Set(pairs.map(function keyOf(pair,): string {
    return [
      pair.runSet,
      pair.entryId,
      String(pair.omissionAt,),
      String(pair.additionAt,),
    ].join(SLOT_SEPARATOR,);
  },),).size;
}

//endregion Relocation pairing
