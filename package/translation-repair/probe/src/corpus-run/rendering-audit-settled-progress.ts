import {
  pageRelationFor,
  pageRelationLabel,
} from './rendering-audit-settled-relation.ts';
import type { SettledAuditRow, } from './rendering-audit-settled-row.ts';

//region Settled audit progress
// The one line a long run prints for each slice it audits, as it lands.

/**
 Prints one audited slice as it lands, so a long run can be watched.

 @param row - what the roster said about one slice

 @example
 ```ts
 printSettledRow({ row, },);
 ```
 */
export function printSettledRow({ row, }: { readonly row: SettledAuditRow; },): void {
  /**
   Where this slice came from and what the two tiers made of it.
   */
  const {
    runSet,
    entryId,
    sliceIndex,
    auditsArchiveText,
    report,
  } = row;

  /**
   Whether a later stage overruled the wording just audited.

   PRINTED BESIDE the archive-versus-fresh token rather than replacing it.
   `FRESH` says the lane produced this wording, which stays true however
   the contest and the consolidation later ruled; without the relation
   beside it a watcher reads every `FRESH` line as the product.
   */
  const relation = pageRelationFor({ row, },);

  /**
   Both agreement tiers, the near misses, the degradation and every voice.
   */
  const {
    corroborated,
    agreed,
    near,
    findings,
    rows,
  } = report;

  /**
   Claims that anchored, across the whole roster.

   PRINTED BESIDE THE TIERS because the difference between them is the
   measurement: voices that claimed plenty and agreed on none says something
   about the matcher, and a silent roster says something else entirely.
   */
  const claimed = rows.reduce(
    function total(
      sum,
      voice,
    ): number {
      /**
       What this voice claimed that anchored.
       */
      const { findings: anchored, } = voice;
      return sum + anchored.length;
    },
    0,
  );

  console.log(
    `${runSet}/${entryId}#${String(sliceIndex,)} ${
      auditsArchiveText ? 'ARCHIVE' : 'FRESH  '
    } ${pageRelationLabel({ relation, },)} claimed=${String(claimed,)} corroborated=${
      String(corroborated.length,)
    } agreed=${
      String(agreed.length,)
    } near=${String(near.length,)}${
      (findings.length === 0) ? '' : ` degraded=${String(findings.length,)}`
    }`,
  );
}

//endregion Settled audit progress
