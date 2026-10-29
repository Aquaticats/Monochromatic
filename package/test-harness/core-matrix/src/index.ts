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
 @internal Container command building and OS specification parsing for
 built-artifact tests; not part of the supported matrix API.
 */
export {
  buildContainerCommand,
  parseOs,
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
