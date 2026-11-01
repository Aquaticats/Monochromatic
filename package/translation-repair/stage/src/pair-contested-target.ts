import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Contested target
// ONE TARGET NAMED BY TWO SOURCES that no voice named together. The first
// `noname` pass of 2026-09-09 paired the original's `## 简介` heading with the
// archive's opening paragraph and dropped the paragraph-to-paragraph pair as
// "non-monotone", because agreement walked the sources in order and the later
// source's claim on the same target could only be a merge or a run-back. The
// heading then had no slice of its own, the paragraph became an insertion the
// page already carried, and the page shipped without the heading 84 of 86
// archives carry.
//
// A target two sources claim without corroboration is a CONTEST, the mirror of
// one source named against two targets, and `bestVoted` already decides that
// one by votes. So does this: the better-voted claim keeps the target, a tie
// keeps neither, and the finding says which.

/**
 A correspondence between one source position and one target position, as
 both pairing wires shape it.
 */
export type PositionPair = {
  readonly source: number;
  readonly target: number;
};

/**
 One pair with the number of voices that named it.

 @example
 ```ts
 const voted: VotedPair<PositionPair> = { pair: { source: 1, target: 0, }, votes: 3, };
 ```
 */
export type VotedPair<PairT extends PositionPair,> = {
  readonly pair: PairT;
  readonly votes: number;
};

/**
 Which claim on a contested target survives.

 @example
 ```ts
 const outcome: ContestedTargetOutcome = { keep: 'later', finding: 'contested target (...)', };
 ```
 */
export type ContestedTargetOutcome = {
  /**
   `earlier` keeps the pair already kept, `later` replaces it with the new
   claim, `neither` drops both.
   */
  readonly keep: 'earlier' | 'later' | 'neither';

  /**
   One line naming the target, both sources and the votes.
   */
  readonly finding: string;
};

/**
 Decides by votes between two sources' uncorroborated claims on one target.

 @param earlier - claim already kept, from the lower source

 @param later - claim from the higher source on the same target

 @returns Which claim to keep, with the finding that says why

 @example
 ```ts
 const outcome = settleContestedTarget({ earlier, later, },);
 ```
 */
export function settleContestedTarget<PairT extends PositionPair,>(
  {
    earlier,
    later,
  }: {
    readonly earlier: VotedPair<PairT>;
    readonly later: VotedPair<PairT>;
  },
): ContestedTargetOutcome {
  /**
   Pair already kept.
   */
  const { pair: kept, } = earlier;

  /**
   Pair newly claiming the same target.
   */
  const { pair: claim, } = later;

  /**
   Target both claims name.
   */
  const target = String(claim.target,);

  /**
   Source of the claim already kept.
   */
  const earlierSource = String(kept.source,);

  /**
   Source of the new claim.
   */
  const laterSource = String(claim.source,);

  if (later.votes > earlier.votes)
    return {
      keep: 'later',
      finding: `contested target (target ${target}: source ${laterSource} outvotes source ${earlierSource}, ${
        String(later.votes,)
      } to ${String(earlier.votes,)})`,
    };
  if (later.votes < earlier.votes)
    return {
      keep: 'earlier',
      finding: `contested target (target ${target}: source ${laterSource} loses to source ${earlierSource}, ${
        String(later.votes,)
      } to ${String(earlier.votes,)})`,
    };
  return {
    keep: 'neither',
    finding: `contested target (target ${target}: sources ${earlierSource} and ${laterSource} tie at ${
      String(later.votes,)
    } ${
      wordForCount({
        count: later.votes,
        one: 'vote',
        many: 'votes',
      },)
    })`,
  };
}

//endregion Contested target
