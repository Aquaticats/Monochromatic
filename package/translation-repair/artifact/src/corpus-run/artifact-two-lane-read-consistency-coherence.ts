import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  agreedQuoteOf,
  agreedSlotOf,
  ConsistencyInvariantError,
  consistencyLabels,
  type ConsistencyRendering,
  isKeptByMajority,
  majorityChoiceOf,
  planConsistencyEdit,
  usableVoicesOf,
  winnerOf,
} from '@monochromatic-dev/translation-repair-consistency/ts';
import {
  artifactGenerationCountsBySpan,
  type TwoLaneArtifactGeneration,
} from './artifact-two-lane-contract.ts';
import type {
  ArtifactConsistencyGroup,
  ArtifactConsistencyOccurrence,
} from './artifact-two-lane-consistency.ts';
import {
  asLocated,
  locatedContradicted,
  reasonContradicted,
} from './artifact-two-lane-read-consistency-located.ts';
import { assertSpanGroupCoherent, } from './artifact-two-lane-read-consistency-span.ts';
import { sameWinner, } from './artifact-two-lane-read-consistency-span-winner.ts';
import {
  assertSeatsHeardOnce,
  winningPositionOf,
} from './artifact-two-lane-read-consistency-tally.ts';

//region Artifact consistency coherence
// WHAT A RECORDED GROUP SAYS ABOUT ITSELF, recounted. The ballots are in the
// file, so the usable count, the winner, the quotes a majority agreed on, the
// places a majority kept and whether a place already carried the chosen words
// are all recomputed here and a file that disagrees with its own ballots is
// refused, as a contest slice's verdict is refused when its ballots say
// otherwise. What the recount cannot reach is the English of a slice, which
// the file does not hold for slices the pass left alone, so "the words occur
// exactly once" is the pass's check alone.

/**
 Checks one place's rendering against the quotes its ballots agreed on.

 @param group - the group, asked of the bench

 @param occurrence - the place

 @param path - where the place sits, for the parse error

 @throws {@link ArtifactParseError} when a located rendering is not the agreed
 quote and slot, or an unlocated one gives a reason the ballots contradict

 @example
 ```ts
 assertRenderingAgrees({ group, occurrence, path, },);
 ```
 */
