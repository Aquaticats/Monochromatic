import {
  choiceTallyOf,
  type ConsistencyChoiceVoice,
  runOffChoiceFinalistsOf,
} from './consistency-choice.ts';
import { decidingQuorumOf, } from './consistency-deciding-quorum.ts';
import { compareCodePoints, } from '@monochromatic-dev/translation-repair-markup/ts';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { wordingKey, } from '@monochromatic-dev/translation-repair-translate-floor/ts';

//region Consistency carry
// CARRYING THE CHOSEN RENDERING TO A RECEIVING PLACE IS A CHOICE AMONG TEXTS.
// The place's own paragraph of English (the current text), the chosen words
// moved into it by code where that is possible (the splice), and each writer's
// rewrite of the paragraph are the candidates. They are offered to the bench
// under neutral labels in an order that depends on the text and not on who
// wrote it, so no position and no author is favoured, and a strict majority of
// the usable choices decides. This module holds what is decided without a
// model: which candidates are distinct, in what order they are shown, and what
// the choices come to. Which block of the English a span lies in is
// `consistency-carry-standing.ts`.

/**
 One text a receiving place's carry chooses among, before it is labelled.
 */
type CarryOfferDraft = {
  /**
   Where the text came from.
   */
  readonly kind: 'current' | 'splice' | 'writer';

  /**
   Seats whose reply was exactly this text or its layout.
   */
  readonly writers: readonly string[];

  /**
   The paragraph.
   */
  readonly text: string;
};

/**
 One text a receiving place's carry chooses among.

 @example
 ```ts
 const offer: CarryOffer = { label: 'CANDIDATE 1', kind: 'current', writers: [], text: 'The cat naps.', };
 ```
 */
export type CarryOffer = CarryOfferDraft & {
  /**
   Label the choice sheet gives it, counted from one in sheet order.
   */
  readonly label: string;
};

/**
 The distinct texts a receiving place's carry chooses among, in the order the
 sheet shows them and under their labels.

 Two texts that publish the same page are one candidate (`wordingKey`), so a
 writer that returned the paragraph unchanged, or the splice that came out as a
 writer's text, does not split the stake of one wording. The current text wins
 the name of the candidate it joins, then the splice, then the first writer.
 The order is by a digest of the text, so it is the same on a resume and favours
 neither the current text nor any seat.

 @param current - receiving paragraph as the English has it

 @param splice - paragraph with the chosen words moved in by code, as a
 list of zero or one: none where that is not possible or a floor refused it

 @param written - each writer's paragraph that passed the floors, in roster order

 @param lineStructured - whether the line-structure rule governs the slice

 @returns Distinct candidates, current text always among them

 @example
 ```ts
 const offers = carryOffersOf({ current, splice: [], written: [], lineStructured: false, },);
 ```
 */
export function carryOffersOf(
  {
    current,
    splice,
    written,
    lineStructured,
  }: {
    readonly current: string;
    readonly splice: readonly string[];
    readonly written: readonly {
      readonly seat: string;
      readonly text: string;
    }[];
    readonly lineStructured: boolean;
  },
): readonly CarryOffer[] {
  /**
   The splice as a draft, none where there is no splice.
   */
  const spliced = splice.map(function draftOfSplice(text,): CarryOfferDraft {
    return {
      kind: 'splice',
      writers: [],
      text,
    };
  },);
  /**
   Every text on offer before duplicates join, in the order they take a name.
   */
  const drafts: readonly CarryOfferDraft[] = [
    {
      kind: 'current',
      writers: [],
      text: current,
    },
    ...spliced,
    ...written.map(function draftOf({
      seat,
      text,
    },): CarryOfferDraft {
      return {
        kind: 'writer',
        writers: [seat,],
        text,
      };
    },),
  ];
  /**
   Every draft with the page its text publishes.
   */
  const keyed = drafts.map(function keyOf(draft,): {
    readonly draft: CarryOfferDraft;
    readonly key: string;
  } {
    return {
      draft,
      key: wordingKey({
        text: draft.text,
        lineStructured,
      },),
    };
  },);
  /**
   The first draft of each page, which gives the candidate its name, in order of first appearance.
   */
  const firsts = keyed.filter(function isFirstOfPage(
    entry,
    at,
  ): boolean {
    return keyed.findIndex(function samePage(other,): boolean {
      return other.key === entry.key;
    },) === at;
  },);
  return firsts
    .map(function joinedWithLater(first,): CarryOfferDraft {
      return {
        ...first.draft,
        writers: keyed
          .filter(function onSamePage(other,): boolean {
            return other.key === first.key;
          },)
          .flatMap(function writersOf({ draft, },): readonly string[] {
            return draft.writers;
          },),
      };
    },)
    .toSorted(function byDigest(
      left,
      right,
    ): number {
      return compareCodePoints({
        left: hashContent({ content: left.text, },),
        right: hashContent({ content: right.text, },),
      },);
    },)
    .map(function labelled(
      draft,
      position,
    ): CarryOffer {
      return {
        ...draft,
        label: `CANDIDATE ${String(position + 1,)}`,
      };
    },);
}

