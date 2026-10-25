import {
  alterSharedNumber,
  type DamageAttempt,
  insertBorrowedSentence,
} from '@monochromatic-dev/translation-repair-repair/ts';
import {
  type EditorDamageKind,
  editorFidelityTrials,
} from '../editor-fidelity-trials.ts';
import {
  furthestFirst,
  MAX_REFERENCE_CHARS,
  MAX_SOURCE_CHARS,
  MIN_REFERENCE_CHARS,
} from '../editor-fidelity-pool-rule.ts';
import { buildReviewedFidelityReference, } from '../fidelity-reference-build.ts';
import type { FidelityReferenceSpec, } from '../fidelity-reference-model.ts';
import {
  joinedBySection,
  specForDamage,
} from './editor-fidelity-pool-spec.ts';
import { sheetCharsOf, } from './editor-fidelity-probe-read.ts';
import {
  type Section,
  sectionsOf,
} from './editor-fidelity-pool-sections.ts';
import { scoreableVerdict, } from './editor-fidelity-pool-verdict.ts';

//region Editor fidelity pool, one entry
// THE DRAW OF ONE ENTRY, applying the rule `editor-fidelity-pool-manifest.ts` states to the two pages of an entry: align
// their heading sections, keep the sections of a size the probe can sheet, and take the first section whose borrowed-sentence
// twin builds and passes the scoreability check for the insertion, and the first other section whose altered-number twin
// does for the alteration (else the insertion's own section). Nothing here reads the clone or asks a model.

/**
 What the draw of one entry took and why it left things out.

 @example
 ```ts
 const { specs, skipped, sheetChars, } = await drawPoolEntry({ entryId, zh, en, corpusSha, reviewedOn, },);
 ```
 */
export type DrawnEntry = {
  /**
   Specifications taken: one for each section, each holding the damages drawn in it.
   */
  readonly specs: readonly FidelityReferenceSpec[];

  /**
   One reason for each thing the draw left out, in scorecard-stable wording.
   */
  readonly skipped: readonly string[];

  /**
   Characters of the sheet of each trial taken.
   */
  readonly sheetChars: readonly number[];
};

/**
 One attempt at one damage in one section.
 */
type Attempt = {
  /**
   The specification when the damage was drawn and passed, none otherwise.
   */
  readonly drawn: readonly {
    readonly spec: FidelityReferenceSpec;
    readonly sheetChars: number
  }[];

  /**
   Reasons the attempt left something out.
   */
  readonly skipped: readonly string[];
};


/**
 What a search has found before it has tried a section.
 */
const NOTHING_YET: Attempt = {
  drawn: [],
  skipped: [],
};

/**
 An attempt that took nothing.

 @param reason - why

 @returns The attempt

 @example
 ```ts
 const attempt = skippedFor({ reason: 'no shared number', },);
 ```
 */
function skippedFor({ reason, }: { readonly reason: string; },): Attempt {
  return {
    drawn: [],
    skipped: [reason,],
  };
}

/**
 Builds the specification of one section and one damage and checks the trial it makes.

 @param entryId - the entry

 @param zh - original page

 @param en - archive page

 @param section - the section

 @param kind - the damage

 @param damage - the twin built

 @param donor - the section the donor sentence came from

 @param corpusSha - commit the pages were read at

 @param reviewedOn - date of the freeze

 @returns The attempt, drawn when the trial builds and passes

 @example
 ```ts
 const attempt = await checkedAttempt({ entryId, zh, en, section, kind, damage, donor, corpusSha, reviewedOn, },);
 ```
 */
