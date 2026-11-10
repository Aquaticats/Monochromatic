/**
 Guards on this package's module shape, which no lint rule enforces.

 `AGENTS.md` rule ST2 asks every logical section of a substantial module to sit inside a
 `//region <purpose>` ... `//endregion <purpose>` pair naming what the section does, and the
 repository review checklist flags a module that carries none. Nothing reads the markers
 mechanically, so each product module of this folder is read here and checked directly.

 A module of at least `SUBSTANTIAL_MODULE_LINE_COUNT` lines must name at least one of its
 sections, and in every module a closing marker must name the region it closes, so a folded
 section cannot end somewhere its name does not claim.

 Test modules are left out: a test's sections follow its suite tree rather than the behaviour a
 region name describes, which is the split `package/electron-infra/core` reads its own modules
 by. Keeping a substantial module unmarked means changing this guard on purpose, which is the
 point.

 @module
 */

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

//region Fixtures

/**
 Folder holding this package's product modules, which is this test's own folder.
 */
const SOURCE_DIRECTORY = import.meta.dirname;

/**
 Shortest module whose sections must be named, in lines.

 A module this long no longer fits one screen, so its section boundaries are invisible until a
 reader folds it. Every module of the 2026-10-17 repository review's unmarked units that this
 package shares a length with already carries markers.
 */
const SUBSTANTIAL_MODULE_LINE_COUNT = 200;

/**
 Extension of the files this guard reads; other files of a source folder are assets.
 */
const SOURCE_SUFFIX = '.ts';

/**
 Suffix of a module holding tests rather than product code.
 */
const TEST_MODULE_SUFFIX = '.unit.test.ts';

/**
 Opening marker, whose purpose follows after a space.
 */
const REGION_OPENER = '//region';

/**
 Closing marker, which names the region it closes.
 */
const REGION_CLOSER = '//endregion';

/**
 Separator between a marker and its purpose.
 */
const MARKER_LABEL_SEPARATOR = ' ';

/**
 Newline that ends a line of a module's text.
 */
const LINE_BREAK = '\n';

/**
 Substantial product text with no section marker, which the guard must report.
 */
const UNMARKED_SAMPLE = [
  '/** A product module. */',
  '',
  'export const sample = 1;',
].join(LINE_BREAK,);

/**
 Product text whose closing marker names another section, which the guard must report.
 */
const MISLABELLED_SAMPLE = [
  '//region Reading',
  'export const sample = 1;',
  '//endregion Writing',
].join(LINE_BREAK,);

/**
 Product text whose region never closes, which the guard must report.
 */
const UNCLOSED_SAMPLE = [
  '//region Reading',
  'export const sample = 1;',
].join(LINE_BREAK,);

/**
 Product text whose opening marker names nothing, which the guard must report.
 */
const UNLABELLED_SAMPLE = [
  '//region',
  'export const sample = 1;',
  '//endregion',
].join(LINE_BREAK,);

/**
 Product text with one named pair, which the guard must accept.
 */
const MARKED_SAMPLE = [
  '//region Reading',
  'export const sample = 1;',
  '//endregion Reading',
].join(LINE_BREAK,);

/**
 Product text whose only marker-shaped line is a word that merely starts with the marker, which
 the guard must read as prose rather than as a named section.
 */
const REGIONAL_SAMPLE = [
  '//regional plan for the module',
  'export const sample = 1;',
].join(LINE_BREAK,);

//endregion Fixtures

//region Scanning

/**
 What one scan reads: a module's text and the length at which a module must name its sections.
 */
type RegionScanOptions = {
  /**
   Text of one product module.
   */
  readonly text: string;

  /**
   Shortest module, in lines, whose sections must be named.
   */
  readonly minimumLines: number;
};

/**
 What one line turned out to be, plus the purpose that follows a region marker.
 */
type MarkerLine = {
  readonly kind: 'marker' | 'other';
  readonly label: string;
};

