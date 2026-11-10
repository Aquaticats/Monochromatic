/**
 Built entry the unit tests import.

 The package entry registers this pi extension's session handlers when it loads, so a
 test cannot import it. This module re-exports the model policy, the apply step, and
 the global-settings restoration the suites drive, so they load what this package
 builds rather than sibling source, as `test-import/require-eventual-artifact`
 requires.

 `rolldown.node.config.ts` builds this file to `dist/final/node/testing.mjs`, beside
 the extension bundle. Nothing in this package imports it, and pi never loads it.

 @module
 */

//region Model policy

/**
 The model-id rules that decide an extension's thinking default.

 @internal
 */
export {
  getModelIdLeaf,
  getThinkingDefaultForModel,
  isGptModelId,
  isXhighAvailable,
  type ThinkingDefaultLevel,
  type ThinkingLevelMapFragment,
} from './model-policy.ts';

//endregion Model policy

//region Applying the default

/**
 The write that applies a policy decision to a session's thinking level.

 @internal
 */
export {
  applyThinkingDefault,
  type ApplyThinkingDefaultResult,
} from './apply-thinking-default.ts';

//endregion Applying the default

//region Global settings

/**
 Reading and restoring the user's persisted global thinking default.

 @internal
 */
export {
  getGlobalSettingsPath,
  PERSISTED_DEFAULT_THINKING_LEVEL,
  restoreGlobalDefaultThinkingLevel,
} from './global-settings.ts';

//endregion Global settings
