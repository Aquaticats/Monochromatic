import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  settleContestedTarget,
  type VotedPair,
} from './pair-contested-target.ts';

//region Pair agreement
// WHICH PAIRS THE ROSTER AGREED ON, counted over every usable voice.
//
// Both pairing stages once tallied votes over every reply but took their
// candidates from the FIRST usable reply alone, so a correspondence two other
// voices named was dropped whenever the first voice omitted it, and which
// pairs survived depended on which seat answered usably first. Agreement is
// per pair: a pair named by enough voices survives whoever named it.
//
// SECTION PAIRS ARE STRICTLY INCREASING. Block pairs are monotone and may stay
// on one side while the other advances, because paragraph splits and merges are
// first-class correspondences in `pair-blocks-wire.ts`. A repeated-side set is
// kept only when enough voices named its members together, so alternatives from
// disjoint replies do not become invented many-to-many structure.
//
// A TARGET TWO SOURCES CLAIM WITHOUT CORROBORATION IS DECIDED BY VOTES, in
// `pair-contested-target.ts`, the mirror of one source named against two
// targets. Deciding it by source order paired the first `noname` original's
// `## 简介` heading with the archive's opening paragraph and left the paragraph
// pair on the floor as "non-monotone".

/**
 A correspondence between one source position and one target position.

 @example
 ```ts
 const pair: IndexPair = { source: 3, target: 4, };
 ```
 */
export type IndexPair = {
  readonly source: number;
  readonly target: number;
};

/**
 Multiplicity pairing consumer can represent.

 @example
 ```ts
 const shape: PairingShape = 'many-to-many';
 ```
 */
export type PairingShape = 'one-to-one' | 'many-to-many';

/**
 Pairs the roster agreed on, with the findings for agreed pairs that could
 not be kept.

 @example
 ```ts
 const { pairs, findings, } = agreePairs({ pairings, needed: 2, },);
 ```
 */
export type PairAgreement<PairT extends IndexPair,> = {
  /**
   Agreed pairs, strictly increasing for one-to-one and monotone otherwise.
   */
  readonly pairs: readonly PairT[];

  /**
   One line per agreed pair that was dropped, naming why.
   */
  readonly findings: readonly string[];
};

/**
 Counts every distinct pair across every voice's pairing.

 A pairing names each pair once, which `readBlockPairing` and
 `readSectionPairing` hold by refusing a reply that names one twice, so a
 count of votes is a count of voices.

 @param pairings - one pairing per usable voice

 @returns Distinct pairs with their vote counts, in first-named order

 @example
 ```ts
 const voted = countAcrossVoices({ pairings, },);
 ```
 */
function countAcrossVoices<PairT extends IndexPair,>(
  { pairings, }: { readonly pairings: readonly (readonly PairT[])[]; },
): readonly VotedPair<PairT>[] {
  /**
   Votes per pair, keyed by both positions, the pair object kept from its
   first naming.
   */
  const byKey = new Map<string, VotedPair<PairT>>();
  for (const pairing of pairings)
    for (const pair of pairing) {
      /**
       Key naming both positions.
       */
      const key = `${String(pair.source,)},${String(pair.target,)}`;
      /**
       Count so far, absent on first naming.
       */
      const seen = byKey.get(key,);
      byKey.set(
        key,
        {
          pair,
          votes: (seen?.votes ?? 0) + 1,
        },
      );
    }
  return [...byKey.values(),];
}

/**
 Reports whether enough voices named every candidate together.

 @param candidates - pairs whose co-occurrence is being tested

 @param pairings - one pairing per usable voice

 @param needed - voices required

 @returns Whether candidates are corroborated as one relation

 @example
 ```ts
 const together = candidatesCoOccur({ candidates, pairings, needed: 2, });
 ```
 */
function candidatesCoOccur<PairT extends IndexPair,>(
  {
    candidates,
    pairings,
    needed,
  }: {
    readonly candidates: readonly VotedPair<PairT>[];
    readonly pairings: readonly (readonly PairT[])[];
    readonly needed: number;
  },
): boolean {
  /**
   Voices naming every candidate as one relation.
   */
  const matching = pairings.filter(function namesEveryCandidate(pairing,): boolean {
    return candidates.every(function namesCandidate(candidate,): boolean {
      /**
       Candidate pair this voice must name.
       */
      const { pair: candidatePair, } = candidate;
      return pairing.some(function isSamePair(pair,): boolean {
        return (pair.source === candidatePair.source)
          && (pair.target === candidatePair.target);
      },);
    },);
  },);
  return matching.length >= needed;
}

/**
 Of candidates naming one source, set to keep under consumer multiplicity.

 @param candidates - agreed pairs sharing source

 @param pairings - one pairing per usable voice

 @param needed - voices required

 @param pairingShape - multiplicity consumer can represent

 @returns Corroborated set, best-voted singleton, or nothing for tie

 @example
 ```ts
 const winner = bestVoted({ candidates, pairings, needed, pairingShape, },);
 ```
 */
