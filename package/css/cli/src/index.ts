/**
 CSS build tool: monorepo-aware `\@import` bundling plus `\@mixin`/`\@apply`
 expansion over the byte-preserving css-edit CST.
 
 Two entry points:
 - {@link buildCss}: file pipeline (read, inline imports, expand mixins, write).
 - {@link expandCssMixins}: pure text pipeline for in-memory CSS
   (Shadow DOM styles in the browser); no filesystem, no process globals.
 
 Browser bundles must import `expandCssMixins` from
 `\@monochromatic-dev/css-cli/ts/expand` directly: this index also
 re-exports the node-only file pipeline, whose path utilities would pull
 node builtins into a client bundle.
 
 @packageDocumentation
 */

export {
  buildCss,
  type CssBuildOptions,
} from './build.ts';
export {
  CircularCssMixinError,
  CssPackageManifestError,
  UnknownCssMixinError,
} from './errors.ts';
export { expandCssMixins, } from './expand.ts';

//region Pieces the built artifact exposes for its tests
//
// Each export below is a step the pipeline runs, exposed so a unit test of
// the built artifact can drive that step on its own input rather than infer
// it from a whole build. Nothing here is a promise to consumers: the
// supported entry points are the two above.

export {
  existsSync,
  readTextFile,
  readTextFileSync,
  writeTextFileEnsuringParentDirectory,
} from './fs.ts';
export { fsRegistry, } from './fs-registry.ts';
export {
  findPackageDir,
  readPackageJson,
  resolveExports,
  resolvePackage,
} from './package-resolver.ts';
export {
  isPackageSpecifier,
  splitPackageSpecifier,
} from './specifier.ts';

//endregion Pieces the built artifact exposes for its tests
