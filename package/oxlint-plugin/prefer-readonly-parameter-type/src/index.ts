import {
  eslintCompatPlugin,
  type Plugin,
} from '@oxlint/plugins';

import { noInvalidParameterEffectContracts, } from './no-invalid-parameter-effect-contracts.ts';
import { noOpaqueParameterEffects, } from './no-opaque-parameter-effects.ts';
import { noReadonlyParameterMutations, } from './no-readonly-parameter-mutations.ts';
import { preferReadonlyParameterTypes, } from './prefer-readonly-parameter-types.ts';
import { initializeExternalImplementationApi, } from './prefer-readonly-parameter-types/external-implementation-project.ts';
import { initializeSemanticBridge, } from './prefer-readonly-parameter-types/semantic-bridge-lifecycle.ts';

export {
  classifyReadonlyType,
  propertyIsReadonly,
  type ReadonlyClassification,
} from './prefer-readonly-parameter-types/readonly-classifier.ts';

export {
  isTerminalSemanticChannelFailure,
} from './prefer-readonly-parameter-types/semantic-channel-failure.ts';

export {
  closeSemanticBridge,
  initializeSemanticBridge,
  retireSemanticBridgeAfterChannelFailure,
} from './prefer-readonly-parameter-types/semantic-bridge-lifecycle.ts';

export {
  openSemanticFile,
  semanticBridgeCacheStats,
  type SemanticBridgeCacheStats,
  type SemanticFileSession,
} from './prefer-readonly-parameter-types/typescript-sync-adapter.ts';

export {
  clearEffectSummaryCache,
  effectSummaryCacheStats,
  type EffectSummaryCacheStats,
} from './prefer-readonly-parameter-types/effect-summary-cache.ts';

export {
  clearFinalEffectIndexCache,
  finalEffectIndexCacheStats,
  type FinalEffectIndexCacheStats,
} from './prefer-readonly-parameter-types/effect-final-index-cache.ts';

export {
  readonlySourceEvidence,
  readonlySourceEvidenceCacheStats,
  type ReadonlySourceEvidence,
  type ReadonlySourceEvidenceCacheStats,
} from './prefer-readonly-parameter-types/readonly-source-evidence.ts';

export {
  buildEffectSummaryIndex,
  NO_EFFECT_SUMMARY,
  type CallableEffectSummary,
  type EffectSummaryIndex,
} from './prefer-readonly-parameter-types/effect-summaries.ts';

export {
  SemanticBridgeError,
  type SemanticBridgeFailureReason,
} from './prefer-readonly-parameter-types/semantic-bridge-error.ts';

export { configuredProjectConfigFileName, } from './prefer-readonly-parameter-types/semantic-file-name.ts';

export {
  findNodeAtOffset,
  typescriptOffset,
} from './prefer-readonly-parameter-types/typescript-node-map.ts';

export { VERIFIED_READER_COUNT, } from './prefer-readonly-parameter-types/effect-default-library-reader-authority.ts';

export {
  parameterBindingSlots,
  type ParameterSlotTable,
  parameterSlotTable,
  slotsByParameterFrom,
} from './prefer-readonly-parameter-types/effect-parameter-slots.ts';

export {
  asEffectSlot,
  asParameterIndex,
  canonicalPropertyKey,
  type EffectSlot,
  NOT_A_STATIC_KEY,
  type ParameterIndex,
} from './prefer-readonly-parameter-types/effect-slot-identity.ts';

initializeSemanticBridge();
/* Beside it, and for the same reason. The external implementation child was created on first demand,
 * which happens mid-lint after oxlint has reserved its per-worker buffers, and that spawn fails with
 * `ENOMEM` at this host's default worker count. */
initializeExternalImplementationApi();

/**
 Pure origin-resolution classifier, exported for built-artifact branch tests.
 
 @internal
 */
export {
  readonlyTypeOriginEvidence,
  readonlyTypeOriginEvidenceFromResolution,
  type ReadonlyTypeOriginEvidence,
  type ReadonlyTypeOriginResolution,
} from './prefer-readonly-parameter-types/readonly-type-origin.ts';

/**
 Records of omissions inside installed packages and the ledger answering which a summary depends on,
 exported for their built-artifact branch tests.

 @internal
 */
export {
  EXTERNAL_OMISSIONS_INVALID,
  type ExternalOmissionRecord,
  mergeExternalOmissions,
  parseExternalOmissionRecords,
} from './prefer-readonly-parameter-types/effect-external-omission.ts';

/**
 Ledger of loaded sources and the edges expansion followed,
 exported for its built-artifact branch tests.

 @internal
 */
export { createOmissionLedger, } from './prefer-readonly-parameter-types/effect-omission-ledger.ts';

/**
 Guidance text chosen from origin evidence,
 exported for the built-artifact origin classification test.

 @internal
 */
export { readonlyPreferenceGuidance, } from './prefer-readonly-parameter-types/readonly-preference-guidance.ts';

