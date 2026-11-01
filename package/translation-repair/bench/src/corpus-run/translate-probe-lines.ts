import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Translate probe lines
// What the translate probe prints, as text and nothing else: the line naming
// the section it chose, the line saying how many slices it will probe, and the
// lines for each slice and what the translators answered.

/**
 Ratio below which a section counts as barely translated for this probe.

 Only picks which section to demonstrate on. Nothing downstream reads it, and
 choosing a threshold for production is exactly the partial-translation question
 the owner rejected, so it is deliberately local to this file.
 */
const SPARSE_RATIO = 0.25;

/**
 Decimal places a coverage ratio prints with.
 */
const RATIO_DIGITS = 3;

/**
 Slices translated in one probe run.

 The first attempt asked for a whole 4641-character section in one call and
 lost two voices of three: one timed out at six minutes, one returned
 schema-invalid output. Editors in this pipeline work on regions of median 75
 characters and at most 562, so that call was eight times larger than anything
 the stage has ever been asked for. A translate stage would run at SLICE
 granularity like every other stage, and this now does.
 */
export const PROBE_SLICES = 3;

/**
 A count and its noun, the noun chosen by the count.

 @param count - how many

 @param one - noun for a count of one

 @param many - noun for any other count

 @returns The count, a space and the noun

 @example
 ```ts
 const text = counted({ count: 1, one: 'block', many: 'blocks', },);
 ```
 */
function counted(
  {
    count,
    one,
    many,
  }: {
    readonly count: number;
    readonly one: string;
    readonly many: string;
  },
): string {
  return `${String(count,)} ${
    wordForCount({
      count,
      one,
      many,
    },)
  }`;
}

/**
 The line naming the section the probe chose and how much of it the
 translation covers.

 @param entryId - entry the section belongs to

 @param sourceBlocks - blocks of the original

 @param sourceChars - characters of the original

 @param targetBlocks - blocks of the translation

 @param targetChars - characters of the translation

 @param ratio - target blocks divided by source blocks

 @returns One line

 @example
 ```ts
 console.log(sectionLine({ entryId: 'XingZ60', sourceBlocks: 3, sourceChars: 15, targetBlocks: 2, targetChars: 18, ratio: 2 / 3, },),);
 ```
 */
export function sectionLine(
  {
    entryId,
    sourceBlocks,
    sourceChars,
    targetBlocks,
    targetChars,
    ratio,
  }: {
    readonly entryId: string;
    readonly sourceBlocks: number;
    readonly sourceChars: number;
    readonly targetBlocks: number;
    readonly targetChars: number;
    readonly ratio: number;
  },
): string {
  return `TRANSLATE ${entryId}: source ${
    counted({
      count: sourceBlocks,
      one: 'block',
      many: 'blocks',
    },)
  } / ${
    counted({
      count: sourceChars,
      one: 'char',
      many: 'chars',
    },)
  }, target ${
    counted({
      count: targetBlocks,
      one: 'block',
      many: 'blocks',
    },)
  } / ${
    counted({
      count: targetChars,
      one: 'char',
      many: 'chars',
    },)
  }, coverage ${ratio.toFixed(RATIO_DIGITS,)}${(ratio < SPARSE_RATIO) ? ' (barely translated)' : ''}`;
}

/**
 The line saying how many slices the section cuts into and how many the
 probe asks about.

 @param sliceCount - slices the section subdivides into

 @returns One line, naming the slices the probe asks about, which are the
 section's own where it cuts into fewer than {@link PROBE_SLICES}

 @example
 ```ts
 console.log(slicesLine({ sliceCount: 7, },),);
 ```
 */
export function slicesLine({ sliceCount, }: { readonly sliceCount: number; },): string {
  return `TRANSLATE section subdivides into ${
    counted({
      count: sliceCount,
      one: 'slice',
      many: 'slices',
    },)
  }; probing the first ${String(Math.min(
    sliceCount,
    PROBE_SLICES,
  ),)}`;
}

/**
 The heading printed before each probed slice.

 @param sourceChars - characters of the slice's original

 @param targetChars - characters of the slice's translation

 @returns The heading, led by a blank line

 @example
 ```ts
 console.log(sliceHeading({ sourceChars: 15, targetChars: 18, },),);
 ```
 */
export function sliceHeading(
  {
    sourceChars,
    targetChars,
  }: {
    readonly sourceChars: number;
    readonly targetChars: number;
  },
): string {
  return `\n--- slice: ${String(sourceChars,)} source ${
    wordForCount({
      count: sourceChars,
      one: 'char',
      many: 'chars',
    },)
  }, ${String(targetChars,)} target ${
    wordForCount({
      count: targetChars,
      one: 'char',
      many: 'chars',
    },)
  } ---`;
}

/**
 The lines printed for what the translators answered about one slice.

 @param heard - voices heard

 @param asked - models the roster held

 @param translations - each heard voice's model and rendered English

 @param findings - roster degradation findings

 @returns The count line, one line per voice, one line per finding

 @example
 ```ts
 const lines = heardLines({ heard: 1, asked: 2, translations: [], findings: [], },);
 ```
 */
export function heardLines(
  {
    heard,
    asked,
    translations,
    findings,
  }: {
    readonly heard: number;
    readonly asked: number;
    readonly translations: readonly {
      readonly modelId: string;
      readonly translation: string;
    }[];
    readonly findings: readonly string[];
  },
): readonly string[] {
  return [
    `HEARD ${String(heard,)}/${String(asked,)}`,
    ...translations.map(function voiceLine({
      modelId,
      translation,
    },): string {
      return `  ${modelId}: ${translation}`;
    },),
    ...findings.map(function findingLine(finding,): string {
      return `  finding: ${finding}`;
    },),
  ];
}

//endregion Translate probe lines
