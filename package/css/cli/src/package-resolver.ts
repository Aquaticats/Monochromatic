/**
 Package specifier resolution for CSS @import.
 
 Resolves bare package specifiers (e.g. `\@scope/pkg/index.css`, `pkg/path.css`)
 to absolute file paths by traversing `node_modules` and consulting
 `exports`, `style`, and `main` fields in `package.json`.
 */
import {
  dirname,
  join,
  resolve,
} from '@monochromatic-dev/fs-path-core/ts';
import {
  existsSync,
  readTextFileSync,
} from './fs.ts';
import { CssPackageManifestError, } from './errors.ts';
import { splitPackageSpecifier, } from './specifier.ts';

//region Sentinels and package lookup

/**
 Sentinel returned by {@link findPackageDir} when no `node_modules/<pkg>` exists.
 A unique Symbol encodes genuine absence without a banned nullish union;
 identity comparison narrows the result back to `string`.
 */
const PACKAGE_NOT_FOUND: unique symbol = Symbol('node_modules package directory not found',);

/**
 Sentinel returned by {@link readPackageJson} when no readable package.json exists.
 A unique Symbol encodes genuine absence without a banned nullish union;
 identity comparison narrows the result back to the parsed record.
 */
const PACKAGE_JSON_ABSENT: unique symbol = Symbol('package.json manifest unreadable or absent',);

/**
 Sentinel returned by {@link resolveExports} when no `exports` entry matches.
 A unique Symbol encodes genuine absence without a banned nullish union;
 identity comparison narrows the result back to `string`.
 */
const NO_EXPORT_MATCH: unique symbol = Symbol('package.json exports entry not matched',);

/**
 Walks up from `startDir` looking for a `node_modules/<packageName>` directory.
 Mimics Node's module resolution algorithm via tail recursion: each call
 checks one directory and recurses into the parent until either the package
 is found or the filesystem root is reached.
 Stays synchronous and hand-rolled rather than using the async `findRoot`
 walker of `@monochromatic-dev/fs-path-core`: it runs inside the
 synchronous `transformStylesheet` visitor and probes through this
 package's in-memory registry.
 
 @param startDir - Directory to inspect this iteration
 
 @param packageName - Package name (e.g. `\@scope/pkg`)
 
 @returns Absolute path to the package directory, or {@link PACKAGE_NOT_FOUND} if no `node_modules/<pkg>` exists up to the filesystem root
 
 @example
 ```ts
 const dir = findPackageDir({ startDir: '/project/src', packageName: '\@scope/tokens' });
 // → '/project/node_modules/\@scope/tokens'
 ```
 */
export function findPackageDir({
  startDir,
  packageName,
}: {
  readonly startDir: string;
  readonly packageName: string;
},):
  | string
    | typeof PACKAGE_NOT_FOUND
  {
  /**
   Candidate node_modules/<pkg> directory
   */
  const candidate = join(
    [
      startDir,
      'node_modules',
      packageName,
    ],
  );
  if (existsSync(candidate,))
    return candidate;

  /**
   Parent directory
   */
  const parent = dirname(startDir,);
  // Reached filesystem root
  if (parent === startDir)
    return PACKAGE_NOT_FOUND;
  return findPackageDir({
    startDir: parent,
    packageName,
  },);
}

/**
 Reads and parses a package.json from the given directory.
 Absence is decided before reading, so a manifest that exists but cannot be
 read or parsed is reported as such instead of being mistaken for one the
 package does not have.
 
 @param packageDir - Absolute path to the package directory
 
 @returns Parsed package.json, or {@link PACKAGE_JSON_ABSENT} when the directory holds no manifest
 
 @throws CssPackageManifestError when a manifest exists but cannot be read or
 parsed; it names the manifest path and carries the underlying failure
 
 @example
 ```ts
 const pkg = readPackageJson('/project/node_modules/\@scope/tokens');
 ```
 */