/**
 Lists the product modules below a folder, skipping test modules.

 @param directory - Folder to read, recursively.

 @returns Paths of the product modules it holds, from `readdir` order.
 */
async function productModules(directory: string,): Promise<readonly string[]> {
  /**
   Directory entries of the folder, read once so a file added mid-read cannot change the list.
   */
  const entries = await readdir(
    directory,
    { withFileTypes: true, },
  );
  /**
   Paths each entry contributed, subfolders expanded in place.
   */
  const contributed = await Promise.all(entries.map(async function readEntry(entry,) {
    /**
     Path of the entry, whether file or folder.
     */
    const path = join(
      directory,
      entry.name,
    );

    if (entry.isDirectory())
      return await productModules(path,);

    if ((!entry.name.endsWith(SOURCE_SUFFIX,)) || (entry.name.endsWith(TEST_MODULE_SUFFIX,)))
      return [];

    return [path,];
  },),);

  return contributed.flat();
}

/**
 Reads a region marker line's purpose.

 @param options - Trimmed line and the marker to look for.

 @returns The marker's kind with its purpose, empty when the marker carries none.

 @example
 ```ts
 markerLabel({ line: '//region Reading', marker: '//region' });
 ```
 */
function markerLabel(options: {
  readonly line: string;
  readonly marker: string;
},): MarkerLine {
  if (!options.line.startsWith(options.marker,))
    return {
      kind: 'other',
      label: '',
    };

  /**
   Text after the marker on this line.
   */
  const rest = options.line.slice(options.marker.length,);

  if ((rest !== '') && (!rest.startsWith(MARKER_LABEL_SEPARATOR,)))
    return {
      kind: 'other',
      label: '',
    };

  return {
    kind: 'marker',
    label: rest.trim(),
  };
}

/**
 Reports the line a closing marker sits on, if the line closes a region.

 @param line - Trimmed line of a module's text.

 @returns The closing marker with its purpose, or the other-line reading.
 */
function regionCloser(line: string,): MarkerLine {
  return markerLabel({
    line,
    marker: REGION_CLOSER,
  },);
}

/**
 Reports what one module's text gets wrong about its region markers.

 @param options - Module text and the length at which its sections must be named.

 @returns One message per problem, empty when the text satisfies the rule.
 */
function regionFindings(options: RegionScanOptions,): readonly string[] {
  /**
   Messages the scan collected, in line order then unclosed-region order.
   */
  const findings: string[] = [];
  /**
   Purposes of the regions open at the line being read, innermost last.
   */
  const openRegions: string[] = [];
  /**
   Lines of the module, with the empty line a final line break opens dropped.
   */
  const lines = options.text.split(LINE_BREAK,);
  /**
   Whether the module names any of its sections.
   */
  let marked = false;

  if (lines.at(-1,) === '')
    lines.pop();

  for (const [
    index,
    line,
  ] of lines.entries()) {
    /**
     Trimmed line, which is where a marker is written.
     */
    const trimmed = line.trim();
    /**
     Line number the message names, 1-based.
     */
    const lineNumber = String(index + 1,);
    /**
     Opening marker reading of this line.
     */
    const opening = markerLabel({
      line: trimmed,
      marker: REGION_OPENER,
    },);

    if (opening.kind === 'marker') {
      marked = true;

      if (opening.label === '')
        findings.push(`line ${lineNumber}: region marker names no section`,);

      openRegions.push(opening.label,);
      continue;
    }

    /**
     Closing marker reading of this line.
     */
    const closing = regionCloser(trimmed,);

    if (closing.kind === 'other')
      continue;

    /**
     Purpose of the region this line closes, absent when none is open.
     */
    const closed = openRegions.pop();

    if (closed === undefined) {
      findings.push(`line ${lineNumber}: ${REGION_CLOSER} closes no region`,);
      continue;
    }

    if (closing.label !== closed) {
      findings.push(
        `line ${lineNumber}: ${REGION_CLOSER} names "${closing.label}" but closes "${closed}"`,
      );
    }
  }

  for (const purpose of openRegions)
    findings.push(`region "${purpose}" is never closed`,);

  if ((!marked) && (lines.length >= options.minimumLines))
    findings.push(`no region marker in ${String(lines.length)} lines`,);

  return findings;
}

