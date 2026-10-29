import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  type ArmCheckReport,
  checkEditorArms,
  lineOfCheck,
} from './editor-fidelity-arm-check.ts';
import {
  cardsAnswer,
  countedBefore,
  declaredAnswer,
  distinctAnswer,
  trialOrderAnswer,
} from './editor-fidelity-arm-bench-answers.ts';
import {
  type ArmFacts,
  type ArmLog,
  countWithNoun,
  factsOf,
} from './editor-fidelity-arm-facts.ts';
import {
  type ArmFigures,
  armFiguresOf,
} from './editor-fidelity-arm-figures.ts';
import { sharedBenchOf, } from './editor-fidelity-arm-shared.ts';
import {
  stopsStatedAnswer,
  trialEndsAnswer,
  unusableWarning,
} from './editor-fidelity-arm-stops.ts';
import { MIN_SHARED_TRIALS, } from './editor-fidelity-decide.ts';

//region Editor fidelity bench checks
// THE CHECKS THAT MAKE THREE OR FOUR LOGS A BENCH, run over the logs the read step is given on top of the checks each arm
// passes. The arms are separate processes, so nothing but these checks stops a bench read from one log, from a log given
// twice, from arms run at other cards or in another order, from an arm that was launched without being told what the arms
// before it spent (the spend guard of each arm holds it to its own `--max-usd` and to the bench total only through the
// `--bench-spent-usd` the operator wrote), from a later arm that walked other trials than the first, or from too few
// trials shared by every arm for the rule to tell anything. The answers are `editor-fidelity-arm-bench-answers.ts`.

/**
 Runs the checks of every arm and the checks that make the logs a bench.

 @param arms - arms, in the order they ran

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
   The first arm, whose trials the later arms walk.
   */
  const first = {
    arm: arms[0]
      ?.arm
      ?? '',
    figures: nonNullishOrThrow(allFigures[0],),
  };
  /**
   The arms with their figures.
   */
  const withFigures = arms.map(function pairOf(
    { arm, },
    at,
  ): {
    readonly arm: string;
    readonly figures: ArmFigures
  } {
    return {
      arm,
      figures: nonNullishOrThrow(allFigures[at],),
    };
  },);
  /**
   The trials every arm completed, and what was left out.
   */
  const shared = sharedBenchOf({ arms: withFigures, },);
  /**
   The checks that make the logs a bench.
   */
  const lines = [
    lineOfCheck({
      scope: 'bench',
      check: 'three or four distinct arm logs',
      answer: distinctAnswer({ arms, },),
    },),
    lineOfCheck({
      scope: 'bench',
      check: 'cards high, medium, a second medium if run, low in the order run',
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
    ...withFigures.slice(1,)
      .map(function orderOf({
        arm,
        figures,
      },): string {
        return lineOfCheck({
          scope: arm,
          check: 'units in the first arm\'s order',
          answer: trialOrderAnswer({
            first,
            later: figures,
          },),
        },);
      },),
    ...withFigures.flatMap(function stopsOf({
      arm,
      figures,
    },): readonly string[] {
      return [
        lineOfCheck({
          scope: arm,
          check: 'stop reasons stated',
          answer: stopsStatedAnswer({ figures, },),
        },),
        lineOfCheck({
          scope: arm,
          check: 'units at the output cap or thrown',
          answer: trialEndsAnswer({ figures, },),
        },),
      ];
    },),
    lineOfCheck({
      scope: 'bench',
      check: 'shared units',
      answer: (shared.trials
        .length
        < MIN_SHARED_TRIALS)
        ? {
          ok: false,
          detail: `${
            countWithNoun({
              count: shared.trials
                .length,
              one: 'shared unit',
              many: 'shared units',
            },)
          } remain of the ${
            String(shared.firstRows,)
          } the first arm completed (${String(shared.thrown,)} left out as thrown, ${
            String(shared.unfinished,)
          } not completed by every arm), under the ${String(MIN_SHARED_TRIALS,)} the rule needs`,
        }
        : {
          ok: true,
          detail: `${
            countWithNoun({
              count: shared.trials
                .length,
              one: 'shared unit',
              many: 'shared units',
            },)
          } over ${
            countWithNoun({
              count: shared.entries,
              one: 'entry',
              many: 'entries',
            },)
          }, ${String(shared.thrown,)} left out as thrown and ${
            String(shared.unfinished,)
          } not completed by every arm, at least the ${String(MIN_SHARED_TRIALS,)} the rule needs`,
        },
    },),
  ];
  /**
   The bench checks that failed.
   */
  const failedHere = lines.filter(function isFail(line,): boolean {
    return line.startsWith('FAIL ',);
  },);
  return {
    lines: [
      ...base.lines,
      ...lines,
      ...withFigures.flatMap(function warnOf({
        arm,
        figures,
      },): readonly string[] {
        return unusableWarning({
          arm,
          figures,
        },);
      },),
    ],
    failed: base.failed + failedHere.length,
  };
}

//endregion Editor fidelity bench checks
