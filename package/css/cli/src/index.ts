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
  UnknownCssMixinError,
} from './errors.ts';
export { expandCssMixins, } from './expand.ts';
