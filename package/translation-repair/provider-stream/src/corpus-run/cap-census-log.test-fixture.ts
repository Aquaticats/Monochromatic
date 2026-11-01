import {
  reportStreamProgress,
  type StreamOutcome,
} from '../../dist/final/node/index.mjs';

//region Cap census log lines
// THE TWO LINE KINDS THE CAP CENSUS READS, written as the logger writes them,
// for the cases of the census's reader and of its runner.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. Stream lines come from
// `reportStreamProgress` itself, so a fixture follows the writer's wording
// rather than a copy of it.

/**
 A stream completion line as the logger writes it: the logger's prefix, then
 the line `reportStreamProgress` itself returns, so the fixture follows the
 writer's wording (a count of one takes "char") rather than a copy of it.

 @param stamp - ISO time of the line

 @param label - served id

 @param outcome - how the stream ended

 @param content - content characters delivered

 @returns The line

 @example
 ```ts
 const line = streamLine({ stamp: '2026-09-28T10:00:00.000Z', label: 'x', outcome: 'completed', content: 59, },);
 ```
 */
export function streamLine(
  {
    stamp,
    label,
    outcome,
    content,
  }: {
    readonly stamp: string;
    readonly label: string;
    readonly outcome: StreamOutcome;
    readonly content: number;
  },
): string {
  /**
   Line the writer returns, after the logger's prefix.
   */
  const written = reportStreamProgress({
    label,
    progress: {
      firstByteMs: 3_644,
      firstContentMs: 3_644,
      maxGapMs: 421,
      chars: 2_977,
      elapsedMs: 7_304,
    },
    unreadableFrames: 0,
    outcome,
    openingText: '',
    generatedChars: {
      content,
      reasoning: 0,
    },
  },);
  return `[info] [${stamp}] [translation-repair] [reportStreamProgress] ${written}`;
}

/**
 A spend line as the logger writes it.

 @param stamp - ISO time of the line

 @param tail - fields after the marker

 @returns The line

 @example
 ```ts
 const line = spendLine({ stamp: '2026-09-28T10:00:00.020Z', tail: 'provider=hyper model=x prompt=1 completion=2', },);
 ```
 */
export function spendLine(
  {
    stamp,
    tail,
  }: {
    readonly stamp: string;
    readonly tail: string;
  },
): string {
  return `[info] [${stamp}] [translation-repair] [reportSpend] SPEND ${tail}`;
}

/**
 Closing note the census prints after its last row, in the words it prints.
 */
export const CAP_CENSUS_CLOSING_NOTE: string = 'A rule reading can differ from a card for reasons that are not the model: a narrower log scope '
  + 'than the 2026-09-09 table, calls from before the caps, or calls the cap itself cut. Read each provider\'s cut '
  + 'columns before moving a cap; completion-cap.ts records the 2026-09-28 reading.';

//endregion Cap census log lines
