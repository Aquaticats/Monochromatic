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
  readCssFileSync,
} from './fs.ts';
import { splitPackageSpecifier, } from './specifier.ts';

//region Package Resolution

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
 
 @param packageDir - Absolute path to the package directory
 
 @returns Parsed package.json, or {@link PACKAGE_JSON_ABSENT} when missing or unparseable
 
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
  try {
    /**
     Raw JSON text
     */
    const raw = readCssFileSync(packageJsonPath,);
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- JSON.parse returns unknown; package.json shape is Record<string, unknown>
    return JSON.parse(raw,) as Record<string, unknown>;
  }
  catch (error: unknown) {
    if (Error.isError(error,))
      return PACKAGE_JSON_ABSENT;

    throw error;
  }
}

/**
 Resolves a subpath using package.json `exports` field.
 Supports simple string mappings and condition objects with `style`, `import`, `default` keys.
 
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
  if (((typeof exports) !== 'object') || (exports === null))
    return NO_EXPORT_MATCH;

  /**
   Exports object keyed by subpath pattern
   */
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- narrowing from object to Record for property access
  const exportsMap = exports as Record<string, unknown>;
  /**
   Value for the requested subpath
   */
  const entry = exportsMap[subpath];

  if ((typeof entry) === 'string')
    return entry;

  // Condition object: check style -> import -> default
  if (((typeof entry) === 'object') && (entry !== null)) {
    /**
     Condition map for this subpath
     */
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- narrowing from object to Record for condition access
    const conditions = entry as Record<string, unknown>;
    for (const key of [
      'style',
      'import',
      'default',
    ]) {
      if ((typeof conditions[key]) === 'string')
        return conditions[key];
    }
  }

  return NO_EXPORT_MATCH;
}

/**
 Resolves a package specifier to an absolute file path.
 Tries `exports` field first (skipping past {@link NO_EXPORT_MATCH}), then
 falls back to `style`/`main` fields, then to the direct file path within
 the package. Throws when {@link findPackageDir} reports
 {@link PACKAGE_NOT_FOUND} or every fallback is exhausted; a
 {@link PACKAGE_JSON_ABSENT} manifest just skips the `exports`/`style`/`main`
 lookups and falls through to the direct file path.
 
 @param specifier - Full package specifier (e.g. `\@scope/pkg/index.css`)
 
 @param fromDir - Directory of the importing file
 
 @returns Absolute resolved path
 
 @throws When the package or file cannot be found
 
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
    // Try exports field
    if (packageJson.exports
      !== undefined) {
      /**
       Resolved path from exports map
       */
      const resolved = resolveExports({
        exports: packageJson.exports,
        subpath,
      },);
      if (resolved !== NO_EXPORT_MATCH) {
        /**
         Absolute path from exports resolution
         */
        const absolutePath = resolve(
          [
            packageDir,
            resolved,
          ],
        );
        if (existsSync(absolutePath,))
          return absolutePath;
      }
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

  // Direct file path fallback: resolve subpath relative to package directory
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
    `Cannot resolve '${specifier}' from '${fromDir}': no matching export, style/main field, or direct file in '${packageDir}'`,
  );
}

//endregion Package Resolution
