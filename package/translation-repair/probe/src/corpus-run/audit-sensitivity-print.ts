import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type {
  AuditVoiceRow,
  RenderingAuditReport,
} from '@monochromatic-dev/translation-repair-rendering-audit/ts';
import {
  pointsAtOracle,
  shownText,
} from './audit-sensitivity-oracle.ts';

//region Audit sensitivity print
// What the audit sensitivity runner says, as text a case can read whole: the
// lines of each arm and the line saying where the run was kept.

/**
 Lines one voice's row leaves, so a failed arm can be attributed rather than
 guessed at.

 @param row - one auditor's screened answer

 @param arm - which arm this row came from

 @returns The voice line, then one line per claim, without newlines

 @example
 ```ts
 const lines = voiceLines({ row, arm: 'flipped', },);
 ```
 */
export function voiceLines(
  {
    row,
    arm,
  }: {
    readonly row: AuditVoiceRow;
    readonly arm: string;
  },
): readonly string[] {
  /**
   Claims from this voice that point at the planted defect.
   */
  const hits = row.findings
    .filter(function atOracle(finding,): boolean {
      return pointsAtOracle({ finding, },);
    },);

  /**
   Why this voice's claims fell, when any did.
   */
  const dropped = row.dropped
    .join(', ',);

  return [
    `  VOICE ${arm} ${row.modelId} verdict=${row.verdict} claims=${
      String(row.findings
        .length,)
    } oracleHits=${String(hits.length,)} dropped=${
      String(row.dropped
        .length,)
    }${(dropped === '') ? '' : ` [${dropped}]`}`,
    ...row.findings
      .map(function claimLine(finding,): string {
        return `    ${pointsAtOracle({ finding, },) ? 'ORACLE' : 'other '} ${finding.category}: ${
          shownText({ reading: finding.source, },)
        } || ${shownText({ reading: finding.candidate, },)}`;
      },),
  ];
}

/**
 Lines one arm leaves: its tally, each voice, and each agreement, near miss
 and degradation the audit reported.

 @param arm - label for the arm

 @param expectation - what a working instrument should conclude, printed only

 @param report - everything the audit returned

 @param oracleVoices - voices that pointed at the planted defect

 @param roster - auditors asked

 @returns The lines, without newlines, in the order they print

 @example
 ```ts
 for (const line of armLines({ arm, expectation, report, oracleVoices: 2, roster, },)) console.log(line,);
 ```
 */
export function armLines(
  {
    arm,
    expectation,
    report,
    oracleVoices,
    roster,
  }: {
    readonly arm: string;
    readonly expectation: string;
    readonly report: RenderingAuditReport;
    readonly oracleVoices: number;
    readonly roster: readonly string[];
  },
): readonly string[] {
  return [
    `SENSITIVITY arm=${arm} expected=${expectation} heard=${
      String(report.rows
        .length,)
    }/${
      String(roster
        .length,)
    } corroborated=${
      String(report.corroborated
        .length,)
    } agreed=${
      String(report.agreed
        .length,)
    } agreedVoices=${
      String(report.agreed
        .reduce(
          function widest(
            best: number,
            group,
          ): number {
            return Math.max(
              best,
              group.voices,
            );
          },
          0,
        ),)
    } near=${
      String(report.near
        .length,)
    } oracleVoices=${String(oracleVoices,)} findings=${
      String(report.findings
        .length,)
    }`,
    ...report.rows
      .flatMap(function toLines(row,): readonly string[] {
        return voiceLines({
          row,
          arm,
        },);
      },),
    ...report.corroborated
      .map(function corroboratedLine(defect,): string {
        return `  CORROBORATED ${arm} ${defect.category} voices=${String(defect.voices,)} at ${
          shownText({ reading: defect.source, },)
        } || ${shownText({ reading: defect.candidate, },)}`;
      },),
    ...report.agreed
      .map(function agreedLine(group,): string {
        return `  AGREED ${arm} ${group.category} voices=${String(group.voices,)} spans=${
          group.members
            .map(function toSpan(member,): string {
              return shownText({ reading: member.finding
                .source, },);
            },)
            .join(' / ',)
        }`;
      },),
    ...report.near
      .map(function nearLine(near,): string {
        return `  NEAR ${arm} ${near.kind}: ${
          near.left
            .finding
            .category
        } (${near.left
          .modelId}) against ${
          near.right
            .finding
            .category
        } (${near.right
          .modelId})`;
      },),
    ...report.findings
      .map(function degradedLine(finding,): string {
        return `  DEGRADED ${arm} ${finding}`;
      },),
  ];
}

/**
 Line saying how many arms were kept and where.

 @param count - arms kept

 @param keptAt - path the run was written to

 @returns The line, without a newline

 @example
 ```ts
 console.log(keptLine({ count: 2, keptAt: '/runs/audit-sensitivity/run.json', },),);
 ```
 */
export function keptLine(
  {
    count,
    keptAt,
  }: {
    readonly count: number;
    readonly keptAt: string;
  },
): string {
  return `SENSITIVITY kept ${String(count,)} ${
    wordForCount({
      count,
      one: 'arm',
      many: 'arms',
    },)
  } at ${keptAt}`;
}

//endregion Audit sensitivity print
