import type { ChatMessage, } from '@monochromatic-dev/llm-llm-type/ts';

import { rosterQuorumSize, } from './roster-quorum-size.ts';
import { assertBenchStart, } from './stage-bench-start.ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Stage fan-out window
// HOW MANY SEATS A ROUND ASKS, since 2026-09-09: quorum plus one spare, from
// a bench rotated by the prompt, with the rest held back for the retry rounds.
//
// EVERY ROUND USED TO ASK THE WHOLE BENCH. On the first `noname` pass of
// 2026-09-09, 1,430 seats were asked across 229 rounds where quorum needed
// about 810, and the surplus was not free: the round waited a grace window
// on it, then abandoned what had not come, and on the per-token provider an
// abandoned stream was still generated and billed. The select stage alone
// asked 496 seats for 70 rounds of quorum 4. The owner's instruction of that
// day, after 200 USD went in a day, was to do everything to stop bleeding.
//
// QUORUM IS UNCHANGED. It is still at least half the bench, rounded up
// (`roster-quorum-size.ts`), computed over the whole bench; what changes is
// that the seats needed to reach it are asked first and the others only when
// a voice is lost. The one spare keeps a single lost voice from costing a
// second round every time, which at the measured loss rate (125 of 1,430
// voices on that pass, 8.7 percent) is what most rounds would otherwise pay.
//
// ROTATED BY THE PROMPT, deterministically, so the same slice asks the same
// seats on a re-run and different slices spread their first asks across the
// bench instead of always sparing the same tail. A round that reaches quorum
// on its window never asks the rest.

/**
 Seats asked beyond what quorum still needs, so one lost voice does not cost
 a second round.
 */
export const FANOUT_SPARE = 1;

/**
 How many seats a gather asks at once: the window of quorum plus the spare,
 which every production stage uses, or the whole bench, which a fixture
 scripting every seat's ballot asks for so its arithmetic reads over the
 bench it wrote.

 @example
 ```ts
 const mode: FanOutMode = 'window';
 ```
 */
export type FanOutMode = 'window' | 'whole-bench';

/**
 FNV-1a offset basis, 32-bit.
 */
const FNV_OFFSET = 0x81_1C_9D_C5;

/**
 FNV-1a prime, 32-bit.
 */
const FNV_PRIME = 0x01_00_01_93;

/**
 Where the bench's rotation starts for one prompt.

 One linear pass over the prompt's code units, FNV-1a, reduced by the bench
 size; a hash rather than a counter so the choice depends on the input and
 not on how many rounds ran before it.

 @param messages - prompt shared by every voice of the round

 @param size - seats on the bench

 @returns Index of the seat asked first, below `size`; zero for an empty bench

 @example
 ```ts
 const start = benchRotation({ messages, size: 7, },);
 ```
 */
export function benchRotation(
  {
    messages,
    size,
  }: {
    readonly messages: readonly ChatMessage[];
    readonly size: number;
  },
): number {
  if (size <= 0)
    return 0;

  /**
   Every message's content in order, as the hash walks it.
   */
  const parts = messages.map(function contentOf(message,): string {
    /**
     Content of this message; the type says text, so no structured form
     exists to stringify.
     */
    const { content, } = message;
    return content;
  },);

  /**
   The parts as one string, joined so a boundary between messages is a
   character the hash sees.
   */
  const text = parts.join('\n',);

  /**
   The text as UTF-8 bytes, the unit FNV-1a was written for; one linear
   encode and one linear fold.
   */
  const bytes = new TextEncoder().encode(text,);

  /**
   Running FNV-1a hash, kept unsigned by the shift at each step.
   */
  const hash = bytes.reduce(
    function fold(
      acc: number,
      byte: number,
    ): number {
      /**
       Accumulator with this byte folded in.
       */
      const mixed = acc ^ byte;

      /**
       Mixed value multiplied by the prime, as a 32-bit product.
       */
      const product = Math.imul(
        mixed,
        FNV_PRIME,
      );
      return product >>> 0;
    },
    FNV_OFFSET,
  );
  return hash % size;
}

/**
 The bench in the order this prompt asks it.

 @param modelIds - bench in roster order

 @param messages - prompt shared by every voice of the round

 @param start - seat the rotation starts at in place of the prompt's own, a whole number below the size of the bench, which a launch
 pins (`StageSettings.editorBenchStart`) so that the seats a fixture seats move with no sheet text; absent in production, where the
 prompt decides

 @returns Same seats, started at the pinned seat or at the prompt's rotation

 @throws {@link StageBenchStartError} when a pinned start is not a whole number from zero to below the size of the bench, which
 this is the one place to ask since the value and the bench are both in hand here

 @example
 ```ts
 const order = rotatedBench({ modelIds, messages, },);
 const pinned = rotatedBench({ modelIds, messages, start: 3, },);
 ```
 */
export function rotatedBench(
  {
    modelIds,
    messages,
    start: pinned,
  }: {
    readonly modelIds: readonly RosterModelId[];
    readonly messages: readonly ChatMessage[];
    readonly start?: number;
  },
): readonly RosterModelId[] {
  if (pinned !== undefined) {
    assertBenchStart({
      start: pinned,
      benchSize: modelIds.length,
    },);
  }
  /**
   Seat the rotation starts at.
   */
  const start = pinned ?? benchRotation({
    messages,
    size: modelIds.length,
  },);
  return [
    ...modelIds.slice(start,),
    ...modelIds.slice(
      0,
      start,
    ),
  ];
}

/**
 Seats one round asks: as many as quorum still needs, plus the spare, from
 the front of what is pending.

 @param pending - seats not yet heard, unasked first and lost after

 @param needed - voices quorum still lacks

 @returns Leading slice of `pending`, never longer than it

 @example
 ```ts
 const asking = askingWindow({ pending, needed: 4, },);
 ```
 */
export function askingWindow(
  {
    pending,
    needed,
  }: {
    readonly pending: readonly RosterModelId[];
    readonly needed: number;
  },
): readonly RosterModelId[] {
  return pending.slice(
    0,
    Math.max(
      0,
      needed,
    ) + FANOUT_SPARE,
  );
}

/**
 Seats a healthy bench costs on its first round: quorum plus the spare,
 never more than the bench holds. What a fixture counts when it asserts how
 many calls one round bought, so its arithmetic follows {@link FANOUT_SPARE}
 instead of restating it.

 @param benchSize - seats on the bench

 @returns Seats the first round asks while nothing has been lost yet

 @example
 ```ts
 const asked = firstRoundWindow({ benchSize: 6, },);
 ```
 */
export function firstRoundWindow(
  { benchSize, }: { readonly benchSize: number; },
): number {
  return Math.min(
    benchSize,
    rosterQuorumSize({ rosterSize: benchSize, },) + FANOUT_SPARE,
  );
}

//endregion Stage fan-out window
