/**
 Holds every package whose unit tests import the package's own built `dist/` to a task that rebuilds it first.

 A unit test imports its subject from `../dist/final/<target>/index.mjs` (testing-practices skill), so a run without a
 build reads whatever the last build left, or dies with `ERR_MODULE_NOT_FOUND` when nothing built at all. The shared
 `[task_templates."test:unit"]` builds nothing, and a child task that extends it cannot prepend a build to the inherited
 `run`, so the ordering is declared as `depends = ["build"]` on the package task; `package/config/pnpr/mise.toml` is the
 model every other package follows.

 The scan reads every `*.unit.test.ts` under `package/<category>/<name>` and every module specifier in it. A specifier
 that resolves inside the same package's own `dist/` folder makes the package a dist reader. Two controls prove the scan
 can see such an import and can also stay silent about a package whose unit tests only reach sibling source.

 A dist reader that declares no `build` task cannot be guarded at all, so the first check names it rather than letting
 the guard pass in silence.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  glob,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { fileURLToPath } from 'node:url';

//region The words, headers and paths the scan reads

/**
 The repository root, because a test file runs with its package folder as the working directory.
 */
const repositoryRoot = fileURLToPath(new URL('../../../../', import.meta.url,),);

/**
 The characters that open and close a quoted module specifier.
 */
const quoteCharacters: readonly string[] = ['"', '\'',];

/**
 The word a re-export or a static import names its module after.
 */
const fromKeyword = 'from';

/**
 The word a side-effect import, a dynamic import call and a static import all begin with.
 */
const importKeyword = 'import';

/**
 The prefixes every relative module specifier starts with, which are the only specifiers a package's own build satisfies.
 */
const relativeSpecifierPrefixes: readonly string[] = ['./', '../',];

/**
 The folder whose presence just below a package folder means a test reads a build product.
 */
const buildOutputFolder = 'dist';

/**
 Header of the task the guard is a dependency of.
 */
const unitTestTaskHeader = '[tasks."test:unit"]';

/**
 Header of the aggregate task a package without a `test:unit` of its own runs its tests from.
 */
const testTaskHeader = '[tasks.test]';

/**
 Header of the task every dist reader needs, because the guard depends on it.
 */
const buildTaskHeader = '[tasks.build]';

/**
 The dependency entry a guarded task carries.
 */
const dependencyLine = 'depends';

/**
 The task-name segment a build dependency holds, so `build`, `build:js:node` and `//package/a/b:build` all count.

 A dependency on one build target counts, because the package decides which target its suites read. The check owns the
 ordering, not the target choice, and a neutral-target reader that guarded only the node target still passes here; that
 limit is recorded rather than guessed at.
 */
const buildTaskSegment = 'build';

/**
 One task table of a `mise.toml`, with the task names it depends on.
 */
type TaskBlock = {
  /** The table header, exactly as written, such as `[tasks."test:unit"]`. */
  readonly header: string;
  /** Everything between this header and the next one. */
  readonly body: string;
  /** The task names the block's `depends` entries name. */
  readonly dependencies: readonly string[];
};

/**
 A package whose unit tests read the package's own build output.
 */
type DistReadingPackage = {
  /** Repository-relative package folder, such as `package/config/pnpr`. */
  readonly path: string;
  /** The package's unit test files that import a path inside the package's own `dist` folder. */
  readonly testFiles: readonly string[];
  /** Whether the package declares the build task the guard depends on. */
  readonly hasBuildTask: boolean;
  /** Whether the package declares its own `test:unit` task. */
  readonly hasUnitTestTask: boolean;
  /** The task names that package `test:unit` depends on. */
  readonly unitTestDependencies: readonly string[];
  /** Whether the package declares a `test` task. */
  readonly hasTestTask: boolean;
  /** The task names that package `test` task depends on. */
  readonly testDependencies: readonly string[];
};

/**
 A package folder with where its `mise.toml` header line sits.
 */
type LocatedHeader = {
  /** The header line, trimmed. */
  readonly header: string;
  /** Index of the header line among the file's lines. */
  readonly index: number;
};

//endregion

//region Reading a text

/**
 Whether one dependency name reaches a build task.

 @param name - one task name from a block's `depends`

 @returns true when the name holds `build` as a whole `:`-separated segment

 @example
 ```ts
 namesBuildTask('build:js:node'); // true
 ```
 */
function namesBuildTask(name: string,): boolean {
  return name.split(':',).includes(buildTaskSegment,);
}

/**
 The quoted strings that follow an import word on the same line, which is where a module specifier sits.

 Both a `from` and an `import` occurrence is read, because a re-export, a static import, a side-effect import and a
 dynamic import call each put the specifier within one line of one of them. Splitting on the keyword yields the text
 after each occurrence, and only the rest of that occurrence's own line is read.

 @param text - the contents of a file

 @returns every quoted string an import word introduces, in source order

 @example
 ```ts
 const specifiers = moduleSpecifiersOf("import { a } from '../dist/final/neutral/index.mjs'");
 ```
 */
