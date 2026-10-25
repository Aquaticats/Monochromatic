import { rosterQuorumSize, } from './roster-quorum-size.ts';

//region Reachable quorum
// A QUORUM IS SIZED ON THE SEATS THAT COULD ANSWER, not on the seats the phase
// seated. The owner's decision of 2026-09-09 (a deciding bench short of quorum
// decides by a share of the reachable weight, with a two-ballot floor) reached
// the select bench first (`candidate-select-bench.ts`); this is the same
// rule for the gather itself, so a stage whose bench lost seats to a provider
// drying mid-phase closes on the seats a wet provider serves instead of
// chasing a quorum those seats cannot reach.
//
// WHY IT EXISTS. On 2026-09-09 at 20:23 UTC the seat reader read Synthetic
// wet at 0.04 percent of its week and seated Qwen3.8-27B and Kimi-K3; the
// provider refused at 20:25 UTC, the router named those two and the dark
// `glm-5.3` unreachable, and the archive block review of `hulicaijia` sized
// its quorum at 6 of 11, heard 5, spent its retry rounds re-asking the three
// refused seats, and interrupted the entry provider-unavailable at 382 s.
//
// THE FLOOR IS TWO VOICES WHEN THE BENCH IS SHORT, as the select minimum's is
// two ballots: a short bench is not closed on a single model, and one with
// one reachable seat, or none, is an outage rather than a quorum. A bench
// that is not short closes on its own quorum, which is one voice for a seated
// bench of one or two seats (after the run withholds seats, ledger B420);
// the floor does not raise that.
//
// A BENCH LEFT WITH EXACTLY ITS QUORUM CLOSES ONE VOICE BELOW IT. Under the
// rule of 2026-09-09 such a bench needed every live seat, so the straggler
// grace never opened and one stream that held without content held the whole
// round (ledger B809: seven seats, three dark, four needed of four). The
// owner answered "Let the grace open" on 2026-10-22: the bench is marked
// short and closes on one fewer, still never below the two-voice floor, so
// three seats with one dark still need both live seats.

/**
 Fewest heard voices a short bench may close on.
 */
export const MIN_STAGE_VOICES = 2;

/**
 Quorum a gather needs once the seats no provider serves are known.

 @example
 ```ts
 const quorum: ReachableQuorum = reachableQuorum({ benchSize: 11, unreachable: 7, },);
 ```
 */
export type ReachableQuorum = {
  /**
   Heard voices the gather needs to close.
   */
  readonly needed: number;

  /**
   Seats a wet provider could serve.
   */
  readonly reachable: number;

  /**
   Quorum the seated bench would have needed with every seat reachable.
   */
  readonly benchQuorum: number;

  /**
   Whether the reachable bench is short of that quorum, or holds exactly it
   and so closes one voice below it; the only case in which `needed` differs
   from `benchQuorum`.
   */
  readonly short: boolean;
};

/**
 Sizes a gather's quorum to the seats that could answer.

 @param benchSize - seats the stage seated, unreachable seats included

 @param unreachable - seats the router refused for want of a wet provider

 @param lastSeatGrace - whether a bench left with exactly its quorum closes
 one voice below it, the rule since 2026-10-22; false reads a record written
 under the rule of 2026-09-09, which waited on every live seat, as an
 artifact of generation 19 or older was

 @returns Voices needed, the reachable count and bench quorum it was sized
 by, and whether the bench was short

 @example
 ```ts
 reachableQuorum({ benchSize: 11, unreachable: 7, },);
 // => { needed: 2, reachable: 4, benchQuorum: 6, short: true, }
 ```
 */
export function reachableQuorum(
  {
    benchSize,
    unreachable,
    lastSeatGrace = true,
  }: {
    readonly benchSize: number;
    readonly unreachable: number;
    readonly lastSeatGrace?: boolean;
  },
): ReachableQuorum {
  /**
   Seats a wet provider could serve.
   */
  const reachable = benchSize - unreachable;

  /**
   Voices the seated bench needs for a quorum.
   */
  const benchQuorum = rosterQuorumSize({ rosterSize: benchSize, },);
  if (
    lastSeatGrace
      && (reachable === benchQuorum)
      && (benchQuorum > MIN_STAGE_VOICES)
  ) {
    return {
      needed: benchQuorum - 1,
      reachable,
      benchQuorum,
      short: true,
    };
  }
  if (reachable >= benchQuorum) {
    return {
      needed: benchQuorum,
      reachable,
      benchQuorum,
      short: false,
    };
  }
  return {
    needed: Math.max(
      MIN_STAGE_VOICES,
      rosterQuorumSize({ rosterSize: reachable, },),
    ),
    reachable,
    benchQuorum,
    short: true,
  };
}

/**
 Finding a gather closed short of its bench quorum carries, so an artifact
 decided on a short bench is told apart in its findings rather than only in
 a log line.

 @param stage - stage the gather served

 @param quorum - quorum the gather applied

 @param benchSize - seats the stage seated

 @returns Finding in scorecard-stable wording

 @example
 ```ts
 shortBenchStageFinding({ stage: 'critic', quorum, benchSize: 11, },);
 // => 'stage-short-bench (critic reachable 4 of 11, quorum 2)'
 ```
 */
export function shortBenchStageFinding(
  {
    stage,
    quorum,
    benchSize,
  }: {
    readonly stage: string;
    readonly quorum: ReachableQuorum;
    readonly benchSize: number;
  },
): string {
  return `stage-short-bench (${stage} reachable ${String(quorum.reachable,)} of ${String(benchSize,)}, quorum ${
    String(quorum.needed,)
  })`;
}

/**
 Finding a stage carries when it made no call because the seats a wet provider
 would serve were fewer than the voices it needs, told apart from a gather
 that closed on a thin bench.

 @param stage - stage that was not asked

 @param reachable - seats a wet provider would serve when the stage was read

 @param benchSize - seats the stage seated

 @param floor - voices the stage needs to settle anything

 @returns Finding in scorecard-stable wording

 @example
 ```ts
 benchBelowFloorStageFinding({ stage: 'lane-contest', reachable: 1, benchSize: 7, floor: 2, },);
 // => 'stage-bench-below-floor (lane-contest reachable 1 of 7, floor 2, no call made)'
 ```
 */
export function benchBelowFloorStageFinding(
  {
    stage,
    reachable,
    benchSize,
    floor,
  }: {
    readonly stage: string;
    readonly reachable: number;
    readonly benchSize: number;
    readonly floor: number;
  },
): string {
  return `stage-bench-below-floor (${stage} reachable ${String(reachable,)} of ${String(benchSize,)}, floor ${
    String(floor,)
  }, no call made)`;
}
//endregion Reachable quorum
