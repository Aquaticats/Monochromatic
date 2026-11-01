/**
 Per-scenario ledger of what the harness did:
 worktree writes,
 deliberate staging,
 commit attempts with their captured bytes,
 and auxiliary wrapper commands.
 The observation gatherer compares repository state against it.

 @module
 */

import { createHash, } from 'node:crypto';

import type { ContentState, } from './invariant-model-fixture.ts';
import type { ProcessOutcome, } from './process-fixture.ts';

//region Types

/**
 Bytes one attempt captured for one selected path;
 `bytes` is absent for a deletion.
 */
export type CapturedPath = Readonly<{
  /**
   Repository path.
   */
  path: string;
  /**
   Exact bytes at invocation.
   */
  bytes?: Buffer;
}>;

/**
 How an attempt selects content.
 */
export type AttemptMode = 'amend' | 'explicit' | 'foreign' | 'index';

/**
 Finished commit attempt.
 */
export type AttemptRecord = Readonly<{
  /**
   Unique label within the scenario.
   */
  label: string;
  /**
   Message token that identifies the attempt's commit in history.
   */
  token: string;
  /**
   Selection mode.
   */
  mode: AttemptMode;
  /**
   Paths a landed commit may change.
   */
  selectedPaths: readonly string[];
  /**
   Bytes captured at invocation.
   */
  captured: readonly CapturedPath[];
  /**
   Monotonic harness time the attempt was started at, right after its bytes were read;
   an attempt started later on a path it shares with an earlier one captured it later,
   because the harness writes a shared path only after the earlier attempt captured it.
   */
  startedAt: number;
  /**
   `HEAD` read immediately before the process started.
   */
  headBefore: string;
  /**
   Branch `HEAD` named at invocation.
   */
  expectedBranch: string;
  /**
   Settled process.
   */
  outcome: ProcessOutcome;
  /**
   Whether the harness killed the process group on purpose.
   */
  killed: boolean;
  /**
   Whether auto-push applies to this attempt.
   */
  requiresRemote: boolean;
}>;

/**
 Finished non-commit wrapper command.
 */
export type AuxiliaryRecord = Readonly<{
  /**
   Unique label within the scenario.
   */
  label: string;
  /**
   Settled process.
   */
  outcome: ProcessOutcome;
  /**
   Whether the accepted design requires success.
   */
  mustSucceed: boolean;
  /**
   Whether the command is a cli-git wrapper invocation whose stderr carries JSONL events.
   */
  wrapper: boolean;
}>;

/**
 Mutable per-scenario ledger.
 */
export type WorkloadLedger = Readonly<{
  /**
   Records the harness's latest worktree write;
   omit bytes for a removal.
   */
  recordWorktree: (entry: CapturedPath,) => void;
  /**
   Marks or clears a path the harness deliberately left staged.
   */
  recordStaged: (entry: Readonly<{
    path: string;
    staged: boolean
  }>,) => void;
  /**
   Adds a finished attempt.
   */
  addAttempt: (record: AttemptRecord,) => void;
  /**
   Adds a finished auxiliary command.
   */
  addAuxiliary: (record: AuxiliaryRecord,) => void;
  /**
   Current contents.
   */
  snapshot: () => Readonly<{
    worktree: ReadonlyMap<string, ContentState>;
    staged: ReadonlySet<string>;
    attempts: readonly AttemptRecord[];
    auxiliaries: readonly AuxiliaryRecord[];
  }>;
}>;

//endregion Types

//region Content digests

/**
 Converts optional bytes to a content state.

 @param bytes - exact bytes, absent for a missing path

 @returns content state with SHA-256 digest

 @example
 ```ts
 contentOf(Buffer.from('a\n')); // => { state: 'present', digest: '...' }
 ```
 */
export function contentOf(bytes?: Buffer,): ContentState {
  if (bytes === undefined)
    return { state: 'absent', };
  return {
    state: 'present',
    digest: createHash('sha256',)
      .update(bytes,)
      .digest('hex',),
  };
}

/**
 Digest of a scenario's workload:
 each attempt's label,
 mode,
 selection,
 and captured bytes,
 independent of completion order and of timing-dependent facts such as `HEAD` or exit codes.
 Equal digests across runs of one seed show the workload replayed exactly.

 @param attempts - finished attempts

 @returns hexadecimal SHA-256 digest

 @example
 ```ts
 workloadDigest(ledger.snapshot().attempts);
 ```
 */
export function workloadDigest(attempts: readonly AttemptRecord[],): string {
  /**
   Canonical attempt descriptions sorted by label.
   */
  const canonical = attempts
    .map(function describe(attempt,) {
      return JSON.stringify([
        attempt.label,
        attempt.mode,
        attempt.selectedPaths,
        attempt.captured
          .map(function capturedDigest(captured,) {
            return [
              captured.path,
              contentOf(captured.bytes,),
            ];
          },),
      ],);
    },)
    .toSorted(function byText(
      left,
      right,
    ) {
      return left.localeCompare(right,);
    },);
  return createHash('sha256',)
    .update(canonical.join('\n',),)
    .digest('hex',);
}

//endregion Content digests

//region Ledger

/**
 Creates an empty ledger.

 @returns ledger

 @example
 ```ts
 const ledger = createLedger();
 ledger.recordWorktree({ path: 'a.txt', bytes: Buffer.from('a\n') });
 ```
 */
export function createLedger(): WorkloadLedger {
  /**
   Latest harness-written content per path.
   */
  const worktree = new Map<string, ContentState>();
  /**
   Paths deliberately left staged.
   */
  const staged = new Set<string>();
  /**
   Finished attempts in completion order.
   */
  const attempts: AttemptRecord[] = [];
  /**
   Finished auxiliary commands in completion order.
   */
  const auxiliaries: AuxiliaryRecord[] = [];
  return {
    recordWorktree(entry: CapturedPath,): void {
      worktree.set(
        entry.path,
        contentOf(entry.bytes,),
      );
    },
    recordStaged(entry: Readonly<{
      path: string;
      staged: boolean
    }>,): void {
      if (entry.staged)
        staged.add(entry.path,);
      else
        staged.delete(entry.path,);
    },
    addAttempt(record: AttemptRecord,): void {
      attempts.push(record,);
    },
    addAuxiliary(record: AuxiliaryRecord,): void {
      auxiliaries.push(record,);
    },
    snapshot() {
      return {
        worktree: new Map(worktree,),
        staged: new Set(staged,),
        attempts: [...attempts,],
        auxiliaries: [...auxiliaries,],
      };
    },
  };
}

//endregion Ledger
