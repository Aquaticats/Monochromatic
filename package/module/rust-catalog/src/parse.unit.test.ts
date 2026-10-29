/**
 Unit tests for the built Rust dependency catalog parser.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  parseRustCatalog,
  RustCatalogError,
} from '../dist/final/node/index.mjs';

//region Helpers

/**
 Parses `lines` joined as a catalog file.

 @param lines - TOML source lines

 @returns parsed document

 @example
 ```ts
 parseLines(['[catalog]', 'anyhow = ">=1"',]);
 ```
 */
function parseLines(lines: readonly string[],): ReturnType<typeof parseRustCatalog> {
  return parseRustCatalog({ source: lines.join('\n',), sourcePath: 'rust-catalog.toml', },);
}

/**
 Parses `lines` expecting a catalog error and returns its issue locations.

 @param lines - TOML source lines that must fail validation

 @returns locations of every reported issue, in order

 @throws {Error} when parsing succeeds or fails with another error type

 @example
 ```ts
 issueLocations(['[catalog]', 'tokio = { version = ">=1", features = ["rt"] }',]); // ['catalog.tokio']
 ```
 */
function issueLocations(lines: readonly string[],): readonly string[] {
  try {
    parseLines(lines,);
  } catch (error) {
    if (error instanceof RustCatalogError)
      return error.issues.map(function toLocation(issue,): string {
        return issue.location;
      },);
    throw error;
  }
  throw new Error('expected a RustCatalogError, but parsing succeeded',);
}

//endregion Helpers

