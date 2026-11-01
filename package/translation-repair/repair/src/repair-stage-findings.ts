import type { ChunkCriticPhase, } from './chunk-critic-phase.ts';

//region Repair stage findings
// The findings a chunk carries out of its critic, screen, panel and dedupe
// stages, folded in one place. Split out of `repair-chunk.ts` for the line
// budget when the claim-author record landed (owner, 2026-09-24).

/**
 Folds the findings of the stages that ran before the editor, naming the
 standing non-translation votes on the proceeding path.

 @param critic - critic phase outcome

 @param screened - the reference screen's findings

 @param panel - the panel's findings

 @param deduped - the dedupe's findings

 @returns Findings in stage order, the votes-stand line last

 @example
 ```ts
 const stageFindings = foldStageFindings({ critic, screened: screened.findings, panel: panel.findings, deduped: deduped.findings, },);
 ```
 */
export function foldStageFindings(
  {
    critic,
    screened,
    panel,
    deduped,
  }: {
    readonly critic: ChunkCriticPhase;
    readonly screened: readonly string[];
    readonly panel: readonly string[];
    readonly deduped: readonly string[];
  },
): readonly string[] {
  return [
    ...critic.findings,
    ...screened,
    ...panel,
    ...deduped,
    // NAMED ON THE PROCEEDING PATH, since the exit that used to name it is gone.
    // The wording says what now happens: the votes stood AND the slice was
    // repaired anyway. The old finding said "slice unchanged", which would be a
    // false statement about this path.
    ...(critic.votesStand
      ? [`non-translation votes stand (${
        String(critic.nonTranslationVotes,)
      }/${String(critic.heardCritics,)} heard); repaired anyway, votes are evidence`,]
      : []),
  ];
}

//endregion Repair stage findings
