import { admitWithinShortfall, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  interiorShortfall,
  type UntranslatedTail,
} from '@monochromatic-dev/translation-repair-archive/ts';
import type { CarriedInsertion, } from '@monochromatic-dev/translation-repair-corpus-source/ts';

//region Insertion coverage model

/**
 One source-only slice proposed by preparation.
 */
export type InsertionCandidate = {
  /**
   Position in prepared slice order.
   */
  readonly position: number;
  /**
   Stable slice index recorded in artifacts.
   */
  readonly sliceIndex: number;
  /**
   Original passage proposed for insertion.
   */
  readonly sourceText: string;
  /**
   Whether syntax-bearing metadata requires insertion.
   */
  readonly frontMatter: boolean;
};

/**
 Source-only slice beside latest coverage and deterministic evidence.
 */
export type InsertionCoverageRow = InsertionCandidate & {
  /**
   Latest semantic coverage outcome.
   */
  readonly verdictKind: 'carried' | 'partly-carried' | 'absent' | 'split' | 'inconclusive';
  /**
   Full coverage votes anchored to target.
   */
  readonly anchoredFull: number;
  /**
   Partial coverage votes anchored to target.
   */
  readonly anchoredPartial: number;
  /**
   Votes reporting passage absent.
   */
  readonly absentCount: number;
  /**
   Voices heard in latest pass.
   */
  readonly heard: number;
  /**
   Models asked in latest pass.
   */
  readonly asked: number;
  /**
   Source destinations absent from target.
   */
  readonly missingDestinationCount: number;
  /**
   Latest count-only coverage finding.
   */
  readonly coverageFinding: string;
  /**
   Accumulated stage findings from coverage rounds.
   */
  readonly stageFindings: readonly string[];
  /**
   Exact target regions supporting anchored coverage votes.
   */
  readonly coverageEvidence: readonly string[];
  /**
   Parser downgrade findings from destination comparison.
   */
  readonly destinationFindings: readonly string[];
};

/**
 Current resolution of all insertion coverage rows.
 */
export type InsertionCoverageClassification = {
  /**
   Positions licensed for local insertion.
   */
  readonly positions: ReadonlySet<number>;
  /**
   Passages proven fully rendered elsewhere.
   */
  readonly carried: readonly CarriedInsertion[];
  /**
   Rows requiring another distinct placement task.
   */
  readonly unresolvedRows: readonly InsertionCoverageRow[];
  /**
   Positions admitted by whole-page shortfall.
   */
  readonly shortfallAdmitted: ReadonlySet<number>;
  /**
   Count-only findings for current state and prior rounds.
   */
  readonly findings: readonly string[];
};

/**
 Classifies latest rows as inserted, carried, or unresolved.

 @param candidates - every source-only slice

 @param rows - latest semantic coverage evidence for prose candidates

 @param frontMatterPositions - metadata positions admitted deterministically

 @param sourceText - whole original for shortfall calculation

 @param targetText - whole target for shortfall calculation

 @param tail - untranslated tail read off the pairing, admitted on its own
 budget (owner, 2026-09-19)

 @returns Current insertion resolution and findings

 @example
 ```ts
 const state = classifyInsertionCoverage({
   candidates: [],
   rows: [],
   frontMatterPositions: new Set(),
   sourceText: '',
   targetText: '',
   tail: readUntranslatedTail({ slices: [], }),
 });
 ```
 */
