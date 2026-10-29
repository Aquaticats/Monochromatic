import type {
  WatchCtx,
  WatchEvent,
  WatchFilter,
} from '../types.ts';

/**
 Normalises an extension token to `.<lowercase>`.
 
 Accepts both forms users naturally type at a CLI (`ts`, `.ts`, `TS`,
 `.TS`) and produces a canonical key for set lookup. Module-scope so the
 function declaration carries TSDoc and stays out of the filter's
 closure.
 
 @param ext - extension token; leading dot optional, case insensitive
 
 @returns canonical `.ext` form, lowercased
 
 @example
 ```ts
 normalizeExt('TS',);  // '.ts'
 normalizeExt('.TS',); // '.ts'
 ```
 */
function normalizeExt(ext: string,): string {
  /**
   Dot-prefixed form so `'ts'` and `'.ts'` collapse to the same canonical key before lowercasing.
   */
  const withDot = ext.startsWith('.',) ? ext : `.${ext}`;
  return withDot.toLowerCase();
}

/**
 Builds a {@link WatchFilter} that admits only events whose file extension
 matches one of `extensions`.
 
 The list is normalised on construction (leading dot added, lowercased),
 stored in a {@link Set} for O(1) lookup, and compared case-insensitively
 against `event.ext` (which arrives in the filesystem's original case).
 
 Empty `extensions` is a vacuous pass-all (returns `true` for every
 event); this lets the CLI flag-to-filter compiler skip an {@link extFilter}
 call entirely when `--ext` is unset, without a second branch.
 
 @param extensions - allowed extensions; leading dot optional, case insensitive
 
 @returns watch filter that returns `true` when `event.ext` matches any
   listed extension (or when the list is empty)
 
 @example
 ```ts
 const filter = extFilter(['.ts', 'tsx',],);
 filter({ event: { ext: '.TS', ... }, ctx, },); // true
 filter({ event: { ext: '.css', ... }, ctx, },); // false
 ```
 */
export function extFilter(extensions: readonly string[],): WatchFilter {
  /**
   Set of normalized extensions; built once at construction so the hot
   path does only a single `Set.has` per event.
   */
  const normalized: ReadonlySet<string> = new Set<string>(
    extensions.map(function mapNormalize(ext,) {
      return normalizeExt(ext,);
    },),
  );
  return function extFilterFn(
    {
      event,
    }: {
      readonly event: WatchEvent;
      readonly ctx: WatchCtx;
    },
  ): boolean {
    if (normalized.size
      === 0)
      return true;
    return normalized.has(event.ext
      .toLowerCase(),);
  };
}
