/**
 Executes planned trace operations through the wrapper with bounded concurrency.

 @module
 */

import {
  editText,
  synthesizeBinary,
  synthesizeText,
} from './content-fixture.ts';
import type { AttemptRecord, } from './ledger-fixture.ts';
import type { ScenarioContext, } from './scenario-model-fixture.ts';
import type { TraceOperation, } from './trace-replay-fixture.ts';
import { reached, } from './barrier-fixture.ts';
import {
  readWorktree,
  runWrapper,
  startAttempt,
  writeWorktree,
} from './worker-fixture.ts';

//region Content

/**
 Computes new bytes for one change against the current worktree.

 @param context - scenario context

 @param operation - owning operation

 @param change - change to apply

 @returns new bytes, absent for deletions

 @example
 ```ts
 await changeBytes({ context, operation, change });
 ```
 */
async function changeBytes({
  context,
  operation,
  change,
}: Readonly<{
  context: ScenarioContext;
  operation: TraceOperation;
  change: TraceOperation['changes'][number];
}>,): Promise<Buffer | 'delete'> {
  /**
   Deterministic source per operation and path.
   */
  const random = context.random
    .fork(`${operation.label}:${change.path}`,);
  if (change.shape
    .kind
    === 'delete')
    return 'delete';
  if (change.shape
    .binary)
    return synthesizeBinary({
      random,
      size: change.shape
        .size,
    },);
  if ((change.shape
    .kind
    === 'add') || (change.shape
      .kind
      === 'rename'))
    return synthesizeText({
      random,
      size: change.shape
        .size,
    },);
  /**
   Current bytes to edit; a missing file is synthesized fresh.
   */
  const current = (await readWorktree({
    repository: context.repository,
    path: change.path,
  },)).bytes;
  if ((current === undefined) || current.includes(0,))
    return synthesizeText({
      random,
      size: change.shape
        .size,
    },);
  return editText({
    random,
    current,
    added: change.shape
      .added,
    deleted: change.shape
      .deleted,
  },);
}

//endregion Content

//region Execution

/**
 Applies an operation's changes in trace order:
 edits read the bytes earlier writes produced,
 and rename sources are removed after their destinations are written.

 @param context - scenario context

 @param operation - operation whose changes to apply

 @example
 ```ts
 await applyChanges({ context, operation });
 ```
 */
async function applyChanges({
  context,
  operation,
}: Readonly<{
  context: ScenarioContext;
  operation: TraceOperation;
}>,): Promise<void> {
  await operation.changes
    .reduce(
    async function applyAfter(
      previous,
      change,
    ) {
      await previous;
    /**
     New bytes, or `delete`.
     */
      const bytes = await changeBytes({
        context,
        operation,
        change,
      },);
      await writeWorktree({
        ...context,
        path: change.path,
        ...(bytes === 'delete' ? {} : { bytes, }),
      },);
      if (change.from !== undefined)
        await writeWorktree({
          ...context,
          path: change.from,
        },);
    },
    Promise.resolve(),
    );
}

/**
 Claim an in-flight operation holds on one path.
 */
export type PathClaim = Readonly<{
  /**
   Settles once the claiming operation captured the path.
   */
  captured: Promise<void>;
  /**
   Settles once the claiming operation's wrapper exited, after its landing.
   */
  finished: Promise<void>;
  /**
   Whether the claiming operation, or an in-flight operation before it, creates the path.
   */
  adds: boolean;
}>;

/**
 Paths an operation creates: additions and rename destinations.

 @param operation - planned operation

 @returns created paths

 @example
 ```ts
 createdPaths(operation);
 ```
 */
export function createdPaths(operation: TraceOperation,): ReadonlySet<string> {
  return new Set(operation.changes
    .flatMap(function created(change,): readonly string[] {
      return (change.shape
        .kind
        === 'add') || (change.shape
          .kind
          === 'rename') ? [change.path,] : [];
    },),);
}

/**
 Paths an operation removes: deletions and rename sources.

 @param operation - planned operation

 @returns removed paths

 @example
 ```ts
 removedPaths(operation);
 ```
 */
export function removedPaths(operation: TraceOperation,): ReadonlySet<string> {
  return new Set(operation.changes
    .flatMap(function removed(change,): readonly string[] {
      return [
        ...(change.shape
          .kind
          === 'delete' ? [change.path,] : []),
        ...(change.from === undefined ? [] : [change.from,]),
      ];
    },),);
}

/**
 What an operation waits for on one of its paths:
 the claimant's capture,
 or its landing when this operation removes a path the claimant creates,
 because the path exists in neither the preparation base nor the worktree until that landing,
 so a native `git commit -- <path>` would fail with "pathspec did not match".

 @param claim - latest claim on the path

 @param removes - whether this operation removes the path

 @example
 ```ts
 blockerOf({ claim, removes: true });
 ```
 */
export function blockerOf({
  claim,
  removes,
}: Readonly<{
  claim: PathClaim;
  removes: boolean;
}>,): Promise<void> {
  return removes && claim.adds ? claim.finished : claim.captured;
}

