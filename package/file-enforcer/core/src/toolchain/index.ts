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
  writeToolEnvFile,
  type ExposureReport,
} from './exposure.ts';
export {
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
  worktreeRootFromModule,
  type ToolchainPaths,
} from './paths.ts';
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
  installMpmPnpm,
  installMpmUvx,
  installRustComponents,
  installRustToolchain,
  parseBinList,
  parseCargoBinstallList,
  releaseTagUrl,
  type ProviderContext,
} from './providers.ts';
export {
  renderArtifacts,
  writeArtifacts,
  type GenerateReport,
} from './generate.ts';
export {
  syncOrder,
  syncToolchain,
  appliesToHost,
  type SyncReport,
} from './sync.ts';
export {
  normalizeVersion,
  observeTool,
  observeTools,
  versionMatches,
  type ToolObservation,
} from './status.ts';
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
