//region Stage round line
// THE TEXT A ROUND LOGS ABOUT ITSELF, pulled out of `stage-round.ts` so it is
// callable on its own: the same reason `reportStreamProgress` in
// `stream-cut.ts` returns the line it logs rather than only emitting it.
// Before this split, a reader's own test had no way to build this exact text
// except retyping the template by hand, which is what let a fixture in
// `corpus-run/run-timing.unit.test.ts` drift into a tag that no caller of
// `runGatherRound` has ever written (ledger B82).

/**
 Time either side of the instant a round's quorum stood, or the count it
 needed where it never did (ledger P12).

 TWO KINDS RATHER THAN ZEROS. A round whose quorum never stood measured no
 time to quorum and spent no grace, and writing zeros for them would report a
 measurement the round never made. A round that never reaches quorum stops
 waiting once nothing is pending, so its "quorum" mark is only when the last
 ask settled.

 @example
 ```ts
 const quorum: RoundLineQuorum = { kind: 'never', needed: 4, };
 ```
 */
export type RoundLineQuorum =
  | {
    readonly kind: 'stood';

    /**
     Time before quorum stood, which is the round doing its work.
     */
    readonly toQuorumMs: number;

    /**
     Time after quorum stood, which is the round waiting on voices it may
     never hear.
     */
    readonly inGraceMs: number;
  }
    | {
      readonly kind: 'never';

    /**
     Voices the round needed and never heard enough of.
     */
      readonly needed: number;
    };

/**
 Builds the line a round logs about itself once it closes: the ratio heard,
 the time the whole round took, and the time either side of quorum where it
 stood, or the count it needed where it never did.

 @param stage - stage label this round ran

 @param heard - voices this round actually heard

 @param asked - models this round asked

 @param totalMs - time the whole round took

 @param quorum - time either side of the instant quorum stood, or the count
 needed where it never did

 @returns The line `runGatherRound` logs

 @example
 ```ts
 const line = roundLine({
   stage: 'editor',
   heard: 6,
   asked: 7,
   totalMs: 91_402,
   quorum: { kind: 'stood', toQuorumMs: 61_401, inGraceMs: 30_001, },
 },);
 ```
 */
export function roundLine(
  {
    stage,
    heard,
    asked,
    totalMs,
    quorum,
  }: {
    readonly stage: string;
    readonly heard: number;
    readonly asked: number;
    readonly totalMs: number;
    readonly quorum: RoundLineQuorum;
  },
): string {
  /**
   Time either side of quorum where it stood, or the count the round needed
   where it never did.
   */
  const timing = (quorum.kind === 'stood')
    ? `${String(quorum.toQuorumMs,)}ms to quorum, ${String(quorum.inGraceMs,)}ms in grace`
    : `no quorum (${String(heard,)} of ${String(quorum.needed,)} needed), every ask settled`;

  return `${stage} round: ${String(heard,)}/${String(asked,)} heard, `
    + `${String(totalMs,)}ms total, ${timing}`;
}

//endregion Stage round line
