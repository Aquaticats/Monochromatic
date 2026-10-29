/**
 Provider installs: one function per toolchain route, each pinned to
 `toolchain.lock.json` and idempotent (the caller probes observed state first).

 The routes live in sibling modules; this module is the single import point.
 */

export {
  installArchive,
  restoreExecBits,
} from './providers.archive.ts';
export {
  installBinSourced,
  ReleaseTagUrlError,
  releaseTagUrl,
  seedBinConfig,
} from './providers.bin.ts';
export {
  installCargoBinstall,
  installCargoGit,
} from './providers.cargo.ts';
export {
  artifactForHost,
  planTool,
  type ProviderContext,
} from './providers.context.ts';
export {
  binCliPath,
  NO_GITHUB_TOKEN,
  resolveGithubToken,
  toolchainPathEnv,
} from './providers.environment.ts';
export {
  installGoInstall,
  installMpmPnpm,
  installMpmUvx,
} from './providers.manager.ts';
export {
  parseBinList,
  parseCargoBinstallList,
} from './providers.parse.ts';
export {
  installAndroidSdk,
  installRustComponents,
  installRustToolchain,
} from './providers.rust-android.ts';
