import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { IntroducedDefectReport, } from '@monochromatic-dev/translation-repair-adjudication/ts';
import { silentStagesOf, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Refine probe verdict
// What the damage probe's report over one rewrite comes to for the rollback
// the naturalness lane now takes on it (ledger L11). Its own file because the
// slice settlement is at its line budget.

/**
 Claims the screen admitted against one rewrite, by direction.

 @example
 ```ts
 const counts: AdmittedClaimCounts = { added: 1, dropped: 0, };
 ```
 */
export type AdmittedClaimCounts = {
  /**
   Claims of added damage whose quote is in the rewrite and absent before it.
   */
  readonly added: number;

  /**
   Claims of dropped content whose quote is in the text before the rewrite
   and absent after it.
   */
  readonly dropped: number;
};

/**
 What the damage probe's report decides for the rewrite it read.

 @example
 ```ts
 const verdict: RefineProbeVerdict = { kind: 'rolled-back', finding: 'refine-probe-unheard (0 of 3 probers heard)', };
 ```
 */
export type RefineProbeVerdict =
  | { readonly kind: 'kept'; }
    | {
      readonly kind: 'rolled-back';

    /**
     Why the rewrite was rolled back, in scorecard-stable wording.
     */
      readonly finding: string;
    };

/**
 Sums the claims the screen admitted across a report's regions.

 ADMITTED, NOT AGREED: a claim counts when the deterministic differential
 bears out its quote, whichever prober made it. Requiring two probers'
 claims on one rewrite would have caught 9 of the 175 flagged over every run.

 @param report - probe report over the rewrite

 @returns Admitted claims, by direction

 @example
 ```ts
 const admitted = admittedClaimCounts({ report: refinementDefects, },);
 ```
 */
function admittedClaimCounts(
  { report, }: { readonly report: IntroducedDefectReport; },
): AdmittedClaimCounts {
  return report.regions
    .reduce<AdmittedClaimCounts>(
    function addRegion(
      running,
      region,
    ): AdmittedClaimCounts {
      return {
        added: running.added + region.corroborated,
        dropped: running.dropped + region.removalCorroborated,
      };
    },
    {
      added: 0,
      dropped: 0,
    },
    );
}

/**
 Whether the damage probe's report lets the rewrite it read ship, and why
 not when it does not.

 AN ADMITTED CLAIM ROLLS THE REWRITE BACK (ledger L11), whatever the number
 of probers heard: one prober's claim counts when the screen bears out its
 quote, so a claim a lone heard prober made is damage evidence on its own.

 A ROUND SHORT OF ITS QUORUM ROLLS IT BACK TOO, under
 `refine-probe-unheard`, as a recheck short of its checkers' quorum does:
 no claim from a round that heard too few probers is silence, not a clean
 bill, and the package's gates between a new text and a standing one ship
 the standing text unless enough voices stood behind the new one. The quorum
 is read through `silentStagesOf`, the reader the caches and the recheck
 use, off the probe stage's own `stage-quorum-unmet` finding.

 CLAIMS ARE READ FIRST because the settler attaches no report to a rollback:
 a short round named unheard would leave a claim the screen admitted in no
 finding and no outcome. Its short round still shows, since the probe
 stage's own findings ride beside either rollback.

 @param report - probe report over the rewrite

 @returns Kept, or rolled back with the finding that says why

 @example
 ```ts
 const verdict = refineProbeVerdict({ report: refinementDefects, },);
 ```
 */
export function refineProbeVerdict(
  { report, }: { readonly report: IntroducedDefectReport; },
): RefineProbeVerdict {
  /**
   Claims the screen admitted against the rewrite: damage it added, quoted
   from the rewrite and absent before, and content it dropped, quoted from
   the text before it and absent after.
   */
  const admitted = admittedClaimCounts({ report, },);
  if ((admitted.added + admitted.dropped) > 0) {
    return {
      kind: 'rolled-back',
      finding: `refine-rolled-back-by-probe (${String(admitted.added,)} added-damage and `
        + `${String(admitted.dropped,)} removal ${
          wordForCount({
            count: admitted.dropped,
            one: 'claim',
            many: 'claims',
          },)
        } admitted against the rewrite)`,
    };
  }

  /**
   The probe stage's findings that it closed short of its quorum, empty when
   enough probers were heard.
   */
  const shortOfQuorum = silentStagesOf({ findings: report.findings, },);
  if (shortOfQuorum.length > 0) {
    return {
      kind: 'rolled-back',
      finding: `refine-probe-unheard (${String(report.heardProbers,)} of ${String(report.configuredProbers,)} ${
        wordForCount({
          count: report.configuredProbers,
          one: 'prober',
          many: 'probers',
        },)
      } heard)`,
    };
  }
  return { kind: 'kept', };
}

//endregion Refine probe verdict
