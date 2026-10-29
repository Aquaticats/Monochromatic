/**
 Guards that every source scan says what it found when it fails. The
 assertion library prints a value longer than forty characters cut to its
 length, so a scan whose package-wide case compares its findings through
 `toEqual` or `toStrictEqual` fails with `expected [ …(31) ] to deeply equal
 []` and nothing a reader can act on (issue 610). The helpers of
 `scan-findings.test-fixture.ts` write every difference whole, one per line,
 and then make the same comparison, so a scan's package-wide case compares
 through them.

 WHAT THE SCAN READS. The scans are the test files the `source-scans` task of
 `mise.toml` names in its `run` line. A case of a scan reads the package when
 its `fn` names a binding imported from `node:fs` or `node:fs/promises`, or a
 function that reads the package: one declared by name in a test file or
 fixture whose body names such a binding or another such function, followed
 through relative imports to a fixed point (`readPackageSource`,
 `readPackageTexts`, a scan's own `scanSource`). In such a case a call of
 `toEqual` or `toStrictEqual` made straight on `expect(...)` is a finding,
 named by the scan's path and the line its `expect(` opens on; a case that
 plants its own few lines is read by none of this, since its value is short
 and written in the case. A scan none of whose cases reads the package is a
 finding too, so the reach this scan reads cannot go blind without a word,
 and so is a path the task names that is no test file under `src`.

 OUT OF ITS REACH: a reader bound to a constant rather than declared by name,
 one reached through a namespace import of a fixture, a comparison made inside
 a function the case calls rather than in the case itself, `expect(...).not`,
 and a case built by a function the suite only calls. It reads names, not
 scopes, so a local binding that shares a reader's name makes a case read as
 one that reads the package: a finding that errs toward reading.

 THE FIXTURE CASE COMES FIRST, so the package-wide case is read against a
 scan shown able to find each kind (ledger M21). Fixtures are cat-themed; the
 package case reads this package's own `mise.toml` and source.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';
import {
  declaringModule,
  readPackageSource,
} from './family-source.test-fixture.ts';
import {
  identifierName,
  isTreeNode,
  literalText,
  memberName,
  nodesUnder,
  parseSource,
  type SourceText,
  type TreeNode,
  unwrapped,
} from './source-scan.test-fixture.ts';

//region Scan comparisons
// Which test files are scans, which of their cases read the package, and the
// comparisons those cases make without a helper that names what differs.

/**
 Header of the task whose `run` line names every scan.
 */
const SCANS_TASK = '[tasks."source-scans"]';

/**
 Modules whose bindings read files.
 */
const FILE_READERS: ReadonlySet<string> = new Set([
  'node:fs',
  'node:fs/promises',
  'fs',
  'fs/promises',
],);

/**
 Matchers whose message cuts a long value.
 */
const CUTTING_MATCHERS: ReadonlySet<string> = new Set([
  'toEqual',
  'toStrictEqual',
],);

/**
 What one import binds a local name to.
 */
type Binding = {
  /**
   Module it comes from: the location of the family file that declares the name for a relative import or
   one naming a family package, the specifier as written for any other.
   */
  readonly module: string;

  /**
   Name the module exports it under.
   */
  readonly imported: string;
};

/**
 What the scan reads of one test file.
 */
type FileSyntax = {
  /**
   Path relative to `src`.
   */
  readonly path: string;

  /**
   File text, which lines are counted in.
   */
  readonly text: string;

  /**
   Parsed program.
   */
  readonly program: TreeNode;

  /**
   Every name its imports bind, with what each binds it to.
   */
  readonly bindings: ReadonlyMap<string, Binding>;

  /**
   Every function declared by name at its top, with the names its body holds.
   */
  readonly functions: ReadonlyMap<string, ReadonlySet<string>>;
};

/**
 The scans a task file names: every `src/` path on the `run` line of its
 `source-scans` task.

 @param mise - text of `mise.toml`

 @returns Paths relative to `src`, in the order the line names them

 @throws Error when the file holds no `source-scans` task or the task no `run`
 line, since a scan of nothing would pass

 @example
 ```ts
 const scans = scanPathsOf({ mise, },); // ['dead-code.unit.test.ts', …]
 ```
 */
