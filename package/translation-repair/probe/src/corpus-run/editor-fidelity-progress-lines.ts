import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Editor fidelity progress lines
// THE LINES AN ARM WRITES ABOUT HOW FAR IT WENT: how many of the pool's trials it planned, where the spend guard capped it
// and how many units it completed, a unit being a trial in one pass over the pool. The read step takes the number of units
// the later arms walk from the first arm's `TRIALS completed` line, whose marker stays so that the runbook's command reads
// the number after it; the noun after the planned count says units.

/**
 Says how many of the pool's trials an arm planned to walk.

 @param planned - trials the arm was asked to walk, or asks when it walks the pool several times

 @param total - trials the pool holds, or asks the passes over it hold

 @param passes - how many times the arm walks the pool, one when it is not said

 @returns Line, without a newline

 @example
 ```ts
 log.info(plannedLine({ planned: 41, total: 88, },),);
 ```
 */
export function plannedLine({
  planned,
  total,
  passes = 1,
}: {
  readonly planned: number;
  readonly total: number;
  readonly passes?: number
},): string {
  if (passes > 1) {
    return `planned: ${String(planned,)} of ${String(total,)} ${
      wordForCount({
        count: total,
        one: 'ask',
        many: 'asks',
      },)
    } (${String(passes,)} ${
      wordForCount({
        count: passes,
        one: 'pass',
        many: 'passes',
      },)
    } over the ${
      String(total / passes,)
    } ${
      wordForCount({
        count: total / passes,
        one: 'trial',
        many: 'trials',
      },)
    } of the pool)`;
  }
  return `planned: ${String(planned,)} of ${String(total,)} ${
    wordForCount({
      count: total,
      one: 'trial',
      many: 'trials',
    },)
  }`;
}

/**
 Says where the spend guard capped an arm.

 @param completed - units completed before the cap

 @param planned - units the arm was asked to walk

 @param said - guard's own sentence

 @returns Line, without a newline

 @example
 ```ts
 log.warn(cappedLine({ completed: 41, planned: 88, said, },),);
 ```
 */
export function cappedLine(
  {
    completed,
    planned,
    said,
  }: {
    readonly completed: number;
    readonly planned: number;
    readonly said: string;
  },
): string {
  return `CAPPED at unit ${String(completed + 1,)} of ${String(planned,)}: ${String(completed,)} ${
    wordForCount({
      count: completed,
      one: 'unit',
      many: 'units',
    },)
  } completed; ${said}`;
}

/**
 Says how many units an arm completed.

 @param completed - units completed

 @param planned - units the arm was asked to walk

 @returns Line, without a newline

 @example
 ```ts
 log.info(trialsLine({ completed: 41, planned: 88, },),);
 ```
 */
export function trialsLine({
  completed,
  planned,
}: {
  readonly completed: number;
  readonly planned: number
},): string {
  return `TRIALS completed ${String(completed,)} of ${String(planned,)} ${
    wordForCount({
      count: planned,
      one: 'unit',
      many: 'units',
    },)
  }`;
}

//endregion Editor fidelity progress lines
