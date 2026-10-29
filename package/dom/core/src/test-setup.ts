/**
 Test setup helper: loads the harness HTML and injects the built bundle
 as an inline module script, then waits for the bridge to expose
 `globalThis.moduleDom`. Centralised here so every `*.e2e.test.ts`
 has the same loading guarantee without repeating boilerplate.
 
 Direct `<script type="module" src="...relative...">` loads from a
 `file://` page are blocked in Chromium for security; injecting the
 bundle inline via `page.addScriptTag` bypasses that restriction
 because the content is treated as same-origin to the host page.
 
 @module
 */
import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import { pathToFileURL, } from 'node:url';

import type { Page, } from '@playwright/test';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

declare global {
  /**
   Bundled module-dom helpers exposed on `globalThis` by the test harness.
   */
  // oxlint-disable-next-line typescript-eslint/consistent-type-imports -- typeof import() cannot use import type syntax
  var moduleDom: typeof import('./index.ts');
}

/**
 Absolute file URL to the test harness HTML (empty body, no scripts).
 */
export const HARNESS_URL: string = pathToFileURL(
  join(
    import.meta.dirname,
    '..',
    'test',
    'harness.html',
  ),
)
  .href;

/**
 Absolute path to the built browser bundle that the tests exercise.
 Built by `mise run //package/dom/core:build:js:browser`.
 */
const BUNDLE_PATH: string = join(
  import.meta.dirname,
  '..',
  'dist',
  'final',
  'neutral',
  'index.mjs',
);

/**
 Single-entry cache for the rewritten bundle source so each test does
 not re-read the dist file. `let` at module root is banned by
 `no-module-root-let`; a `Map` keyed on the bundle path is the
 project-recommended container shape.
 */
const bundleSourceCache = new Map<string, string>();

/**
 Tests whether `c` is `\s` whitespace (space, tab, newline, carriage
 return, form feed, vertical tab).
 
 @param c - single character
 
 @returns whether `c` is whitespace
 */
function isWhitespaceChar(c: string,): boolean {
  return (c === ' ')
    || (c === '\t')
    || (c === '\n')
    || (c === '\r')
    || (c === '\f')
    || (c === '\v');
}

/**
 Returns the largest index strictly less than `end` whose char is
 non-whitespace per {@link isWhitespaceChar}; `-1` when only whitespace
 precedes `end`.
 
 @param s - source string
 
 @param end - exclusive upper bound for the scan
 
 @returns rightmost non-whitespace index, or `-1` when none found
 */
function lastNonWhitespaceIndex({
  s,
  end,
}: Readonly<{
  s: string;
  end: number;
}>,): number {
  if (end <= 0)
    return -1;
  if (!isWhitespaceChar(s.charAt(end - 1,),))
    return end - 1;
  return lastNonWhitespaceIndex({
    s,
    end: end - 1,
  },);
}

/**
 Tests whether `c` qualifies as a `\w` word char (alphanumeric or `_`);
 used to verify `export` sits on a word boundary.
 
 @param c - single character
 
 @returns whether `c` is a word char
 */
function isWordChar(c: string,): boolean {
  return ((c >= 'a') && (c <= 'z'))
    || ((c >= 'A') && (c <= 'Z'))
    || ((c >= '0') && (c <= '9'))
    || (c === '_');
}

/**
 Sentinel returned by {@link parseTrailingExportClause} when the trailing
 `export { ... }` clause is missing or malformed. A unique `Symbol` keeps
 "no clause" distinct from any parsed result without a nullish escape.
 */
const NO_CLAUSE: unique symbol = Symbol('module-dom trailing export clause absent',);

/**
 Locates the trailing `export { ... }` clause of a rolldown bundle and
 extracts the named-export list.
 
 Mirrors the shape of `/export\s*\{\s*([^}]+)\s*\}\s*;?\s*$/`: walks
 back from end-of-string via {@link lastNonWhitespaceIndex}, skips
 trailing whitespace and an optional `;`, requires `}`, finds the
 matching `{`, then verifies `export` sits at a word boundary
 (checked with {@link isWordChar}) immediately before the `{`.
 
 @param source - bundle source text
 
 @returns named-export list and the byte offset of `export`, or
   {@link NO_CLAUSE} when the trailing clause is missing or malformed
 */
