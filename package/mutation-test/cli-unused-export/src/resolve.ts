/**
 Import-specifier resolution over discovered workspace sources.
 
 @example
 ```ts
 const resolve = workspaceResolver({ packageDirsByName, fileSet });
 resolve('./util.ts', 'package/mutation-test/cli-unused-export/src/cli.ts');
 ```
 */

import { posix, } from 'node:path';

/**
 Workspace package scope every internal import starts with.
 */
const WORKSPACE_SCOPE = '@monochromatic-dev/';

/**
 Built-artifact path marker; test files import package behavior from
 built dist, and mapping those imports back to source keeps test usage
 visible to reference counting.
 */
const DIST_MARKER = '/dist/final/';

/**
 Returns the first candidate present among analyzable sources.
 
 @param candidates - Workspace-relative candidate paths in priority order.
 
 @param fileSet - Every analyzable source path.
 
 @returns First present candidate, or null for none.
 
 @example
 ```ts
 firstPresent({ candidates: ['a.ts', 'a/index.ts'], fileSet });
 ```
 */
function firstPresent({
  candidates,
  fileSet,
}: Readonly<{
  candidates: readonly string[];
  fileSet: ReadonlySet<string>;
  // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- Feeds the yuku-analyzer resolve contract, whose external-module sentinel is the literal `null`.
}>,): string | null {
  for (const candidate of candidates) {
    if (fileSet.has(candidate,))
      return candidate;
  }
  return null;
}

/**
 Creates the yuku-analyzer resolver for one discovered workspace.
 
 Relative specifiers resolve against the importer with extension and
 index probing. Built-dist specifiers map back to the owning package's
 source entry. Workspace `@monochromatic-dev/<name>/ts` specifiers
 resolve through the shared `./ts` exports convention. Everything else
 is external.
 
 @param packageDirsByName - Package directory looked up by package name.
 
 @param fileSet - Every analyzable source path.
 
 @returns Resolver mapping specifiers to added file paths or null.
 
 @example
 ```ts
 const resolve = workspaceResolver({ packageDirsByName, fileSet });
 ```
 */
export function workspaceResolver({
  packageDirsByName,
  fileSet,
}: Readonly<{
  packageDirsByName: ReadonlyMap<string, string>;
  fileSet: ReadonlySet<string>;
}>,): (
  specifier: string,
  importerPath: string,
  // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- Mirrors the yuku-analyzer resolve contract, whose external-module sentinel is the literal `null`.
) => string | null {
  return function resolveSpecifier(
    specifier: string,
    importerPath: string,
    // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- Mirrors the yuku-analyzer resolve contract, whose external-module sentinel is the literal `null`.
  ): string | null {
    if (specifier.startsWith('.',)) {
      /**
       Importer-relative target normalized to workspace-relative form.
       */
      const joined = posix.normalize(posix.join(
        posix.dirname(importerPath,),
        specifier,
      ),);

      if (joined.includes(DIST_MARKER,)) {
        /**
         Owning package directory ahead of the dist marker.
         */
        const packageDir = joined.slice(
          0,
          joined.indexOf(DIST_MARKER,),
        );
        return firstPresent({
          candidates: [posix.join(
            packageDir,
            'src/index.ts',
          ),],
          fileSet,
        },);
      }

      return firstPresent({
        candidates: [
          joined,
          `${joined}.ts`,
          `${joined}.tsx`,
          posix.join(
            joined,
            'index.ts',
          ),
        ],
        fileSet,
      },);
    }

    if (!specifier.startsWith(WORKSPACE_SCOPE,))
      return null;

    /**
     Specifier segments after the workspace scope.
     */
    const segments = specifier
      .slice(WORKSPACE_SCOPE.length,)
      .split('/',);
    /**
     Package directory for the named workspace package.
     */
    const packageDir = packageDirsByName.get(`${WORKSPACE_SCOPE}${segments[0] ?? ''}`,);

    if ((packageDir === undefined) || (segments[1] !== 'ts'))
      return null;

    /**
     Subpath below the `./ts` exports entry, empty for the entry itself.
     */
    const subpath = segments
      .slice(2,)
      .join('/',);

    if (subpath === '')
      return firstPresent({
        candidates: [posix.join(
          packageDir,
          'src/index.ts',
        ),],
        fileSet,
      },);

    return firstPresent({
      candidates: [
        posix.join(
          packageDir,
          'src',
          subpath,
        ),
        posix.join(
          packageDir,
          'src',
          `${subpath}.ts`,
        ),
        posix.join(
          packageDir,
          'src',
          subpath,
          'index.ts',
        ),
      ],
      fileSet,
    },);
  };
}
