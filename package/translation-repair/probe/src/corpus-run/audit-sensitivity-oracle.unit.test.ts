/**
 Tests for scoring what an auditor pointed at against the planted span.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { RUN_MODELS, } from '@monochromatic-dev/translation-repair-run';
import type { RenderingAuditReport, } from '@monochromatic-dev/translation-repair-rendering-audit';
import {
  pointsAtOracle,
  shownText,
  sightedVoices,
} from '../../dist/final/node/index.mjs';
import {
  ADDITION_FINDING,
  ELSEWHERE_FINDING,
  NARROW_POLARITY_FINDING,
  POLARITY_FINDING,
  QUIET_AUDIT_REPORT,
} from '@monochromatic-dev/translation-repair-provider/ts/audit-scripted-client.test-fixture.ts';
import { auditReportOver, } from '../audit-report.test-fixture.ts';

/**
 First auditor of the roster.
 */
const [FIRST, SECOND,] = RUN_MODELS.checkerModelIds;

/**
 Whether each claim of each voice points at the planted span, in voice order.

 @param report - report the audit built

 @returns One flag per claim

 @example
 ```ts
 const flags = pointedByEveryClaim({ report, },);
 ```
 */
function pointedByEveryClaim({ report, }: { readonly report: RenderingAuditReport; },): readonly boolean[] {
  return report.rows.flatMap(function hits(row,): readonly boolean[] {
    return row.findings
      .map(function pointed(finding,): boolean {
        return pointsAtOracle({ finding, },);
      },);
  },);
}

await describe({
  name: '',
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    describe({
      name: pointsAtOracle.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'POINTS when both sides quote the planted span, and when the source side alone or the candidate side alone does',
          fn: async () => {
            /**
             One voice per kind of claim: both sides, a narrower quote, candidate only.
             */
            const report = await auditReportOver({
              reportFor: function perVoice(modelId,) {
                return {
                  verdict: 'defects-found',
                  findings: [
                    (modelId === FIRST)
                      ? POLARITY_FINDING
                      : ((modelId === SECOND) ? NARROW_POLARITY_FINDING : ADDITION_FINDING),
                  ],
                };
              },
              clean: false,
            },);

            expect(pointedByEveryClaim({ report, },),).toEqual([
              true,
              true,
              true,
            ],);
          },
        },),

        it({
          name: 'DOES NOT POINT at a claim about another clause whose candidate side rests on nothing',
          fn: async () => {
            /**
             Every voice claims the sleeping place.
             */
            const report = await auditReportOver({
              reportFor: function elsewhere() {
                return {
                  verdict: 'defects-found',
                  findings: [ELSEWHERE_FINDING,],
                };
              },
              clean: false,
            },);

            expect(pointedByEveryClaim({ report, },),).toEqual([
              false,
              false,
              false,
            ],);
          },
        },),
      ],
    },),

    describe({
      name: sightedVoices.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KEEPS the voices with at least one claim at the planted span and leaves out the quiet and the elsewhere ones',
          fn: async () => {
            /**
             First points at the span, second is quiet, third claims elsewhere.
             */
            const report = await auditReportOver({
              reportFor: function perVoice(modelId,) {
                if (modelId === FIRST)
                  return {
                    verdict: 'defects-found',
                    findings: [
                      ELSEWHERE_FINDING,
                      POLARITY_FINDING,
                    ],
                  };
                return (modelId === SECOND)
                  ? QUIET_AUDIT_REPORT
                  : {
                    verdict: 'defects-found',
                    findings: [ELSEWHERE_FINDING,],
                  };
              },
              clean: false,
            },);

            expect(sightedVoices({ rows: report.rows, },).map(function idOf(row,) {
              return row.modelId;
            },),).toEqual([FIRST,],);
          },
        },),

        it({
          name: 'KEEPS none from no rows',
          fn: async () => {
            expect(sightedVoices({ rows: [], },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: shownText.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SHOWS the focus wording of an anchored side and a dash for a side the category does not use',
          fn: async () => {
            /**
             Every voice claims the sleeping place, an original-only claim.
             */
            const report = await auditReportOver({
              reportFor: function elsewhere() {
                return {
                  verdict: 'defects-found',
                  findings: [ELSEWHERE_FINDING,],
                };
              },
              clean: false,
            },);
            const finding = nonNullishOrThrow(nonNullishOrThrow(report.rows[0],).findings[0],);

            expect(`${shownText({ reading: finding.source, },)}|${shownText({ reading: finding.candidate, },)}`,).toBe(
              '窗台|-',
            );
          },
        },),
      ],
    },),
  ],
},);