function scanPathsOf({ mise, }: { readonly mise: string; },): readonly string[] {
  /**
   The file's lines.
   */
  const lines = mise.split('\n',);
  /**
   Where the task's table opens.
   */
  const header = lines.indexOf(SCANS_TASK,);
  if (header === (-1))
    throw new Error(`mise.toml holds no ${SCANS_TASK} table, so no scan can be read`,);
  /**
   The task's lines, up to the next table.
   */
  const rest = lines.slice(header + 1,);
  /**
   Where the next table opens, or the file's end.
   */
  const next = rest.findIndex(function opensTable(line,): boolean {
    return line.startsWith('[',);
  },);
  /**
   The task's run line.
   */
  const run = rest
    .slice(0, (next === (-1)) ? rest.length : next,)
    .find(function isRun(line,): boolean {
      return line.startsWith('run = "',);
    },);
  if (run === undefined)
    throw new Error(`mise.toml's ${SCANS_TASK} table holds no run line, so no scan can be read`,);
  return run
    .slice(
    'run = "'.length,
    -1,
    )
    .split(' ',)
    .filter(function isScan(token,): boolean {
      return token.startsWith('src/',);
    },)
    .map(function fromSrc(token,): string {
      return token.slice('src/'.length,);
    },);
}

/**
 Every identifier name under a node.

 @param node - node read

 @returns Names, each once

 @example
 ```ts
 const names = namesIn({ node: declaration, },);
 ```
 */
function namesIn({ node, }: { readonly node: TreeNode; },): ReadonlySet<string> {
  return new Set(nodesUnder({ root: node, },)
    .map(function nameOf(child,): string {
      return identifierName({ node: child, },);
    },)
    .filter(function isName(name,): boolean {
      return name.length > 0;
    },),);
}

/**
 Whether a value is a syntax node, as `isTreeNode` reads it: declared in this
 file, so a filter handed it is handed a function of one parameter it can see.

 @param value - value read

 @returns Whether it is a node

 @example
 ```ts
 const nodes = values.filter(isNode,);
 ```
 */
function isNode(value: unknown,): value is TreeNode {
  return isTreeNode(value,);
}

/**
 A program's top-level statements.

 @param program - file's program

 @returns Its statements in order

 @example
 ```ts
 const statements = statementsOf({ program, },);
 ```
 */
function statementsOf({ program, }: { readonly program: TreeNode; },): readonly TreeNode[] {
  return Array.isArray(program.body,) ? program.body.filter(isNode,) : [];
}

/**
 What a file's imports bind.

 @param path - file's path relative to `src`

 @param program - file's program

 @returns Each local name with the module and export it binds

 @example
 ```ts
 const bindings = bindingsOf({ path: 'cat.unit.test.ts', program, },);
 ```
 */
function bindingsOf(
  {
    path,
    program,
  }: {
    readonly path: string;
    readonly program: TreeNode;
  },
): ReadonlyMap<string, Binding> {
  /**
   The bindings found so far.
   */
  const bindings = new Map<string, Binding>();
  for (const statement of statementsOf({ program, },)) {
    if (statement.type !== 'ImportDeclaration')
      continue;
    /**
     The specifier as written.
     */
    const specifier = literalText({ node: statement.source, },);
    for (const bound of (Array.isArray(statement.specifiers,) ? statement.specifiers : []).filter(isNode,)) {
      /**
       The export it binds: its name, `default` or `*`.
       */
      const imported = (bound.type === 'ImportSpecifier')
        ? (identifierName({ node: bound.imported, },) || literalText({ node: bound.imported, },))
        : ((bound.type === 'ImportDefaultSpecifier') ? 'default' : '*');
      bindings.set(
        identifierName({ node: bound.local, },),
        {
          // The file that declares the name, for a relative specifier or one naming a family package;
          // the specifier as written for any other.
          module: declaringModule({
            fromPath: path,
            specifier,
            imported,
          },),
          imported,
        },
      );
    }
  }
  return bindings;
}

/**
 Every function a file declares by name at its top, exported or not.

 @param program - file's program

 @returns Each function's name with the names its body holds

 @example
 ```ts
 const functions = functionsOf({ program, },);
 ```
 */
