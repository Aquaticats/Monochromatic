/**
 Unit tests for the pure Rust catalog sync.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  RustCatalogSyncError,
  syncRustCatalog,
} from './sync.ts';
import type {
  ManifestFile,
  SyncResult,
} from './types.ts';

//region Fixtures

/**
 Joins lines into file text with a trailing newline.

 @param lines - file lines

 @returns text

 @example
 ```ts
 text(['[package]',]); // '[package]\n'
 ```
 */
function text(lines: readonly string[],): string {
  return `${lines.join('\n',)}\n`;
}

/**
 Builds a crate manifest fixture.

 @param dir - crate directory

 @param body - lines after `[package]`

 @param version - package version

 @returns manifest file

 @example
 ```ts
 crate({ dir: 'package/cli/a', body: ['[dependencies]', 'anyhow = "1"',], });
 ```
 */
function crate({ dir, body, version = '0.1.0', }: { readonly dir: string; readonly body: readonly string[]; readonly version?: string; },): ManifestFile {
  /**
   Package name from the directory's last segment.
   */
  const name = dir.split('/',).at(-1,) ?? 'x';
  return { path: `${dir}/Cargo.toml`, source: text(['[package]', `name = "${name}"`, `version = "${version}"`, '', ...body, '', '[workspace]',],), };
}

/**
 Runs the sync on fixtures.

 @param catalog - catalog lines

 @param manifests - manifest fixtures

 @returns sync result

 @example
 ```ts
 run({ catalog: ['[catalog]',], manifests: [], });
 ```
 */
function run({ catalog, manifests, }: { readonly catalog: readonly string[]; readonly manifests: readonly ManifestFile[]; },): SyncResult {
  return syncRustCatalog({ catalogPath: 'rust-catalog.toml', catalogSource: text(catalog,), manifests, },);
}

/**
 Runs the sync expecting failure and returns each issue as `file location`.

 @param catalog - catalog lines

 @param manifests - manifest fixtures

 @returns issue labels in order

 @throws Error when the sync succeeds or fails differently

 @example
 ```ts
 issues({ catalog: ['[catalog]',], manifests, });
 ```
 */
function issues({ catalog, manifests, }: { readonly catalog: readonly string[]; readonly manifests: readonly ManifestFile[]; },): readonly string[] {
  try {
    run({ catalog, manifests, },);
  } catch (error) {
    if (error instanceof RustCatalogSyncError) {
      return error.issues.map(function toLabel(issue,): string {
        return `${issue.file} ${issue.location}`;
      },);
    }
    throw error;
  }
  throw new Error('expected a RustCatalogSyncError',);
}

/**
 New manifest text for a crate directory.

 @param result - sync result

 @param dir - crate directory

 @returns manifest text

 @throws Error when the result has no such manifest

 @example
 ```ts
 manifestText({ result, dir: 'package/cli/a', });
 ```
 */
function manifestText({ result, dir, }: { readonly result: SyncResult; readonly dir: string; },): string {
  /**
   Manifest text.
   */
  const found = result.manifests.get(`${dir}/Cargo.toml`,);
  if (found === undefined)
    throw new Error(`no manifest for ${dir}`,);
  return found;
}

//endregion Fixtures