function parseTrailingExportClause(source: string,): {
  namedExports: string;
  clauseStart: number;
} | typeof NO_CLAUSE {
  /**
   Last non-whitespace position; `-1` means the source is empty/whitespace-only.
   */
  const lastIdx = lastNonWhitespaceIndex({
    s: source,
    end: source.length,
  },);
  if (lastIdx === (-1))
    return NO_CLAUSE;
  /**
   Position of the closing `}`; the semicolon is optional in the original regex.
   */
  const closeBrace = (source.charAt(lastIdx,)
    === ';')
    ? lastNonWhitespaceIndex({
      s: source,
      end: lastIdx,
    },)
    : lastIdx;
  if ((closeBrace === (-1)) || (source.charAt(closeBrace,)
    !== '}'))
    return NO_CLAUSE;
  /**
   Position of the matching `{`; the original regex requires `[^}]+` between, so the last `{` before `}` is correct.
   */
  const openBrace = source.lastIndexOf(
    '{',
    closeBrace - 1,
  );
  if (openBrace === (-1))
    return NO_CLAUSE;
  /**
   Position immediately before `{`, skipping intervening whitespace; the `export` keyword should end here.
   */
  const beforeOpen = lastNonWhitespaceIndex({
    s: source,
    end: openBrace,
  },);
  /**
   Literal keyword the clause must lead with.
   */
  const EXPORT = 'export';
  if (beforeOpen < (EXPORT.length
    - 1))
    return NO_CLAUSE;
  /**
   Inclusive start of the `export` keyword candidate; the byte before it must not be a word char.
   */
  const wordStart = (beforeOpen - EXPORT
    .length) + 1;
  if (
    source.slice(
      wordStart,
      beforeOpen + 1,
    )
      !== EXPORT
  ) {
    return NO_CLAUSE;
  }
  if ((wordStart > 0) && isWordChar(source.charAt(wordStart - 1,),))
    return NO_CLAUSE;
  return {
    namedExports: source
      .slice(
      openBrace + 1,
      closeBrace,
      )
      .trim(),
    clauseStart: wordStart,
  };
}

/**
 Reads the built bundle, locates the trailing `export ` ... ` ` clause via
 {@link parseTrailingExportClause} (throwing below on {@link NO_CLAUSE}),
 rewrites it into a `globalThis.moduleDom = ` ... ` ` assignment, and
 returns the resulting module source ready for inline injection.
 
 @returns Module-script source that, when injected, exposes all named
   exports as `globalThis.moduleDom`
 
 @throws Error when the bundle is missing its trailing
   `export ` ... ` ` (the rebuild may have failed or the format changed)
 
 @example
 ```ts
 const content = await bundleAsGlobalAssignment();
 await page.addScriptTag({ content, type: 'module', },);
 ```
 */
async function bundleAsGlobalAssignment(): Promise<string> {
  /**
   Previously-computed rewritten bundle; reused across calls in the same Playwright run.
   */
  const cached = bundleSourceCache.get(BUNDLE_PATH,);
  if (cached !== undefined)
    return cached;

  /**
   Raw bundle contents read from disk before the trailing `export { ... }` rewrite.
   */
  const source = await readFile(
    BUNDLE_PATH,
    'utf8',
  );
  /**
   Parsed trailing `export { ... }` clause; throws when the bundle shape is unexpected.
   */
  const clause = parseTrailingExportClause(source,);
  if (clause === NO_CLAUSE) {
    throw new Error(
      'test setup: could not locate trailing `export { ... }` in bundle; rebuild may have failed',
    );
  }

  /**
   Bundle text with the trailing `export { ... }` removed; ready for global rewrite.
   */
  const stripped = source.slice(
    0,
    clause.clauseStart,
  );
  /**
   Final bundle text where the named exports are assigned to `globalThis.moduleDom` for inline-script consumption.
   */
  const rewritten = `${stripped}globalThis.moduleDom = { ${clause.namedExports} };`;
  bundleSourceCache.set(
    BUNDLE_PATH,
    rewritten,
  );
  return rewritten;
}

/**
 Structural subset of Playwright `Page` covering only the methods
 {@link loadHarness} calls.
 A full `Page` carries unrelated mutable members such as `keyboard` and `mouse`;
 this view keeps the harness boundary limited to capabilities it actually invokes.
 A real `Page` satisfies the view structurally at every call site.
 */
type HarnessPage = Readonly<{
  /**
   Navigates to the harness URL.
   */
  goto: Page['goto'];

  /**
   Injects the bundled helpers as an inline module script.
   */
  addScriptTag: Page['addScriptTag'];

  /**
   Polls until `globalThis.moduleDom` is set.
   */
  waitForFunction: Page['waitForFunction'];
}>;

/**
 Loads the {@link HARNESS_URL} page, injects the bundled helpers built by
 {@link bundleAsGlobalAssignment} as an inline script, and waits until
 `globalThis.moduleDom` is set so subsequent `page.evaluate` calls can
 reference it directly.
 
 @param page - Playwright Page to set up
 
 @param query - Optional query string (with or without leading `?`)
   appended to the harness URL so `globalThis.location.search` reflects
   it when `onLoadSetCssFromUrlParams` runs
 
 @mutates page - `page.goto`, `page.addScriptTag`, and `page.waitForFunction` change browser session state
 
 @example
 ```ts
 await loadHarness({ page, query: '--brand=red', },);
 await page.evaluate(() => globalThis.moduleDom.onLoadSetCssFromUrlParams(),);
 ```
 */
export async function loadHarness(
  {
    page,
    query,
  }: ForeignBorrowed<Readonly<{
    page: HarnessPage;
    query?: string;
  }>>,
): Promise<void> {
  /**
   Harness URL with the optional query string normalised to a leading `?`.
   */
  const url = (query === undefined) || (query === '')
    ? HARNESS_URL
    : `${HARNESS_URL}${query.startsWith('?',) ? query : `?${query}`}`;
  await page.goto(url,);

  /**
   Bundle text where the named exports are rewritten as a `globalThis.moduleDom` assignment for inline injection.
   */
  const content = await bundleAsGlobalAssignment();
  await page.addScriptTag({
    content,
    type: 'module',
  },);

  await page.waitForFunction(function isReady() {
    return globalThis.moduleDom
      !== undefined;
  },);
}
