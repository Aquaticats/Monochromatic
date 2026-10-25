import type { ConsolidationPolishConfig, } from '@monochromatic-dev/translation-repair-consolidate/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Consolidation slice seating
// THE ROSTER A PER-SLICE HOOK HANDS THE CONSOLIDATION (ledger H5), as the
// lanes' hooks hand the repair and translate lanes theirs. A roster is handed
// whole: writers, slate judges and the naturalness roles read together, since
// a bench re-read under a hold can seat a polish configuration of its own or
// none, and a field-by-field fallback could not say "none".

/**
 The consolidation's roster for one slice.

 @example
 ```ts
 const roster: ConsolidateRoster = { modelIds, judgeModelIds, };
 ```
 */
export type ConsolidateRoster = {
  /**
   Writers asked for consolidations.
   */
  readonly modelIds: readonly RosterModelId[];
  /**
   Voices that judge each slate and gate its winner.
   */
  readonly judgeModelIds: readonly RosterModelId[];
  /**
   Final naturalness roles and document guard facts, absent where the bench
   configures none.
   */
  readonly polishConfig?: ConsolidationPolishConfig;
};

/**
 What a per-slice hook hands the consolidation.

 @example
 ```ts
 const seating: ConsolidateSliceSeating = { roster, };
 ```
 */
export type ConsolidateSliceSeating = {
  /**
   Roster read since the consolidation started, absent while the given one
   stands.
   */
  readonly roster?: ConsolidateRoster;

  /**
   Slate judges a wet provider would serve as of the reading, which an
   eligible standing is held to the selection's floor by before a slate is
   bought; absent where the hook did not count. Kept beside the roster rather
   than in it, since the roster is folded into every key and a count is not.
   */
  readonly slateReachable?: number;
};

//endregion Consolidation slice seating
