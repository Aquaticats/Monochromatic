import { EDITOR_BENCH_MAX_USD, } from './editor-fidelity-args.ts';
import {
  type Answer,
  errorAnswer,
  exitAnswer,
  guardAnswer,
  manifestAnswer,
  poolAnswer,
  seatAnswer,
  spendAnswer,
} from './editor-fidelity-arm-answers.ts';
import {
  ceilingAnswer,
  editorAnswer,
  plannedAnswer,
  rowAnswer,
} from './editor-fidelity-arm-trial-answers.ts';
import {
  type ArmFacts,
  type ArmLog,
  factsOf,
  usdSaid,
} from './editor-fidelity-arm-facts.ts';

//region Editor fidelity arm checks
// THE CHECKS AN ARM'S LOG PASSES BEFORE ANY FIGURE OF IT IS READ, each as one PASS or FAIL line, and the two across the arms:
// the same manifest digest in all of them, and the money of the whole bench. The command that reads the logs refuses when any
// line is a FAIL. The answers are `editor-fidelity-arm-answers.ts`; the facts they read are `editor-fidelity-arm-facts.ts`.

/**
 What the checks said.

 @example
 ```ts
 const report: ArmCheckReport = { lines: ['PASS low: exit 0: the process exited with status 0',], failed: 0, };
 ```
 */
export type ArmCheckReport = {
  /**
   One line for each check, a PASS or a FAIL, in the order the checks ran.
   */
  readonly lines: readonly string[];

  /**
   How many of them are a FAIL.
   */
  readonly failed: number;
};

/**
 Writes one check's line.

 @param scope - arm, or `bench`

 @param check - check's name

 @param answer - what it found

 @returns Line, PASS or FAIL first

 @example
 ```ts
 const line = lineOfCheck({ scope: 'low', check: 'exit 0', answer, },);
 ```
 */
export function lineOfCheck({
  scope,
  check,
  answer,
}: {
  readonly scope: string;
  readonly check: string;
  readonly answer: Answer
}): string {
  return `${answer.ok ? 'PASS' : 'FAIL'} ${scope}: ${check}${(answer.detail === '') ? '' : `: ${answer.detail}`}`;
}

/**
 The answer of the check that every arm printed the same manifest digest.

 @param arms - every arm with its facts

 @returns Whether the digests agree

 @example
 ```ts
 const answer = digestAnswer({ arms, },);
 ```
 */
function digestAnswer(
  { arms, }: { readonly arms: readonly {
    readonly arm: string;
    readonly facts: ArmFacts
  }[]; },
): Answer {
  /**
   The arms that printed a digest.
   */
  const printed = arms.filter(function hasDigest({ facts, },): boolean {
    return facts.digest !== '';
  },);
  /**
   The first arm that printed a digest.
   */
  const [first,] = printed;
  /**
   The first arm that printed another digest than the first arm's.
   */
  const other = printed.find(function differs({ facts, },): boolean {
    return facts.digest
      !== first?.facts
        .digest;
  },);
  return ((first === undefined) || (other === undefined))
    ? {
      ok: true,
      detail: '',
    }
    : {
      ok: false,
      detail: `${first.arm} printed ${first.facts
        .digest} and ${other.arm} printed ${other.facts
          .digest}`,
    };
}

/**
 The answer of the check that the whole bench fits its amount.

 @param arms - every arm with its facts

 @returns Whether the guards' totals are under the amount and the caps written do not pass it

 @example
 ```ts
 const answer = benchAnswer({ arms, },);
 ```
 */
