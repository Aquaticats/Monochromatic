import type { ArchiveOriginalSpan, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  guardFootnoteAssembly,
  withholdLoneContainerHalves,
} from '@monochromatic-dev/translation-repair-assembly/ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { SliceReplacement, } from '@monochromatic-dev/translation-repair-page-pass/ts';
import {
  type PagePassesOutcome,
  runPagePasses,
} from './page-assembly-passes.ts';

//region Page assembly rounds
// THE PAGE PASSES READ THE PAGE THE GUARD LEAVES (ledger K5). A lane row the
// footnote guard takes back, or a container half withheld because its partner
// ships nothing, leaves the archive's text standing at that slice; but the
// passes had read the lane's row there, so the archive text shipped as no pass
// had read it: a day-first date an untouched slice would carry month first
// stayed day first. And a cross-slice pass (a handle's gloss placed at its
// first appearance) could decide on a row the page no longer carries (A15).
// So the passes and the guard run again over the rows left, until a round
// takes back no slice an earlier round had not; each round after the first
// adds a slice to those taken back, so the rounds are at most one more than
// the slices.
//
// A ROUND THAT TAKES BACK ONLY WHAT WAS ALREADY TAKEN BACK SETTLES (ledger
// T8, 2026-09-29). Two passes rewrite the archive's text on every slice the
// page carries, so a later round can hand the guard a pass-made row at a
// slice whose lane row an earlier round withdrew; were the guard to take that
// row back, every later round would make and lose it again. That round threw
// until 2026-09-29, which stopped the entry's page; the owner's rule is that
// a run always ships. The page it settles on is consistent: the slice stays
// withdrawn and ships the archive's text, and the caller writes no pass row
// the settling round's guard took back.

/**
 One round: the container halves, every page pass, and the footnote guard.

 @example
 ```ts
 const round: PageAssemblyRound = assembleRound({ slices, sourceText, targetText, replacements, archiveOriginalSpans, incumbentBySlice, },);
 ```
 */
export type PageAssemblyRound = {
  /**
   Container halves withheld and why.
   */
  readonly halves: ReturnType<typeof withholdLoneContainerHalves>;

  /**
   What the passes made of the rows left.
   */
  readonly passes: PagePassesOutcome;

  /**
   The footnote guard's reading of the page the passes left.
   */
  readonly guarded: ReturnType<typeof guardFootnoteAssembly>;
};

/**
 What a round took back: the halves it withheld and the rows its guard
 withdrew, the two fields the rounds read to decide whether to run again.

 @example
 ```ts
 const taken: RoundTakeBacks = { halves: { withheld: [], }, guarded: { revertedChunkIndices: [3,], }, };
 ```
 */
export type RoundTakeBacks = {
  /**
   Container halves the round withheld.
   */
  readonly halves: Pick<PageAssemblyRound['halves'], 'withheld'>;

  /**
   Rows the round's guard withdrew.
   */
  readonly guarded: Pick<PageAssemblyRound['guarded'], 'revertedChunkIndices'>;
};

/**
 Rows that change the archive, less any that repeat its own wording at their
 slice.

 A REPEAT IS NO CHANGE FOR THE ASSEMBLER: a content slice whose archive
 wording is blank and which ships nothing reaches it as an empty write, and a
 restoration that brings a slice back to the archive's exact wording leaves
 its override row saying what the page carries. The guard's first read and
 every round's footnote check kept their own copy of this test (audit area
 six, 2026-09-28); one copy now serves both.

 @param replacements - lane rows as they stand

 @param incumbentBySlice - archive text of every slice, by index

 @returns Rows whose wording differs from the archive's at their slice

 @example
 ```ts
 const changing = rowsChangingArchive({ replacements, incumbentBySlice, },);
 ```
 */
export function rowsChangingArchive(
  {
    replacements,
    incumbentBySlice,
  }: {
    readonly replacements: readonly SliceReplacement[];
    readonly incumbentBySlice: ReadonlyMap<number, string>;
  },
): readonly SliceReplacement[] {
  return replacements.filter(function changes(replacement,): boolean {
    return replacement.replacementText !== incumbentBySlice.get(replacement.sliceIndex,);
  },);
}

/**
 Runs one round over a set of lane rows.

 @param slices - preparation defining replacement spans

 @param sourceText - the original document

 @param targetText - archive text the replacement spans address

 @param replacements - lane rows this round starts from

 @param archiveOriginalSpans - spans sealed as the English original

 @param incumbentBySlice - archive text of every slice, by index

 @returns The round's halves, passes and guard

 @example
 ```ts
 const round = assembleRound({ slices, sourceText, targetText, replacements, archiveOriginalSpans, incumbentBySlice, },);
 ```
 */