await describe({
  name: parseRustCatalog.name,
  children: [
    //region Accepted shapes

    it({
      name: 'reads default floors, renames, and git pins in source order',
      fn: async () => {
        /**
         Document with every accepted default-catalog entry form.
         */
        const document = parseLines([
          '[catalog]',
          'anyhow = ">=1.0.104"',
          'tracing-log = { version = ">=0.2", package = "tracing-log" }',
          'helix-core = { git = "https://github.com/helix-editor/helix", rev = "ba40e547426b0f9896c8bdc699a4ab11f2b37dbc" }',
        ],);
        expect([...document.catalog.keys(),],).toEqual(['anyhow', 'tracing-log', 'helix-core',],);
        expect(document.catalog.get('anyhow',),).toEqual({ kind: 'registry', version: '>=1.0.104', },);
        expect(document.catalog.get('tracing-log',),).toEqual({
          kind: 'registry',
          version: '>=0.2',
          package: 'tracing-log',
        },);
        expect(document.catalog.get('helix-core',),).toEqual({
          kind: 'git',
          git: 'https://github.com/helix-editor/helix',
          rev: 'ba40e547426b0f9896c8bdc699a4ab11f2b37dbc',
        },);
      },
    },),

    it({
      name: 'reads bounded named catalogs',
      fn: async () => {
        /**
         Document with caret, tilde, and exact named-catalog entries.
         */
        const document = parseLines([
          '[catalogs.windows-legacy]',
          'windows = "0.58"',
          'a = "^1.2"',
          'b = "~1.2.3"',
          'c = "=1.18.1"',
        ],);
        expect(document.catalogs.get('windows-legacy',)?.get('windows',),).toEqual({ kind: 'registry', version: '0.58', },);
        expect(document.catalogs.get('windows-legacy',)?.get('c',),).toEqual({ kind: 'registry', version: '=1.18.1', },);
      },
    },),

    it({
      name: 'applies pnpm defaults when settings are absent',
      fn: async () => {
        /**
         Settings of a file that declares none.
         */
        const { settings, } = parseLines(['[catalog]',],);
        expect(settings,).toEqual({
          minimumReleaseAge: 1440,
          minimumReleaseAgeExclude: [],
          minimumReleaseAgeExcludePrune: false,
          minimumReleaseAgeIgnoreMissingTime: true,
          minimumReleaseAgeStrict: false,
          trustPolicy: 'off',
          trustPolicyExclude: [],
          trustPolicyExcludePrune: false,
        },);
      },
    },),

    it({
      name: 'defaults strict mode on when minimumReleaseAge is explicit and reads every setting',
      fn: async () => {
        /**
         Settings of a file that declares all of them.
         */
        const { settings, } = parseLines([
          'minimumReleaseAge = 1440',
          'minimumReleaseAgeExclude = ["ring", "zbus@5.19.0 || 5.19.1"]',
          'minimumReleaseAgeExcludePrune = true',
          'minimumReleaseAgeIgnoreMissingTime = false',
          'trustPolicy = "no-downgrade"',
          'trustPolicyExclude = ["clap@4.6.7"]',
          'trustPolicyExcludePrune = true',
          'trustPolicyIgnoreAfter = 525600',
          '[catalog]',
        ],);
        expect(settings,).toEqual({
          minimumReleaseAge: 1440,
          minimumReleaseAgeExclude: ['ring', 'zbus@5.19.0 || 5.19.1',],
          minimumReleaseAgeExcludePrune: true,
          minimumReleaseAgeIgnoreMissingTime: false,
          minimumReleaseAgeStrict: true,
          trustPolicy: 'no-downgrade',
          trustPolicyExclude: ['clap@4.6.7',],
          trustPolicyExcludePrune: true,
          trustPolicyIgnoreAfter: 525600,
        },);
      },
    },),

    it({
      name: 'honours an explicit minimumReleaseAgeStrict = false',
      fn: async () => {
        /**
         Settings that turn strict mode off explicitly.
         */
        const { settings, } = parseLines(['minimumReleaseAge = 60', 'minimumReleaseAgeStrict = false', '[catalog]',],);
        expect(settings.minimumReleaseAgeStrict,).toBe(false,);
      },
    },),

    //endregion Accepted shapes

    //region Rejected entries

    it({
      name: 'rejects use-site keys in catalog entries',
      fn: async () => {
        expect(issueLocations([
          '[catalog]',
          'tokio = { version = ">=1", features = ["rt"] }',
          'turso = { version = ">=0.8", default-features = false, optional = true }',
          'own = { path = "../own" }',
        ],),).toEqual([
          'catalog.tokio',
          'catalog.turso',
          'catalog.turso',
          'catalog.own',
          'catalog.own',
        ],);
      },
    },),

    it({
      name: 'rejects bounded versions in the default catalog and floors in named catalogs',
      fn: async () => {
        expect(issueLocations([
          '[catalog]',
          'anyhow = "1.0.104"',
          'serde = { version = "^1" }',
          '[catalogs.legacy]',
          'windows = ">=0.58"',
        ],),).toEqual(['catalog.anyhow', 'catalog.serde', 'catalogs.legacy.windows',],);
      },
    },),

    it({
      name: 'rejects malformed git entries',
      fn: async () => {
        expect(issueLocations([
          '[catalog]',
          'a = { git = "https://example.com/a" }',
          'b = { git = "https://example.com/b", rev = "abc", version = ">=1" }',
          'c = { rev = "abc", version = ">=1" }',
          'd = { git = "https://example.com/d", branch = "main", rev = "abc" }',
        ],),).toEqual(['catalog.a', 'catalog.b', 'catalog.c', 'catalog.d',],);
      },
    },),

    it({
      name: 'rejects adversarial names, unknown keys, and non-table values',
      fn: async () => {
        expect(issueLocations([
          '[catalog]',
          '"__proto__" = ">=1"',
          '"../escape" = ">=1"',
          '"a\\nb" = ">=1"',
          '"9lives" = ">=1"',
          'x = { version = ">=1", registry = "other" }',
          'y = 3',
          'z = { version = ">=1", package = "bad name" }',
        ],),).toEqual([
          'catalog.__proto__',
          'catalog.../escape',
          'catalog.a\nb',
          'catalog.9lives',
          'catalog.x',
          'catalog.y',
          'catalog.z',
        ],);
      },
    },),

    it({
      name: 'rejects unknown top-level keys, catalogs.default, and malformed tables',
      fn: async () => {
        expect(issueLocations(['dependencies = {}', 'catalog = 1', '[catalogs.default]', 'a = "1"',],),).toEqual([
          'dependencies',
          'catalog',
          'catalogs.default',
        ],);
        expect(issueLocations(['catalogs = 1',],),).toEqual(['catalogs',],);
      },
    },),

    it({
      name: 'rejects malformed settings',
      fn: async () => {
        expect(issueLocations([
          'minimumReleaseAge = -1',
          'minimumReleaseAgeExclude = "ring"',
          'minimumReleaseAgeStrict = "yes"',
          'trustPolicy = "trust-everyone"',
          'trustPolicyIgnoreAfter = 1.5',
          '[catalog]',
        ],),).toEqual([
          'minimumReleaseAge',
          'trustPolicyIgnoreAfter',
          'minimumReleaseAgeExclude',
          'minimumReleaseAgeStrict',
          'trustPolicy',
        ],);
      },
    },),

    it({
      name: 'names every issue, its remedy, and the decision doc in the message',
      fn: async () => {
        try {
          parseLines(['[catalog]', 'tokio = { version = ">=1", features = ["rt"] }',],);
        } catch (error) {
          expect(error,).toBeInstanceOf(RustCatalogError,);
          expect((error as Error).message,).toContain('catalog.tokio: declares `features`.',);
          expect((error as Error).message,).toContain('doc/decision/rust-dependency-catalog.md',);
          return;
        }
        throw new Error('expected a RustCatalogError',);
      },
    },),

    //endregion Rejected entries
  ],
},);
