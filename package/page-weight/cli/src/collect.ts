/**
 Per-page asset collection pipeline.
 
 Given an HTML file inside a dist root, computes the total wire transfer
 size of the HTML itself plus every asset a browser would fetch to render it.
 */
import { readFile, } from 'node:fs/promises';
import { extname, } from 'node:path';

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { extractCssUrls, } from './css.ts';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { extractHtmlRefs, } from './html.ts';
import {
  resolveReference,
  UNRESOLVABLE_REFERENCE,
} from './resolve.ts';
import {
  wireSize,
  WIRE_SIZE_UNAVAILABLE,
} from './size.ts';

/**
 Result of weighing a single HTML page.
 */
export type PageWeight = {
  /**
   Path of the HTML file relative to the dist root.
   */
  readonly page: string;
  /**
   Sum of wire sizes of the HTML and every asset it references.
   */
  readonly totalBytes: number;
  /**
   Number of unique assets contributing to the total (including the HTML).
   */
  readonly resourceCount: number;
  /**
   References the walker saw but could not resolve to a file under root.
   */
  readonly missing: readonly string[];
};

/**
 Reads a UTF-8 text file. Thin wrapper kept so callers read clearly.
 
 @param absolutePath - absolute filesystem path
 
 @returns file contents as a string
 */
function readText(absolutePath: string,): Promise<string> {
  return readFile(
    absolutePath,
    'utf8',
  );
}

/**
 Sentinel returned by {@link readCssOrAbsent} when a CSS file cannot be read
 (missing, permissions, I/O). A `unique symbol`; callers narrow with
 `=== CSS_UNREADABLE` to skip dead links without aborting the walk.
 */
const CSS_UNREADABLE: unique symbol = Symbol('page-weight CSS file cannot be read',);

/**
 Reads a CSS file and returns the raw source, or {@link CSS_UNREADABLE} if it
 cannot be read.
 
 Skipping on read failure keeps the pipeline robust against broken links
 without masking them; the caller still receives the path via `missing`.
 
 @param absolutePath - absolute CSS path
 
 @returns CSS source, or {@link CSS_UNREADABLE} on read failure
 */
async function readCssOrAbsent(absolutePath: string,): Promise<string | typeof CSS_UNREADABLE> {
  try {
    return await readText(absolutePath,);
  }
  catch (error) {
    console.warn(
      `[page-weight] CSS read failed for ${absolutePath}: ${caughtValueText(error,)}`,
    );
    return CSS_UNREADABLE;
  }
}

/**
 Walks a CSS file's `url()` references into resolved absolute paths,
 recursing through `@import` chains. Skips files that read as
 {@link CSS_UNREADABLE} instead of aborting the walk.
 
 @param startPath - absolute path of the initial CSS file
 
 @param root - absolute dist root
 
 @param seen - read-only view of absolute paths already counted by the
   caller, so an asset reachable from both the HTML and the CSS graph is
   reported once
 
 @returns assets discovered through the CSS graph plus references that
   resolved to {@link UNRESOLVABLE_REFERENCE}, for the caller to merge
   into its own accumulators
 */
async function walkCss(
  {
    startPath,
    root,
    seen,
  }: {
    readonly startPath: string;
    readonly root: string;
    readonly seen: ReadonlySet<string>;
  },
): Promise<{
  readonly assets: readonly string[];
  readonly missing: readonly string[];
}> {
  /**
   Accumulator of unique asset paths discovered through the CSS graph.
   */
  const collected: string[] = [];
  /**
   Accumulator of references that could not be resolved to a file under root.
   */
  const missing: string[] = [];
  /**
   Local dedup view seeded from the caller's counted set; grows as new assets are discovered.
   */
  const counted = new Set<string>(seen,);
  /**
   BFS frontier so chained `@import`s are followed before the function returns.
   */
  const queue: string[] = [startPath,];
  /**
   Cycle guard so a CSS file reached twice is not re-parsed.
   */
  const visited = new Set<string>();

  while (queue.length
    > 0) {
    /**
     Next stylesheet to process; the while-condition guarantees a value.
     */
    const cssPath = nonNullishOrThrow(queue.shift(),);
    if (visited.has(cssPath,))
      continue;
    visited.add(cssPath,);

    /**
     CSS text, or `CSS_UNREADABLE` when the file cannot be read so dead links don't abort the walk.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- BFS over a queue that grows as each iteration parses imports; each step depends on the previous shift and the shared `visited` set, so parallelisation would race on dedup state.
    const source = await readCssOrAbsent(cssPath,);
    if (source === CSS_UNREADABLE)
      continue;

    for (const ref of extractCssUrls(source,)) {
      /**
       Absolute asset path, or `UNRESOLVABLE_REFERENCE` when the reference escapes the dist root.
       */
      const resolved = resolveReference({
        root,
        fromFile: cssPath,
        ref,
      },);
      if (resolved === UNRESOLVABLE_REFERENCE) {
        missing.push(ref,);
        continue;
      }
      if ((extname(resolved,)
        .toLowerCase()
        === '.css') && (!visited.has(resolved,)))
        queue.push(resolved,);
      if (!counted.has(resolved,)) {
        counted.add(resolved,);
        collected.push(resolved,);
      }
    }
  }

  return {
    assets: collected,
    missing,
  };
}

