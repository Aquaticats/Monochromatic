import {
  lifecycleStamp,
  wallClockNow,
} from '@monochromatic-dev/translation-repair-markup/ts';
import {
  type OverlapSetting,
  readOverlapSetting,
} from '@monochromatic-dev/translation-repair-slice-cache/ts';
import { printRunLine, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Pass overlap
// Corpus pass keeps four slices in flight since the four matched pairs of
// 2026-08-27 and 2026-08-28 were read into a default on 2026-09-06
// (`doc/decision/translation-repair-pass-overlap.md`). One was the value
// while the decision waited for those pairs.
//
// READ ONCE PER PASS, BY THE ENTRY FILE, since 2026-10-10. Each entry read the
// variable for itself inside `settleEntry`, so a value nothing could read
// refused the pass at its first entry, after the runs directory was claimed,
// and a test chose an entry's overlap by writing the process's environment.
// The entry file reads it beside the pass's other overrides and every entry
// is handed the same setting, logged on its own `OVERLAP` line as before.

/**
 Corpus-pass slice overlap when invocation sets no environment override.
 */
const PASS_OVERLAP = 4;

/**
 Reads the corpus pass's overlap once, from the environment the entry file
 hands in.

 @param env - environment of the launch

 @returns Slices each driver may keep in flight, beside where the value came from

 @throws StatedRefusalError when environment value is invalid

 @example
 ```ts
 const overlap = readPassOverlap({ env: process.env, },);
 ```
 */
export function readPassOverlap(
  { env, }: { readonly env: Readonly<NodeJS.ProcessEnv>; },
): OverlapSetting {
  return readOverlapSetting({
    fallback: PASS_OVERLAP,
    env,
  },);
}

/**
 Logs one entry's overlap once before entry work starts.

 @param entryId - entry receiving same value across every per-slice driver

 @param setting - overlap the pass read at launch, beside its source

 @param attempt - number of the entry's attempt this line opens, as the pass's attempts file holds it

 @param wallClock - clock the line's time is read on; the wall clock unless a test fixes it

 @returns Slices each driver may keep in flight

 @throws `LifecycleAttemptError` (markup package) when the attempt is not a whole number from 1

 @example
 ```ts
 const overlap = entryOverlap({ entryId: 'XingZ60', setting, attempt: 2, },);
 ```
 */
export function entryOverlap(
  {
    entryId,
    setting,
    attempt,
    wallClock = wallClockNow,
  }: {
    readonly entryId: string;
    readonly setting: OverlapSetting;
    readonly attempt: number;
    readonly wallClock?: () => Date;
  },
): number {
  printRunLine({
    line: `OVERLAP ${entryId} ${
      lifecycleStamp({
        attempt,
        wallClock,
      },)
    } value=${String(setting.overlap,)} source=${setting.source}`,
  },);
  return setting.overlap;
}

//endregion Pass overlap
