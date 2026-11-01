import type { OverlapSetting, } from '../../dist/final/node/index.mjs';

//region Slice overlap dial
// SETS THE SLICE OVERLAP VARIABLE FOR ONE CASE, for the cases showing that a
// reader handed its environment, or a pass entry handed its overlap, reads
// that rather than `TRANSLATION_REPAIR_SLICE_OVERLAP` as the process holds it.
//
// AND THE OVERLAP A CORPUS PASS READS WHEN NOTHING SETS THE VARIABLE, which a
// case hands `settleEntry` and the pass when it is not about the overlap.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. Two test files kept a copy each, since the
// global-writes scan once followed a writer only within its own file (ledger
// B111); it follows a writer into every file that imports it by name since
// ledger B120, so the copies are this one. `process.env` is process-wide, so
// every suite around a case calling this runs one case at a time
// (`global-writes-sequenced.unit.test.ts`).

/**
 Environment variable naming how many slices may run at once.
 */
const OVERLAP_VAR = 'TRANSLATION_REPAIR_SLICE_OVERLAP';

/**
 Four slices in flight from the pass's own fallback, as `readPassOverlap`
 reads an environment that sets no overlap.

 @example
 ```ts
 const outcome = await settleEntry({ ...entryInput, overlap: PASS_FALLBACK_OVERLAP, },);
 ```
 */
export const PASS_FALLBACK_OVERLAP: OverlapSetting = {
  overlap: 4,
  source: 'fallback',
};

/**
 Sets the dial for the duration of one case, restoring whatever was there.

 @param says - value to set, or nothing to clear it

 @returns Disposable putting the invoker's own value back

 @example
 ```ts
 using dial = dialSaying({ says: '4', },);
 ```
 */
export function dialSaying({ says, }: { readonly says?: string; },): Disposable {
  /**
   Value as the invoking shell left it.
   */
  const before = process.env[OVERLAP_VAR];

  if (says === undefined) {
    Reflect.deleteProperty(
      process.env,
      OVERLAP_VAR,
    );
  } else
    process.env[OVERLAP_VAR] = says;

  return {
    [Symbol.dispose]: function restore(): void {
      if (before === undefined) {
        Reflect.deleteProperty(
          process.env,
          OVERLAP_VAR,
        );
      } else
        process.env[OVERLAP_VAR] = before;
    },
  };
}

//endregion Slice overlap dial
