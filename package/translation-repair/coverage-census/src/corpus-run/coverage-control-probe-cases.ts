import {
  type CorpusPin,
  listCorpusPeople,
  readCorpusFile,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { listCoverageCandidates, } from '@monochromatic-dev/translation-repair-archive/ts';
import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { askedAmong, } from '@monochromatic-dev/translation-repair-run/ts';
import type { CoverageControlCase, } from './coverage-control.ts';

//region Coverage control probe cases
// The passages the coverage control probe asks the roster about, walked out of
// the corpus entry by entry until enough are gathered.

/**
 Cases gathered before the control is called.

 MORE THAN THE CONTROL WILL USE, because a case is only damageable if the
 roster returns `carried` with evidence, which is not known until it is asked.
 Offering spares keeps a run from ending with nothing measured.
 */
const CASES_OFFERED = 8;

/**
 Collects passages to try, walking entries until enough are gathered.

 @param onlyIds - entries to restrict the walk to, empty for all

 @param pin - corpus clone and commit every read resolves against

 @returns Cases the control may try, in the order the corpus lists the
 entries and each entry's aligners list their passages

 @throws StatedRefusalError when an id named is no entry of the corpus at the
 pin, or when an entry the walk reaches lacks either page

 @example
 ```ts
 const cases = await gatherCases({ onlyIds, pin, },);
 ```
 */
export async function gatherCases(
  {
    onlyIds,
    pin,
  }: {
    readonly onlyIds: readonly string[];
    readonly pin: CorpusPin;
  },
): Promise<readonly CoverageControlCase[]> {
  /**
   Cases found so far.
   */
  const cases: CoverageControlCase[] = [];

  /**
   Entries to walk.
   */
  const entryIds = askedAmong({
    asked: onlyIds,
    known: await listCorpusPeople({ pin, },),
    source: '--only',
    within: 'the corpus at the pin',
  },);

  for (const entryId of entryIds) {
    if (cases.length >= CASES_OFFERED)
      break;

    /* oxlint-disable eslint/no-await-in-loop -- sequential by design: the walk stops as soon as enough cases are found, and reading every entry up front would read the whole corpus to use a few pages of it */
    /**
     Original side at the pin.
     */
    const sourceText = await readCorpusFile({
      pin,
      relPath: `people/${entryId}/page.md`,
    },);
    /* oxlint-enable eslint/no-await-in-loop */

    /* oxlint-disable eslint/no-await-in-loop -- paired with the original's read; the two sides of one entry are read together or not at all */
    /**
     Translation at the pin.
     */
    const targetText = await readCorpusFile({
      pin,
      relPath: `people/${entryId}/page.en.md`,
    },);
    /* oxlint-enable eslint/no-await-in-loop */

    /**
     Translation parsed once, since every case for this entry shares it.
     */
    const translation = parseDocument({ text: targetText, },);

    /**
     Passages this entry's aligners refuse to pair.
     */
    const candidates = listCoverageCandidates({
      source: parseDocument({ text: sourceText, },),
      target: translation,
    },);

    for (const candidate of candidates) {
      if (cases.length >= CASES_OFFERED)
        break;

      cases.push({
        where: `${entryId} ${
          (candidate.scale === 'section')
            ? `section ${String(candidate.sourceIndex,)}`
            : `pair ${String(candidate.pairIndex,)} block ${String(candidate.sourceIndex,)}`
        }`,
        sourcePassage: candidate.sourceText,
        translation,
      },);
    }
  }

  return cases;
}

//endregion Coverage control probe cases