/**
 Runs one operation after the in-flight owners of its paths captured theirs,
 or landed when it removes a path they create.

 @param context - scenario context

 @param operation - operation to run

 @param claims - claim per path of the latest claiming operation

 @returns finished attempt

 @example
 ```ts
 await runOperation({ context, operation, claims });
 ```
 */
async function runOperation({
  context,
  operation,
  claims,
}: Readonly<{
  context: ScenarioContext;
  operation: TraceOperation;
  claims: Map<string, PathClaim>;
}>,): Promise<AttemptRecord> {
  /**
   Paths this operation removes.
   */
  const removes = removedPaths(operation,);
  /**
   Paths this operation creates.
   */
  const creates = createdPaths(operation,);
  /**
   Signals this operation must wait for.
   */
  const blockers = operation.selected
    .flatMap(function blocker(path,) {
    /**
     Latest claim on the path.
     */
      const claim = claims.get(path,);
      return claim === undefined ? [] : [blockerOf({
        claim,
        removes: removes.has(path,),
      },),];
    },);
  /**
   This operation's own capture signal.
   */
  const captured = Promise.withResolvers<void>();
  /**
   This operation's own exit signal.
   */
  const finished = Promise.withResolvers<void>();
  operation.selected
    .forEach(function claim(path,) {
      claims.set(
        path,
        {
          captured: captured.promise,
          finished: finished.promise,
        // A path an earlier in-flight operation creates exists in HEAD only once this operation landed too.
          adds: creates.has(path,) || (claims.get(path,)
            ?.adds
            ?? false),
        },
      );
    },);
  /**
   Settles both signals however the operation ends, so no waiting operation hangs.
   */
  using _settled = {
    [Symbol.dispose]: function settle(): void {
      captured.resolve();
      finished.resolve();
    },
  };
  return await runClaimedOperation({
    context,
    operation,
    blockers,
    captured,
  },);
}

/**
 Runs an operation whose claims are registered:
 waits for its blockers,
 writes its changes,
 and commits.

 @param context - scenario context

 @param operation - operation to run

 @param blockers - signals to wait for

 @param captured - resolved once this operation captured

 @returns finished attempt
 */
async function runClaimedOperation({
  context,
  operation,
  blockers,
  captured,
}: Readonly<{
  context: ScenarioContext;
  operation: TraceOperation;
  blockers: readonly Promise<void>[];
  captured: PromiseWithResolvers<void>;
}>,): Promise<AttemptRecord> {
  await Promise.all(blockers,);
  await applyChanges({
    context,
    operation,
  },);
  if (operation.adds
    .length
    > 0) {
    await runWrapper({
      ...context,
      label: `${operation.label} add`,
      args: [
        'add',
        '--',
        ...operation.adds,
      ],
      mustSucceed: true,
    },);
    operation.adds
      .forEach(function staged(path,) {
        context.ledger
          .recordStaged({
            path,
            staged: true,
          },);
      },);
  }
  /**
   Started explicit-path commit.
   */
  const attempt = await startAttempt({
    ...context,
    label: operation.label,
    paths: operation.selected,
    mode: 'explicit',
  },);
  await reached({
    repository: context.repository,
    token: attempt.token,
    running: attempt.running,
    event: 'pre-commit',
  },);
  captured.resolve();
  /**
   Finished record.
   */
  const record = await attempt.finished;
  if (record.outcome
    .exitCode
    === 0) {
    operation.adds
      .forEach(function landed(path,) {
        context.ledger
          .recordStaged({
            path,
            staged: false,
          },);
      },);
  }
  return record;
}

/**
 Runs operations with at most `concurrency` in flight,
 starting them in trace order.

 @param context - scenario context; the repository needs a `pre-commit` hook

 @param operations - planned operations

 @param concurrency - in-flight bound

 @returns finished attempts in completion order

 @example
 ```ts
 await runTraceOperations({ context, operations, concurrency: 4 });
 ```
 */
export async function runTraceOperations({
  context,
  operations,
  concurrency,
}: Readonly<{
  context: ScenarioContext;
  operations: readonly TraceOperation[];
  concurrency: number;
}>,): Promise<readonly AttemptRecord[]> {
  /**
   Shared cursor over operations.
   */
  const cursor = { next: 0, };
  /**
   Claims per path.
   */
  const claims = new Map<string, PathClaim>();
  /**
   Finished attempts.
   */
  const finished: AttemptRecord[] = [];
  /**
   One pool lane pulling operations until none remain.

   @example
   ```ts
   await lane();
   ```
   */
  async function lane(): Promise<void> {
    for (let operation = operations[cursor.next]; operation !== undefined; operation = operations[cursor.next]) {
      cursor.next += 1;
      // oxlint-disable-next-line no-await-in-loop -- Each lane runs its operations one at a time by design.
      finished.push(await runOperation({
        context,
        operation,
        claims,
      },),);
    }
  }
  await Promise.all(Array.from(
    { length: concurrency, },
    lane,
  ),);
  return finished;
}

//endregion Execution
