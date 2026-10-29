import { assertReplacementsChange, } from './assembly-invariant.ts';
import {
  attributeFinding,
  everyFindingSuspected,
  footnoteRoundFindings,
} from './assembly-withdrawal-proof.ts';
import {
  advancingStructuralWithdrawal,
  singleStructuralWithdrawal,
} from './assembly-structural-withdrawal.ts';
import {
  definitionBlockCount,
  trimOrphanDefinitions,
} from './assembly-orphan-trim.ts';
import {
  introducedFootnoteFindings,
  introducedStructuralRegressions,
  UNREADABLE_PAGE,
} from './assembly-regressions.ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  type SliceReplacement,
  spliceSlices,
} from '@monochromatic-dev/translation-repair-page-pass/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';

export {
  introducedFootnoteFindings,
  introducedStructuralRegressions,
} from './assembly-regressions.ts';

//region Assembly integrity
// The check no per-slice decision can make: a footnote is a relation BETWEEN
// slices. One slice carries `body text[^1]` and another carries `[^1]: the
// note`, and selection settles each on its own, so a candidate that drops,
// renames or invents a marker validates perfectly inside its slice and breaks
// the document.
//
// The ASSEMBLED DOCUMENT is the authority on what is broken. It is parsed the
// same way any document is, and its footnote findings are diffed against the
// incumbent's, so a defect the archive already carried is never blamed on the
// lane and never repaired by it either.
//
// Attribution, which decides WHICH replacement to withdraw, is a different
// reading and lives in `footnote-mentions.ts`.

/**
 What the guard settled on for one document.

 @example
 ```ts
 const guarded: GuardedAssembly = { assembledText, replacements, revertedChunkIndices: [], findings: [], };
 ```
 */
export type GuardedAssembly = {
  /**
   Document as it now stands, spliced from the replacements that survived.
   */
  readonly assembledText: string;

  /**
   Replacements that survived, in the order they were given.
   */
  readonly replacements: readonly SliceReplacement[];

  /**
   Slices whose replacement was withdrawn, in the order they were withdrawn.
   */
  readonly revertedChunkIndices: readonly number[];

  /**
   What the guard did, in scorecard-stable wording.
   */
  readonly findings: readonly string[];

  /**
   Surviving replacements whose text the guard trimmed, with the text the
   document carries, so a ledger can say what shipped rather than what was
   decided.
   */
  readonly trimmed: readonly SliceReplacement[];
};

/**
 Splices replacements into a document and settles what it can carry, repeating
 until nothing is left to take back.

 THREE OUTCOMES, and the name only says the first. A replacement that breaks
 the footnote graph is withdrawn, blamed by the identifier it moved. A
 STRUCTURAL parse regression first tests whether one withdrawal repairs the
 whole document; only when no such proof exists does it take every replacement. An
 assembly that reassembles to the archive text is CANONICALIZED rather than
 withdrawn for fault: nobody did anything wrong, and the document simply says
 so. Any reader of `revertedChunkIndices` is reading all three.

 ITERATES TO A FIXPOINT rather than checking once. Withdrawing a replacement
 can orphan an identifier a DIFFERENT replacement introduced alongside it:
 one slice renumbers `[^1]` to `[^2]` while another supplies the `[^2]`
 definition, so withdrawing the first leaves the second's definition with
 nothing pointing at it. Each round withdraws at least one replacement or
 trims at least one definition block, so the loop is bounded by their count
 plus the count of definition blocks.

 AN ORPHAN DEFINITION IN A DEFINITIONS-ONLY REPLACEMENT is trimmed rather
 than withdrawn with its siblings (`assembly-orphan-trim.ts`): the nineteenth
 `hakureico` pass of 2026-09-09 lost the definition its body needed because
 the one beside it had no reference on the sealed page.

 The guard runs at ASSEMBLY, after per-slice records were settled and cached,
 so a withdrawn slice's record still says it changed while the document ships
 the archive's text. That is deliberate: the record says what the judges
 chose, and this says what the document could carry. Every caller derives its
 shipped counts from {@link GuardedAssembly.replacements} rather than from the
 records. The guard is deterministic, so a resumed run withdraws the same
 slices without asking anyone again.

 @param targetText - translation as it stands, which is also the fallback

 @param slices - prepared slice pairs in document order

 @param replacements - accepted replacement per changed slice

 @returns Assembled document, surviving replacements, and what was withdrawn

 @throws {@link Error} when the loop cannot settle, which its own bound makes
 unreachable and which must never be reported as a clean assembly

 @throws AssemblyContractError when a replacement names an unknown slice or
 repeats its own incumbent, checked HERE rather than left to each caller: a
 no-op replacement reassembles to the archive text, so the net-zero
 canonicalization would otherwise adopt it as a legitimate outcome and return
 an empty surviving set, which nothing downstream can tell from an honest one

 @example
 ```ts
 const guarded = guardFootnoteAssembly({ targetText, slices, replacements, },);
 ```
 */
