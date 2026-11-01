import {
  readdir,
  readFile,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import {
  allInInputOrder,
  isJsonRecord,
} from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Rotation sweep packages
// WHICH TEST FILES THE SWEEP RUNS (ledger B701). A scripted case can ask a
// seat only through the stage package, so the packages swept are those whose
// manifest reaches the stage package through the family's dependencies, the
// stage package among them; the test files are the ones `test:unit` runs.

/**
 Directory holding the family's packages.
 */
export const FAMILY_ROOT: string = resolve(
  import.meta.dirname,
  '..',
  '..',
  '..',
);

/**
 Name of the package that owns `benchRotation`, from which the packages to sweep are found.
 */
const STAGE_PACKAGE_NAME = '@monochromatic-dev/translation-repair-stage';

/**
 Package directories left out of the sweep: the scans read source and docs and
 never ask a seat, and run once per salt they would only repeat themselves.
 */
const NOT_SWEPT: ReadonlySet<string> = new Set(['source-scan',],);

/**
 What the sweep reads of one package's manifest.
 */
type PackageEntry = {
  /**
   Directory name under the family directory.
   */
  readonly directory: string;
  /**
   Package name.
   */
  readonly name: string;
  /**
   Names of the packages it lists under `dependencies` and `devDependencies`.
   */
  readonly dependencies: readonly string[];
};

/**
 Names a manifest lists under one of its dependency fields.

 @param manifest - parsed `package.json`

 @param field - `dependencies` or `devDependencies`

 @returns The names, none when the field is absent

 @example
 ```ts
 const names = dependencyNames({ manifest: { dependencies: { purr: '1', }, }, field: 'dependencies', },); // ['purr']
 ```
 */
function dependencyNames(
  {
    manifest,
    field,
  }: {
    readonly manifest: Readonly<Record<string, unknown>>;
    readonly field: string;
  },
): readonly string[] {
  /**
   The field's value.
   */
  const listed = manifest[field];
  return isJsonRecord(listed,) ? Object.keys(listed,) : [];
}

/**
 Every package directory of the family that holds a manifest.

 @returns One entry per package, in directory order

 @throws Error when a manifest names no package

 @example
 ```ts
 const entries = await packageEntries();
 ```
 */
async function packageEntries(): Promise<readonly PackageEntry[]> {
  /**
   Directories of the family that hold a manifest.
   */
  const directories = (await readdir(
    FAMILY_ROOT,
    { withFileTypes: true, },
  )).filter(function isDirectory(entry,): boolean {
    return entry.isDirectory();
  },)
    .map(function directoryName(entry,): string {
      return entry.name;
    },)
    .toSorted();
  /**
   Entries, a directory without a manifest left out.
   */
  const entries = await allInInputOrder({
    members: directories.map(async function entryOf(directory,): Promise<readonly PackageEntry[]> {
      if (!(await readdir(join(
        FAMILY_ROOT,
        directory,
      ),)).includes('package.json',))
        return [];
      /**
       The parsed manifest.
       */
      const manifest: unknown = JSON.parse(await readFile(
        join(
          FAMILY_ROOT,
          directory,
          'package.json',
        ),
        'utf8',
      ),);
      if ((!isJsonRecord(manifest,)) || ((typeof manifest.name) !== 'string'))
        throw new Error(`rotation sweep: ${directory}/package.json names no package`,);
      return [
        {
          directory,
          name: manifest.name,
          dependencies: [
            ...dependencyNames({
              manifest,
              field: 'dependencies',
            },),
            ...dependencyNames({
              manifest,
              field: 'devDependencies',
            },),
          ],
        },
      ];
    },),
  },);
  return entries.flat();
}

/**
 Packages that can reach the stage package, by directory name.

 @returns Directories of the family whose `package.json` names the stage package among its dependencies, directly or through other family packages, the stage package included, the scans left out

 @example
 ```ts
 const names = await packagesReachingStage(); // ['adjudication', ..., 'stage']
 ```
 */
export async function packagesReachingStage(): Promise<readonly string[]> {
  /**
   Dependencies by package name.
   */
  const dependenciesOf = new Map((await packageEntries()).map(function pair(entry,): readonly [
    string,
    PackageEntry,
  ] {
    return [
      entry.name,
      entry,
    ];
  },),);
  /**
   Whether a package reaches the stage package, walking dependencies once each.

   @param name - package to start from

   @param seen - packages passed on the way here

   @returns Whether the stage package is the package or one of its dependencies, direct or not
   */
  function reaches(
    {
      name,
      seen,
    }: {
      readonly name: string;
      readonly seen: ReadonlySet<string>;
    },
  ): boolean {
    if (name === STAGE_PACKAGE_NAME)
      return true;
    if (seen.has(name,))
      return false;
    /**
     Packages passed on the way here, this one included.
     */
    const passed = new Set([
      ...seen,
      name,
    ],);
    return (dependenciesOf.get(name,)
      ?.dependencies
      ?? []).some(function dependencyReaches(dependency,): boolean {
        return dependenciesOf.has(dependency,) && reaches({
          name: dependency,
          seen: passed,
        },);
      },);
  }
  return [...dependenciesOf.values(),]
    .filter(function isSwept(entry,): boolean {
      return (!NOT_SWEPT.has(entry.directory,)) && reaches({
        name: entry.name,
        seen: new Set(),
      },);
    },)
    .map(function directoryName(entry,): string {
      return entry.directory;
    },)
    .toSorted();
}

/**
 Unit test files of a package, as the `test:unit` task finds them: every `*.unit.test.ts` under its `src`, `*.expensive.*` left out.

 @param packageDirectory - directory name under the family directory

 @returns Absolute paths, sorted

 @example
 ```ts
 const files = await unitTestFilesOf({ packageDirectory: 'stage', },);
 ```
 */
export async function unitTestFilesOf(
  { packageDirectory, }: { readonly packageDirectory: string; },
): Promise<readonly string[]> {
  // Only `src` is read: a package's `node_modules` holds the logger's files, hundreds of thousands of them, and every test file sits under `src`.
  /**
   Source directory of the package, absolute.
   */
  const root = join(
    FAMILY_ROOT,
    packageDirectory,
    'src',
  );
  return (await readdir(
    root,
    { recursive: true, },
  ))
    .filter(function isUnitTest(path,): boolean {
      return path.endsWith('.unit.test.ts',) && (!path.includes('.expensive.',));
    },)
    .map(function absolute(path,): string {
      return join(
        root,
        path,
      );
    },)
    .toSorted();
}

//endregion Rotation sweep packages
