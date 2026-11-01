import type { SelectionRound, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type { BenchCall, } from './bench-record.ts';

//region Roster bench row
// ONE SLICE RUN AT ONE WIDTH, as the bench records it and the report reads it.
//
// SPLIT OUT OF `roster-bench.ts` when that file became wiring only: an entry
// module may not declare anything a barrel exports, since rolldown folds the
// export into a shared chunk and `import.meta.main` reads false in the built
// command.

/**
 One slice run at one width, with everything the report reads.

 @example
 ```ts
 const row: BenchRow = { width: 3, entryId: 'Mittens', ... };
 ```

 @internal
 */
export type BenchRow = {
  /**
   Producers seated for this run.
   */
  readonly width: number;

  /**
   Which pass over this width, so a repeat is distinguishable.
   */
  readonly pass: number;

  /**
   Entry the slice came from.
   */
  readonly entryId: string;

  /**
   Slice position within that entry.
   */
  readonly index: number;

  /**
   Source characters, since decline rates may well move with size.
   */
  readonly sourceChars: number;

  /**
   Incumbent characters, zero when the archive has no translation here.
   */
  readonly incumbentChars: number;

  /**
   Exact seats, so a report can say who width 3 was.
   */
  readonly translators: readonly string[];

  /**
   How the round ended.
   */
  readonly decision: string;

  /**
   Whether the text that shipped was the one already there.
   */
  readonly keptIncumbent: boolean;

  /**
   Weight the winner drew.
   */
  readonly voteWeight: number;

  /**
   Judges seated, ballots cast, abstentions and self-votes.
   */
  readonly judgesAvailable: number;

  /**
   Ballots that arrived.
   */
  readonly ballots: number;

  /**
   Ballots naming no usable candidate.
   */
  readonly abstentions: number;

  /**
   Ballots a judge cast for its own work.
   */
  readonly selfVotes: number;

  /**
   This round as {@link selfPreference} needs to read it: who wrote each
   candidate, and every ballot cast over that slate.

   KEPT RATHER THAN COUNTED, because `selfVotes` cannot answer the
   question it looks like it answers. How often a producer backs its own work
   says nothing alone: a model whose translations are better would do that
   without any favouritism. The paired comparison needs to know what judges
   holding NO stake in the same candidate thought of it, and that needs the
   slate and the ballots rather than a sum.

   This is the cheaper of the two routes to that number. The other is the
   per-slice selection field in the settled artifact, which answers it
   corpus-wide instead of on a bench.
   */
  readonly round: SelectionRound;

  /**
   Distinct proposals the judges saw.
   */
  readonly candidateCount: number;

  /**
   Translators heard out of those seated.
   */
  readonly heardTranslators: number;

  /**
   Everything the stage recorded, verbatim.
   */
  readonly findings: readonly string[];

  /**
   Exchanges this row cost.
   */
  readonly calls: readonly BenchCall[];

  /**
   Time of the whole stage call, on `performance.now()`.
   */
  readonly ms: number;
};

//endregion Roster bench row
