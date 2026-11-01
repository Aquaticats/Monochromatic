/**
 Summarize the runtime recall results (`./recall-runtime.ts`) per ledger
 row: which sidecar files detected the bug in the bounded layer, split into
 files that specify behaviour and files that pin current behaviour. Runs on
 the host over `dist/recall/runtime/runtime.jsonl`; prints JSON.

 Only files listed in `dist/recall/baseline-files.txt` (the sidecar as it
 stood when the audit started) count, so files added during the audit, and
 machine-local files, never inflate recall.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { resolve, } from 'node:path';

import { isPinningFile, } from './mutation-mutant.ts';
import { fieldOf, } from './recall-json.ts';

/**
 Recall output directory.
 */
const RECALL = resolve(
  import.meta.dirname,
  '..',
  'dist',
  'recall',
);

/**
 One detection entry as `./recall-runtime.ts` writes it.
 */
type Detection = {
  readonly file: string;
  readonly failures: readonly string[];
};

/**
 Whether a parsed value is a detection entry.

 @param value - Parsed JSON.

 @returns True for an object with a string `file` and a `failures` array.

 @example
 ```ts
 isDetection({ failures: [], file: 'src/a.ts', }); // true
 ```
 */
function isDetection(value: unknown,): value is Detection {
  /**
   Failures field.
   */
  const failures = fieldOf({
    key: 'failures',
    value,
  },);
  return ((typeof fieldOf({
    key: 'file',
    value,
  },)) === 'string') && Array.isArray(failures,)
    && failures.every(function isString(name: unknown,) {
      return (typeof name) === 'string';
    },);
}

/**
 Test names in one detection, without the suite rollup.

 @param entry - Detection of one file.

 @returns Names of failing tests, which carry a suite and a test bracket.

 @example
 ```ts
 testsOf({ failures: ['[s] [t]', '[s]',], file: 'src/a.ts', }); // ['[s] [t]']
 ```
 */
function testsOf(entry: Detection,): readonly string[] {
  return entry.failures
    .filter(function isTest(name,) {
      return name.includes('] [',);
    },);
}

if (import.meta.main) {
  /**
   Baseline sidecar files.
   */
  const baseline = new Set((await readFile(
    resolve(
      RECALL,
      'baseline-files.txt',
    ),
    'utf8',
  ))
    .split('\n',)
    .filter(function nonEmpty(line,) {
      return line !== '';
    },),);
  /**
   Rows in run order.
   */
  const rows = (await readFile(
    resolve(
      RECALL,
      'runtime',
      'runtime.jsonl',
    ),
    'utf8',
  ))
    .split('\n',)
    .filter(function nonEmpty(line,) {
      return line !== '';
    },)
    .map(function summarize(line,) {
      /**
       Parsed record.
       */
      const record: unknown = JSON.parse(line,);
      /**
       Bounded-layer detections in baseline files.
       */
      const bounded = [fieldOf({
        key: 'bounded',
        value: record,
      },),]
        .flat()
        .filter(isDetection,)
        .filter(function inBaseline(entry,) {
          return baseline.has(entry.file,);
        },);
      return {
        control: fieldOf({
          key: 'control',
          value: record,
        },),
        id: fieldOf({
          key: 'id',
          value: record,
        },),
        pinning: bounded
          .filter(function pins(entry,) {
            return isPinningFile(entry.file,);
          },)
          .flatMap(testsOf,),
        specifying: bounded
          .filter(function specifies(entry,) {
            return !isPinningFile(entry.file,);
          },)
          .flatMap(testsOf,),
      };
    },);
  /**
   Latest row per id: `recall-runtime.ts` appends, so a rerun of one id adds
   a second row that would otherwise count twice.
   */
  const latest = new Map(rows.map(function keyed(row,) {
    return [
      String(row.id,),
      row,
    ] as const;
  },),);
  console.log(JSON.stringify(
    [...latest.values(),],
    undefined,
    2,
  ),);
}