export function readPackageJson(
  packageDir: string,
): Record<string, unknown> | typeof PACKAGE_JSON_ABSENT {
  /**
   Path to package.json
   */
  const packageJsonPath = join(
    [
      packageDir,
      'package.json',
    ],
  );
  if (!existsSync(packageJsonPath,))
    return PACKAGE_JSON_ABSENT;

  try {
    /**
     Raw JSON text
     */
    const raw = readTextFileSync(packageJsonPath,);
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- JSON.parse returns unknown; package.json shape is Record<string, unknown>
    return JSON.parse(raw,) as Record<string, unknown>;
  }
  catch (error: unknown) {
    throw new CssPackageManifestError({
      manifestPath: packageJsonPath,
      cause: error,
    },);
  }
}

//endregion Sentinels and package lookup

//region Exports field resolution

/**
 Condition keys honored inside an exports entry, in the order they win.
 `style` comes first because a stylesheet consumer asked for CSS; `import`
 and `default` are Node's own fallbacks.
 */
const EXPORT_CONDITION_KEYS = [
  'style',
  'import',
  'default',
] as const;

/**
 Sentinel returned by {@link matchSubpathPattern} when a pattern key does not
 describe the requested subpath.
 */
const NO_PATTERN_MATCH: unique symbol = Symbol('exports pattern did not match the subpath',);

/**
 Normalizes an `exports` value into a map keyed by subpath.
 Node allows the field to be a string, an array of alternatives, or a
 condition object at the top level; all three describe the `.` subpath, so
 they are wrapped in one rather than rejected.
 
 @param exports - The `exports` field value from package.json
 
 @returns Subpath-keyed map, or {@link NO_EXPORT_MATCH} when the value cannot
 describe any subpath
 */
function toSubpathMap(exports: unknown,): Record<string, unknown> | typeof NO_EXPORT_MATCH {
  if (((typeof exports) === 'string') || Array.isArray(exports,))
    return { '.': exports, };

  if (((typeof exports) !== 'object') || (exports === null))
    return NO_EXPORT_MATCH;

  /**
   Exports object, keyed either by subpath or by condition.
   */
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- narrowing from object to Record for property access
  const exportsRecord = exports as Record<string, unknown>;
  /**
   Whether the object is keyed by subpath rather than by condition name.
   */
  const isSubpathMap = Object.keys(exportsRecord,)
    .some(function startsWithDot(key,) {
      return key.startsWith('.',);
    },);
  return isSubpathMap ? exportsRecord : { '.': exportsRecord, };
}

/**
 Resolves one exports entry to a path: a string stands for itself, an array
 lists alternatives in preference order, and an object is a condition map
 read in {@link EXPORT_CONDITION_KEYS} order.
 Substitutes the pattern wildcard into the chosen target, as Node does for
 `./*` style keys.
 
 @param target - Exports entry or nested condition value
 
 @param wildcard - Text a pattern key captured, empty for an exact key
 
 @returns Relative path, or {@link NO_EXPORT_MATCH} when no alternative yields one
 */
function resolveExportTarget({
  target,
  wildcard,
}: {
  readonly target: unknown;
  readonly wildcard: string;
},): string | typeof NO_EXPORT_MATCH {
  if ((typeof target) === 'string')
    return target.replaceAll(
      '*',
      wildcard,
    );

  if (Array.isArray(target,)) {
    for (const alternative of target) {
      /**
       First alternative that resolves wins; Node treats the rest as
       unreachable for a resolvable specifier.
       */
      const resolvedAlternative = resolveExportTarget({
        target: alternative,
        wildcard,
      },);
      if (resolvedAlternative !== NO_EXPORT_MATCH)
        return resolvedAlternative;
    }
    return NO_EXPORT_MATCH;
  }

  if (((typeof target) === 'object') && (target !== null)) {
    /**
     Condition map for this entry
     */
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- narrowing from object to Record for condition access
    const conditions = target as Record<string, unknown>;
    for (const key of EXPORT_CONDITION_KEYS) {
      /**
       Value the condition selects, absent when the manifest omits the key.
       */
      const conditionTarget = conditions[key];
      if (conditionTarget === undefined)
        continue;

      /**
       Path the condition selects, when it selects one.
       */
      const resolvedCondition = resolveExportTarget({
        target: conditionTarget,
        wildcard,
      },);
      if (resolvedCondition !== NO_EXPORT_MATCH)
        return resolvedCondition;
    }
  }

  return NO_EXPORT_MATCH;
}

