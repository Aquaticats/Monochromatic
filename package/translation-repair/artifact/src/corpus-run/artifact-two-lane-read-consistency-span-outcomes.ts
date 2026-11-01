import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  consistencyLabels,
  type ConsistencyRendering,
  isKeptByMajority,
  planConsistencyEdit,
  usableVoicesOf,
} from '@monochromatic-dev/translation-repair-consistency/ts';
import type { ArtifactConsistencyGroup, } from './artifact-two-lane-consistency.ts';
import {
  assertCarriedOutcomes,
  type CarriedPlace,
} from './artifact-two-lane-read-consistency-span-shared.ts';
import { winningPositionOf, } from './artifact-two-lane-read-consistency-tally.ts';

//region Artifact consistency span outcomes
// WHAT BECAME OF EACH PLACE OF A RECORDED GROUP OF THE SIXTEENTH SHAPE, recounted from the ballots, the recorded renderings and the
// carries. Its own file because the coherence of the group (`artifact-two-lane-read-consistency-span.ts`) recounts the renderings, the
// winner and the carries and this recounts what followed from them, and the two had grown past the length one file may have.

/**
 A group the bench was asked about.
 */
type AskedGroup = ArtifactConsistencyGroup & { readonly verdict: { readonly kind: 'asked'; }; };

/**
 Checks what became of each place of a group the bench decided, and the carries
 recorded beside them.

 @param group - the group, asked of the bench, with a winner

 @param renderings - each place's rendering in the counter's form

 @param path - where the group sits, for the parse error

 @param marks - whether a splice is recounted with the marks beside and inside
 the words compared, as a section that recorded them was written

 @param partialUnits - whether the section records a unit carried in part, so a
 place code cannot move the chosen words into may sit under a splice built for
 another place of its record and keeps its words when the splice is chosen

 @throws {@link ArtifactParseError} when an outcome is not the one the ballots,
 the recorded renderings and the carries give, or the carries are not one per
 receiving place

 @example
 ```ts
 assertSpanOutcomesFollow({ group, renderings, path, marks: 'compared', partialUnits: true, },);
 ```
 */
export function assertSpanOutcomesFollow(
  {
    group,
    renderings,
    path,
    marks,
    partialUnits,
  }: {
    readonly group: AskedGroup;
    readonly renderings: readonly ConsistencyRendering[];
    readonly path: string;
    readonly marks: 'compared' | 'unrecorded';
    readonly partialUnits: boolean;
  },
): void {
  /**
   The usable ballots of the group.
   */
  const voices = usableVoicesOf({
    voices: group.verdict
      .ballots,
    labels: consistencyLabels({ count: group.occurrences
      .length, },),
  },);
  /**
   Position of the chosen place, none when no place won.
   */
  const chosenAt = winningPositionOf({ group, },);
  /**
   Rendering of the chosen place.
   */
  const chosen = renderings[chosenAt];
  /**
   Labels of the places the chosen rendering is carried to, which the carries must name in order.
   */
  const receivers: string[] = [];
  /**
   The places the chosen words are carried to, with what each is checked against once all are known.
   */
  const carriedTo: CarriedPlace[] = [];
  group.occurrences
    .forEach(function checkOne(
      occurrence,
      at,
    ): void {
    /**
     Where the place sits, for the parse error.
     */
      const here = `${path}.occurrences[${String(at,)}]`;
    /**
     Rendering of this place in the counter's form.
     */
      const rendering = renderings[at];
    /**
     Fails the place's outcome unless it is the one named.
     */
      function requireOutcome({ expected, }: { readonly expected: string; },): void {
        if (occurrence.outcome !== expected) {
          throw new ArtifactParseError({
            path: `${here}.outcome`,
            reason: `${expected}, as the ballots and the renderings give for this place`,
          },);
        }
      }
      if (chosenAt === (-1)) {
        requireOutcome({ expected: 'untouched', },);
        return;
      }
      if (at === chosenAt) {
        requireOutcome({ expected: 'chosen', },);
        return;
      }
      if (rendering?.kind !== 'located') {
        requireOutcome({ expected: 'unlocated', },);
        return;
      }
      if (isKeptByMajority({
        voices,
        label: occurrence.label,
      },)) {
        requireOutcome({ expected: 'kept-context', },);
        return;
      }
      if (chosen?.kind !== 'located') {
        throw new ArtifactParseError({
          path: `${path}.verdict.winner`,
          reason: 'a place whose rendering is located, since a winner is named only where its own rendering is located',
        },);
      }
    /**
     What moving the chosen words over this place by code amounts to.
     */
      const plan = planConsistencyEdit({
        chosen,
        target: rendering,
        framed: group.repeat
          .kind
          === 'refrain',
        marks,
      },);
      if (plan.kind === 'same') {
        requireOutcome({ expected: 'consistent', },);
        return;
      }
      receivers.push(occurrence.label,);
    /**
     The carry recorded for this place.
     */
      const carry = (group.carries ?? []).find(function carriesIt(entry,): boolean {
        return entry.label === occurrence.label;
      },);
      if (carry === undefined) {
        throw new ArtifactParseError({
          path: `${path}.carries`,
          reason: `a carry for ${occurrence.label}, which the chosen words are carried to`,
        },);
      }
      carriedTo.push({
        occurrence,
        here,
        plan: plan.kind,
        carry,
      },);
    },);
  assertCarriedOutcomes({
    carriedTo,
    path,
    marks,
    partialUnits,
  },);
  if ((group.carries ?? []).map(function labelOf({ label, },): string {
    return label;
  },)
    .join('\n',)
    !== receivers.join('\n',)) {
    throw new ArtifactParseError({
      path: `${path}.carries`,
      reason: 'one carry for each place the chosen words are carried to, in the order of the places',
    },);
  }
}

//endregion Artifact consistency span outcomes