async function checkedAttempt(
  {
    entryId,
    zh,
    en,
    section,
    kind,
    damage,
    donor,
    corpusSha,
    reviewedOn,
  }: {
    readonly entryId: string;
    readonly zh: string;
    readonly en: string;
    readonly section: Section;
    readonly kind: EditorDamageKind;
    readonly damage: Extract<DamageAttempt, { readonly kind: 'damaged'; }>;
    readonly donor: Section;
    readonly corpusSha: string;
    readonly reviewedOn: string;
  },
): Promise<Attempt> {
  /**
   The specification the damage is drawn into.
   */
  const spec = specForDamage({
    entryId,
    section,
    kind,
    damage,
    donor,
    corpusSha,
    reviewedOn,
  },);
  try {
    /**
     The trial the specification makes, built as the probe builds it.
     */
    const [trial,] = editorFidelityTrials({
      references: [
        buildReviewedFidelityReference({
          sourceFile: zh,
          archiveFile: en,
          spec,
        },),
      ],
      damageKinds: [kind,],
    },);
    if (trial === undefined)
      return skippedFor({ reason: 'no trial built', },);
    /**
     Why the trial cannot join the pool, empty when it can.
     */
    const verdict = await scoreableVerdict({ trial, },);
    if (verdict !== '')
      return skippedFor({ reason: `unscoreable: ${verdict}`, },);
    return {
      drawn: [
        {
          spec,
          sheetChars: sheetCharsOf({ trial, },),
        },
      ],
      skipped: [],
    };
  }
  catch (error) {
    return skippedFor({ reason: `replay failed: ${Error.isError(error,) ? error.name : 'unknown'}`, },);
  }
}

/**
 Tries one damage in one section.

 @param entryId - the entry

 @param zh - original page

 @param en - archive page

 @param sections - every section of the entry

 @param section - the section tried

 @param kind - the damage tried

 @param corpusSha - commit the pages were read at

 @param reviewedOn - date of the freeze

 @returns The attempt

 @example
 ```ts
 const attempt = await attemptIn({ entryId, zh, en, sections, section, kind, corpusSha, reviewedOn, },);
 ```
 */
function attemptIn(
  {
    entryId,
    zh,
    en,
    sections,
    section,
    kind,
    corpusSha,
    reviewedOn,
  }: {
    readonly entryId: string;
    readonly zh: string;
    readonly en: string;
    readonly sections: readonly Section[];
    readonly section: Section;
    readonly kind: EditorDamageKind;
    readonly corpusSha: string;
    readonly reviewedOn: string;
  },
): Promise<Attempt> {
  /**
   The other sections, furthest first, later first between two as far.
   */
  const donors = furthestFirst({
    indices: sections.map(function indexOf(other,): number {
      return other.index;
    },),
    from: section.index,
  },)
    .flatMap(function sectionAt(index,): readonly Section[] {
      return sections.filter(function isAt(other,): boolean {
        return other.index === index;
      },);
    },);
  if (kind === 'alteration') {
    /**
     The twin that changes a number both pages state.
     */
    const altered = alterSharedNumber({
      cleanText: section.referenceText,
      sourceText: section.sourceRaw,
    },);
    /**
     The furthest other section, the alteration's donor.
     */
    const [furthest,] = donors;
    if ((altered.kind !== 'damaged') || (furthest === undefined)) {
      return Promise.resolve(skippedFor({ reason: (altered.kind !== 'damaged') ? 'no shared number' : 'no donor span', },),);
    }
    return checkedAttempt({
      entryId,
      zh,
      en,
      section,
      kind,
      damage: altered,
      donor: furthest,
      corpusSha,
      reviewedOn,
    },);
  }
  /**
   The first donor whose sentence the twin can borrow, with the twin it gives.
   */
  const borrowed = donors
    .map(function borrowFrom(donor,): {
      readonly donor: Section;
      readonly damage: DamageAttempt
    } {
      return {
        donor,
        damage: insertBorrowedSentence({
          cleanText: section.referenceText,
          donorTexts: [donor.archiveRaw,],
        },),
      };
    },)
    .find(function builds(candidate,): boolean {
      return candidate.damage
        .kind
        === 'damaged';
    },);
  if ((borrowed === undefined) || (borrowed.damage
    .kind
    !== 'damaged'))
    return Promise.resolve(skippedFor({ reason: 'no donor for insertion', },),);
  return checkedAttempt({
    entryId,
    zh,
    en,
    section,
    kind,
    damage: borrowed.damage,
    donor: borrowed.donor,
    corpusSha,
    reviewedOn,
  },);
}

/**
 Tries a damage in each of the sections in turn and stops at the first that is drawn.

 @param sections - sections to try, in order

 @param attemptAt - tries the damage in one section

 @returns The first drawn attempt with every reason the earlier sections were left out, or every reason when none was

 @example
 ```ts
 const first = await firstDrawn({ sections, attemptAt, },);
 ```
 */