await describe({
  name: syncRustCatalog.name,
  children: [
    //region Adoption

    it({
      name: 'adopts literals across crates into one floor, keeping use-site features and comments',
      fn: async () => {
        /**
         Sync over two crates with differing literal precision.
         */
        const result = run({
          catalog: ['[catalog]',],
          manifests: [
            crate({ dir: 'package/cli/a', body: ['[dependencies]', '# Error type.', 'anyhow = "1"', 'clap = { version = "4", features = ["derive"] }',], },),
            crate({ dir: 'package/cli/b', body: ['[target.\'cfg(unix)\'.dependencies]', 'anyhow = "1.0.104"',], },),
          ],
        },);
        expect(result.catalog,).toBe(text(['[catalog]', 'anyhow = ">=1.0.104"', 'clap = ">=4"',],),);
        expect(manifestText({ result, dir: 'package/cli/a', },),).toContain('# Error type.\nanyhow = { workspace = true }\nclap = { workspace = true, features = ["derive"] }\n',);
        expect(manifestText({ result, dir: 'package/cli/a', },),).toContain('[workspace.dependencies]\nanyhow = ">=1.0.104"\nclap = ">=4"\n',);
        expect(manifestText({ result, dir: 'package/cli/b', },),).toContain('[target.\'cfg(unix)\'.dependencies]\nanyhow = { workspace = true }\n',);
        expect(result.affectedCrates,).toEqual(['package/cli/a', 'package/cli/b',],);
      },
    },),

    it({
      name: 'is idempotent: a second sync over its own output changes nothing',
      fn: async () => {
        /**
         First pass output.
         */
        const first = run({
          catalog: ['[catalog]',],
          manifests: [crate({ dir: 'package/cli/a', body: ['[dependencies]', 'anyhow = "1"',], },),],
        },);
        /**
         Second pass over the first pass's files.
         */
        const second = syncRustCatalog({
          catalogPath: 'rust-catalog.toml',
          catalogSource: first.catalog,
          manifests: [...first.manifests.entries(),].map(function toFile([path, source,],): ManifestFile {
            return { path, source, };
          },),
        },);
        expect(second.catalog,).toBe(first.catalog,);
        expect([...second.manifests.values(),],).toEqual([...first.manifests.values(),],);
        expect(second.affectedCrates,).toEqual([],);
      },
    },),

    it({
      name: 'raises an existing floor when a literal needs a newer release',
      fn: async () => {
        /**
         Sync with an existing lower floor.
         */
        const result = run({
          catalog: ['[catalog]', 'zbus = ">=5"',],
          manifests: [crate({ dir: 'package/cli/a', body: ['[dependencies]', 'zbus = "5.19.0"',], },),],
        },);
        expect(result.catalog,).toBe(text(['[catalog]', 'zbus = ">=5.19.0"',],),);
      },
    },),

    it({
      name: 'adopts git pins and rewrites section-form entries in place',
      fn: async () => {
        /**
         Sync with a git literal and a section-form literal.
         */
        const result = run({
          catalog: ['[catalog]',],
          manifests: [crate({
            dir: 'package/cli/a',
            body: [
              '[dependencies]',
              'helix-core = { git = "https://github.com/helix-editor/helix", rev = "abc" }',
              '',
              '[dependencies.serde]',
              'version = "1"',
              'features = ["derive"]',
            ],
          },),],
        },);
        expect(result.catalog,).toContain('helix-core = { git = "https://github.com/helix-editor/helix", rev = "abc" }\n',);
        expect(manifestText({ result, dir: 'package/cli/a', },),).toContain('[dependencies.serde]\nworkspace = true\nfeatures = ["derive"]\n',);
      },
    },),

    it({
      name: 'resolves named-catalog selections for exact pins',
      fn: async () => {
        /**
         Sync where the crate selects a pinned entry.
         */
        const result = run({
          catalog: ['[catalog]', '', '[catalogs.pinned]', 'smithay = "=0.7.0"',],
          manifests: [crate({
            dir: 'package/cli/a',
            body: [
              '[package.metadata.rust-catalog.catalogs]',
              'smithay = "pinned"',
              '',
              '[dependencies]',
              'smithay = { version = "=0.7.0", default-features = false }',
            ],
          },),],
        },);
        expect(manifestText({ result, dir: 'package/cli/a', },),).toContain('smithay = { workspace = true, default-features = false }',);
        expect(manifestText({ result, dir: 'package/cli/a', },),).toContain('[workspace.dependencies]\nsmithay = "=0.7.0"\n',);
      },
    },),

    //endregion Adoption

    //region Internal crates

    it({
      name: 'writes the exact target version on internal path dependencies',
      fn: async () => {
        /**
         Sync with one outdated and one missing path version.
         */
        const result = run({
          catalog: ['[catalog]',],
          manifests: [
            crate({ dir: 'package/rust-module/lib', body: [], version: '0.1.1', },),
            crate({ dir: 'package/cli/a', body: ['[dependencies]', 'lib = { path = "../../rust-module/lib", version = "0.1.0" }',], },),
            crate({ dir: 'package/cli/b', body: ['[dependencies.lib]', 'path = "../../rust-module/lib"', 'features = ["x"]',], },),
          ],
        },);
        expect(manifestText({ result, dir: 'package/cli/a', },),).toContain('lib = { path = "../../rust-module/lib", version = "=0.1.1" }',);
        expect(manifestText({ result, dir: 'package/cli/b', },),).toContain('features = ["x"]\nversion = "=0.1.1"\n',);
        expect(result.affectedCrates,).toEqual([],);
      },
    },),

    //endregion Internal crates

    //region Pruning

    it({
      name: 'prunes unused entries with their comments when catalogPrune is on, and warns when it is off',
      fn: async () => {
        /**
         Catalog with one used and one unused entry.
         */
        const catalog = ['[catalog]', 'anyhow = ">=1"', '# Rationale for gone.', 'gone = ">=1"',];
        /**
         One crate using anyhow.
         */
        const manifests = [crate({ dir: 'package/cli/a', body: ['[dependencies]', 'anyhow = { workspace = true }',], },),];
        expect(run({ catalog: ['catalogPrune = true', ...catalog,], manifests, },).catalog,).toBe(text(['catalogPrune = true', '[catalog]', 'anyhow = ">=1"',],),);
        /**
         Result with pruning off.
         */
        const kept = run({ catalog, manifests, },);
        expect(kept.catalog,).toBe(text(catalog,),);
        expect(kept.warnings,).toEqual(['rust-catalog.toml: catalog.gone is used by no crate; set catalogPrune = true to remove unused entries on sync',],);
      },
    },),

    //endregion Pruning

    //region Issues

    it({
      name: 'refuses exact pins without a selection and incompatible majors',
      fn: async () => {
        expect(issues({
          catalog: ['[catalog]',],
          manifests: [
            crate({ dir: 'package/cli/a', body: ['[dev-dependencies]', 'i-slint-backend-testing = "=1.18.1"', 'windows = "0.58"',], },),
            crate({ dir: 'package/cli/b', body: ['[dependencies]', 'windows = "0.62"',], },),
          ],
        },),).toEqual([
          'package/cli/a/Cargo.toml dev-dependencies.i-slint-backend-testing',
          'package/cli/a/Cargo.toml dev-dependencies.windows',
        ],);
      },
    },),

    it({
      name: 'reports missing entries, bad paths, stale selections, and unsupported forms together',
      fn: async () => {
        expect(issues({
          catalog: ['[catalog]', '', '[catalogs.pinned]', 'smithay = "=0.7.0"',],
          manifests: [crate({
            dir: 'package/cli/a',
            body: [
              '[package.metadata.rust-catalog.catalogs]',
              'smithay = "pinned"',
              'tokio = "missing"',
              '',
              '[dependencies]',
              'anyhow = { workspace = true }',
              'outside = { path = "../../../../elsewhere" }',
              'pinned-dep = { workspace = true, version = "1" }',
              'other-registry = { version = "1", registry = "corp" }',
              'unpinned = { git = "https://example.com/u" }',
              'tokio = { workspace = true }',
            ],
          },),],
        },),).toEqual([
          'package/cli/a/Cargo.toml dependencies.pinned-dep',
          'package/cli/a/Cargo.toml dependencies.other-registry',
          'package/cli/a/Cargo.toml dependencies.unpinned',
          'package/cli/a/Cargo.toml dependencies.anyhow',
          'package/cli/a/Cargo.toml dependencies.outside',
          'package/cli/a/Cargo.toml dependencies.tokio',
          'package/cli/a/Cargo.toml package.metadata.rust-catalog.catalogs.smithay',
        ],);
      },
    },),

    it({
      name: 'rejects git pins that disagree across crates',
      fn: async () => {
        expect(issues({
          catalog: ['[catalog]',],
          manifests: [
            crate({ dir: 'package/cli/a', body: ['[dependencies]', 'h = { git = "https://x", rev = "1" }',], },),
            crate({ dir: 'package/cli/b', body: ['[dependencies]', 'h = { git = "https://x", rev = "2" }',], },),
          ],
        },),).toEqual(['package/cli/a/Cargo.toml h',],);
      },
    },),

    it({
      name: 'rejects a hand-written inline workspace.dependencies',
      fn: async () => {
        expect(issues({
          catalog: ['[catalog]',],
          manifests: [{
            path: 'package/cli/a/Cargo.toml',
            source: text(['[package]', 'name = "a"', 'version = "0.1.0"', '', '[workspace]', 'dependencies = { anyhow = "1" }',],),
          },],
        },),).toEqual(['package/cli/a/Cargo.toml workspace.dependencies',],);
      },
    },),

    //endregion Issues
  ],
},);
