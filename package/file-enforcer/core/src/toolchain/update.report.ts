/**
 Per-tool report lines and the summary for `toolchain:update`.

 @module
 */

import type { UpdateOutcome, } from './update.resolve.ts';

/**
 Selects the outcomes of one kind.

 @param outcomes - outcomes to filter

 @param kind - outcome kind to keep

 @returns outcomes of that kind, in report order

 @example
 ```ts
 outcomesOfKind({ outcomes, kind: 'failed', },).length; // 0
 ```
 */
export function outcomesOfKind(
  {
    outcomes,
    kind,
  }: {
    readonly outcomes: readonly UpdateOutcome[];
    readonly kind: UpdateOutcome['kind'];
  },
): readonly UpdateOutcome[] {
  return outcomes.filter(function hasKind(outcome,): boolean {
    return outcome.kind === kind;
  },);
}

/**
 Counts the outcomes of one kind.

 @param outcomes - outcomes to count

 @param kind - outcome kind to count

 @returns how many outcomes have that kind
 */
function countKind(
  {
    outcomes,
    kind,
  }: {
    readonly outcomes: readonly UpdateOutcome[];
    readonly kind: UpdateOutcome['kind'];
  },
): number {
  return outcomesOfKind({
    outcomes,
    kind,
  },)
    .length;
}

/**
 Renders the per-tool report lines and the summary.

 @param outcomes - outcomes to report

 @returns report text

 @example
 ```ts
 formatReport({ outcomes, },); // 'age unchanged\ntoolchain update: 0 updated, 1 unchanged, 0 pinned, 0 failed'
 ```
 */
export function formatReport(
  { outcomes, }: { readonly outcomes: readonly UpdateOutcome[]; },
): string {
  /**
   Per-tool lines.
   */
  const lines: string[] = [];
  for (const outcome of outcomes) {
    if (outcome.kind === 'updated') {
      lines.push(`${
        outcome.id
      } ${
        outcome.oldVersion
      } -> ${
        outcome.newVersion
      }`,);
      continue;
    }
    if (outcome.kind === 'pinned') {
      lines.push(`${
        outcome.id
      } pinned (${
        outcome.note ?? 'no newer'
      })`,);
      continue;
    }
    if (outcome.kind === 'failed') {
      lines.push(`${
        outcome.id
      } failed: ${
        outcome.note ?? 'unknown error'
      }`,);
      continue;
    }
    lines.push(`${outcome.id} unchanged`,);
  }
  lines.push(
    `toolchain update: ${
      String(countKind({
        outcomes,
        kind: 'updated',
      },),)
    } updated, ${
      String(countKind({
        outcomes,
        kind: 'unchanged',
      },),)
    } unchanged, ${
      String(countKind({
        outcomes,
        kind: 'pinned',
      },),)
    } pinned, ${
      String(countKind({
        outcomes,
        kind: 'failed',
      },),)
    } failed`,
  );
  return lines.join('\n',);
}
