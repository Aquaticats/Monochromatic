import {
  readdir,
  readFile,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import { compareCodePoints, } from '@monochromatic-dev/translation-repair-markup';

import {
  childNodes,
  identifierName,
  literalText,
  nodesUnder,
  parseSource,
  type SourceText,
} from './source-scan.test-fixture.ts';

//region Family source
// The files of the translation-repair family the scans read, and the resolution of the imports that join them.
//
// ONE PROGRAM, ONE KEY. The family is split into packages under `package/translation-repair`, and the
// scans measure it as the one program it was before the split. Every file is named by its package
// directory and its path under that package's `src`, joined by a slash (`adjudication/adjudicate-prompt.ts`,
// `run/corpus-run/cli-refusal.ts`), since every package has a `src/index.ts` and a bare path would name
// several files. Allowlists, findings and locations all use this key.

/**
 Directory holding the family's packages, one directory each.
 */
const FAMILY_ROOT: string = resolve(
  import.meta.dirname,
  '..',
  '..',
);

/**
 Names of the family's package directories, which prefix every location.

 @returns Directory names, sorted, those without a `src` directory left out

 @example
 ```ts
 const packages = await familyPackages(); // ['adjudication', 'archive', ...]
 ```
 */
export async function familyPackages(): Promise<readonly string[]> {
  /**
   Entries of the family directory.
   */
  const entries = await readdir(
    FAMILY_ROOT,
    { withFileTypes: true, },
  );
  /**
   Directories among them that hold a `src` directory.
   */
  const withSource = await Promise.all(entries
    .filter(function isDirectory(entry,): boolean {
      return entry.isDirectory();
    },)
    .map(async function hasSource({ name, },): Promise<string> {
      return (await readdir(join(
        FAMILY_ROOT,
        name,
      ),)).includes('src',) ? name : '';
    },),);
  return withSource
    .filter(function named(name,): boolean {
      return name !== '';
    },)
    .toSorted();
}

/**
 Every TypeScript file under every family package's `src`, with its text.

 @returns Files in no particular order, each located as the package directory and its path under `src`

 @example
 ```ts
 const files = await readPackageSource(); // paths like 'adjudication/adjudicate-prompt.ts'
 ```
 */
export async function readPackageSource(): Promise<readonly SourceText[]> {
  return (await Promise.all((await familyPackages()).map(async function readPackage(name,): Promise<readonly SourceText[]> {
    /**
     Source root of this package.
     */
    const srcDir = join(
      FAMILY_ROOT,
      name,
      'src',
    );
    /**
     Paths of the TypeScript files under it.
     */
    const paths = (await readdir(
      srcDir,
      { recursive: true, },
    ))
      .filter(function isTypeScript(path,): boolean {
        return path.endsWith('.ts',);
      },);
    return await Promise.all(paths.map(async function read(path,): Promise<SourceText> {
      return {
        path: `${name}/${path}`,
        text: await readFile(
          join(
            srcDir,
            path,
          ),
          'utf8',
        ),
        isTest: path.endsWith('.test.ts',) || path.endsWith('.test-fixture.ts',),
      };
    },),);
  },),)).flat();
}

/**
 Path of a location under its package's `src`, the package directory cut away.

 @param path - location as `readPackageSource` names it

 @returns The path relative to the package's `src`

 @example
 ```ts
 const underSrc = pathUnderSrc({ path: 'run/corpus-run/cli-refusal.ts', },); // 'corpus-run/cli-refusal.ts'
 ```
 */
export function pathUnderSrc({ path, }: { readonly path: string; },): string {
  return path.slice(path.indexOf('/',) + 1,);
}

/**
 File system path of a location.

 @param path - location as `readPackageSource` names it

 @returns The absolute path of the file under its package's `src`

 @example
 ```ts
 const at = sourcePathOf({ path: 'run/corpus-run/cli-refusal.ts', },); // '.../package/translation-repair/run/src/corpus-run/cli-refusal.ts'
 ```
 */
export function sourcePathOf({ path, }: { readonly path: string; },): string {
  return join(
    FAMILY_ROOT,
    packageDirectoryOf({ path, },),
    'src',
    pathUnderSrc({ path, },),
  );
}

/**
 Package directory of a location.

 @param path - location as `readPackageSource` names it

 @returns The directory name of the package holding the file

 @example
 ```ts
 const directory = packageDirectoryOf({ path: 'run/corpus-run/cli-refusal.ts', },); // 'run'
 ```
 */
export function packageDirectoryOf({ path, }: { readonly path: string; }): string {
  return path.slice(
    0,
    path.indexOf('/',),
  );
}

/**
 Resolves a relative module specifier against the file that holds it.

 @param fromPath - importing file's location

 @param specifier - relative specifier the import or re-export names

 @returns The location it names

 @example
 ```ts
 resolveSpecifier({ fromPath: 'run/corpus-run/a.test.ts', specifier: '../b.test-fixture.ts', },); // 'run/b.test-fixture.ts'
 ```
 */
function resolveSpecifier(
  {
    fromPath,
    specifier,
  }: {
    readonly fromPath: string;
    readonly specifier: string;
  },
): string {
  /**
   Directory segments of the importing file, its own file name dropped.
   */
  const resolved = fromPath
    .split('/',)
    .slice(
    0,
    -1,
    );
  for (const segment of specifier.split('/',)) {
    if (segment === '..')
      resolved.pop();
    else if ((segment !== '.') && (segment !== ''))
      resolved.push(segment,);
  }
  return resolved.join('/',);
}

/**
 What every family package's name opens with.
 */
const FAMILY_NAME_OPENING = '@monochromatic-dev/translation-repair-';

/**
 What a family package specifier names.
 */
type FamilySpecifier = {
  /**
   The package directory.
   */
  readonly directory: string;

  /**
   The path under `src` after `/ts/`, absent for the package's index (`<name>` or `<name>/ts`).
   */
  readonly subpath?: string;
};

/**
 Reads a specifier that names a family package.

 @param specifier - module specifier an import, re-export or dynamic import names

 @returns What it names, none for a specifier of any other package or a subpath other than `ts`

 @example
 ```ts
 const [named] = familySpecifiersOf({ specifier: '@monochromatic-dev/translation-repair-archive/ts/cat.ts', },); // { directory: 'archive', subpath: 'cat.ts', }
 ```
 */
function familySpecifiersOf({ specifier, }: { readonly specifier: string; },): readonly FamilySpecifier[] {
  if (!specifier.startsWith(FAMILY_NAME_OPENING,))
    return [];
  /**
   What follows the common opening: the directory, then perhaps `/ts` and a path.
   */
  const rest = specifier.slice(FAMILY_NAME_OPENING.length,);
  /**
   Where the directory ends.
   */
  const slash = rest.indexOf('/',);
  if (slash === (-1))
    return [{ directory: rest, },];
  /**
   The package directory.
   */
  const directory = rest.slice(
    0,
    slash,
  );
  /**
   What follows the directory.
   */
  const after = rest.slice(slash + 1,);
  if (after === 'ts')
    return [{ directory, },];
  return after.startsWith('ts/',)
    ? [{
      directory,
      subpath: after.slice('ts/'.length,),
    },]
    : [];
}

/**
 The module that declares each name a package's `src/index.ts` re-exports.

 @param directory - package directory

 @returns Exported name to the location of the module the index re-exports it from, empty for a package
 with no index

 @example
 ```ts
 const declarers = await indexDeclarersOf({ directory: 'archive', },); // 'confirmAbsoluteNaturalness' to 'archive/absolute-naturalness-confirmation.ts'
 ```
 */
async function indexDeclarersOf({ directory, }: { readonly directory: string; },): Promise<ReadonlyMap<string, string>> {
  /**
   Directory holding the package's source.
   */
  const srcDir = join(
    FAMILY_ROOT,
    directory,
    'src',
  );
  if (!(await readdir(srcDir,)).includes('index.ts',))
    return new Map();
  /**
   Location the index goes by.
   */
  const indexPath = `${directory}/index.ts`;
  /**
   The index parsed.
   */
  const { program, } = parseSource({
    file: {
      path: indexPath,
      text: await readFile(
        join(
          srcDir,
          'index.ts',
        ),
        'utf8',
      ),
      isTest: false,
    },
  },);
  /**
   Table built from its re-exports.
   */
  const declarers = new Map<string, string>();
  for (const statement of childNodes({ node: program, },)) {
    if (statement.type !== 'ExportNamedDeclaration')
      continue;
    /**
     Module the statement re-exports from, empty for an export list of the index's own names.
     */
    const from = literalText({ node: statement.source, },);
    if (from === '')
      continue;
    for (const specifier of childNodes({ node: statement, },)) {
      if (specifier.type !== 'ExportSpecifier')
        continue;
      declarers.set(
        identifierName({ node: specifier.exported, },) || literalText({ node: specifier.exported, },),
        resolveSpecifier({
          fromPath: indexPath,
          specifier: from,
        },),
      );
    }
  }
  return declarers;
}

/**
 Declaring module of every name each family package's index re-exports, by package directory. Read when the
 fixture loads rather than by `declaringModule`, which then reads no file: a fixture case of a scan resolves
 imports without touching the disk, and a scan of the scans takes a function that reads for one that reads the package.
 */
const INDEX_DECLARERS: ReadonlyMap<string, ReadonlyMap<string, string>> = new Map(await Promise.all(
  (await familyPackages()).map(async function declarersOfPackage(directory,): Promise<readonly [
    string,
    ReadonlyMap<string, string>,
  ]> {
    return [
      directory,
      await indexDeclarersOf({ directory, },),
    ];
  },),
),);

/**
 Whether a module specifier names a file of the family: a relative one, or one that names a family package
 (`<name>`, `<name>/ts` or `<name>/ts/<path>`).

 @param specifier - module specifier an import, re-export or dynamic import names

 @returns Whether `declaringModule` resolves it to a location

 @example
 ```ts
 const family = namesFamilyModule({ specifier: '@monochromatic-dev/translation-repair-archive/ts', },); // true
 ```
 */
export function namesFamilyModule({ specifier, }: { readonly specifier: string; },): boolean {
  return specifier.startsWith('.',) || (familySpecifiersOf({ specifier, },)
    .length
    > 0);
}

/**
 The module a name imported through a specifier is declared in: the file a relative specifier names, the file
 under `src` a family package's `/ts/<path>` names, the module a family package's index (`<name>` or
 `<name>/ts`) re-exports the name from, or the specifier itself for any other package.

 @param fromPath - importing file's location

 @param specifier - module specifier the import or re-export names

 @param imported - name taken from the module, `*` for a namespace and `default` for a default

 @returns The declaring module's location, the package's index when it re-exports no such name, or the specifier
 for a package outside the family

 @example
 ```ts
 const at = declaringModule({ fromPath: 'run/a.ts', specifier: '@monochromatic-dev/translation-repair-archive/ts', imported: 'confirmAbsoluteNaturalness', },);
 ```
 */
export function declaringModule(
  {
    fromPath,
    specifier,
    imported,
  }: {
    readonly fromPath: string;
    readonly specifier: string;
    readonly imported: string;
  },
): string {
  if (specifier.startsWith('.',)) {
    return resolveSpecifier({
      fromPath,
      specifier,
    },);
  }
  /**
   The family package the specifier names, absent for any other package.
   */
  const [family,] = familySpecifiersOf({ specifier, },);
  if (family === undefined)
    return specifier;
  /**
   Package directory and path after `/ts/` the specifier names, the path absent for the package's index.
   */
  const {
    directory,
    subpath,
  } = family;
  if (subpath !== undefined)
    return `${directory}/${subpath}`;
  return INDEX_DECLARERS.get(directory,)
    ?.get(imported,)
    ?? `${directory}/index.ts`;
}

/**
 One package's task file.
 */
type TaskFile = {
  /**
   Package directory that holds it.
   */
  readonly directory: string;

  /**
   File text.
   */
  readonly text: string;
};

/**
 The task file of every family package, which together carry the tasks the single package's task file once did.

 @returns One file per package that has a `src`, in package order

 @example
 ```ts
 const tasks = await readTaskFiles(); // [{ directory: 'adjudication', text: '[tasks.build]...', }, ...]
 ```
 */
export async function readTaskFiles(): Promise<readonly TaskFile[]> {
  return await Promise.all((await familyPackages()).map(async function readTaskFile(directory,): Promise<TaskFile> {
    return {
      directory,
      text: await readFile(
        join(
          FAMILY_ROOT,
          directory,
          'mise.toml',
        ),
        'utf8',
      ),
    };
  },),);
}

/**
 One runner the family builds: a command the pipeline's tasks start, found in the `build-entries.ts` of the
 package that owns its entry file.
 */
type RunnerEntry = {
  /**
   The command's name, which names its built file and its task.
   */
  readonly name: string;

  /**
   Package directory that lists it.
   */
  readonly directory: string;

  /**
   The entry file's location, as `readPackageSource` names it.
   */
  readonly path: string;
};

/**
 Opening every runner entry's source path carries in a `build-entries.ts`.
 */
const ENTRY_SOURCE_OPENING = './src/';

/**
 Name of the entry that builds a package's library index rather than a runner.
 */
const INDEX_ENTRY = 'index';

/**
 Every runner the family builds, read off each package's `build-entries.ts`: the packages that own runners each list
 their own, and the library index each of them lists is left out, since what it reaches is library source.

 @param files - every file of the family; a package without a `build-entries.ts` lists none

 @returns One entry per runner, packages in path order and the runners of one package in no particular order

 @example
 ```ts
 const runners = runnerEntriesOf({ files: await readPackageSource(), },); // [{ name: 'corpus-pass', directory: 'corpus-pass', path: 'corpus-pass/corpus-run/corpus-pass.ts', }, ...]
 ```
 */
export function runnerEntriesOf({ files, }: { readonly files: readonly SourceText[]; },): readonly RunnerEntry[] {
  return files
    .filter(function isBuildEntries({ path, },): boolean {
      return pathUnderSrc({ path, },) === 'build-entries.ts';
    },)
    .toSorted(function byPath(
      left,
      right,
    ): number {
      return compareCodePoints({
        left: left.path,
        right: right.path,
      },);
    },)
    .flatMap(function entriesOf(file,): readonly RunnerEntry[] {
      return nodesUnder({ root: parseSource({ file, },)
        .program, },)
        .filter(function isPair(node,): boolean {
          return node.type === 'ArrayExpression';
        },)
        .flatMap(function pairOf(node,): readonly RunnerEntry[] {
          /**
           The two strings of a name and a source path, absent for any other array.
           */
          const [name = '', source = '', ...rest] = childNodes({ node, },)
            .map(function textOf(element,): string {
              return literalText({ node: element, },);
            },);
          if ((rest.length > 0) || (name === '')
            || (name === INDEX_ENTRY)
            || (!source.startsWith(ENTRY_SOURCE_OPENING,))
            || (!source.endsWith('.ts',)))
            return [];
          return [{
            name,
            directory: packageDirectoryOf({ path: file.path, },),
            path: `${packageDirectoryOf({ path: file.path, },)}/${source.slice(ENTRY_SOURCE_OPENING.length,)}`,
          },];
        },);
    },);
}

//endregion Family source
