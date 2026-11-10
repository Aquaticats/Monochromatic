/**
 Region-marker guards for substantial modules, which no lint rule enforces.

 `AGENTS.md` rule ST2 asks every logical section of a substantial module to sit
 inside a `//region <purpose>` ... `//endregion <purpose>` pair naming what the
 section does. Nothing checks that mechanically, so each Electron package reads
 its own product modules with `readProductModules` and asserts that
 `unmarkedSubstantialModules` names none of them.

 The scan is textual and deliberately shallow: a module counts as marked once it
 carries one opening marker, because that is the evidence a reader needs that the
 convention was applied at all. Sections left unmarked inside an already-marked
 module stay a review matter.

 @module
 */

import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

//region Thresholds and markers

/**
 Line count above which a module is substantial.

 A module this long no longer fits one screen, so its section boundaries are
 invisible without markers. Every module the 2026-10-17 repository review named
 as unmarked is longer than this, and every marked module of similar length
 already carries its sections.
 */
export const SUBSTANTIAL_MODULE_LINE_COUNT = 200;

/**
 Opening of a named section, written at the start of a line.
 */
const REGION_MARKER_OPENING = '//region';

/**
 Extension of a TypeScript module; other files of a source folder are assets.
 */
const TYPESCRIPT_MODULE_SUFFIX = '.ts';

/**
 Suffix of a module that holds tests rather than product code.
 */
const TEST_MODULE_SUFFIX = '.unit.test.ts';

/**
 Newline that ends a line of a module's text.
 */
const LINE_BREAK = '\n';

//endregion Thresholds and markers

//region Module text

/**
 One module as a guard read it: the file name the guard reports, relative to the
 folder it read, and the full contents that name's checks run over.
 */
export type ModuleText = {
  readonly path: string;
  readonly text: string;
};

/**
 Reads the product modules of one source folder.

 Test modules are left out: a test's sections follow its suite tree rather than
 the behaviour a region name would describe, so a marker there would not say what
 the rule asks it to say.

 @param directory - Folder to read, normally the `src` folder of the package
 that guards itself.

 @returns Every TypeScript module of the folder that is not a test, in file-name
 order.

 @throws When the folder cannot be read, which means the guard was pointed at a
 path this package does not have.

 @example
 ```ts
 const modules = await readProductModules({ directory: '/repo/package/electron-infra/core/src' });
 ```
 */
export async function readProductModules(
  { directory, }: { readonly directory: string; },
): Promise<readonly ModuleText[]> {
  /**
   Directory entries of the folder, read once so a file added mid-read cannot
   change the list.
   */
  const entries = await readdir(
    directory,
    { withFileTypes: true, },
  );

  /**
   File names of the product modules, in the order the guard reads them.
   */
  const names = entries
    .filter(function isProductModule(entry,): boolean {
      if (!entry.isFile())
        return false;
      /**
       Whether the name marks the file as a TypeScript module.
       */
      const typescriptModule = entry.name
        .endsWith(TYPESCRIPT_MODULE_SUFFIX,);
      if (!typescriptModule)
        return false;
      /**
       Whether the name marks the file as a test module.
       */
      const testModule = entry.name
        .endsWith(TEST_MODULE_SUFFIX,);
      return !testModule;
    },)
    .map(function entryName(entry,): string {
      return entry.name;
    },)
    .toSorted();

  return Promise.all(
    names.map(async function readModule(name,): Promise<ModuleText> {
      return {
        path: name,
        text: await readFile(
          join(
            directory,
            name,
          ),
          'utf8',
        ),
      };
    },),
  );
}

/**
 Counts the lines of one module's text.

 A trailing line break ends the last line rather than opening an empty one, so a
 module written with a final newline counts the same as one without.

 @param text - Text to count lines over.

 @returns Number of lines the text occupies.

 @example
 ```ts
 countLines({ text: 'first\nsecond\n' });
 ```
 */
function countLines({ text, }: { readonly text: string; },): number {
  /**
   Lines the text holds, the last one possibly empty.
   */
  const lines = text.split(LINE_BREAK,);
  /**
   One when a final line break opened an empty last line, zero otherwise.
   */
  const trailingEmptyLine = (lines.at(-1) === '') ? 1 : 0;
  return lines.length - trailingEmptyLine;
}

/**
 Names the substantial modules of one folder that carry no region marker.

 @param lineCountThreshold - Modules longer than this are substantial; a caller
 passes `SUBSTANTIAL_MODULE_LINE_COUNT` unless it checks a fixture.

 @param modules - Modules to inspect, as `readProductModules` returned them.

 @returns File names of the unmarked substantial modules, in the order given, so
 an empty array is the passing case.

 @example
 ```ts
 unmarkedSubstantialModules({ lineCountThreshold: SUBSTANTIAL_MODULE_LINE_COUNT, modules: [] });
 ```
 */
export function unmarkedSubstantialModules(
  {
    lineCountThreshold,
    modules,
  }: {
    readonly lineCountThreshold: number;
    readonly modules: readonly ModuleText[];
  },
): readonly string[] {
  return modules
    .filter(function substantiallyLong(module,): boolean {
      return countLines({ text: module.text, },) > lineCountThreshold;
    },)
    .filter(function unmarked(module,): boolean {
      /**
       Whether the module names at least one of its sections.
       */
      const marked = module.text
        .includes(REGION_MARKER_OPENING,);
      return !marked;
    },)
    .map(function modulePath(module,): string {
      return module.path;
    },);
}

//endregion Module text
