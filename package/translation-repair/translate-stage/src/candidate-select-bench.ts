import { rosterQuorumSize, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Selection bench
// THE OWNER'S RULINGS OF 2026-10-25, which removed every weight floor from the pipeline: "floor of 3 - don't set a
// floor." (about the panel tally) and "Every weight floor" (the scope). A selection round used to need a summed weight
// of two, scaled by reachable over quorum on a bench short of quorum (the owner's decision of 2026-09-09,
// `translation-repair-short-bench-share.md`); a leader drawing 1.5 from two ballots, a full vote and a self-vote, was
// declined on a bench at quorum, and the machinery that let a run-off or a condemned fallback win under that weight
// (`condemnedFallbackLoses`, `select-runoff-under-minimum`, `select-condemned-fallback-under-minimum`) existed only to
// get around it. What is left is the older standing rule of the owner, which the ruling "Two ballots decide
// (Recommended)" restates: a leader that two or more answering judges named wins, and one judge alone never decides
// (2026-09-09). A self-voting judge is one answering judge; its half weight only decides which candidate leads.
//
// A BENCH SHORT OF QUORUM IS STILL SAID IN THE FINDINGS. The reachable bench is the seats a wet provider serves, and
// `selectionBench` still reads it, because a round decided on three of eleven seats is told apart from one decided on
// a full bench in the artifact; it no longer sizes any threshold.

/**
 The reachable bench a selection round was seated on, read for its findings.

 @example
 ```ts
 const bench: SelectionBench = { reachable: 3, quorum: 6, short: true, };
 ```
 */
export type SelectionBench = {
  /**
   Seats a wet provider could serve: the bench less the seats the router refused.
   */
  readonly reachable: number;

  /**
   Voices the seated bench needs for a quorum.
   */
  readonly quorum: number;

  /**
   Whether the reachable bench is short of that quorum.
   */
  readonly short: boolean;
};

/**
 Reads the seats that could answer.

 @param benchSize - judges the round seated, unreachable seats included

 @param unreachable - seats the router refused for want of a wet provider

 @returns Reachable count, quorum of the seated bench, and whether the bench was short

 @example
 ```ts
 selectionBench({ benchSize: 11, unreachable: 8, },);
 // => { reachable: 3, quorum: 6, short: true, }
 ```
 */
export function selectionBench(
  {
    benchSize,
    unreachable,
  }: {
    readonly benchSize: number;
    readonly unreachable: number;
  },
): SelectionBench {
  /**
   Seats a wet provider could serve.
   */
  const reachable = benchSize - unreachable;

  /**
   Voices the seated bench needs for a quorum.
   */
  const quorum = rosterQuorumSize({ rosterSize: benchSize, },);
  return {
    reachable,
    quorum,
    short: reachable < quorum,
  };
}

/**
 Finding a round seated short of quorum carries, so a page decided this way is told apart in its findings rather than
 only in a log line.

 @param bench - bench the round applied

 @param benchSize - judges the round seated

 @returns Finding in scorecard-stable wording

 @example
 ```ts
 shortBenchFinding({ bench, benchSize: 11, },);
 // => 'select-short-bench (reachable 3 of 11, quorum 6)'
 ```
 */
export function shortBenchFinding(
  {
    bench,
    benchSize,
  }: {
    readonly bench: SelectionBench;
    readonly benchSize: number;
  },
): string {
  return `select-short-bench (reachable ${String(bench.reachable,)} of ${String(benchSize,)}, quorum ${
    String(bench.quorum,)
  })`;
}

//endregion Selection bench
