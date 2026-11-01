import { compareCodePoints, } from '@monochromatic-dev/translation-repair-markup/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type CandidateRates,
  type MeasureVerdict,
  MINIMUM_PRODUCED_READINGS,
  type SeatedBand,
} from './reader-measure-score.ts';

//region Reader measure report
// WHAT THE PERSON WHO RAN `reader-measure` WITH A KEY READS: the model and
// route measured, the entries the matched pictures came from, the seated band,
// the candidate's rates beside the floor and ceiling they are held to, the
// verdict on each clause, and the one edit a pass calls for, so the next step
// is never a question and a verdict is checked from the printout. Raw output,
// deliberately: this is text a person reads, not pipeline logging.
//
// THE NEXT LINE NAMES THE INPUT THAT LIMITED THE RESULT (ledger B800). A
// candidate that produced too few readings was limited by one of three inputs,
// and only one of them is cured by a larger `--sample`: the trusted records
// when every picture was tried and few matched one, the sample when pictures
// were left untried, and the candidate's own answers when pictures matched and
// few readings came back. The runs of 2026-10-21 (UTC) had 3 trusted records
// match 3 of 291 pictures and were told to ask again with a larger sample.

/**
 Hold the card carries until this measurement passes.
 */
const HOLD = 'openrouter-pictures-unmeasured';

/**
 Name an entry takes for a picture that sits directly in the pictures directory
 and so belongs to no entry directory.
 */
const NO_ENTRY_DIRECTORY = '(no entry directory)';

/**
 Two decimals of a share.

 @param value - share between zero and one

 @returns The share with two decimals

 @example
 ```ts
 twoPlaces({ value: 0.8, },); // '0.80'
 ```
 */
function twoPlaces({ value, }: { readonly value: number; },): string {
  return value.toFixed(2,);
}

/**
 An entry with the number of matched pictures that came from it.
 */
type EntryCount = {
  readonly entry: string;
  readonly count: number;
};

/**
 Entry a picture belongs to: the first directory of its path under the pictures
 directory, which is how the corpus lays out a person's pictures.

 @param assetName - the picture's path under the pictures directory

 @returns The first directory of the path, or a stand-in name for a picture
 that sits directly in the pictures directory

 @example
 ```ts
 entryOf({ assetName: 'Mittens/photos/photo1.webp', },); // 'Mittens'
 ```
 */
function entryOf({ assetName, }: { readonly assetName: string; },): string {
  /**
   The path's directories and its file name.
   */
  const [first = assetName, ...rest] = assetName.split('/',);
  return (rest.length === 0) ? NO_ENTRY_DIRECTORY : first;
}

/**
 The line that counts the matched pictures by the entry they came from.

 @param matchedAssets - paths of the pictures matched to a trusted record

 @returns The line, most pictures first and ties in code point order

 @example
 ```ts
 const line = entriesLine({ matchedAssets: ['Mittens/photos/photo1.webp',], },); // 'ENTRIES: 1 matched picture came from 1 entry: Mittens 1.'
 ```
 */
function entriesLine({ matchedAssets, }: { readonly matchedAssets: readonly string[]; },): string {
  if (matchedAssets.length === 0)
    return 'ENTRIES: no picture matched a trusted record.';

  /**
   Pictures per entry.
   */
  const counts = new Map<string, number>();
  for (const assetName of matchedAssets) {
    /**
     Entry this picture came from.
     */
    const entry = entryOf({ assetName, });
    counts.set(
      entry,
      (counts.get(entry,) ?? 0) + 1,
    );
  }

  /**
   Each entry with its count, most pictures first.
   */
  const ordered: readonly EntryCount[] = [...counts.keys(),]
    .map(function counted(entry,): EntryCount {
      return {
        entry,
        count: counts.get(entry,) ?? 0,
      };
    },)
    .toSorted(function mostFirst(
      left,
      right,
    ): number {
      /**
       How many more pictures the right entry has.
       */
      const difference = right.count - left.count;
      if (difference !== 0)
        return difference;
      return compareCodePoints({
        left: left.entry,
        right: right.entry,
      },);
    },);
  return `ENTRIES: ${String(matchedAssets.length,)} matched ${
    wordForCount({
      count: matchedAssets.length,
      one: 'picture',
      many: 'pictures',
    },)
  } came from ${String(ordered.length,)} ${
    wordForCount({
      count: ordered.length,
      one: 'entry',
      many: 'entries',
    },)
  }: ${
    ordered
      .map(function named(one,): string {
        return `${one.entry} ${String(one.count,)}`;
      },)
      .join(', ',)
  }.`;
}

