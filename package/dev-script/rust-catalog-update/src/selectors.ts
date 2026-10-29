/**
 pnpm-style exclude selectors for `minimumReleaseAgeExclude` and `trustPolicyExclude`.

 A selector is a package name, a name pattern with `*` wildcards, or a name
 with exact versions joined by `||` (`webpack@4.47.0 || 5.102.1`), as pnpm
 documents in `docs/settings/dependency-resolution.md`.

 @module
 */

//region Parse

/**
 A parsed selector.

 @example
 ```ts
 const selector: ParsedSelector = { pattern: 'zbus', versions: ['5.19.0',], };
 ```
 */
export type ParsedSelector = {
  readonly pattern: string;
  readonly versions: readonly string[];
};

/**
 Parses one selector string.

 @param selector - selector text such as `zbus@5.19.0 || 5.19.1`

 @returns name pattern and exact versions (empty for every version)

 @example
 ```ts
 parseSelector('ring'); // { pattern: 'ring', versions: [] }
 ```
 */
export function parseSelector(selector: string,): ParsedSelector {
  /**
   Position of the version separator.
   */
  const at = selector.indexOf('@',);
  if (at === -1)
    return { pattern: selector.trim(), versions: [], };
  return {
    pattern: selector.slice(0, at,).trim(),
    versions: selector.slice(at + 1,).split('||',).map(function trimVersion(version: string,): string {
      return version.trim();
    },).filter(function isPresent(version: string,): boolean {
      return version.length > 0;
    },),
  };
}

//endregion Parse

//region Match

/**
 Whether `name` matches a pattern whose `*` stands for any run of characters.

 @param pattern - name pattern

 @param name - crate name

 @returns whether the whole name matches

 @example
 ```ts
 matchesPattern({ pattern: 'tokio*', name: 'tokio-util', }); // true
 ```
 */
export function matchesPattern({ pattern, name, }: { readonly pattern: string; readonly name: string; },): boolean {
  /**
   Literal pieces between wildcards.
   */
  const pieces = pattern.split('*',);
  if (pieces.length === 1)
    return pattern === name;
  /**
   First literal piece, which must start the name.
   */
  const first = pieces[0] ?? '';
  /**
   Last literal piece, which must end the name.
   */
  const last = pieces.at(-1,) ?? '';
  if (!name.startsWith(first,) || !name.endsWith(last,) || (name.length < first.length + last.length))
    return false;
  /**
   Search state: offset after the last matched piece.
   */
  const cursor = { offset: first.length, };
  for (const piece of pieces.slice(1, -1,)) {
    /**
     Next occurrence of the middle piece.
     */
    const found = name.indexOf(piece, cursor.offset,);
    if ((found === -1) || (found + piece.length > name.length - last.length))
      return false;
    cursor.offset = found + piece.length;
  }
  return true;
}

/**
 Whether any selector covers `name@version`.

 @param selectors - selector strings

 @param name - crate name

 @param version - locked version

 @returns whether the version is excluded

 @example
 ```ts
 isExcluded({ selectors: ['zbus@5.19.0',], name: 'zbus', version: '5.19.0', }); // true
 ```
 */
export function isExcluded(
  { selectors, name, version, }: { readonly selectors: readonly string[]; readonly name: string; readonly version: string; },
): boolean {
  return selectors.some(function covers(selector: string,): boolean {
    /**
     Parsed selector.
     */
    const parsed = parseSelector(selector,);
    return matchesPattern({ pattern: parsed.pattern, name, },) && ((parsed.versions.length === 0) || parsed.versions.includes(version,));
  },);
}

/**
 Selectors that still match a locked version or are name patterns (pnpm keeps patterns when pruning).

 @param selectors - selector strings

 @param locked - every locked `name@version`

 @returns selectors to keep, in order; version-specific selectors keep only versions still locked

 @example
 ```ts
 pruneSelectors({ selectors: ['gone@1.0.0',], locked: new Set(), }); // []
 ```
 */
export function pruneSelectors({ selectors, locked, }: { readonly selectors: readonly string[]; readonly locked: ReadonlySet<string>; },): readonly string[] {
  /**
   Selectors kept so far.
   */
  const kept: string[] = [];
  for (const selector of selectors) {
    /**
     Parsed selector.
     */
    const parsed = parseSelector(selector,);
    if (parsed.pattern.includes('*',)) {
      kept.push(selector,);
      continue;
    }
    if (parsed.versions.length === 0) {
      /**
       Whether any locked version has this name.
       */
      const present = [...locked,].some(function hasName(entry: string,): boolean {
        return entry.startsWith(`${parsed.pattern}@`,);
      },);
      if (present)
        kept.push(selector,);
      continue;
    }
    /**
     Versions of this selector still locked.
     */
    const remaining = parsed.versions.filter(function isLocked(version: string,): boolean {
      return locked.has(`${parsed.pattern}@${version}`,);
    },);
    if (remaining.length > 0)
      kept.push(`${parsed.pattern}@${remaining.join(' || ',)}`,);
  }
  return kept;
}

//endregion Match
