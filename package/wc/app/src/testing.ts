/**
 Built entry the unit tests import.

 The site keeps running from source (`node src/build.ts`) and builds a client bundle
 and one HTML file, so no module under test has an artifact to load. This entry
 re-exports exactly the render, favicon, style, and statistic behavior the unit suites
 drive, so they load what this package builds rather than sibling source, as
 `test-import/require-eventual-artifact` requires.

 `rolldown.node.config.ts` alone builds it, to `dist/final/node/testing.mjs`. Nothing
 in this package imports it, and the generated site never loads it.

 @module
 */

//region Statistics

/**
 Text statistics and word frequency, computed in the browser and here.
 */
export { analyzeText, } from './stat/analyze.ts';
export { computeFrequency, } from './stat/frequency.ts';
export {
  computeMaxLength,
  countBytes,
  countGraphemes,
  isBlankLine,
  splitGraphemes,
  splitLines,
  splitParagraphs,
  splitSentences,
  splitWords,
} from './stat/tokenize.ts';

//endregion Statistics

//region Page assembly

/**
 The rendered page and the tiles it reports.
 */
export {
  FREQUENCY_COLUMN_LABELS,
  renderPage,
} from './page.ts';
export {
  renderStatsSection,
  STAT_TILES,
} from './page-stats.ts';

//endregion Page assembly

//region Styles

/**
 The style sheet renderers, colour scheme included.
 */
export { renderStyles, } from './styles.ts';
export {
  renderDarkColors,
  renderRootColors,
} from './styles-colors.ts';

//endregion Styles

//region Favicon

/**
 The generated favicon, as SVG and as PNG bytes.
 */
export {
  FAVICON_SIZE,
  renderFaviconPngBase64,
  renderFaviconSvg,
} from './favicon.ts';

//endregion Favicon