/**
 The line that sets the candidate's own rates beside the band they are held to.

 @param candidate - how the candidate's readings fared

 @param band - how the seated readers corroborate one another

 @returns The line, or one saying there is nothing to compare when no reading was produced

 @example
 ```ts
 const line = candidateLine({ candidate, band, },);
 ```
 */
function candidateLine(
  {
    candidate,
    band,
  }: {
    readonly candidate: CandidateRates;
    readonly band: SeatedBand;
  },
): string {
  if (candidate.produced === 0)
    return 'CANDIDATE: no readings were produced, so there are no rates to hold to the band.';
  return `CANDIDATE: ${String(candidate.produced,)} ${
    wordForCount({
      count: candidate.produced,
      one: 'reading',
      many: 'readings',
    },)
  }; corroborated by all ${twoPlaces({ value: candidate.corroboratedByAll, },)} (floor ${
    twoPlaces({ value: band.floorCorroboratedByAll, },)
  }), corroborated by none ${twoPlaces({ value: candidate.corroboratedByNone, },)} (ceiling ${
    twoPlaces({ value: band.ceilingCorroboratedByNone, },)
  }).`;
}

/**
 The advice that follows a candidate whose readings were too few for a verdict,
 which names the input that limited the count.

 @param matched - pictures matched to a trusted record and asked

 @param sample - how many matched pictures the run was allowed to ask

 @param trustedRecords - trusted records the run read

 @param pictures - pictures found under the pictures directory

 @returns What to do next, keeping the hold

 @example
 ```ts
 const advice = adviceForFewReadings({ matched: 3, sample: 9, trustedRecords: 3, pictures: 291, },);
 ```
 */
function adviceForFewReadings(
  {
    matched,
    sample,
    trustedRecords,
    pictures,
  }: {
    readonly matched: number;
    readonly sample: number;
    readonly trustedRecords: number;
    readonly pictures: number;
  },
): string {
  /**
   Whether every picture was tried: the loop stops short of its sample only when
   the pictures run out.
   */
  const everyPictureTried = matched < sample;
  if (matched < MINIMUM_PRODUCED_READINGS) {
    if (everyPictureTried) {
      return `NEXT: keep the hold '${HOLD}'; more trusted records are needed: the ${String(trustedRecords,)} trusted ${
        wordForCount({
          count: trustedRecords,
          one: 'record',
          many: 'records',
        },)
      } matched ${String(matched,)} of the ${String(pictures,)} ${
        wordForCount({
          count: pictures,
          one: 'picture',
          many: 'pictures',
        },)
      }, fewer than the ${String(MINIMUM_PRODUCED_READINGS,)} a verdict needs, and a larger sample would add none`
        + ` because every picture was already tried. Add --trusted directories naming runs directories whose`
        + ` slice-cache/<entry>/picture.<key>.json files belong to other entries than the ones the ENTRIES line counts`
        + ` (older single-entry runs directories hold them, not only the recent corpus-pass ones), or point --pictures at the`
        + ' people directory that holds the pictures those records read, then ask again.';
    }
    return `NEXT: keep the hold '${HOLD}'; the sample of ${String(sample,)} ended the count at ${String(matched,)} matched ${
      wordForCount({
        count: matched,
        one: 'picture',
        many: 'pictures',
      },)
    }, fewer than the ${String(MINIMUM_PRODUCED_READINGS,)} a verdict needs, and pictures were left untried:`
      + ` ask again with a larger --sample (${String(MINIMUM_PRODUCED_READINGS,)} or more).`;
  }

  /**
   What to do about the pictures: a larger sample only helps where pictures were left.
   */
  const closing = everyPictureTried
    ? 'then ask again.'
    : 'then ask again, with a larger --sample if the reasons are about the pictures and not the candidate.';
  return `NEXT: keep the hold '${HOLD}'; ${String(matched,)} ${
    wordForCount({
      count: matched,
      one: 'picture',
      many: 'pictures',
    },)
  } matched but the candidate gave few readings: `
    + `read the per-picture reasons the run printed, ${closing}`;
}