/**
 What the choices among a place's candidates decided.

 @example
 ```ts
 const choice: CarryChoice = { kind: 'none', reason: 'no-majority', };
 ```
 */
export type CarryChoice =
  | {
    /**
     A strict majority of the usable choices named a candidate.
     */
    readonly kind: 'chosen';

    /**
     Label of the candidate.
     */
    readonly label: string;
  }
    | {
    /**
     No candidate won, so the place's English stands.
     */
      readonly kind: 'none';

    /**
     Why: the usable choices missed the quorum, or no label has more than half.
     */
      readonly reason: 'no-quorum' | 'no-majority';
    };

/**
 Counts the choices among a place's candidates.

 @param voices - every heard choice with its seat

 @param labels - labels the sheet showed

 @param quorum - voices the gather closed on, which the usable choices must reach and never fewer than two (a quorum of
 one is raised to two)

 @returns How many choices counted and what they decided

 @example
 ```ts
 const { usable, choice, } = carryChoiceOf({ voices, labels: ['CANDIDATE 1', 'CANDIDATE 2',], quorum: 2, },);
 ```
 */
export function carryChoiceOf(
  {
    voices,
    labels,
    quorum,
  }: {
    readonly voices: readonly ConsistencyChoiceVoice[];
    readonly labels: readonly string[];
    readonly quorum: number;
  },
): {
  readonly usable: number;
  readonly choice: CarryChoice;
} {
  /**
   How many choices counted, and the candidate more than half of them named.
   */
  const {
    usable,
    chosen,
  } = choiceTallyOf({
    voices,
    labels,
  },);
  // A quorum of one (a seated bench of one or two seats) is raised to two: one judge alone never decides.
  if ((usable === 0) || (usable < decidingQuorumOf({ quorum, },))) {
    return {
      usable,
      choice: {
        kind: 'none',
        reason: 'no-quorum',
      },
    };
  }
  /**
   The candidate a strict majority named, if one did.
   */
  const [label,] = chosen;
  return {
    usable,
    choice: (label === undefined)
      ? {
        kind: 'none',
        reason: 'no-majority',
      }
      : {
        kind: 'chosen',
        label,
      },
  };
}

/**
 The candidates a carry choice runs off: when the choices reached the quorum and no candidate has a strict majority, the
 finalists the first round names (the leaders, and the candidates next behind a lone leader), as the first round runs off among
 its places (the lead's adoption of 2026-10-15 of the owner's rule of 2026-10-11 for this ask, extended on 2026-10-18 to a lone
 leader, both open to the owner's veto). A choice that missed its quorum or was won is not run off.

 @param voices - every heard choice with its seat

 @param labels - labels the sheet showed

 @param quorum - voices the gather closed on, which the usable choices must reach

 @returns Finalists in sheet order, none when the choice is not run off

 @example
 ```ts
 const finalists = carryRunOffFinalistsOf({ voices, labels: ['CANDIDATE 1', 'CANDIDATE 2',], quorum: 2, },);
 ```
 */
export function carryRunOffFinalistsOf(
  {
    voices,
    labels,
    quorum,
  }: {
    readonly voices: readonly ConsistencyChoiceVoice[];
    readonly labels: readonly string[];
    readonly quorum: number;
  },
): readonly string[] {
  /**
   What the choices decided on their own.
   */
  const { choice, } = carryChoiceOf({
    voices,
    labels,
    quorum,
  },);
  return ((choice.kind === 'none') && (choice.reason === 'no-majority'))
    ? runOffChoiceFinalistsOf({
      voices,
      labels,
    },)
    : [];
}

//endregion Consistency carry
