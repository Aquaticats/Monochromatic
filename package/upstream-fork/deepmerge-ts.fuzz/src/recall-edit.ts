/**
 Pure helpers for the historical-recall run: splice a ledger row's exact-text
 edits into upstream source, read failing test names from `module-test`
 output, and attribute failures to a bug by subtracting the fixed build's.

 @module
 */

import type { SourceEdit, } from './recall-ledger.ts';

/**
 Error for an edit whose `find` text does not occur exactly `count` times, so
 a transplant never silently lands somewhere else or nowhere.
 */
export class RecallEditError extends Error {
  /**
   @param message - Which edit failed and how often its text occurred.
   */
  constructor(message: string,) {
    super(message,);
    this.name = 'RecallEditError';
  }
}

/**
 Number of non-overlapping occurrences of `find` in `text`.

 @param text - Text to scan.

 @param find - Non-empty needle.

 @returns Occurrence count.

 @example
 ```ts
 occurrences({ find: 'a', text: 'aba', }); // 2
 ```
 */
function occurrences(
  {
    text,
    find,
  }: {
    readonly text: string;
    readonly find: string;
  },
): number {
  return text.split(find,)
    .length
    - 1;
}

/**
 Apply one file's edits in ledger order.

 @param source - Original file text.

 @param edits - Edits for this file.

 @returns Edited text.

 @throws {@link RecallEditError} When an edit's text is empty or occurs other than `count` times.

 @example
 ```ts
 applyEdits({ edits: [{ count: 1, file: 'utils.ts', find: 'a', replace: 'b', },], source: 'a', }); // 'b'
 ```
 */
export function applyEdits(
  {
    source,
    edits,
  }: {
    readonly source: string;
    readonly edits: readonly SourceEdit[];
  },
): string {
  return edits.reduce(
    function applyOne(
      text,
      edit,
    ) {
      if (edit.find === '')
        throw new RecallEditError(`edit in ${edit.file} has empty find text`,);
      /**
       How often the edit's text occurs before it is applied.
       */
      const found = occurrences({
        find: edit.find,
        text,
      },);
      if (found !== edit.count)
        throw new RecallEditError(`edit in ${edit.file} expected ${String(edit.count,)} occurrence(s), found ${String(found,)}: ${JSON.stringify(edit.find,)}`,);
      return text.split(edit.find,)
        .join(edit.replace,);
    },
    source,
  );
}

/**
 Marker `module-test` prints after a failing test's bracketed path.
 */
const FAIL_MARKER = ' [FAIL]';

/**
 Names of failing tests in one file's `module-test` output.

 Each failing line reads `[level] [timestamp] [suite] [test] [FAIL] ...`;
 the name keeps every bracket after the timestamp, so a test is identified by
 its suite path.

 @param output - Combined stdout and stderr of one test file.

 @returns Failing test names in output order, without duplicates.

 @example
 ```ts
 failingTests('[error] [t] [suite] [case] [FAIL] (1ms)'); // ['[suite] [case]']
 ```
 */
export function failingTests(output: string,): readonly string[] {
  return [...new Set(output.split('\n',)
    .filter(function isFailLine(line,) {
      return line.includes(FAIL_MARKER,);
    },)
    .map(function nameOf(line,) {
      /**
       Line up to the fail marker.
       */
      const head = line.slice(
        0,
        line.indexOf(FAIL_MARKER,),
      );
      /**
       Start of the third bracket group, after level and timestamp.
       */
      const third = head.split('] [',)
        .slice(2,)
        .join('] [',);
      return (third === '') ? head : `[${third}`;
    },),),];
}

/**
 Failures the bug alone explains: present on the buggy build and absent on
 the fixed one.

 @param buggy - Failure keys on the buggy build.

 @param fixed - Failure keys on the fixed build.

 @returns Buggy-only keys in their original order.

 @example
 ```ts
 attributable({ buggy: ['a', 'b',], fixed: ['b',], }); // ['a']
 ```
 */
export function attributable(
  {
    buggy,
    fixed,
  }: {
    readonly buggy: readonly string[];
    readonly fixed: readonly string[];
  },
): readonly string[] {
  /**
   Fixed-build keys for constant-time lookup.
   */
  const fixedKeys = new Set(fixed,);
  return buggy.filter(function onlyBuggy(key,) {
    return !fixedKeys.has(key,);
  },);
}
