/**
 Unit tests for the Cargo source of catalog-tighten.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { parseRustCatalog, } from '@monochromatic-dev/module-rust-catalog/ts';

import { directDependencyVersions, } from './lockfile.ts';
import {
  type CrateUsage,
  planCargoTighten,
  rewriteCatalogFloors,
} from './tighten.ts';

/**
 Catalog fixture with every entry kind.
 */
const CATALOG = [
  '# Floors.',
  '[catalog]',
  'anyhow = ">=1"',
  'serde = ">=1.0.229"',
  'renamed = { version = ">=0.2", package = "real-name" }',
  'unused = ">=3"',
  'helix-core = { git = "https://x", rev = "abc" }',
  '',
  '[catalogs.pinned]',
  'salsa = "=0.27.2"',
  '',
].join('\n',);

/**
 Lockfile fixture where `syn` appears twice and the root names its version.
 */
const LOCK = [
  'version = 4',
  '',
  '[[package]]',
  'name = "app"',
  'version = "0.1.0"',
  'dependencies = ["anyhow", "syn 2.0.110", "local"]',
  '',
  '[[package]]',
  'name = "anyhow"',
  'version = "1.0.104"',
  'source = "registry+https://github.com/rust-lang/crates.io-index"',
  '',
  '[[package]]',
  'name = "syn"',
  'version = "1.0.109"',
  'source = "registry+https://github.com/rust-lang/crates.io-index"',
  '',
  '[[package]]',
  'name = "syn"',
  'version = "2.0.110"',
  'source = "registry+https://github.com/rust-lang/crates.io-index"',
  '',
  '[[package]]',
  'name = "local"',
  'version = "0.1.0"',
  '',
].join('\n',);

await describe({
  name: 'cargo catalog tightening',
  children: [
    it({
      name: directDependencyVersions.name,
      fn: async () => {
        /**
         Direct versions of the fixture's root package.
         */
        const versions = directDependencyVersions({ source: LOCK, packageName: 'app', },);
        expect(versions.get('anyhow',),).toEqual(['1.0.104',],);
        expect(versions.get('syn',),).toEqual(['2.0.110',],);
        expect(versions.has('local',),).toBe(false,);
        expect(directDependencyVersions({ source: LOCK, packageName: 'missing', },).size,).toBe(0,);
      },
    },),

    it({
      name: 'raises floors to the highest locked release across consuming crates and skips the rest',
      fn: async () => {
        /**
         Two crates locking different anyhow releases, one locking a renamed package.
         */
        const crates: readonly CrateUsage[] = [
          { crateDir: 'a', uses: new Set(['anyhow', 'serde', 'renamed',],), lockVersions: new Map([['anyhow', ['1.0.102',],], ['serde', ['1.0.229',],], ['real-name', ['0.2.7',],],],), },
          { crateDir: 'b', uses: new Set(['anyhow',],), lockVersions: new Map([['anyhow', ['1.0.104',],], ['unused', ['3.9.0',],],],), },
          { crateDir: 'c', uses: new Set(['anyhow',],), lockVersions: new Map([['anyhow', ['2.0.0-rc.1',],],],), },
        ];
        /**
         Planned results by name.
         */
        const results = planCargoTighten({ document: parseRustCatalog({ source: CATALOG, sourcePath: 'rust-catalog.toml', },), crates, },);
        expect(results.map(function toStatus(result,): string {
          return `${result.status} ${result.name} ${result.newRequirement}`;
        },),).toEqual([
          'TIGHT anyhow >=1.0.104',
          'OK serde >=1.0.229',
          'TIGHT renamed >=0.2.7',
          'MISS unused ',
          'SKIP helix-core ',
          'SKIP salsa ',
        ],);
        /**
         Catalog after writing the tightened floors.
         */
        const rewritten = rewriteCatalogFloors({ source: CATALOG, results, },);
        expect(rewritten,).toBe(CATALOG.replace('anyhow = ">=1"', 'anyhow = ">=1.0.104"',).replace('version = ">=0.2"', 'version = ">=0.2.7"',),);
      },
    },),

    it({
      name: 'never rewrites a same-named entry outside [catalog]',
      fn: async () => {
        /**
         Catalog where a named catalog repeats a tightened name.
         */
        const source = ['[catalog]', 'anyhow = ">=1"', '', '[catalogs.legacy]', 'anyhow = "1"', '',].join('\n',);
        /**
         One tightened result.
         */
        const rewritten = rewriteCatalogFloors({
          source,
          results: [{ name: 'anyhow', status: 'TIGHT', oldRequirement: '>=1', newRequirement: '>=1.0.104', detail: '', },],
        },);
        expect(rewritten,).toBe(source.replace('anyhow = ">=1"', 'anyhow = ">=1.0.104"',),);
      },
    },),
  ],
},);
