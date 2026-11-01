import type { ConsistencyWinner, } from '@monochromatic-dev/translation-repair-consistency/ts';
import type { ArtifactConsistencyRunOff, } from '@monochromatic-dev/translation-repair-artifact/ts';

//region Pass consistency run-off line
// THE ONE LINE THAT REPORTS A RUN-OFF'S FINAL COUNT (the lead's ruling of 2026-10-15). The run of TianqiChen66625 logged that
// a run-off was asked, and every call of it, and never what the run-off came to: a reader had to open the artifact to learn
// that the choices tied on 2 of 4 heard, were retried, and settled on OCCURRENCE 3. The line says how many of the choices
// heard were usable, the quorum they were held to, and the winner or the reason for none.

/**
 Says what a run-off came to, in one line.

 @param runOff - the run-off as the artifact records it

 @param winner - the winner after the run-off

 @returns The line, without a tag

 @example
 ```ts
 const line = runOffResultLine({ runOff, winner: { kind: 'chosen', label: 'OCCURRENCE 3', }, },);
 // 'the run-off counted 4 usable of 4 heard (quorum 2): OCCURRENCE 3 won'
 ```
 */
export function runOffResultLine(
  {
    runOff,
    winner,
  }: {
    readonly runOff: ArtifactConsistencyRunOff;
    readonly winner: ConsistencyWinner;
  },
): string {
  return `the run-off counted ${String(runOff.usable,)} usable of ${String(runOff.ballots
    .length,)} heard (quorum ${String(runOff.quorum,)}): ${
    (winner.kind === 'chosen') ? `${winner.label} won` : `no winner (${winner.reason})`
  }`;
}

//endregion Pass consistency run-off line
