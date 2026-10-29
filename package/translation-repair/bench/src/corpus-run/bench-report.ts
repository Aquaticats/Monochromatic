import { mkdir, } from 'node:fs/promises';
import { join, } from 'node:path';

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  StatedRefusalError,
  writeFileAtomic,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import type {
  BenchCall,
  CallTokens,
} from './bench-record.ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  describeSelfPreference,
  type SelectionRound,
  selfPreference,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type { BenchRow, } from './roster-bench-row.ts';

//region Bench report
// What the roster bench writes down, and what it prints.
//
// Counts are reported with the number of incumbent-bearing slices beside them.
// The corpus is mostly incumbent-bearing, so a bare decline rate would quietly
// generalize from the case the lane is least needed for to the case it exists
// for, and the split is what keeps that readable.

/**
 Raised when a roster cannot be benched because nothing in it varies.

 A STATED REFUSAL BY CLASS. The roster is the operator's, so a roster too
 narrow to vary is the operator's to widen, and every printer reports it as
 the command declining to run rather than as a fault with frames. Until
 2026-10-06 the class was a plain `Error` and `roster-bench` wrapped it by
 hand; a caller without the wrap printed it at exit 5.

 @example
 ```ts
 throw new BenchReportError({ seats: 1, },);
 ```
 */
export class BenchReportError extends StatedRefusalError {
  /**
   Declares this message safe to forward: it names the roster's size and
   writes the rest itself.
   */
  override readonly messageNamesOnly: true = true;

  /**
   Builds the refusal from the roster's size.

   @param seats - how many models the roster holds, fewer than the
   narrowest width a bench compares

   @example
   ```ts
   throw new BenchReportError({ seats: 1, },);
   ```
   */
  public constructor({ seats, }: { readonly seats: number; },) {
    super({ says: `a roster of ${String(seats,)} cannot be benched: nothing to vary`, },);
    this.name = 'BenchReportError';
  }
}

/**
 Narrowest producer roster worth benching: below two there is nothing to
 choose between except the incumbent.
 */
const NARROWEST_WIDTH = 2;

/**
 Adds a list of numbers.

 @param values - numbers to add

 @returns Their sum, zero when there are none

 @example
 ```ts
 const total = sumOf({ values: [1, 2,], },);
 ```
 */
function sumOf({ values, }: { readonly values: readonly number[]; },): number {
  return values.reduce(
    function add(
      total,
      value,
    ): number {
      return total + value;
    },
    0,
  );
}

/**
 Distinct numbers in a list, ascending.

 @param values - numbers to reduce to a sorted set

 @returns Each distinct value once

 @example
 ```ts
 const widths = distinctAscending({ values: rowWidths, },);
 ```
 */
function distinctAscending(
  { values, }: { readonly values: readonly number[]; },
): readonly number[] {
  return [...new Set(values,),].toSorted(function ascending(
    left,
    right,
  ): number {
    return left - right;
  },);
}

/**
 Widths this roster supports, and which one is measured twice.

 Derived from the roster length rather than written down, because the provider
 changes its offering often and a bench that hardcoded six would silently stop
 measuring the widest case the day a model is added.

 @param roster - models available to seat

 @returns Every width from the narrowest to the whole roster, plus the width
 whose repeat measures the run-to-run band

 @throws {@link BenchReportError} when the roster is too small to vary at
 all, a stated refusal naming the roster's size

 @example
 ```ts
 const { widths, repeated, } = benchWidths({ roster: RUN_ROSTER, },);
 ```
 */
export function benchWidths(
  { roster, }: { readonly roster: readonly string[]; },
): {
  readonly widths: readonly number[];
  readonly repeated: number;
} {
  /**
   Every width from the narrowest up to the whole roster.
   */
  const widths = Array.from(
    { length: Math.max(
      0,
      (roster.length - NARROWEST_WIDTH) + 1,
    ), },
    function toWidth(
      _unused,
      position,
    ): number {
      return NARROWEST_WIDTH + position;
    },
  );
  if (widths.length === 0)
    throw new BenchReportError({ seats: roster.length, },);

  return {
    widths,
    // The MIDDLE width carries the repeat. The band is meant to describe the
    // bench as a whole, and the extremes are its two least representative
    // points.
    repeated: nonNullishOrThrow(widths[Math.floor(widths.length / 2,)],),
  };
}

/**
 Writes every row so far, replacing the report each time.

 Rewritten after every row rather than once at the end: a bench that spends
 hours of quota and is then killed must leave everything it already bought.

 @param runsDir - runs directory the report is written under, which the entry
 resolves from its environment and hands down; until 2026-10-10 this read the
 environment itself

 @param rows - rows accumulated so far

 @param headSha - pipeline commit these rows were produced by

 @param widths - widths this run sweeps

 @param repeated - width run twice

 @param roster - full judge roster, which every width shares

 @example
 ```ts
 await writeBenchReport({ runsDir, rows, headSha, widths, repeated, roster, },);
 ```
 */
export async function writeBenchReport(
  {
    runsDir,
    rows,
    headSha,
    widths,
    repeated,
    roster,
  }: {
    readonly runsDir: string;
    readonly rows: readonly BenchRow[];
    readonly headSha: string;
    readonly widths: readonly number[];
    readonly repeated: number;
    readonly roster: readonly string[];
  },
): Promise<void> {
  /**
   Where bench reports live, beside the run's other artifacts.
   */
  const benchDir = join(
    runsDir,
    'roster-bench',
  );
  await mkdir(
    benchDir,
    { recursive: true, },
  );
  await writeFileAtomic({
    path: join(
      benchDir,
      'rows.json',
    ),
    text: JSON.stringify(
      {
        headSha,
        widths,
        repeated,
        roster,
        rows,
      },
      undefined,
      2,
    ),
  },);
}

