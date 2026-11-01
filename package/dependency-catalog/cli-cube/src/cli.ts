#!/usr/bin/env node

/**
 `deps-cube` CLI entrypoint.
 
 Reads the workspace catalog, probes every entry, renders the HTML
 report, writes it to
 `<package>/dist/deps-cube-<YYYY-MM-DDTHH-MM-SSZ>.html`, and prints
 exactly one line to stdout: `Saved to <abs-path>`. Probe progress
 goes to stderr via `console.error` in {@link probeAll}.
 
 Output is anchored to this package's `dist/` via {@link PACKAGE_ROOT},
 resolved at module load by {@link findRootCached} from
 `@monochromatic-dev/fs-path-core`. The helper walks up from
 `import.meta.dirname` until the package's own `package.json` is
 found. This works identically in source mode (`node src/cli.ts`) and
 in built mode (tsdown emits `<pkg>/dist/final/node/cli.mjs` and
 `package.json#bin` points there). A hardcoded relative offset would
 land on different absolute paths in the two modes.
 
 No flags: the CLI is intentionally zero-config. Filenames carry
 ISO 8601 UTC down to seconds (with `:` rewritten as `-` for
 filesystem safety), so each invocation produces a distinct artifact
 and re-runs do not overwrite earlier reports.
 
 @example
 ```bash
 deps-cube
 # → Saved to /abs/path/to/deps-cube/dist/deps-cube-2026-05-12T21-23-45Z.html
 ```
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { resolve as resolvePath, } from 'node:path';

import {
  findRootCached,
  packageNamed,
} from '@monochromatic-dev/fs-path-core/ts';

import { createCache, } from './cache.ts';
import { readCatalog, } from './catalog.ts';
import { probeAll, } from './probe.ts';
import { renderHtml, } from './render-html.ts';

//region Package root

/**
 Absolute path of this package's root directory, resolved at module load.
 
 Walks up from `import.meta.dirname` to the `package.json` whose
 `name` matches `@monochromatic-dev/dependency-catalog-cli-cube`. Result is
 memoised by the helper per marker and start directory; `render-html.ts`
 resolves the same root from its own directory.
 */
const PACKAGE_ROOT = await findRootCached({
  cwd: import.meta.dirname,
  marker: packageNamed('@monochromatic-dev/dependency-catalog-cli-cube',),
},);

//endregion Package root

//region Helpers

/**
 Length of the `YYYY-MM-DDTHH:MM:SS` prefix in an ISO 8601 UTC timestamp; drops `.sssZ`.
 */
const ISO_SECONDS_PREFIX_LENGTH = 19;

/**
 Builds the output filename for the current run.
 
 Uses ISO 8601 UTC down to seconds: `YYYY-MM-DDTHH-MM-SSZ`. The `:`
 separators required by the canonical ISO form are replaced with
 `-` so the name is filesystem-safe (Windows refuses `:` in
 filenames, and shell history is friendlier without quoting). UTC
 keeps re-runs across timezones stably ordered, and second
 granularity means each invocation gets its own artifact rather
 than overwriting earlier runs.
 
 @returns Filename like `deps-cube-2026-05-12T21-23-45Z.html`.
 
 @example
 ```ts
 currentRunOutputFilename();
 // → 'deps-cube-2026-05-12T21-23-45Z.html'
 ```
 */
function currentRunOutputFilename(): string {
  /**
   ISO 8601 UTC timestamp (`YYYY-MM-DDTHH:MM:SS.sssZ`) captured at filename-construction time.
   */
  const iso = new Date().toISOString();
  /**
   Drop the millisecond suffix and trailing `Z` ({@link iso} = `YYYY-MM-DDTHH:MM:SS.sssZ`).
   */
  const seconds = iso.slice(
    0,
    ISO_SECONDS_PREFIX_LENGTH,
  );
  /**
   Filesystem-safe form: every `:` becomes `-`.
   */
  const stem = seconds.replaceAll(
    ':',
    '-',
  );
  return `deps-cube-${stem}Z.html`;
}

//endregion Helpers

//region Entry

/**
 Catalog entries parsed from `pnpm-workspace.yaml` upward from cwd.
 */
const entries = await readCatalog();

/**
 Shared per-package JSON file cache rooted at `~/.cache/monochromatic/deps-cube`.
 */
const cache = createCache();

/**
 Resolved attribute vector for every catalog entry.
 */
const probes = await probeAll({
  entries,
  cache,
},);

/**
 Composed self-contained HTML report.
 */
const html = await renderHtml({
  probes,
},);

/**
 Absolute path of this package's `dist/` directory, anchored on {@link PACKAGE_ROOT}.
 */
const distDir = resolvePath(
  PACKAGE_ROOT,
  'dist',
);

await mkdir(
  distDir,
  { recursive: true, },
);

/**
 Absolute path of today's output file under `<package>/dist/`.
 */
const absPath = resolvePath(
  distDir,
  currentRunOutputFilename(),
);

await writeFile(
  absPath,
  html,
  'utf8',
);
console.log(`Saved to ${absPath}`,);

//endregion Entry