function assertRenderingAgrees(
  {
    group,
    occurrence,
    path,
  }: {
    readonly group: ArtifactConsistencyGroup & { readonly verdict: { readonly kind: 'asked'; }; };
    readonly occurrence: ArtifactConsistencyOccurrence;
    readonly path: string;
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
   Quote and slot a strict majority gave this place.
   */
  const quotes = agreedQuoteOf({
    voices,
    label: occurrence.label,
  },);
  /**
   The quote a strict majority gave, if one did.
   */
  const [quote,] = quotes;
  /**
   The slot a strict majority gave beside that quote.
   */
  const slots = agreedSlotOf({
    voices,
    label: occurrence.label,
    ...((quote === undefined) ? {} : { quote, }),
  },);
  /**
   The slot a strict majority gave, if one did.
   */
  const [slot,] = slots;
  /**
   What the ballots allow the recorded rendering to say.
   */
  const { rendering, } = occurrence;
  if (rendering.kind === 'located') {
    /**
     Whether the recorded slot is not the agreed one, which only a frame has.
     */
    const slotDisagrees = (group.repeat
      .kind
      === 'refrain') && (rendering.slot !== slot);
    if ((rendering.quote !== quote) || slotDisagrees)
      throw new ArtifactParseError({
        path: `${path}.rendering`,
        reason: 'the quote and slot a strict majority of the usable ballots gave',
      },);
    if (locatedContradicted({
      rendering,
      framed: group.repeat
        .kind
        === 'refrain',
    },)) {
      throw new ArtifactParseError({
        path: `${path}.rendering`,
        reason: 'words that show a reader something, hold their markup in pairs and, in a frame, leave frame words beside the slot',
      },);
    }
    return;
  }
  if (reasonContradicted({
    reason: rendering.reason,
    quotes,
    slots,
    framed: group.repeat
      .kind
      === 'refrain',
  },)) {
    throw new ArtifactParseError({
      path: `${path}.rendering.reason`,
      reason: 'a reason the usable ballots do not contradict',
    },);
  }
}

/**
 Checks what became of each place of a group the bench decided.

 @param group - the group, asked of the bench, with a winner

 @param path - where the group sits, for the parse error

 @throws {@link ArtifactParseError} when an outcome is not the one the ballots
 and the recorded renderings give

 @example
 ```ts
 assertOutcomesFollow({ group, path, },);
 ```
 */
function assertOutcomesFollow(
  {
    group,
    path,
  }: {
    readonly group: ArtifactConsistencyGroup & { readonly verdict: { readonly kind: 'asked'; }; };
    readonly path: string;
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
   Rendering the chosen place carries, when there is a winner.
   */
  const chosen = group.occurrences[winningPositionOf({ group, },)];
  group.occurrences
    .forEach(function checkOne(
      occurrence,
      at,
    ): void {
    /**
     Where the place sits, for the parse error.
     */
      const here = `${path}.occurrences[${String(at,)}]`;
      if (chosen === undefined) {
        if (occurrence.outcome !== 'untouched')
          throw new ArtifactParseError({
            path: `${here}.outcome`,
            reason: 'untouched, since no place won',
          },);
        return;
      }
      if (occurrence === chosen) {
        if (occurrence.outcome !== 'chosen')
          throw new ArtifactParseError({
            path: `${here}.outcome`,
            reason: 'chosen, for the winning place',
          },);
        return;
      }
      if (occurrence.rendering
        .kind
        === 'unlocated') {
        if (occurrence.outcome !== 'unlocated')
          throw new ArtifactParseError({
            path: `${here}.outcome`,
            reason: 'unlocated, for a place whose rendering is not located',
          },);
        return;
      }
      if (isKeptByMajority({
        voices,
        label: occurrence.label,
      },)) {
        if (occurrence.outcome !== 'kept-context')
          throw new ArtifactParseError({
            path: `${here}.outcome`,
            reason: 'kept-context, for a place a strict majority keeps',
          },);
        return;
      }
      if (chosen.rendering
        .kind
        !== 'located') {
        throw new ConsistencyInvariantError({ invariant: 'a winner is named only where its own rendering is located, and the winner was recounted before the outcomes', },);
      }
    /**
     What moving the chosen words over this place amounts to.
     */
      const plan = planConsistencyEdit({
        chosen: asLocated({
          rendering: chosen.rendering,
          path: `${path}.occurrences`,
        },),
        target: asLocated({
          rendering: occurrence.rendering,
          path: `${here}.rendering`,
        },),
        framed: group.repeat
          .kind
          === 'refrain',
      // A generation fifteen section kept no mark beside a span, so its splice is recounted by the words alone.
        marks: 'unrecorded',
      },);
    /**
     Whether the outcome is one a replacement can end in.
     */
      const replacing = (occurrence.outcome === 'replaced') || (occurrence.outcome === 'kept-floor')
        || (occurrence.outcome === 'overlapped');
      if ((plan.kind === 'same') !== (occurrence.outcome === 'consistent')) {
        throw new ArtifactParseError({
          path: `${here}.outcome`,
          reason: 'consistent for a place that already carries the chosen words, and for no other',
        },);
      }
      if ((plan.kind === 'replace') && (!replacing)) {
        throw new ArtifactParseError({
          path: `${here}.outcome`,
          reason: 'replaced, kept-floor or overlapped, for a place the chosen words would replace',
        },);
      }
    },);
}

/**
 Checks a group asked of the bench against its own ballots.

 @param group - the group, asked of the bench

 @param path - where the group sits, for the parse error

 @param generation - generation the section was written under: the first shape
 is recounted by the exact text of the quotes as it was written, the sixteenth
 by the span of the place's English they locate, with the run-off and the carries

 @throws {@link ArtifactParseError} when the usable count, the quorum test, the
 winner, a rendering or an outcome is not what the ballots give

 @example
 ```ts
 assertAskedGroupCoherent({ group, path, generation: 15, },);
 ```
 */
export function assertAskedGroupCoherent(
  {
    group,
    path,
    generation,
  }: {
    readonly group: ArtifactConsistencyGroup & { readonly verdict: { readonly kind: 'asked'; }; };
    readonly path: string;
    readonly generation: TwoLaneArtifactGeneration;
  },
): void {
  if (artifactGenerationCountsBySpan({ generation, },)) {
    assertSpanGroupCoherent({
      group,
      path,
      generation,
    },);
    return;
  }
  /**
   Labels the sheet showed.
   */
  const labels = consistencyLabels({ count: group.occurrences
    .length, },);
  /**
   The usable ballots of the group.
   */
  const voices = usableVoicesOf({
    voices: group.verdict
      .ballots,
    labels,
  },);
  assertSeatsHeardOnce({
    seats: group.verdict
      .ballots
      .map(function seatOf({ seat, },): string {
        return seat;
      },),
    path: `${path}.verdict.ballots`,
  },);
  if (voices.length
    !== group.verdict
      .usable) {
    throw new ArtifactParseError({
      path: `${path}.verdict.usable`,
      reason: 'the number of ballots that can be counted against the places',
    },);
  }
  group.occurrences
    .forEach(function agrees(
      occurrence,
      at,
    ): void {
      assertRenderingAgrees({
        group,
        occurrence,
        path: `${path}.occurrences[${String(at,)}]`,
      },);
    },);
  /**
   The winner the ballots give.
   */
  const expected = winnerOf({
    usable: voices.length,
    quorum: group.verdict
      .quorum,
    chosen: majorityChoiceOf({ voices, },),
    labels,
    renderings: group.occurrences
      .map(function asCounted(
        occurrence,
        at,
      ): ConsistencyRendering {
        return (occurrence.rendering
          .kind
          === 'located')
          ? asLocated({
            rendering: occurrence.rendering,
            path: `${path}.occurrences[${String(at,)}].rendering`,
          },)
          : occurrence.rendering;
      },),
  },);
  if (!sameWinner({
    left: expected,
    right: group.verdict
      .winner,
  },)) {
    throw new ArtifactParseError({
      path: `${path}.verdict.winner`,
      reason: 'the winner a strict majority of the usable ballots, reaching the quorum, give',
    },);
  }
  assertOutcomesFollow({
    group,
    path,
  },);
}

//endregion Artifact consistency coherence
