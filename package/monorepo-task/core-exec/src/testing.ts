/**
 Built testing seam for task utility implementation helpers.
 
 This sub-entry exists only so package tests exercise bundled code. Every
 re-exported declaration is internal and carries no compatibility promise.
 
 @module
 */

export {
  firstGlobMetaIndex,
} from './depends-resolve-glob.ts';
export {
  augmentOxlintOutput,
  extractRuleName,
  formatGuidanceLine,
  isHelpLine,
  NO_RULE,
  stripAnsi,
} from './oxlint-augment.ts';
export {
  fixUntilStable,
  MAX_AUTOFIX_PASSES,
  normalizeForConvergence,
  type OxlintRunResult,
} from './oxlint-fix-loop.ts';
export { RULE_GUIDANCE, } from './oxlint-guidance.ts';
export {
  downloadBytes,
  sha256Hex,
  type VersionAnswer,
  versionOf,
} from './mpm-shadow-lock-fetch.ts';
export {
  type MpmShadowSource,
  resolveLockedMpmShadow,
  type ResolvedMpmShadow,
} from './mpm-shadow-lock-logic.ts';
export {
  type ArchiveRow,
  BOOTSTRAP_CACHE_DIRECTORY,
  hostLockPlatformId,
  type LockedMpmShadow,
  lockPlatformId,
  MPM_SHADOW_TOOL_ID,
  MpmShadowLockError,
  parseVersionOutput,
  readLockedMpmShadow,
  TOOLCHAIN_BIN_LINK,
} from './mpm-shadow-lock-read.ts';
export {
  ensurePlaywrightImage,
  PLAYWRIGHT_IMAGE_PLATFORM,
  PLAYWRIGHT_LOCAL_IMAGE,
  PLAYWRIGHT_REGISTRY_IMAGE,
  PLAYWRIGHT_TAG_LENGTH,
  PlaywrightImageError,
  type PlaywrightImageSource,
  type Podman,
  type PodmanResult,
  playwrightImageTag,
} from './playwright-image-logic.ts';
export {
  filterPnpmOutput,
  isAllowedCycleWarning,
} from './pnpm-output-filter.ts';
export { buildTscArgs, } from './tsc-args.ts';
export {
  filterTscOutput,
  isContinuationLine,
  isDiagnosticLine,
  isNodeModulesDiagnostic,
} from './tsc-output-filter.ts';
