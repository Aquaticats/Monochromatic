import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import type { DeclaredNamePair, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import type {
  ArtifactConsistencyGroup,
  ArtifactConsistencyPlaceName,
  ArtifactConsistencySlice,
  ConsistencyKeptReason,
  ConsistencyOccurrenceOutcome,
} from '@monochromatic-dev/translation-repair-artifact/ts';
import {
  type ConsistencyFloorVerdict,
  consistencyFloorVerdict,
} from './pass-consistency-floors.ts';
import type { ConsistencySliceFacts, } from './pass-consistency-slices.ts';
import { ConsistencyInvariantError, } from '@monochromatic-dev/translation-repair-consistency/ts';
import { settledSlicesOf, } from './pass-consistency-apply-plan.ts';
import {
  gainFindingOf,
  keptReasonOfGain,
} from './pass-consistency-apply-settled.ts';
import {
  carriedPlaceName,
  finalGroupsOf,
  pendingNamed,
  winnerOver,
} from './pass-consistency-apply-record.ts';
import type { SettledGroup, } from './pass-consistency-types.ts';

//region Pass consistency apply
// WHERE THE GROUPS' REPLACEMENTS MEET EACH OTHER AND THE FLOORS. Every group
// was judged over the English as it stood when the pass began, so two groups
// can address one slice, and a replacement of one can overlap a replacement of
// another. Replacements are applied slice by slice, the earlier group winning
// wherever two overlap (the later is recorded `overlapped` for every place it
// carried), and the slice with its replacements made is put through every floor
// as one text: a slice the floors refuse keeps the wording it had, and every
// place a replacement made in it carried is recorded `kept-floor` (the owner's
// ruling of 2026-10-11). A replacement already left out for overlapping an
// earlier one stays `overlapped`, whether the floors then pass or refuse its
// slice. `OVERLAPPED` NEVER NAMES A PLACE OF THE REPLACEMENT'S OWN GROUP: the
// places of one group that a block holds are carried by one replacement
// (`pass-consistency-carry.ts`), and the standing of a span
// (`consistency-carry-standing.ts`) names no block around the block of other
// words of the same English, at any depth of nesting (a span that no single
// paragraph of a container holds has no block), so the blocks of a group's
// replacements never nest. A BREACH IS CAUGHT WHERE THE REPLACEMENTS ARE NAMED:
// `sliceEditsOf` (`pass-consistency-apply-record.ts`) throws a
// `ConsistencyInvariantError` naming both replacements when two replacements of
// one group overlap in a slice, before any outcome is recorded, so a change that
// breaks the guarantee fails here in the writer and not when a resume reads the
// section back, where `assertGaveWayResolves` refuses an overlapped place that
// names its own group.
//
// A REPLACEMENT THAT CHANGES THE WORDS ANOTHER GROUP LOCATED IS LEFT OUT UNLESS
// THAT GROUP IS REPLACING THEM. A writer's block for one group can cover a place
// of another group, and the groups settle at the same time, so the writer is only
// asked to keep those words (`pass-consistency-carry-units.ts`). Here the
// replacement that covers another group's located place and changes its words is
// left out whichever group comes first, recorded `overlapped` for every place it
// carried, with a finding and a warning that name both. A code splice replaces
// only the words of its own receiving places, so it is left out against every
// located place of another group that group has no pending replacement over in
// the slice (`pass-consistency-apply-located.ts`); against a place the other
// group replaces, the earlier group wins.
//
// AN OVERLAPPED PLACE NAMES WHAT IT GAVE WAY TO (`gaveWayTo`): the located place
// whose words the replacement would have changed, for a replacement left out
// here, and the place carried by the earlier replacement that won the words, for
// a later one that overlapped it. THE GROUP THAT GIVES WAY KEEPS ITS PLACES' WORDS AS
// THEY WERE, so the page stays inconsistent for that group, the cost of settling the
// groups at the same time over the English as it stood when the pass began.
//
// NO REPLACEMENT THAT IS MADE CHANGES THE LOCATED WORDS OF A PLACE THAT IS NOT
// ALREADY RECORDED `overlapped`. A block is made only where it changes no located
// words of another group. A code splice is made over such words only where their
// group replaces them, and that replacement holds the words, so the two meet and
// one of them is the later one, left out. The place is then `overlapped` and
// names what it gave way to, either from the left-out branch or from the branch
// that overlaps a later replacement with an earlier one, so nothing is left to
// record once the replacements are made. `pass-consistency-apply.unit.test.ts`
// pins this over every small arrangement of groups built through `applyConsistency`.
//
// A CODE SPLICE THAT CARRIES AN APOSTROPHE IN IS CHECKED ON THE WHOLE PAGE. The planner
// (`consistency-splice.ts`) plans a replacement that brings a proven apostrophe to an
// edge where the receiving words held none, and whether that mark stays an apostrophe
// depends on the quotation characters of the receiving page, in any slice of it, and on
// the other replacements of the pass. So before the first slice is applied, the page is
// looked at with every replacement of every slice made
// (`pass-consistency-apply-plan.ts`, `pass-consistency-apply-gains.ts`): a splice whose
// carried apostrophe is not proven there is left out, repeated until the choice no
// longer changes, and the splices left out are then tried again one at a time on the
// final page (`pass-consistency-apply-readmit.ts`), so one that the page would carry is
// made. Each place a splice that stays out carried is recorded `kept-current` with a
// finding. THE FINDING STATES THE REASON THAT HOLDS ON THE FINAL PAGE, not the reason the
// first reading gave: a mark of the page that could pair with the splice's apostrophe, a
// carried apostrophe of a splice that stands that it would unprove, a floor that refuses the
// slice with it, words that overlap a replacement that is made, or a slice the floors
// refused that keeps its wording whole (`gainFindingOf`). THE RECORD OF EACH PLACE STATES THE SAME REASON AS DATA
// (`keptBecause`, a closed set: `keptReasonOfGain`), which the reader of the artifact requires for a place
// the pass left alone after the splice was chosen; the planner's own refusals at the edge of a span
// (`edgeRefusalOf`) are recorded the same way.
// THE FLOORS RUN BEFORE ANYTHING IS RECORDED, inside `settledSlicesOf`, in the same loop as that
// check: a slice the floors refuse keeps its wording, which puts back the words its replacements
// took out (a closing mark among them), so the page is looked at again without the refused
// slice's replacements, and what that leaves out is put through the floors again, until no floor
// refuses another slice. The loop over its result only records what that settled, slice by slice.
//
// A REPLACEMENT CARRIES SEVERAL PLACES WHEN A WRITER'S BLOCK DOES: the block is
// rewritten once for every receiving place in it, so the replacement is named by
// all of them and its outcome is every one of theirs.

/**
 What the pass came to once the replacements were applied.

 @example
 ```ts
 const applied: AppliedConsistency = { groups: [], slices: [], findings: [], };
 ```
 */
export type AppliedConsistency = {
  /**
   Every group, with its final outcomes.
   */
  readonly groups: readonly ArtifactConsistencyGroup[];

  /**
   Slices whose text changed.
   */
  readonly slices: readonly ArtifactConsistencySlice[];

  /**
   What the pass reports beside its groups.
   */
  readonly findings: readonly string[];
};

/**
 Runs the floors on a changed slice.

 @param facts - slice

 @param currentText - wording the slice carries now

 @param candidateText - slice with the replacements made

 @param declared - name pairs the front matter declares

 @param declaredNames - declared name forms

 @param declaredSubjectForms - declared forms among them that name the subject, which may stand in for one another

 @returns What the floors found

 @example
 ```ts
 const verdict = floorsOn({ facts, currentText, candidateText, declared, declaredNames, },);
 ```
 */
function floorsOn(
  {
    facts,
    currentText,
    candidateText,
    declared,
    declaredNames,
    declaredSubjectForms,
  }: {
    readonly facts: ConsistencySliceFacts;
    readonly currentText: string;
    readonly candidateText: string;
    readonly declared: readonly DeclaredNamePair[];
    readonly declaredNames: readonly string[];
    readonly declaredSubjectForms?: readonly string[];
  },
): ConsistencyFloorVerdict {
  return consistencyFloorVerdict({
    sliceIndex: facts.sliceIndex,
    sourceText: facts.sourceText,
    currentText,
    candidateText,
    lineStructured: facts.lineStructured,
    declared,
    declaredNames,
    ...((declaredSubjectForms === undefined) ? {} : { declaredSubjectForms, }),
    disputedWordings: facts.disputedWordings,
  },);
}

/**
 Applies every group's replacements slice by slice and puts each changed slice
 through the floors.

 @param settled - groups in page order, with their pending replacements

 @param slices - what the pass knows of each slice

 @param declared - name pairs the front matter declares

 @param declaredNames - declared name forms

 @param declaredSubjectForms - declared forms among them that name the subject, which may stand in for one another

 @param l - logger the pass writes to

 @returns Groups with final outcomes, changed slices and findings

 @example
 ```ts
 const applied = applyConsistency({ settled, slices, declared, declaredNames, l, },);
 ```
 */
export function applyConsistency(
  {
    settled,
    slices,
    declared,
    declaredNames,
    declaredSubjectForms,
    l,
  }: {
    readonly settled: readonly SettledGroup[];
    readonly slices: readonly ConsistencySliceFacts[];
    readonly declared: readonly DeclaredNamePair[];
    readonly declaredNames: readonly string[];
    readonly declaredSubjectForms?: readonly string[];
    readonly l: Logger;
  },
): AppliedConsistency {
  /**
   Logger pre-tagged with this function's name.
   */
  const al = tagged({
    tag: applyConsistency.name,
    l,
  },);
  /**
   Final outcome of every place a replacement carried, by group and place.
   */
  const outcomes = new Map<string, ConsistencyOccurrenceOutcome>();
  /**
   Slices changed.
   */
  const changed: ArtifactConsistencySlice[] = [];
  /**
   What the pass reports beside its groups.
   */
  const findings: string[] = [];
  /**
   The place each overlapped place gave way to, by group and place.
   */
  const gaveWayTo = new Map<string, ArtifactConsistencyPlaceName>();
  /**
   Records the place that every place a replacement carried gave way to.

   @param carriers - replacements with the places each carries

   @param ids - names of the replacements

   @param name - place they gave way to
   */
  function recordGaveWay(
    {
      carriers,
      ids,
      name,
    }: {
      readonly carriers: ReadonlyMap<string, readonly string[]>;
      readonly ids: readonly string[];
      readonly name: ArtifactConsistencyPlaceName;
    },
  ): void {
    for (const id of ids) {
      for (const key of carriers.get(id,) ?? []) {
        gaveWayTo.set(
          key,
          name,
        );
      }
    }
  }
  /**
   Why the pass left a place as it was, by group and place, for the places a code splice that was left out carried.
   */
  const keptBecause = new Map<string, ConsistencyKeptReason>();
  /**
   Records why a code splice that was left out leaves every place it carried as it was.

   @param carriers - replacements with the places each carries

   @param id - name of the splice

   @param reason - why the final page does not carry it
   */
  function recordKept(
    {
      carriers,
      id,
      reason,
    }: {
      readonly carriers: ReadonlyMap<string, readonly string[]>;
      readonly id: string;
      readonly reason: ConsistencyKeptReason;
    },
  ): void {
    for (const key of carriers.get(id,) ?? []) {
      keptBecause.set(
        key,
        reason,
      );
    }
  }
  /**
   Records what became of a replacement for every place it carried.

   @param carriers - replacements with the places each carries

   @param ids - names of the replacements

   @param outcome - what became of them
   */
  function recordOutcome(
    {
      carriers,
      ids,
      outcome,
    }: {
      readonly carriers: ReadonlyMap<string, readonly string[]>;
      readonly ids: readonly string[];
      readonly outcome: ConsistencyOccurrenceOutcome;
    },
  ): void {
    for (const id of ids) {
      for (const key of carriers.get(id,) ?? []) {
        outcomes.set(
          key,
          outcome,
        );
      }
    }
  }
  for (
    const {
      plan: {
        facts,
        currentText,
        carriers,
        pendings,
        leftOut,
        toMake,
        unprovenGains,
      },
      made,
      verdict,
      gainReasons,
    } of settledSlicesOf({
      settled,
      slices,
      floorsOn: function floorsOfPlan(
        {
          plan,
          candidateText,
        },
      ): ConsistencyFloorVerdict {
        return floorsOn({
          facts: plan.facts,
          currentText: plan.currentText,
          candidateText,
          declared,
          declaredNames,
          ...((declaredSubjectForms === undefined) ? {} : { declaredSubjectForms, }),
        },);
      },
    },)
  ) {
    for (const {
      id,
      other,
    } of leftOut) {
      /**
       What is reported of the replacement left out.
       */
      const sentence = `slice ${String(facts.sliceIndex,)}: the consistency replacement of ${id} was left out because it `
        + `changes the words that group ${String(other.group + 1,)} ${other.label} located`;
      al.warn(sentence,);
      findings.push(sentence,);
      recordGaveWay({
        carriers,
        ids: [id,],
        name: {
          group: other.group,
          label: other.label,
        },
      },);
    }
    recordOutcome({
      carriers,
      ids: leftOut.map(function idOf({ id, },): string {
        return id;
      },),
      outcome: 'overlapped',
    },);
    for (const { id, } of unprovenGains) {
      /**
       Why the replacement is left out, on the final page.
       */
      const reason = gainReasons.get(id,);
      if (reason === undefined) {
        throw new ConsistencyInvariantError({ invariant:
          `slice ${String(facts.sliceIndex,)} left out the consistency replacement of ${id} and gave no reason, `
            + 'and the settling gives a reason for every replacement it leaves out', },);
      }
      /**
       What is reported of the replacement left out.
       */
      const sentence = gainFindingOf({
        sliceIndex: facts.sliceIndex,
        id,
        reason,
      },);
      al.warn(sentence,);
      findings.push(sentence,);
      recordKept({
        carriers,
        id,
        reason: keptReasonOfGain({ reason, },),
      },);
    }
    recordOutcome({
      carriers,
      ids: unprovenGains.map(function idOf({ id, },): string {
        return id;
      },),
      outcome: 'kept-current',
    },);
    // A slice with nothing to make was never put through the floors.
    if ((made === undefined) || (verdict === undefined))
      continue;
    recordOutcome({
      carriers,
      ids: made.overlapped,
      outcome: 'overlapped',
    },);
    /**
     The replacements that were made, in priority order.
     */
    const madeEdits = toMake.filter(function wasMade({ id, },): boolean {
      return made.applied
        .includes(id,);
    },);
    /**
     The replacements left out for overlapping an earlier one.
     */
    const losers = toMake.filter(function wasOverlapped({ id, },): boolean {
      return made.overlapped
        .includes(id,);
    },);
    for (const loser of losers) {
      /**
       The replacement that won the words the later one overlapped.
       */
      const winner = winnerOver({
        loser,
        made: madeEdits,
      },);
      recordGaveWay({
        carriers,
        ids: [loser.id,],
        name: carriedPlaceName(pendingNamed({
          pendings,
          id: winner.id,
        },),),
      },);
    }
    if (verdict.kind === 'passes') {
      recordOutcome({
        carriers,
        ids: made.applied,
        outcome: 'replaced',
      },);
      changed.push({
        sliceIndex: facts.sliceIndex,
        text: made.text,
      },);
      continue;
    }
    recordOutcome({
      carriers,
      ids: made.applied,
      outcome: 'kept-floor',
    },);
    al.warn(
      `slice ${String(facts.sliceIndex,)}: the floors refused the consistency replacement, so the slice keeps its wording: ${
        verdict.findings
          .join(' | ',)
      }`,
    );
    findings.push(
      `slice ${String(facts.sliceIndex,)}: the floors refused the consistency replacement: ${
        verdict.findings
          .join(' | ',)
      }`,
    );
  }
  return {
    groups: finalGroupsOf({
      settled,
      outcomes,
      gaveWayTo,
      keptBecause,
    },),
    slices: changed,
    findings,
  };
}

//endregion Pass consistency apply
