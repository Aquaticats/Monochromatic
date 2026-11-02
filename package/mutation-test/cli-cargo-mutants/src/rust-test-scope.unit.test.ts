import {
  mkdir,
  mkdtempDisposable,
  writeFile,
} from 'node:fs/promises';
import { homedir } from 'node:os';
import {
  dirname,
  join,
} from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { CargoMutantsToolingError } from './error.ts';
import {
  parseCargoTargets,
  scanTargetModules,
} from './rust-crate-modules.ts';
import {
  reachingModules,
  scopeFilterset,
  testScope,
} from './rust-test-scope.ts';

/**
 Scratch parent of the fixture crates; never the system temporary directory, whose backing device is unknowable.
 */
const scratchParent = join(
  homedir(),
  'temp',
  'agent',
);

/**
 Manifest of the fixture crate: a library with another name than the package, and two test targets.
 */
const manifest = [
  '[package]',
  'name = "fixture-crate"',
  'version = "0.1.0"',
  '',
  '[lib]',
  'name = "fixture_lib"',
  'path = "src/native/lib.rs"',
  '',
  '[[test]]',
  'name = "slow_binary"',
  'path = "src/native/slow_binary_tests.rs"',
  '',
  '[[test]]',
  'name = "consumer"',
  'path = "src/native/consumer_tests.rs"',
  '',
].join('\n');

/**
 Sources of the fixture crate, shaped like the native wrapper: a flat library whose test files hang under their module.

 `target` is mutated in the tests. `direct` imports it, `indirect` imports `direct` only, `grouped` imports it through
 a use group, `unrelated` and `island` do not reach it, and `cycle_a` and `cycle_b` import each other and `target`.
 */
const sources: Readonly<Record<string, string>> = {
  'Cargo.toml': manifest,
  'src/native/lib.rs': [
    'pub mod target;',
    'pub mod direct;',
    'pub mod indirect;',
    'pub mod grouped;',
    'pub mod unrelated;',
    'pub mod island;',
    'pub mod cycle_a;',
    'pub mod cycle_b;',
    '#[cfg(test)]',
    '#[path = "sibling_tests.rs"]',
    'mod sibling_tests;',
  ].join('\n'),
  'src/native/target.rs': 'pub fn f() {}\n#[cfg(test)]\n#[path = "target_tests.rs"]\nmod tests;\n',
  'src/native/target_tests.rs': 'use super::*;\n',
  'src/native/direct.rs': 'use super::target::f;\npub fn g() { f() }\n#[cfg(test)]\n#[path = "direct_tests.rs"]\nmod tests;\n',
  'src/native/direct_tests.rs': 'use super::*;\n',
  'src/native/indirect.rs': 'use super::direct::g;\npub fn h() { g() }\n#[cfg(test)]\n#[path = "indirect_tests.rs"]\nmod tests;\n',
  'src/native/indirect_tests.rs': 'use super::*;\n',
  'src/native/grouped.rs': 'use crate::{unrelated::z, target};\npub fn i() { target::f() }\n',
  'src/native/unrelated.rs': 'pub fn z() {}\n',
  'src/native/island.rs': '// use crate::target;\npub fn j() { let s = "crate::target::f"; }\n',
  'src/native/cycle_a.rs': 'use super::{cycle_b, target};\n',
  'src/native/cycle_b.rs': 'use super::cycle_a::k;\n',
  'src/native/sibling_tests.rs': 'use crate::indirect::h;\n',
  'src/native/slow_binary_tests.rs': 'use fixture_lib::target::f;\n',
  'src/native/consumer_tests.rs': 'use fixture_lib::grouped::i;\n',
};

/**
 Write the fixture crate into a disposable directory.

 @param overrides - files that replace or add to the default sources, by path below the crate

 @returns the disposable directory, whose `path` is the crate directory; declare it with `await using`

 @example
 ```ts
 await using crate = await fixtureCrate({});
 ```
 */
async function fixtureCrate(overrides: Readonly<Record<string, string>>): Promise<AsyncDisposable & { readonly path: string }> {
  await mkdir(
    scratchParent,
    { recursive: true },
  );
  /**
   Fresh directory for this crate.
   */
  const crate = await mkdtempDisposable(join(
    scratchParent,
    'rust-scope-',
  ));
  await Promise.all(Object.entries({
    ...sources,
    ...overrides,
  })
    .map(async function writeSource([relative, text]) {
      /**
       Absolute path of the source file.
       */
      const file = join(
        crate.path,
        relative,
      );
      await mkdir(
        dirname(file),
        { recursive: true },
      );
      await writeFile(
        file,
        text,
      );
    }));
  return crate;
}

