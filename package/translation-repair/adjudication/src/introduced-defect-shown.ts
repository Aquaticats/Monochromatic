import { withoutTranslatorsNotes, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { RepairRegion, } from './repair-region.ts';

//region Introduced-defect shown texts
// THE PROBER IS SHOWN THE BASELINE, THE WINDOW AND EVERY REGION WITHOUT THE
// NOTES THE ARCHIVE'S TRANSLATORS LEFT EACH OTHER, as the critic and the panel
// whose issues the edit was cut for were (`critic-prompt.ts`): no reader sees
// one, so an edit may neither be faulted for dropping a note nor excused for
// keeping one. Only what is SHOWN is cut; the regions the plan numbers are those
// the caller passed, and the original's own comments stay in view.

/**
 One region with the two texts the prober reads for it.

 @example
 ```ts
 const shown: ShownRegion = { region, before: 'The cat slept.', editorAfter: 'The cat napped.', };
 ```
 */
export type ShownRegion = {
  /**
   The region as the caller passed it, whose identity the plan keeps.
   */
  readonly region: RepairRegion;

  /**
   Text the edit replaced, without the notes between translators.
   */
  readonly before: string;

  /**
   Text the editor wrote, without the notes between translators.
   */
  readonly editorAfter: string;
};

/**
 The texts of a prober's sheet as the prober reads them.

 @example
 ```ts
 const shown: ProberShown = { baseline: 'The cat slept.', incumbent: '', regions: [], };
 ```
 */
export type ProberShown = {
  /**
   The baseline translation, without the notes between translators.
   */
  readonly baseline: string;

  /**
   The existing translation next door, without the notes between translators;
   empty when there is none or it holds nothing else.
   */
  readonly incumbent: string;

  /**
   Every region, in the order the caller passed them.
   */
  readonly regions: readonly ShownRegion[];
};

/**
 The texts whose comments are the original's own on a prober's sheet: the
 original, and the original next door.

 THE ONE DEFINITION BOTH SIDES OF THE SHEET READ. The sheet cuts the notes
 between translators from every translation text against these texts
 (`proberShownTexts`), and the quotes the probers copy out of it are screened
 against the same texts (`screenIntroducedDefects`); two spellings of the list
 could disagree about which comment is the original's.

 @param sourceText - original chunk text

 @param neighbouringSourceText - original next door, absent when the slice
 stands alone

 @returns The texts joined by line feeds, for `withoutTranslatorsNotes`

 @example
 ```ts
 const original = proberOriginalOf({ sourceText, },);
 ```
 */
export function proberOriginalOf(
  {
    sourceText,
    neighbouringSourceText,
  }: {
    readonly sourceText: string;
    readonly neighbouringSourceText?: string;
  },
): string {
  return [
    sourceText,
    neighbouringSourceText ?? '',
  ].join('\n',);
}

/**
 Cuts the notes between translators from every translation text a prober reads.

 @param sourceText - original chunk text, whose own comments stay

 @param neighbouringSourceText - original next door, whose own comments stay

 @param baselineText - translation as it stood before any replacement

 @param neighbouringIncumbentText - existing translation next door

 @param regions - replaced regions, in prompt numbering order

 @returns The baseline, the window and each region without the notes

 @example
 ```ts
 const shown = proberShownTexts({ sourceText, baselineText, regions, },);
 ```
 */
export function proberShownTexts(
  {
    sourceText,
    neighbouringSourceText,
    baselineText,
    neighbouringIncumbentText,
    regions,
  }: {
    readonly sourceText: string;
    readonly neighbouringSourceText?: string;
    readonly baselineText: string;
    readonly neighbouringIncumbentText?: string;
    readonly regions: readonly RepairRegion[];
  },
): ProberShown {
  /**
   Texts whose comments are the original's own.
   */
  const original = proberOriginalOf({
    sourceText,
    ...((neighbouringSourceText === undefined) ? {} : { neighbouringSourceText, }),
  },);
  return {
    baseline: withoutTranslatorsNotes({
      text: baselineText,
      original,
    },),
    incumbent: withoutTranslatorsNotes({
      text: neighbouringIncumbentText ?? '',
      original,
    },),
    regions: regions.map(function shownRegion(region,): ShownRegion {
      return {
        region,
        before: withoutTranslatorsNotes({
          text: region.before,
          original,
        },),
        editorAfter: withoutTranslatorsNotes({
          text: region.editorAfter,
          original,
        },),
      };
    },),
  };
}

//endregion Introduced-defect shown texts
