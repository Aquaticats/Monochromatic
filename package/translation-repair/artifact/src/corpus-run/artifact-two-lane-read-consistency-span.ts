import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  agreedWordsOf,
  consistencyLabels,
  type ConsistencyRendering,
  containedQuoteWordsOf,
  foldedWordsOf,
  usableVoicesOf,
} from '@monochromatic-dev/translation-repair-consistency/ts';
import { rendersAsNothing, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  artifactGenerationCarriesPartialUnits,
  artifactGenerationCutsContainmentAtJoints,
  artifactGenerationRecordsMarks,
  type TwoLaneArtifactGeneration,
} from './artifact-two-lane-contract.ts';
import type {
  ArtifactConsistencyGroup,
  ArtifactConsistencyOccurrence,
} from './artifact-two-lane-consistency.ts';
import { assertCarriesRecount, } from './artifact-two-lane-read-consistency-carry-recount.ts';
import {
  asLocatedBySpan,
  edgeMarksContradictedBySpan,
  locatedContradictedBySpan,
  reasonContradictedBySpan,
} from './artifact-two-lane-read-consistency-span-located.ts';
import { assertSpanOutcomesFollow, } from './artifact-two-lane-read-consistency-span-outcomes.ts';
import { expectedSlotOf, } from './artifact-two-lane-read-consistency-span-slot.ts';
import {
  expectedWinnerOf,
  sameWinner,
} from './artifact-two-lane-read-consistency-span-winner.ts';
import { assertSeatsHeardOnce, } from './artifact-two-lane-read-consistency-tally.ts';

//region Artifact consistency span coherence
// WHAT A RECORDED GROUP OF THE SIXTEENTH SHAPE SAYS ABOUT ITSELF, recounted. The
// ballots are in the file, so the usable count, the words a strict majority
// quoted for each place, the slot inside them, the places a majority kept, the
// finalists and the run-off that settled it, the winner, each receiving
// place's carry and what became of each place are all recomputed here, and a
// file that disagrees with its own ballots is refused. What the recount cannot
// reach is the English of a slice, which the file does not hold for slices the
// pass left alone, so "the words stand exactly once there" is the pass's check
// alone, and the text of the current English candidate of a carry is taken as
// recorded. A splice is recounted with the marks beside and inside the words
// compared from generation seventeen, which recorded them
// (`consistency-splice.ts`), and by the words alone before, as those sections
// were written.
//
// THE RECOUNT HOLDS NO PAGE, SO THE PASS RECORDS THE DEFINITIONS THE SLOT NEEDS (ledger B661, B665). The pass reads a slot with the link
// and footnote definitions of the whole page (`pageDefinitionsOf` over the page text, `consistency-count.ts`), so a shortcut reference
// such as `[b]` inside a span is a link when its definition stands in another slice: `a[b]c` is then one word and `b` is no stand in
// it, and the pass records no slot. The file could not supply those definitions by itself. `parseConsistency` is handed the section and
// the indexes of the slices, the `slices` rows hold only the slices the pass changed (as they ended), the comparison rows hold each
// lane's text before the consolidation, and the consolidated English the pass read is recorded nowhere; rebuilding it takes the
// corpus files at the artifact's commit (`artifact-two-lane-rebuild.ts`), which a reader of one file does not have. So, FROM GENERATION
// EIGHTEEN, a located place of a frame records `definitions`, the smallest set of the page's definitions that gives its slot, wherever the
// page's definitions move the slot at all (`definitionsMovingSlotOf`, `consistency-slot-definitions.ts`), and the recount reads the slot
// with them: a place that records none is recounted over the span alone, so a pass-written place whose definitions were left out is
// refused, as are definitions that are not in the form a page's definitions are read in or that move no slot. GENERATION SEVENTEEN AND
// EARLIER KEEP THE LIMIT THEY WERE WRITTEN UNDER: they record no definitions, so the slot is recounted over the span alone, where `b`
// stands once, and a pass-written refrain of those generations whose span holds a reference defined in another slice, with a slot at the
// joint of the reference, is refused here (the recount expects a slot the pass could not find); the converse direction is the same limit
// in `asLocatedBySpan` (`artifact-two-lane-read-consistency-span-located.ts`). The shape needs a cross-slice definition and a slot at
// a word joint, and a case pins it for generation seventeen.
//
// A UNIT CARRIED IN PART IS READ FROM THE RECORD ALONE, from generation seventeen.
// The places of one block share one bench record, so the carries recorded under
// them are the same candidates, ballots and decision, and a splice among the
// candidates was built from the places of the block code could move the chosen words
// into. A place whose recomputed plan is `unavailable` then ends `kept-current` when
// the splice was chosen, a place whose plan is `replace` ends in one of the
// outcomes a replacement can end in, and a splice candidate under a place whose
// plan is `unavailable` is accepted only when another place of the group, recorded
// under the same record, has a `replace` plan (`artifact-two-lane-read-consistency-span-shared.ts`).
// Sixteen and fifteen keep the rule they were written under: such a place withholds
// the splice, so no carry of theirs holds a splice candidate under it.
//
// A SPLICE THAT BRINGS AN APOSTROPHE TO AN EDGE IS ALLOWED TO HAVE BEEN LEFT OUT.
// Whether a proven apostrophe carried to an edge where the receiving words held none
// is still an apostrophe on the page the pass makes depends on the quotation characters
// of the English and on the other replacements, which the record does not hold. So a
// place whose plan is `replace` and whose words would gain such an apostrophe
// (`bringsEdgeApostrophe`) may end `kept-current` after the splice was chosen, as well
// as in the outcomes a replacement ends in. The reader checks that the outcome is one
// the carry allows; it cannot check that the apostrophe was in fact not proven there.

