/**
 Server-side library surface of the SSG.

 The site build itself runs from source (`node src/build.ts`), and the browser
 bundle stays in `rolldown.client.config.ts`. This entry exists so unit tests
 load the render, content, and style code through the rolldown bundle under
 `dist/final/node` rather than through sibling source: a test that reaches
 source proves nothing about what the package builds, which is what the
 `test-import/require-eventual-artifact` rule enforces.

 Every export is internal. The package declares no `exports` map and nothing
 outside the package imports this entry, so the entry tracks the tests rather
 than a consumer contract.

 @module
 */

/**
 Compression helpers: worker distribution, the worker-count heuristic, tally
 arithmetic, and per-bucket compression.

 @internal
 */
export {
  addTallies,
  compressBucket,
  distribute,
  EMPTY_TALLY,
  resolveWorkerCount,
  type Tally,
} from './build/compress-lib.ts';

/**
 Fingerprinted-filename primitives.

 @internal
 */
export {
  insertHash,
  sha256Buffer,
} from './build/fingerprint-naming.ts';

/**
 Leaf-asset exclusion predicates.

 @internal
 */
export {
  containsHiddenDirectory,
  isLeafExcluded,
} from './build/postprocess-excludes.ts';

/**
 Linear scanners that read icon call sites out of project source text.

 @internal
 */
export { stripBlockComments, } from './build/project-source-scanning.ts';

/**
 Component styles and markup the suites assert on.

 @internal
 */
export { css as calloutAlertCss, } from './component/callout-alert.ts';

/**
 Alert callout component, for its localized indicator label.

 @internal
 */
export {
  CalloutAlert,
  type CalloutAlertProps,
} from './component/callout-alert.ts';

/**
 MDX component map, for asserting which labels a locale reaches.

 @internal
 */
export {
  createMdxComponents,
  type MdxComponents,
} from './component/index.ts';

/**
 Language switcher styles.

 @internal
 */
export { css as langSwitcherStyles, } from './component/lang-switcher.ts';

/**
 Post card markup, or the list item that holds it.

 @internal
 */
export { html as postCardHtml, } from './component/post-card.ts';

/**
 Post grid markup.

 @internal
 */
export { html as postListHtml, } from './component/post-list.ts';

/**
 Multi-correct quiz question component and its styles.

 @internal
 */
export {
  css as questionCheckboxStyles,
  QuestionCheckbox,
  type QuestionOption as QuestionCheckboxOption,
} from './component/question-checkbox.ts';

/**
 Single-correct quiz question component and its styles.

 @internal
 */
export {
  css as questionRadioStyles,
  QuestionRadio,
  type QuestionOption as QuestionRadioOption,
} from './component/question-radio.ts';

/**
 Footer newsticker markup and styles.

 @internal
 */
export {
  css as siteFooterStyles,
  html as siteFooterHtml,
} from './component/site-footer.ts';

/**
 Header markup and styles.

 @internal
 */
export {
  css as siteHeaderStyles,
  html as siteHeaderHtml,
} from './component/site-header.ts';

/**
 Search widget styles.

 @internal
 */
export { css as siteSearchStyles, } from './component/site-search.ts';

/**
 Locale registry and the label vocabulary the pages render.

 @internal
 */
export {
  i18n,
  isLocale,
  locales,
  type Locale,
} from './i18n/index.ts';

/**
 Locale autonyms shown in the language picker and switcher.

 @internal
 */
export { LANG_NAMES, } from './i18n/lang-names.ts';

/**
 Content loading, frontmatter parsing, and post grouping.

 @internal
 */
export {
  attachDates,
  loadContent,
  type LoadedPost,
  type Post,
  type PostFrontmatter,
  type ResolvedDates,
} from './lib/content.ts';

/**
 Frontmatter date divergence diagnostics.

 @internal
 */
export { warnOnAuthoredDateDivergence, } from './lib/date-divergence.ts';

/**
 Reads the UTC offset an ISO-8601 timestamp was written in.

 @internal
 */
export { authorOffsetMinutes, } from './lib/git-dates.ts';

/**
 YAML frontmatter fence scanner.

 @internal
 */
export {
  parseFrontmatter,
  type ParsedFrontmatter,
} from './lib/frontmatter.ts';

/**
 Markup wrapper the JSX runtime returns.

 @internal
 */
export type { SafeHtml, } from './lib/jsx-to-html.ts';

/**
 Build cache lookup and entry construction.

 @internal
 */
export {
  CACHE_MISS,
  createCacheEntry,
  getCachedEntry,
  type BuildManifest,
  type CacheEntry,
} from './lib/cache.ts';

/**
 MDX rendering, for asserting the localized chrome a post body produces.

 @internal
 */
export { renderMdx, } from './lib/markdown.ts';

/**
 Cache-invalidation glob naming the pipeline sources.

 @internal
 */
export { PIPELINE_GLOB, } from './lib/pipeline-glob.ts';

/**
 Per-language RSS feed generation.

 @internal
 */
export { generateLanguageRss, } from './lib/rss.ts';

/**
 Rehype plugin that pre-computes Lezer highlight ranges.

 @internal
 */
export { default as rehypeHighlight, } from './lib/rehype-highlight.ts';

/**
 Logger subset the diagnostics accept.

 @internal
 */
export type { Logger, } from './lib/types.ts';

/**
 Root index page: the language picker.

 @internal
 */
export { indexPage, } from './page/index.ts';

/**
 Full post page and its missing-translation fallback.

 @internal
 */
export { postPage, } from './page/post.ts';

/**
 Shared CSS-side constants.

 @internal
 */
export { FONT_WEIGHT_EMPHASIS, } from './style/constants.ts';

/**
 Colour tokens and the brand hexes the head metadata interpolates.

 @internal
 */
export {
  PRIMARY_DARK,
  PRIMARY_LIGHT,
  tokenStyles,
} from './style/tokens.ts';

/**
 `<head>` template the pages compose.

 @internal
 */
export {
  headFragment,
  type ArticleDates,
} from './template/head.ts';
