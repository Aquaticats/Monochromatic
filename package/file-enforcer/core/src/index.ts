
export { applyCargoPlan, } from './cargo/apply-plan.ts';
export { manageRustCatalog, } from './cargo/catalog/manage-rust-catalog.ts';
export {
  RustCatalogSyncError,
  syncRustCatalog,
} from './cargo/catalog/sync.ts';
export type {
  ManifestFile,
  SyncIssue,
  SyncResult,
} from './cargo/catalog/types.ts';
export { manageCargoManifests, } from './cargo/manage-cargo-manifests.ts';
export type {
  CanonicalTomlValue,
  CargoBlockInsertion,
  CargoEnforcement,
  CargoManifestPlan,
  CargoManifestSpec,
} from './cargo/types.ts';
export {
  invalidatePaths,
  readCache,
  readCached,
  updateCache,
} from './io/cache.ts';
export {
  cat,
  globResults,
} from './io/cat.ts';
export type {
  GlobResult,
  GlobResults,
} from './io/cat.ts';
export { splitGlob, } from './io/glob-split.ts';
export {
  expandGlob,
  firstGlobMetaIndex,
  mirrorGlobPath,
} from './io/glob.ts';
export {
  publishLockOwnerPublication,
  removeLockOwnerPublication,
  stageLockOwnerPublication,
} from './io/staleness-manifest-lock-owner-publish.ts';
export { lockOwnerState, } from './io/staleness-manifest-lock-owner.ts';
export { acquireManifestLock, } from './io/staleness-manifest-lock.ts';
export { freshStalenessManifest, } from './io/staleness-run.ts';
export { manageMiseManifests, } from './mise/manage-mise-manifests.ts';
export {
  findNodeModulesRoot,
  NODE_MODULES_DIRECTORY_NAME,
} from './io/staleness-root.ts';
/**
 Re-exported so built-artifact tests build an in-memory filesystem whose
 types match the bundled copy of the root-discovery contract.

 @internal
 */
export { createMemoryRootFilesystem, } from '@monochromatic-dev/fs-path-core/ts';
/**
 Re-exported so built-artifact tests check thrown errors against the bundled
 copy of the TOML editor's error class, not a second copy loaded from source.

 @internal
 */
export { TomlEditError, } from '@monochromatic-dev/toml-edit-core/ts';
export type {
  ContentBuilder,
  GlobResultsBuilder,
  OverwriteContent,
  OverwriteEachFiles,
} from './io/write-lazy.ts';
export { writeFileAtomically, } from './io/write-atomic.ts';
export type { AtomicTempFileWriter, } from './io/write-atomic.ts';
export { overwriteTomlKey, } from './io/write-toml.ts';
export {
  ABSENT_FILE_CONTENT,
  overwrite,
  overwriteEach,
  overwriteIfNotExists,
  readExisting,
} from './io/write.ts';
export {
  buildLanguageSettingsEntry,
  buildUserDefinedEntry,
} from './jetbrains/lsp4ij-entries.ts';
export { manageLsp4ijServerSettings, } from './jetbrains/lsp4ij.ts';
export type {
  Lsp4ijBaseServerMatch,
  Lsp4ijConfigPatch,
  Lsp4ijOptionsFiles,
  Lsp4ijScopedServer,
  Lsp4ijServerSettings,
} from './jetbrains/lsp4ij-types.ts';
export {
  compareVersionParts,
  latestJetbrainsOptionsDirectory,
  NO_JETBRAINS_OPTIONS_DIRECTORY,
  NOT_A_MATCHING_PRODUCT,
  parseVersionParts,
} from './jetbrains/options-dir.ts';
export type { JetbrainsOptionsDirectory, } from './jetbrains/options-dir.ts';
export { l, } from './logger.ts';
export { tagged, } from '@monochromatic-dev/logger-core/ts';
export type { Logger, } from '@monochromatic-dev/logger-core/ts';
export {
  ensurePackage,
  registerPackages,
} from './package/ensure-package.ts';
export {
  MANAGER_KEYS,
  MANAGERS,
} from './package/manager-defs.ts';
export {
  binaryExists,
  canProvide,
  detectManager,
  installPackage,
  isRoot,
  NO_PACKAGE_MANAGER,
  resetManagerCache,
  resetRootCache,
} from './package/manager.ts';
export { mergeOverrides, } from './package/merge.ts';
export {
  DEFAULT_CHECK,
  p,
} from './package/p.ts';
export { firstWhitespaceToken, } from './package/registry-parse.ts';
export type {
  PackageEntry,
  PackageManager,
  PackageMapping,
  PackageSpec,
} from './package/types.ts';
export { exec, } from './pipeline/exec.ts';
export { inspect, } from './pipeline/inspect.ts';
export {
  formatJsonObject,
  isJsonObject,
  mergeFlatJson,
  mergeObjectDefaults,
  omitJsonKey,
  parseJsonObject,
} from './pipeline/json.ts';
export type {
  JsonObject,
  JsonValue,
} from './pipeline/json.ts';
export {
  editTomlKey,
  getTomlProperty,
} from './pipeline/toml.ts';
export {
  dedup,
  getJsonProperty,
} from './pipeline/transform.ts';
export {
  escapeXmlAttribute,
  isDigitCodePoint,
  unescapeXmlAttribute,
  xmlOptionLine,
} from './pipeline/xml-coding.ts';
export {
  ABSENT_XML_ENTRY,
  ABSENT_XML_VALUE,
  findXmlEntryByKey,
  getXmlOptionValue,
  listXmlEntries,
  replaceOrInsertXmlEntry,
} from './pipeline/xml.ts';
export type { XmlEntry, } from './pipeline/xml.ts';
export type {
  Command,
  PlatformCommands,
  PlatformEntry,
  Predicate,
} from './platform/evaluate-predicate.ts';
export { evaluatePredicate, } from './platform/evaluate-predicate.ts';
export {
  addWatchedPaths,
  captureTrackedSources,
  globs,
  reads,
  reset,
  resetWriteTimestamps,
  setWriteTimestamp,
  trackDest,
  trackGlob,
  trackRead,
  trackWriteTime,
  writes,
  writeTimestamps,
} from './tracker.ts';
export type {
  CapturedSources,
  SourceCaptureCallback,
  TrackedGlob,
} from './tracker.ts';
/**
 Re-exported so the toolchain's built-artifact tests reach the toolchain
 modules through the shipped bundle instead of their source files.

 @internal
 */
