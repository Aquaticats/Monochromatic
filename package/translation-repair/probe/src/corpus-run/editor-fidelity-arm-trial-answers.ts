import {
  type ArmFacts,
  countWithNoun,
  firstStarting,
  isRowBody,
  numberAfter,
} from './editor-fidelity-arm-facts.ts';
import type { Answer, } from './editor-fidelity-arm-answers.ts';

//region Editor fidelity arm trial answers
// THE ANSWERS OF THE CHECKS THAT COUNT AN ARM'S TRIALS: the trials the planned line says the arm was asked to walk, the
// ceiling priced for them, the tally over the trials it completed, and a row for each trial completed. An arm that the spend
// guard capped completes fewer trials than it planned, and a CAPPED line must say so.

/**
 The answer of the planned-trials check.

 @param facts - arm's facts

 @returns Whether the planned line names no more units than the pool's trials hold through the passes it says the arm walks

 @example
 ```ts
 const answer = plannedAnswer({ facts, },);
 ```
 */
export function plannedAnswer({ facts, }: { readonly facts: ArmFacts; }): Answer {
  if (facts.planned < 0) {
    return {
      ok: false,
      detail: 'no planned line was logged',
    };
  }
  if (facts.poolTrials < 0) {
    return {
      ok: false,
      detail: `${
        countWithNoun({
          count: facts.planned,
          one: (facts.passes === 1) ? 'trial' : 'unit',
          many: (facts.passes === 1) ? 'trials' : 'units',
        },)
      } planned, and no pool line was logged to hold them against`,
    };
  }
  /**
   Units the passes over the pool hold: the pool's trials once for each pass.
   */
  const held = facts.poolTrials * facts.passes;
  /**
   Whether the line plans more units than the passes hold.
   */
  const over = facts.planned > held;
  if (facts.passes === 1) {
    return {
      ok: !over,
      detail: `${countWithNoun({
        count: facts.planned,
        one: 'trial',
        many: 'trials',
      },)} planned of a pool of ${String(facts.poolTrials,)}`,
    };
  }
  /**
   The units planned with their noun, the passes, and the pool.
   */
  const said = `${countWithNoun({
    count: facts.planned,
    one: 'unit',
    many: 'units',
  },)} planned in ${
    countWithNoun({
      count: facts.passes,
      one: 'pass',
      many: 'passes',
    },)
  } over a pool of ${
    countWithNoun({
      count: facts.poolTrials,
      one: 'trial',
      many: 'trials',
    },)
  }`;
  return {
    ok: !over,
    detail: over
      ? `${said}, more than the ${
        countWithNoun({
          count: held,
          one: 'unit',
          many: 'units',
        },)
      } the passes hold`
      : said,
  };
}

/**
 The answer of the ceiling check.

 @param facts - arm's facts

 @returns Whether a ceiling line was logged for the number of trials the planned line says

 @example
 ```ts
 const answer = ceilingAnswer({ facts, },);
 ```
 */
export function ceilingAnswer({ facts, }: { readonly facts: ArmFacts; }): Answer {
  /**
   The ceiling line.
   */
  const line = firstStarting({
    bodies: facts.bodies,
    marker: 'ceiling ',
  },);
  if (line === '') {
    return {
      ok: false,
      detail: 'no ceiling line was logged',
    };
  }
  /**
   Trials the line is for.
   */
  const trials = numberAfter({
    text: line,
    marker: ': ',
  },);
  return (trials === facts.planned)
    ? {
      ok: true,
      detail: `${countWithNoun({
        count: trials,
        one: 'unit',
        many: 'units',
      },)} priced`,
    }
    : {
      ok: false,
      detail: `${countWithNoun({
        count: trials,
        one: 'unit',
        many: 'units',
      },)}, for ${String(facts.planned,)} planned`,
    };
}

/**
 The answer of the EDITOR line check.

 @param facts - arm's facts

 @returns Whether the arm's card has a tally over the pool's trials

 @example
 ```ts
 const answer = editorAnswer({ facts, },);
 ```
 */
export function editorAnswer({ facts, }: { readonly facts: ArmFacts; }): Answer {
  /**
   The tally line of the card.
   */
  const line = firstStarting({
    bodies: facts.bodies,
    marker: `EDITOR ${facts.card}: clean fix `,
  },);
  if (line === '') {
    return {
      ok: false,
      detail: `no EDITOR line for ${facts.card} was logged`,
    };
  }
  /**
   Trials the tally counts.
   */
  const counts = numberAfter({
    text: line,
    marker: ' of ',
  },);
  return (counts === facts.completed)
    ? {
      ok: true,
      detail: line,
    }
    : {
      ok: false,
      detail: `counts ${countWithNoun({
        count: counts,
        one: 'unit',
        many: 'units',
      },)}, for ${
        String(facts.completed,)
      } completed`,
    };
}

/**
 The answer of the row check.

 @param facts - arm's facts

 @returns Whether a row was logged once for each trial the arm says it completed, and a CAPPED line says why an arm that
 completed fewer than it planned stopped

 @example
 ```ts
 const answer = rowAnswer({ facts, },);
 ```
 */
export function rowAnswer({ facts, }: { readonly facts: ArmFacts; }): Answer {
  if (facts.completed < 0) {
    return {
      ok: false,
      detail: 'no TRIALS line was logged',
    };
  }
  /**
   Row numbers the log wrote, one for each row line.
   */
  const numbers = facts.bodies
    .filter(function isRow(body,): boolean {
      return isRowBody({ body, },);
    },)
    .map(function numberOf(body,): number {
      return numberAfter({
        text: body,
        marker: 'row ',
      },);
    },);
  if (numbers.length !== facts.completed) {
    return {
      ok: false,
      detail: `${countWithNoun({
        count: numbers.length,
        one: 'row',
        many: 'rows',
      },)} logged for ${String(facts.completed,)} completed ${(facts.completed === 1) ? 'unit' : 'units'}`,
    };
  }
  if (new Set(numbers,).size !== numbers.length) {
    return {
      ok: false,
      detail: 'a row number repeats, so a row was logged twice and another not at all',
    };
  }
  // A ROW THAT MADE NO CALL IS NO TRIAL. The run client logs a PROMPT-REUSE line for a reply it gives from an earlier one, and
  // such a row measures nothing: on the billed first arm of `dd4b15b75c` it was a repeat pass compared with itself.
  if (facts.reusedRows > 0) {
    return {
      ok: false,
      detail: `${String(facts.reusedRows,)} of ${String(numbers.length,)} ${
        (numbers.length === 1) ? 'row was' : 'rows were'
      } answered by prompt reuse and made no call, so none of them is a trial`,
    };
  }
  /**
   Whether the arm stopped short of the trials it planned.
   */
  const stoppedShort = facts.completed < facts.completedOf;
  if (stoppedShort && (!facts.capped)) {
    return {
      ok: false,
      detail: `${String(facts.completed,)} of ${
        countWithNoun({
          count: facts.completedOf,
          one: 'planned unit',
          many: 'planned units',
        },)
      } were completed and no CAPPED line says why the arm stopped`,
    };
  }
  return {
    ok: true,
    detail: `${countWithNoun({
      count: numbers.length,
      one: 'row',
      many: 'rows',
    },)} logged, each once${
      stoppedShort ? `; the guard capped the arm before unit ${String(facts.completed + 1,)}` : ''
    }`,
  };
}

//endregion Editor fidelity arm trial answers
