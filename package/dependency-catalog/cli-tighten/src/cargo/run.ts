/**
 Runs the Cargo source of catalog-tighten against `rust-catalog.toml` and every crate's `Cargo.lock`.

 @module
 */

import {
  glob,
  readFile,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';

import {
  parseRustCatalog,
  RUST_CATALOG_FILENAME,
} from '@monochromatic-dev/dependency-catalog-core-rust/ts';
import {
  parseTomlEdit,
  tomlGetValue,
} from '@monochromatic-dev/toml-edit-core/ts';

import { directDependencyVersions, } from './lockfile.ts';
import {
  type CargoTightenResult,
  type CrateUsage,
  planCargoTighten,
  rewriteCatalogFloors,
} from './tighten.ts';

//region Files

/**
 Sentinel for a file that does not exist.
 */
const ABSENT_FILE: unique symbol = Symbol('catalog-tighten/cargo: file does not exist on disk',);

/**
 Width of the status column, matching the JS tightener's output.
 */
const STATUS_WIDTH = 5;

/**
 Reads a UTF-8 file, or reports that it does not exist.

 @param path - file to read

 @returns file text, or {@link ABSENT_FILE} when the file does not exist

 @throws Error for every read failure other than a missing file

 @example
 ```ts
 await readOptionalFile('package/cli/a/Cargo.lock');
 ```
 */
async function readOptionalFile(path: string,): Promise<string | typeof ABSENT_FILE> {
  try {
    return await readFile(
      path,
      'utf8',
    );
  } catch (error) {
    if (Error.isError(error,) && ('code' in error)
      && (error.code === 'ENOENT'))
      return ABSENT_FILE;
    throw error;
  }
}

//endregion Files

//region Crates

/**
 Manifest discovery globs, matching file-enforcer's Rust catalog sync.
 */
const MANIFEST_GLOBS = [
  'package/*/*/Cargo.toml',
  'package/*/*/*/Cargo.toml',
] as const;

/**
 Whether `value` is a plain table.

 @param value - materialized TOML value

 @returns whether it is a non-array object

 @example
 ```ts
 isTable({}); // true
 ```
 */
function isTable(value: unknown,): value is Readonly<Record<string, unknown>> {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value,));
}

/**
 Reads one crate's default-catalog usage and lockfile versions.

 A dependency counts as default-catalog usage when it appears in the crate's
 generated `[workspace.dependencies]` table and the crate does not select a
 named catalog for it.

 @param root - repository root

 @param manifestPath - repo-relative manifest path

 @returns usage, or none when the crate has no `Cargo.lock` or no package name

 @example
 ```ts
 await readCrateUsage({ root, manifestPath: 'package/cli/a/Cargo.toml', });
 ```
 */
async function readCrateUsage({
  root,
  manifestPath,
}: {
  readonly root: string;
  readonly manifestPath: string
},): Promise<readonly CrateUsage[]> {
  /**
   Crate directory.
   */
  const crateDir = dirname(manifestPath,);
  /**
   Manifest as plain values.
   */
  const manifest = tomlGetValue({
    edit: parseTomlEdit({ source: await readFile(
      join(
        root,
        manifestPath,
      ),
      'utf8',
    ), },),
    path: [],
  },);
  if (!isTable(manifest,))
    return [];
  /**
   `[package]` table.
   */
  const pkg = isTable(manifest.package,) ? manifest.package : {};
  /**
   Generated catalog table.
   */
  const workspace = isTable(manifest.workspace,) ? manifest.workspace : {};
  /**
   Generated dependency entries.
   */
  const generated = isTable(workspace.dependencies,) ? workspace.dependencies : {};
  /**
   Named-catalog selections.
   */
  const metadata = isTable(pkg.metadata,) ? pkg.metadata : {};
  /**
   This tool's metadata table.
   */
  const catalogMetadata = isTable(metadata['rust-catalog'],) ? metadata['rust-catalog'] : {};
  /**
   Selected names, excluded from default-catalog usage.
   */
  const selected = isTable(catalogMetadata.catalogs,) ? new Set(Object.keys(catalogMetadata.catalogs,),) : new Set<string>();
  /**
   Lockfile text; a crate without one has nothing to tighten against.
   */
  const lock = await readOptionalFile(join(
    root,
    crateDir,
    'Cargo.lock',
  ),);
  if ((lock === ABSENT_FILE) || ((typeof pkg.name) !== 'string'))
    return [];
  return [{
    crateDir,
    uses: new Set(Object.keys(generated,)
      .filter(function isDefault(name: string,): boolean {
        return !selected.has(name,);
      },),),
    lockVersions: directDependencyVersions({
      source: lock,
      packageName: pkg.name,
    },),
  },];
}

