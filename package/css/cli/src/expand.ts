import {
  asCssSource,
  parseCss,
  stringifyNodes,
} from '@monochromatic-dev/css-core-edit/ts';
import {
  collectMixins,
  expandApplyRules,
  expandMixinRegistry,
} from './mixin.ts';

/**
 Expands `\@mixin`/`\@apply` in CSS text, in memory: definitions are
 collected from `mixinCss` (when given) and from `css` itself, nested
 references between definitions resolve first, and every `\@apply` in `css`
 is replaced by its mixin body. Definitions disappear from the output.
 
 Browser-safe: no filesystem access, no `\@import` resolution, no process
 globals. Untouched CSS survives byte-exactly, comments included.
 
 @param css - CSS text whose `\@apply` references get expanded.
 
 @param mixinCss - Optional CSS text carrying shared `\@mixin` definitions
 (Shadow DOM consumers bundle these as a string import).
 
 @returns Expanded CSS text.
 
 @throws UnknownCssMixinError when an `\@apply` references an unregistered
 mixin.
 
 @throws CircularCssMixinError when definitions reference each other in a
 cycle.
 
 @throws CssParseError when either input is not parseable CSS.
 
 @example
 ```ts
 const expanded = expandCssMixins({
   css: '.btn { \@apply --card; }',
   mixinCss: '\@mixin --card { padding: 1rem; }',
 },);
 // => '.btn { padding: 1rem; }'
 ```
 */
export function expandCssMixins({
  css,
  mixinCss = '',
}: {
  readonly css: string;
  readonly mixinCss?: string;
},): string {
  /**
   Definitions collected from the shared mixin source.
   */
  const fromMixinCss = collectMixins({
    root: parseCss({ source: asCssSource(mixinCss,), },)
      .root,
  },);

  /**
   Consumer sheet with its own inline definitions collected out.
   */
  const fromCss = collectMixins({
    root: parseCss({ source: asCssSource(css,), },)
      .root,
  },);

  /**
   Combined registry; inline definitions in `css` override shared ones on
   name collision, matching nearest-definition-wins intuition.
   */
  const mixins = expandMixinRegistry({
    mixins: new Map([
      ...fromMixinCss.mixins,
      ...fromCss.mixins,
    ],),
  },);

  return stringifyNodes({
    nodes: expandApplyRules({
      root: fromCss.root,
      mixins,
    },)
      .children,
  },);
}
