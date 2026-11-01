import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import { foldedLatinWords, } from '@monochromatic-dev/translation-repair-markup/ts';
import type { ReadonlyEntry, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Assembly content survival
// THE DAMAGE EVERY OTHER INSTRUMENT HERE MISSES.
//
// On `saurikissa` slice 4 the repair lane replaced a hundred and thirty-six
// words with twenty-one, deleting what she wore and what she collected from a
// memorial page. The assembled document lost seventy characters of four
// thousand four hundred and seventy-nine, because generic prose went in where
// the detail had been.
//
// So the length looked right, the structure was intact, nothing repeated,
// nothing was severed, no footnote broke, and the page had stopped being about
// a particular person. Length, repetition, severing and footnote checks all
// read clean on it. `doc/audit/the-damage-no-instrument-was-catching.md` carries
// the measurement.
//
// REPAIR LANE ONLY, and that is a correctness requirement rather than a
// shortcut. The translate lane writes a fresh rendering from the source, so it
// may carry the same detail in different English words, and a rare-word test
// cannot tell re-wording from deletion. The repair lane EDITS the incumbent, so
// a distinctive word it drops really is a detail removed or blurred.
//
// WORDS ARE LATIN LETTERS ONLY, folded (`foldedLatinWords`): punctuation and
// digits separate them, unlike `whitespaceTokensOf` in `assembly-repetition.ts`, which
// keeps punctuation on its token. That file compares passages, where `soon.`
// and `soon,` are different sentences; this one compares vocabulary, where
// they are the same word, and so are `Château`, `Cha\u{0302}teau` and
// `Chateau`. A local ASCII copy of the scan cut `Château` in two until ledger
// B18, and counted a combining mark as a letter towards a word's length.

/**
 Shortest word that can carry a specific.

 Below six letters the vocabulary is mostly function words and common verbs,
 which survive any rewrite and say nothing about whether meaning did.
 */
const MIN_DISTINCTIVE_LETTERS = 6;

/**
 Most times the archive may use a word for it to still count as distinctive.

 A word the archive leans on repeatedly is part of its register rather than one
 of its specifics, and register is exactly what a repair is allowed to change.
 */
const MAX_ARCHIVE_USES = 2;

/**
 What the archive's specifics did in the shipped document.

 CARRIES NO WORDING, like every finding in this package: a findings list
 travels into logs and artifacts where corpus text does not belong.

 @example
 ```ts
 const survival: ContentSurvival = { distinctive: 216, kept: 197, lost: 19, };
 ```
 */
export type ContentSurvival = {
  /**
   Distinctive words the archive carries.
   */
  readonly distinctive: number;

  /**
   How many of them the shipped document still carries.
   */
  readonly kept: number;

  /**
   How many it no longer carries.
   */
  readonly lost: number;
};

/**
 Words the archive uses rarely and at length, which carry its specifics.

 @param archiveText - translation as it stood before the pipeline ran

 @returns Distinctive words, each once

 @example
 ```ts
 const specifics = distinctiveWords({ archiveText, },);
 ```
 */
export function distinctiveWords(
  { archiveText, }: { readonly archiveText: string; },
): readonly string[] {
  /**
   Times the archive uses each word.
   */
  const uses = new Map<string, number>();
  for (const word of foldedLatinWords({ text: archiveText, },))
    uses.set(
      word,
      (uses.get(word,) ?? 0) + 1,
    );

  return [...uses,]
    .filter(function isDistinctive([word, count,]: ReadonlyEntry<string, number>,): boolean {
      return (word.length >= MIN_DISTINCTIVE_LETTERS) && (count <= MAX_ARCHIVE_USES);
    },)
    .map(function toWord([word,]: ReadonlyEntry<string, number>,): string {
      return word;
    },);
}

/**
 Measures how much of the archive's specific vocabulary the document still has.

 @param archiveText - translation as it stood before the pipeline ran

 @param shippedText - assembled document the repair lane produced

 @returns Counts, never wording

 @example
 ```ts
 const survival = measureContentSurvival({ archiveText, shippedText, },);
 ```
 */
export function measureContentSurvival(
  {
    archiveText,
    shippedText,
  }: {
    readonly archiveText: string;
    readonly shippedText: string;
  },
): ContentSurvival {
  /**
   Archive words carrying its specifics.
   */
  const distinctive = distinctiveWords({ archiveText, },);

  /**
   Every word the shipped document carries, for membership tests.
   */
  const shipped = new Set(foldedLatinWords({ text: shippedText, },),);

  /**
   How many specifics survived.
   */
  const kept = distinctive
    .filter(function survives(word,): boolean {
      return shipped.has(word,);
    },)
    .length;

  return {
    distinctive: distinctive.length,
    kept,
    lost: distinctive.length - kept,
  };
}

/**
 Distinctive archive words the assembled document no longer carries, for the
 log line that names where they were (ledger L12); the finding stays counts.

 @param archiveText - translation as it stood before the pipeline ran

 @param shippedText - assembled document the lane produced

 @returns Lost words in archive order, each once

 @example
 ```ts
 const lost = lostDistinctiveWords({ archiveText, shippedText, },);
 ```
 */
export function lostDistinctiveWords(
  {
    archiveText,
    shippedText,
  }: {
    readonly archiveText: string;
    readonly shippedText: string;
  },
): readonly string[] {
  /**
   Every word the shipped document carries, for membership tests.
   */
  const shipped = new Set(foldedLatinWords({ text: shippedText, },),);
  return distinctiveWords({ archiveText, },)
    .filter(function isLost(word,): boolean {
      return !shipped.has(word,);
    },);
}

/**
 Renders content survival as an assembly finding.

 ALWAYS ONE FINDING, rather than one only when a threshold is crossed. The
 healthy and damaged runs measured so far sit at 8% and 45% loss on one entry
 and 23% and 29% on another, which is not enough separation to place a
 threshold honestly. Reporting the rate every time leaves that judgement to
 whoever reads the run, and costs one line.

 @param archiveText - translation as it stood before the pipeline ran

 @param shippedText - assembled document the repair lane produced

 @returns One finding naming the counts

 @example
 ```ts
 const findings = contentSurvivalFindings({ archiveText, shippedText, },);
 ```
 */
export function contentSurvivalFindings(
  {
    archiveText,
    shippedText,
  }: {
    readonly archiveText: string;
    readonly shippedText: string;
  },
): readonly string[] {
  /**
   What the specifics did.
   */
  const survival = measureContentSurvival({
    archiveText,
    shippedText,
  },);
  if (survival.distinctive === 0)
    return [];

  return [
    `content-survival (${String(survival.kept,)} of ${
      String(survival.distinctive,)
    } distinctive archive ${
      wordForCount({
        count: survival.distinctive,
        one: 'word',
        many: 'words',
      },)
    } kept, ${String(survival.lost,)} lost)`,
  ];
}

//endregion Assembly content survival