export function guardFootnoteAssembly(
  {
    targetText,
    slices,
    replacements,
  }: {
    readonly targetText: string;
    readonly slices: readonly ChunkPair[];
    readonly replacements: readonly SliceReplacement[];
  },
): GuardedAssembly {
  // FIRST, before anything is spliced. Every lane already runs this check, and
  // running it here too is what makes the guard sound for a caller that does
  // not: a replacement repeating its own incumbent is indistinguishable, once
  // spliced, from a slice nobody touched, and the net-zero canonicalization
  // (`assembly-net-zero-canonicalized`) would
  // canonicalize it into an empty surviving set that reads as an honest run.
  assertReplacementsChange({
    slices,
    replacements,
  },);

  /**
   Archive text of every slice, keyed by chunk index as a string so the map
   is JSON-shaped like everything else that crosses this module.
   */
  const incumbentBySlice = new Map(slices.map(function toEntry(slice,) {
    return [
      String(slice.target
        .sliceIndex,),
      slice.target
        .text,
    ] as const;
  },),);

  /**
   Original text of every slice, keyed the same way, for attributing a
   doubled definition to the carrier that does not own it.
   */
  const sourceBySlice = new Map(slices.map(function toSourceEntry(slice,) {
    return [
      String(slice.target
        .sliceIndex,),
      slice.source
        .text,
    ] as const;
  },),);

  /**
   Chunk indices withdrawn so far, in withdrawal order.
   */
  const withdrawn: number[] = [];

  /**
   What the guard did, accumulated across rounds.
   */
  const findings: string[] = [];

  /**
   Assembly once no round has anything left to withdraw.
   */
  const settled = (function settle(): {
    readonly assembledText: string;
    readonly surviving: readonly SliceReplacement[];
  } {
    /**
     Replacements still standing at the start of a round.
     */
    let surviving = replacements;
    /**
     Rounds the loop may take: one withdrawal per replacement, one trim per
     definition block, and the round that settles.
     */
    const rounds = replacements.length + definitionBlockCount({ replacements, },);
    for (let round = 0; round <= rounds; round += 1) {
      /**
       This round's replacements under a name nothing reassigns, so every
       closure this round makes reads the round it was made in rather than the cursor.
       */
      const standing = surviving;

      /**
       Document as the surviving replacements make it.
       */
      const assembledText = spliceSlices({
        targetText,
        slices,
        replacements: standing,
      },);

      // A NET NO-OP, which is a real outcome rather than a contradiction. Every
      // standing replacement differs from its own incumbent, and together they
      // reassemble to the archive text anyway: moving a line break across a
      // join is enough. Left alone, this round would return the archive text
      // beside a non-empty surviving set, and every caller would name those
      // slices as ones the document carries a change for while carrying none.
      //
      // Canonicalized rather than asserted against, because nobody did anything
      // wrong. The decisions survive in each lane's per-slice wordings; what
      // changes is only the document-level claim, which becomes the true one.
      if ((standing.length > 0) && (assembledText === targetText)) {
        findings.push(
          `assembly-net-zero-canonicalized (${
            String(standing.length,)
          } ${
            wordForCount({
              count: standing.length,
              one: 'slice',
              many: 'slices',
            },)
          }), since their replacements reassemble to the archive text`,
        );
        withdrawn.push(...standing.map(function toIndex(replacement,): number {
          return replacement.sliceIndex;
        },),);
        return {
          assembledText: targetText,
          surviving: [],
        };
      }

      /**
       Parse regressions this assembly introduced, which no identifier names:
       a stray comment opener masks markers document-wide, a downgrade
       means the strict parser refused what the archive accepted, and a page
       no grammar reads has no footnote graph to ask.
       */
      const regressions = introducedStructuralRegressions({
        incumbentText: targetText,
        assembledText,
      },);

      /**
       Footnote defects this assembly introduced, none read from a page that
       no grammar reads.
       */
      const introduced = regressions.includes(UNREADABLE_PAGE,)
        ? []
        : introducedFootnoteFindings({
          incumbentText: targetText,
          assembledText,
        },);
      if ((introduced.length === 0) && (regressions.length === 0)) {
        return {
          assembledText,
          surviving: standing,
        };
      }

      /**
       Orphan definitions cut out of definitions-only replacements, so the
       notes the page needs stay beside the one it cannot carry.
       */
      const trimmed = trimOrphanDefinitions({
        findings: introduced,
        replacements: standing,
        incumbentBySlice,
      },);
      if (trimmed.trimmed) {
        findings.push(...trimmed.findings,);
        withdrawn.push(...trimmed.withdrawn,);
        surviving = trimmed.replacements;
        continue;
      }

      /**
       What each finding may withdraw: the suspects whose withdrawal provably clears it on the page as it will be
       written, the slice that lost a missing reference first (`assembly-withdrawal-proof.ts`).
       */
      const attributions = introduced.map(function attribute(finding,) {
        return attributeFinding({
          finding,
          standing,
          targetText,
          slices,
          incumbentBySlice,
          sourceBySlice,
        },);
      },);
      /**
       Slices to withdraw this round.
       */
      const culprits = new Set(attributions.flatMap(function toProven(attribution,) {
        return attribution.proven;
      },),);
      if ((culprits.size === 0) && (regressions.length > 0)) {
        /**
         Whole-document counterfactual may prove a withdrawal without an identifier.
         */
        const proven = singleStructuralWithdrawal({
          targetText,
          slices,
          replacements: standing,
        },);
        for (const sliceIndex of proven) {
          culprits.add(sliceIndex,);
          findings.push(
            `assembly-structure-single-withdrawal slice ${String(sliceIndex,)}: reverting this replacement leaves no introduced structural or footnote defect`,
          );
        }
        /**
         With no single repair, the withdrawal that moves the first strict
         refusal furthest later (class fifty-eight): the next round reads what
         is left, one break at a time, instead of giving the page up.
         */
        const advancing = (proven.length === 0)
          ? advancingStructuralWithdrawal({
            targetText,
            slices,
            replacements: standing,
          },)
          : [];
        for (const step of advancing) {
          culprits.add(step.sliceIndex,);
          /**
           What reverting it does to the strict parse, offsets on the page as
           assembled this round.
           */
          const effect = step.cleared
            ? `clears the strict-parse refusal at offset ${String(step.from,)}`
            : `moves the first strict-parse refusal from offset ${String(step.from,)} to ${String(step.to,)}`;
          findings.push(
            `assembly-structure-advancing-withdrawal slice ${String(step.sliceIndex,)}: reverting this replacement ${effect}`,
          );
        }
      }
      // NOTHING PROVABLY CLEARS A FINDING THAT HAS SUSPECTS: nothing is withdrawn and the orphan is reported (the lead's
      // adoption of 2026-10-29, `assembly-withdrawal-proof.ts`). A withdrawal that cannot clear it only loses a good
      // translation, as slice 115 of XingZ6022 did, and the publish check names what stays.
      /**
       Whether no withdrawal is proven and the page has no parse regression to answer for either.
       */
      const nothingProven = (culprits.size === 0) && (regressions.length === 0);
      if (nothingProven && everyFindingSuspected({ attributions, },)) {
        findings.push(...footnoteRoundFindings({
          attributions,
          round,
        },),);
        return {
          assembledText,
          surviving: standing,
        };
      }
      if (culprits.size === 0) {
        // Nothing changed its mention of the identifier at fault, or the defect
        // names no identifier at all: it came from how the replacements MEET
        // rather than from what any one of them says. A slice's stray comment
        // opener masks markers document-wide, and a replacement whose tail runs
        // into the next block can stop an untouched definition line being read
        // as one.
        //
        // No single withdrawal proved a structurally and footnote-valid page.
        // Choosing a slice to withdraw here would still be a guess, and shipping
        // a document the lane knowingly broke is worse. So every replacement is
        // withdrawn: the archive's own text is the one thing certain to parse
        // as it did before, and the per-slice records still hold every decision
        // the judges made.
        for (const finding of introduced) {
          findings.push(
            `assembly-footnote-unattributable ${finding.kind} `
              + `${finding.convention} ${finding.identifier}`,
          );
        }
        for (const kind of regressions)
          findings.push(`assembly-structure-unattributable ${kind}`,);
        findings.push(
          `assembly-withdrew-every-replacement (${
            String(standing.length,)
          } ${
            wordForCount({
              count: standing.length,
              one: 'slice',
              many: 'slices',
            },)
          }), since no slice could be blamed for the defect`,
        );
        withdrawn.push(...standing.map(function toIndex(replacement,): number {
          return replacement.sliceIndex;
        },),);
        return {
          assembledText: targetText,
          surviving: [],
        };
      }
      findings.push(...footnoteRoundFindings({
        attributions,
        round,
      },),);
      // Recorded in the SAME round it is SEEN, not left to the next one. A
      // regression this round's withdrawal happens to answer as well never
      // reaches another round, so without this the document reports only the
      // footnote that was blamed and the parse damage leaves no trace at all.
      //
      // OBSERVED rather than reverted, and the difference is not pedantry.
      // What this round withdraws is whatever a FOOTNOTE identifier named;
      // whether that also answers the parse regression is unknown until the
      // next round splices what is left. A regression that survives is
      // reported again by the unattributable branch, which is what actually
      // withdraws for it.
      for (const kind of regressions) {
        findings.push(
          `assembly-structure-observed ${kind} (round ${String(round + 1,)})`,
        );
      }
      withdrawn.push(...culprits,);
      surviving = standing.filter(function stands(replacement,): boolean {
        return !culprits.has(replacement.sliceIndex,);
      },);
    }
    throw new Error(
      `footnote assembly guard ran ${String(rounds + 1,)} ${
        wordForCount({
          count: rounds + 1,
          one: 'round',
          many: 'rounds',
        },)
      } `
        + 'without settling, which its own bound makes impossible: every round '
        + 'withdraws at least one replacement or trims one definition block',
    );
  })();

  /**
   Text each replacement arrived with, by slice.
   */
  const givenBySlice = new Map(replacements.map(function toEntry(replacement,) {
    return [
      replacement.sliceIndex,
      replacement.replacementText,
    ] as const;
  },),);
  return {
    assembledText: settled.assembledText,
    replacements: settled.surviving,
    revertedChunkIndices: withdrawn,
    findings,
    trimmed: settled.surviving
      .filter(function wasTrimmed(replacement,): boolean {
        return givenBySlice.get(replacement.sliceIndex,) !== replacement.replacementText;
      },),
  };
}

//endregion Assembly integrity
