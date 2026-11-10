/**
 Test matrix runner.

 The public surface is {@link matrix} and its option types. The command
 builders below are `@internal` exports so the package's tests can exercise
 the built artifact instead of sibling source.
 */

export { matrix, } from './matrix.ts';
export type {
  Combination,
  ExcludeEntry,
  ExcludeMatcher,
  MatrixOptions,
  PackageManager,
  ParsedOs,
  Runtime,
  UserContext,
} from './types.ts';

/**
 @internal Runtime install and exec command builders for built-artifact tests;
 not part of the supported matrix API.
 */
export {
  runtimeExecCommand,
  runtimeInstallCommand,
} from './runtime.ts';

/**
 @internal Container command building, OS specification parsing, image resolution and the container backend itself for
 built-artifact tests; not part of the supported matrix API.
 */
export {
  buildContainerCommand,
  containerRunArguments,
  parseOs,
  resolveImage,
  runContainer,
} from './container.ts';

/**
 @internal The host backend, for built-artifact tests that check what it does with a run's stderr; not part of the
 supported matrix API.
 */
export { runHost, } from './host.ts';

/**
 @internal This package's own library sources with their comments blanked, for the source scans of its built-artifact
 tests; not part of the supported matrix API.
 */
export { librarySourceTexts, } from './library-source-text.ts';
export type { LibrarySource, } from './library-source-text.ts';

/**
 The read-only mount that lets any container resolve this checkout's installed packages, for every task in the
 repository that runs a JavaScript tool against the mounted checkout.
 */
export {
  pnpmStoreDirectory,
  pnpmStoreMountArguments,
  STORE_DIRECTORY_FROM_HOME,
  STORE_DIRECTORY_VARIABLE,
} from './pnpm-store-mount.ts';

/**
 @internal Distro detection and per-manager command builders for
 built-artifact tests; not part of the supported matrix API.
 */
export {
  detectPackageManager,
  prerequisiteCommand,
  runtimeDependencyCommand,
  userCreationCommand,
} from './distro.ts';

/**
 @internal Default `*.unit.matrix.test.ts` discovery for built-artifact tests;
 not part of the supported matrix API.
 */
export { discoverTestFiles, } from './discover.ts';

/**
 CachyOS base image selection by host CPU level, with the Alpine fallback for hosts that are not x86-64.
 Supported API for container builds elsewhere in the repository.
 */
export {
  ALPINE_IMAGE,
  CACHYOS_IMAGE_BUILD_ARGUMENT_NAME,
  CACHYOS_IMAGE_BY_LEVEL,
  cachyosBuildArguments,
  detectCachyosLevel,
  LoaderUnavailableError,
  NoAlpineVariantError,
  readLoaderOutput,
  resolveBaseImage,
} from './cachyos-image.ts';
export type {
  BaseImageKind,
  BaseImageResolution,
  CachyosLevel,
  ResolveBaseImageOptions,
} from './cachyos-image.ts';

/**
 @internal Argument parsing core of the `cachyos-image` bin, exported for built-artifact tests.
 */
export {
  CACHYOS_IMAGE_USAGE,
  runCachyosImageCli,
} from './cachyos-image-cli.ts';
export type { CachyosImageCliResult, } from './cachyos-image-cli.ts';

/**
 The Rust toolchain `toolchain.lock.json` records, for container builds that install Rust themselves.
 */
export {
  LockedRustToolchainError,
  lockedRustToolchainFromText,
  readLockedRustToolchain,
  RUST_TOOLCHAIN_BUILD_ARGUMENT_NAME,
  TOOLCHAIN_LOCK_FILE_NAME,
} from './locked-rust-toolchain.ts';

/**
 Build arguments that hand the lock row of Zig or `cargo-about` to a container build that installs the tool itself.
 */
export {
  LOCK_FILE_NAME,
  LOCKED_BUILD_ARGUMENT_TOOLS,
  LockedToolBuildArgumentsError,
  lockedToolBuildArgumentsFromText,
  readLockedToolBuildArguments,
  ZIG_LOCK_PLATFORM_BY_HOST_ARCHITECTURE,
} from './locked-tool-build-arguments.ts';
export type { LockedBuildArgumentTool, } from './locked-tool-build-arguments.ts';

/**
 Builds a local container image again when the inputs it was built from, labelled on the image, have changed.
 */
export {
  buildContainerImage,
  ensureContainerImage,
  IMAGE_BUILD_INPUTS_LABEL_NAME,
  inputsDigestOf,
  isContainerImageCurrent,
  podmanEngineWith,
} from './container-image-rebuild-on-changed-inputs.ts';
export type {
  ContainerEngine,
  ContainerImageInputs,
  ContainerImageOptions,
  ImageLabelQuery,
} from './container-image-rebuild-on-changed-inputs.ts';