await describe({
  name: 'the tests that can reach a mutated file',
  children: [
    describe({
      name: parseCargoTargets.name,
      children: [
        it({
          name: 'reads the library name and root and the declared test targets, with their default paths',
          fn: async () => {
            expect(parseCargoTargets({
              manifest: `${manifest}\n[[test]]\nname = "defaulted"\n`,
              directory: '/crate',
            })).toEqual({
              libraryName: 'fixture_lib',
              libraryRoot: '/crate/src/native/lib.rs',
              testTargets: [
                {
                  name: 'slow_binary',
                  root: '/crate/src/native/slow_binary_tests.rs',
                },
                {
                  name: 'consumer',
                  root: '/crate/src/native/consumer_tests.rs',
                },
                {
                  name: 'defaulted',
                  root: '/crate/tests/defaulted.rs',
                },
              ],
            });
          },
        }),
        it({
          name: 'names the library after the package, with underscores, when the manifest has no lib table',
          fn: async () => {
            expect(parseCargoTargets({
              manifest: '[package]\nname = "some-crate"\n',
              directory: '/crate',
            })).toEqual({
              libraryName: 'some_crate',
              libraryRoot: '/crate/src/lib.rs',
              testTargets: [],
            });
          },
        }),
        it({
          name: 'refuses a manifest without a package name',
          fn: async () => {
            expect(() => parseCargoTargets({
              manifest: '[lib]\nname = "x"\n',
              directory: '/crate',
            })).toThrow(CargoMutantsToolingError);
          },
        }),
      ],
    }),
    describe({
      name: scanTargetModules.name,
      children: [
        it({
          name: 'gives each file the module path that declares it, including a test file loaded through a path attribute',
          fn: async () => {
            await using crate = await fixtureCrate({});
            /**
             The library's modules.
             */
            const scanned = await scanTargetModules({
              root: join(
                crate.path,
                'src/native/lib.rs',
              ),
              referenceRoots: new Set(['crate', 'super']),
            });
            expect(scanned.modules
              .map(function pathText(entry) {
                return entry.path.join('::');
              })
              .toSorted()).toContain('direct::tests');
            expect(scanned.modules
              .map(function pathText(entry) {
                return entry.path.join('::');
              })
              .toSorted()).toContain('sibling_tests');
          },
        }),
        it({
          name: 'follows a nested declaration to the file beside the owner when the owner is not a root',
          fn: async () => {
            await using crate = await fixtureCrate({
              'src/native/lib.rs': 'pub mod outer;\n',
              'src/native/outer.rs': 'pub mod inner;\n',
              'src/native/outer/inner.rs': 'pub fn x() {}\n',
            });
            /**
             The library's modules.
             */
            const scanned = await scanTargetModules({
              root: join(
                crate.path,
                'src/native/lib.rs',
              ),
              referenceRoots: new Set(['crate', 'super']),
            });
            expect(scanned.modules
              .map(function pathText(entry) {
                return entry.path.join('::');
              })).toContain('outer::inner');
          },
        }),
        it({
          name: 'leaves out a declared module whose file does not exist, such as one that cfg removes',
          fn: async () => {
            await using crate = await fixtureCrate({ 'src/native/lib.rs': 'pub mod target;\n#[cfg(windows)]\nmod missing;\n' });
            /**
             The library's modules.
             */
            const scanned = await scanTargetModules({
              root: join(
                crate.path,
                'src/native/lib.rs',
              ),
              referenceRoots: new Set(['crate', 'super']),
            });
            expect(scanned.modules
              .map(function pathText(entry) {
                return entry.path.join('::');
              })).not.toContain('missing');
          },
        }),
        it({
          name: 'stops at a declaration cycle instead of looping',
          fn: async () => {
            await using crate = await fixtureCrate({
              'src/native/lib.rs': 'pub mod loop_a;\n',
              'src/native/loop_a.rs': '#[path = "loop_a.rs"]\nmod again;\n',
            });
            await expect(scanTargetModules({
              root: join(
                crate.path,
                'src/native/lib.rs',
              ),
              referenceRoots: new Set(['crate', 'super']),
            })).rejects.toThrow(CargoMutantsToolingError);
          },
        }),
      ],
    }),
    describe({
      name: reachingModules.name,
      children: [
        it({
          name: 'follows importers of importers and stops at a cycle',
          fn: async () => {
            expect(reachingModules({
              importers: new Map([
                ['a', new Set(['b'])],
                ['b', new Set(['c', 'a'])],
                ['c', new Set(['b'])],
              ]),
              start: ['a'],
            })).toEqual([
              'a',
              'b',
              'c',
            ]);
          },
        }),
        it({
          name: 'keeps a module with no importer alone',
          fn: async () => {
            expect(reachingModules({
              importers: new Map(),
              start: ['x'],
            })).toEqual(['x']);
          },
        }),
      ],
    }),
    describe({
      name: testScope.name,
      children: [
        it({
          name: 'takes the module itself, its importers at every depth, and the cycle that imports it, and leaves out the rest',
          fn: async () => {
            await using crate = await fixtureCrate({});
            expect((await testScope({
              crateDirectory: crate.path,
              mutatedFiles: ['src/native/target.rs'],
              excludedTestTargets: ['slow_binary'],
            }))[0]).toMatchObject({
              libraryModules: [
                'cycle_a',
                'cycle_b',
                'direct',
                'grouped',
                'indirect',
                'sibling_tests',
                'target',
              ],
              mutatedModules: ['target'],
            });
          },
        }),
        it({
          name: 'counts the modules of the library, so a caller can say how much of it the scope covers',
          fn: async () => {
            await using crate = await fixtureCrate({});
            /**
             The scope of the mutated file.
             */
            const [scope] = await testScope({
              crateDirectory: crate.path,
              mutatedFiles: ['src/native/target.rs'],
              excludedTestTargets: [],
            });
            // The eight declared modules and the one test module declared at the root.
            expect(scope?.libraryModuleCount).toBe(9);
          },
        }),
        it({
          name: 'takes a test target that refers to a module in scope, and leaves out an excluded one and one that does not',
          fn: async () => {
            await using crate = await fixtureCrate({});
            expect((await testScope({
              crateDirectory: crate.path,
              mutatedFiles: ['src/native/target.rs'],
              excludedTestTargets: ['slow_binary'],
            }))[0]?.testTargets).toEqual(['consumer']);
            expect((await testScope({
              crateDirectory: crate.path,
              mutatedFiles: ['src/native/target.rs'],
              excludedTestTargets: [],
            }))[0]?.testTargets).toEqual(['slow_binary', 'consumer']);
            expect((await testScope({
              crateDirectory: crate.path,
              mutatedFiles: ['src/native/unrelated.rs'],
              excludedTestTargets: [],
            }))[0]?.testTargets).toEqual(['consumer']);
          },
        }),
        it({
          name: 'does not take a module that only mentions the mutated one in a comment or a string',
          fn: async () => {
            await using crate = await fixtureCrate({});
            expect((await testScope({
              crateDirectory: crate.path,
              mutatedFiles: ['src/native/target.rs'],
              excludedTestTargets: [],
            }))[0]?.libraryModules).not.toContain('island');
          },
        }),
        it({
          name: 'unions the scopes of several mutated files',
          fn: async () => {
            await using crate = await fixtureCrate({});
            expect((await testScope({
              crateDirectory: crate.path,
              mutatedFiles: ['src/native/unrelated.rs', 'src/native/island.rs'],
              excludedTestTargets: [],
            }))[0]?.libraryModules).toEqual([
              'grouped',
              'island',
              'unrelated',
            ]);
          },
        }),
        it({
          name: 'finds no scope for a file that is not a module of the library',
          fn: async () => {
            await using crate = await fixtureCrate({ 'src/native/stray.rs': 'pub fn s() {}\n' });
            expect(await testScope({
              crateDirectory: crate.path,
              mutatedFiles: ['src/native/stray.rs'],
              excludedTestTargets: [],
            })).toEqual([]);
          },
        }),
      ],
    }),
    describe({
      name: scopeFilterset.name,
      children: [
        it({
          name: 'selects library tests by an anchored module-path expression and test targets by name',
          fn: async () => {
            expect(scopeFilterset({
              libraryModules: ['a', 'b_c'],
              libraryModuleCount: 9,
              testTargets: ['consumer'],
              mutatedModules: ['a'],
            })).toBe('(kind(lib) & test(/^(?:a|b_c)::/)) | binary(=consumer)');
          },
        }),
        it({
          name: 'selects only a target when no library module is in scope, and the empty text when neither is',
          fn: async () => {
            expect(scopeFilterset({
              libraryModules: [],
              libraryModuleCount: 9,
              testTargets: ['consumer'],
              mutatedModules: [],
            })).toBe('binary(=consumer)');
            expect(scopeFilterset({
              libraryModules: [],
              libraryModuleCount: 9,
              testTargets: [],
              mutatedModules: [],
            })).toBe('');
          },
        }),
        it({
          name: 'refuses a name that the filterset syntax could read as something else',
          fn: async () => {
            for (const name of ['a)|all()', 'a/b', 'a b', '', 'a::b']) {
              expect(() => scopeFilterset({
                libraryModules: [name],
                libraryModuleCount: 1,
                testTargets: [],
                mutatedModules: [],
              })).toThrow(CargoMutantsToolingError);
            }
          },
        }),
      ],
    }),
  ],
});
