/**
 Public surface of the toolchain modules: the typed plan, generators, path
 algebra, providers, sync and status entry points.
 */

export {
  buildBinPathsOutput,
  type BinPathsOutput,
} from './bin-paths.ts';
export {
  downloadVerified,
  extractArchive,
  run,
  sha256File,
  type RunResult,
} from './exec.ts';
export {
  rebuildExposure,
  runPostinstallCommands,
  runPostinstallLinks,
  unresolvableBins,
  type ExposureReport,
} from './exposure.ts';
export { writeToolEnvFile, } from './exposure.env.ts';
export {
  BIN_NOT_FOUND,
  hostPlatformId,
  findToolBin,
  type BinSearchContext,
} from './layout.ts';
export {
  renderToolchainLockJson,
  toolchainLockFromTools,
} from './lock.ts';
export {
  renderMpmManagersToml,
  renderMpmPackagesToml,
  tomlKey,
  tomlString,
} from './mpm-manifest.ts';
export {
  EXPOSURE_DIR_RELATIVE,
  resolveToolchainPaths,
  toolchainRootFromEnv,
  toolInstallDir,
  worktreeRootFromModule,
  type PathEnv,
  type ToolchainPaths,
} from './paths.ts';
export type { ToolArtifacts, } from './plan.artifacts.data.ts';
export { loadPlanFromDir, } from './plan.load.ts';
export {
  ALL_TOOLS,
  BIN_BOOTSTRAP,
  MPM_BOOTSTRAP,
  TOOLS,
  TOOLS_BY_ID,
  toolById,
  toolPlanInstallableNames,
} from './plan.ts';
export {
  installAndroidSdk,
  installArchive,
  installBinSourced,
  installCargoBinstall,
  installCargoGit,
  installGoInstall,
  installMpmPnpm,
  installMpmUvx,
  installRustComponents,
  installRustToolchain,
  parseBinList,
  parseCargoBinstallList,
  ReleaseTagUrlError,
  releaseTagUrl,
  seedBinConfig,
  type ProviderContext,
} from './providers.ts';
export {
  renderArtifacts,
  writeArtifacts,
  type GenerateReport,
} from './generate.ts';
export {
  replacementWarning,
  syncToolchain,
  type SyncReport,
} from './sync.ts';
export {
  decideUpdatesSlice,
  insideUpdatesSlice,
  NOT_RELAUNCHED,
  relaunchInUpdatesSlice,
  runChild,
  systemdRunArguments,
  TOOLCHAIN_UPDATES_SLICE,
  UPDATES_SLICE_GUARD_ENVIRONMENT,
  UPDATES_SLICE_OPT_OUT_ENVIRONMENT,
  type ChildOutcome,
  type SystemdRunner,
  type UpdatesSliceFacts,
} from './updates-slice.ts';
export {
  appliesToHost,
  syncOrder,
} from './sync.order.ts';
export { NOT_INSTALLED, } from './status.observe.ts';
export {
  normalizeVersion,
  observeTool,
  observeTools,
  versionMatches,
  type ToolObservation,
} from './status.ts';
export { isNewerVersion, } from './version.order.ts';
export {
  isRepositoryPublished,
  keptNewerWarning,
  lockOutcome,
  REPOSITORY_CRATE_PREFIX,
  type LockOutcome,
} from './lock.outcome.ts';
export {
  TOOL_ROUTES,
  type PostinstallLink,
  type ToolchainLock,
  type ToolchainLockTool,
  type ToolEnv,
  type ToolOptions,
  type ToolPlanEntry,
  type ToolRequest,
  type ToolRoute,
} from './types.ts';
export {
  patchPlanVersions,
  renderPlanArtifactsTs,
} from './update.render.ts';
export {
  mapGithubRows,
  parseShasums,
  resolveAws,
  resolveGoModule,
  resolveGoRuntime,
  resolveNpm,
  resolveRust,
  resolveRustup,
  resolveZigLatest,
  versionFromTag,
  type Fetcher,
} from './update.resolvers.ts';
export {
  applyResolutions,
  formatReport,
  resolveAll,
  type UpdateOutcome,
} from './update.ts';
