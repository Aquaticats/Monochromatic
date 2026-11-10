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
// Where the markers are read from, the two marker words, and the length beyond
// which a module is expected to hold more than one section.

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
 Word a line writes to open a region, read after the line's indentation.
 */
const REGION_OPENER = '//region';

/**
 Word a line writes to close the innermost open region.
 */
const REGION_CLOSER = '//endregion';

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
 What one line of module text writes: a region opener, a region closer, or
 neither, so a reader switches on the kind rather than testing for absence.
 */
type LineMarker = {
  /**
   Marker the line writes.
   */
  readonly kind: 'opener' | 'closer' | 'none';
  /**
   Label after the marker word, empty when the line names no purpose or writes
   no marker.
   */
  readonly purpose: string;
};

/**
 Whether one line writes a given marker word, and the label written after it.
 */
type MarkerReading = {
  /**
   Whether the line writes this marker at all.
   */
  readonly isMarker: boolean;
  /**
   Label after the marker word, empty when the line names no purpose.
   */
  readonly purpose: string;
};

/**
 Reading of a line that writes other text, returned rather than minted per line.
 */
const OTHER_TEXT: MarkerReading = { isMarker: false, purpose: '', };

/**
 Reading of a line that writes no region marker.
 */
const NO_MARKER: LineMarker = { kind: 'none', purpose: '', };

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
 Reads one marker word off a line.

 The word counts as the line's marker only when the label after it is empty or
 separated by whitespace, so a comment that merely begins with the word, as
 `//regions are read here` does, stays ordinary comment text.

 @param line - one line of module text, as written
 @param marker - marker word to read, `//region` or `//endregion`
 @returns whether the line writes this marker, and the label after it

 @example
 ```ts
 readMarker({ line: '  //region Constants', marker: REGION_OPENER });
 ```
 */
function readMarker({ line, marker, }: {
  readonly line: string;
  readonly marker: string;
},): MarkerReading {
  /**
   Line with its indentation removed, so a marker nested in a block still
   matches.
   */
  const trimmed = line.trimStart();
  if (!trimmed.startsWith(marker,)) {
    return OTHER_TEXT;
  }
  /**
   Text after the marker word, which is the label when it starts with
   whitespace.
   */
  const rest = trimmed.slice(marker.length,);
  if ((rest !== '') && (rest.trimStart() === rest)) {
    return OTHER_TEXT;
  }
  return { isMarker: true, purpose: rest.trim(), };
}

/**
 Reads the marker one line writes.

 @param line - one line of module text, as written
 @returns the opener or closer the line writes, or `none` for every other line

 @example
 ```ts
 markerOn('//endregion Constants');
 ```
 */
function markerOn(line: string,): LineMarker {
  /**
   Reading of this line as an opener.
   */
  const opener = readMarker({ line, marker: REGION_OPENER, },);
  if (opener.isMarker) {
    return { kind: 'opener', purpose: opener.purpose, };
  }
  /**
   Reading of this line as a closer.
   */
  const closer = readMarker({ line, marker: REGION_CLOSER, },);
  if (closer.isMarker) {
    return { kind: 'closer', purpose: closer.purpose, };
  }
  return NO_MARKER;
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
    if (marker.kind === 'none') {
      continue;
    }
    /**
     1-based line number the marker sits on.
     */
    const lineNumber = index + 1;
    /**
     Marker word the line writes, named in every message below.
     */
    const word = marker.kind === 'opener' ? REGION_OPENER : REGION_CLOSER;
    if (marker.purpose === '') {
      problems.push(`${name}:${String(lineNumber,)} ${word} names no purpose`,);
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
      problems.push(`${name}:${String(lineNumber,)} ${word} ${marker.purpose} closes no region`,);
      continue;
    }
    if (opened.purpose !== marker.purpose) {
      problems.push(`${name}:${String(opened.line,)} ${REGION_OPENER} ${opened.purpose} is closed at line ${String(lineNumber,)} by ${word} ${marker.purpose}`,);
    }
  }
  for (const leftover of open) {
    problems.push(`${name}:${String(leftover.line,)} ${REGION_OPENER} ${leftover.purpose} is never closed`,);
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
   Names of the entries that are modules, sorted so a failure reads the same way
   twice.
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
                /**
                 Whether the module is long enough to hold several sections.
                 */
                const isLong = lineCount(module.text,) > LENGTH_REQUIRING_A_REGION;
                /**
                 Whether any of the module's lines opens a region.
                 */
                const opensRegion = module.text
                  .split('\n',)
                  .some(function isOpener(line: string,): boolean {
                    return markerOn(line,).kind === 'opener';
                  },);
                if (!isLong) {
                  return false;
                }
                return !opensRegion;
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
             Module text whose markers state no purpose for their section.
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
        it({
          name: 'ignores comment text that merely begins with a marker word',
          fn: async () => {
            expect(markerOn('//regions are read here',),).toEqual({ kind: 'none', purpose: '', },);
            expect(markerOn('//endregion and then some',),).toEqual({
              kind: 'closer',
              purpose: 'and then some',
            },);
          },
        }),
      ],
    }),
  ],
},);

//endregion The guard