/**
 Lines of the verdict and what follows from it.

 @param modelId - candidate measured

 @param verdict - what the measurement decides

 @param matched - pictures matched to a trusted record and asked

 @param sample - how many matched pictures the run was allowed to ask

 @param trustedRecords - trusted records the run read

 @param pictures - pictures found under the pictures directory

 @returns The verdict lines

 @example
 ```ts
 const lines = verdictLines({ modelId, verdict, matched: 3, sample: 9, trustedRecords: 3, pictures: 291, },);
 ```
 */
function verdictLines(
  {
    modelId,
    verdict,
    matched,
    sample,
    trustedRecords,
    pictures,
  }: {
    readonly modelId: string;
    readonly verdict: MeasureVerdict;
    readonly matched: number;
    readonly sample: number;
    readonly trustedRecords: number;
    readonly pictures: number;
  },
): readonly string[] {
  if (verdict.kind === 'meets-both-clauses') {
    return [
      'RESULT: both clauses met.',
      `NEXT: remove '${HOLD}' from the holds of ${modelId} in shared-contract/src/model-cards.ts, rebuild, and run the package checks;`
        + ' that seats it as a picture reader on OpenRouter (zero data retention on every request).',
    ];
  }
  if (verdict.kind === 'inconclusive') {
    return [
      `RESULT: not measured: ${String(verdict.produced,)} of the ${String(MINIMUM_PRODUCED_READINGS,)} ${
        wordForCount({
          count: MINIMUM_PRODUCED_READINGS,
          one: 'reading',
          many: 'readings',
        },)
      } a verdict needs were produced.`,
      adviceForFewReadings({
        matched,
        sample,
        trustedRecords,
        pictures,
      },),
    ];
  }
  return [
    ...verdict.clauses
      .map(function failed(clause,): string {
        return `RESULT: ${clause} clause failed.`;
      },),
    `NEXT: keep the hold '${HOLD}' on ${modelId}.`,
  ];
}

/**
 Lines the measurement prints.

 @param modelId - candidate measured

 @param matchedAssets - paths of the pictures matched to a trusted record and asked, which name the entries they
 came from

 @param sample - how many matched pictures the run was allowed to ask

 @param trustedRecords - trusted records the run read

 @param pictures - pictures found under the pictures directory

 @param band - how the seated readers corroborate one another

 @param candidate - how the candidate's readings fared

 @param verdict - what the measurement decides

 @returns The report, one line per element

 @example
 ```ts
 console.log(renderMeasureReport({ modelId, matchedAssets, sample, trustedRecords, pictures, band, candidate, verdict, },).join('\n',),);
 ```
 */
export function renderMeasureReport(
  {
    modelId,
    matchedAssets,
    sample,
    trustedRecords,
    pictures,
    band,
    candidate,
    verdict,
  }: {
    readonly modelId: string;
    readonly matchedAssets: readonly string[];
    readonly sample: number;
    readonly trustedRecords: number;
    readonly pictures: number;
    readonly band: SeatedBand;
    readonly candidate: CandidateRates;
    readonly verdict: MeasureVerdict;
  },
): readonly string[] {
  return [
    `MEASURED: ${modelId} through OpenRouter (zero data retention), ${String(matchedAssets.length,)} ${
      wordForCount({
        count: matchedAssets.length,
        one: 'picture',
        many: 'pictures',
      },)
    } matched,`
      + ` ${String(candidate.produced,)} ${
      wordForCount({
        count: candidate.produced,
        one: 'reading',
        many: 'readings',
      },)
    } produced.`,
    entriesLine({ matchedAssets, },),
    `SEATED: ${String(band.readings,)} ${
      wordForCount({
        count: band.readings,
        one: 'reading',
        many: 'readings',
      },)
    }; seated floor ${twoPlaces({ value: band.floorCorroboratedByAll, },)}`
      + ` corroborated by all, seated ceiling ${twoPlaces({ value: band.ceilingCorroboratedByNone, },)} corroborated by none.`,
    candidateLine({
      candidate,
      band,
    },),
    ...(candidate.produced === 0 ? [] : [
      `OVERLAP: mean trigram overlap of the candidate's readings with the trusted readings ${
        twoPlaces({ value: candidate.meanOverlap, },)
      }.`,
    ]),
    ...verdictLines({
      modelId,
      verdict,
      matched: matchedAssets.length,
      sample,
      trustedRecords,
      pictures,
    },),
  ];
}

//endregion Reader measure report
