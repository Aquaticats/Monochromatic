import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import type {
  PatchOperation,
  PatchRejection,
} from './apply-patch.ts';
import {
  type MarkupDelta,
  settleMarkupMoves,
} from '@monochromatic-dev/translation-repair-markup/ts';

//region Patch markup settlement
// The one gate of `applyPatchOperations` that reads the patch as a whole
// rather than an edit alone (ledger L4): an atom one edit drops and another
// writes has survived, so which edits the markup rule refuses is only known
// once every other gate has ruled on every edit. Its own file because the
// patch applier is at its line budget.

/**
 An operation every per-edit gate passed, waiting on the patch-level markup
 settlement.

 @example
 ```ts
 const gated: GatedOperation = { operation, restored, delta: NO_MARKUP_DELTA, };
 ```
 */
export type GatedOperation = {
  /**
   Operation as proposed, which is what a refusal records.
   */
  readonly operation: PatchOperation;

  /**
   Operation as it would ship, typography restored.
   */
  readonly restored: PatchOperation;

  /**
   What it did to its envelope's markup.
   */
  readonly delta: MarkupDelta;
};

/**
 Delta of an edit the markup rule does not read, as under the skipped gate.
 */
export const NO_MARKUP_DELTA: MarkupDelta = {
  unexcused: [],
  gained: [],
};

/**
 Refuses the gated operations whose lost markup the patch writes nowhere
 else, and merges those refusals into the rest in input order.

 @param operations - every proposed operation, in input order

 @param gated - operations that passed every per-edit gate, in input order

 @param rejected - refusals the per-edit gates already made

 @returns Operations that ship, and every refusal, both in input order

 @example
 ```ts
 const { applied, rejected: refused, } = settleGatedOperations({ operations, gated, rejected, },);
 ```
 */
export function settleGatedOperations(
  {
    operations,
    gated,
    rejected,
  }: {
    readonly operations: readonly PatchOperation[];
    readonly gated: readonly GatedOperation[];
    readonly rejected: readonly PatchRejection[];
  },
): {
  readonly applied: readonly PatchOperation[];
  readonly rejected: readonly PatchRejection[];
} {
  /**
   Kinds each gated operation was refused for, empty where it ships. One
   entry per `gated` operation, in the same order: `settleMarkupMoves`
   returns its input `deltas` mapped one-for-one, and `deltas` is itself
   `gated` mapped one-for-one, so `ships` and `toRefusal` each read a
   present entry at every index of `gated`.
   */
  const refusals = settleMarkupMoves({
    deltas: gated.map(function toDelta(entry,) {
      return entry.delta;
    },),
  },);
  return {
    applied: gated
      .filter(function ships(
        _entry,
        index,
      ): boolean {
        /**
         Kinds this operation lost.
         */
        const kinds = nonNullishOrThrow(refusals[index],);
        return kinds.length === 0;
      },)
      .map(function toRestored(entry,) {
        return entry.restored;
      },),
    rejected: [
      ...rejected,
      ...gated.flatMap(function toRefusal(
        entry,
        index,
      ): readonly PatchRejection[] {
        /**
         Kinds this operation lost, none where it ships.
         */
        const kinds = nonNullishOrThrow(refusals[index],);
        // Kinds only, never the atoms: a destination or a code span is page
        // content, and the reason is stored.
        return (kinds.length === 0)
          ? []
          : [{
            operation: entry.operation,
            reason: `preservation-lost-markup (${kinds.join(', ',)})`,
          },];
      },),
    ].toSorted(function byInputOrder(
      left,
      right,
    ): number {
      return operations.indexOf(left.operation,) - operations.indexOf(right.operation,);
    },),
  };
}

//endregion Patch markup settlement
