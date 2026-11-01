import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  EDITOR_DAMAGE_KINDS,
  type EditorFidelityTrial,
} from '../editor-fidelity-trials.ts';

//region Editor fidelity pool line
// The line that says what a run drew from the frozen pool, so the log of an arm shows the trials it saw.

/**
 Says how many trials a run drew, over how many entries and in which defect families.

 @param trials - the trials the run built, before the cap

 @returns The line, without a newline

 @example
 ```ts
 log.info(poolLine({ trials, },),);
 ```
 */
export function poolLine({ trials, }: { readonly trials: readonly EditorFidelityTrial[]; },): string {
  /**
   Entries the trials come from, each once.
   */
  const entries = new Set(trials.map(function entryOf(trial,): string {
    return trial.spec
      .entryId;
  },),);
  /**
   Count of each family, in the manifest's order.
   */
  const families = EDITOR_DAMAGE_KINDS.map(function countOf(kind,): string {
    return `${kind} ${String(trials.filter(function isKind(trial,): boolean {
      return trial.damageKind === kind;
    },)
      .length,)}`;
  },);
  return `pool: ${String(trials.length,)} ${
    wordForCount({
      count: trials.length,
      one: 'trial',
      many: 'trials',
    },)
  } over ${String(entries.size,)} ${
    wordForCount({
      count: entries.size,
      one: 'entry',
      many: 'entries',
    },)
  } (${families.join(', ',)})`;
}

//endregion Editor fidelity pool line
