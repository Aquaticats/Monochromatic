/**
 Holds every package whose unit tests import the package's own built `dist/` to a task that rebuilds it first.

 A unit test imports its subject from `../dist/final/<target>/index.mjs` (testing-practices skill), so a run without a
 build reads whatever the last build left, or dies with `ERR_MODULE_NOT_FOUND` when nothing built at all. The shared
 `[task_templates."test:unit"]` builds nothing, and a child task that extends it cannot prepend a build to the inherited
 `run`, so the ordering is declared as `depends = ["build"]` on the package task; `package/config/pnpr/mise.toml` is the
 model every other package follows.

 The scan reads every `*.unit.test.ts` under `package/<category>/<name>` and every relative import specifier in it. A specifier that
 resolves inside the same package's own `dist/` folder makes the package a dist reader. Two controls prove the scan can
 see such an import and can also stay silent about a package whose unit tests only reach sibling source.

 A dist reader that declares no `build` task cannot be guarded at all, so the first check names it rather than letting
 the guard pass in silence.
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

//region The paths, headers and patterns the scan reads

/**
 The repository root, because a test file runs with its package folder as the working directory.
 */
const repositoryRoot = fileURLToPath(new URL('../../../../', import.meta.url,),);

/**
 One pattern per import form that can name a relative module: a static import or re-export, a dynamic import call, and a
 side-effect import. Each captures the specifier in group one.
 */
const relativeSpecifierSources: readonly string[] = [
  'from\\s*[\'"](\\.{1,2}/[^\'"]*)[\'"]',
  'import\\s*\\(\\s*[\'"](\\.{1,2}/[^\'"]*)[\'"]',
  '^\\s*import\\s+[\'"](\\.{1,2}/[^\'"]*)[\'"]',
];

/**
 The folder whose presence at the top of a resolved specifier means a test reads a build product.
 */
const buildOutputFolder = 'dist';

/**
 The folder name every package's build output sits in, used as the guard's dependency.
 */
const buildTaskName = 'build';

/**
 Header of the task whose dependency the guard belongs on.
 */
const unitTestTaskHeader = '[tasks."test:unit"]';

/**
 Header of the aggregate task a package without a `test:unit` of its own runs its tests from.
 */
const testTaskHeader = '[tasks.test]';

/**
 Header of the task every dist reader needs, because the guard is a dependency on it.
 */
const buildTaskHeader = '[tasks.build]';

/**
 The dependency line a guarded task carries.
 */
const dependencyLine = 'depends';

/**
 A package whose unit tests read the package's own build output, with the task that runs them.
 */
interface DistReadingPackage {
  /** Repository-relative package folder, such as `package/config/pnpr`. */
  readonly path: string;
  /** The package's unit test files that import a path inside its own `dist/`. */
  readonly testFiles: readonly string[];
  /** The package's `test:unit` task, absent when the package declares none. */
  readonly unitTestTask: TaskBlock | undefined;
  /** The package's `test` task, absent when the package declares none. */
  readonly testTask: TaskBlock | undefined;
  /** The package's `build` task, absent when the package declares none. */
  readonly buildTask: TaskBlock | undefined;
}

/**
 One task table of a `mise.toml`, with the task names it depends on.
 */
interface TaskBlock {
  /** The table header, exactly as written, such as `[tasks."test:unit"]`. */
  readonly header: string;
  /** Everything between this header and the next one. */
  readonly body: string;
  /** The task names the block's `depends` entries name. */
  readonly dependencies: readonly string[];
}

//endregion

//region Readings

/**
 The quoted words of a text, in order.

 @param text - text holding TOML string values or an array of them

 @returns every quoted word, so a dependency of either the string or the inline-table form is read

 @example
 ```ts
 const names = quotedWordsOf('depends = ["build"]');
 ```
 */
function quotedWordsOf(text: string,): readonly string[] {
  return [...text.matchAll(/["']([^"']+)["']/gu),].map(function nameOf(match,) {
    return match[1];
  },);
}

/**
 The task names a task block's `depends` entry names.

 The entry is read as the lines from `depends` to its closing bracket, so an array broken across lines reads the same as
 one written on one line.

 @param body - a task block's body

 @returns every task name the block depends on

 @example
 ```ts
 const names = dependenciesOfTaskBody('depends = ["build"]\n');
 ```
 */
