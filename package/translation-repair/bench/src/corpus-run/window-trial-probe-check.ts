import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import type { WindowTrialRow, } from './window-trial-ledger.ts';
import { TRIAL_ARMS, } from './window-trial-report.ts';
import { assertWindowReachedJudges, } from './window-trial-witness.ts';

//region Window trial probe check
// The one check only a live run can make: that the neighbouring original
// reached the judges of the first wide arm. Moved out of `window-trial-probe.ts`.
// The state it keeps is a value each step takes and returns.

/**
 State of the live window check: wide arms bought under the witness, and
 whether the check has passed and the wrapper been dropped.

 @example
 ```ts
 const check: WindowCheck = { wideArms: 0, passed: false, };
 ```
 */
export type WindowCheck = {
  /**
   Wide arms bought under the witness so far.
   */
  readonly wideArms: number;

  /**
   Whether the check has passed, so later slices go straight to the client.
   */
  readonly passed: boolean;
};

/**
 The check before any slice.
 */
export const WINDOW_UNCHECKED: WindowCheck = {
  wideArms: 0,
  passed: false,
};

/**
 Counts the wide arms among the arms one slice bought.

 @param rows - arms the slice bought

 @returns How many of them were wide

 @example
 ```ts
 const wide = wideArmsIn({ rows, },);
 ```
 */
function wideArmsIn({ rows, }: { readonly rows: readonly WindowTrialRow[]; },): number {
  return rows
    .filter(function isWide(row,): boolean {
      return row.arm === TRIAL_ARMS.wide;
    },)
    .length;
}

/**
 Runs the check on one bought slice, on the earliest slice that bought a wide
 arm.

 Resumption can leave a slice owing narrow arms only, so this waits for a wide
 arm rather than for the first purchase.

 @param check - state before the slice

 @param rows - arms the slice bought

 @param sheets - everything the witness saw so far

 @param judges - judges seated per arm, which every wide arm should have shown
 the window to

 @param l - logger the pass is told to

 @returns State after the slice

 @throws {@link WindowEvidenceError} when the wide arms bought did not put the
 window in front of every judge

 @example
 ```ts
 const after = checkWindowReached({ check, rows, sheets: witness.sheets, judges: 4, l, },);
 ```
 */
export function checkWindowReached(
  {
    check,
    rows,
    sheets,
    judges,
    l,
  }: {
    readonly check: WindowCheck;
    readonly rows: readonly WindowTrialRow[];
    readonly sheets: readonly string[];
    readonly judges: number;
    readonly l: Logger;
  },
): WindowCheck {
  if (check.passed)
    return check;

  /**
   Wide arms bought so far, this slice's included.
   */
  const wideArms = check.wideArms + wideArmsIn({ rows, },);
  if (wideArms === 0)
    return {
      wideArms,
      passed: false,
    };

  assertWindowReachedJudges({
    sheets,
    expected: wideArms * judges,
  },);
  l.info('the window reached every judge of the first wide arm',);
  return {
    wideArms,
    passed: true,
  };
}

//endregion Window trial probe check