/**
 Oxlint plugin separating readonly preference, mutation, opacity, and contract policies.
 
 @example
 ```typescript
 // oxlint.config.ts
 import { defineConfig } from 'oxlint';
 export default defineConfig({
   jsPlugins: ['\@monochromatic-dev/oxlint-plugin-prefer-readonly-parameter-type'],
 });
 ```
 */
const plugin: Plugin = eslintCompatPlugin({
  meta: {
    name: 'prefer-readonly-parameter-type',
  },
  rules: {
    'prefer-readonly-parameter-types': preferReadonlyParameterTypes,
    'no-readonly-parameter-mutations': noReadonlyParameterMutations,
    'no-opaque-parameter-effects': noOpaqueParameterEffects,
    'no-invalid-parameter-effect-contracts': noInvalidParameterEffectContracts,
  },
},);

export default plugin;

/**
 Verified inert collection members, exported for the built-artifact probe test.
 
 @internal
 */
export {
  ITERATOR_MEMBER_NAMES,
  MEMBER_CHANNEL_INTERNAL_SLOT,
  MEMBER_CHANNEL_RECEIVER_INDEX,
  MEMBER_CHANNEL_RECEIVER_INDEX_AND_COERCION,
  MEMBER_CHANNEL_RECEIVER_INDEX_AND_SPECIES,
  MEMBER_CHANNELS_BY_INTERFACE,
  memberInvokesObserver,
  OBSERVER_BEARING_MEMBER_NAMES,
  VERIFIED_MEMBER_CHANNEL_COUNT,
} from './prefer-readonly-parameter-types/effect-member-channel-authority.ts';

/**
 External formal-to-actual mapping, exported for its own test.
 
 No shape in the fixture corpus reaches it through a diagnostic, because that needs an installed
 package with a locked version whose shipped implementation provably mutates a formal, invoked with
 a spread. A mutant restoring the previous actual-position indexing survived the whole suite, which
 is what this export answers.
 
 @internal
 */
export { formalArgumentIndexes, } from './prefer-readonly-parameter-types/effect-external-application.ts';

/**
 Per-snapshot request memo, exported so its test can exercise built output.

 @internal
 */
export {
  MAXIMUM_MEMOIZED_RESPONSES_PER_SNAPSHOT,
  MEMOIZED_SNAPSHOT_REQUESTS,
  memoizeApiSnapshotRequests,
  memoizeSnapshotRequests,
  type SnapshotRequestClient,
  type SnapshotRequestMemo,
  type SnapshotRequestMemoStats,
} from './prefer-readonly-parameter-types/semantic-request-memo.ts';

/**
 Ancestor directory walk, exported so its test can exercise built output.
 
 @internal
 */
export { ancestorDirectories, } from './prefer-readonly-parameter-types/ancestor-directories.ts';

/**
 Persistent cache bound enforcement, exported so its test can exercise built output.

 @internal
 */
export { maintainPersistentEffectCache, } from './prefer-readonly-parameter-types/effect-summary-cache-maintenance.ts';

/**
 Reachability of the structural projection helper, exported for its built-artifact test.
 
 @internal
 */
export {
  clearTypeFestReachabilityCache,
  typeFestResolvesFrom,
} from './prefer-readonly-parameter-types/type-fest-reachability.ts';

/**
 Verified member result relations, exported for the built-artifact probe test.
 
 @internal
 */
export {
  UNPAIRED_VIEW_INTERFACES,
  VERIFIED_UNPAIRED_VIEW_COUNT,
} from './prefer-readonly-parameter-types/effect-unpaired-view-authority.ts';

export {
  FRESH_CONTAINER_MEMBER_NAMES,
  RESULT_PROVENANCE_BY_INTERFACE,
  RESULT_RELATION_OBSERVER_RETURN,
  RESULT_RELATION_RECEIVER_ELEMENTS_PAIRED,
  RESULT_RELATION_RECEIVER_VALUE,
  VERIFIED_RESULT_RELATION_COUNT,
} from './prefer-readonly-parameter-types/effect-result-provenance-authority.ts';

/**
 Cross-project analysis key, project fingerprint and shared-hit verification,
 exported so the sharing tests can read a key's fallback reason and components from built output.
 
 @internal
 */
export {
  createSharedKeyResolver,
  type SharedKey,
} from './prefer-readonly-parameter-types/shared-analysis-key.ts';

export {
  type EffectProjectFingerprint,
  effectProjectFingerprint,
} from './prefer-readonly-parameter-types/effect-project-fingerprint.ts';

export { verifySharedHit, } from './prefer-readonly-parameter-types/shared-analysis-verification.ts';

export {
  VERIFY_SHARED_DIRECTORY_ENVIRONMENT_VARIABLE,
  VERIFY_SHARED_ENVIRONMENT_VARIABLE,
} from './prefer-readonly-parameter-types/shared-analysis-telemetry.ts';