function moduleSpecifiersOf(text: string,): readonly string[] {
  return [fromKeyword, importKeyword,].flatMap(function specifiersOfKeyword(keyword,) {
    const [, ...afterKeyword] = text.split(keyword,);
    return afterKeyword.flatMap(function specifierOf(rest,) {
      const [lineRest = '',] = rest.split('\n',);
      return quotedPathAfterImportWord(lineRest,);
    },);
  },);
}

/**
 The quoted path that opens right after an import word, which is where a module specifier sits.

 Only whitespace, and for a dynamic import call one parenthesis, may stand between the word and the path, so an import
 written inside a string of a fixture is not mistaken for one this file performs. The path runs to the next occurrence
 of its own quote character, so a quote that never closes before the line ends contributes nothing.

 @param restOfLine - the text after an import word, up to the end of its line

 @returns the quoted path as a one-element list, or an empty list when no quote opens there

 @example
 ```ts
 quotedPathAfterImportWord(" { a } from '../x.mjs'"); // []
 ```
 */
function quotedPathAfterImportWord(restOfLine: string,): readonly string[] {
  const trimmed = restOfLine.trimStart();
  const withoutParenthesis = trimmed.startsWith('(',) ? trimmed.slice(1,).trimStart() : trimmed;
  const quote = quoteCharacters.find(function opensThePath(candidate,) {
    return withoutParenthesis.startsWith(candidate,);
  },);
  if (quote === undefined)
    return [];
  const parts = withoutParenthesis.slice(1,).split(quote,);
  if (parts.length < 2)
    return [];
  const path = parts.at(0,);
  return path === undefined ? [] : [path,];
}

/**
 The double-quoted names of a text.

 Splitting on the quote character leaves the quoted contents at every odd index, so a text reads as its names with no
 scan. Single-quoted names are not read, and a `depends` written that way fails the checks below instead of passing
 them.

 @param text - the text of a `depends` entry, on one line or broken across several

 @returns every double-quoted name it holds, in order

 @example
 ```ts
 quotedNamesOf('["build", "lint"]'); // ['build', 'lint']
 ```
 */
function quotedNamesOf(text: string,): readonly string[] {
  const [, ...afterFirstQuote] = text.split('"',);
  return afterFirstQuote.filter(function wasQuoted(_chunk, index,) {
    return (index % 2) === 0;
  },);
}

/**
 The task names a task block's `depends` entry names.

 The entry is read from the first `depends` word to the first closing bracket, so an array broken across lines reads
 the same as one written on one line.

 @param body - a task block's body

 @returns every task name the block depends on

 @example
 ```ts
 const names = dependenciesOfTaskBody('depends = ["build"]\n');
 ```
 */
function dependenciesOfTaskBody(body: string,): readonly string[] {
  const [, afterDependency = '',] = body.split(dependencyLine,);
  const [entry = '',] = afterDependency.split(']',);
  return quotedNamesOf(entry,);
}

/**
 The task tables of one `mise.toml`, in file order.

 A line at column zero that opens a bracket starts a new table, which is the only structure this scan needs: every task
 header and every sub-table starts one.

 @param text - the contents of a `mise.toml`

 @returns every table with its body and its dependencies

 @example
 ```ts
 const blocks = taskBlocksOf('[tasks.build]\nrun = "x"\n');
 ```
 */
function taskBlocksOf(text: string,): readonly TaskBlock[] {
  const lines = text.split('\n',);
  const headers = lines.flatMap(function locateHeader(line, index,) {
    return line.startsWith('[',) ? [{ header: line.trim(), index, },] : [];
  },);
  return headers.map(function blockAt(entry, position,) {
    const next = headers.at(position + 1,);
    const body = lines.slice(entry.index + 1, next === undefined ? lines.length : next.index,)
      .join('\n',);
    return { header: entry.header, body, dependencies: dependenciesOfTaskBody(body,), };
  },);
}

/**
 Where a relative module specifier points, as repository-relative path segments.

 @param file - repository-relative path of the importing file

 @param specifier - the relative specifier that file imports

 @returns the resolved segments, with `..` applied and `.` dropped

 @example
 ```ts
 resolvedSegmentsOf({ file: 'package/a/b/src/x.unit.test.ts', specifier: '../dist/x.mjs' });
 ```
 */
function resolvedSegmentsOf(
  { file, specifier, }: { readonly file: string; readonly specifier: string; },
): readonly string[] {
  const segments = file.split('/',).slice(0, -1,);
  for (const part of specifier.split('/',)) {
    if (part === '..')
      segments.pop();
    else if ((part !== '.') && (part !== ''))
      segments.push(part,);
  }
  return segments;
}

