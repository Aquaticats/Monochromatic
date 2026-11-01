import {
  type CorpusPin,
  isMissingCorpusObject,
  readCorpusFile,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { deriveOmissionSeeds, } from '@monochromatic-dev/translation-repair-repair/ts';
import { splitFrontMatter, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { BenchmarkEntry, } from '@monochromatic-dev/translation-repair-archive/ts';
import { bandOf, } from '@monochromatic-dev/translation-repair-insertion/ts';

//region Recall benchmark entry
// ONE CORPUS ENTRY, SEEDED OR SET ASIDE WITH THE REASON.
//
// SPLIT OUT OF `recall-benchmark.ts` when that file became wiring only. What
// is read comes through the pin the caller hands in, so a case reads an
// invented clone and never the pinned corpus.

/**
 Seeds planted per entry. Each is a whole deleted sentence, so a handful per
 document gives a usable denominator without turning the translation into
 something no reviewer would call a translation.
 */
export const SEEDS_PER_ENTRY = 3;

/**
 Outcome of trying to seed one corpus id. A discriminated result rather than
 a nullish return, because "this entry cannot be seeded" is an ordinary,
 expected answer that the caller must branch on, not an absence.
 */
export type RecallSeedOutcome =
  | {
    readonly kind: 'seeded';
    readonly entry: BenchmarkEntry;
    readonly band: ReturnType<typeof bandOf>;
  }
    | {
      readonly kind: 'skipped';
      readonly reason: string;
    };

/**
 Builds the seeded benchmark entry for one corpus id, reporting why when it
 cannot be seeded.

 @param id - corpus person id

 @param sizer - shared encoder measuring source bytes for banding

 @param pin - corpus clone and commit the pages are read at

 @returns Seeded entry with its band, or the reason it was skipped

 @example
 ```ts
 const outcome = await seedRecallEntry({ id: 'Whiskers', sizer, pin, },);
 ```
 */
export async function seedRecallEntry(
  {
    id,
    sizer,
    pin,
  }: {
    readonly id: string;
    readonly sizer: TextEncoder;
    readonly pin: CorpusPin;
  },
): Promise<RecallSeedOutcome> {
  try {
    /**
     Original zh page, front matter included: the repair loop reads it whole.
     */
    const sourceText = await readCorpusFile({
      pin,
      relPath: `people/${id}/page.md`,
    },);

    /**
     Clean en translation, the text seeds are planted into.
     */
    const targetText = await readCorpusFile({
      pin,
      relPath: `people/${id}/page.en.md`,
    },);

    /**
     Body only. Seeds must come from prose, never from front matter, whose
     deletion would break identity rather than plant an omission.
     */
    const { body, } = splitFrontMatter({ text: targetText, },);

    /**
     Deletions to plant, longest sentences first.
     */
    const seeds = deriveOmissionSeeds({
      text: body,
      maxSeeds: SEEDS_PER_ENTRY,
    },);
    if (seeds.length === 0) {
      return {
        kind: 'skipped',
        reason: 'no seedable sentence',
      };
    }

    return {
      kind: 'seeded',
      band: bandOf({
        sourceBytes: sizer.encode(sourceText,)
          .length,
      },),
      entry: {
        entryId: id,
        sourceText,
        targetText,
        seeds,
      },
    };
  }
  catch (error) {
    // A missing side is an expected non-pair; anything else is a real fault,
    // and since 2026-08-26 the error says which it was.
    if (!isMissingCorpusObject(error,))
      throw error;
    return {
      kind: 'skipped',
      reason: 'incomplete pair',
    };
  }
}

//endregion Recall benchmark entry