function functionsOf({ program, }: { readonly program: TreeNode; },): ReadonlyMap<string, ReadonlySet<string>> {
  return new Map(statementsOf({ program, },)
    .map(function declared(statement,): unknown {
      return (statement.type === 'ExportNamedDeclaration') ? statement.declaration : statement;
    },)
    .filter(isNode,)
    .filter(function isFunction(declaration,): boolean {
      return declaration.type === 'FunctionDeclaration';
    },)
    .map(function named(declaration,): readonly [string, ReadonlySet<string>,] {
      return [
        identifierName({ node: declaration.id, },),
        namesIn({ node: declaration, },),
      ];
    },),);
}

/**
 Parses a test file into what the scan reads of it.

 @param file - file read

 @returns Its program, bindings and functions

 @example
 ```ts
 const syntax = syntaxOf({ file, },);
 ```
 */
function syntaxOf({ file, }: { readonly file: SourceText; },): FileSyntax {
  /**
   Parsed program.
   */
  const { program, } = parseSource({ file, },);
  return {
    path: file.path,
    text: file.text,
    program,
    bindings: bindingsOf({
      path: file.path,
      program,
    },),
    functions: functionsOf({ program, },),
  };
}

/**
 Whether any of a set of names, as a file binds or declares them, reads files.

 @param names - names read

 @param syntax - file the names stand in

 @param reading - functions known to read, as `path#name`

 @returns Whether one names a file reader or a function known to read

 @example
 ```ts
 const reads = namesRead({ names, syntax, reading, },);
 ```
 */
function namesRead(
  {
    names,
    syntax,
    reading,
  }: {
    readonly names: ReadonlySet<string>;
    readonly syntax: FileSyntax;
    readonly reading: ReadonlySet<string>;
  },
): boolean {
  return [...names,].some(function reads(name,): boolean {
    /**
     What an import binds the name to, absent for a name no import binds.
     */
    const binding = syntax.bindings.get(name,);
    if (binding !== undefined)
      return FILE_READERS.has(binding.module,) || reading.has(`${binding.module}#${binding.imported}`,);
    return syntax.functions.has(name,) && reading.has(`${syntax.path}#${name}`,);
  },);
}

/**
 Every function the test files declare that reads files, directly or through
 another such function, taken to a fixed point.

 @param syntaxes - test files read

 @returns Functions that read, as `path#name`

 @example
 ```ts
 const reading = readingFunctions({ syntaxes, },);
 ```
 */
function readingFunctions({ syntaxes, }: { readonly syntaxes: readonly FileSyntax[]; },): ReadonlySet<string> {
  /**
   Functions found to read so far.
   */
  const reading = new Set<string>();
  for (let grown = true; grown;) {
    grown = false;
    for (const syntax of syntaxes) {
      for (const [name, names,] of syntax.functions) {
        /**
         The function's key.
         */
        const key = `${syntax.path}#${name}`;
        if (reading.has(key,) || (!namesRead({ names, syntax, reading, },)))
          continue;
        reading.add(key,);
        grown = true;
      }
    }
  }
  return reading;
}

/**
 Line a node starts on.

 @param text - file's text

 @param node - node read

 @returns One-based line number

 @example
 ```ts
 const line = startLine({ text, node, },);
 ```
 */
function startLine({ text, node, }: { readonly text: string; readonly node: TreeNode; },): number {
  return text
    .slice(
    0,
    node.start,
    )
    .split('\n',)
    .length;
}

/**
 The matcher a call names when it compares straight on `expect(...)` through
 a matcher that cuts a long value.

 @param node - node read

 @returns Matcher's name, empty for any other node

 @example
 ```ts
 const matcher = cuttingMatcherOf({ node, },); // 'toEqual'
 ```
 */
function cuttingMatcherOf({ node, }: { readonly node: TreeNode; },): string {
  if (node.type !== 'CallExpression')
    return '';
  /**
   What the call calls.
   */
  const { inner: callee, } = unwrapped({ node: node.callee, },);
  if ((!isTreeNode(callee,)) || (callee.type !== 'MemberExpression'))
    return '';
  /**
   The member called.
   */
  const matcher = memberName({ node: callee, },);
  if (!CUTTING_MATCHERS.has(matcher,))
    return '';
  /**
   What the matcher is read off.
   */
  const { inner: subject, } = unwrapped({ node: callee.object, },);
  return (isTreeNode(subject,) && (subject.type === 'CallExpression')
    && (identifierName({ node: unwrapped({ node: subject.callee, },).inner, },) === 'expect'))
    ? matcher
    : '';
}

