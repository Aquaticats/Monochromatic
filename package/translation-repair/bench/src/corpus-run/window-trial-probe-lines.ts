import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { TrialSlice, } from './window-trial-draw.ts';
import type { WindowTrialRow, } from './window-trial-ledger.ts';

//region Window trial probe lines
// The sentences the walk tells the log, apart from the ones the ledger report
// carries. Moved out of `window-trial-probe.ts` so each is a function a case
// reads whole.

/**
 Digest characters printed in the run's opening line.

 Enough to tell two protocols apart at a glance in a log, and short enough that
 the line stays readable; the ledger carries the whole digest either way.
 */
const PROTOCOL_LOG_CHARS = 12;

/**
 Opening line: the protocol the run buys under and how many arms it already
 holds.

 @param protocol - digest this run buys under

 @param armsBought - arms the ledger already holds under it

 @returns The line

 @example
 ```ts
 l.info(openingLine({ protocol, armsBought: 3, },),);
 ```
 */
export function openingLine(
  {
    protocol,
    armsBought,
  }: {
    readonly protocol: string;
    readonly armsBought: number;
  },
): string {
  return `protocol ${
    protocol.slice(
      0,
      PROTOCOL_LOG_CHARS,
    )
  }; ${String(armsBought,)} ${
    wordForCount({
      count: armsBought,
      one: 'arm',
      many: 'arms',
    },)
  } already bought`;
}

/**
 What one bought slice yielded, arm by arm.

 @param entryId - entry the slice belongs to

 @param pick - slice bought, with its class

 @param rows - arms it bought

 @returns The line

 @example
 ```ts
 l.info(boughtLine({ entryId, pick, rows, },),);
 ```
 */
export function boughtLine(
  {
    entryId,
    pick,
    rows,
  }: {
    readonly entryId: string;
    readonly pick: TrialSlice;
    readonly rows: readonly WindowTrialRow[];
  },
): string {
  return `${entryId}/${String(pick.sliceIndex,)} (${pick.sliceClass}): ${
    rows
      .map(function toOutcome(row,): string {
        return `${row.arm}=${row.shipped ? 'replaced' : 'kept'}`;
      },)
      .join(' ',)
  }`;
}

/**
 What the walk bought and refused.

 @param count - slices that bought at least one arm

 @param refused - slices that refused

 @returns The line

 @example
 ```ts
 l.info(walkEndLine({ count: 2, refused: 0, },),);
 ```
 */
export function walkEndLine(
  {
    count,
    refused,
  }: {
    readonly count: number;
    readonly refused: number;
  },
): string {
  return `bought ${String(count,)} ${
    wordForCount({
      count,
      one: 'slice',
      many: 'slices',
    },)
  } this run; ${
    String(refused,)
  } refused`;
}

/**
 What the ledger read for the report left out of the rows the report counts.

 @param rows - rows the report is counted from

 @param leftOut - whole lines the read left out as no trial row of this build

 @param tornTail - whether the last line was cut short by a kill

 @returns The line

 @example
 ```ts
 l.info(ledgerReadLine({ rows: 6, leftOut: 0, tornTail: false, },),);
 ```
 */
export function ledgerReadLine(
  {
    rows,
    leftOut,
    tornTail,
  }: {
    readonly rows: number;
    readonly leftOut: number;
    readonly tornTail: boolean;
  },
): string {
  return `ledger read for this report: ${String(rows,)} ${
    wordForCount({
      count: rows,
      one: 'row',
      many: 'rows',
    },)
  } under every protocol; ${String(leftOut,)} whole ${
    wordForCount({
      count: leftOut,
      one: 'line',
      many: 'lines',
    },)
  } left out as no trial row of this build; last line ${
    tornTail ? 'cut short by a kill and not counted' : 'whole'
  }`;
}

//endregion Window trial probe lines