/**
 Text a `*` in an exports key captures for a requested subpath.
 
 @param pattern - Exports key holding one `*`
 
 @param subpath - Requested subpath
 
 @returns Captured text, or {@link NO_PATTERN_MATCH} when the key does not
 describe the subpath
 */
function matchSubpathPattern({
  pattern,
  subpath,
}: {
  readonly pattern: string;
  readonly subpath: string;
},): string | typeof NO_PATTERN_MATCH {
  /**
   Index of the single wildcard a pattern key may hold.
   */
  const starIndex = pattern.indexOf('*',);
  if (starIndex === (-1))
    return NO_PATTERN_MATCH;

  /**
   Key text the subpath must start with.
   */
  const prefix = pattern.slice(
    0,
    starIndex,
  );
  /**
   Key text the subpath must end with.
   */
  const suffix = pattern.slice(starIndex + 1,);
  if ((!subpath.startsWith(prefix,)) || (!subpath.endsWith(suffix,)))
    return NO_PATTERN_MATCH;
  if (subpath.length < (prefix.length + suffix.length))
    return NO_PATTERN_MATCH;

  return subpath.slice(
    prefix.length,
    subpath.length - suffix.length,
  );
}

/**
 Pattern keys of an exports map, most specific first: Node prefers the
 pattern whose prefix before the wildcard is longest, then the longest key.
 
 @param subpathMap - Exports map keyed by subpath
 
 @returns Pattern keys in resolution order
 */
function patternKeysBySpecificity(
  subpathMap: Readonly<Record<string, unknown>>,
): string[] {
  return Object.keys(subpathMap,)
    .filter(function holdsWildcard(key,) {
      return key.includes('*',);
    },)
    .toSorted(function mostSpecificFirst(
      left,
      right,
    ) {
      return (right.indexOf('*',) - left.indexOf('*',))
        || (right.length - left.length);
    },);
}

/**
 Resolves a subpath using package.json `exports` field, following Node's
 resolution shapes: a top-level string or array describes `.`, exact subpath
 keys win outright, `*` keys match by longest prefix, and a value may be a
 string, an array of alternatives, or nested condition objects read in
 `style` then `import` then `default` order.
 
 @param exports - The `exports` field value from package.json
 
 @param subpath - Subpath to resolve (e.g. `./index.css` or `.`)
 
 @returns Resolved relative path, or {@link NO_EXPORT_MATCH} when no entry matches
 
 @example
 ```ts
 resolveExports({
   exports: { '.': { style: './dist/index.css' } },
   subpath: '.',
 }); // → './dist/index.css'
 ```
 */
export function resolveExports({
  exports,
  subpath,
}: {
  readonly exports: unknown;
  readonly subpath: string;
},): string | typeof NO_EXPORT_MATCH {
  /**
   Exports map keyed by subpath, or the sentinel when the field shape cannot
   describe any subpath.
   */
  const subpathMap = toSubpathMap(exports,);
  if (subpathMap === NO_EXPORT_MATCH)
    return NO_EXPORT_MATCH;

  /**
   Value the exact subpath key carries, absent when the map has no such key.
   */
  const exactTarget = subpathMap[subpath];
  // An exact key states its own answer; no pattern may override it.
  if (exactTarget !== undefined) {
    return resolveExportTarget({
      target: exactTarget,
      wildcard: '',
    },);
  }

  for (const pattern of patternKeysBySpecificity(subpathMap,)) {
    /**
     Text the pattern captures for this subpath.
     */
    const wildcard = matchSubpathPattern({
      pattern,
      subpath,
    },);
    if (wildcard === NO_PATTERN_MATCH)
      continue;

    /**
     Path the matching pattern selects.
     */
    const resolved = resolveExportTarget({
      target: subpathMap[pattern],
      wildcard,
    },);
    if (resolved !== NO_EXPORT_MATCH)
      return resolved;
  }

  return NO_EXPORT_MATCH;
}

