import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import {
  type ArmCount,
  decideEffort,
} from './editor-fidelity-decide.ts';
import {
  type ArmFigures,
  armFiguresOf,
} from './editor-fidelity-arm-figures.ts';
import type { ArmLog, } from './editor-fidelity-arm-facts.ts';

//region Editor fidelity arm report
// THE LINES THE READ STEP PRINTS ONCE THE CHECKS OF A BENCH PASS, and only then: an arm with a failed check is not
// evidence, and no figure of it is read. For each arm, its clean fix count pooled and for each kind with the defect-only
// count beside it as a secondary count (a reply that removed the defect and changed more is not a clean fix and is not
// in the headline), and what went unusable, was refused, threw and was abandoned; for each arm, how the asks of every
// row ended; and last the verdict of the decision rule over the four arms, which is `decideEffort`, the rule whose text
// the probe printed before its first call.

/**
 The arms of a bench: the low effort, the default twice and the high effort.
 */
const BENCH_ARMS = 4;

/**
 Says an arm's figures.

 @param arm - the arm's name

 @param figures - the arm's figures

 @returns The line, without a newline

 @example
 ```ts
 const line = armLine({ arm: 'low', figures, });
 ```
 */
function armLine({
  arm,
  figures,
}: {
  readonly arm: string;
  readonly figures: ArmFigures
},): string {
  return `ARM ${arm} (${figures.card}): clean fix ${String(figures.fixed,)} of ${String(figures.trials,)} (insertion ${
    String(figures.insertion
      .fixed,)
  } of ${String(figures.insertion
    .trials,)}, alteration ${String(figures.alteration
      .fixed,)} of ${
    String(figures.alteration
      .trials,)
  }); defect only ${String(figures.defectOnly,)} (insertion ${String(figures.insertion
    .defectOnly,)}, alteration ${
    String(figures.alteration
      .defectOnly,)
  }); unusable ${String(figures.unusable,)}, refused ${String(figures.refused,)}, threw ${String(figures.threw,)}, abandoned ${
    String(figures.abandoned,)
  }`;
}

/**
 Says how the asks of every row of an arm ended.

 @param arm - the arm's name

 @param figures - the arm's figures

 @returns The line, without a newline

 @example
 ```ts
 const line = stopsLine({ arm: 'low', figures, });
 ```
 */
function stopsLine({
  arm,
  figures,
}: {
  readonly arm: string;
  readonly figures: ArmFigures
},): string {
  return `STOPS ${arm}: ${
    figures.rows
      .map(function stopOf({
        row,
        stops,
      },): string {
      return `row ${String(row,)} ${stops.join('+',)}`;
    },)
      .join('; ',)
  }`;
}

/**
 The counts of an arm that the decision rule reads.

 @param figures - the arm's figures

 @returns The clean fix counts and the trials for each kind

 @example
 ```ts
 const counts = countsOf({ figures, });
 ```
 */
function countsOf({ figures, }: { readonly figures: ArmFigures; },): ArmCount {
  return {
    insertion: {
      trials: figures.insertion
        .trials,
      fixed: figures.insertion
        .fixed,
    },
    alteration: {
      trials: figures.alteration
        .trials,
      fixed: figures.alteration
        .fixed,
    },
  };
}

/**
 The lines the read step prints for a bench whose checks passed.

 @param arms - the four arms, in the order they ran: low, medium, medium, high

 @returns The ARM line and the STOPS line of each arm, then the verdict

 @throws {@link Error} when the arms are not four, since the rule reads exactly four and the bench checks have already
 refused any other number

 @example
 ```ts
 for (const said of benchReportLines({ arms, }))
   process.stdout.write(`${said}\n`,);
 ```
 */
export function benchReportLines({ arms, }: { readonly arms: readonly ArmLog[]; },): readonly string[] {
  if (arms.length !== BENCH_ARMS)
    throw new Error(`the verdict is read over four arms, and ${String(arms.length,)} were given`,);
  /**
   The figures of each arm.
   */
  const figures = arms.map(function figuresOf(log,): ArmFigures {
    return armFiguresOf({ log, },);
  },);
  /**
   The figures of the low arm, the two medium arms and the high arm.
   */
  const [lowFigures, mediumAFigures, mediumBFigures, highFigures,] = figures;
  /**
   The sentence the rule says its verdict in.
   */
  const { said, } = decideEffort({
    low: countsOf({ figures: nonNullishOrThrow(lowFigures,), },),
    mediumA: countsOf({ figures: nonNullishOrThrow(mediumAFigures,), },),
    mediumB: countsOf({ figures: nonNullishOrThrow(mediumBFigures,), },),
    high: countsOf({ figures: nonNullishOrThrow(highFigures,), },),
  },);
  return [
    ...arms.map(function armOf(
      { arm, },
      at,
    ): string {
      return armLine({
        arm,
        figures: nonNullishOrThrow(figures[at],),
      },);
    },),
    ...arms.map(function stopsOf(
      { arm, },
      at,
    ): string {
      return stopsLine({
        arm,
        figures: nonNullishOrThrow(figures[at],),
      },);
    },),
    said,
  ];
}

//endregion Editor fidelity arm report