/**
 The `fn` a case is built with.

 @param node - node read

 @returns Function handed to `it` as `fn`, absent for any other node

 @example
 ```ts
 const fn = caseFunctionOf({ node, },);
 ```
 */
function caseFunctionOf({ node, }: { readonly node: TreeNode; },): unknown {
  if ((node.type !== 'CallExpression') || (identifierName({ node: node.callee, },) !== 'it'))
    return undefined;
  /**
   The call's arguments.
   */
  const handed: unknown = node.arguments;
  /**
   The case's options, its first argument.
   */
  const options: unknown = Array.isArray(handed,) ? handed.at(0,) : undefined;
  if ((!isTreeNode(options,)) || (options.type !== 'ObjectExpression') || (!Array.isArray(options.properties,)))
    return undefined;
  return options.properties
    .filter(isNode,)
    .find(function isFn(property,): boolean {
      return identifierName({ node: property.key, },) === 'fn';
    },)
    ?.value;
}

/**
 What one scan's cases hold: whether any reads the package, and every
 comparison such a case makes through a cutting matcher.

 @param syntax - scan read

 @param reading - functions known to read, as `path#name`

 @returns Whether a case reads the package, and each comparison as
 `path:line matcher`, the line its `expect` opens on, in line order

 @example
 ```ts
 const { readsPackage, comparisons, } = scanCases({ syntax, reading, },);
 ```
 */
function scanCases(
  {
    syntax,
    reading,
  }: {
    readonly syntax: FileSyntax;
    readonly reading: ReadonlySet<string>;
  },
): {
  readonly readsPackage: boolean;
  readonly comparisons: readonly string[];
} {
  /**
   Cases that read the package.
   */
  const wholeCases = nodesUnder({ root: syntax.program, },)
    .map(function fnOf(node,): unknown {
      return caseFunctionOf({ node, },);
    },)
    .filter(isNode,)
    .filter(function readsPackage(fn,): boolean {
      return namesRead({
        names: namesIn({ node: fn, },),
        syntax,
        reading,
      },);
    },);
  return {
    readsPackage: wholeCases.length > 0,
    comparisons: wholeCases
      .flatMap(function comparisonsIn(fn,): readonly TreeNode[] {
        return nodesUnder({ root: fn, },).filter(function compares(node,): boolean {
          return cuttingMatcherOf({ node, },).length > 0;
        },);
      },)
      .toSorted(function byStart(left, right,): number {
        return left.start - right.start;
      },)
      .map(function described(node,): string {
        return `${syntax.path}:${String(startLine({ text: syntax.text, node, },),)} ${cuttingMatcherOf({ node, },)}`;
      },),
  };
}

/**
 The scans and every test file or fixture they reach through relative
 imports, each parsed once: the files a reader could be declared in, and no
 more, since parsing every test file holds the process long enough for the
 test logger's own checks to time out.

 @param scans - scan paths relative to `src`

 @param files - package's files

 @returns What the scan reads of each file reached, scans among them

 @example
 ```ts
 const syntaxes = reachedSyntaxes({ scans: ['dead-code.unit.test.ts',], files, },);
 ```
 */
function reachedSyntaxes(
  {
    scans,
    files,
  }: {
    readonly scans: readonly string[];
    readonly files: readonly SourceText[];
  },
): readonly FileSyntax[] {
  /**
   Test files and fixtures by path.
   */
  const tests: ReadonlyMap<string, SourceText> = new Map(files
    .filter(function isTest(file,): boolean {
      return file.isTest;
    },)
    .map(function keyed(file,): readonly [string, SourceText,] {
      return [file.path, file,];
    },),);
  /**
   Files parsed so far.
   */
  const reached: FileSyntax[] = [];
  /**
   Paths already taken from the queue.
   */
  const taken = new Set<string>();
  /**
   Paths still to read.
   */
  const pending = [...scans,];
  for (let path = pending.pop(); path !== undefined; path = pending.pop()) {
    /**
     The file at the path, absent where no test file stands there.
     */
    const file = tests.get(path,);
    if (taken.has(path,) || (file === undefined))
      continue;
    taken.add(path,);
    /**
     What the scan reads of it.
     */
    const syntax = syntaxOf({ file, },);
    reached.push(syntax,);
    for (const { module, } of syntax.bindings.values())
      pending.push(module,);
  }
  return reached;
}

