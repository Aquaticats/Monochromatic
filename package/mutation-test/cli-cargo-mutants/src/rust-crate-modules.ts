/**
 The targets of a Rust crate and the module tree of each, read from `Cargo.toml` and the sources without running Cargo.
 */
import {
  access,
  readFile,
} from 'node:fs/promises';
import {
  basename,
  dirname,
  join,
  resolve,
} from 'node:path';

import { CargoMutantsToolingError } from './error.ts';
import { isMissing } from './fs-guards.ts';
import {
  type ModuleDeclaration,
  moduleDeclarations,
} from './rust-module-declarations.ts';
import {
  referencedNames,
  rootReexports,
} from './rust-module-scan.ts';
import { tokenizeRust } from './rust-tokens.ts';

/**
 One test target of a crate.
 */
export type TestTarget = {
  /**
   Name nextest shows after the package name, such as `native_binary`.
   */
  readonly name: string;
  /**
   Absolute path of the target's root source file.
   */
  readonly root: string;
};

/**
 The targets of a crate that tests can live in.
 */
export type CrateTargets = {
  /**
   Name other targets use to reach the library: the `[lib]` name, or the package name with hyphens turned to
   underscores.
   */
  readonly libraryName: string;
  /**
   Absolute path of the library's root source file.
   */
  readonly libraryRoot: string;
  readonly testTargets: readonly TestTarget[];
};

/**
 One module of a target and the file that holds it.
 */
export type ModuleEntry = {
  /**
   Module path below the target's root: empty for the root itself.
   */
  readonly path: readonly string[];
  /**
   Absolute path of the file.
   */
  readonly file: string;
};

/**
 A target's modules and what each of its files refers to.
 */
export type TargetModules = {
  readonly modules: readonly ModuleEntry[];
  /**
   For each file, the names that follow a root in its paths (see `referencedNames`).
   */
  readonly references: ReadonlyMap<string, ReadonlySet<string>>;
  /**
   For each item name the root module re-exports with `use`, the modules it may come from; `*` lists the modules the root
   re-exports with a glob.
   */
  readonly reexports: ReadonlyMap<string, readonly string[]>;
};

/**
 Deepest module nesting the scan follows before it decides the declarations loop.
 */
const maximumModuleDepth = 64;

/**
 Section of `Cargo.toml` being read and the keys read so far.
 */
type CargoSection = {
  readonly table: string;
  readonly name?: string;
  readonly path?: string;
};

/**
 Read a `name = "value"` or `path = "value"` line.

 @param line - one trimmed line of `Cargo.toml`

 @returns the key and the text between the first pair of quotes, or nothing for any other line

 @example
 ```ts
 quotedPair('name = "a-b"');
 ```
 */
function quotedPair(line: string): readonly string[] {
  /**
   Position of the equals sign.
   */
  const equals = line.indexOf('=');
  /**
   The key in front of it.
   */
  const key = line.slice(
    0,
    equals,
  )
    .trim();
  /**
   The value after it, which must open with a quote.
   */
  const value = line.slice(equals + 1)
    .trim();
  if ((equals === (-1)) || (![
    'name',
    'path'
  ].includes(key))
    || (!value.startsWith('"')))
    return [];
  /**
   Position of the closing quote.
   */
  const close = value.indexOf(
    '"',
    1,
  );
  return close === (-1) ? [] : [
    key,
    value.slice(
      1,
      close,
    ),
  ];
}

/**
 Read the targets a crate declares.

 Only single-line `name = "..."` and `path = "..."` keys of `[package]`, `[lib]` and `[[test]]` are read, which is how
 this repository writes them; a test target a crate leaves to Cargo's discovery (`tests/*.rs` without a `[[test]]`
 table) is not listed.

 @param manifest - the text of `Cargo.toml`

 @param directory - the crate's directory, against which the paths resolve

 @returns the library name and root, and the declared test targets

 @throws CargoMutantsToolingError when the manifest names no package

 @example
 ```ts
 parseCargoTargets({ manifest: '[package]\nname = "a-b"\n', directory: '/crate' });
 ```
 */
