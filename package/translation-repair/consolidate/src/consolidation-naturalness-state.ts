import type { AbsoluteNaturalnessReviewOutcome, } from '@monochromatic-dev/translation-repair-archive/ts';

//region Consolidation naturalness state helpers

/**
 Renders latest structured findings for stage telemetry.

 @param review - exact rejected review feeding correction

 @returns Paragraph-located descriptions

 @example
 ```ts
 const findings = describeReviewFindings({ review, });
 ```
 */
export function describeReviewFindings(
  { review, }: { readonly review: AbsoluteNaturalnessReviewOutcome; },
): readonly string[] {
  return review.findings
    .map(function describe(finding,): string {
      return `Paragraph ${String(finding.paragraph,)}: ${finding.problem}`;
    },);
}

//endregion Consolidation naturalness state helpers
