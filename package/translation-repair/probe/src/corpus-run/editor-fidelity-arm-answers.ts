import { readSpendLine, } from '@monochromatic-dev/translation-repair-run/ts';
import {
  type ArmFacts,
  type ArmLog,
  countWithNoun,
  firstStarting,
  numberAfter,
  usdSaid,
} from './editor-fidelity-arm-facts.ts';

//region Editor fidelity arm answers
// THE ANSWER OF EACH CHECK an arm's log passes before any figure of it is read. A bench of arms is evidence only when each arm
// ran the pool it was asked, to the end, under the guard, with the money it says it spent accounted for; a count read off an arm
// that failed one of these is a count of something else.

/**
 Trials the pool holds at least, which an arm that asked fewer did not ask the pool.
 */
const POOL_TRIALS_LEAST = 40;

/**
 How far the SPEND lines' sum may be from the guard's total, in dollars: the total prints six decimals and each line nine, so
 only a rounding of the last printed digit separates a sum that is the total.
 */
const SUM_TOLERANCE_USD = 0.000005;

/**
 One check's answer: whether it holds and what it found.
 */
export type Answer = {
  /**
   Whether the check holds.
   */
  readonly ok: boolean;

  /**
   What the check found, written after its name.
   */
  readonly detail: string;
};

/**
 The answer of the exit check.

 @param log - arm

 @returns Whether the process exited with status zero

 @example
 ```ts
 const answer = exitAnswer({ log, },);
 ```
 */
export function exitAnswer({ log, }: { readonly log: ArmLog; }): Answer {
  if (!log.exit
    .recorded) {
    return {
      ok: false,
      detail: 'no exit status was recorded for this arm',
    };
  }
  return {
    ok: log.exit
      .code
      === 0,
    detail: `the process exited with status ${String(log.exit
      .code,)}`,
  };
}

/**
 The answer of the manifest check.

 @param facts - arm's facts

 @returns Whether the arm logged the digest of the manifest it asked

 @example
 ```ts
 const answer = manifestAnswer({ facts, },);
 ```
 */
export function manifestAnswer({ facts, }: { readonly facts: ArmFacts; }): Answer {
  return (facts.digest === '')
    ? {
      ok: false,
      detail: 'no manifest line was logged',
    }
    : {
      ok: true,
      detail: `digest ${facts.digest}`,
    };
}

/**
 The answer of the pool check.

 @param facts - arm's facts

 @returns Whether the pool line names a pool of enough trials of both defect families

 @example
 ```ts
 const answer = poolAnswer({ facts, },);
 ```
 */
export function poolAnswer({ facts, }: { readonly facts: ArmFacts; }): Answer {
  /**
   The pool line.
   */
  const line = firstStarting({
    bodies: facts.bodies,
    marker: 'pool: ',
  },);
  if (line === '') {
    return {
      ok: false,
      detail: 'no pool line was logged',
    };
  }
  /**
   Trials of the insertion family the line says.
   */
  const insertions = numberAfter({
    text: line,
    marker: '(insertion ',
  },);
  /**
   Trials of the alteration family the line says.
   */
  const alterations = numberAfter({
    text: line,
    marker: 'alteration ',
  },);
  if (facts.poolTrials < POOL_TRIALS_LEAST) {
    return {
      ok: false,
      detail: `${countWithNoun({
        count: facts.poolTrials,
        one: 'trial',
        many: 'trials',
      },)}, fewer than the ${
        String(POOL_TRIALS_LEAST,)
      } the pool holds`,
    };
  }
  if ((insertions < 1) || (alterations < 1)) {
    return {
      ok: false,
      detail: `insertion ${String(insertions,)} and alteration ${String(alterations,)}: both defect families are needed`,
    };
  }
  return {
    ok: true,
    detail: line,
  };
}

/**
 The answer of the guard check.

 @param facts - arm's facts

 @returns Whether a guard line naming the cap and the spend of the earlier arms was logged

 @example
 ```ts
 const answer = guardAnswer({ facts, },);
 ```
 */
export function guardAnswer({ facts, }: { readonly facts: ArmFacts; }): Answer {
  if (facts.maxUsd < 0) {
    return {
      ok: false,
      detail: 'no guard line was logged',
    };
  }
  return (facts.benchSpentUsd < 0)
    ? {
      ok: false,
      detail: 'the guard line states no --bench-spent-usd',
    }
    : {
      ok: true,
      detail: `held to --max-usd ${String(facts.maxUsd,)} USD after ${
        String(facts.benchSpentUsd,)
      } USD spent by the arms before it`,
    };
}

