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
  VERSION_TIMEOUT_MS,
  type VersionAnswer,
  versionOf,
} from './mpm-shadow-lock-fetch.ts';
export {
  findProgramUnder,
  isFinishedVersion,
  listVersionNames,
  STALE_LEFTOVER_AGE_MS,
} from './mpm-shadow-lock-folder.ts';
export { installLockedArchive, } from './mpm-shadow-lock-install.ts';
export {
  finishLayout,
  publishStaged,
} from './mpm-shadow-lock-publish.ts';
export {
  type MpmShadowSource,
  resolveLockedMpmShadow,
  type ResolvedMpmShadow,
} from './mpm-shadow-lock-logic.ts';
export {
  type ArchiveRow,
  checkVersionName,
  COMPLETE_MARKER,
  compareVersions,
  CURRENT_LINK_NAME,
  hostLockPlatformId,
  type LockedMpmShadow,
  lockPlatformId,
  MPM_SHADOW_TOOL_ID,
  MpmShadowLockError,
  parseVersionOutput,
  readLockedMpmShadow,
  TOOL_DIRECTORY_NAME,
  TOOLCHAIN_BIN_LINK,
  toolchainRootOf,
  toolchainRootsOf,
  UPGRADE_COMMAND,
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
export {
  dailyUpdateGroups,
  INSTALL_TIMEOUT_MS,
  miseExec,
  miseTask,
  type PathSelector,
  RESOLVE_TIMEOUT_MS,
  selects,
  type UpdateCommand,
  type UpdateGroup,
  type UpdateStep,
} from './daily-update-plan.ts';
export {
  absoluteOf,
  type CommandOutcome,
  type DailyUpdateEffects,
  type GroupReport,
  relativeOf,
  restoreFiles,
  runDailyUpdate,
  runGroup,
  runStep,
} from './daily-update-run.ts';
export {
  findOnPath,
  renderUnit,
  SERVICE_LIMITS,
  SERVICE_UNIT_NAME,
  TIMER_UNIT_NAME,
  UNIT_SOURCE_DIRECTORY,
} from './daily-update-timer-install-logic.ts';
export {
  isAlive,
  takeRunLock,
} from './daily-update-lock.ts';
export { runCommandBounded, } from './daily-update-command.ts';