function benchAnswer(
  { arms, }: { readonly arms: readonly {
    readonly arm: string;
    readonly facts: ArmFacts
  }[]; },
): Answer {
  /**
   Dollars the guards counted, and the caps written, over every arm.
   */
  const measured = arms.reduce(
    function add(
      total: number,
      { facts, },
    ): number {
      return total + Math.max(
        0,
        facts.guardUsd,
      );
    },
    0,
  );
  /**
   Dollars the caps written add to.
   */
  const caps = arms.reduce(
    function add(
      total: number,
      { facts, },
    ): number {
      return total + Math.max(
        0,
        facts.maxUsd,
      );
    },
    0,
  );
  if (measured >= EDITOR_BENCH_MAX_USD) {
    return {
      ok: false,
      detail: `${usdSaid({ usd: measured, },)} USD measured is not under the ${String(EDITOR_BENCH_MAX_USD,)} USD of the bench`,
    };
  }
  if (caps > EDITOR_BENCH_MAX_USD) {
    return {
      ok: false,
      detail: `the --max-usd of the arms add to ${String(caps,)} USD, over the ${String(EDITOR_BENCH_MAX_USD,)} USD of the bench`,
    };
  }
  return {
    ok: true,
    detail: `${usdSaid({ usd: measured, },)} USD measured under ${String(EDITOR_BENCH_MAX_USD,)} USD; caps add to ${String(caps,)} USD`,
  };
}

/**
 Runs every check over the logs of the arms of one bench.

 @param arms - arms, in the order they ran

 @returns One line for each check and how many of them failed

 @example
 ```ts
 const { lines, failed, } = checkEditorArms({ arms, },);
 ```
 */
export function checkEditorArms({ arms, }: { readonly arms: readonly ArmLog[]; }): ArmCheckReport {
  /**
   Each arm with its facts.
   */
  const read = arms.map(function readOne(log,): {
    readonly log: ArmLog;
    readonly arm: string;
    readonly facts: ArmFacts
  } {
    return {
      log,
      arm: log.arm,
      facts: factsOf({ log, },),
    };
  },);
  /**
   Every line, the arms' checks first.
   */
  const lines = [
    ...read.flatMap(function checksOf({
      log,
      arm,
      facts,
    },): readonly string[] {
      return [
        lineOfCheck({
          scope: arm,
          check: 'exit 0',
          answer: exitAnswer({ log, },),
        },),
        lineOfCheck({
          scope: arm,
          check: 'manifest line',
          answer: manifestAnswer({ facts, },),
        },),
        lineOfCheck({
          scope: arm,
          check: 'pool line',
          answer: poolAnswer({ facts, },),
        },),
        lineOfCheck({
          scope: arm,
          check: 'planned line',
          answer: plannedAnswer({ facts, },),
        },),
        lineOfCheck({
          scope: arm,
          check: 'ceiling line',
          answer: ceilingAnswer({ facts, },),
        },),
        lineOfCheck({
          scope: arm,
          check: 'guard line',
          answer: guardAnswer({ facts, },),
        },),
        lineOfCheck({
          scope: arm,
          check: 'EDITOR line',
          answer: editorAnswer({ facts, },),
        },),
        lineOfCheck({
          scope: arm,
          check: 'row count',
          answer: rowAnswer({ facts, },),
        },),
        lineOfCheck({
          scope: arm,
          check: 'no [error] line',
          answer: errorAnswer({ log, },),
        },),
        lineOfCheck({
          scope: arm,
          check: 'seat line',
          answer: seatAnswer({ facts, },),
        },),
        lineOfCheck({
          scope: arm,
          check: 'spend identity',
          answer: spendAnswer({
            log,
            facts,
          },),
        },),
      ];
    },),
    lineOfCheck({
      scope: 'bench',
      check: 'same manifest digest in every arm',
      answer: digestAnswer({ arms: read, },),
    },),
    lineOfCheck({
      scope: 'bench',
      check: 'total spend under the bench amount',
      answer: benchAnswer({ arms: read, },),
    },),
  ];
  return {
    lines,
    failed: lines.filter(function isFail(line,): boolean {
      return line.startsWith('FAIL ',);
    },)
      .length,
  };
}

//endregion Editor fidelity arm checks
