/**
 Surgical catalog-range rewriter for catalog-tighten.
 
 Writes tightened ranges back into `pnpm-workspace.yaml` by string replacement
 on the raw text, preserving the file's formatting, comments, ordering, and
 quote style. A yaml parse-then-stringify round-trip would lose all of that,
 so the writer stays surgical even though the reader uses the yaml library.
 
 The prior writer matched only double-quoted entries (`"name": "range"`) and
 silently rewrote nothing on the single-quoted file (#258); this matches the
 entry by its key, regardless of quote style, and replaces only the value.
 */

//region Catalog rewrite

/**
 One tightened catalog entry: the original value text and its replacement.
 */
export type TightenedRange = {
  /**
   Catalog key (package name) whose line carries the value to rewrite.
   */
  readonly name: string;
  /**
   Original catalog value as written, e.g. `>=1.2.0` or `npm:@jsr/x@>=1.2.0`.
   */
  readonly oldRange: string;
  /**
   Replacement value, e.g. `>=1.3.0`.
   */
  readonly newRange: string;
};

/**
 Sentinel returned by {@link lineKey} for a line with no `key:` shape. A
 `unique symbol`; callers narrow with `=== NO_KEY`.
 */
const NO_KEY: unique symbol = Symbol('catalog-tighten/line carries no catalog key',);

/**
 Extracts the catalog key from one entry line, stripping a matching single or
 double quote pair. Returns {@link NO_KEY} when the line has no `key:` shape
 (blank lines, comments, the `catalog:` header).
 
 @param line - one raw line from the workspace file
 
 @returns unquoted key, or {@link NO_KEY}
 
 @example
 ```ts
 lineKey("  'oxlint': '>=1.71.0'") // "oxlint"
 lineKey("  # comment") // NO_KEY
 ```
 */
function lineKey(line: string,): string | typeof NO_KEY {
  /**
   Whitespace-trimmed line; leading indent and trailing space removed before key extraction.
   */
  const trimmed = line.trim();
  /**
   Position of the key/value colon; `<= 0` means there is no key before a colon.
   */
  const colonIdx = trimmed.indexOf(':',);
  if (colonIdx <= 0)
    return NO_KEY;
  /**
   Raw key segment before the colon, trailing space stripped.
   */
  const rawKey = trimmed
    .slice(
    0,
    colonIdx,
    )
    .trimEnd();
  /**
   Leading character; a wrapping quote must match on both ends to be stripped.
   */
  const first = rawKey.charAt(0,);
  /**
   Whether the leading character is an ASCII quote that could wrap the key.
   */
  const isQuote = (first === '"')
    || (first === '\'');
  /**
   Whether `rawKey` is wrapped in a matching quote pair, so a layer can be stripped.
   */
  const wrapped = (rawKey.length >= 2)
    && isQuote
    && rawKey.endsWith(first,);
  if (wrapped) {
    return rawKey.slice(
      1,
      -1,
    );
  }
  return rawKey;
}

/**
 Extracts a top-level YAML key, ignoring indented catalog entries.
 
 @param line - one raw line from the workspace file
 
 @returns top-level key, or {@link NO_KEY}
 
 @example
 ```ts
 topLevelLineKey('catalog:') // "catalog"
 topLevelLineKey('  foo: \">=1.0.0\"') // NO_KEY
 ```
 */
function topLevelLineKey(line: string,): string | typeof NO_KEY {
  /**
   Trimmed line used to distinguish blank and comment lines from YAML keys.
   */
  const trimmed = line.trim();
  if ((trimmed === '') || trimmed.startsWith('#'))
    return NO_KEY;
  if (line.startsWith(' ') || line.startsWith('\t'))
    return NO_KEY;
  return lineKey(line,);
}

/**
 Rewrites every tightened entry's value in the default `catalog:` block,
 preserving formatting. Named `catalogs:` blocks are left untouched even when
 they repeat a default catalog key. On a match, only the first occurrence of
 the old value is replaced, leaving surrounding quotes and spacing untouched.
 
 @param content - raw `pnpm-workspace.yaml` text
 
 @param results - {@link TightenedRange} entries to write back
 
 @returns rewritten file text
 
 @example
 ```ts
 rewriteCatalogRanges({
   content: "catalog:\n  'oxlint': '>=1.71.0'",
   results: [{ name: 'oxlint', oldRange: '>=1.71.0', newRange: '>=1.71.1' }],
 }) // "catalog:\n  'oxlint': '>=1.71.1'"
 ```
 */
export function rewriteCatalogRanges(
  {
    content,
    results,
  }: {
    readonly content: string;
    readonly results: readonly TightenedRange[];
  },
): string {
  /**
   Rewritten content produced by the stateful line walk.
   */
  const rewritten = (function rewriteLines(): string {
    /**
     Source lines traversed once for block detection and replacement.
     */
    const sourceLines = content.split('\n',);
    /**
     Whether the current line is inside the explicit default catalog block.
     */
    let inDefaultCatalog = false;
    /**
     Rewritten lines accumulated in source order.
     */
    const rewrittenLines: string[] = [];

    for (const line of sourceLines) {
      /**
       Top-level key used to enter or leave the default catalog block.
       */
      const topLevelKey = topLevelLineKey(line,);
      if (topLevelKey !== NO_KEY)
        inDefaultCatalog = topLevelKey === 'catalog';

      if (!inDefaultCatalog) {
        rewrittenLines.push(line,);
        continue;
      }

      /**
       Catalog key parsed from the current line; comments and headers pass through.
       */
      const key = lineKey(line,);
      if (key === NO_KEY) {
        rewrittenLines.push(line,);
        continue;
      }
      /**
       Tightened entry whose key matches this line, if any.
       */
      const result = results.find(function matchesKey(candidate,): boolean {
        return candidate.name === key;
      },);
      rewrittenLines.push(result === undefined
        ? line
        : line.replace(
          result.oldRange,
          result.newRange,
        ),);
    }

    return rewrittenLines.join('\n',);
  })();
  return rewritten;
}

//endregion Catalog rewrite
