import { readFile } from 'node:fs/promises';
import { join } from 'node:path';

//region Types: bundle pattern and package manifest shapes

/**
 Bundle-inclusion pattern accepted by {@link packageExternals}.
 
 Three forms cover every pattern the repository uses:
 `@scope/**` (scope prefix), `name-**` (name prefix), and exact names.
 
 @example
 ```ts
 const patterns: readonly BundlePattern[] = ['\@monochromatic-dev/**', 'nano-spawn'];
 ```
 */
export type BundlePattern = string;

/**
 Dependency-bearing subset of a package manifest read by {@link packageExternals}.
 
 @example
 ```ts
 const manifest: PackageManifestDependencies = { dependencies: { browserslist: '>=4' } };
 ```
 */
export type PackageManifestDependencies = {
  readonly dependencies?: Readonly<Record<string, string>>;
  readonly peerDependencies?: Readonly<Record<string, string>>;
};

//endregion Types: bundle pattern and package manifest shapes

//region Pattern matching and regex construction

/**
 Decide whether a bare dependency name matches one bundle-inclusion pattern.
 
 Suffix `/**` matches the scope or path prefix,
 bare suffix `**` matches a name prefix,
 anything else matches exactly.
 
 @param name - Bare dependency name from the consuming manifest.
 
 @param pattern - Bundle-inclusion pattern in one of three supported forms.
 
 @returns Whether name falls under pattern and must stay bundled.
 
 @example
 ```ts
 matchesBundlePattern({ name: '\@monochromatic-dev/logger-core', pattern: '\@monochromatic-dev/**' });
 ```
 */
export function matchesBundlePattern({
  name,
  pattern
}: {
  readonly name: string;
  readonly pattern: BundlePattern;
}): boolean {
  if (pattern.endsWith('/**')) {
    return name.startsWith(pattern.slice(
      0,
      -'**'.length
    ));
  }
  if (pattern.endsWith('**')) {
    return name.startsWith(pattern.slice(
      0,
      -'**'.length
    ));
  }
  return name === pattern;
}

/**
 Escape a bare package name for anchored RegExp construction.
 
 @param name - Bare dependency name to escape.
 
 @returns Escaped text safe inside a RegExp source.
 
 @example
 ```ts
 escapePackageNameForRegExp('\@scope/pkg');
 ```
 */
function escapePackageNameForRegExp(name: string): string {
  // oxlint-disable-next-line no-restricted-syntax/no-regex -- Escaping metacharacters in a short bare package name; input bounded by manifest dependency keys; single linear character-class pass, no backtracking.
  return name.replaceAll(
    /[.*+?^${}()|[\]\\]/gu,
    String.raw`\$&`
  );
}

/**
 Build the anchored matcher marking one bare package name external,
 covering the bare name and every subpath under it.
 
 @param name - Bare dependency name to match.
 
 @returns Anchored matcher for rolldown's `external` input option.
 
 @example
 ```ts
 anchoredExternal('\@earendil-works/pi-ai');
 ```
 */
export function anchoredExternal(name: string,): RegExp {
  // oxlint-disable-next-line no-restricted-syntax/no-regex -- Dynamic source is one metacharacter-escaped bare package name anchored at both ends; input bounded by manifest dependency keys; no repetition beyond the literal name, no backtracking.
  return new RegExp(
    `^${escapePackageNameForRegExp(name)}(/|$)`,
    'u',
  );
}

//endregion Pattern matching and regex construction

//region Externals resolution

/**
 Narrow an unknown parsed manifest to its dependency-bearing shape.
 
 Validates only what {@link packageExternals} reads:
 a plain object whose optional dependency maps are objects.
 
 @param value - Parsed package.json content to narrow.
 
 @returns Nothing; success narrows value for the caller.
 
 @throws Error when value is not manifest-shaped.
 
 @example
 ```ts
 assertPackageManifestDependencies(JSON.parse('{}'));
 ```
 */
function assertPackageManifestDependencies(
  value: unknown,
): asserts value is PackageManifestDependencies {
  if (((typeof value) !== 'object') || (value === null)) {
    throw new Error('package.json must parse to an object',);
  }
  /**
   Manifest object exposing only the optional dependency maps to validate.
   */
  const withDependencyMaps = value as {
    readonly dependencies?: unknown;
    readonly peerDependencies?: unknown;
  };
  /**
   Optional dependency maps in validation order.
   */
  const dependencyMaps = [
    withDependencyMaps.dependencies,
    withDependencyMaps.peerDependencies,
  ];
  if (dependencyMaps.some(function isInvalidMap(map: unknown,): boolean {
    return (map !== undefined) && (((typeof map) !== 'object') || (map === null));
  },)) {
    throw new Error('package.json dependency maps must be objects when present',);
  }
}

/**
 Build the rolldown `external` array for one consuming package.
 
 Replicates the repository's tsdown-era auto-externalization:
 every declared dependency and peer dependency stays external
 unless a bundle pattern forces it inline,
 `node:` builtins stay external,
 and undeclared bare imports (transitives of inlined workspace source)
 bundle by omission so artifacts stay self-contained outside the monorepo.
 
 Anchored regexes cover subpath imports (`name/sub`) without matching
 unrelated prefixes,
 and the array form avoids rolldown's per-module JS callback overhead.
 
 @param packageDir - Directory holding the consuming package.json; defaults to the build cwd.
 
 @param alwaysBundle - Patterns whose matching dependencies must stay inline.
 
 @returns Regex list for rolldown's `external` input option.
 
 @example
 ```ts
 const external = await packageExternals({ alwaysBundle: ['\@monochromatic-dev/**'] });
 ```
 */
export async function packageExternals({
  packageDir = process.cwd(),
  alwaysBundle
}: {
  readonly packageDir?: string;
  readonly alwaysBundle: readonly BundlePattern[];
}): Promise<RegExp[]> {
  /**
   Raw parsed manifest before shape narrowing.
   */
  const parsedManifest: unknown = JSON.parse(
    await readFile(
      join(
        packageDir,
        'package.json'
      ),
      'utf8'
    ),
  );
  assertPackageManifestDependencies(parsedManifest,);

  /**
   Dependency-bearing manifest slice of the consuming package.
   */
  const manifest = parsedManifest;

  /**
   Declared dependency names staying external after bundle patterns apply.
   */
  const externalNames = Object.keys({
    ...manifest.dependencies,
    ...manifest.peerDependencies,
  })
    .filter(function isExternalName(name: string): boolean {
      return !alwaysBundle.some(function matchesName(pattern: BundlePattern): boolean {
        return matchesBundlePattern({
          name,
          pattern
        });
      });
    });

  return [
    // oxlint-disable-next-line no-restricted-syntax/no-regex -- Anchored protocol prefix test for node builtins; input is one bare import specifier; no repetition, no backtracking.
    /^node:/u,
    ...externalNames.map(function toAnchoredMatcher(name: string): RegExp {
      return anchoredExternal(name,);
    }),
  ];
}

//endregion Externals resolution
