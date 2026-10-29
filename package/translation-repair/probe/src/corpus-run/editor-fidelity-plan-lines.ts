import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Editor fidelity plan lines
// THE LINES A RUN SAYS ABOUT WHAT IT PRICED AND WHAT IT KEPT, apart from the plan so that file stays under the
// file-length cap.

/**
 Line saying a request was accepted and no model was asked, which is what a cap of zero buys.

 @param cardIds - cards the request seats

 @param selected - pinned reference specifications the request selected

 @param planned - asks the ceiling is priced over

 @param total - asks the passes over the pool hold

 @param passes - how many times the pool is walked, one when it is not said

 @returns Line, without a newline

 @example
 ```ts
 log.info(editorPreflightLine({ cardIds, selected: 3, planned: 88, total: 88, },),);
 ```
 */
export function editorPreflightLine(
  {
    cardIds,
    selected,
    planned,
    total,
    passes = 1,
  }: {
    readonly cardIds: readonly RosterModelId[];
    readonly selected: number;
    readonly planned: number;
    readonly total: number;
    readonly passes?: number;
  },
): string {
  /**
   What the ceiling is priced over: the whole pool, or its leading trials; for several passes, every ask of them or the
   leading asks.
   */
  const priced = (planned === total)
    ? ((passes > 1)
      ? `every ask of the ${String(passes,)} ${
        wordForCount({
          count: passes,
          one: 'pass',
          many: 'passes',
        },)
      } over the pool`
      : 'every trial of the pool')
    : `the first ${String(planned,)} of ${String(total,)} ${
      (passes > 1)
        ? 'asks'
        : wordForCount({
          count: total,
          one: 'trial',
          many: 'trials',
        },)
    }`;
  return `preflight only: ${String(selected,)} pinned reference ${
    wordForCount({
      count: selected,
      one: 'specification',
      many: 'specifications',
    },)
  } selected for ${cardIds.join(', ',)}; the corpus was read and no model was called; the ceiling is priced over ${priced}`;
}

/**
 Line saying how many rows were kept and where.

 @param count - rows kept

 @param keptAt - path of the kept run

 @returns Line, without a newline

 @example
 ```ts
 log.info(editorKeptLine({ count: 4, keptAt: '/runs/x.json', },),);
 ```
 */
export function editorKeptLine(
  {
    count,
    keptAt,
  }: {
    readonly count: number;
    readonly keptAt: string;
  },
): string {
  return `kept ${String(count,)} editor ${
    wordForCount({
      count,
      one: 'row',
      many: 'rows',
    },)
  } at ${keptAt}`;
}

//endregion Editor fidelity plan lines