export {
  ALL_TOOLS,
  appliesToHost,
  applyResolutions,
  BIN_BOOTSTRAP,
  buildBinPathsOutput,
  formatReport,
  hostPlatformId,
  installArchive,
  installBinSourced,
  installCargoBinstall,
  installCargoGit,
  installGoInstall,
  installMpmPnpm,
  loadPlanFromDir,
  mapGithubRows,
  MPM_BOOTSTRAP,
  normalizeVersion,
  NOT_INSTALLED,
  observeTool,
  parseBinList,
  parseCargoBinstallList,
  parseShasums,
  patchPlanVersions,
  rebuildExposure,
  ReleaseTagUrlError,
  releaseTagUrl,
  renderArtifacts,
  renderMpmManagersToml,
  renderMpmPackagesToml,
  renderPlanArtifactsTs,
  renderToolchainLockJson,
  replacementWarning,
  resolveAll,
  resolveGoModule,
  resolveGoRuntime,
  resolveNpm,
  resolveRust,
  resolveRustup,
  resolveToolchainPaths,
  resolveZigLatest,
  run,
  runPostinstallCommands,
  runPostinstallLinks,
  seedBinConfig,
  sha256File,
  syncOrder,
  TOOL_ROUTES,
  toolById,
  toolchainLockFromTools,
  toolInstallDir,
  toolPlanInstallableNames,
  TOOLS,
  TOOLS_BY_ID,
  tomlKey,
  tomlString,
  versionFromTag,
  versionMatches,
} from './toolchain/index.ts';
/**
 Toolchain types the built-artifact tests name.

 @internal
 */
export type {
  Fetcher,
  PathEnv,
  ProviderContext,
  ToolArtifacts,
  ToolchainPaths,
  ToolPlanEntry,
  UpdateOutcome,
} from './toolchain/index.ts';
export type { Path, } from './types.ts';
export { notifyWriteProtection, } from './watch/notify.ts';
export {
  DEBOUNCE_MS,
  watchDirectory,
} from './watch/watch-dir.ts';
export {
  createWatchRerunQueue,
} from './watch/watch-rerun-queue.ts';
export type {
  WatchRerunBatch,
  WatchRerunErrorHandler,
  WatchRerunHandler,
  WatchRerunQueue,
  WatchRerunReporterLogger,
} from './watch/watch-rerun-queue.ts';
export type { EventKind, } from './watch/watch-filter.ts';
export {
  classifyEvent,
  shouldTrigger,
  watchDirs,
} from './watch/watch-filter.ts';
export { startWatching, } from './watch/watch.ts';
