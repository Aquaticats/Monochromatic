/**
 Guards that every module in this package marks its logical sections with a
 matched `//region`/`//endregion` pair.

 `AGENTS.md` rule `ST2` asks for those markers and no lint rule enforces it, so a
 closing marker whose label drifted away from the opening one survives `lint` and
 every unit test: `src/testing.ts` carried three shifted closings, and nine
 modules over a hundred lines carried no marker at all.

 The rules read here are the ones an editor reads: markers nest, a closer repeats
 the label of the marker it closes, each label names a purpose, and a module long
 enough to hold several sections opens at least one region.

 @module
 */

import type { Dirent, } from 'node:fs';

import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

//region Vocabulary
// Where the markers are read from, what one marker line looks like, and the
// length beyond which a module is expected to hold more than one section.

/**
 Directory holding this package's modules, which is this test file's own folder.
 */
const SOURCE_DIRECTORY = import.meta.dirname;

/**
 Suffix naming a module scanned here, so a stray editor backup or declaration
 file needs no separate exclusion unless it ends this way too.
 */
const SOURCE_FILE_SUFFIX = '.ts';

/**
 One `//region` line, with the label after the marker captured as `purpose`.
 */
const REGION_OPENER = /^\s*\/\/region(?:\s+(?<purpose>.*?))?\s*$/u;

/**
 One `//endregion` line, with the label after the marker captured as `purpose`.
 */
const REGION_CLOSER = /^\s*\/\/endregion(?:\s+(?<purpose>.*?))?\s*$/u;

/**
 Length past which a module is expected to open a region.

 A hundred lines is where this package's own modules stop fitting one concern:
 the daemon, the install steps and each test suite all cross it, while the
 constants and state modules stay well under and read as one block.
 */
const LENGTH_REQUIRING_A_REGION = 100;

//endregion Vocabulary

//region Reading one module's markers
// A module's text in, every pairing fault it holds out, with the module name and
// line number a reader needs to find it.

/**
 One module of this package, named so a fault reports where it lives.
 */
type SourceModule = {
  /**
   File name within {@link SOURCE_DIRECTORY}.
   */
  readonly name: string;
  /**
   Whole file text.
   */
  readonly text: string;
};

/**
 Which marker a line writes, and the label it writes after it.
 */
type Marker = {
  /**
   Marker that opens a region or closes the innermost open one.
   */
  readonly kind: 'opener' | 'closer';
  /**
   Label after the marker, empty when the line writes none.
   */
  readonly purpose: string;
};

/**
 A region opened on a line and not yet closed.
 */
type OpenRegion = {
  /**
   1-based line the opener sits on.
   */
  readonly line: number;
  /**
   Label the opener names, which its closer must repeat.
   */
  readonly purpose: string;
};

/**
 Reads one marker from a line of module text.

 @param line - one line of source text, as written

 @returns the marker the line writes, or `undefined` for every other line

 @example
 ```ts
 markerOn('//region Constants');
 ```
 */
function markerOn(line: string,): Marker | undefined {
  /**
   Opener match, when the line writes one.
   */
  const opener = REGION_OPENER.exec(line,);
  if (opener !== null) {
    return { kind: 'opener', purpose: (opener.groups?.['purpose'] ?? '').trim(), };
  }
  /**
   Closer match, when the line writes one.
   */
  const closer = REGION_CLOSER.exec(line,);
  if (closer !== null) {
    return { kind: 'closer', purpose: (closer.groups?.['purpose'] ?? '').trim(), };
  }
  return undefined;
}

/**
 Counts the lines a module holds, as `wc --lines` does.

 @param text - whole file text

 @returns number of newline characters in the text

 @example
 ```ts
 lineCount('a\nb\n');
 ```
 */
function lineCount(text: string,): number {
  return text.split('\n',).length - 1;
}

/**
 Reports every pairing fault in one module's markers.

 @param module - module name and text to read

 @returns one message per fault, empty when every marker pairs

 @example
 ```ts
 markerProblems({ name: 'nvim.ts', text });
 ```
 */
