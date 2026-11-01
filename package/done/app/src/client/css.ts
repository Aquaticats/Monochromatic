/**
 Re-exports the h-css hyperscript function for CSS generation.
 
 Replaces the PostCSS-based `@apply` mixin expansion with pure
 TypeScript function composition via `mixins.ts`.
 */
export { hCss as $, } from '@monochromatic-dev/hyperscript-core/ts';
