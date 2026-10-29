import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import type { Answer, } from './editor-fidelity-arm-answers.ts';
import {
  type ArmFacts,
  type ArmLog,
  countWithNoun,
  usdSaid,
} from './editor-fidelity-arm-facts.ts';
import type {
  ArmFigures,
  RowStops,
} from './editor-fidelity-arm-figures.ts';
import { effortOf, } from './editor-fidelity-arm-shared.ts';

//region Editor fidelity bench answers
// THE ANSWERS OF THE CHECKS ACROSS ARMS: that the logs are three or four distinct arms, that their cards ran in the bench's
// order, that each arm was told what the arms before it spent, and that each later arm walked the first arm's trials in
// the first arm's order. `editor-fidelity-arm-bench.ts` turns each answer into a line.

/**
 Fewest arms of a bench: the high effort, the default and the low effort.
 */
export const BENCH_ARMS_LEAST = 3;

/**
 Most arms of a bench: a second run of the default may be given.
 */
export const BENCH_ARMS_MOST = 4;

/**
 The efforts of the arms of a bench in the order they run: the high effort first, whose completed trials the later arms
 walk, then the default, then the low effort; a second run of the default may stand after the first.
 */
const BENCH_EFFORT_ORDERS = [
  'high, medium, low',
  'high, medium, medium, low',
] as const;

/**
 The answer of the check that the logs are three or four distinct arms.

 @param arms - logs, in the order given

 @returns Whether three or four logs were given and no two hold the same lines

 @example
 ```ts
 const answer = distinctAnswer({ arms, },);
 ```
 */
export function distinctAnswer({ arms, }: { readonly arms: readonly ArmLog[]; }): Answer {
  if ((arms.length < BENCH_ARMS_LEAST) || (arms.length > BENCH_ARMS_MOST)) {
    return {
      ok: false,
      detail: `${String(arms.length,)} ${(arms.length === 1) ? 'log was' : 'logs were'} given, and a bench is three or four`,
    };
  }
  /**
   The name of each log, in the order given.
   */
  const names = arms.map(function nameOf({ arm, },): string {
    return arm;
  },);
  /**
   Each log's lines as one text, to compare logs by.
   */
  const bodies = arms.map(function bodyOf({ lines, },): string {
    return lines.join('\n',);
  },);
  for (const [first, body,] of bodies.entries()) {
    /**
     Where a later log holds the same lines, minus one when none does.
     */
    const second = bodies.findIndex(function repeats(
      other,
      at,
    ): boolean {
      return (at > first) && (other === body);
    },);
    if (second !== (-1)) {
      return {
        ok: false,
        detail: `${nonNullishOrThrow(names[first],)} and ${nonNullishOrThrow(names[second],)} hold the same lines`,
      };
    }
  }
  return {
    ok: true,
    detail: names.join(', ',),
  };
}

/**
 The answer of the check that the arms ran at the cards high, medium, a second medium if one was run, and low in that
 order.

 @param facts - facts of each arm, in the order given

 @returns Whether the efforts of the cards are the bench's, in order

 @example
 ```ts
 const answer = cardsAnswer({ facts, },);
 ```
 */
export function cardsAnswer({ facts, }: { readonly facts: readonly ArmFacts[]; }): Answer {
  /**
   The effort each arm ran at, in the order given.
   */
  const efforts = facts.map(function runAt({ card, },): string {
    return effortOf({ card, },);
  },)
    .join(', ',);
  return BENCH_EFFORT_ORDERS.some(function isOrder(order,): boolean {
    return order === efforts;
  },)
    ? {
      ok: true,
      detail: '',
    }
    : {
      ok: false,
      detail: `the logs ran ${efforts}`,
    };
}

/**
 What the guards of the arms before one counted, in dollars.

 @param facts - facts of every arm, in the order given

 @param at - where the arm sits among them

 @returns Sum of the guard totals of the arms before it, zero for one whose total is not logged

 @example
 ```ts
 const countedBeforeUsd = countedBefore({ facts, at: 2, });
 ```
 */