/**
 Every scan the task file names that compares through a cutting matcher in a
 case that reads the package, holds no case that reads it, or is no test file.

 @param mise - text of `mise.toml`

 @param files - family's files, tests and fixtures among them

 @param prefix - what the task file's `src/` paths are located under in `files`: the scan package's
 directory and a slash, empty where `files` belong to no package

 @returns Findings, scan by scan in the task's order and line by line within
 a scan

 @example
 ```ts
 const findings = scanComparisons({ mise, files: await readPackageSource(), },);
 ```
 */
function scanComparisons(
  {
    mise,
    files,
    prefix = '',
  }: {
    readonly mise: string;
    readonly files: readonly SourceText[];
    readonly prefix?: string;
  },
): readonly string[] {
  /**
   The scans the task names.
   */
  const scans = scanPathsOf({ mise, },).map(function located(path,): string {
    return `${prefix}${path}`;
  },);
  /**
   What the scan reads of each scan and each file it imports.
   */
  const syntaxes = reachedSyntaxes({
    scans,
    files,
  },);
  /**
   Functions that read the package.
   */
  const reading = readingFunctions({ syntaxes, },);
  return scans.flatMap(function findingsOf(path,): readonly string[] {
    /**
     What the scan reads of the scan's own file, absent where no test file
     stands at its path.
     */
    const syntax = syntaxes.find(function isScan(read,): boolean {
      return read.path === path;
    },);
    if (syntax === undefined)
      return [`${path}: named by the source-scans task and no test file under src`,];
    /**
     What its cases hold.
     */
    const { readsPackage, comparisons, } = scanCases({
      syntax,
      reading,
    },);
    return readsPackage ? comparisons : [`${path}: holds no case that reads the package`,];
  },);
}

//endregion Scan comparisons

/**
 A task file naming three cat scans, one of which is not there.
 */
const CAT_MISE = [
  '[tasks.build]',
  'run = "rolldown"',
  '',
  SCANS_TASK,
  'description = "every scan"',
  'run = "mise run test:unit src/napping.unit.test.ts src/purring.unit.test.ts src/dozing.unit.test.ts '
    + 'src/stray.unit.test.ts"',
  '',
  '[tasks.lint]',
  'run = "oxlint src/grooming.unit.test.ts"',
].join('\n',);

/**
 Cat-themed test files: a fixture holding a reader, a reader through it written
 first so that only a second pass over the files finds it, and a function that
 reads nothing; three scans; and a test no task names.
 */
