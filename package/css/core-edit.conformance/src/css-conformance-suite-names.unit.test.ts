/**
 Guards that every suite this package's CSS conformance corpus registers carries
 a name, the way the sibling
 `@monochromatic-dev/jsonc-edit-core-ts.conformance` corpus names its root.

 Why: an unnamed suite adds no name segment to the harness log, and this package
 runs the corpus alone, so its log lines carry only the innermost case name and
 nothing says which corpus produced them. The corpus root is the level that lost
 its name: `name: ''` there made the whole run report `[valid corpus]`,
 `[context amplification]` and `[invalid corpus]` with no CSS corpus above them.
 This corpus naming itself is a deliberate departure from the harness allowance
 for an empty top-level name, which assumes a runner that prints the test file;
 this package's run prints its task command line instead, so the corpus name is
 the only thing a log line from the run carries above its cases.

 The scan reads the corpus test file's text rather than its descriptors, because
 the descriptors run the tests when they are awaited. A suite is read when the
 line opening its options object ends with {@link SUITE_OPENER_TAIL} and the
 line after it declares the name, which is where this package's corpus writes
 it; a restructure that moves a name elsewhere is caught by the assertion that
 the scan read a name for every opener.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

//region Fixtures

/**
 Corpus test whose suite names this guard reads; it is the only file in this
 package that registers suites.
 */
const CORPUS_TEST_FILE_NAME = 'css.conformance.unit.test.ts';

/**
 Tail of the line that opens a suite's options object.
 */
const SUITE_OPENER_TAIL = 'describe({';

/**
 Head of the line that carries a suite's name inside its options object.
 */
const NAME_FIELD_HEAD = 'name:';

/**
 Separator that ends the name field.
 */
const FIELD_SEPARATOR = ',';

/**
 Quote character wrapping every suite name across the corpus's suite options.
 */
const NAME_QUOTE = "'";

/**
 Options line of a suite whose name is the empty string, the shape the guard
 reports.
 */
const UNNAMED_NAME_LINE = `  ${NAME_FIELD_HEAD} ${NAME_QUOTE}${NAME_QUOTE}${FIELD_SEPARATOR}`;

/**
 Lines opening the suite tagged by {@link UNNAMED_NAME_LINE}.
 */
const UNNAMED_SUITE_LINES: readonly string[] = [
  `await ${SUITE_OPENER_TAIL}`,
  UNNAMED_NAME_LINE,
];

/**
 Options line of a suite that names itself, the shape the guard accepts.
 */
const NAMED_NAME_LINE = `  ${NAME_FIELD_HEAD} ${NAME_QUOTE}the corpus${NAME_QUOTE}${FIELD_SEPARATOR}`;

/**
 Lines opening the suite tagged by {@link NAMED_NAME_LINE}.
 */
const NAMED_SUITE_LINES: readonly string[] = [
  `await ${SUITE_OPENER_TAIL}`,
  NAMED_NAME_LINE,
];

/**
 Options line declaring a field other than the name, which declares no name.
 */
const OTHER_FIELD_LINE = '  children: [],';

//endregion Fixtures

//region Helpers

/**
 Suites and names one test file's text declares.
 */
type SuiteNameScan = {
  /**
   Number of `describe` calls whose options object opens on a line.
   */
  readonly openers: number;
  /**
   Name of each suite the scan read, in source order.
   */
  readonly names: readonly string[];
};

/**
 Reads the name a suite's options line declares.

 @param line - line of test source that follows a suite opener

 @returns the declared name unwrapped from its quotes, or no entry when the
 line declares no name field

 @example
 ```ts
 parsedName("  name: 'the corpus',") // ['the corpus']
 ```
 */
function parsedName(line: string,): readonly string[] {
  /**
   Line trimmed of its indentation.
   */
  const trimmed = line.trim();
  if (!trimmed.startsWith(`${NAME_FIELD_HEAD} `))
    return [];
  /**
   Declared value after the field head, with its trailing separator removed.
   */
  const declared = trimmed.slice(`${NAME_FIELD_HEAD} `.length,);
  /**
   Declared value with the separator that may end the field dropped.
   */
  const unseparated = declared.endsWith(FIELD_SEPARATOR,)
    ? declared.slice(
      0,
      -FIELD_SEPARATOR.length,
    )
    : declared;
  if (unseparated.startsWith(NAME_QUOTE,) && unseparated.endsWith(NAME_QUOTE,))
    return [unseparated.slice(
      NAME_QUOTE.length,
      -NAME_QUOTE.length,
    ),];
  return [unseparated,];
}

/**
 Reads every suite name out of one test file's lines.

 @param lines - test source split into lines

 @returns the suites whose options object opened, and the names read from them
 */
function scanSuiteNames(lines: readonly string[],): SuiteNameScan {
  return lines.reduce<SuiteNameScan>(
    function readSuite(scan, line, index,) {
      if (!line.trimEnd()
        .endsWith(SUITE_OPENER_TAIL,))
        return scan;
      /**
       Line after the opener, the only one a name is read from.
       */
      const nameLine = lines[index + 1];
      if (nameLine === undefined)
        return {
          openers: scan.openers + 1,
          names: scan.names,
        };
      return {
        openers: scan.openers + 1,
        names: [...scan.names, ...parsedName(nameLine,),],
      };
    },
    {
      openers: 0,
      names: [],
    },
  );
}

//endregion Helpers

await describe({
  name: 'the CSS conformance corpus suite names',
  children: [
    it({
      name: 'reads the empty name of an unnamed suite options line',
      fn: async () => {
        expect(parsedName(UNNAMED_NAME_LINE,),).toEqual(['',],);
      },
    },),
    it({
      name: 'reads the name of a named suite options line',
      fn: async () => {
        expect(parsedName(NAMED_NAME_LINE,),).toEqual(['the corpus',],);
      },
    },),
    it({
      name: 'reads no name from a suite options line declaring another field',
      fn: async () => {
        expect(parsedName(OTHER_FIELD_LINE,),).toEqual([],);
      },
    },),
    it({
      name: 'reads one opener and its name from a suite it is handed',
      fn: async () => {
        /**
         Suites and names the control lines declare.
         */
        const scan = scanSuiteNames(NAMED_SUITE_LINES,);
        expect(scan.openers,).toBe(1,);
        expect(scan.names,).toEqual(['the corpus',],);
      },
    },),
    it({
      name: 'finds every corpus suite and reads a non-empty name on each',
      fn: async () => {
        /**
         Corpus test source split into lines.
         */
        const lines = (await readFile(
          join(
            import.meta.dirname,
            CORPUS_TEST_FILE_NAME,
          ),
          'utf8',
        )).split(
          '\n',
        );
        /**
         Suites and names the corpus declares.
         */
        const scan = scanSuiteNames(lines,);
        // A scan that read no suite proves nothing about the corpus.
        expect(scan.openers,).toBeGreaterThan(0,);
        // Every opener names itself, so no suite reads as nameless.
        expect(scan.names.length,).toBe(scan.openers,);
        expect(
          scan.names.filter(function unnamed(name,) {
            return name.trim() === '';
          },),
        ).toEqual([],);
      },
    },),
  ],
},);