/**
 Whether one specifier of one test file reaches the importing package's own build output.

 @param file - repository-relative path of the importing file

 @param packagePath - repository-relative folder of the package that owns the file

 @param specifier - the relative specifier to resolve

 @returns true when the resolved path sits inside the package's own `dist` folder

 @example
 ```ts
 const reads = reachesOwnDist({ file, packagePath: 'package/a/b', specifier: '../dist/x.mjs' });
 ```
 */
function reachesOwnDist(
  { file, packagePath, specifier, }: {
    readonly file: string;
    readonly packagePath: string;
    readonly specifier: string;
  },
): boolean {
  const packageSegments = packagePath.split('/',);
  const resolved = resolvedSegmentsOf({ file, specifier, },);
  if (resolved.slice(0, packageSegments.length,).join('/',) !== packagePath)
    return false;
  return resolved.at(packageSegments.length,) === buildOutputFolder;
}

//endregion

//region Reading the packages

/**
 Every package folder under `package/<category>/<name>`, which is the shape the root `[monorepo] config_roots` declares.

 @returns repository-relative package folders, sorted

 @example
 ```ts
 const folders = await packagePaths();
 ```
 */
async function packagePaths(): Promise<readonly string[]> {
  const manifestSuffix = '/mise.toml';
  const manifestPaths = await Array.fromAsync(
    glob(`package/*/*${manifestSuffix}`, { cwd: repositoryRoot, },),
  );
  return manifestPaths.map(function folderOf(manifestPath,) {
    return manifestPath.slice(0, -manifestSuffix.length,);
  },).toSorted();
}

/**
 Whether a package's unit test files import the package's own build output.

 @param path - repository-relative package folder

 @returns the test files that do, sorted; empty when the package reads no build output

 @example
 ```ts
 const readers = await distReadingTestFiles('package/config/pnpr');
 ```
 */
async function distReadingTestFiles(path: string,): Promise<readonly string[]> {
  const relativeTestFiles = await Array.fromAsync(
    glob('**/*.unit.test.ts', {
      cwd: join(repositoryRoot, path,),
      exclude: function isSkippedFolder(name,) {
        return (name === 'node_modules') || (name === buildOutputFolder);
      },
    },),
  );
  const readings = await Promise.all(relativeTestFiles.toSorted().map(async function readsDist(
    relativeTestFile,
  ) {
    const file = `${path}/${relativeTestFile}`;
    const text = await readFile(join(repositoryRoot, file,), 'utf8',);
    const specifiers = moduleSpecifiersOf(text,).filter(function isRelative(specifier,) {
      return relativeSpecifierPrefixes.some(function startsRelative(prefix,) {
        return specifier.startsWith(prefix,);
      },);
    },);
    return {
      file,
      readsDist: specifiers.some(function reachesDist(specifier,) {
        return reachesOwnDist({ file, packagePath: path, specifier, },);
      },),
    };
  },),);
  return readings.filter(function isReader(reading,) {
    return reading.readsDist;
  },).map(function fileOf(reading,) {
    return reading.file;
  },);
}

/**
 The task names one task table's entry declares.

 @param blocks - the task tables of a `mise.toml`

 @param header - the table header to read, exactly as written

 @returns the task names that table depends on; empty when the file declares no such table

 @example
 ```ts
 const names = dependenciesOfHeader({ blocks, header: '[tasks.build]' });
 ```
 */
function dependenciesOfHeader(
  { blocks, header, }: { readonly blocks: readonly TaskBlock[]; readonly header: string; },
): readonly string[] {
  const block = blocks.find(function hasHeader(candidate,) {
    return candidate.header === header;
  },);
  return block === undefined ? [] : block.dependencies;
}

/**
 Every package whose unit tests read its own build output, with the tasks whose ordering the guard owns.

 @returns the dist readers, sorted by package path

 @example
 ```ts
 const readers = await distReadingPackages();
 ```
 */
async function distReadingPackages(): Promise<readonly DistReadingPackage[]> {
  const candidates = await packagePaths();
  const scanned = await Promise.all(candidates.map(async function scan(path,) {
    const testFiles = await distReadingTestFiles(path,);
    const text = await readFile(join(repositoryRoot, path, 'mise.toml',), 'utf8',);
    const blocks = taskBlocksOf(text,);
    const headers = new Set(blocks.map(function headerOf(block,) {
      return block.header;
    },),);
    return {
      path,
      testFiles,
      hasBuildTask: headers.has(buildTaskHeader,),
      hasUnitTestTask: headers.has(unitTestTaskHeader,),
      unitTestDependencies: dependenciesOfHeader({ blocks, header: unitTestTaskHeader, },),
      hasTestTask: headers.has(testTaskHeader,),
      testDependencies: dependenciesOfHeader({ blocks, header: testTaskHeader, },),
    } satisfies DistReadingPackage;
  },),);
  return scanned.filter(function readsDist(row,) {
    return row.testFiles.length > 0;
  },);
}

