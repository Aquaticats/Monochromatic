import type { BudgetView, } from '@monochromatic-dev/translation-repair-provider/ts';
import { judgeSeatsFor, } from './run-seats.ts';
import {
  benchesOf,
  unreachableWritingBenches,
} from './run-seats-floor.ts';
import {
  BENCHES_ANY_PHASE_LEANS_ON,
  shortBenches,
} from './run-seats-wait.ts';

//region Give-up bench floors
// THE BENCH FLOORS AN ENTRY'S GIVE-UP OF A PROVIDER MUST LEAVE MET, read by the budgets through `unmetFloors`
// (`provider-budget-give-up-aside.ts`).
//
// AN ENTRY STOPS WAITING FOR A PROVIDER'S COOLDOWNS ONLY WHILE THE PROVIDERS LEFT CAN STILL DO WHAT WAITING WOULD HAVE
// BOUGHT (the lead's ruling of 2026-10-25, open to the owner's veto). What the entry's stages need is what the seat reader
// holds a phase to: every writing bench at its floor (`WRITING_BENCH_FLOOR`) and every bench any phase leans on at its
// quorum (`shortBenches`), counted over the seats a wet provider would serve. This module asks the seat reader's own
// helpers about a dryness view in which only the providers left are wet, so no number is copied: a floor or a quorum that
// the reader changes is changed here with it.
//
// ALL PHASES AT ONCE. A run goes through every phase, and a give-up lasts the entry, so a bench that only a later phase
// leans on counts as much as one the phase now seating leans on.

/**
 Names the bench floors that stay unmet when only the providers with a false entry in the view can be asked.

 @param dry - per provider, whether it cannot be asked: given up, dry by its meter, or held by a refusal

 @returns One clause per unmet floor, the writing floors and then the quorums, as the `JUDGE SEATS` lines word them;
 none when every bench any phase leans on can be seated

 @example
 ```ts
 const unmet = unmetBenchFloors({ dry: { ...allDry, hyper: false, }, },);
 // => []
 ```
 */
export function unmetBenchFloors({ dry, }: { readonly dry: BudgetView; },): readonly string[] {
  /**
   Every bench, derived from the view the way the seat reader derives it.
   */
  const benches = benchesOf({ seats: judgeSeatsFor({ dry, },), },);
  return [
    ...unreachableWritingBenches({
      benches,
      names: BENCHES_ANY_PHASE_LEANS_ON,
      dry,
    },),
    ...shortBenches({
      benches,
      names: BENCHES_ANY_PHASE_LEANS_ON,
      dry,
    },),
  ];
}

//endregion Give-up bench floors