export function classifyInsertionCoverage(
  {
    candidates,
    rows,
    frontMatterPositions,
    sourceText,
    targetText,
    tail,
  }: {
    readonly candidates: readonly InsertionCandidate[];
    readonly rows: readonly InsertionCoverageRow[];
    readonly frontMatterPositions: ReadonlySet<number>;
    readonly sourceText: string;
    readonly targetText: string;
    readonly tail: UntranslatedTail;
  },
): InsertionCoverageClassification {
  /**
   Rows latest roster found wholly absent, and rows whose split carried no
   anchored claim of coverage at all.

   A SPLIT NOBODY ANCHORED IS ABSENCE SHORT OF A MAJORITY (class forty-eight,
   shi_Yumiaoya2, 2026-09-17): two seats asked on a skeleton archive, one
   voted absent, one claimed partial coverage and quoted the Chinese passage
   itself, which anchors nowhere on the page; the passage shipped as a silent
   gap. An unanchorable claim is still no vote for absence, but where the only
   votes are absent ones, the whole-page shortfall corroboration is what
   decides, as it does for a majority.
   */
  const absent = rows.filter(function absentVerdict(row,): boolean {
    if (row.verdictKind === 'absent')
      return true;
    return (row.verdictKind === 'split')
      && (row.anchoredFull === 0)
      && (row.anchoredPartial === 0)
      && (row.absentCount > 0);
  },);
  /**
   Rows proven fully rendered elsewhere.
   */
  const carried = rows
    .filter(function carriedVerdict(row,): boolean {
      return row.verdictKind === 'carried';
    },)
    .map(function carriedInsertion(row,): CarriedInsertion {
      return {
        position: row.position,
        sliceIndex: row.sliceIndex,
        sourceText: row.sourceText,
        evidence: row.coverageEvidence,
      };
    },);
  /**
   Passages standing in an untranslated tail too large for the last agreed
   pair to have absorbed, admitted on that bound (owner, 2026-09-19;
   XingZ608). THE BOUND DECIDES A SPLIT TOO (class sixty, XingZ610,
   2026-09-19): the rule first admitted absent verdicts alone, and three
   tail slices, two paragraphs and a footnote definition split on every
   run, stayed unresolved on one voice quoting an archive line the pairing
   never assigned, and shipped as gaps. A minority anchored claim is no
   majority; where a majority found the passage carried, the row stays out.
   */
  const tailAdmitted = new Set(rows
    .filter(function inTail(row,): boolean {
      /**
       Whether this row stands in the tail.
       */
      const inTailRun = tail.positions
        .has(row.position,);
      /**
       Whether no majority found the passage carried.
       */
      const noMajorityCarried = (row.verdictKind === 'absent') || (row.verdictKind === 'split');
      return tail.exceedsLastPair
        && inTailRun
        && noMajorityCarried;
    },)
    .map(function toPosition(row,): number {
      return row.position;
    },),);
  /**
   Absent passages requiring whole-page shortfall corroboration.
   */
  const shortfallPassages = absent
    .filter(function needsShortfall(row,): boolean {
      return (row.missingDestinationCount === 0) && (!tailAdmitted.has(row.position,));
    },)
    .map(function toPassage(row,) {
      return {
        position: row.position,
        sourceText: row.sourceText,
      };
    },);
  /**
   Positions admitted by remaining whole-page shortfall budget.
   */
  const shortfallAdmitted = new Set(admitWithinShortfall({
    sourceText,
    targetText,
    passages: shortfallPassages,
    shortfall: interiorShortfall({
      sourceText,
      targetText,
      tail,
    },),
  },),);
  /**
   Positions backed by semantic absence and independent signal.
   */
  const positions = new Set([
    ...frontMatterPositions,
    ...rows
      .filter(function corroborated(row,): boolean {
        if (tailAdmitted.has(row.position,))
          return true;
        return absent.includes(row,)
          && ((row.missingDestinationCount > 0) || shortfallAdmitted.has(row.position,));
      },)
      .map(function toPosition(row,): number {
        return row.position;
      },),
  ],);
  /**
   Positions proven carried elsewhere.
   */
  const carriedPositions = new Set(carried.map(function carriedPosition(candidate,): number {
    return candidate.position;
  },),);
  /**
   Rows neither admitted nor proven fully carried elsewhere.
   */
  const unresolvedRows = rows.filter(function wasNotResolved(row,): boolean {
    return (!positions.has(row.position,)) && (!carriedPositions.has(row.position,));
  },);
  /**
   Count-only findings for every candidate and coverage round.
   */
  const findings = [
    ...candidates
      .filter(function isFrontMatter(candidate,): boolean {
        return candidate.frontMatter;
      },)
      .map(function metadataFinding(candidate,): string {
        return `insertion-front-matter-admitted (slice ${String(candidate.sliceIndex,)})`;
      },),
    ...rows.flatMap(function evidence(row,): readonly string[] {
      /**
       Whether the tail bound admitted this row over a minority anchored claim.
       */
      const splitInTail = (row.verdictKind === 'split')
        && tailAdmitted.has(row.position,)
        && (!absent.includes(row,));
      return [
        row.coverageFinding,
        ...((row.verdictKind === 'split') && absent.includes(row,)
          ? [`insertion-split-unanchored (slice ${String(row.sliceIndex,)}, absent ${String(row.absentCount,)} of ${
            String(row.asked,)
          } asked, no anchored claim; read as absent for the shortfall)`,]
          : []),
        ...(splitInTail
          ? [`insertion-split-in-tail (slice ${String(row.sliceIndex,)}, full ${String(row.anchoredFull,)}, partial ${
            String(row.anchoredPartial,)
          }, absent ${String(row.absentCount,)} of ${String(row.asked,)} asked; no majority, the tail bound decides)`,]
          : []),
        `insertion-corroboration (slice ${String(row.sliceIndex,)}, ${
          tailAdmitted.has(row.position,)
            ? 'tail admitted'
            : `shortfall ${shortfallAdmitted.has(row.position,) ? 'admitted' : 'refused'}`
        }, missing destinations ${String(row.missingDestinationCount,)}, admission ${
          positions.has(row.position,) ? 'admitted' : 'refused'
        })`,
        ...row.stageFindings,
        ...row.destinationFindings,
      ];
    },),
  ];
  return {
    positions,
    carried,
    unresolvedRows,
    shortfallAdmitted,
    findings,
  };
}

//endregion Insertion coverage model