//endregion Exports field resolution

//region Resolution pipeline

/**
 Resolves a package specifier to an absolute file path.
 A package that declares `exports` answers from that map alone: a subpath the
 map does not match, or matches to a missing file, throws instead of reaching
 into the package tree for something the author did not export. Without an
 `exports` field the `style`/`main` fields answer a bare `.` reference, and a
 direct file path answers any other subpath.
 
 @param specifier - Full package specifier (e.g. `\@scope/pkg/index.css`)
 
 @param fromDir - Directory of the importing file
 
 @returns Absolute resolved path
 
 @throws When the package, the exported subpath, or the file cannot be found
 
 @example
 ```ts
 const path = resolvePackage({
   specifier: '\@scope/tokens/index.css',
   fromDir: '/project/src',
 });
 ```
 */
export function resolvePackage({
  specifier,
  fromDir,
}: {
  readonly specifier: string;
  readonly fromDir: string;
},): string {
  /**
   Split decouples package directory lookup from sub-path resolution.
   */
  const [packageName, subpath,] = splitPackageSpecifier(specifier,);
  /**
   Absolute path to the package directory in node_modules
   */
  const packageDir = findPackageDir({
    startDir: fromDir,
    packageName,
  },);

  if (packageDir === PACKAGE_NOT_FOUND)
    throw new Error(`Cannot find package '${packageName}' from '${fromDir}'`,);

  /**
   Parsed package.json for exports/main/style lookup
   */
  const packageJson = readPackageJson(packageDir,);

  if (packageJson !== PACKAGE_JSON_ABSENT) {
    if (packageJson.exports
      !== undefined) {
      /**
       Path the exports map names for this subpath.
       */
      const resolved = resolveExports({
        exports: packageJson.exports,
        subpath,
      },);
      if (resolved === NO_EXPORT_MATCH) {
        throw new Error(
          `Cannot resolve '${specifier}' from '${fromDir}': package '${packageName}' does not export subpath '${subpath}'`,
        );
      }

      /**
       Absolute path from exports resolution
       */
      const absolutePath = resolve(
        [
          packageDir,
          resolved,
        ],
      );
      if (!existsSync(absolutePath,)) {
        throw new Error(
          `Cannot resolve '${specifier}' from '${fromDir}': package '${packageName}' exports subpath '${subpath}' as '${resolved}', which does not exist`,
        );
      }
      return absolutePath;
    }

    // For bare package reference (subpath is '.'), try style/main fields
    if (subpath === '.') {
      for (const field of [
        'style',
        'main',
      ]) {
        /**
         Field value from package.json
         */
        const value = packageJson[field];
        if ((typeof value) === 'string') {
          /**
           Absolute path from style/main field
           */
          const absolutePath = resolve(
            [
              packageDir,
              value,
            ],
          );
          if (existsSync(absolutePath,))
            return absolutePath;
        }
      }
    }
  }

  // Direct file path fallback: reachable only for a package without an
  // exports map, which answered style/main above or has no manifest at all.
  if (subpath !== '.') {
    // subpath starts with './': strip it for join
    /**
     Relative portion after stripping leading ./
     */
    const relativePart = subpath.startsWith('./',) ? subpath.slice(2,) : subpath;
    /**
     Absolute path from direct file reference
     */
    const directPath = join(
      [
        packageDir,
        relativePart,
      ],
    );
    if (existsSync(directPath,))
      return directPath;
  }

  throw new Error(
    `Cannot resolve '${specifier}' from '${fromDir}': no style/main field or direct file in '${packageDir}'`,
  );
}

//endregion Resolution pipeline