const CAT_FILES: readonly SourceText[] = [
  {
    path: 'den.test-fixture.ts',
    isTest: true,
    text: [
      'import { readFile, } from \'node:fs/promises\';',
      'export async function readBasket() { return await readDen(); }',
      'export async function readDen() { return await readFile(\'den.txt\', \'utf8\',); }',
      'export function tidyDen(text: string) { return text.trim(); }',
    ].join('\n',),
  },
  {
    path: 'napping.unit.test.ts',
    isTest: true,
    text: [
      'import { readBasket, tidyDen, } from \'./den.test-fixture.ts\';',
      'async function readAll() { return [await readBasket(),]; }',
      'await describe({ name: \'napping\', children: [',
      '  it({ name: \'plants\', fn: async () => { expect(tidyDen(\' nap \',),).toEqual(\'nap\',); }, },),',
      '  it({ name: \'reads through a local reader\', fn: async () => { expect(await readAll(),).toEqual([],); }, },),',
      '  it({ name: \'reads straight\', fn: async () => {',
      '    expect(await readBasket(),)',
      '      .toStrictEqual(\'\',);',
      '  }, },),',
      '  it({ name: \'reads through a helper\', fn: async () => {',
      '    expectNoFindings({ findings: await readAll(), },);',
      '    expect((await readAll()).length,).toBeGreaterThan(0,);',
      '    expect(await readAll(),).not.toEqual([],);',
      '  }, },),',
      '], },);',
    ].join('\n',),
  },
  {
    path: 'purring.unit.test.ts',
    isTest: true,
    text: [
      'import * as fs from \'node:fs\';',
      'import { readdirSync, } from \'node:fs\';',
      'function denNames() { return readdirSync(\'den\',); }',
      'await describe({ name: \'purring\', children: [',
      '  it({ name: \'reads a namespace\', fn: async () => { expect(fs.readFileSync(\'purr.txt\', \'utf8\',),).toEqual(\'\',); }, },),',
      '  it({ name: \'lists through a local reader\', fn: async () => { expect(denNames(),).toEqual([],); }, },),',
      '], },);',
    ].join('\n',),
  },
  {
    path: 'dozing.unit.test.ts',
    isTest: true,
    text: [
      'import { tidyDen, } from \'./den.test-fixture.ts\';',
      'await describe({ name: \'dozing\', children: [',
      '  it({ name: \'plants alone\', fn: async () => { expect(tidyDen(\' doze \',),).toEqual(\'doze\',); }, },),',
      '], },);',
    ].join('\n',),
  },
  {
    path: 'grooming.unit.test.ts',
    isTest: true,
    text: [
      'import { readDen, } from \'./den.test-fixture.ts\';',
      'await describe({ name: \'grooming\', children: [',
      '  it({ name: \'reads\', fn: async () => { expect(await readDen(),).toEqual(\'\',); }, },),',
      '], },);',
    ].join('\n',),
  },
];

await describe({
  name: 'scan comparisons that cut what they found',
  children: [
    it({
      name: 'FINDS toEqual through a local reader of a fixture reader found on a second pass, toStrictEqual on a line '
        + 'of its own, toEqual through a namespace of node:fs and through a local function naming readdirSync, in '
        + 'cases that read; a scan no case of which reads; and a scan no file holds; and leaves a planted case, a '
        + 'helper, a count, expect(...).not and a test no task names',
      fn: async () => {
        expect(scanComparisons({
          mise: CAT_MISE,
          files: CAT_FILES,
        },),).toEqual([
          'napping.unit.test.ts:5 toEqual',
          'napping.unit.test.ts:7 toStrictEqual',
          'purring.unit.test.ts:5 toEqual',
          'purring.unit.test.ts:6 toEqual',
          'dozing.unit.test.ts: holds no case that reads the package',
          'stray.unit.test.ts: named by the source-scans task and no test file under src',
        ],);
      },
    },),
    it({
      name: 'REFUSES a task file with no source-scans table, and one whose source-scans table holds no run line '
        + 'before the next table',
      fn: async () => {
        /** What a task file with no such table threw. */
        const noTable = caught(function readsNoTable(): void {
          scanPathsOf({ mise: '[tasks.build]\nrun = "rolldown"', },);
        },);
        /** What a task file whose table holds no run line threw. */
        const noRun = caught(function readsNoRun(): void {
          scanPathsOf({ mise: `${SCANS_TASK}\ndescription = "naps"\n\n[tasks.build]\nrun = "rolldown"`, },);
        },);
        expect(noTable,).toBeInstanceOf(Error,);
        expect(String(noTable,),).toBe(`Error: mise.toml holds no ${SCANS_TASK} table, so no scan can be read`,);
        expect(noRun,).toBeInstanceOf(Error,);
        expect(String(noRun,),).toBe(`Error: mise.toml's ${SCANS_TASK} table holds no run line, so no scan can be read`,);
      },
    },),
    it({
      name: 'COMPARES NO SCAN\'S PACKAGE-WIDE FINDINGS through toEqual or toStrictEqual, and every scan the '
        + 'source-scans task names reads the package in a case',
      fn: async () => {
        /**
         The package's task file.
         */
        const mise = await readFile(
          join(
            import.meta.dirname,
            '..',
            'mise.toml',
          ),
          'utf8',
        );
        expect(scanPathsOf({ mise, },).includes('scan-direct-comparisons.unit.test.ts',),).toBe(true,);
        expectNoFindings({
          findings: scanComparisons({
            mise,
            files: await readPackageSource(),
            prefix: 'source-scan/',
          },),
        },);
      },
    },),
  ],
},);
