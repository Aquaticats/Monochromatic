import {
  readingsCorroborate,
  trigramOverlap,
} from '@monochromatic-dev/translation-repair-markup/ts';

//region Reader measure score
// HOLDS A CANDIDATE PICTURE READER TO THE READERS THE PIPELINE ALREADY TRUSTS,
// by the rule written on 2026-09-08 for the three Gemma sizes (the openrouter
// planning record, "The three Gemma sizes against the seated readers on nine
// pictures"), now run through OpenRouter instead of Bedrock:
//
// 1. each reading the candidate produces is corroborated by every trusted
//    reading of the same picture at least as often as the seated reader that
//    was corroborated least often, and
// 2. it produces a reading no trusted reading corroborates no more often than
//    the seated reader that was uncorroborated most often.
//
// A CALL THAT PRODUCES NOTHING COUNTS FOR NEITHER CLAUSE, as in the first
// measurement: the clauses are about the readings a reader gives, and a reader
// that gives few is reported by how few (`produced`), with a verdict only at
// `MINIMUM_PRODUCED_READINGS` or more. Corroboration is the pipeline's own
// (`readingsCorroborate`, trigram overlap at the corroboration share), so a
// pass here means the candidate would have corroborated as the seated readers
// do.

/**
 One reading of a picture: the model that gave it and what it transcribed.
 */
export type TrustedReading = {
  readonly modelId: string;
  readonly text: string;
};

/**
 Every reading the pipeline stored for one picture.
 */
export type TrustedRecord = readonly TrustedReading[];

/**
 The least number of produced readings a verdict rests on: two thirds of the
 default sample of nine. Fewer say how the candidate failed to answer, not how
 it reads.
 */
export const MINIMUM_PRODUCED_READINGS = 6;

/**
 How the seated readers corroborate one another over the matched pictures.

 @example
 ```ts
 const band: SeatedBand = { floorCorroboratedByAll: 0.8, ceilingCorroboratedByNone: 0.1, readings: 20, };
 ```
 */
export type SeatedBand = {
  /**
   Lowest share of a seated reader's readings that every other reading of the
   picture corroborated.
   */
  readonly floorCorroboratedByAll: number;

  /**
   Highest share of a seated reader's readings that no other reading of the
   picture corroborated.
   */
  readonly ceilingCorroboratedByNone: number;

  /**
   Readings the band counts, across all readers.
   */
  readonly readings: number;
};

/**
 How a candidate's produced readings fared against the trusted readings.

 @example
 ```ts
 const rates: CandidateRates = { produced: 9, corroboratedByAll: 0.9, corroboratedByNone: 0, meanOverlap: 0.7, };
 ```
 */
export type CandidateRates = {
  /**
   Readings the candidate produced.
   */
  readonly produced: number;

  /**
   Share corroborated by every trusted reading of its picture.
   */
  readonly corroboratedByAll: number;

  /**
   Share corroborated by none of them.
   */
  readonly corroboratedByNone: number;

  /**
   Mean over the readings of the mean trigram overlap with the trusted readings.
   */
  readonly meanOverlap: number;
};

/**
 What the measurement decides.

 @example
 ```ts
 const verdict: MeasureVerdict = { kind: 'meets-both-clauses', };
 ```
 */
export type MeasureVerdict = {
  readonly kind: 'meets-both-clauses';
} | {
  readonly kind: 'fails-clauses';

  /**
   Each clause that failed, in order.
   */
  readonly clauses: readonly ('first' | 'second')[];
} | {
  readonly kind: 'inconclusive';

  /**
   Readings the candidate produced, below the least a verdict rests on.
   */
  readonly produced: number;
};

/**
 Share of some count, zero for none.

 @param part - the count in the share

 @param whole - the count it is a share of

 @returns The share, zero when there is no whole

 @example
 ```ts
 shareOf({ part: 1, whole: 4, },); // 0.25
 ```
 */
function shareOf(
  {
    part,
    whole,
  }: {
    readonly part: number;
    readonly whole: number;
  },
): number {
  return (whole === 0) ? 0 : (part / whole);
}

/**
 What one seated reader's readings came to: how many, how many every other
 reading corroborated, and how many no other reading corroborated.
 */
type ReaderTally = {
  readonly total: number;
  readonly byAll: number;
  readonly byNone: number;
};

/**
 What the other readings of a picture say about one reading.

 @param text - the reading asked about

 @param others - the other readings of the same picture

 @returns Whether every other reading corroborates it, whether none does, and
 their mean overlap

 @example
 ```ts
 const stand = standingOf({ text, others: ['Mittens naps.',], },);
 ```
 */
function standingOf(
  {
    text,
    others,
  }: {
    readonly text: string;
    readonly others: readonly string[];
  },
): {
  readonly byAll: boolean;
  readonly byNone: boolean;
  readonly meanOverlap: number
} {
  /**
   Whether each other reading corroborates this one.
   */
  const corroborations = others.map(function corroborates(other,): boolean {
    return readingsCorroborate({
      left: text,
      right: other,
    },)
      .kind
      === 'corroborated';
  },);

  /**
   Overlap with each other reading.
   */
  const overlaps = others.map(function overlapWith(other,): number {
    return trigramOverlap({
      left: text,
      right: other,
    },);
  },);
  return {
    byAll: corroborations.every(Boolean,),
    byNone: !corroborations.some(Boolean,),
    meanOverlap: shareOf({
      part: overlaps.reduce(
        function total(
          sum,
          overlap,
        ): number {
          return sum + overlap;
        },
        0,
      ),
      whole: overlaps.length,
    },),
  };
}