function firstDrawn(
  {
    sections,
    attemptAt,
  }: {
    readonly sections: readonly Section[];
    readonly attemptAt: (section: Section,) => Promise<Attempt>;
  },
): Promise<Attempt> {
  return sections.reduce(
    async function next(
      before: Promise<Attempt>,
      section: Section,
    ): Promise<Attempt> {
      /**
       What the earlier sections gave.
       */
      const earlier = await before;
      if (earlier.drawn
        .length
        > 0)
        return earlier;
      /**
       What this section gives.
       */
      const here = await attemptAt(section,);
      return {
        drawn: here.drawn,
        skipped: [
          ...earlier.skipped,
          ...here.skipped,
        ],
      };
    },
    Promise.resolve(NOTHING_YET,),
  );
}

/**
 Draws the insertion and the alteration of one entry.

 @param entryId - the entry, a plain id

 @param zh - original page

 @param en - archive page

 @param corpusSha - commit the pages were read at

 @param reviewedOn - date of the freeze

 @returns What was drawn, why things were left out, and the sheet sizes

 @example
 ```ts
 const { specs, } = await drawPoolEntry({ entryId: 'tabby', zh, en, corpusSha, reviewedOn: '2026-10-23', },);
 ```
 */
export async function drawPoolEntry(
  {
    entryId,
    zh,
    en,
    corpusSha,
    reviewedOn,
  }: {
    readonly entryId: string;
    readonly zh: string;
    readonly en: string;
    readonly corpusSha: string;
    readonly reviewedOn: string;
  },
): Promise<DrawnEntry> {
  /**
   Every heading section of the entry.
   */
  const sections = sectionsOf({
    zh,
    en,
  },);
  if (sections.length === 0) {
    return {
      specs: [],
      skipped: ['entry not equal shape',],
      sheetChars: [],
    };
  }
  /**
   Sections of a size the probe can sheet, with a nonempty original.
   */
  const eligible = sections.filter(function sheetable(section,): boolean {
    return (section.referenceText
      .length
      >= MIN_REFERENCE_CHARS)
      && (section.referenceText
        .length
        <= MAX_REFERENCE_CHARS)
      && (section.sourceRaw
        .length
        > 0)
      && (section.sourceRaw
        .length
        <= MAX_SOURCE_CHARS);
  },);
  /**
   The insertion: the first eligible section whose twin builds and passes.
   */
  const insertion = await firstDrawn({
    sections: eligible,
    attemptAt: function tryInsertion(section,): Promise<Attempt> {
      return attemptIn({
        entryId,
        zh,
        en,
        sections,
        section,
        kind: 'insertion',
        corpusSha,
        reviewedOn,
      },);
    },
  },);
  /**
   Section the insertion was drawn in, none when it was not drawn.
   */
  const insertionSection = insertion.drawn[0]
    ?.spec
    .id;
  /**
   The alteration: the first other eligible section whose twin builds and passes, else the insertion's own.
   */
  const alteration = await firstDrawn({
    sections: [
      ...eligible.filter(function other(section,): boolean {
        return `${entryId}-s${String(section.index,)}` !== insertionSection;
      },),
      ...eligible.filter(function own(section,): boolean {
        return `${entryId}-s${String(section.index,)}` === insertionSection;
      },),
    ],
    attemptAt: function tryAlteration(section,): Promise<Attempt> {
      return attemptIn({
        entryId,
        zh,
        en,
        sections,
        section,
        kind: 'alteration',
        corpusSha,
        reviewedOn,
      },);
    },
  },);
  /**
   Everything drawn, the damages of one section joined into one specification.
   */
  const drawn = [
    ...insertion.drawn,
    ...alteration.drawn,
  ];
  return {
    specs: joinedBySection({
      specs: drawn.map(function specOf({ spec, },): FidelityReferenceSpec {
        return spec;
      },),
    },),
    skipped: [
      ...insertion.skipped,
      ...alteration.skipped,
    ],
    sheetChars: drawn.map(function charsOf({ sheetChars, },): number {
      return sheetChars;
    },),
  };
}

//endregion Editor fidelity pool, one entry
