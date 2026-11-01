import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  CommitShape,
  CommitShapeTrace,
  ShapeChange,
} from './commit-shape-trace-fixture.ts';
import {
  editText,
  joinLines,
  MAX_TEXT_BYTES,
  splitLines,
  synthesizeBinary,
  synthesizeText,
} from './content-fixture.ts';
import {
  formatResult,
  runPassed,
  summarize,
} from './report-fixture.ts';
import type { ScenarioResult, } from './scenario-model-fixture.ts';
import { createSeededRandom, } from './seeded-random-fixture.ts';
import {
  planTraceOperations,
  selectTraceWindow,
  tracePath,
} from './trace-replay-fixture.ts';
import {
  blockerOf,
  createdPaths,
  removedPaths,
} from './trace-execution-fixture.ts';

//region Fixtures

/**
 Builds a change shape.

 @param fields - differing fields

 @returns change shape

 @example
 ```ts
 change({ kind: 'add', path: 0 });
 ```
 */
function change(fields: Pick<ShapeChange, 'kind' | 'path'> & Readonly<{ from?: number; mode?: ShapeChange['mode']; size?: number; binary?: boolean; }>,): ShapeChange {
  return { mode: 'file', size: 100, binary: false, added: 2, deleted: 1, ...fields, };
}

/**
 Window exercising add, modify, rename, delete, binary, and overlap.
 */
const WINDOW: readonly CommitShape[] = [
  { changes: [change({ kind: 'add', path: 0, },), change({ kind: 'modify', path: 1, binary: true, size: 64, },),], },
  { changes: [change({ kind: 'modify', path: 0, },), change({ kind: 'rename', path: 3, from: 2, },),], },
  { changes: [change({ kind: 'delete', path: 1, },),], },
];

/**
 Trace with ineligible commits around the window.
 */
const TRACE: CommitShapeTrace = {
  schemaVersion: 1,
  source: 'fixture',
  commitCount: 5,
  pathCount: 5,
  commits: [
    { changes: [], },
    ...WINDOW,
    { changes: [change({ kind: 'add', path: 4, mode: 'gitlink', },),], },
  ],
};

/**
 Result fixture for report tests.
 */
const RESULT: ScenarioResult = {
  name: 'concurrent-disjoint-paths',
  group: 'concurrency',
  gitVersion: '2.55.0',
  status: 'fail',
  violations: [{ invariant: 'all-commits-succeed', subject: 'scenario', detail: 'w1 exited 2', },],
  durationMs: 12,
  attempts: [{ label: 'w0', exitCode: 0, killed: false, }, { label: 'w1', exitCode: -1, killed: true, },],
};

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: selectTraceWindow.name,
      children: [
        it({
          name: 'skips empty, oversized, and gitlink commits',
          fn: async () => {
            expect(selectTraceWindow({ trace: TRACE, random: createSeededRandom(1,), length: 3, maxChanges: 2, },),).toEqual(WINDOW,);
            expect(selectTraceWindow({ trace: TRACE, random: createSeededRandom(1,), length: 3, maxChanges: 1, },),)
              .toEqual([WINDOW[2],],);
          },
        },),
      ],
    },),
    describe({
      name: planTraceOperations.name,
      children: [
        it({
          name: 'seeds only paths that must pre-exist and stages added and renamed paths',
          fn: async () => {
            const { operations, seeds, } = planTraceOperations(WINDOW,);
            expect(seeds,).toEqual([
              { path: tracePath(1,), binary: true, size: 64, },
              { path: tracePath(2,), binary: false, size: 100, },
            ],);
            expect(operations.map(function summary(operation,) {
              return [operation.label, operation.adds, operation.selected,];
            },),).toEqual([
              ['t0', ['trace/p0',], ['trace/p0', 'trace/p1',],],
              ['t1', ['trace/p3',], ['trace/p0', 'trace/p2', 'trace/p3',],],
              ['t2', [], ['trace/p1',],],
            ],);
          },
        },),
        it({
          name: 'a removal of a path an in-flight operation creates waits for that operation to land, every other overlap for its capture',
          fn: async () => {
            const { operations, } = planTraceOperations([
              ...WINDOW,
              { changes: [change({ kind: 'delete', path: 0, },), change({ kind: 'delete', path: 3, },),], },
            ],);
            const [add, rename, , remove,] = operations;
            if ((add === undefined) || (rename === undefined) || (remove === undefined))
              throw new Error('planTraceOperations dropped an operation',);
            expect([...createdPaths(add,),],).toEqual(['trace/p0',],);
            expect([...removedPaths(rename,),],).toEqual(['trace/p2',],);
            expect([...removedPaths(remove,),],).toEqual(['trace/p0', 'trace/p3',],);
            const captured = Promise.resolve();
            const finished = Promise.resolve();
            expect(blockerOf({ claim: { captured, finished, adds: true, }, removes: true, },),).toBe(finished,);
            expect(blockerOf({ claim: { captured, finished, adds: true, }, removes: false, },),).toBe(captured,);
            expect(blockerOf({ claim: { captured, finished, adds: false, }, removes: true, },),).toBe(captured,);
          },
        },),
      ],
    },),
    describe({
      name: 'content synthesis',
      children: [
        it({
          name: 'produces canonical text that final-newline normalization leaves unchanged',
          fn: async () => {
            const text = synthesizeText({ random: createSeededRandom(1,), size: 500, },);
            expect(text.toString('utf8',).endsWith('\n',),).toBe(true,);
            expect(text.toString('utf8',).endsWith('\n\n',),).toBe(false,);
            expect(text.includes('\r',),).toBe(false,);
            expect(synthesizeText({ random: createSeededRandom(1,), size: 10 * MAX_TEXT_BYTES, },).length,)
              .toBeLessThan(2 * MAX_TEXT_BYTES,);
          },
        },),
        it({
          name: 'edits text into different canonical text and round-trips lines',
          fn: async () => {
            const current = joinLines(['a', 'b', 'c',],);
            const edited = editText({ random: createSeededRandom(2,), current, added: 0, deleted: 3, },);
            expect(edited.equals(current,),).toBe(false,);
            expect(splitLines(edited,).length,).toBeGreaterThanOrEqual(1,);
            expect(splitLines(current,),).toEqual(['a', 'b', 'c',],);
            expect(joinLines([],).toString('utf8',),).toBe('empty\n',);
          },
        },),
        it({
          name: 'produces binary bytes that start with NUL',
          fn: async () => {
            const bytes = synthesizeBinary({ random: createSeededRandom(3,), size: 0, },);
            expect(bytes[0],).toBe(0,);
            expect(bytes.length,).toBe(2,);
          },
        },),
      ],
    },),
    describe({
      name: 'report',
      children: [
        it({
          name: 'formats results with exits and violations and summarizes by group',
          fn: async () => {
            expect(formatResult(RESULT,),).toEqual([
              'FAIL  git 2.55.0 concurrent-disjoint-paths (12 ms) exits: w0=0 w1=killed',
              '      all-commits-succeed [scenario]: w1 exited 2',
            ],);
            expect(summarize([RESULT, { ...RESULT, status: 'pass', }, { ...RESULT, group: 'baseline', status: 'skip', },],),).toEqual([
              'summary: 3 scenario runs',
              '  baseline skip: 1',
              '  concurrency fail: 1',
              '  concurrency pass: 1',
            ],);
            expect(runPassed([{ ...RESULT, status: 'pass', }, { ...RESULT, status: 'skip', },],),).toBe(true,);
            expect(runPassed([RESULT,],),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
