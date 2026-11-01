/**
 * Static site build script for the inference canary viewer.
 *
 * Reads enriched artifacts from the sibling inference-canary package,
 * generates a single-page HTML dashboard, and writes it to `dist/final/`.
 *
 * Exceeds 100 lines: top-level build orchestration script with sequential
 * pipeline steps that must remain in a single execution scope.
 */
import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { buildCss, } from '@monochromatic-dev/css-cli/ts';

import { renderSvgSprite, } from './data/model-icons.ts';
import { readArtifacts, } from './data/read-artifacts.ts';
import { computeThreshold, } from './data/threshold.ts';
import { hasMultipleProbes, } from './data/viewer-types.ts';
import { renderDashboard, } from './html/dashboard.ts';
import { renderAllOverlays, } from './html/detail-overlay.ts';
import { renderPage, } from './html/page.ts';
import { renderByModel, } from './html/view-model.ts';
import {
  type ModelSummary,
  renderOverview,
} from './html/view-overview.ts';
import { renderByProbe, } from './html/view-probe.ts';

export {};

/**
 * Absolute path to this package's root directory
 */
const PACKAGE_DIR: string = new URL(
  '..',
  import.meta.url,
)
  .pathname;
/**
 * Output directory for the generated site
 */
const DIST_DIR = join(
  PACKAGE_DIR,
  'dist',
  'final',
);
/**
 * Directory containing CSS source files
 */
const CSS_DIR = join(
  PACKAGE_DIR,
  'src',
  'css',
);

console.error('[viewer] reading artifacts...',);
/**
 * All run entries and per-probe detail data loaded from artifact directories.
 */
const {
  entries,
  probeDetails,
} = await readArtifacts();

console.error(
  `[viewer] ${String(entries.length,)} runs, ${String(probeDetails.size,)} probe details`,
);

//region Build model labels and thresholds from the canonical model registry

/**
 * Unique model labels across all entries
 */
const uniqueLabels = [...new Set(entries.map(function getLabel(entry,) {
  return entry.label;
},),),];

/**
 * Map from model label to computed degradation threshold
 */
const thresholds = new Map<string, number>(
  uniqueLabels.map(function buildThreshold(label,) {
    return [
      label,
      computeThreshold({
        label,
        entries,
      },)
        .threshold,
    ];
  },),
);

//endregion Build model labels and thresholds

//region Build model summaries for the overview

/**
 * Summaries for the overview table, one per model
 */
const summaries: ModelSummary[] = uniqueLabels.flatMap(function buildSummary(label,) {
  /**
   * Entries that share the model label being summarised this iteration.
   */
  const modelEntries = entries.filter(function matchLabel(entry,) {
    return entry.label
      === label;
  },);
  /**
   * Latest multi-probe run for meaningful overall score; fall back to latest run
   */
  /* oxlint-disable-next-line unicorn/no-array-callback-reference -- hasMultipleProbes is a type-compatible predicate */
  const latestMultiProbe = modelEntries.filter(hasMultipleProbes,)
    .at(-1,);
  /**
   * Run that drives the summary row; prefers a multi-probe run for representative score.
   */
  const latest = latestMultiProbe ?? modelEntries
    .at(-1,);
  if (latest === undefined)
    return [];

  /**
   * Degradation threshold for this model; `0` when no threshold is registered.
   */
  const threshold = thresholds.get(label,)
    ?? 0;
  return [{
    model: latest.model,
    label,
    latestScore: latest.overallScore,
    latestTimestamp: latest.timestamp,
    runCount: modelEntries.length,
    failed: latest.failed,
    threshold,
    degraded: (!latest.failed) && (latest.overallScore
      < threshold),
  },];
},);

//endregion Build model summaries

//region Render all HTML sections

console.error('[viewer] rendering HTML...',);

/**
 * Overview table HTML for the dashboard.
 */
const overviewHtml = renderOverview({
  summaries,
  entries,
},);
/**
 * By-model charts HTML for the dashboard.
 */
const byModelHtml = renderByModel({
  entries,
  thresholds,
},);
/**
 * By-probe charts HTML for the dashboard.
 */
const byProbeHtml = renderByProbe({ entries, },);
/**
 * Detail overlay popovers HTML for all entries.
 */
const overlaysHtml = await renderAllOverlays({
  entries,
  probeDetails,
},);

/**
 * Assembled dashboard HTML combining all sections.
 */
const dashboardHtml = renderDashboard({
  overviewHtml,
  byModelHtml,
  byProbeHtml,
  overlaysHtml,
},);
/**
 * Inline SVG icon sprite sheet.
 */
const spriteHtml = renderSvgSprite();
/**
 * Complete page HTML ready to write to disk.
 */
const pageHtml = renderPage({
  body: spriteHtml + dashboardHtml,
  title: 'Inference canary dashboard',
},);

//endregion Render all HTML sections

//region Build CSS and write output

await mkdir(
  DIST_DIR,
  { recursive: true, },
);

/**
 * Destructured CSS build result; HTML write result is discarded.
 */
const [, cssResult,] = await Promise.all([
  writeFile(
    join(
      DIST_DIR,
      'index.html',
    ),
    pageHtml,
    'utf8',
  ),
  buildCss({
    input: join(
      CSS_DIR,
      'index.css',
    ),
    output: join(
      DIST_DIR,
      'style.css',
    ),
  },),
],);

console.error(`[viewer] wrote ${
  join(
    DIST_DIR,
    'index.html',
  )
}`,);
console.error(
  `[viewer] wrote ${
    join(
      DIST_DIR,
      'style.css',
    )
  } (${String(cssResult.length,)} bytes)`,
);

//endregion Build CSS and write output