function dependenciesOfTaskBody(body: string,): readonly string[] {
  const dependencyText: string[] = [];
  let insideDependency = false;
  for (const line of body.split('\n',)) {
    if (!insideDependency && !line.trimStart().startsWith(`${dependencyLine} `,))
      continue;
    insideDependency = true;
    dependencyText.push(line,);
    if (line.includes(']',))
      break;
  }
  return quotedWordsOf(dependencyText.join('\n',),);
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
  const blocks: TaskBlock[] = [];
  let header: string | undefined;
  let body: string[] = [];
  const push = (): void => {
    if (header === undefined)
      return;
    const oneBody = body.join('\n',);
    blocks.push({ header, body: oneBody, dependencies: dependenciesOfTaskBody(oneBody,), },);
  };
  for (const line of text.split('\n',)) {
    if (line.startsWith('[',)) {
      push();
      header = line.trim();
      body = [];
      continue;
    }
    body.push(line,);
  }
  push();
  return blocks;
}

/**
 The task block of one table header.

 @param blocks - the task blocks of a `mise.toml`

 @param header - the table header to find, exactly as written

 @returns the block, or undefined when the file declares no such table

 @example
 ```ts
 const buildTask = taskBlockOf({ blocks, header: '[tasks.build]' });
 ```
 */
function taskBlockOf(
  { blocks, header, }: { readonly blocks: readonly TaskBlock[]; readonly header: string; },
): TaskBlock | undefined {
  return blocks.find(function hasHeader(block,) {
    return block.header === header;
  },);
}

/**
 The relative import specifiers of one test file.

 @param text - the contents of a test file

 @returns every relative specifier, in source order, across the import forms the sources above name

 @example
 ```ts
 const specifiers = relativeSpecifiersOf("import { a } from '../dist/final/neutral/index.mjs'");
 ```
 */
function relativeSpecifiersOf(text: string,): readonly string[] {
  return relativeSpecifierSources.flatMap(function specifiersOfSource(source,) {
    return [...text.matchAll(new RegExp(source, 'gmu',),),].map(function specifierOf(match,) {
      return match[1];
    },);
  },);
}

/**
 Where a relative specifier points, as repository-relative path segments.

 @param file - repository-relative path of the importing file

 @param specifier - the relative specifier that file imports

 @returns the resolved segments, with `..` applied and `.` dropped

 @example
 ```ts
 const segments = resolvedSegmentsOf({ file: 'package/a/b/src/x.unit.test.ts', specifier: '../dist/x.mjs' });
 ```
 */
function resolvedSegmentsOf(
  { file, specifier, }: { readonly file: string; readonly specifier: string; },
): readonly string[] {
  return specifier.split('/',).reduce<readonly string[]>(function resolve(segments, part,) {
    if (part === '.' || part === '')
      return segments;
    if (part === '..')
      return segments.slice(0, -1,);
    return [...segments, part,];
  }, file.split('/',).slice(0, -1,),);
}

/**
 Whether one specifier of one test file reaches the importing package's own build output.

 @param file - repository-relative path of the importing file

 @param packagePath - repository-relative folder of the package that owns the file

 @param specifier - the relative specifier to resolve

 @returns true when the resolved path sits at or under the package's own `dist/` folder

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
  return resolved.slice(0, packageSegments.length,).join('/',) === packagePath
    && resolved[packageSegments.length] === buildOutputFolder;
}

/**
 Every package folder under `package/<category>/<name>`, which is the shape the root `[monorepo] config_roots` declares.

 @returns repository-relative package folders, sorted

 @example
 ```ts
 const packages = await packagePaths();
 ```
 */
async function packagePaths(): Promise<readonly string[]> {
  const manifestPaths = await Array.fromAsync(
    glob('package/*/*/mise.toml', { cwd: repositoryRoot, },),
  );
  return manifestPaths.map(function packageOf(manifestPath,) {
    return manifestPath.slice(0, -'/mise.toml'.length,);
  },).toSorted();
}

/**
 Whether a package's unit test files import its own build output.

 @param path - repository-relative package folder

 @returns the test files that do, sorted; empty when the package reads no build output

 @example
 ```ts
 const readers = await distReadingTestFiles('package/config/pnpr');
 ```
 */