/**
 One line naming a dist reader and the task whose guard is missing.

 @param reader - the package the guard is missing from

 @param detail - what the missing guard would have been declared on

 @returns a line that names the package, its dist-reading file count and the task

 @example
 ```ts
 const line = offenderLineOf({ reader, detail: unitTestTaskHeader });
 ```
 */
function offenderLineOf(
  { reader, detail, }: { readonly reader: DistReadingPackage; readonly detail: string; },
): string {
  const firstTestFile = reader.testFiles.at(0,) ?? reader.path;
  return `${reader.path} (${String(reader.testFiles.length,)} dist-reading unit test files, `
    + `first ${firstTestFile}): ${detail}`;
}

/**
 The dependency names of a task, or a word saying it declares none.

 @param dependencies - the task names a block depends on

 @returns the names, comma separated

 @example
 ```ts
 const text = dependencyNamesOf([]); // 'nothing'
 ```
 */
function dependencyNamesOf(dependencies: readonly string[],): string {
  return dependencies.length === 0 ? 'nothing' : dependencies.join(', ',);
}

//endregion

//region The two controls

/**
 The package whose guard is the model every other package follows, which the scan must find.
 */
const knownDistReader = 'package/config/pnpr';

/**
 A package whose unit tests import sibling source only, about which the scan must stay silent.
 */
const knownSiblingOnlyReader = 'package/or-throw/core';

//endregion

//region What the checks read

// Both readings are taken once so every check below reads the same snapshot, and so a failure names a package rather
// than walking the tree again per check.
const readers = await distReadingPackages();
const readerPaths = readers.map(function pathOf(reader,) {
  return reader.path;
},);

//endregion

await describe({
  name: 'package unit tests build first',
  concurrency: 1,
  timeout: 300_000,
  children: [
    //region The scan reaches what it claims

    it({
      name: `finds the package the guard is modelled on, ${knownDistReader}, among its dist readers`,
      fn: async (): Promise<void> => {
        /** The model package must be readable, or every check below proves nothing. */
        expect(readerPaths,).toContain(knownDistReader,);
      },
    },),
    it({
      name: 'finds no build output among unit tests that import sibling source only',
      fn: async (): Promise<void> => {
        /** A scan that flagged every package would pass the checks below vacuously. */
        expect(readerPaths,).not.toContain(knownSiblingOnlyReader,);
      },
    },),

    //endregion

    //region Every dist reader rebuilds first

    it({
      name: 'names every dist reader that declares no build task, which no dependency can guard',
      fn: async (): Promise<void> => {
        /** Without a build task the guard has no target, so the package needs a decision, not a dependency. */
        const offenders = readers.filter(function lacksBuildTask(reader,) {
          return !reader.hasBuildTask;
        },).map(function pathOf(reader,) {
          return offenderLineOf({ reader, detail: `no ${buildTaskHeader}`, },);
        },);
        expect(offenders,).toEqual([],);
      },
    },),
    it({
      name: 'rebuilds before every package test:unit that runs dist-reading unit tests',
      fn: async (): Promise<void> => {
        /** The trap this check closes: a package test:unit run against a stale or absent build. */
        const offenders = readers.filter(function lacksUnitGuard(reader,) {
          if (!reader.hasUnitTestTask)
            return false;
          return !reader.unitTestDependencies.some(namesBuildTask,);
        },).map(function describeOffender(reader,) {
          return offenderLineOf({
            reader,
            detail: `${unitTestTaskHeader} depends on ${dependencyNamesOf(reader.unitTestDependencies,)}`,
          },);
        },);
        expect(offenders,).toEqual([],);
      },
    },),
    it({
      name: 'rebuilds inside the test task of every dist reader that declares no test:unit',
      fn: async (): Promise<void> => {
        /**
         A package whose unit files no `test:unit` runs needs the guard on the task that does run them, and a package
         with neither task is named here, because the root aggregate builds nothing either.
         */
        const offenders = readers.filter(function lacksUnitTestTask(reader,) {
          return !reader.hasUnitTestTask;
        },).flatMap(function describeOffender(reader,) {
          if (!reader.hasTestTask)
            return [offenderLineOf({ reader, detail: `no ${unitTestTaskHeader} and no ${testTaskHeader}`, },),];
          return reader.testDependencies.some(namesBuildTask,)
            ? []
            : [
              offenderLineOf({
                reader,
                detail: `${testTaskHeader} depends on ${dependencyNamesOf(reader.testDependencies,)}`,
              },),
            ];
        },);
        expect(offenders,).toEqual([],);
      },
    },),

    //endregion
  ],
},);