/**
 A group the bench was asked about.
 */
type AskedGroup = ArtifactConsistencyGroup & { readonly verdict: { readonly kind: 'asked'; }; };

/**
 Checks one place's rendering against the words its ballots agreed on.

 @param group - the group, asked of the bench

 @param occurrence - the place

 @param path - where the place sits, for the parse error

 @param generation - generation the section was written under, which says whether the edges of a located place's words may hold a proven apostrophe and are checked for one

 @returns The place's rendering in the counter's form

 @throws {@link ArtifactParseError} when a located rendering is not the agreed
 words or slot, holds at an edge a single mark the pass could not have put
 there, or an unlocated one gives a reason the ballots contradict

 @example
 ```ts
 const rendering = assertSpanRenderingAgrees({ group, occurrence, path, generation: 17, },);
 ```
 */
function assertSpanRenderingAgrees(
  {
    group,
    occurrence,
    path,
    generation,
  }: {
    readonly group: AskedGroup;
    readonly occurrence: ArtifactConsistencyOccurrence;
    readonly path: string;
    readonly generation: TwoLaneArtifactGeneration;
  },
): ConsistencyRendering {
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
   The spans the quotes of more than half of all the usable ballots contain, in the fold, longest first. THE RECOUNT CANNOT SAY WHICH OF
   THEM STANDS EXACTLY ONCE in the English (the file does not hold the English of a slice the pass left alone), so a located
   rendering is held to be one of them and an unlocated one is held to a reason one of them allows. A joint of a longer word bars
   containment from generation nineteen, as the locator reads it; a file of eighteen or earlier is recounted as its pass counted,
   with only a letter, a digit or a combining mark beside the match barring it (`artifactGenerationCutsContainmentAtJoints`).
   */
  const candidates = containedQuoteWordsOf({
    voices,
    label: occurrence.label,
    edges: artifactGenerationCutsContainmentAtJoints({ generation, },) ? 'locator' : 'letters',
  },);
  /**
   The candidates that show a reader something, which the pass searches the English for.
   */
  const readable = candidates.filter(function showsSomething(words,): boolean {
    return !rendersAsNothing({ text: words, },);
  },);
  /**
   What the ballots allow the recorded rendering to say.
   */
  const { rendering, } = occurrence;
  if (rendering.kind === 'unlocated') {
    // A FILE OF THE SEVENTEENTH GENERATION WRITTEN BEFORE THE SPANS WERE LOCATED BY CONTAINMENT (the run of TianqiChen66625 and
    // those before it) recorded `no-agreement` where a strict majority of the usable ballots had quoted no one text, so the
    // recorded reason is refused only when NEITHER reading allows it: the words a strict majority of the ballots quoted, as the
    // pass located them before, and the spans the quotes of more than half of all the usable ballots contain, as it does now. The earlier agreed
    // words are always among the candidates, so the two readings differ only for a place the earlier one left unlocated.
    if (reasonContradictedBySpan({
      reason: rendering.reason,
      // No agreement is contradicted by any candidate; the other reasons are read against the first candidate that shows
      // something, or against the first candidate when none does.
      words: ((rendering.reason === 'no-agreement') || (readable.length === 0)) ? candidates : readable.slice(
        0,
        1,
      ),
    },) && reasonContradictedBySpan({
      reason: rendering.reason,
      words: agreedWordsOf({
        voices,
        label: occurrence.label,
      },),
    },)) {
      throw new ArtifactParseError({
        path: `${path}.rendering.reason`,
        reason: 'a reason the usable ballots do not contradict',
      },);
    }
    return rendering;
  }
  /**
   The folded words of the recorded rendering, which are one of the spans the ballots allow.
   */
  const agreed = foldedWordsOf({ text: rendering.quote, },);
  if (!readable.includes(agreed,)) {
    throw new ArtifactParseError({
      path: `${path}.rendering`,
      reason: 'words that the quotes of more than half of all the usable ballots contain, as the place\'s English holds them',
    },);
  }
  if (locatedContradictedBySpan({ rendering, },)) {
    throw new ArtifactParseError({
      path: `${path}.rendering`,
      reason: 'words that show a reader something and hold their markup in pairs, and a slot that does the same and is not all of them',
    },);
  }
  if (artifactGenerationRecordsMarks({ generation, },) && edgeMarksContradictedBySpan({ rendering, },)) {
    throw new ArtifactParseError({
      path: `${path}.rendering`,
      reason: 'words that hold at most one apostrophe at an edge, directly beside a letter or a digit and never the opening '
        + 'curly single mark, since the pass puts only a proven apostrophe there',
    },);
  }
  /**
   The slot the ballots pin down inside the recorded words, none for a run or when they pin none.
   */
  const [expected,] = expectedSlotOf({
    voices,
    label: occurrence.label,
    agreed,
    span: rendering.quote,
    refrain: (group.repeat
      .kind
      === 'refrain'),
    ...((rendering.definitions === undefined) ? {} : { definitions: rendering.definitions, }),
    generation,
    path: `${path}.rendering`,
  },);
  if (rendering.slot !== expected?.slot) {
    throw new ArtifactParseError({
      path: `${path}.rendering`,
      reason: 'the slot a strict majority of the usable ballots gave inside those words, and none where they gave none',
    },);
  }
  return asLocatedBySpan({
    rendering,
    path: `${path}.rendering`,
    generation,
  },);
}

