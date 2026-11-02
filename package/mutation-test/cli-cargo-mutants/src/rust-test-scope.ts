/**
 The tests that can reach a mutated source file, as a nextest filterset.

 A test can only exercise a module that its own module, or a module that module imports, imports in turn. So the tests
 that reach a mutated file are the tests of its module and of every module that imports it directly or through other
 modules: the closure of importers over the crate's `use` graph. Pass 1 of a two-pass mutation campaign runs those
 tests first; a mutant they all miss may still be caught by a test the graph cannot see (a binary-level test, a
 re-export, a macro), so pass 2 runs the whole suite on the survivors.
 */
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';

import {
  alphanumericCharacters,
  usesOnly,
} from '@monochromatic-dev/shell-argument-core/ts/character-set.ts';
import { CargoMutantsToolingError } from './error.ts';
import { everyModule } from './rust-module-scan.ts';
import {
  parseCargoTargets,
  type ModuleEntry,
  scanTargetModules,
  type TargetModules,
} from './rust-crate-modules.ts';

/**
 The tests found for a mutated file.
 */
export type TestScope = {
  /**
   Top-level library modules whose tests are in scope (the mutated file's own module and its closure of importers).
   */
  readonly libraryModules: readonly string[];
  /**
   Top-level modules of the library, in scope or not.
   */
  readonly libraryModuleCount: number;
  /**
   Test targets in scope: those whose files refer to a module in scope, leaving out the excluded targets.
   */
  readonly testTargets: readonly string[];
  /**
   Top-level modules the mutated files belong to.
   */
  readonly mutatedModules: readonly string[];
};

/**
 Group the modules of a target by their top-level module.

 @param modules - the target's modules

 @returns the top-level module name of each module below the root, the root excluded

 @example
 ```ts
 topLevelNames([{ path: ['a', 'tests'], file: '/c/a_tests.rs' }]);
 ```
 */
function topLevelNames(modules: readonly ModuleEntry[]): ReadonlySet<string> {
  return new Set(modules.flatMap(function topLevel(entry) {
    return entry.path
      .slice(
      0,
      1,
      );
  }));
}

/**
 Work out which top-level modules a name in a file can mean.

 A module's own name means that module. {@link everyModule} means all of them. Any other name may be an item the crate
 root re-exports, and means the modules it re-exports from; because a glob re-export can supply any item, the modules the
 root re-exports with a glob are meant too.

 @param name - a name a file refers to

 @param names - the top-level modules

 @param reexports - the root's re-exports

 @returns the top-level modules the name can mean

 @example
 ```ts
 modulesNamedBy({ name: 'value', names: new Set(['a']), reexports: new Map([['value', ['a']]]) });
 ```
 */
function modulesNamedBy({
  name,
  names,
  reexports,
}: {
  readonly name: string;
  readonly names: ReadonlySet<string>;
  readonly reexports: ReadonlyMap<string, readonly string[]>;
}): readonly string[] {
  if (name === everyModule)
    return [...names];
  if (names.has(name))
    return [name];
  return [
    ...reexports.get(name) ?? [],
    ...reexports.get(everyModule) ?? []
  ].filter(function isModule(module) {
    return names.has(module);
  });
}

/**
 Build the importers of each top-level module: the top-level modules that refer to it.

 @param target - the library's modules and references

 @returns for each top-level module, the other top-level modules that refer to it

 @example
 ```ts
 importersOf(await scanTargetModules({ root, referenceRoots }));
 ```
 */
function importersOf(target: TargetModules): ReadonlyMap<string, ReadonlySet<string>> {
  /**
   Top-level module names.
   */
  const names = topLevelNames(target.modules);
  /**
   Importers per module.
   */
  const importers = new Map<string, Set<string>>();
  for (const entry of target.modules) {
    /**
     The top-level module this entry belongs to.
     */
    const [owner] = entry.path;
    if (owner === undefined)
      continue;
    for (const name of target.references
      .get(entry.file)
      ?? []) {
      for (const module of modulesNamedBy({
        name,
        names,
        reexports: target.reexports,
      })) {
        if (module !== owner) {
          importers.set(
            module,
            new Set([
              ...importers.get(module) ?? [],
              owner
            ]),
          );
        }
      }
    }
  }
  return importers;
}

/**
 Collect the modules that import a set of modules directly or through others.

 @param importers - importers per module

 @param start - the mutated modules

 @returns the start modules and every module that reaches one of them, in discovery order

 @example
 ```ts
 reachingModules({ importers, start: ['repo_read_query'] });
 ```
 */
