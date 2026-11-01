/**
 Composes the final HTML audit report.
 
 Bundles the browser-side controller via rolldown (IIFE,
 minified), inlines the probe array as `globalThis.__PROBES__`, embeds
 the control-panel HTML from {@link renderControls}, and wraps
 everything in an HTML document with the package's CSS in a
 `<style>` block. The output is a single self-contained HTML file:
 no external `<link>` or `<script src>` references; open from disk
 and the page renders.
 
 @example
 ```ts
 import { renderHtml } from './render-html.ts';
 import { writeFile } from 'node:fs/promises';
 const html = await renderHtml({ probes });
 await writeFile('deps-cube-2026-05-12.html', html, 'utf8');
 ```
 */

import { readFile, } from 'node:fs/promises';
import { resolve as resolvePath, } from 'node:path';

import {
  type OutputChunk,
  rolldown,
} from 'rolldown';

import {
  findRootCached,
  packageNamed,
} from '@monochromatic-dev/fs-path-core/ts';

import type { PackageProbe, } from './probe.ts';
import { renderControls, } from './render-controls.ts';
import { defaultState, } from './script/state.ts';

//region Constants

/**
 Absolute path of this package's root directory, resolved at module load.
 
 Walks up from `import.meta.dirname` to the `package.json` whose
 `name` matches `@monochromatic-dev/dependency-catalog-cli-cube`. Result is
 memoised by the helper per marker and start directory; `cli.ts` resolves
 the same root from its own directory.
 */
const PACKAGE_ROOT = await findRootCached({
  cwd: import.meta.dirname,
  marker: packageNamed('@monochromatic-dev/dependency-catalog-cli-cube',),
},);

/**
 Absolute path to the browser-side controller source entry point.
 
 Anchored on {@link PACKAGE_ROOT} so it points at
 `<pkg>/src/script/controller.ts` regardless of where this module
 is evaluated from: an `import.meta.url`-relative path would resolve
 to `<pkg>/dist/final/node/scripts/controller.ts` after tsdown
 bundles this file into a single `cli.mjs`, and that target doesn't
 exist. rolldown needs the original TypeScript source path; the
 source tree is shipped alongside the built artifacts (see
 `package.json#files`) so the path resolves in both modes.
 */
const CONTROLLER_ENTRY_PATH = resolvePath(
  PACKAGE_ROOT,
  'src',
  'scripts',
  'controller.ts',
);

/**
 Inlined contents of `src/styles.css`, read once at module load.
 
 Originally imported via `with { type: 'text' }`, but tsdown's
 `css-guard` plugin throws unconditionally on `.css` files (no
 configuration opt-out) when bundling for Node. Replacing the
 import-attribute with a runtime `readFile` lets tsdown bundle
 `render-html.ts` without invoking the CSS plugin, while still
 inlining the stylesheet as a string at first use. The styles file
 is shipped via `package.json#files: ["src"]` so the path resolves
 after a tsdown build.
 */
const stylesCss = await readFile(
  resolvePath(
    PACKAGE_ROOT,
    'src',
    'styles.css',
  ),
  'utf8',
);

/**
 Document `<title>` for the generated HTML.
 */
const PAGE_TITLE = 'deps-cube; catalog dependency audit';

//endregion Constants

//region Helpers

/**
 Bundles the browser-side controller via rolldown into a single
 self-contained IIFE string.
 
 @returns Minified IIFE JS, ready to splice into a `<script>` block.
 
 @throws When rolldown emits no JS chunk for the controller entry point.
   rolldown surfaces compile/resolve failures by rejecting on its own, so
   those propagate without extra wrapping.
 */
async function bundleController(): Promise<string> {
  /**
   In-memory rolldown handle; `await using` releases its native resources even when `generate` rejects.
   */
  await using bundle = await rolldown({
    input: CONTROLLER_ENTRY_PATH,
    platform: 'browser',
  },);
  /**
   Generated bundle; `output` holds at least one chunk for the single entry point.
   */
  const { output, } = await bundle.generate({
    format: 'iife',
    minify: true,
  },);
  /**
   First emitted JS chunk; assets carry no `code` field, so the chunk is selected by its discriminant.
   */
  const chunk = output.find(function isChunk(part,): part is OutputChunk {
    return part.type === 'chunk';
  },);
  if (chunk === undefined)
    throw new Error('rolldown emitted no chunk for controller entry point',);
  return chunk.code;
}

/**
 Escapes a string for safe inlining inside an HTML `<script>` block.
 
 The two dangerous sequences are `</script` (closes the script tag
 prematurely so following content is parsed as HTML) and `<!--`
 (an HTML comment opener that browsers tolerate inside scripts but
 parse specially in legacy modes). Both are neutralised by
 inserting a backslash; the JS engine still parses the result as
 the original token.
 
 @param js - Source string destined for a `<script>` block.
 
 @returns Same string with unsafe sequences neutralised.
 */
function escapeForScriptTag(js: string,): string {
  return js
    .replaceAll(
    '</script',
    String.raw`<\/script`,
    )
    .replaceAll(
    '<!--',
    String.raw`<\!--`,
    );
}

//endregion Helpers

//region Public API

/**
 Serializes probe data embedded in report HTML.
 
 @param probes - Probe records that may expose serialization hooks.
 
 @returns compact JSON text.
 
 @mutates probes - `JSON.stringify` may invoke `toJSON`, getters, or proxy traps.
 
 @example
 ```ts
 serializeProbes([]);
 ```
 */
function serializeProbes(probes: object,): string {
  return JSON.stringify(probes,);
}

/**
 Composes the full HTML report.
 
 @param probes - Resolved package probes for every catalog entry.
 
 @param bundle - Controller-bundler seam; defaults to the rolldown-backed
   bundler. Tests override it to skip a multi-second real build.
 
 @returns Self-contained HTML document as a single string.
 
 @mutates probes - `JSON.stringify` may invoke hooks on probe records.
 
 @example
 ```ts
 const html = await renderHtml({ probes });
 await writeFile('out.html', html, 'utf8');
 ```
 */
export async function renderHtml(
  {
    probes,
    bundle = bundleController,
  }: {
    probes: readonly PackageProbe[];
    readonly bundle?: () => Promise<string>;
  },
): Promise<string> {
  /**
   Initial controller state used to seed the control-panel markup; the browser-side controller re-derives its state from this.
   */
  const initialState = defaultState({
    probes,
  },);
  /**
   Pre-rendered control panel HTML, spliced into `<main>` below so the page is usable before JS executes.
   */
  const controlsHtml = renderControls({
    probes,
    state: initialState,
  },);
  /**
   Minified IIFE JS for the browser-side controller; built once per `renderHtml` call.
   */
  const bundleJs = await bundle();
  /**
   Probe array serialised for inlining as `globalThis.__PROBES__`; escaped below to neutralise `</script>` sequences.
   */
  const probesJson = serializeProbes(probes,);
  return `<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>${PAGE_TITLE}</title>
  <style>${stylesCss}</style>
</head>
<body>
  <main id="canvas-host"><canvas id="deck-canvas"></canvas></main>
  ${controlsHtml}
  <script>globalThis.__PROBES__ = ${escapeForScriptTag(probesJson,)};</script>
  <script>${escapeForScriptTag(bundleJs,)}</script>
</body>
</html>
`;
}

//endregion Public API