/**
 Computes the wire transfer size of one HTML page.
 
 Algorithm:
 
 1. Add the HTML's own wire size.
 2. Walk HTML for referenced assets (link, script, img, picture, ...) via
    {@link extractHtmlRefs}.
 3. For each CSS reference, walk its `url()` graph via {@link walkCss} to
    pick up fonts, background images, and chained `@import`s.
 4. For each `<style>` block, apply the same CSS `url()` scan using the
    HTML's own path as the resolution base.
 5. Sum the unique wire sizes via {@link wireSize}, recording
    {@link WIRE_SIZE_UNAVAILABLE} assets into `missing` instead.
 
 @param htmlPath - absolute path to the HTML file
 
 @param root - absolute dist root
 
 @returns a `PageWeight` record
 
 @example
 ```ts
 const weight = await weighPage({
   htmlPath: '/srv/dist/index.html',
   root: '/srv/dist',
 });
 ```
 */
export async function weighPage(
  {
    htmlPath,
    root,
  }: {
    readonly htmlPath: string;
    readonly root: string;
  },
): Promise<PageWeight> {
  /**
   Accumulator for unresolvable references, surfaced on the returned record.
   */
  const missing: string[] = [];
  /**
   Dedup set seeded with the HTML so it is not re-added via a self-reference.
   */
  const seen = new Set<string>([htmlPath,],);
  /**
   Ordered list of unique asset paths, beginning with the HTML itself.
   */
  const assets: string[] = [htmlPath,];

  /**
   Raw HTML scanned for asset references.
   */
  const htmlSource = await readText(htmlPath,);
  /**
   Destructured pair so direct URLs and inline `<style>` blocks can be walked separately.
   */
  const {
    urls,
    inlineStyles,
  } = extractHtmlRefs(htmlSource,);

  for (const ref of urls) {
    /**
     Absolute path of the referenced asset, or `UNRESOLVABLE_REFERENCE` when it escapes the dist root.
     */
    const resolved = resolveReference({
      root,
      fromFile: htmlPath,
      ref,
    },);
    if (resolved === UNRESOLVABLE_REFERENCE) {
      missing.push(ref,);
      continue;
    }
    if (!seen.has(resolved,)) {
      seen.add(resolved,);
      assets.push(resolved,);
    }
    if (extname(resolved,)
      .toLowerCase()
      === '.css') {
      /**
       Assets reachable through the CSS `@import` / `url()` graph rooted at this stylesheet.
       */
      // oxlint-disable-next-line eslint/no-await-in-loop -- each walkCss reads the running `seen` set to avoid double-counting assets already claimed by earlier stylesheets, so ordering is significant and parallel calls would race on that shared dedup state.
      const nested = await walkCss({
        startPath: resolved,
        root,
        seen,
      },);
      for (const asset of nested.assets) {
        seen.add(asset,);
        assets.push(asset,);
      }
      for (const nestedRef of nested.missing)
        missing.push(nestedRef,);
    }
  }

  for (const inline of inlineStyles) {
    for (const ref of extractCssUrls(inline,)) {
      /**
       Absolute asset path for an inline `<style>` reference, or `UNRESOLVABLE_REFERENCE` when it escapes the dist root.
       */
      const resolved = resolveReference({
        root,
        fromFile: htmlPath,
        ref,
      },);
      if (resolved === UNRESOLVABLE_REFERENCE) {
        missing.push(ref,);
        continue;
      }
      if (!seen.has(resolved,)) {
        seen.add(resolved,);
        assets.push(resolved,);
      }
    }
  }

  /**
   Wire sizes per asset, computed in parallel; `WIRE_SIZE_UNAVAILABLE` slots are surfaced via `missing` below.
   */
  const sizes = await Promise.all(
    assets.map(function measure(asset: string,): Promise<number | typeof WIRE_SIZE_UNAVAILABLE> {
      return wireSize(asset,);
    },),
  );
  /**
   Sum of every successfully measured asset, with unmeasurable assets recorded into `missing`.
   */
  const totalBytes = sizes.reduce(
    function sumNonNull(
      acc: number,
      size: number | typeof WIRE_SIZE_UNAVAILABLE,
      index: number,
    ): number {
      if (size === WIRE_SIZE_UNAVAILABLE) {
        missing.push(nonNullishOrThrow(assets[index],),);
        return acc;
      }
      return acc + size;
    },
    0,
  );

  /**
   Page path stripped of the dist root prefix so the report stays readable.
   */
  const relativePage = htmlPath.startsWith(root,)
    ? htmlPath.slice(root.length
      + 1,)
    : htmlPath;

  return {
    page: relativePage,
    totalBytes,
    resourceCount: assets.length,
    missing,
  };
}