//endregion Crates

//region Run

/**
 Prints one result in the JS tightener's status format.

 @param result - tighten outcome

 @example
 ```ts
 printResult({ name: 'anyhow', status: 'OK', oldRequirement: '>=1', newRequirement: '>=1', detail: '', });
 ```
 */
function printResult(result: CargoTightenResult,): void {
  /**
   Status column padded like the JS tightener.
   */
  const status = result.status
    .padEnd(STATUS_WIDTH,);
  if (result.status === 'TIGHT')
    console.info(`${status} ${result.name}: ${result.oldRequirement} -> ${result.newRequirement}`,);
  else
    console.info(`${status} ${result.name}: ${result.oldRequirement}${result.detail === '' ? '' : ` (${result.detail})`}`,);
}

/**
 Tightens `rust-catalog.toml` floors to the highest versions the consuming crates' lockfiles hold.

 A repository without the catalog is left alone.

 @param root - repository root

 @param dryRun - report without writing

 @returns results, empty when there is no catalog

 @throws RustCatalogError when the catalog is invalid

 @example
 ```ts
 await tightenRustCatalog({ root: process.cwd(), dryRun: true, });
 ```
 */
export async function tightenRustCatalog({
  root,
  dryRun,
}: {
  readonly root: string;
  readonly dryRun: boolean
},): Promise<readonly CargoTightenResult[]> {
  /**
   Catalog path.
   */
  const catalogPath = join(
    root,
    RUST_CATALOG_FILENAME,
  );
  /**
   Catalog text and document; absent when the repository has no catalog.
   */
  const catalogSource = await readOptionalFile(catalogPath,);
  if (catalogSource === ABSENT_FILE) {
    console.info(`\nNo ${RUST_CATALOG_FILENAME}; skipping the Cargo catalog.`,);
    return [];
  }
  /**
   Validated catalog.
   */
  const document = parseRustCatalog({
    source: catalogSource,
    sourcePath: catalogPath,
  },);
  /**
   Every first-party manifest path.
   */
  const manifests = (await Promise.all(MANIFEST_GLOBS.map(async function collect(pattern,): Promise<readonly string[]> {
    return await Array.fromAsync(glob(
      pattern,
      { cwd: root, },
    ),);
  },),)).flat()
    .filter(function isFirstParty(path: string,): boolean {
      return (!path.includes('/target/',)) && (!path.includes('/node_modules/',));
    },)
    .toSorted();
  /**
   Usage of every crate with a lockfile.
   */
  const crates = (await Promise.all(manifests.map(async function usage(manifestPath: string,): Promise<readonly CrateUsage[]> {
    return await readCrateUsage({
      root,
      manifestPath,
    },);
  },),)).flat();

  console.info(`\n${RUST_CATALOG_FILENAME} (${String(crates.length,)} crates with a Cargo.lock):`,);
  /**
   Planned results.
   */
  const results = planCargoTighten({
    document,
    crates,
  },);
  for (const result of results)
    printResult(result,);
  /**
   Entries that would change.
   */
  const tightened = results.filter(function isTight(result: CargoTightenResult,): boolean {
    return result.status === 'TIGHT';
  },);
  if (tightened.length === 0)
    console.info('No Cargo catalog floors to tighten.',);
  else if (dryRun)
    console.info(`Dry run: ${String(tightened.length,)} Cargo catalog floors would be tightened.`,);
  else {
    await writeFile(
      catalogPath,
      rewriteCatalogFloors({
        source: catalogSource,
        results,
      },),
    );
    console.info(`Wrote ${String(tightened.length,)} tightened floors to ${RUST_CATALOG_FILENAME}; run \`mise run sync:files\` to update the crates.`,);
  }
  return results;
}

//endregion Run
