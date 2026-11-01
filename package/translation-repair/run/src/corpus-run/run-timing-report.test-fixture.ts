import { roundLine, } from '@monochromatic-dev/translation-repair-stage';
import { reportStreamProgress, } from '@monochromatic-dev/translation-repair-provider-stream';

//region Run timing report fixture
// Log lines a cat-themed run wrote, as the run-timing report reads them.
//
// EVERY LINE A CURRENT WRITER STILL PRODUCES IS BUILT BY CALLING THAT WRITER
// (`reportStreamProgress` and `roundLine`), so the fixture cannot drift from
// the line a log carries; only the logger's own prefix is a literal. The two
// lines of an older shape, which no writer can produce now, are literals.

/**
 Logger prefix of a line a round of a repair slice writes.
 */
const ROUND_PREFIX = '[info] [2026-08-25T10:00:30.000Z] [Tabby] [runDocumentLanes] [repairPreparedDocument] '
  + '[repair slice 3]';

/**
 A round whose quorum stood: six of seven voices heard, the last thirty
 seconds spent waiting after quorum.
 */
export const ROUND_STOOD_LINE: string = `${ROUND_PREFIX} ${
  roundLine({
    stage: 'editor',
    heard: 6,
    asked: 7,
    totalMs: 91_402,
    quorum: {
      kind: 'stood',
      toQuorumMs: 61_401,
      inGraceMs: 30_001,
    },
  },)
}`;

/**
 A round whose quorum never stood: one of seven voices heard, four needed.
 */
export const ROUND_NEVER_LINE: string = `${ROUND_PREFIX} ${
  roundLine({
    stage: 'editor',
    heard: 1,
    asked: 7,
    totalMs: 120_000,
    quorum: {
      kind: 'never',
      needed: 4,
    },
  },)
}`;

/**
 Writes a completion line a call ended on, as the stream reporter writes it.

 @param stamp - instant the logger stamped the line with, which is when the call ended

 @param elapsedMs - how long the call ran

 @returns The whole log line

 @example
 ```ts
 const line = timedCallLine({ stamp: '2026-08-25T10:00:10.000Z', elapsedMs: 10_000, },);
 ```
 */
export function timedCallLine(
  {
    stamp,
    elapsedMs,
  }: {
    readonly stamp: string;
    readonly elapsedMs: number;
  },
): string {
  return `[info] [${stamp}] [translation-repair] [reportStreamProgress] ${
    reportStreamProgress({
      label: 'hf:whiskers',
      progress: {
        firstByteMs: 40,
        firstContentMs: 40,
        maxGapMs: 4,
        chars: 512,
        elapsedMs,
      },
      unreadableFrames: 0,
      outcome: 'completed',
      openingText: '',
      generatedChars: {
        content: 40,
        reasoning: 7,
      },
    },)
  }`;
}

/**
 A completion line written before call durations were recorded: no `elapsed`
 field anywhere on it.
 */
export const UNTIMED_CALL_LINE: string = '[info] [2026-08-25T10:00:11.000Z] [translation-repair] '
  + '[reportStreamProgress] stream hf:mittens: completed, firstByte 40ms, maxGap 4ms, '
  + '512 raw chars, 0 unreadable frames, 40 content chars, 7 reasoning chars';

/**
 A completion line carrying a duration whose stamp the logger never writes.
 */
export const UNSTAMPED_CALL_LINE: string = '[info] [2026-08-25T10:00:12] [translation-repair] '
  + '[reportStreamProgress] stream hf:mittens: completed, elapsed 5000ms, firstByte 40ms, maxGap 4ms, '
  + '512 raw chars, 0 unreadable frames, 40 content chars, 7 reasoning chars';

//endregion Run timing report fixture