/**
 Checks a group of the sixteenth shape, asked of the bench, against its own
 ballots.

 @param group - the group, asked of the bench

 @param path - where the group sits, for the parse error

 @param generation - generation the section was written under, which says
 whether the marks beside each located place's words were recorded and a splice
 is recounted with them compared (seventeen) or by the words alone (sixteen)

 @throws {@link ArtifactParseError} when the usable count, a rendering, the
 winner, the run-off, a carry or an outcome is not what the ballots give

 @example
 ```ts
 assertSpanGroupCoherent({ group, path, generation: 17, },);
 ```
 */
export function assertSpanGroupCoherent(
  {
    group,
    path,
    generation,
  }: {
    readonly group: AskedGroup;
    readonly path: string;
    readonly generation: TwoLaneArtifactGeneration;
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
  /**
   Each place's rendering in the counter's form, checked against the ballots.
   */
  const renderings = group.occurrences
    .map(function agrees(
      occurrence,
      at,
    ): ConsistencyRendering {
      return assertSpanRenderingAgrees({
        group,
        occurrence,
        path: `${path}.occurrences[${String(at,)}]`,
        generation,
      },);
    },);
  if (!sameWinner({
    left: expectedWinnerOf({
      group,
      renderings,
      path,
      generation,
    },),
    right: group.verdict
      .winner,
  },)) {
    throw new ArtifactParseError({
      path: `${path}.verdict.winner`,
      reason: 'the winner a strict majority of the usable ballots, reaching the quorum, give, after the run-off when they tied',
    },);
  }
  assertCarriesRecount({
    carries: group.carries ?? [],
    path: `${path}.carries`,
    generation,
  },);
  assertSpanOutcomesFollow({
    group,
    renderings,
    path,
    marks: artifactGenerationRecordsMarks({ generation, },) ? 'compared' : 'unrecorded',
    partialUnits: artifactGenerationCarriesPartialUnits({ generation, },),
  },);
}

//endregion Artifact consistency span coherence
