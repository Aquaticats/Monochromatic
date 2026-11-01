import {
  type ConsistencyCount,
  ConsistencyInvariantError,
  consistencyLabel,
  type ConsistencyRendering,
  pageApostropheFormOf,
  planConsistencyEdit,
  quoteReferencesIn,
  runOffWinnerOf,
} from '@monochromatic-dev/translation-repair-consistency/ts';
import { pageDefinitionsOf, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type {
  ArtifactConsistencyRunOff,
  ConsistencyOccurrenceOutcome,
} from '@monochromatic-dev/translation-repair-artifact/ts';
import {
  type EdgeRefusal,
  edgeRefusalOf,
} from './pass-consistency-edge-refusal.ts';
import type { ConsistencyPlace, } from './pass-consistency-group.ts';
import type { ConsistencyRunOffAsked, } from './pass-consistency-run-off.ts';
import type {
  CarryReceiver,
  LocatedPlace,
} from './pass-consistency-types.ts';

//region Pass consistency decide
// WHO WON, AND WHAT EACH PLACE IS TO RECEIVE: the winner after any run-off, the
// winning place with its located rendering, and for every other place its
// provisional outcome and, where the chosen rendering is to be carried to it, the
// receiver. `pass-consistency-settle.ts` assembles these into the group.

/**
 The winner and the run-off's record, after the finalists of a split with no strict majority are run off.

 @param count - what the first round's ballots came to

 @param labels - labels of the places, in order

 @param runOff - what the run-off heard, as a list of zero or one: none when the choices had no finalists

 @returns The winner, and the run-off as the artifact records it when there was one, as a list of zero or one

 @throws {@link ConsistencyInvariantError} when a run-off was heard for choices that had no finalists, or the choices had finalists and no run-off was heard

 @example
 ```ts
 const { winner, } = afterRunOff({ count, labels, runOff: [], },);
 ```
 */
export function afterRunOff(
  {
    count,
    labels,
    runOff,
  }: {
    readonly count: ConsistencyCount;
    readonly labels: readonly string[];
    readonly runOff: readonly ConsistencyRunOffAsked[];
  },
): {
  readonly winner: ConsistencyCount['winner'];
  readonly runOff: readonly ArtifactConsistencyRunOff[];
} {
  /**
   What the run-off heard, when it was asked.
   */
  const [heard,] = runOff;
  if (heard === undefined) {
    if (count.finalists
      .length
      > 0) {
      throw new ConsistencyInvariantError({ invariant:
        'the choices had finalists and the bench was not asked again, and finalists are run off before the group is settled', },);
    }
    return {
      winner: count.winner,
      runOff: [],
    };
  }
  if (count.finalists
    .length
    === 0) {
    throw new ConsistencyInvariantError({ invariant:
      'a run-off was heard for choices that had no finalists, and the bench is asked again only when the choices have finalists', },);
  }
  /**
   What the run-off's choices come to.
   */
  const decided = runOffWinnerOf({
    voices: heard.voices,
    finalists: count.finalists,
    quorum: heard.quorum,
    labels,
    renderings: count.renderings,
  },);
  return {
    winner: decided.winner,
    runOff: [
      {
        finalists: count.finalists,
        ballots: heard.voices
          .map(function recordedChoice({
            seat,
            reason,
            choice,
          },) {
            return {
              seat,
              reason,
              choice,
            };
          },),
        usable: decided.usable,
        quorum: heard.quorum,
      },
    ],
  };
}

/**
 The winning place with its located rendering.

 @param places - the places a slice holds whole

 @param renderings - each place's rendering, in the order of the places

 @param position - position of the winner among the places, minus one when none won

 @returns The place and its rendering as a list of one, none when no place won

 @throws {@link ConsistencyInvariantError} when a winner's place or rendering is missing or its rendering is not located, since a winner is named only where its own rendering is located

 @example
 ```ts
 const chosen = chosenPlaceOf({ places, renderings, position: 0, },);
 ```
 */
export function chosenPlaceOf(
  {
    places,
    renderings,
    position,
  }: {
    readonly places: readonly ConsistencyPlace[];
    readonly renderings: readonly ConsistencyRendering[];
    readonly position: number;
  },
): readonly LocatedPlace[] {
  if (position === (-1))
    return [];
  /**
   The winning place.
   */
  const place = places[position];
  /**
   Its rendering.
   */
  const rendering = renderings[position];
  if ((place === undefined) || (rendering?.kind !== 'located')) {
    throw new ConsistencyInvariantError({ invariant: 'a winner is named only where its own rendering is located', },);
  }
  return [
    {
      place,
      rendering,
    },
  ];
}

/**
 What one place comes to once the winner is known: its provisional outcome, and
 the receiver it makes when the chosen rendering is to be carried to it.

 @param chosen - the winning place, as a list of zero or one

 @param place - the place

 @param position - position of the place among the group's places

 @param count - what the first round's ballots came to

 @param framed - whether the group is a frame with a slot

 @returns The provisional outcome and the receiver, as a list of zero or one; and, for a place whose splice the planner
 refused at the edge of the span on the English around it, where the recorded words and marks alone would let code move the
 chosen words in, the refusal, which the record keeps as the reason the place was left as it was

 @example
 ```ts
 const { outcome, receiver, } = plannedPlace({ chosen, place, position: 1, count, framed: false, },);
 ```
 */
export function plannedPlace(
  {
    chosen,
    place,
    position,
    count,
    framed,
  }: {
    readonly chosen: readonly LocatedPlace[];
    readonly place: ConsistencyPlace;
    readonly position: number;
    readonly count: ConsistencyCount;
    readonly framed: boolean;
  },
): {
  readonly outcome: ConsistencyOccurrenceOutcome;
  readonly receiver: readonly CarryReceiver[];
  readonly edge?: EdgeRefusal;
} {
  /**
   This place's rendering.
   */
  const rendering = count.renderings[position];
  /**
   The winner, when there is one.
   */
  const [winning,] = chosen;
  if ((winning === undefined) || (rendering === undefined)) {
    return {
      outcome: 'untouched',
      receiver: [],
    };
  }
  if (winning.place === place) {
    return {
      outcome: 'chosen',
      receiver: [],
    };
  }
  if (rendering.kind === 'unlocated') {
    return {
      outcome: 'unlocated',
      receiver: [],
    };
  }
  if (count.kept[position] === true) {
    return {
      outcome: 'kept-context',
      receiver: [],
    };
  }
  /**
   The wording the receiving place's slice carries now, the English the receiving words were located in.
   */
  const { current: receivingWording, } = place.slice;
  /**
   The character references of the page that decode to a quotation-like character: those of the English of the whole page where
   the pass holds it, else of the receiving slice, and those of the English of every slice of the page, a slice read alone
   included (`wholePageReferences`). The planner and the record of the edge refusal ask this one function, for any splice.

   @returns The references as the English writes them, none where the receiving place carries no wording
   */
  function pageReference(): readonly string[] {
    if (receivingWording.kind !== 'present')
      return [];
    /**
     The English of the whole page and where the receiving slice starts in it, absent where the pass holds one slice alone, and
     the references of the English of every slice, absent for facts built without them.
     */
    const {
      page,
      wholePageReferences,
    } = place.slice;
    /**
     The references of the page's own English, or of the receiving slice's where the pass holds one slice alone.
     */
    const own = quoteReferencesIn({ english: (page === undefined) ? receivingWording.text : page.text, },);
    return (wholePageReferences === undefined)
      ? own
      : [
        ...new Set([
          ...own,
          ...wholePageReferences,
        ],),
      ];
  }
  /**
   The link and footnote definitions of the page the receiving slice stands in, which a reference in the slice's wording resolves
   against whichever slice holds the definition; none where the pass holds one slice alone (n173).
   */
  const { page: receivingPage, } = place.slice;
  /**
   The page's definitions, empty for a slice read alone.
   */
  const definitions = (receivingPage === undefined)
    ? ''
    : pageDefinitionsOf({ pageText: receivingPage.text, },);
  /**
   What putting the chosen words here by code amounts to.
   */
  const plan = planConsistencyEdit({
    chosen: winning.rendering,
    target: rendering,
    framed,
    marks: 'compared',
    // The English is handed so the planner can read every mark outside the receiving words before and after the splice.
    ...((receivingWording.kind === 'present')
      ? {
        english: receivingWording.text,
        definitions,
      }
      : {}),
    pageReference,
    // The English of the page is read only when the receiving words share no apostrophe form.
    pageApostrophe: function pageApostropheOfPage(): readonly string[] {
      /**
       The wording the receiving place's slice carries now.
       */
      const { current, } = place.slice;
      if (current.kind !== 'present')
        return [];
      /**
       The English of the page and where the receiving slice starts in it, absent where the pass holds one slice alone.
       */
      const { page, } = place.slice;
      /**
       The page the chosen words' slice starts in, absent where the pass holds one slice alone.
       */
      const chosenPage = winning.place
        .slice
        .page;
      /**
       The chosen words as the chosen place holds them.
       */
      const {
        core: chosenCore,
        start: chosenStart,
      } = winning.rendering;
      /**
       The receiving words.
       */
      const { core: receivingCore, } = rendering;
      /**
       Where the chosen words end in their slice.
       */
      const chosenEnd = chosenStart + chosenCore.length;
      /**
       Where the chosen words stand in the page, none where the pass holds one slice alone. They arrive with their
       apostrophes, so they say nothing of the page's form.
       */
      const apart = (chosenPage === undefined)
        ? []
        : [
          {
            start: chosenPage.offset + chosenStart,
            end: chosenPage.offset + chosenEnd,
          },
        ];
      return pageApostropheFormOf({
        english: current.text,
        start: rendering.start,
        end: rendering.start + receivingCore.length,
        ...((page === undefined)
          ? {}
          : {
            page,
            apart,
          }),
      },);
    },
  },);
  if (plan.kind === 'same') {
    return {
      outcome: 'consistent',
      receiver: [],
    };
  }
  /**
   Why the splice was refused at the edge of the span, where the recorded words and marks alone would let code move the
   chosen words in; none where it was not, or where the pass holds no English of the place.
   */
  const edge = ((plan.kind === 'unavailable') && (receivingWording.kind === 'present'))
    ? edgeRefusalOf({
      chosen: winning.rendering,
      target: rendering,
      framed,
      english: receivingWording.text,
      definitions,
      pageReference,
    },)
    : 'unrefused';
  return {
    outcome: 'kept-current',
    ...((edge === 'unrefused') ? {} : { edge, }),
    receiver: [
      {
        place,
        rendering,
        position,
        label: consistencyLabel({ position, },),
        plan,
      },
    ],
  };
}

//endregion Pass consistency decide
