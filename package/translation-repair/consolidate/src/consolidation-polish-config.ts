import type { PreparedDocumentPair, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { ConsolidationPolishConfig, } from './consolidation-polish.ts';
import {
  collectDefinitions,
  parseDocument,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { RepairModels, } from '@monochromatic-dev/translation-repair-assembly/ts';

//region Consolidation polish configuration
// ONE BUILDER, NO "OFF" KIND (ledger T8, 2026-09-30). The only production
// caller is the corpus pass, whose roster always seats refiners
// (`RunRepairModels`), so the pass never met a roster without them; the
// function that answered one with a disabled kind had no production caller
// left once the pass built through this one. A library caller whose roster
// seats no refiners passes no `polishConfig` to the consolidation, which then
// runs no polish.

/**
 Builds final body polish configuration from a roster that seats refiners.

 @param prepared - shared source-target preparation

 @param models - repair role configuration with its refiners seated, which
 the run's roster always is

 @param gateModelIds - whole roster running final fidelity gate

 @returns Final polish roles and document facts

 @example
 ```ts
 const polishConfig = configuredConsolidationPolish({ prepared, models, gateModelIds, });
 ```
 */
export function configuredConsolidationPolish(
  {
    prepared,
    models,
    gateModelIds,
  }: {
    readonly prepared: PreparedDocumentPair;
    readonly models: RepairModels & { readonly refinerModelIds: readonly RosterModelId[]; };
    readonly gateModelIds: readonly RosterModelId[];
  },
): ConsolidationPolishConfig {
  return {
    refinerModelIds: models.refinerModelIds,
    judgeModelIds: models.judgeModelIds,
    gateModelIds,
    declaredNames: prepared.declaredNames,
    ...((prepared.declaredSubjectForms === undefined) ? {} : { declaredSubjectForms: prepared.declaredSubjectForms, }),
    ...((prepared.declaredNamePairs === undefined) ? {} : { declaredNamePairs: prepared.declaredNamePairs, }),
    definitions: collectDefinitions({
      document: parseDocument({ text: prepared.targetText, },),
    },),
  };
}

//endregion Consolidation polish configuration