export function reachingModules({
  importers,
  start,
}: {
  readonly importers: ReadonlyMap<string, ReadonlySet<string>>;
  readonly start: readonly string[];
}): readonly string[] {
  /**
   Modules found, which doubles as the queue.
   */
  const found = new Set(start);
  for (const name of found) {
    for (const importer of importers.get(name) ?? [])
      found.add(importer);
  }
  return [...found];
}

/**
 Find the tests that can reach a mutated file.

 @param crateDirectory - the crate's directory, which holds `Cargo.toml`

 @param mutatedFiles - the mutated source files, as paths relative to the crate directory (or absolute)

 @param excludedTestTargets - test targets that never take part in pass 1, such as a slow binary-level suite

 @returns the scope as the only element, or no element when a mutated file is not a module of the library

 @throws CargoMutantsToolingError when `Cargo.toml` or a source file cannot be read

 @example
 ```ts
 await testScope({ crateDirectory: '/crate', mutatedFiles: ['src/native/a.rs'], excludedTestTargets: ['native_binary'] });
 ```
 */
export async function testScope({
  crateDirectory,
  mutatedFiles,
  excludedTestTargets,
}: {
  readonly crateDirectory: string;
  readonly mutatedFiles: readonly string[];
  readonly excludedTestTargets: readonly string[];
}): Promise<readonly TestScope[]> {
  /**
   Targets of the crate.
   */
  const targets = parseCargoTargets({
    manifest: await readFile(
      resolve(
        crateDirectory,
        'Cargo.toml',
      ),
      'utf8',
    ),
    directory: crateDirectory,
  });
  /**
   Modules of the library and what they refer to.
   */
  const library = await scanTargetModules({
    root: targets.libraryRoot,
    referenceRoots: new Set([
      'crate',
      'super'
    ]),
  });
  /**
   Absolute paths of the mutated files.
   */
  const mutatedPaths = new Set(mutatedFiles.map(function absolute(file) {
    return resolve(
      crateDirectory,
      file,
    );
  }));
  /**
   Top-level modules the mutated files belong to.
   */
  const mutatedModules = [...new Set(library.modules
    .flatMap(function owningModule(entry) {
      return mutatedPaths.has(entry.file)
        ? entry.path
          .slice(
          0,
          1,
          )
        : [];
    }))];
  if (mutatedModules.length === 0)
    return [];
  /**
   Top-level modules that reach the mutated ones.
   */
  const libraryModules = reachingModules({
    importers: importersOf(library),
    start: mutatedModules,
  });
  /**
   The test targets that refer to a module in scope.
   */
  const testTargets = (await Promise.all(targets.testTargets
    .filter(function takesPart(target) {
      return !excludedTestTargets.includes(target.name);
    })
    .map(async function referencesScope(target) {
      /**
       Modules of the test target and what they refer to.
       */
      const scanned = await scanTargetModules({
        root: target.root,
        referenceRoots: new Set([targets.libraryName]),
      });
      return [...scanned.references
        .values()].some(function reachesScope(names) {
          return libraryModules.some(function isReferenced(name) {
            return names.has(name);
          });
        }) ? [target.name] : [];
    }))).flat();
  return [{
    libraryModules: libraryModules.toSorted(),
    libraryModuleCount: topLevelNames(library.modules)
      .size,
    testTargets,
    mutatedModules,
  }];
}

/**
 Build the nextest filterset that selects a scope.

 Library tests are selected by anchored regular expression on the module path at the start of the test name, so
 `repo_read_layout` selects `repo_read_layout::tests::a` and `repo_read_layout::worktree_tests::b` and not
 `other::repo_read_layout::c`. Test targets are selected whole by name.

 @param scope - the scope found for the mutated files

 @returns the filterset text, or the empty text when the scope selects no module and no target

 @throws CargoMutantsToolingError when a name holds a character that the filterset syntax would read differently

 @example
 ```ts
 scopeFilterset({ libraryModules: ['a', 'b'], libraryModuleCount: 9, testTargets: ['x'], mutatedModules: ['a'] });
 ```
 */
export function scopeFilterset(scope: TestScope): string {
  for (const name of [
    ...scope.libraryModules,
    ...scope.testTargets
  ]) {
    if ((name === '') || (!usesOnly({
      text: name,
      allowed: `${alphanumericCharacters}_`,
    })))
      throw new CargoMutantsToolingError(`The name ${name} is not a plain identifier, so it cannot go into a nextest filterset.`);
  }
  return [
    ...(scope.libraryModules
      .length
      === 0 ? [] : [`(kind(lib) & test(/^(?:${scope.libraryModules
        .join('|')})::/))`]),
    ...scope.testTargets
      .map(function targetClause(name) {
        return `binary(=${name})`;
      }),
  ].join(' | ');
}