/**
 How the seated readers corroborate one another.

 @param records - every trusted record of the matched pictures

 @returns The lowest share corroborated by all and the highest corroborated by
 none, over the readers, and how many readings that rests on

 @example
 ```ts
 const band = seatedBandOf({ records, },);
 ```
 */
export function seatedBandOf({ records, }: { readonly records: readonly TrustedRecord[]; },): SeatedBand {
  /**
   Counts per reader: readings, corroborated by all, corroborated by none.
   */
  const tallies = new Map<string, ReaderTally>();
  for (const record of records) {
    for (const [index, reading,] of record.entries()) {
      /**
       Standing of this reading among the others of its picture.
       */
      const stand = standingOf({
        text: reading.text,
        others: record
          .filter(function isOther(
            _other,
            otherIndex,
          ): boolean {
            return otherIndex !== index;
          },)
          .map(function textOf(other,): string {
            return other.text;
          },),
      },);

      /**
       Counts so far for this reader.
       */
      const tally = tallies.get(reading.modelId,) ?? {
        total: 0,
        byAll: 0,
        byNone: 0,
      };
      tallies.set(
        reading.modelId,
        {
          total: tally.total + 1,
          byAll: tally.byAll + (stand.byAll ? 1 : 0),
          byNone: tally.byNone + (stand.byNone ? 1 : 0),
        },
      );
    }
  }

  /**
   Counts of each reader.
   */
  const readers = [...tallies.values(),];
  if (readers.length === 0) {
    return {
      floorCorroboratedByAll: 0,
      ceilingCorroboratedByNone: 0,
      readings: 0,
    };
  }
  return {
    floorCorroboratedByAll: Math.min(...readers.map(function allShare(reader: ReaderTally,): number {
      return shareOf({
        part: reader.byAll,
        whole: reader.total,
      },);
    },),),
    ceilingCorroboratedByNone: Math.max(...readers.map(function noneShare(reader: ReaderTally,): number {
      return shareOf({
        part: reader.byNone,
        whole: reader.total,
      },);
    },),),
    readings: readers
      .map(function totalOf(reader: ReaderTally,): number {
        return reader.total;
      },)
      .reduce(
      function sum(
        left,
        right,
      ): number {
        return left + right;
      },
      0,
      ),
  };
}

/**
 How the candidate's produced readings fared.

 @param rows - each produced reading with the trusted record of its picture

 @returns The shares corroborated by all and by none, and the mean overlap;
 all zero for no rows

 @example
 ```ts
 const rates = candidateRatesOf({ rows: [{ candidate, trusted, },], },);
 ```
 */
export function candidateRatesOf(
  { rows, }: {
    readonly rows: readonly {
      readonly candidate: string;
      readonly trusted: TrustedRecord;
    }[];
  },
): CandidateRates {
  /**
   Standing of each produced reading among the trusted ones.
   */
  const stands = rows.map(function standOf({
    candidate,
    trusted,
  },) {
    return standingOf({
      text: candidate,
      others: trusted.map(function textOf(reading,): string {
        return reading.text;
      },),
    },);
  },);
  return {
    produced: rows.length,
    corroboratedByAll: shareOf({
      part: stands.filter(function isAll(stand,): boolean {
        return stand.byAll;
      },)
        .length,
      whole: rows.length,
    },),
    corroboratedByNone: shareOf({
      part: stands.filter(function isNone(stand,): boolean {
        return stand.byNone;
      },)
        .length,
      whole: rows.length,
    },),
    meanOverlap: shareOf({
      part: stands.reduce(
        function total(
          sum,
          stand,
        ): number {
          return sum + stand.meanOverlap;
        },
        0,
      ),
      whole: rows.length,
    },),
  };
}

/**
 Decides the two clauses.

 @param band - how the seated readers corroborate one another

 @param candidate - how the candidate's readings fared

 @returns Both clauses met, the clauses that failed, or that too few readings
 were produced to say

 @example
 ```ts
 const verdict = verdictOf({ band, candidate, },);
 ```
 */
export function verdictOf(
  {
    band,
    candidate,
  }: {
    readonly band: SeatedBand;
    readonly candidate: CandidateRates;
  },
): MeasureVerdict {
  if (candidate.produced < MINIMUM_PRODUCED_READINGS) {
    return {
      kind: 'inconclusive',
      produced: candidate.produced,
    };
  }

  /**
   Clauses the candidate failed.
   */
  const clauses = [
    ...((candidate.corroboratedByAll < band.floorCorroboratedByAll) ? ['first' as const,] : []),
    ...((candidate.corroboratedByNone > band.ceilingCorroboratedByNone) ? ['second' as const,] : []),
  ];
  if (clauses.length === 0)
    return { kind: 'meets-both-clauses', };
  return {
    kind: 'fails-clauses',
    clauses,
  };
}

//endregion Reader measure score