//endregion Scanning

//region Guard

await describe({
  name: 'region markers',
  children: [
    //region The scan reports what the rule forbids

    it({
      name: 'reports a substantial module with no region marker',
      fn: async () => {
        expect(
          regionFindings({
            text: UNMARKED_SAMPLE,
            minimumLines: UNMARKED_SAMPLE.split(LINE_BREAK,).length,
          },),
        ).toEqual([
          `no region marker in ${String(UNMARKED_SAMPLE.split(LINE_BREAK,).length)} lines`,
        ],);
      },
    },),
    it({
      name: 'reports a closing marker that names another section',
      fn: async () => {
        expect(
          regionFindings({
            text: MISLABELLED_SAMPLE,
            minimumLines: Number.POSITIVE_INFINITY,
          },),
        ).toEqual(['line 3: //endregion names "Writing" but closes "Reading"',],);
      },
    },),
    it({
      name: 'reports a region that never closes',
      fn: async () => {
        expect(
          regionFindings({
            text: UNCLOSED_SAMPLE,
            minimumLines: Number.POSITIVE_INFINITY,
          },),
        ).toEqual(['region "Reading" is never closed',],);
      },
    },),
    it({
      name: 'reports a region marker that names no section',
      fn: async () => {
        expect(
          regionFindings({
            text: UNLABELLED_SAMPLE,
            minimumLines: Number.POSITIVE_INFINITY,
          },),
        ).toEqual(['line 1: region marker names no section',],);
      },
    },),
    it({
      name: 'accepts a marked module and a short unmarked one',
      fn: async () => {
        expect(
          regionFindings({
            text: MARKED_SAMPLE,
            minimumLines: 1,
          },),
        ).toEqual([],);
        expect(
          regionFindings({
            text: UNMARKED_SAMPLE,
            minimumLines: Number.POSITIVE_INFINITY,
          },),
        ).toEqual([],);
      },
    },),

    //endregion The scan reports what the rule forbids

    //region This package's product modules

    it({
      name: 'reads a word that only starts with the marker as prose',
      fn: async () => {
        expect(
          regionFindings({
            text: REGIONAL_SAMPLE,
            minimumLines: Number.POSITIVE_INFINITY,
          },),
        ).toEqual([],);
        expect(
          regionFindings({
            text: REGIONAL_SAMPLE,
            minimumLines: 1,
          },),
        ).toEqual([
          `no region marker in ${String(REGIONAL_SAMPLE.split(LINE_BREAK,).length)} lines`,
        ],);
      },
    },),
    it({
      name: `names a region in every product module of at least ${String(SUBSTANTIAL_MODULE_LINE_COUNT)} lines`,
      fn: async () => {
        /**
         Product modules of this package.
         */
        const modules = await productModules(SOURCE_DIRECTORY,);

        expect(modules.length,).toBeGreaterThan(0,);

        /**
         Problems found, named by the module's path relative to this folder.
         */
        const findings = (await Promise.all(modules.map(async function readModule(path,) {
          /**
           Text of one product module.
           */
          const text = await readFile(
            path,
            'utf8',
          );

          return regionFindings({
            text,
            minimumLines: SUBSTANTIAL_MODULE_LINE_COUNT,
          },).map(function toMessage(finding,): string {
            return `${path.replace(`${SOURCE_DIRECTORY}/`, '',)}: ${finding}`;
          },);
        },),)).flat();

        expect(findings,).toEqual([],);
      },
    },),

    //endregion This package's product modules
  ],
},);

//endregion Guard
