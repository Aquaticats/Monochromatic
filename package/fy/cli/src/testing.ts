/**
 Built entry the unit tests import.

 The package's only other entry is the `cli-fy` bin, which parses `process.argv`
 and exits, so no test can drive an argument coercion or a specifier resolution
 through it. This module re-exports exactly the two functions the tests exercise,
 so they load the artifact this package builds rather than sibling source, as
 `test-import/require-eventual-artifact` requires.

 Nothing in this package imports it, and no consumer needs it: these exports are
 not part of the package's API.

 `rolldown.node.config.ts` builds this file to `dist/final/node/testing.mjs`,
 beside the bin bundle.

 @module
 */

//region Argument coercion

/**
 Conversion of one CLI argument string to the value the called export receives.

 @internal
 */
export { coerceArg, } from './coerce.ts';

//endregion Argument coercion

//region Specifier resolution

/**
 Resolution of the CLI's specifier to a loadable module path.

 @internal
 */
export { resolveSpecifier, } from './resolve.ts';

//endregion Specifier resolution
