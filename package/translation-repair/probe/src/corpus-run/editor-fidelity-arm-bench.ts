import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import type { Answer, } from './editor-fidelity-arm-answers.ts';
import {
  type ArmCheckReport,
  checkEditorArms,
  lineOfCheck,
} from './editor-fidelity-arm-check.ts';
import {
  type ArmFigures,
  armFiguresOf,
} from './editor-fidelity-arm-figures.ts';
import {
  capAndThrowAnswer,
  stopsStatedAnswer,
  unusableWarning,
} from './editor-fidelity-arm-stops.ts';
import {
  type ArmFacts,
  type ArmLog,
  factsOf,
  usdSaid,
} from './editor-fidelity-arm-facts.ts';

//region Editor fidelity bench checks
// THE CHECKS THAT MAKE FOUR LOGS A BENCH, run over the logs the read step is given on top of the checks each arm passes.
// The arms are separate processes, so nothing but these checks stops a bench read from one log, from a log given twice,
// from arms run at other cards or in another order, or from an arm that was launched without being told what the arms
// before it spent: the spend guard of each arm holds it to its own `--max-usd` and to the bench total only through the
// `--bench-spent-usd` the operator wrote.

/**
 The efforts of the four arms of a bench in the order they run: the default effort is run twice to measure its own
 spread, between the lowest and the highest.
 */
const BENCH_EFFORTS = [
  'low',
  'medium',
  'medium',
  'high',
] as const;

/**
 Marker that stands before the effort in a card's id.
 */
const EFFORT_MARKER = '-effort-';

/**
 Names the effort a card runs at.

 @param card - the card an arm seated, empty when its log names none

 @returns The text after the effort marker, or `(none)` when the card has no effort in its id

 @example
 ```ts
 // yields 'low'
 const effort = effortOf({ card: 'claude-sonnet-5-5-effort-low', },);
 ```
 */
function effortOf({ card, }: { readonly card: string; }): string {
  /**
   Where the effort marker sits in the id.
   */
  const at = card.lastIndexOf(EFFORT_MARKER,);
  return (at === (-1)) ? '(none)' : card.slice(at + EFFORT_MARKER.length,);
}

/**
 The answer of the check that the logs are four distinct arms.

 @param arms - the logs, in the order given

 @returns Whether four logs were given and no two hold the same lines

 @example
 ```ts
 const answer = distinctAnswer({ arms, },);
 ```
 */
function distinctAnswer({ arms, }: { readonly arms: readonly ArmLog[]; }): Answer {
  if (arms.length !== BENCH_EFFORTS.length) {
    return {
      ok: false,
      detail: `${String(arms.length,)} ${(arms.length === 1) ? 'log was' : 'logs were'} given, and a bench is four`,
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
 The answer of the check that the arms ran at the cards low, medium, medium and high in that order.

 @param facts - the facts of each arm, in the order given

 @returns Whether the efforts of the cards are the bench's, in order

 @example
 ```ts
 const answer = cardsAnswer({ facts, },);
 ```
 */
function cardsAnswer({ facts, }: { readonly facts: readonly ArmFacts[]; }): Answer {
  /**
   The effort each arm ran at, in the order given.
   */
  const efforts = facts.map(function runAt({ card, },): string {
    return effortOf({ card, },);
  },)
    .join(', ',);
  return (efforts === BENCH_EFFORTS.join(', ',))
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

 @param facts - the facts of every arm, in the order given

 @param at - where the arm sits among them

 @returns The sum of the guard totals of the arms before it, zero for one whose total is not logged

 @example
 ```ts
 const countedBeforeUsd = countedBefore({ facts, at: 2, });
 ```
 */
function countedBefore(
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

 @param facts - the arm's facts

 @param countedBeforeUsd - what the guards of the arms before it counted, in dollars

 @returns Whether the spend the arm's guard line states covers what the earlier guards counted

 @example
 ```ts
 const answer = declaredAnswer({ facts, countedBeforeUsd: 0.875, },);
 ```
 */
function declaredAnswer(
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
 Runs the checks of every arm and the checks that make the logs a bench.

 @param arms - the arms, in the order they ran

 @returns One line for each check and how many of them failed

 @example
 ```ts
 const { lines, failed, } = checkEditorBench({ arms, },);
 ```
 */
export function checkEditorBench({ arms, }: { readonly arms: readonly ArmLog[]; }): ArmCheckReport {
  /**
   The checks of each arm and across them.
   */
  const base = checkEditorArms({ arms, },);
  /**
   The facts of each arm.
   */
  const facts = arms.map(function readOne(log,): ArmFacts {
    return factsOf({ log, },);
  },);
  /**
   The figures of each arm.
   */
  const allFigures = arms.map(function figuresOf(log,): ArmFigures {
    return armFiguresOf({ log, },);
  },);
  /**
   The checks that make the logs a bench.
   */
  const lines = [
    lineOfCheck({
      scope: 'bench',
      check: 'four distinct arm logs',
      answer: distinctAnswer({ arms, },),
    },),
    lineOfCheck({
      scope: 'bench',
      check: 'cards low, medium, medium, high in the order run',
      answer: cardsAnswer({ facts, },),
    },),
    ...arms.map(function declaredOf(
      { arm, },
      at,
    ): string {
      return lineOfCheck({
        scope: arm,
        check: 'earlier spend declared',
        answer: declaredAnswer({
          facts: nonNullishOrThrow(facts[at],),
          countedBeforeUsd: countedBefore({
            facts,
            at,
          },),
        },),
      },);
    },),
    ...arms.flatMap(function stopsOf(
      { arm, },
      at,
    ): readonly string[] {
      /**
       This arm's figures.
       */
      const figures = nonNullishOrThrow(allFigures[at],);
      return [
        lineOfCheck({
          scope: arm,
          check: 'stop reasons stated',
          answer: stopsStatedAnswer({ figures, },),
        },),
        lineOfCheck({
          scope: arm,
          check: 'no trial ended at the output cap or threw',
          answer: capAndThrowAnswer({ figures, },),
        },),
      ];
    },),
  ];
  /**
   The bench checks that failed.
   */
  const failedHere = lines.filter(function isFail(line,): boolean {
    return line.startsWith('FAIL ',);
  },);
  /**
   The warnings, which are no failure.
   */
  const warnings = arms.flatMap(function warnOf(
    { arm, },
    at,
  ): readonly string[] {
    return unusableWarning({
      arm,
      figures: nonNullishOrThrow(allFigures[at],),
    },);
  },);
  return {
    lines: [
      ...base.lines,
      ...lines,
      ...warnings,
    ],
    failed: base.failed + failedHere.length,
  };
}

//endregion Editor fidelity bench checks
