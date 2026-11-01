import type { LaneContestBallot, } from '@monochromatic-dev/translation-repair-lane-contest/ts';
import { rendersAsNothing, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Consolidation brief
// Renders what the lane contest's judges said about the two candidates, as the
// block a consolidating producer is shown.
//
// FINDINGS ARE CLAIMS, NOT FACTS. These are judge output, and judge output is
// exactly what `M3 fix D` had to teach the critic policy not to treat as
// golden. A producer that obeys a false finding introduces a defect this
// pipeline authored itself, which is worse than the one it was sent to fix. So
// the brief presents each judge's reading as something to check against the
// original, and the sheet says so in as many words.
//
// ONE BLOCK PER JUDGE rather than two pooled lists. A judge that names a
// candidate and nothing else, `unsupported: ["repair"]`, has put the substance
// of its finding in its reason, and pooling the lists separates the two. Every
// bare name would arrive detached from what it was about.
//
// JUDGES ARE NUMBERED, NOT NAMED. Which model said what is not evidence about
// the passage, and naming them invites a producer to weigh a finding by its
// author. `self-preference.ts` exists because that weighing is measurable.

/**
 Blank findings a judge may return, dropped before rendering.

 @param findings - findings as a judge wrote them

 @returns Same findings, trimmed, without the ones showing a reader nothing,
 invisible characters `trim()` keeps among them (ledger B40)

 @example
 ```ts
 const kept = usableFindings({ findings: [ ' repair ', '', ], },);
 ```
 */
function usableFindings(
  { findings, }: { readonly findings: readonly string[]; },
): readonly string[] {
  return findings
    .map(function trimOne(finding,): string {
      return finding.trim();
    },)
    .filter(function saysSomething(finding,): boolean {
      return !rendersAsNothing({ text: finding, },);
    },);
}

/**
 Renders one list of findings under its heading, or nothing.

 @param heading - what this list of findings is about

 @param findings - findings as that judge wrote them

 @returns Lines for this list, empty when the judge listed nothing

 @example
 ```ts
 const lines = renderFindingList({ heading: 'Unsupported', findings, },);
 ```
 */
function renderFindingList(
  {
    heading,
    findings,
  }: {
    readonly heading: string;
    readonly findings: readonly string[];
  },
): readonly string[] {
  /**
   Findings that say something.
   */
  const kept = usableFindings({ findings, },);
  if (kept.length === 0)
    return [];
  return [
    `  ${heading}:`,
    ...kept.map(function asItem(finding,): string {
      return `    - ${finding}`;
    },),
  ];
}

/**
 Renders one judge's reading of the two candidates.

 @param ballot - that judge's ballot

 @param position - which judge this is, counted from one

 @returns Lines for this judge

 @example
 ```ts
 const lines = renderBallot({ ballot, position: 1, },);
 ```
 */
function renderBallot(
  {
    ballot,
    position,
  }: {
    readonly ballot: LaneContestBallot;
    readonly position: number;
  },
): readonly string[] {
  /**
   That judge's own words about why, when it gave any.
   */
  const reason = ballot.reason
    .trim();
  return [
    `Judge ${String(position,)} would publish: ${ballot.choice}`,
    ...renderFindingList({
      heading: 'Says what the original does not',
      findings: ballot.unsupportedRaw,
    },),
    ...renderFindingList({
      heading: 'Omits what the original says',
      findings: ballot.droppedRaw,
    },),
    ...(rendersAsNothing({ text: reason, },)
      ? []
      : [ `  Why: ${reason}`, ]),
  ];
}

/**
 Renders every judge's reading as the brief a producer is shown.

 @param ballots - usable ballots from the lane contest for this slice

 @returns Brief, empty when no judge was heard

 @example
 ```ts
 const brief = renderConsolidationBrief({ ballots, },);
 ```
 */
export function renderConsolidationBrief(
  { ballots, }: { readonly ballots: readonly LaneContestBallot[]; },
): string {
  if (ballots.length === 0)
    return '';
  return ballots
    .flatMap(function renderOne(
      ballot,
      index,
    ): readonly string[] {
      return renderBallot({
        ballot,
        position: index + 1,
      },);
    },)
    .join('\n',);
}

//endregion Consolidation brief
