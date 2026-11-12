/**
 The one owner of the TSDoc block-tag list every plugin scanner asks about.

 The `@mutates` contract scanner in this package and the TSDoc block scanner
 in `@monochromatic-dev/oxlint-plugin-tsdoc` ask the same question of a comment
 line, "does this tag end the block that is open?", so the answer lives here
 once: a tag added or removed below reaches both scanners, and neither can
 start disagreeing about where a block ends.

 @module
 */

/**
 Block tags whose appearance at the start of an unfenced comment line ends the
 open block.
 
 Holds every standard TSDoc block tag plus the project's own `@mutates` and
 `@yields` tags. Modifier and inline tags (`@internal`, `@readonly`,
 `@inheritDoc`, `@link`) are absent on purpose: they open no block, so they
 remain part of the preceding block's text.
 */
export const BLOCK_TAG_NAMES: ReadonlySet<string> = new Set([
  '@decorator',
  '@defaultValue',
  '@deprecated',
  '@example',
  '@param',
  '@privateRemarks',
  '@remarks',
  '@returns',
  '@see',
  '@throws',
  '@typeParam',
  '@jsx',
  '@jsxRuntime',
  '@jsxFrag',
  '@jsxImportSource',
  '@mutates',
  '@yields',
]);
