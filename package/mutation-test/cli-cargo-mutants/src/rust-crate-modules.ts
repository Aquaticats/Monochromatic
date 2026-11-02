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
import {
  moduleDeclarations,
  referencedNames,
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
      sections.push({ table: text.replaceAll(
        /^\[+|\]+$/gu,
        '',
      ) });
    else {
      /**
       The key, the equals sign and a quoted value, when the line is one.
       */
      const match = (/^(name|path)\s*=\s*"([^"]*)"/u).exec(text);
      /**
       The section the line belongs to.
       */
      const current = sections.pop();
      if (current !== undefined)
        sections.push(match === null ? current : {
          ...current,
          [match[1] ?? '']: match[2],
        });
    }
  }
  /**
   The package name.
   */
  const packageName = sections.find(function isPackage(section) {
    return section.table === 'package';
  })?.name;
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
  catch {
    // An absent file is the answer, not a failure.
    return false;
  }
}

/**
 Find the file a `mod name;` declaration loads.

 @param declaration - the declaration's name and optional `#[path]`

 @param owner - the file that holds the declaration

 @param ownerIsRoot - whether the owner is a target root or a `mod.rs`, whose submodules sit beside it

 @returns the absolute path, or nothing when no candidate file exists (a module left out by `cfg`, say)

 @example
 ```ts
 await declaredFile({ declaration: { name: 'a' }, owner: '/c/src/lib.rs', ownerIsRoot: true });
 ```
 */
async function declaredFile({
  declaration,
  owner,
  ownerIsRoot,
}: {
  readonly declaration: { readonly name: string; readonly path?: string };
  readonly owner: string;
  readonly ownerIsRoot: boolean;
}): Promise<string | undefined> {
  if (declaration.path !== undefined) {
    /**
     The attribute's file, relative to the directory of the owner.
     */
    const named = resolve(
      dirname(owner),
      declaration.path,
    );
    return await exists(named) ? named : undefined;
  }
  /**
   Directory in which the owner's submodules sit.
   */
  const base = ownerIsRoot
    ? dirname(owner)
    : join(
      dirname(owner),
      basename(
        owner,
        '.rs',
      ),
    );
  for (const candidate of [
    join(
      base,
      `${declaration.name}.rs`,
    ),
    join(
      base,
      declaration.name,
      'mod.rs',
    ),
  ]) {
    if (await exists(candidate))
      return candidate;
  }
  return undefined;
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
   Modules found so far.
   */
  const modules: ModuleEntry[] = [];
  /**
   References per file.
   */
  const references = new Map<string, ReadonlySet<string>>();
  /**
   Modules still to read: a work stack, so the nesting of modules costs no call depth.
   */
  const pending: ModuleEntry[] = [{
    path: [],
    file: root,
  }];
  for (let entry = pending.pop(); entry !== undefined; entry = pending.pop()) {
    if (entry.path.length > maximumModuleDepth)
      throw new CargoMutantsToolingError(`Module declarations nest deeper than ${maximumModuleDepth} below ${root}; the scan stopped at ${entry.file}.`);
    modules.push(entry);
    /**
     Tokens of the entry's file.
     */
    let tokens;
    try {
      tokens = tokenizeRust(await readFile(
        entry.file,
        'utf8',
      ));
    }
    catch (error) {
      throw new CargoMutantsToolingError(`Cannot read the Rust source ${entry.file} for the module scan: ${String(error)}`);
    }
    if (!references.has(entry.file)) {
      references.set(
        entry.file,
        referencedNames({
          tokens,
          roots: referenceRoots,
        }),
      );
    }
    for (const declaration of moduleDeclarations(tokens)) {
      /**
       The declared module's file, when one exists.
       */
      const file = await declaredFile({
        declaration,
        owner: entry.file,
        ownerIsRoot: (entry.path.length === 0) || (basename(entry.file) === 'mod.rs'),
      });
      if (file !== undefined) {
        pending.push({
          path: [...entry.path, declaration.name],
          file,
        });
      }
    }
  }
  return {
    modules,
    references,
  };
}