function markerProblems({ name, text, }: SourceModule,): readonly string[] {
  /**
   Faults found so far, each naming the module and the line.
   */
  const problems: string[] = [];
  /**
   Regions opened and not yet closed, innermost last.
   */
  const open: OpenRegion[] = [];
  /**
   Lines the region markers are read from.
   */
  const lines = text.split('\n',);
  for (const [index, line,] of lines.entries()) {
    /**
     Marker this line writes, if any.
     */
    const marker = markerOn(line,);
    if (marker === undefined) {
      continue;
    }
    /**
     1-based line number the marker sits on.
     */
    const lineNumber = index + 1;
    if (marker.purpose === '') {
      problems.push(`${name}:${String(lineNumber,)} //${marker.kind === 'opener' ? 'region' : 'endregion'} names no purpose`,);
      continue;
    }
    if (marker.kind === 'opener') {
      open.push({ line: lineNumber, purpose: marker.purpose, },);
      continue;
    }
    /**
     Region this closer must close, innermost first.
     */
    const opened = open.pop();
    if (opened === undefined) {
      problems.push(`${name}:${String(lineNumber,)} //endregion ${marker.purpose} closes no region`,);
      continue;
    }
    if (opened.purpose !== marker.purpose) {
      problems.push(`${name}:${String(opened.line,)} //region ${opened.purpose} is closed at line ${String(lineNumber,)} by //endregion ${marker.purpose}`,);
    }
  }
  for (const leftover of open) {
    problems.push(`${name}:${String(leftover.line,)} //region ${leftover.purpose} is never closed`,);
  }
  return problems;
}

/**
 Reads every module in this package's source folder.

 @returns each module's name and text, by file name

 @example
 ```ts
 const modules = await sourceModules();
 ```
 */
async function sourceModules(): Promise<readonly SourceModule[]> {
  /**
   Every entry of the source folder, read with its kind.
   */
  const entries = await readdir(SOURCE_DIRECTORY, { withFileTypes: true, },);
  /**
   Names of the entries that are modules, sorted so a failure reads the same
   way twice.
   */
  const names = entries
    .filter(function isModule(entry: Dirent,): boolean {
      return entry.isFile() && entry.name.endsWith(SOURCE_FILE_SUFFIX,);
    },)
    .map(function fileName(entry: Dirent,): string {
      return entry.name;
    },)
    .toSorted(function byName(left: string, right: string,): number {
      return left.localeCompare(right,);
    },);
  return Promise.all(names.map(async function read(name: string,): Promise<SourceModule> {
    return { name, text: await readFile(join(SOURCE_DIRECTORY, name,), 'utf8',), };
  },),);
}

//endregion Reading one module's markers

//region The guard
// Every module in the folder, then one case per rule the markers follow.

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'region markers',
      children: [
        it({
          name: 'close each region with the label it opened with',
          fn: async () => {
            /**
             Every module in the folder.
             */
            const modules = await sourceModules();
            /**
             Faults across the folder, empty when every region pairs.
             */
            const problems = modules.flatMap(function ofModule(module: SourceModule,): readonly string[] {
              return markerProblems(module,);
            },);
            expect(problems,).toEqual([],);
          },
        }),
        it({
          name: 'open a region in every module past the length threshold',
          fn: async () => {
            /**
             Every module in the folder.
             */
            const modules = await sourceModules();
            /**
             Modules long enough to hold several sections that open no region.
             */
            const unmarked = modules
              .filter(function isLongAndUnmarked(module: SourceModule,): boolean {
                return lineCount(module.text,) > LENGTH_REQUIRING_A_REGION
                  && !module.text.split('\n',).some(function opensRegion(line: string,): boolean {
                    return markerOn(line,)?.kind === 'opener';
                  },);
              },)
              .map(function moduleName(module: SourceModule,): string {
                return module.name;
              },);
            expect(unmarked,).toEqual([],);
          },
        }),
        it({
          name: 'reports a shifted label and a region left open',
          fn: async () => {
            /**
             Module text whose closing labels drifted away from its opening ones,
             the fault this guard was written after.
             */
            const shifted = [
              '//region Keys',
              '//endregion Neovim',
              '//region evdev',
              '//endregion Keys',
              '//region Orphan',
            ].join('\n',);
            /**
             Module text whose markers all pair.
             */
            const paired = [
              '//region Keys',
              'const a = 1;',
              '//endregion Keys',
            ].join('\n',);
            expect(markerProblems({ name: 'shifted.ts', text: shifted, },),).toEqual([
              'shifted.ts:1 //region Keys is closed at line 2 by //endregion Neovim',
              'shifted.ts:3 //region evdev is closed at line 4 by //endregion Keys',
              'shifted.ts:5 //region Orphan is never closed',
            ],);
            expect(markerProblems({ name: 'paired.ts', text: paired, },),).toEqual([],);
          },
        }),
        it({
          name: 'reports a marker that names no purpose',
          fn: async () => {
            /**
             Module text whose opener states no purpose for its section.
             */
            const anonymous = [
              '//region',
              'const a = 1;',
              '//endregion',
            ].join('\n',);
            expect(markerProblems({ name: 'anonymous.ts', text: anonymous, },),).toEqual([
              'anonymous.ts:1 //region names no purpose',
              'anonymous.ts:3 //endregion names no purpose',
            ],);
          },
        }),
      ],
    }),
  ],
},);

//endregion The guard
