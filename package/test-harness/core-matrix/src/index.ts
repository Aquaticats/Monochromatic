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
 @internal Container command building, OS specification parsing and image resolution for
 built-artifact tests; not part of the supported matrix API.
 */
export {
  buildContainerCommand,
  parseOs,
  resolveImage,
} from './container.ts';

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
