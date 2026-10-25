import { foldInvisibleVariants, } from '@monochromatic-dev/translation-repair-markup/ts';
import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { alignDocumentSections, } from '@monochromatic-dev/translation-repair-slice-reading/ts';

//region Editor fidelity pool sections
// THE HEADING SECTIONS OF AN ENTRY, pairing the original's with the archive's through the production aligner, which the draw
// takes its trials from. An entry whose pages the aligner reports a finding for has no sections here: it is not of equal
// shape, and the aligner would have to guess which section answers which.

/**
 One heading section of the entry, the pair of its two sides.

 @example
 ```ts
 const section: Section | undefined = sectionsOf({ zh, en, })[0];
 ```
 */
export type Section = {
  /**
   Position among the sections, from zero.
   */
  readonly index: number;

  /**
   Section of the original, byte for byte.
   */
  readonly sourceRaw: string;

  /**
   Section of the archive, byte for byte.
   */
  readonly archiveRaw: string;

  /**
   Archive range of the section.
   */
  readonly archive: {
    readonly startOffset: number;
    readonly endOffset: number
  };

  /**
   Original range of the section.
   */
  readonly source: {
    readonly startOffset: number;
    readonly endOffset: number
  };

  /**
   The archive section with invisible variants folded, which is the reference text.
   */
  readonly referenceText: string;
};

/**
 Splits the two pages into their heading sections, when they are of equal shape.

 @param zh - original page

 @param en - archive page

 @returns The sections, or none when the aligner reports a finding

 @example
 ```ts
 const sections = sectionsOf({ zh, en, },);
 ```
 */
export function sectionsOf({
  zh,
  en,
}: {
  readonly zh: string;
  readonly en: string
},): readonly Section[] {
  /**
   The aligner's pairs and findings.
   */
  const {
    pairs,
    findings,
  } = alignDocumentSections({
    source: parseDocument({ text: zh, },),
    target: parseDocument({ text: en, },),
  },);
  if (findings.length > 0)
    return [];
  return pairs.flatMap(function sectionOf(
    pair,
    index,
  ): readonly Section[] {
    if (pair.target
      .kind
      === 'insertion')
      return [];
    /**
     The archive section as the archive holds it.
     */
    const archiveRaw = en.slice(
      pair.target
        .startOffset,
      pair.target
        .endOffset,
    );
    return [
      {
        index,
        sourceRaw: zh.slice(
          pair.source
            .startOffset,
          pair.source
            .endOffset,
        ),
        archiveRaw,
        archive: {
          startOffset: pair.target
            .startOffset,
          endOffset: pair.target
            .endOffset,
        },
        source: {
          startOffset: pair.source
            .startOffset,
          endOffset: pair.source
            .endOffset,
        },
        referenceText: foldInvisibleVariants({ text: archiveRaw, },)
          .text,
      },
    ];
  },);
}

//endregion Editor fidelity pool sections