/**
 The answer of the error check.

 @param log - arm

 @returns Whether no line was written at the error level

 @example
 ```ts
 const answer = errorAnswer({ log, },);
 ```
 */
export function errorAnswer({ log, }: { readonly log: ArmLog; }): Answer {
  /**
   The first line at the error level.
   */
  const first = log.lines
    .find(function isError(line,): boolean {
      return line.startsWith('[error]',);
    },);
  return (first === undefined)
    ? {
      ok: true,
      detail: 'no line at the error level',
    }
    : {
      ok: false,
      detail: first,
    };
}

/**
 The answer of the seat check.

 @param facts - arm's facts

 @returns Whether the card's seat line states what the arm needs and nothing withheld or dark was logged

 @example
 ```ts
 const answer = seatAnswer({ facts, },);
 ```
 */
export function seatAnswer({ facts, }: { readonly facts: ArmFacts; }): Answer {
  /**
   The seat line of the card.
   */
  const line = firstStarting({
    bodies: facts.bodies,
    marker: `SEAT ${facts.card} `,
  },);
  if (line === '') {
    return {
      ok: false,
      detail: `no SEAT line for ${facts.card}`,
    };
  }
  if (!line.includes(' threw=',)) {
    return {
      ok: false,
      detail: `threw is not stated: ${line}`,
    };
  }
  /**
   Seats the router refused before any request.
   */
  const unreachable = numberAfter({
    text: line,
    marker: 'unreachable=',
  },);
  if (unreachable !== 0) {
    return {
      ok: false,
      detail: `unreachable=${String(unreachable,)}: the seat was refused before any request`,
    };
  }
  if (facts.bodies
    .some(function isRefused(body,): boolean {
      return body.startsWith('SEAT CREDENTIAL REFUSED',);
    },)) {
    return {
      ok: false,
      detail: 'a seat was withheld: credential refused',
    };
  }
  if (facts.bodies
    .some(function isDark(body,): boolean {
      return body.startsWith('SEATS DARK',);
    },)) {
    return {
      ok: false,
      detail: 'a SEATS DARK line was logged',
    };
  }
  return {
    ok: true,
    detail: line,
  };
}

/**
 The answer of the spend check.

 @param log - arm

 @param facts - arm's facts

 @returns Whether the SPEND lines add to the total the guard counted

 @example
 ```ts
 const answer = spendAnswer({ log, facts, },);
 ```
 */
export function spendAnswer({
  log,
  facts,
}: {
  readonly log: ArmLog;
  readonly facts: ArmFacts
}): Answer {
  if (facts.guardUsd < 0) {
    return {
      ok: false,
      detail: 'no GUARD total line was logged',
    };
  }
  /**
   What each SPEND line says.
   */
  const readings = log.lines
    .map(function reading(line,) {
      return readSpendLine({ line, },);
    },);
  if (readings.includes('unreadable',)) {
    return {
      ok: false,
      detail: 'a SPEND line cannot be read',
    };
  }
  /**
   The cost each SPEND record states, or the named absence.
   */
  const costs = readings.flatMap(function costOf(reading,) {
    return ((typeof reading) === 'string') ? [] : [reading.costUsd,];
  },);
  if (costs.includes('unreported',)) {
    return {
      ok: false,
      detail: 'a SPEND line states no cost, so the sum cannot be compared with the guard\'s total',
    };
  }
  /**
   Dollars the SPEND lines add to.
   */
  const summed = costs.reduce(
    function add(
      total: number,
      cost,
    ): number {
      return total + ((typeof cost) === 'number' ? cost : 0);
    },
    0,
  );
  return (Math.abs(summed - facts.guardUsd,) < SUM_TOLERANCE_USD)
    ? {
      ok: true,
      detail: `the SPEND lines and the guard both say ${usdSaid({ usd: summed, },)} USD`,
    }
    : {
      ok: false,
      detail: `the SPEND lines add to ${usdSaid({ usd: summed, },)} USD and the guard counted ${
        usdSaid({ usd: facts.guardUsd, },)
      } USD`,
    };
}

//endregion Editor fidelity arm answers