export function parseCargoTargets({
  manifest,
  directory,
}: {
  readonly manifest: string;
  readonly directory: string;
}): CrateTargets {
  /**
   Sections in file order, each with the keys read from it.
   */
  const sections: CargoSection[] = [{ table: '' }];
  for (const line of manifest.split('\n')) {
    /**
     The line without surrounding blanks.
     */
    const text = line.trim();
    if (text.startsWith('[') && text.endsWith(']'))
      sections.push({
        table: text.replaceAll(
          '[',
          '',
        )
          .replaceAll(
          ']',
          '',
          ),
      });
    else {
      /**
       The key and the quoted value of a `key = "value"` line, or nothing for any other line.
       */
      const pair = quotedPair(text);
      /**
       The section the line belongs to.
       */
      const current = sections.pop();
      if (current !== undefined)
        sections.push(pair.length === 0 ? current : {
          ...current,
          [pair[0] ?? '']: pair[1],
        });
    }
  }
  /**
   The package name.
   */
  const packageName = sections.find(function isPackage(section) {
    return section.table === 'package';
  })
    ?.name;
  if (packageName === undefined)
    throw new CargoMutantsToolingError('Cargo.toml has no [package] name, so the crate\'s targets cannot be read.');
  /**
   The `[lib]` section, if any.
   */
  const library = sections.find(function isLibrary(section) {
    return section.table === 'lib';
  });
  return {
    libraryName: library?.name ?? packageName.replaceAll(
      '-',
      '_',
    ),
    libraryRoot: resolve(
      directory,
      library?.path ?? 'src/lib.rs',
    ),
    testTargets: sections.filter(function isTest(section) {
      return (section.table === 'test') && (section.name !== undefined);
    })
      .map(function testTarget(section) {
        return {
          name: section.name ?? '',
          root: resolve(
            directory,
            section.path ?? `tests/${section.name ?? ''}.rs`,
          ),
        };
      }),
  };
}

/**
 Check whether a file exists.

 @param file - path to check

 @returns true when it can be accessed

 @example
 ```ts
 await exists('/crate/src/lib.rs');
 ```
 */
async function exists(file: string): Promise<boolean> {
  try {
    await access(file);
    return true;
  }
  catch (error) {
    // An absent file is the answer; any other failure also shows when the file is read.
    if (!isMissing(error))
      console.error(`Cannot access ${file}, so the module scan treats it as absent: ${String(error)}`);
    return false;
  }
}

/**
 Read a Rust source file for the module scan.

 @param file - absolute path

 @returns the text

 @throws CargoMutantsToolingError when the file cannot be read

 @example
 ```ts
 await readSource('/crate/src/lib.rs');
 ```
 */
async function readSource(file: string): Promise<string> {
  try {
    return await readFile(
      file,
      'utf8',
    );
  }
  catch (error) {
    throw new CargoMutantsToolingError(`Cannot read the Rust source ${file} for the module scan: ${String(error)}`);
  }
}

/**
 Find the files a `mod name;` declaration loads.

 @param declaration - the declaration, with its `#[path]` attributes and enclosing inline modules

 @param owner - the file that holds the declaration

 @param ownerIsRoot - whether the owner is a target root or a `mod.rs`, whose submodules sit beside it

 @returns the absolute paths of the files that exist: the one a direct `#[path]` names; or, without one, the file the
 module name gives (the first of `name.rs` and `name/mod.rs` that exists) and every `cfg_attr` path that exists. None when
 no candidate exists (a module left out by `cfg`, say).

 @example
 ```ts
 await declaredFiles({ declaration: { name: 'a' }, owner: '/c/src/lib.rs', ownerIsRoot: true });
 ```
 */
