/**
 Runtime CSS mixin expansion for web component Shadow DOM styles.
 
 Web components can't inherit global CSS. Each component calls
 `css(\`...\`)` with `@apply --mixin-name` rules; the mixin bodies
 are expanded at runtime using the build-css pipeline.
 
 The mixin definitions are imported as a text string (bundled inline
 by tsdown) so no filesystem access is needed at runtime.
 */
// Browser bundles import the expand module directly: the package index also
// re-exports the node-only buildCss pipeline, whose module-fs-path imports
// would drag node builtins into the client bundle.
import { expandCssMixins, } from '@monochromatic-dev/css-cli/ts/expand';
// tsdown inlines the CSS file content as a string at bundle time
import mixinSource from './mixins.css' with { type: 'text', };

/**
 Expands \@apply references in a CSS string using shared mixin definitions.
 
 Delegates to build-css's high-level expandCssMixins() so this module never
 touches a CSS parser directly; all parse/expand/serialize logic lives in
 one place.
 
 @param raw - CSS string containing \@apply references
 
 @returns Expanded CSS with all \@apply rules replaced by mixin bodies
 
 @example
 ```ts
 const styles = css(':host { \@apply --flex-column; }');
 ```
 */
export function css(raw: string,): string {
  return expandCssMixins({
    css: raw,
    mixinCss: mixinSource,
  },);
}