export function countedBefore(
  {
    facts,
    at,
  }: {
    readonly facts: readonly ArmFacts[];
    readonly at: number;
  },
): number {
  /**
   The facts of the arms before this one.
   */
  const earlier = facts.slice(
    0,
    at,
  );
  return earlier.reduce(
    function add(
      total: number,
      { guardUsd, },
    ): number {
      return total + Math.max(
        0,
        guardUsd,
      );
    },
    0,
  );
}

/**
 The answer of the check that an arm was told at least what the arms before it counted.

 @param facts - arm's facts

 @param countedBeforeUsd - what the guards of the arms before it counted, in dollars

 @returns Whether the spend the arm's guard line states covers what the earlier guards counted

 @example
 ```ts
 const answer = declaredAnswer({ facts, countedBeforeUsd: 0.875, },);
 ```
 */
export function declaredAnswer(
  {
    facts,
    countedBeforeUsd,
  }: {
    readonly facts: ArmFacts;
    readonly countedBeforeUsd: number;
  },
): Answer {
  if (facts.benchSpentUsd < 0) {
    return {
      ok: false,
      detail: 'the guard line states no --bench-spent-usd',
    };
  }
  // The operator rounds up to the cent, so the declared figure may exceed the count but never fall short of it.
  return (facts.benchSpentUsd >= countedBeforeUsd)
    ? {
      ok: true,
      detail: `--bench-spent-usd ${String(facts.benchSpentUsd,)} covers the ${
        usdSaid({ usd: countedBeforeUsd, },)
      } USD the arms before it counted`,
    }
    : {
      ok: false,
      detail: `--bench-spent-usd ${String(facts.benchSpentUsd,)} is less than the ${
        usdSaid({ usd: countedBeforeUsd, },)
      } USD the arms before it counted`,
    };
}

/**
 Says which trial a row walked and, for a later pass over the pool, the pass.

 @param row - row

 @returns Trial's handle, then `(pass 2)` and so on after it for a later pass

 @example
 ```ts
 // yields 'cat-0-s0/insertion (pass 2)'
 const said = unitSaid({ row, },);
 ```
 */
function unitSaid({ row, }: { readonly row: RowStops; },): string {
  return (row.pass > 1) ? `${row.trial} (pass ${String(row.pass,)})` : row.trial;
}

/**
 The answer of the check that a later arm walked the first arm's units in the first arm's order.

 @param first - first arm's name and figures

 @param later - later arm's figures

 @returns Whether the later arm's trials are the leading trials of the first arm's, in order

 @example
 ```ts
 const answer = trialOrderAnswer({ first: { arm: 'high', figures, }, later: figures, },);
 ```
 */
export function trialOrderAnswer(
  {
    first,
    later,
  }: {
    readonly first: {
      readonly arm: string;
      readonly figures: ArmFigures;
    };
    readonly later: ArmFigures;
  },
): Answer {
  if (later.rows
    .length
    > first.figures
      .rows
      .length) {
    return {
      ok: false,
      detail: `it walked ${
        countWithNoun({
          count: later.rows
            .length,
          one: 'unit',
          many: 'units',
        },)
      } and ${first.arm} completed ${
        String(first.figures
          .rows
          .length,)
      }`,
    };
  }
  /**
   The first arm's rows, the order the later arm is held to.
   */
  const firstRows = first.figures
    .rows;
  /**
   Where the later arm first walked another trial, or the same trial in another pass, than the first arm, minus one
   when it never did.
   */
  const differsAt = later.rows
    .findIndex(function walkedOther(
      walked,
      at,
    ): boolean {
      return unitSaid({ row: walked, },)
        !== unitSaid({ row: nonNullishOrThrow(firstRows[at],), },);
    },);
  if (differsAt !== (-1)) {
    return {
      ok: false,
      detail: `row ${String(differsAt + 1,)} walked ${
        unitSaid({ row: nonNullishOrThrow(later.rows[differsAt],), },)
      } where ${first.arm} walked ${
        unitSaid({ row: nonNullishOrThrow(firstRows[differsAt],), },)
      }`,
    };
  }
  return {
    ok: true,
    detail: `the ${
      countWithNoun({
        count: later.rows
          .length,
        one: 'unit',
        many: 'units',
      },)
    } it walked are the first ${String(later.rows
      .length,)} of ${first.arm}'s`,
  };
}

//endregion Editor fidelity bench answers
