/**
 Guards the region markers `AGENTS.md` rule ST2 requires in substantial source files, which no lint rule enforces.

 A product module of this package that reaches {@link REGION_MINIMUM_LINES} lines must split its text into labelled
 `//region`/`//endregion` sections, so a reader finds argument parsing, execution and helpers by folding rather than by
 reading the whole file. The same scan runs over the fixture texts below, so the guard is shown to report an unmarked
 file, a missing label and a label that does not match, and not only to pass the files that already carry markers.

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
 Shortest file whose sections must be labelled, as the repository-wide review that filed this defect measured.
 */
const REGION_MINIMUM_LINES = 150;

/**
 Suffix of a test file, which this guard skips: a test groups its cases in `describe` children instead.
 */
const TEST_FILE_SUFFIX = '.unit.test.ts';

/**
 Extension of the files this guard reads.
 */
const SOURCE_SUFFIX = '.ts';

/**
 Opening marker, which the label follows after a space.
 */
const REGION_OPENER = '//region';

/**
 Closing marker, which carries the label of the region it closes.
 */
const REGION_CLOSER = '//endregion';

/**
 Separator between a marker and its label.
 */
const MARKER_LABEL_SEPARATOR = ' ';

/**
 Substantial product text with no section marker, which the guard must report.
 */
const UNMARKED_SAMPLE = [
  '/** A product module. */',
  '',
  'export const sample = 1;',
].join('\n',);

/**
 Product text whose closing marker names another region, which the guard must report.
 */
const MISLABELLED_SAMPLE = [
  '//region Reading',
  'export const sample = 1;',
  '//endregion Writing',
].join('\n',);

/**
 Product text whose region never closes, which the guard must report.
 */
const UNCLOSED_SAMPLE = [
  '//region Reading',
  'export const sample = 1;',
].join('\n',);

/**
 Product text whose region marker explains nothing, which the guard must report.
 */
const UNLABELLED_SAMPLE = [
  '//region',
  'export const sample = 1;',
  '//endregion',
].join('\n',);

/**
 Product text with one labelled pair, which the guard must accept.
 */
const MARKED_SAMPLE = [
  '//region Reading',
  'export const sample = 1;',
  '//endregion Reading',
].join('\n',);

//endregion Fixtures

//region Scanning

/**
 What one scan reads: the module text and the length at which a module must mark its sections.
 */
type RegionScanOptions = {
  /**
   Text of one product module.
   */
  readonly text: string;

  /**
   Shortest module, in lines, whose sections must be marked.
   */
  readonly minimumLines: number;
};

/**
 Lists the product modules below a folder, skipping test files.
 
 @param directory - Folder to read, recursively.
 
 @returns Paths of the product modules it holds.
 */
async function productModules(directory: string,): Promise<readonly string[]> {
  const entries = await readdir(
    directory,
    { withFileTypes: true, },
  );
  const modules = await Promise.all(entries.map(async function readEntry(entry,) {
    const path = join(directory, entry.name,);

    if (entry.isDirectory())
      return await productModules(path,);

    if ((!entry.name.endsWith(SOURCE_SUFFIX,)) || (entry.name.endsWith(TEST_FILE_SUFFIX,)))
      return [];

    return [path,];
  },),);

  return modules.flat();
}

/**
 What one line turned out to be, plus the label that follows a region marker.
 */
type MarkerLine = {
  readonly kind: 'marker' | 'other';
  readonly label: string;
};

/**
 Reads a region marker line's label.
 
 @param options - Trimmed line and the marker to look for.
 
 @returns The marker with its label, empty when the marker carries none, or the other-line reading.
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
 Reports what one module text gets wrong about its region markers.
 
 @param options - Module text and the length at which its sections must be marked.
 
 @returns One message per problem, empty when the text satisfies the rule.
 */
function regionFindings(options: RegionScanOptions,): readonly string[] {
  const findings: string[] = [];
  const open: string[] = [];
  const lines = options.text.split('\n',);
  let marked = false;

  if (lines.at(-1,) === '')
    lines.pop();

  for (const [
    index,
    line,
  ] of lines.entries()) {
    const trimmed = line.trim();
    const lineNumber = String(index + 1,);
    const opens = markerLabel({
      line: trimmed,
      marker: REGION_OPENER,
    },);

    if (opens.kind === 'marker') {
      marked = true;

      if (opens.label === '')
        findings.push(`line ${lineNumber}: region marker gives no label`,);

      open.push(opens.label,);
      continue;
    }

    const closes = markerLabel({
      line: trimmed,
      marker: REGION_CLOSER,
    },);

    if (closes.kind === 'other')
      continue;

    const { label, } = closes;
    const opening = open.pop();

    if (opening === undefined) {
      findings.push(`line ${lineNumber}: ${REGION_CLOSER} closes no region`,);
      continue;
    }

    if (label !== opening)
      findings.push(`line ${lineNumber}: ${REGION_CLOSER} names "${label}" but closes "${opening}"`,);
  }

  for (const label of open)
    findings.push(`region "${label}" is never closed`,);

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
        const findings = regionFindings({
          text: UNMARKED_SAMPLE,
          minimumLines: UNMARKED_SAMPLE.split('\n',).length,
        },);

        expect(findings,).toEqual([
          `no region marker in ${String(UNMARKED_SAMPLE.split('\n',).length)} lines`,
        ],);
      },
    },),
    it({
      name: 'reports a closing marker that names another region',
      fn: async () => {
        const findings = regionFindings({
          text: MISLABELLED_SAMPLE,
          minimumLines: Number.POSITIVE_INFINITY,
        },);

        expect(findings,).toHaveLength(1,);
        expect(findings[0],).toContain('names "Writing" but closes "Reading"',);
      },
    },),
    it({
      name: 'reports a region that never closes',
      fn: async () => {
        const findings = regionFindings({
          text: UNCLOSED_SAMPLE,
          minimumLines: Number.POSITIVE_INFINITY,
        },);

        expect(findings,).toEqual(['region "Reading" is never closed',],);
      },
    },),
    it({
      name: 'reports a region marker with no label',
      fn: async () => {
        const findings = regionFindings({
          text: UNLABELLED_SAMPLE,
          minimumLines: Number.POSITIVE_INFINITY,
        },);

        expect(findings,).toEqual(['line 1: region marker gives no label',],);
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
      name: `every product module of at least ${String(REGION_MINIMUM_LINES)} lines carries region markers`,
      fn: async () => {
        const modules = await productModules(SOURCE_DIRECTORY,);
        expect(modules.length,).toBeGreaterThan(0,);
        const findings = (await Promise.all(modules.map(async function readModule(path,) {
          const text = await readFile(path, 'utf8',);
          return regionFindings({
            text,
            minimumLines: REGION_MINIMUM_LINES,
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
