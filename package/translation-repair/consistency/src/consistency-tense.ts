import {
  type ConsistencyEnglish,
  consistencyLabel,
} from './consistency-wire.ts';
import {
  tenseCandidateGroups,
  tenseCandidateLines,
} from '@monochromatic-dev/translation-repair-translate-stage/ts';

//region Consistency tense
// THE TENSE-CANDIDATE BLOCK OF THE CONSISTENCY SHEET (the lead's decision of
// 2026-10-12, after the owner's tense rulings of 2026-10-11). The judges of the
// contest, the slates, the run-off and the gate are shown a labelled block of
// the present-tense candidates of each text they choose among when the entry
// records a death. The consistency judges choose among renderings too, so they
// could carry a present-tense rendering to every place. They are shown the same
// block, built by the same functions (`tenseCandidateGroups`,
// `tenseCandidateLines`), for the English of each slice the sheet shows. It is
// evidence to weigh, never a refusal: nothing in the count reads it.

/**
 One slice's English the sheet shows, with the label it is shown under.

 @example
 ```ts
 const shown: ShownEnglish = { label: 'OCCURRENCE 1 TRANSLATION', text: 'Mittens naps.', };
 ```
 */
type ShownEnglish = {
  /**
   Label the sheet heads the English with.
   */
  readonly label: string;

  /**
   The slice's English.
   */
  readonly text: string;
};

/**
 The English the sheet shows, once per slice, under the label of the first
 place that stands in the slice, which is the label its block carries.

 @param places - every place in page order with its slice and English

 @returns One entry per slice that carries English

 @example
 ```ts
 const shown = shownEnglishOf({ places: [], },); // []
 ```
 */
function shownEnglishOf(
  {
    places,
  }: {
    readonly places: readonly {
      readonly sliceIndex: number;
      readonly english: ConsistencyEnglish;
    }[];
  },
): readonly ShownEnglish[] {
  /**
   Slices whose English an earlier place already shows.
   */
  const seen = new Set<number>();
  return places.flatMap(function shownAt(
    place,
    position,
  ): readonly ShownEnglish[] {
    if ((place.english
      .kind
      === 'absent') || seen.has(place.sliceIndex,))
      return [];
    seen.add(place.sliceIndex,);
    return [
      {
        label: `${consistencyLabel({ position, },)} TRANSLATION`,
        text: place.english
          .text,
      },
    ];
  },);
}

/**
 The lines of the tense block for the English a consistency sheet shows.

 @param lifeEnded - whether the original's entry records a death

 @param subjectNames - names the translation declares for its subject, read as named subjects beside the pronouns

 @param places - every place in page order with its slice and English

 @returns The block's lines as the other judging sheets print them, empty when
 the entry records no death or no shown English has a candidate

 @example
 ```ts
 const lines = consistencyTenseLines({ lifeEnded: true, places, },);
 ```
 */
export function consistencyTenseLines(
  {
    lifeEnded,
    subjectNames,
    places,
  }: {
    readonly lifeEnded: boolean;
    readonly subjectNames?: readonly string[];
    readonly places: readonly {
      readonly sliceIndex: number;
      readonly english: ConsistencyEnglish;
    }[];
  },
): readonly string[] {
  return tenseCandidateLines({
    groups: tenseCandidateGroups({
      lifeEnded,
      ...((subjectNames === undefined) ? {} : { subjectNames, }),
      texts: shownEnglishOf({ places, },),
    },),
  },);
}

//endregion Consistency tense
