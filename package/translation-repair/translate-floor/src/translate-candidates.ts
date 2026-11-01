import {
  type Candidate,
  mergeProducers,
  producerModelIds,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import { rendersAsNothing, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { HeardVoice, } from '@monochromatic-dev/translation-repair-stage/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { TranslateReportWire, } from './translate-wire.ts';
import { requireFoldedVoices, } from './translator-answer-fold.ts';
import { wordingKey, } from './wording-key.ts';

//region Translate candidate assembly
// Turning heard translator voices into the slate judges compare, with the
// translation that was already there standing in it as one candidate.
//
// Ordered by ROSTER position rather than arrival, for the same reason the editor
// path is: voices come back in whatever order the provider answered, so arrival
// order would make the anonymized candidate numbering and the duplicate-collapse
// winner vary between runs over identical inputs.
//
// The incumbent is assembled FIRST, matching where `repairChunk` puts the
// unchanged text in its own two-candidate selection. That is assembly order
// rather than ballot order: `runTranslateStage` rotates the slate by a hash of
// the slice before judges see it, so the incumbent does not sit in the same
// position on every slice and its win rate is not also a measure of whatever
// position preference the judges have.

/**
 Where one candidate translation came from.

 @example
 ```ts
 const origin: TranslateOrigin = 'incumbent';
 ```
 */
export type TranslateOrigin =
  | 'incumbent'
    | 'fresh';

/**
 One candidate translation with the fact that decides whether the slice was
 kept or replaced.

 Origin rides on the VALUE rather than being inferred from the producer,
 because the two answer different questions: the producer says who must not
 judge this text, and the origin says whether shipping it changes the
 document. They come apart exactly when a model reproduces the incumbent.

 @example
 ```ts
 const value: TranslateCandidateValue = { text: 'The cat naps.', origin: 'fresh', };
 ```
 */
export type TranslateCandidateValue = {
  /**
   Text that ships when this candidate wins.
   */
  readonly text: string;

  /**
   Whether this text was already there.
   */
  readonly origin: TranslateOrigin;
};

/**
 One lane's text offered to a consolidation slate: what the repair lane or
 the translate lane would ship, put before the judges when the contest
 endorsed neither or the standing failed the deterministic rule (class
 forty, 2026-09-17).

 @example
 ```ts
 const offered: LaneText = { lane: 'repair', text: '> The cat naps.', };
 ```
 */
export type LaneText = {
  /**
   Which lane would ship it.
   */
  readonly lane: 'repair' | 'translate';

  /**
   Text that lane would ship, as it would ship.
   */
  readonly text: string;
};

/**
 Slate judges compare, plus what building it revealed.

 @example
 ```ts
 const { candidates, collapsed, } = buildTranslateCandidates({ voices, translatorModelIds, incumbentText, lineStructured: false, },);
 ```
 */
export type TranslateCandidateSet = {
  /**
   Distinct proposals, incumbent first when it has text, then any lane texts
   in lane order, then fresh translations in roster order.
   */
  readonly candidates: readonly Candidate<TranslateCandidateValue>[];

  /**
   Proposals collapsed into an earlier identical one; showing judges the same
   text twice would only split the ballot into a spurious tie.
   */
  readonly collapsed: number;

  /**
   Blank replies and incumbent matches, in scorecard-stable wording.
   */
  readonly findings: readonly string[];
};

/**
 Renders one lane's text as a candidate the slate carries beside the
 proposals: fresh, since the lane wrote it this run, and attributed to its
 lane rather than to any one model. Exported so the slate tests offer lanes
 exactly as the consolidation does rather than keeping a copy of this shape
 (ledger B116).

 @param laneText - text one lane would ship

 @returns Candidate carrying that lane's text

 @example
 ```ts
 const candidate = laneCandidate({ lane: 'repair', text: '> The cat naps.', },);
 ```
 */
export function laneCandidate(laneText: LaneText,): Candidate<TranslateCandidateValue> {
  return {
    producer: {
      kind: 'lane',
      lane: laneText.lane,
      matched: [],
    },
    value: {
      text: laneText.text,
      origin: 'fresh',
    },
    rendered: laneText.text,
  };
}

/**
 Assembles the candidate slate for one slice.

 ONE WORDING, ONE CANDIDATE. Two proposals that publish the same page share a
 candidate and a stake (`wordingKey`): trailing whitespace, prose quote style
 (ledger B24), and on a slice the line-structure rule does not govern, where a
 paragraph's soft line breaks fall (ledger B26). A model whose rendering is
 the incumbent's in all but those is reported as matching it.

 @param voices - heard translator replies in arrival order, each folded where
 its lane first held it (`foldTranslatorVoices`), so every check before this
 read the bytes that ship

 @param translatorModelIds - roster, fixing candidate order

 @param incumbentText - translation as it stands, blank when this slice has
 none

 @param lineStructured - whether the line-structure rule governs the slice,
 which keeps two renderings whose lines differ apart

 @param laneTexts - what the repair and translate lanes would ship, offered
 to a consolidation slate whose standing is neither contest-endorsed nor
 eligible (class forty, 2026-09-17); none on a translate slate

 @returns Distinct candidates, how many collapsed, and what that revealed

 @throws {@link import('./translator-answer-fold.ts').UnfoldedTranslationError}
 when a voice arrives with a translation the fold would still change, which
 every lane's intake rules out (ledger B112)

 @example
 ```ts
 const set = buildTranslateCandidates({ voices, translatorModelIds, incumbentText, lineStructured: false, },);
 ```
 */
export function buildTranslateCandidates(
  {
    voices,
    translatorModelIds,
    incumbentText,
    lineStructured,
    laneTexts = [],
  }: {
    readonly voices: readonly HeardVoice<TranslateReportWire>[];
    readonly translatorModelIds: readonly RosterModelId[];
    readonly incumbentText: string;
    readonly lineStructured: boolean;
    readonly laneTexts?: readonly LaneText[];
  },
): TranslateCandidateSet {
  /**
   Voices sorted by roster position so candidate numbering never depends on
   which model answered first.
   */
  const ordered = [...voices,].toSorted(function byRoster(
    left,
    right,
  ) {
    return translatorModelIds.indexOf(left.modelId,)
      - translatorModelIds.indexOf(right.modelId,);
  },);

  // FOLDED BEFORE THIS, NOT HERE (ledger B112). Each lane folds an answer
  // where it first holds it (`translator-answer-fold.ts`), so the publication
  // rule, the repair turn and the floor read the bytes the judges see and the
  // page gets. A voice the fold would still change means a caller skipped
  // that, and the checks before this read other bytes; it is refused.
  requireFoldedVoices({ voices: ordered, },);

  // A BACKSTOP SINCE THE WIRE GUARD TIGHTENED, asked of the FOLDED bytes. The
  // guard refuses a reply that shows nothing, so the ordinary path brings
  // none here; the filter stays because judges cannot be shown an empty
  // candidate whatever the route: it reads as a legitimate option to render
  // nothing, and a slice whose incumbent is also empty would then have a
  // whole ballot of nothing. Asked of the raw reply with `trim()`, it let a
  // zero-width space through, which the fold then emptied (ledger B40).
  /**
   Translators that answered with nothing to ship.
   */
  const blank = ordered.filter(function isBlank(voice,): boolean {
    return rendersAsNothing({ text: voice.value
      .translation, },);
  },);

  /**
   Translators that proposed text.
   */
  const folded = ordered.filter(function isUsable(voice,): boolean {
    return !rendersAsNothing({ text: voice.value
      .translation, },);
  },);

  /**
   Every proposal worth judging: the incumbent when it has text, then any
   lane texts, then each translator's own rendering.

   A blank incumbent is the case this lane exists for, a passage nobody has
   translated, and offering it as a candidate would put "leave it untranslated"
   on the ballot.

   LANE TEXTS SIT BETWEEN, so the incumbent's bytes win a collapse against a
   lane that reproduces them and a lane's bytes win one against a fresh
   proposal that reproduces the lane: what was already there stays what was
   already there, and what a lane would ship stays the lane's (class forty,
   2026-09-17).
   */
  const offered: readonly Candidate<TranslateCandidateValue>[] = [
    ...(rendersAsNothing({ text: incumbentText, },)
      ? []
      : [
        {
          producer: {
            kind: 'incumbent',
            matched: [],
          },
          value: {
            text: incumbentText,
            origin: 'incumbent',
          },
          rendered: incumbentText,
        } satisfies Candidate<TranslateCandidateValue>,
      ]),
    ...laneTexts
      .filter(function isUsable(laneText,): boolean {
        return !rendersAsNothing({ text: laneText.text, },);
      },)
      .map(laneCandidate,),
    ...folded.map(function toCandidate(voice,): Candidate<TranslateCandidateValue> {
      return {
        producer: {
          kind: 'model',
          modelId: voice.modelId,
        },
        value: {
          text: voice.value
            .translation,
          origin: 'fresh',
        },
        rendered: voice.value
          .translation,
      };
    },),
  ];

  /**
   Kept candidates by comparison key, merging the stakes of every duplicate
   into the survivor. First seen wins the text, which puts the incumbent's
   exact bytes on the ballot whenever a model matched it.
   */
  const byText = new Map<string, Candidate<TranslateCandidateValue>>();

  /**
   Models whose rendering turned out to be the incumbent's, which is the
   measurement telling a kept translation apart from an unexamined one.
   */
  const matchedIncumbent: RosterModelId[] = [];
  for (const candidate of offered) {
    /**
     Key this candidate competes under.
     */
    const key = wordingKey({
      text: candidate.rendered,
      lineStructured,
    },);

    /**
     Earlier candidate with the same key, when one exists.
     */
    const kept = byText.get(key,);
    if (kept === undefined) {
      byText.set(
        key,
        candidate,
      );
      continue;
    }
    if (kept.value
      .origin
      === 'incumbent')
      matchedIncumbent.push(...producerModelIds(candidate.producer,),);
    byText.set(
      key,
      {
        ...kept,
        producer: mergeProducers({
          left: kept.producer,
          right: candidate.producer,
        },),
      },
    );
  }

  return {
    candidates: [...byText.values(),],
    collapsed: offered.length - byText.size,
    findings: [
      ...blank.map(function toBlankFinding(voice,): string {
        return `translate-blank (${voice.modelId})`;
      },),
      ...matchedIncumbent.map(function toMatchFinding(modelId,): string {
        return `translate-matched-incumbent (${modelId})`;
      },),
    ],
  };
}

//endregion Translate candidate assembly
