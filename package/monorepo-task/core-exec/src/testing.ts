/**
 Built testing seam for task utility implementation helpers.
 
 This sub-entry exists only so package tests exercise bundled code. Every
 re-exported declaration is internal and carries no compatibility promise.
 
 @module
 */

export { executeWithCollapsedOutput, } from './depends-exec.ts';
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
  detectNativeAnalysisCrash,
  NO_NATIVE_ANALYSIS_CRASH,
  type NativeAnalysisCrash,
} from './oxlint-analysis-crash.ts';
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
  LOCK_FILE_NAME,
  lockFilePath,
  type LockedMpmShadow,
  lockPlatformId,
  MPM_SHADOW_TOOL_ID,
  MpmShadowLockError,
  olderLockNote,
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
  type CapturedSubprocessFailure,
  AnalysisCrashError,
  isCapturedSubprocessFailure,
  ToolKilledBySignalError,
  ToolNotStartedError,
} from './tool-failure.ts';
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
export {
  CHECK_PASS_DEADLINE_MS,
  RUST_CHECK_PARALLEL,
  TYPESCRIPT_CHECK_PARALLEL,
} from './daily-update-check-bounds.ts';
export { compareChecks, } from './daily-update-check-comparison.ts';
export { runChecks, } from './daily-update-check-run.ts';
export {
  runUpdateWithChecks,
  type UpdateWithChecksResult,
} from './daily-update-check-phase.ts';
export {
  issueMarker,
  MAXIMUM_MOVE_LINES,
  renderIssue,
} from './daily-update-issue-text.ts';
export { publishIssues, } from './daily-update-issue-publish.ts';
export {
  causeOf,
  firstErrorOf,
  NO_ERROR_IN_OUTPUT,
} from './daily-update-first-error.ts';
export {
  cargoLockMoves,
  pnpmImporterMoves,
  toolchainLockMoves,
} from './daily-update-lock-moves.ts';
export { checksFromTaskNames, } from './daily-update-package-check-discovery.ts';
export type {
  CheckComparison,
  CheckEffects,
  CheckOutcome,
  CheckReport,
  CheckResult,
  FirstError,
  GhRequest,
  GhResult,
  LockMove,
  MoveSource,
  PackageCheck,
  PublishOutcome,
  RenderedIssue,
} from './daily-update-package-check-types.ts';
export { LISTED_ISSUE_LINES, } from './daily-update-check-after-update.ts';
export { notifierEnvironment, } from './daily-update-effects.ts';
