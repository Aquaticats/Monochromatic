/**
 V8 block coverage projected onto source lines of one target file.

 The target here is a single third-party JavaScript file (the installed
 deepmerge-ts `dist/index.mjs`), so unlike the workspace campaigns' readers
 this module needs no URL classification or TypeScript offset handling:
 V8 offsets index the file's text directly.

 Projection: for each process's script entry, paint a per-character bitmap
 with the innermost range deciding (ranges painted longest first, so a
 never-taken block inside a covered function wins), then count a line as
 covered when any non-whitespace character on it is covered. Processes are
 unioned at the line level.

 @module
 */

/**
 One V8 source range: a half-open character span and its execution count.
 */
export type V8Range = {
  readonly startOffset: number;
  readonly endOffset: number;
  readonly count: number;
};

/**
 One V8 function coverage entry.
 */
export type V8Function = {
  readonly functionName: string;
  readonly ranges: readonly V8Range[];
};

/**
 One covered script inside a `NODE_V8_COVERAGE` output file.
 */
export type V8Script = {
  readonly url: string;
  readonly functions: readonly V8Function[];
};

/**
 Whether parsed JSON is a `NODE_V8_COVERAGE` output file.

 @param value - Parsed JSON from the coverage directory.

 @returns Whether `value` has a `result` array of scripts.

 @example
 ```ts
 isCoverageFile({ result: [], }); // true
 ```
 */
export function isCoverageFile(value: unknown,): value is { readonly result: readonly V8Script[]; } {
  return ((typeof value) === 'object')
    && (value !== null)
    && Array.isArray(Reflect.get(
      value,
      'result',
    ),);
}

/**
 Covered line numbers (0-based) of one script entry.

 @param source - Text of the target file.
 
 @param script - One process's coverage of that file.

 @returns Lines holding at least one covered non-whitespace character.

 @example
 ```ts
 const lines = coveredLines({ source, script, });
 ```
 */
export function coveredLines(
  {
    source,
    script,
  }: {
    readonly source: string;
    readonly script: V8Script
  },
): ReadonlySet<number> {
  /**
   Every range of the script, longest first so inner ranges paint last.
   */
  const ranges = script.functions
    .flatMap(function rangesOf(entry,) {
      return entry.ranges;
    },)
    .toSorted(function longestFirst(
      left,
      right,
    ) {
      return (right.endOffset - right.startOffset) - (left.endOffset - left.startOffset);
    },);
  /**
   Per-character covered flag after painting.
   */
  const painted = ranges.reduce(
    function paint(
      bitmap,
      range,
    ) {
      return bitmap.fill(
        range.count > 0 ? 1 : 0,
        range.startOffset,
        range.endOffset,
      );
    },
    new Uint8Array(source.length,),
  );
  /**
   Covered line numbers, filled in one pass over the characters.
   */
  const covered = new Set<number>();
  /**
   Line number of the character being scanned.
   */
  let line = 0;
  // V8 offsets index UTF-16 code units, so the scan walks code units with
  // charAt rather than splitting the text into code points.
  for (let index = 0; index < source.length; index += 1) {
    /**
     Code unit at this offset.
     */
    const character = source.charAt(index,);
    if (character === '\n')
      line += 1;
    else if ((character.trim() !== '') && (painted[index] === 1))
      covered.add(line,);
  }
  return covered;
}

/**
 Number of lines holding any non-whitespace character, the gate's denominator.

 @param source - Text of the target file.

 @returns Count of non-blank lines.

 @example
 ```ts
 codeLineCount('a\n\nb'); // 2
 ```
 */
export function codeLineCount(source: string,): number {
  return source
    .split('\n',)
    .filter(function isCode(line,) {
      return line.trim() !== '';
    },)
    .length;
}

/**
 Names of functions never entered in any process.

 @param scripts - Every process's coverage of the target.

 @returns Sorted names whose function range has count 0 everywhere.

 @example
 ```ts
 const missed = uncalledFunctions([script,]);
 ```
 */
export function uncalledFunctions(scripts: readonly V8Script[],): readonly string[] {
  /**
   Names entered at least once in some process.
   */
  const called = new Set(scripts.flatMap(function calledIn(script,) {
    return script.functions
      .filter(function wasEntered(entry,) {
        return (entry.ranges[0]
          ?.count
          ?? 0) > 0;
      },)
      .map(function nameOf(entry,) {
        return entry.functionName;
      },);
  },),);
  /**
   Every named function across processes.
   */
  const named = scripts.flatMap(function namesIn(script,) {
    return script.functions
      .map(function nameOf(entry,) {
        return entry.functionName;
      },);
  },);
  return [
    ...new Set(named.filter(function neverCalled(name,) {
      return (name !== '') && (!called.has(name,));
    },),),
  ]
    .toSorted();
}