function assembleRound(
  {
    slices,
    sourceText,
    targetText,
    replacements,
    archiveOriginalSpans,
    incumbentBySlice,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly sourceText: string;
    readonly targetText: string;
    readonly replacements: readonly SliceReplacement[];
    readonly archiveOriginalSpans: readonly ArchiveOriginalSpan[];
    readonly incumbentBySlice: ReadonlyMap<number, string>;
  },
): PageAssemblyRound {
  /**
   Rows less any container half whose partner ships nothing (class
   fifty-seven), so the guard never reads a closing tag with no opening.
   */
  const halves = withholdLoneContainerHalves({
    slices,
    replacements,
  },);
  /**
   What every page-assembly pass made of those rows.
   */
  const passes = runPagePasses({
    slices,
    sourceText,
    targetText,
    replacements: halves.replacements,
    archiveOriginalSpans,
  },);
  return {
    halves,
    passes,
    guarded: guardFootnoteAssembly({
      targetText,
      slices,
      replacements: rowsChangingArchive({
        replacements: passes.replacements,
        incumbentBySlice,
      },),
    },),
  };
}

/**
 Slices one round took back.

 @param round - round to read

 @returns Withheld halves, then the guard's withdrawals

 @example
 ```ts
 const taken = takenBackIn({ round, },);
 ```
 */
function takenBackIn({ round, }: { readonly round: RoundTakeBacks; },): readonly number[] {
  return [
    ...round.halves
      .withheld,
    ...round.guarded
      .revertedChunkIndices,
  ];
}

/**
 Slices one round took back that no earlier round had.

 @param round - round to read

 @param takenBack - slices earlier rounds took back

 @returns Its new withdrawals and withheld halves, in round order, each once

 @example
 ```ts
 freshlyTakenBack({ round, takenBack: [3,], },);
 ```
 */
export function freshlyTakenBack(
  {
    round,
    takenBack,
  }: {
    readonly round: RoundTakeBacks;
    readonly takenBack: readonly number[];
  },
): readonly number[] {
  return [
    ...new Set(takenBackIn({ round, },)
      .filter(function fresh(sliceIndex,): boolean {
        return !takenBack.includes(sliceIndex,);
      },),),
  ];
}

/**
 Whether a round took back any slice for the first time, which is what runs
 another round.

 @param fresh - slices it took back that no earlier round had

 @returns True when there is at least one

 @example
 ```ts
 if (takesBackAny({ fresh, },)) rerun();
 ```
 */
function takesBackAny({ fresh, }: { readonly fresh: readonly number[]; },): boolean {
  return fresh.length > 0;
}

/**
 Runs rounds until one takes back nothing an earlier round had not.

 @param slices - preparation defining replacement spans

 @param sourceText - the original document

 @param targetText - archive text the replacement spans address

 @param replacements - lane rows the page would write

 @param archiveOriginalSpans - spans sealed as the English original

 @param incumbentBySlice - archive text of every slice, by index

 @returns The settling round, every slice taken back in round order, and the
 findings of the rounds that took them back; the settling round's own guard
 may take back again a pass-made row at a slice already among them

 @example
 ```ts
 const { final, takenBack, earlierFindings, } = settlePageRounds({ slices, sourceText, targetText, replacements, archiveOriginalSpans, incumbentBySlice, },);
 ```
 */
export function settlePageRounds(
  {
    slices,
    sourceText,
    targetText,
    replacements,
    archiveOriginalSpans,
    incumbentBySlice,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly sourceText: string;
    readonly targetText: string;
    readonly replacements: readonly SliceReplacement[];
    readonly archiveOriginalSpans: readonly ArchiveOriginalSpan[];
    readonly incumbentBySlice: ReadonlyMap<number, string>;
  },
): {
  readonly final: PageAssemblyRound;
  readonly takenBack: readonly number[];
  readonly earlierFindings: readonly string[];
} {
  /**
   Slices taken back so far, in round order.
   */
  const takenBack: number[] = [];
  /**
   Findings of the rounds that took something back.
   */
  const earlierFindings: string[] = [];
  /**
   The first round.
   */
  const first = assembleRound({
    slices,
    sourceText,
    targetText,
    replacements,
    archiveOriginalSpans,
    incumbentBySlice,
  },);
  /**
   The latest round and what it took back that no earlier round had: A NAMED
   CELL, since the loop replaces both.
   */
  const state = {
    round: first,
    fresh: freshlyTakenBack({
      round: first,
      takenBack,
    },),
  };
  while (takesBackAny({ fresh: state.fresh, },)) {
    takenBack.push(...state.fresh,);
    /**
     What the round that took them back found.
     */
    const {
      halves,
      guarded,
    } = state.round;
    earlierFindings.push(
      ...halves.findings,
      ...guarded.findings,
    );
    state.round = assembleRound({
      slices,
      sourceText,
      targetText,
      replacements: replacements.filter(function stillStanding(replacement,): boolean {
        return !takenBack.includes(replacement.sliceIndex,);
      },),
      archiveOriginalSpans,
      incumbentBySlice,
    },);
    state.fresh = freshlyTakenBack({
      round: state.round,
      takenBack,
    },);
  }
  return {
    final: state.round,
    takenBack,
    earlierFindings,
  };
}

//endregion Page assembly rounds