async function declaredFiles({
  declaration,
  owner,
  ownerIsRoot,
}: {
  readonly declaration: ModuleDeclaration;
  readonly owner: string;
  readonly ownerIsRoot: boolean;
}): Promise<readonly string[]> {
  /**
   Directory in which the owner's submodules sit, before any inline module.
   */
  const submoduleBase = ownerIsRoot
    ? dirname(owner)
    : join(
      dirname(owner),
      basename(
        owner,
        '.rs',
      ),
    );
  /**
   Directory the declaration's files are looked for in: the inline modules around it are directories.
   */
  const base = join(
    submoduleBase,
    ...declaration.inline ?? [],
  );
  /**
   Directory a `#[path]` is relative to: the owner's own directory, unless inline modules enclose the declaration.
   */
  const pathBase = declaration.inline === undefined ? dirname(owner) : base;
  if (declaration.path !== undefined) {
    /**
     The attribute's file.
     */
    const named = resolve(
      pathBase,
      declaration.path,
    );
    return await exists(named) ? [named] : [];
  }
  /**
   The two places Rust looks for the module, in the order it prefers them.
   */
  const candidates = [
    join(
      base,
      `${declaration.name}.rs`,
    ),
    join(
      base,
      declaration.name,
      'mod.rs',
    ),
  ];
  /**
   Whether each default candidate exists.
   */
  const found = await Promise.all(candidates.map(exists));
  /**
   The first default candidate that exists, as the only element, or none.
   */
  const byName = candidates.filter(function isFound(
    _candidate,
    index
  ) {
    return found[index] === true;
  })
    .slice(
    0,
    1,
    );
  /**
   The `cfg_attr` paths, resolved.
   */
  const conditional = (declaration.conditionalPaths ?? []).map(function resolved(path) {
    return resolve(
      pathBase,
      path,
    );
  });
  /**
   Whether each conditional path exists.
   */
  const conditionalFound = await Promise.all(conditional.map(exists));
  return [
    ...byName,
    ...conditional.filter(function isFound(
      _path,
      index
    ) {
      return conditionalFound[index] === true;
    }),
  ];
}

/**
 Read the module tree of one target and what its files refer to.

 @param root - absolute path of the target's root source file

 @param referenceRoots - identifiers that start a path into the crate (see `referencedNames`)

 @returns every module with its file, root included, and the names each file refers to; a file loaded as several modules
 appears once per module

 @throws CargoMutantsToolingError when the root cannot be read or the declarations nest deeper than the scan follows

 @example
 ```ts
 await scanTargetModules({ root: '/crate/src/lib.rs', referenceRoots: new Set(['crate', 'super']) });
 ```
 */
export async function scanTargetModules({
  root,
  referenceRoots,
}: {
  readonly root: string;
  readonly referenceRoots: ReadonlySet<string>;
}): Promise<TargetModules> {
  /**
   References per file.
   */
  const references = new Map<string, ReadonlySet<string>>();
  /**
   What the root module re-exports.
   */
  const reexports = new Map<string, readonly string[]>();
  /**
   Read one module, then the modules it declares, which are independent of each other.

   @param entry - the module to read

   @returns the module and all below it, in declaration order

   @throws CargoMutantsToolingError when the declarations nest deeper than the scan follows or a file cannot be read
   */
  async function visit(entry: ModuleEntry): Promise<readonly ModuleEntry[]> {
    if (entry.path
      .length
      > maximumModuleDepth)
      throw new CargoMutantsToolingError(`Module declarations nest deeper than ${maximumModuleDepth} below ${root}; the scan stopped at ${entry.file}.`);
    /**
     Tokens of the entry's file.
     */
    const tokens = tokenizeRust(await readSource(entry.file));
    if (entry.path
      .length
      === 0) {
      for (const [item, modules] of rootReexports(tokens)) {
        reexports.set(
          item,
          [
            ...reexports.get(item) ?? [],
            ...modules
          ],
        );
      }
    }
    references.set(
      entry.file,
      new Set([
        ...references.get(entry.file) ?? [],
        ...referencedNames({
          tokens,
          roots: referenceRoots,
          depth: entry.path
            .length,
        }),
      ]),
    );
    /**
     The declared modules whose files exist, each as an entry; a module with several candidate files gives several.
     */
    const children = (await Promise.all(moduleDeclarations(tokens)
      .map(async function declaredEntries(declaration) {
        /**
         The declared module's files that exist.
         */
        const files = await declaredFiles({
          declaration,
          owner: entry.file,
          ownerIsRoot: (entry.path
            .length
            === 0) || (basename(entry.file) === 'mod.rs'),
        });
        return files.map(function childEntry(file) {
          return {
            path: [
              ...entry.path,
              ...declaration.inline ?? [],
              declaration.name,
            ],
            file,
          };
        });
      }))).flat();
    return [
      entry,
      ...(await Promise.all(children.map(visit))).flat(),
    ];
  }
  return {
    modules: await visit({
      path: [],
      file: root,
    }),
    references,
    reexports,
  };
}
