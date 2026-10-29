import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import type { PictureReaderSeating, } from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  type Reseated,
  reseatHookFor,
} from './pass-reseat-hook.ts';
import type {
  JudgeSeats,
  SeatReadingClient,
} from '@monochromatic-dev/translation-repair-run/ts';

//region Picture reader re-seating
// THE PICTURE READINGS' PER-PICTURE HOOK (ledger X12), beside the lane
// contest's (`pass-contest-reseat.ts`), so a test can drive it with the
// reading rig the other phases' hooks are tested with.
//
// THE READINGS READ THEIR READERS ONCE PER CALL: a dry-out between two
// pictures left every later picture on the readers read before it. Holds
// began after the pictures reading 11 times in 3 of 4,122 run logs (measured
// 2026-09-28, an upper bound: the census counts the preparation stages that
// follow the readings under the same reading). The hook now re-reads the
// readers while a hold runs, waiting it out where the readers cannot reach
// quorum, and keeps handing them over once it has ended
// (`pass-reseat-hook.ts`).

/**
 Hook the picture readings call before each picture.

 @example
 ```ts
 const hooks: PicturesHooks = picturesHooksFor({ client, signal, l, },);
 ```
 */
export type PicturesHooks = {
  readonly beforePicture: () => Promise<PictureReaderSeating>;
};

/**
 What the picture readings take from a seat reading: the readers, as
 `pass-seated-pictures.ts` seats them.

 @param seats - benches as of this picture

 @returns Readers the picture is read by, beside the line naming them

 @example
 ```ts
 const { seating, line, } = pictureSeatingOf({ seats, },);
 ```
 */
function pictureSeatingOf(
  { seats, }: { readonly seats: JudgeSeats; },
): Reseated<PictureReaderSeating> {
  /**
   Readers the picture is read by, for the line.
   */
  const readers = seats.readers
    .join(',',);
  return {
    seating: { readerModelIds: seats.readers, },
    line: `picture re-seated under a hold: readers=${readers}`,
  };
}

/**
 Builds the picture readings' per-picture hook.

 @param client - run client whose dryness view and holds are the router's own

 @param signal - entry abort the readings honour

 @param l - entry logger

 @returns Per-picture reader, handing each picture the readers it runs on

 @example
 ```ts
 const hooks = picturesHooksFor({ client, signal, l, },);
 ```
 */
export function picturesHooksFor(
  {
    client,
    signal,
    l,
  }: {
    readonly client: SeatReadingClient;
    readonly signal: AbortSignal;
    readonly l: Logger;
  },
): PicturesHooks {
  return {
    beforePicture: reseatHookFor<PictureReaderSeating>({
      client,
      signal,
      phase: 'pictures',
      unseated: {},
      seatingOf: pictureSeatingOf,
      l,
    },),
  };
}

//endregion Picture reader re-seating