/**
 Token cost of every exchange a set of rows made, both halves kept apart.

 Summed here rather than inside the summary line, because a width comparison
 is read one half at a time: seating another producer resends the same prompt
 and adds one more answer, and only the split says which of those the wider
 roster actually spent.

 @param rows - rows to total

 @returns Sending half, answering half, and reported total across every call

 @example
 ```ts
 const cost = tokensOfRows({ rows, },);
 ```
 */
function tokensOfRows(
  { rows, }: { readonly rows: readonly BenchRow[]; },
): CallTokens {
  /**
   Every exchange these rows made, flattened so each half is one pass.
   */
  const calls = rows.flatMap(function toCalls(row,): readonly BenchCall[] {
    return row.calls;
  },);

  return {
    promptTokens: sumOf({ values: calls.map(function toPrompt(call,): number {
      return call.promptTokens;
    },), },),
    completionTokens: sumOf({ values: calls.map(function toCompletion(call,): number {
      return call.completionTokens;
    },), },),
    tokens: sumOf({ values: calls.map(function toTotal(call,): number {
      return call.tokens;
    },), },),
  };
}

/**
 One line describing what a set of rows decided and cost.

 @param rows - rows to describe, all of one width and pass

 @returns Printable summary

 @throws Error when handed no row, which `summarizeBench` never does

 @example
 ```ts
 console.log(describeRows({ rows, },),);
 ```
 */
function describeRows(
  { rows, }: { readonly rows: readonly BenchRow[]; },
): string {
  // `summarizeBench` reads each width and each pass off the rows it then
  // groups by them, so a group holds the row its width and pass came from.
  // Over no row the mean this line ends on divides nothing by nothing and
  // would print `NaNms per slice`.
  if (rows.length === 0)
    throw new Error(
      'unreachable: describeRows was handed no row, though summarizeBench groups rows by a width and a pass it '
        + 'read off those rows',
    );

  /**
   Rows whose slice already had a translation.
   */
  const withIncumbent = rows.filter(function hasIncumbent(row,): boolean {
    return row.incumbentChars > 0;
  },);

  /**
   Rows the judges declined either way.
   */
  const declined = rows.filter(function wasDeclined(row,): boolean {
    return row.decision
      .startsWith('declined',);
  },);

  /**
   Rows that shipped the text already there.
   */
  const kept = rows.filter(function wasKept(row,): boolean {
    return row.keptIncumbent;
  },);

  /**
   Exchanges every row in this set made.
   */
  const calls = sumOf({ values: rows.map(function toCalls(row,): number {
    return row.calls
      .length;
  },), },);

  /**
   Tokens those exchanges moved, sending and answering halves apart.
   */
  const cost = tokensOfRows({ rows, },);

  /**
   Time this set took.
   */
  const ms = sumOf({ values: rows.map(function toMs(row,): number {
    return row.ms;
  },), },);

  /**
   Self-votes cast across the set.
   */
  const selfVotes = sumOf({ values: rows.map(function toSelfVotes(row,): number {
    return row.selfVotes;
  },), },);

  /**
   What those self-votes are worth once paired against the judges who held no
   stake in the same candidates.

   `selfVotes` IS NOT THE ANSWER, which is why both are printed. A roster
   whose producers write the best candidates would cast many self-votes and
   show no excess at all; one that favours its own work shows the same count
   and a positive excess. Only the second is what the half-weight discount
   exists to correct.
   */
  const preference = selfPreference({ rounds: rows.map(function toRound(row,): SelectionRound {
    return row.round;
  },), },);

  return [
    `${String(rows.length,)} ${
      wordForCount({
        count: rows.length,
        one: 'slice',
        many: 'slices',
      },)
    } (${String(withIncumbent.length,)} with an incumbent)`,
    `declined ${String(declined.length,)}`,
    `kept ${String(kept.length,)}`,
    `self-votes ${String(selfVotes,)}`,
    describeSelfPreference({ preference, },),
    `calls ${String(calls,)}`,
    `tokens ${String(cost.tokens,)} (in ${String(cost.promptTokens,)}, out ${String(cost.completionTokens,)})`,
    `${String(Math.round(ms / rows.length,),)}ms per slice`,
  ].join(', ',);
}

/**
 Prints the comparison the bench exists for.

 @param rows - every row the run produced

 @example
 ```ts
 summarizeBench({ rows, },);
 ```

 @internal
 */
export function summarizeBench(
  { rows, }: { readonly rows: readonly BenchRow[]; },
): void {
  /**
   Widths these rows cover.
   */
  const widths = distinctAscending({ values: rows.map(function toWidth(row,): number {
    return row.width;
  },), },);
  for (const width of widths) {
    /**
     Rows recorded at this width.
     */
    const atWidth = rows.filter(function matchesWidth(row,): boolean {
      return row.width === width;
    },);

    /**
     Passes recorded at this width.
     */
    const passes = distinctAscending({ values: atWidth.map(function toPass(row,): number {
      return row.pass;
    },), },);
    for (const pass of passes) {
      /**
       Rows of this width and pass.
       */
      const atPass = atWidth.filter(function matchesPass(row,): boolean {
        return row.pass === pass;
      },);
      console.log(
        `BENCH width ${String(width,)} pass ${String(pass,)}: ${
          describeRows({ rows: atPass, },)
        }`,
      );
    }
  }
}

//endregion Bench report
