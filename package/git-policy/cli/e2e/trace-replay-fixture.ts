/**
 Replays a window of the committed commit-shape trace as explicit-path commits with synthesized content.

 Workers touching a path another in-flight worker selected wait until that worker's `pre-commit` hook ran
 (its bytes are captured by then),
 so every attempt's captured bytes are exactly what the harness wrote for it.

 @module
 */

import type {
  CommitShape,
  CommitShapeTrace,
  ShapeChange,
} from './commit-shape-trace-fixture.ts';
import type { SeededRandom, } from './seeded-random-fixture.ts';

//region Planning

/**
 First reference of a path in window order: whether the path pre-exists or the window adds it, and the change.
 */
type FirstReference = Readonly<{
  kind: 'add' | 'existing';
  change: ShapeChange
}>;

/**
 Path identifier paired with its first reference, as a `Map` entry reads.
 */
type FirstReferenceEntry = readonly [
  number,
  FirstReference
];

/**
 One trace commit turned into worker operations.
 */
export type TraceOperation = Readonly<{
  /**
   Unique label.
   */
  label: string;
  /**
   Path changes to apply.
   */
  changes: readonly Readonly<{
    path: string;
    from?: string;
    shape: ShapeChange
  }>[];
  /**
   Paths to stage with `git add` before committing.
   */
  adds: readonly string[];
  /**
   Paths the explicit commit selects.
   */
  selected: readonly string[];
}>;

/**
 Repository path for an anonymous trace path ID.

 @param id - trace path ID

 @returns repository path

 @example
 ```ts
 tracePath(12); // => 'trace/p12'
 ```
 */
export function tracePath(id: number,): string {
  return `trace/p${String(id,)}`;
}

/**
 Reports whether a trace commit can be replayed:
 at least one change,
 no more than the cap,
 and no gitlinks or symlinks.

 @param commit - trace commit

 @param maxChanges - change cap

 @returns eligibility

 @example
 ```ts
 isReplayable({ commit, maxChanges: 16 });
 ```
 */
function isReplayable({
  commit,
  maxChanges,
}: Readonly<{
  commit: CommitShape;
  maxChanges: number;
}>,): boolean {
  return (commit.changes
    .length
    > 0)
    && (commit.changes
      .length
      <= maxChanges)
    && commit.changes
      .every(function plainFile(change,) {
        return (change.mode === 'file') || (change.mode === 'executable');
      },);
}

/**
 Selects consecutive replayable trace commits from a seeded start.

 @param trace - committed trace

 @param random - seeded source

 @param length - commits in the window

 @param maxChanges - per-commit change cap

 @returns window commits oldest first

 @example
 ```ts
 selectTraceWindow({ trace, random, length: 12, maxChanges: 16 });
 ```
 */
export function selectTraceWindow({
  trace,
  random,
  length,
  maxChanges,
}: Readonly<{
  trace: CommitShapeTrace;
  random: SeededRandom;
  length: number;
  maxChanges: number;
}>,): readonly CommitShape[] {
  /**
   Replayable commits in trace order.
   */
  const eligible = trace.commits
    .filter(function replayable(commit,) {
      return isReplayable({
        commit,
        maxChanges,
      },);
    },);
  /**
   Window start.
   */
  const start = random.integer({
    min: 0,
    max: Math.max(
      0,
      eligible.length - length,
    ),
  },);
  return eligible.slice(
    start,
    start + length,
  );
}

/**
 Turns a window into operations and the seed files they need.

 @param window - trace commits

 @returns operations plus paths that must exist before the window starts

 @example
 ```ts
 planTraceOperations(window);
 ```
 */
export function planTraceOperations(window: readonly CommitShape[],): Readonly<{
  operations: readonly TraceOperation[];
  seeds: readonly Readonly<{
    path: string;
    binary: boolean;
    size: number
  }>[];
}> {
  /**
   First reference of each path in window order.
   */
  const firstReference = new Map<number, FirstReference>();
  window.forEach(function recordFirst(commit,) {
    commit.changes
      .forEach(function recordChange(change,) {
        if ((change.from !== undefined) && (!firstReference.has(change.from,)))
          firstReference.set(
            change.from,
            {
              kind: 'existing',
              change,
            },
          );
        if (!firstReference.has(change.path,))
          firstReference.set(
            change.path,
            {
              kind: (change.kind === 'add') || (change.kind === 'rename') ? 'add' : 'existing',
              change,
            },
          );
      },);
  },);
  return {
    operations: window.map(function operation(
      commit,
      index,
    ): TraceOperation {
      /**
       Changes with repository paths.
       */
      const changes: TraceOperation['changes'] = commit.changes
        .map(function named(change,) {
          return {
            path: tracePath(change.path,),
            ...(change.from === undefined ? {} : { from: tracePath(change.from,), }),
            shape: change,
          };
        },);
      return {
        label: `t${String(index,)}`,
        changes,
        adds: changes.filter(function added(change,) {
          return (change.shape
            .kind
            === 'add') || (change.shape
              .kind
              === 'rename');
        },)
          .map(function addedPath(change,) {
            return change.path;
          },),
        selected: [...new Set(changes.flatMap(function selectedPaths(change,) {
          return change.from === undefined ? [change.path,] : [
            change.from,
            change.path,
          ];
        },),),],
      };
    },),
    seeds: [...firstReference,].filter(function existing([, reference,]: FirstReferenceEntry,) {
      return reference.kind === 'existing';
    },)
      .map(function seed([id, reference,]: FirstReferenceEntry,) {
        return {
          path: tracePath(id,),
          binary: reference.change
            .binary,
          size: reference.change
            .size,
        };
      },),
  };
}

//endregion Planning