function bestVoted<PairT extends IndexPair,>(
  {
    candidates,
    pairings,
    needed,
    pairingShape,
  }: {
    readonly candidates: readonly VotedPair<PairT>[];
    readonly pairings: readonly (readonly PairT[])[];
    readonly needed: number;
    readonly pairingShape: PairingShape;
  },
): readonly VotedPair<PairT>[] {
  if ((pairingShape === 'many-to-many')
    && candidatesCoOccur({
      candidates,
      pairings,
      needed,
    },))
    return candidates;
  /**
   Candidates from the most-voted down.
   */
  const ranked = candidates.toSorted(function byVotesDesc(
    left,
    right,
  ): number {
    return right.votes - left.votes;
  },);
  // AT LEAST ONE CANDIDATE: `agreePairs` asks about a source it drew from the
  // agreed pairs themselves, and hands over the agreed pairs naming it.
  /**
   The two best, the runner-up absent where one candidate stands alone.
   */
  const [top, second,] = ranked;
  /**
   Most-voted candidate.
   */
  const first = nonNullishOrThrow(top,);
  if ((second !== undefined) && (second.votes === first.votes))
    return [];
  return [first,];
}

/**
 Pairs enough voices named, kept strictly increasing on both sides.

 @param pairings - one pairing per usable voice

 @param needed - voices a pair needs to count as agreed

 @param pairingShape - multiplicity consumer can represent

 @returns Agreed pairs in source order, plus a finding per agreed pair dropped

 @example
 ```ts
 const agreement = agreePairs({ pairings, needed: 2, },);
 ```
 */
export function agreePairs<PairT extends IndexPair,>(
  {
    pairings,
    needed,
    pairingShape = 'one-to-one',
  }: {
    readonly pairings: readonly (readonly PairT[])[];
    readonly needed: number;
    readonly pairingShape?: PairingShape;
  },
): PairAgreement<PairT> {
  /**
   Agreed pairs in source, then target, order.
   */
  const agreed = countAcrossVoices({ pairings, },)
    .filter(function enoughAgree(voted,): boolean {
      return voted.votes >= needed;
    },)
    .toSorted(function bySourceThenTarget(
      left,
      right,
    ): number {
      /**
       Pair on the left of the comparison.
       */
      const { pair: before, } = left;
      /**
       Pair on the right of the comparison.
       */
      const { pair: after, } = right;
      return (before.source - after.source) || (before.target - after.target);
    },);

  /**
   Source of every agreed pair, in order, repeats included.
   */
  const named = agreed.map(function toSource(voted,): number {
    /**
     Pair this vote names.
     */
    const { pair, } = voted;
    return pair.source;
  },);

  /**
   Distinct sources in ascending order.
   */
  const sources = [...new Set(named,),];

  /**
   Pairs kept so far with their votes, monotone on both sides.
   */
  const kept: VotedPair<PairT>[] = [];

  /**
   Findings for agreed pairs that could not be kept.
   */
  const findings: string[] = [];

  for (const source of sources) {
    /**
     Agreed pairs naming this source.
     */
    const candidates = agreed.filter(function namesSource(voted,): boolean {
      /**
       Pair this vote names.
       */
      const { pair, } = voted;
      return pair.source === source;
    },);
    /**
     The one to keep for this source, absent on a tie.
     */
    const winners = bestVoted({
      candidates,
      pairings,
      needed,
      pairingShape,
    },);
    if (winners.length === 0) {
      findings.push(`contested (source ${String(source,)} named against ${String(candidates.length,)} ${
        wordForCount({
          count: candidates.length,
          one: 'target',
          many: 'targets',
        },)
      })`,);
      continue;
    }
    for (const winner of winners) {
      /**
       Last pair kept, absent before first.
       */
      const last = kept.at(-1,);
      if (last === undefined) {
        kept.push(winner,);
        continue;
      }
      /**
       Pair this winner names.
       */
      const { pair: chosen, } = winner;
      /**
       Pair kept last.
       */
      const { pair: previous, } = last;
      if (chosen.target < previous.target) {
        findings.push(
          `non-monotone (${String(source,)},${String(chosen.target,)} runs back behind ${String(previous.target,)})`,
        );
        continue;
      }
      if (chosen.target > previous.target) {
        kept.push(winner,);
        continue;
      }
      /**
       Whether the repeated target is a merge enough voices named together,
       which the block wire represents and the section wire does not.
       */
      const corroboratedMerge = (pairingShape === 'many-to-many')
        && candidatesCoOccur({
          candidates: [
            last,
            winner,
          ],
          pairings,
          needed,
        },);
      if (corroboratedMerge) {
        kept.push(winner,);
        continue;
      }
      /**
       Which of the two uncorroborated claims on this target survives.
       */
      const outcome = settleContestedTarget({
        earlier: last,
        later: winner,
      },);
      findings.push(outcome.finding,);
      if (outcome.keep === 'earlier')
        continue;
      kept.pop();
      if (outcome.keep === 'later')
        kept.push(winner,);
    }
  }
  return {
    pairs: kept.map(function toPair(voted,): PairT {
      /**
       Pair this vote names.
       */
      const { pair, } = voted;
      return pair;
    },),
    findings,
  };
}

//endregion Pair agreement