async function distReadingTestFiles(path: string,): Promise<readonly string[]> {
  const testFiles = await Array.fromAsync(
    glob('**/*.unit.test.ts', {
      cwd: join(repositoryRoot, path,),
      exclude: function isSkippedFolder(name,) {
        return name === 'node_modules' || name === buildOutputFolder;
      },
    },),
  );
  const readers: string[] = [];
  for (const testFile of testFiles.toSorted()) {
    const file = `${path}/${testFile}`;
    const text = await readFile(join(repositoryRoot, file,), 'utf8',);
    const readsDist = relativeSpecifiersOf(text,).some(function specifierReachesDist(specifier,) {
      return reachesOwnDist({ file, packagePath: path, specifier, },);
    },);
    if (readsDist)
      readers.push(file,);
  }
  return readers;
}

/**
 Every package whose unit tests read its own build output, with the tasks that must rebuild it first.

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
    if (testFiles.length === 0)
      return undefined;
    const text = await readFile(join(repositoryRoot, path, 'mise.toml',), 'utf8',);
    const blocks = taskBlocksOf(text,);
    return {
      path,
      testFiles,
      unitTestTask: taskBlockOf({ blocks, header: unitTestTaskHeader, },),
      testTask: taskBlockOf({ blocks, header: testTaskHeader, },),
      buildTask: taskBlockOf({ blocks, header: buildTaskHeader, },),
    } satisfies DistReadingPackage;
  },),);
  return scanned.filter(function isReader(reader,) {
    return reader !== undefined;
  },);
}

/**
 The readers whose named task carries no build dependency.

 @param readers - the dist readers

 @param taskOf - which task of a reader to inspect

 @returns one line per offender, naming the package and the task header the guard is missing from

 @example
 ```ts
 const offenders = unguardedReaders({ readers, taskOf: (reader) => reader.unitTestTask });
 ```
 */
function unguardedReaders(
  { readers, taskOf, }: {
    readonly readers: readonly DistReadingPackage[];
    readonly taskOf: (reader: DistReadingPackage) => TaskBlock | undefined;
  },
): readonly string[] {
  return readers.filter(function lacksGuard(reader,) {
    const task = taskOf(reader,);
    return task === undefined || !task.dependencies.includes(buildTaskName,);
  },).map(function describeOffender(reader,) {
    return `${reader.path} (${String(reader.testFiles.length,)} dist-reading unit test `
      + `files, ${unitTestTaskHeader} first: ${reader.testFiles[0] ?? reader.path})`;
  },);
}

/**
 The readers the scan must find and the reader it must not find, as the controls below prove.
 */
const knownDistReader = 'package/config/pnpr';

/**
 A package whose unit tests import sibling source only, so the scan must stay silent about it.
 */
const knownSiblingOnlyReader = 'package/or-throw/core';

//endregion

//region Right now

// Both readings are taken before the suites so every check below reads the same snapshot, and so a failure names the
// package rather than re-walking the tree once per check.
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
        /** A scan that flagged every package would pass vacuously. */
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
          return reader.buildTask === undefined;
        },).map(function nameOffender(reader,) {
          return reader.path;
        },);
        expect(offenders,).toEqual([],);
      },
    },),
    it({
      name: 'rebuilds before every package test:unit that runs dist-reading unit tests',
      fn: async (): Promise<void> => {
        /** The trap this check closes: a package test:unit run against a stale or absent build. */
        const offenders = unguardedReaders({
          readers: readers.filter(function hasUnitTestTask(reader,) {
            return reader.unitTestTask !== undefined;
          },),
          taskOf: function unitTestTaskOf(reader,) {
            return reader.unitTestTask;
          },
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
          return reader.unitTestTask === undefined;
        },).map(function describeOffender(reader,) {
          if (reader.testTask === undefined)
            return `${reader.path} (${String(reader.testFiles.length,)} dist-reading unit test files, no `
              + `${unitTestTaskHeader} and no ${testTaskHeader})`;
          return reader.testTask.dependencies.includes(buildTaskName,)
            ? undefined
            : `${reader.path} (${String(reader.testFiles.length,)} dist-reading unit test files, `
              + `${testTaskHeader} depends on ${reader.testTask.dependencies.join(', ',)}, not ${buildTaskName})`;
        },).filter(function isOffender(line,) {
          return line !== undefined;
        },);
        expect(offenders,).toEqual([],);
      },
    },),

    //endregion
  ],
},);
