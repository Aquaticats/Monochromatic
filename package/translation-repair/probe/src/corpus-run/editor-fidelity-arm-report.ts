import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  BENCH_ARMS_LEAST,
  BENCH_ARMS_MOST,
} from './editor-fidelity-arm-bench-answers.ts';
import { decideEffort, } from './editor-fidelity-decide.ts';
import {
  type ArmFigures,
  armFiguresOf,
} from './editor-fidelity-arm-figures.ts';
import {
  type SharedBench,
  sharedBenchOf,
} from './editor-fidelity-arm-shared.ts';
import {
  type ArmLog,
  countWithNoun,
} from './editor-fidelity-arm-facts.ts';

//region Editor fidelity arm report
// THE LINES THE READ STEP PRINTS ONCE THE CHECKS OF A BENCH PASS, and only then: an arm with a failed check is not
// evidence, and no figure of it is read. For each arm, its clean fix count pooled and for each kind with the defect-only
// count beside it as a secondary count (a reply that removed the defect and changed more is not a clean fix and is not
// in the headline), and what went unusable, was refused, threw and was abandoned; the trials every arm shared; for each
// arm, how the asks of its rows ended; and last the paired lines and the verdict of the decision rule over the shared
// trials, which is `decideEffort`, the rule whose text the probe printed before its first call.

/**
 Says an arm's figures.

 @param arm - arm's name

 @param figures - arm's figures

 @returns Line, without a newline

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
 Says how the asks of the rows of an arm ended: a count for each stop reason, and the rows that did not end ok.

 @param arm - arm's name

 @param figures - arm's figures

 @returns Line, without a newline

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
  /**
   The stop words of each row, `not stated` for a row whose line states none.
   */
  const words = figures.rows
    .map(function wordOf({
      stated,
      stops,
    },): string {
      return stated ? stops.join('+',) : 'not stated';
    },);
  /**
   The rows for each stop word, in the order the words first appear.
   */
  const counted = Array.from(
    new Set(words,),
    function countOf(word,): {
      readonly word: string;
      readonly count: number
    } {
      return {
        word,
        count: words.filter(function isWord(each,): boolean {
          return each === word;
        },)
          .length,
      };
    },
  )
    .toSorted(function mostFirst(
      a,
      b,
    ): number {
      return b.count - a.count;
    },);
  /**
   The rows that did not end ok, with their stop words.
   */
  const notOk = figures.rows
    .flatMap(function notOkOf(
      { row, },
      at,
    ): readonly string[] {
      /**
       This row's stop word.
       */
      const word = nonNullishOrThrow(words[at],);
      return (word === 'ok') ? [] : [`row ${String(row,)} ${word}`,];
    },);
  return `STOPS ${arm}: ${
    counted.map(function wordCount({
      word,
      count,
    },): string {
      return `${word} ${String(count,)}`;
    },)
      .join(', ',)
  }${(notOk.length === 0) ? '' : `; not ok: ${notOk.join(', ',)}`}`;
}

/**
 Says the trials every arm shared and what was left out.

 @param shared - shared trials and counts left out

 @returns Line, without a newline

 @example
 ```ts
 const line = sharedLine({ shared, },);
 ```
 */
function sharedLine({ shared, }: { readonly shared: SharedBench; },): string {
  return `SHARED ${
    countWithNoun({
      count: shared.trials
        .length,
      one: 'unit',
      many: 'units',
    },)
  } over ${
    countWithNoun({
      count: shared.entries,
      one: 'entry',
      many: 'entries',
    },)
  } (${
    shared.completed
      .map(function completedBy({
        arm,
        rows,
      },): string {
        return `${arm} completed ${String(rows,)}`;
      },)
      .join(', ',)
  }; ${String(shared.thrown,)} left out as thrown, ${String(shared.unfinished,)} not completed by every arm)`;
}

/**
 The lines the read step prints for a bench whose checks passed.

 @param arms - three or four arms, in the order they ran: high, medium, a second medium if run, low

 @returns ARM line of each arm, SHARED line, STOPS line of each arm, then the paired lines and the verdict

 @throws {@link Error} when the arms are not three or four, since the rule reads exactly those and the bench checks have
 already refused any other number

 @example
 ```ts
 for (const said of benchReportLines({ arms, }))
   process.stdout.write(`${said}\n`,);
 ```
 */
export function benchReportLines({ arms, }: { readonly arms: readonly ArmLog[]; },): readonly string[] {
  if ((arms.length < BENCH_ARMS_LEAST) || (arms.length > BENCH_ARMS_MOST))
    throw new Error(`the verdict is read over three or four arms, and ${String(arms.length,)} were given`,);
  /**
   The figures of each arm.
   */
  const figures = arms.map(function figuresOf(log,): ArmFigures {
    return armFiguresOf({ log, },);
  },);
  /**
   The trials every arm shared.
   */
  const shared = sharedBenchOf({
    arms: arms.map(function withFigures(
      { arm, },
      at,
    ) {
      return {
        arm,
        figures: nonNullishOrThrow(figures[at],),
      };
    },),
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
    sharedLine({ shared, },),
    ...arms.map(function stopsOf(
      { arm, },
      at,
    ): string {
      return stopsLine({
        arm,
        figures: nonNullishOrThrow(figures[at],),
      },);
    },),
    ...decideEffort({ trials: shared.trials, },)
      .lines,
  ];
}

//endregion Editor fidelity arm report
