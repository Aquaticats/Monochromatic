import type { Logger, } from '@monochromatic-dev/logger-core/ts';

//region Ballot irregularity log
// Ledger L12: the panel and checker stages fold each ballot's irregularities
// (a duplicated verdict, a number outside the sheet, an unknown vote) into
// their findings, so the artifact records them, and logged none; the audit
// counted 10, 14 and 26 on three runs' artifacts and none in their logs. The
// finding strings name no model, so this line is the one place that says whose
// ballot it was.

/**
 Logs every irregularity a stage found in its resolved ballots, one line per
 finding, naming the stage and the model that cast the ballot.

 @param ballots - resolved ballots keyed by the model that cast each

 @param stage - stage the ballots were cast in, as its lines name it

 @param l - stage logger, which carries the slice

 @example
 ```ts
 logBallotIrregularities({ ballots, stage: 'checker', l, },);
 ```
 */
export function logBallotIrregularities(
  {
    ballots,
    stage,
    l,
  }: {
    readonly ballots: Readonly<Record<string, { readonly findings: readonly string[]; }>>;
    readonly stage: string;
    readonly l: Logger;
  },
): void {
  for (const [modelId, ballot,] of Object.entries(ballots,)) {
    for (const finding of ballot.findings)
      l.warn(`${stage} ballot from ${modelId} irregular: ${finding}`,);
  }
}

//endregion Ballot irregularity log
