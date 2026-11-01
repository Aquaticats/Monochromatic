# inference-canary-viewer

Static dashboard for inference canary results.
Generates a single-page HTML viewer from enriched artifacts produced by [inference-canary](../inference-canary/).

## How it works

The build script reads enriched artifact directories from the sibling `inference-canary` package,
groups them by (model, timestamp) to reconstruct per-run entries,
and renders a self-contained HTML dashboard with an external CSS stylesheet.

The output is static HTML with a small client-side script for syntax highlighting.
View switching uses native `<details>` elements with CSS `:has()` to hide non-open siblings.
Run detail overlays use the Popover API (`popover="auto"` + `popovertarget`) for light-dismiss behavior.
Syntax highlighting uses the CSS Custom Highlight API via Lezer tokenization;
code blocks degrade gracefully to unstyled monospace text when JavaScript is disabled
or the Highlight API is unavailable.

### Views

The dashboard provides three switchable views:

- **Overview**: combined all-models scatter plot of overall scores over time,
  plus a summary table with latest score, last run date, run count,
  degradation threshold, and status flags per model
- **By model**: collapsible `<details>` per model,
  each containing an overall score scatter plot with threshold line
  and nested per-probe breakdowns
- **By probe**: collapsible `<details>` per probe with a cross-model scatter plot
  showing all models overlaid, plus nested per-model breakdowns

Scatter plots are CSS-positioned `<button>` elements inside a relative container.
Each point opens a Popover API overlay via `popovertarget`.
Pass-1 scores render as filled circles; pass-2 (fix) scores render as hollow circles at the same X position.
Every chart has a backing `<table>` (visually hidden, accessible to screen readers)
or a compact card grid when only two columns remain.

### Overlays

Run overlays show a probe grid with clickable cards linking to per-probe overlays.
Probe overlays include:

- Status badges (partial, error, non-stop finish reason)
- Collapsible timing and token usage metadata for initial and fix passes
- Source code with CSS Custom Highlight API syntax highlighting,
  or a side-by-side diff (via `git diff --no-index`) when a fix pass exists
- Collapsible reasoning traces, raw responses, fix prompts, and config snapshots rendered as markdown via micromark

### Data pipeline

1. Reads `meta.json`, `canary.ts`, and `response.txt` from each artifact directory under `src/canary-lint/`
2. Groups initial-pass and fix-pass artifacts by run key (`label::timestamp`)
3. Computes per-probe scores and overall score (mean of initial-pass probe scores)
4. Calculates per-model degradation thresholds (mean - 2 * stddev, floored at 0.3, minimum 3 samples)
5. Renders HTML via the `h()` builder from `module-es`
6. Builds CSS via `build-tool-css` and writes both files to `dist/final/`

## Usage

```bash
mise run build
```

### Development

Watch for source or canary-lint changes and rebuild automatically:

```bash
mise run dev:watch
```

Serve the built dashboard locally:

```bash
mise run dev:serve
```

Both combined:

```bash
mise run dev
```

## Architecture

```
src/
  build.ts                  Build entry point, orchestrates reading and rendering
  svg.d.ts                  SVG import type declarations
  data/
    read-artifacts.ts       Artifact reader, groups by (model, timestamp) into run entries
    viewer-types.ts         ViewerEntry, ProbeDetail, ArtifactData types
    threshold.ts            Degradation threshold: mean - 2*stddev, floored at 0.3
    model-colors.ts         Vendor color resolution from OpenRouter model IDs
    model-icons.ts          SVG vendor icon resolution from OpenRouter model IDs
    diff.ts                 Line-level diff via git diff --no-index
  html/
    page.ts                 HTML document shell (DOCTYPE, head, body)
    dashboard.ts            Three-section layout with <details>-based view switching
    view-overview.ts        All-models scatter plot and summary table
    view-model.ts           Per-model <details> with overall + per-probe scatter plots
    view-probe.ts           Per-probe <details> with cross-model + per-model scatter plots
    detail-overlay.ts       Popover API overlays with source, diff, and enriched metadata
    overlay-meta.ts         Badges, timing/usage metadata, collapsible reasoning/config
  chart/
    scatter.ts              CSS-positioned scatter plot with popovertarget buttons
    axis.ts                 Y-axis (0.0-1.0) and X-axis (timestamps) rendering
    legend.ts               Color legend and shape legend for scatter plots
    threshold-line.ts       Horizontal threshold line overlay
    data-table.ts           Accessible backing table and compact card grid
  client/
    index.ts                Client-side entry point (syntax highlighting via CSS Custom Highlight API)
    tags.ts                 Lezer tag-to-highlight-group mapping
  css/
    index.css               CSS entry point
    base.css                Reset and base styles
    mixin.css               Composable CSS mixins
    views.css               View section and switcher layout
    chart.css               Scatter plot and axis styling
    overlay.css             Popover overlay styling
    diff.css                Side-by-side diff view styling
    glow.css                ::highlight() syntax highlighting rules
```
