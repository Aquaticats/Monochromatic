/**
 Assertion machinery for the gated disposable-host verification:
 the check collector and the comparison of served rows against the projection
 of the checked-in listing.

 Split out of `verify-host.ts` so each file stays inside the line budget.
 Everything here is verification-only:
 the package entry never imports this module.

 @module
 */

import type {
  Snapshot,
} from './disposable-host.ts';
import type {
  QwenTokenPlanIndividualExpectedRow,
} from './verify-host-expectations.ts';

//region Constants

/**
 Model the live listing serves and the builtin catalog lacks,
 `deepseek-v4.1-flash-1017` documented in
 `doc/audit/qwen-token-plan-individual-model-facts-2026-11-11.md`.
 */
export const LIVE_ONLY_FLASH = 'deepseek-v4.1-flash-1017';

/**
 Model the live listing serves and the builtin catalog lacks,
 `qwen4-max-preview` documented in
 `doc/audit/qwen-token-plan-individual-model-facts-2026-11-11.md`.
 */
export const LIVE_ONLY_PREVIEW = 'qwen4-max-preview';

//endregion Constants

//region Types

/**
 Assertion outcomes collected during one run.
 */
export type Checks = {
  /**
   Claims that held, each with the measured value behind it.
   */
  readonly passes: string[];
  /**
   Claims that did not hold, each with the measured value behind it.
   */
  readonly failures: string[];
};

//endregion Types

//region Assertions

/**
 Record one assertion outcome.

 @param checks - collector the outcome is written to

 @param label - what the assertion claims

 @param ok - whether the claim held

 @param detail - measured value the reader needs when the claim did not hold

 @example
 ```typescript
 check({ checks, label: 'served set matches', ok: true, detail: '14 models' });
 ```
 */
export function check(
  {
    checks,
    label,
    ok,
    detail,
  }: {
    readonly checks: Checks;
    readonly label: string;
    readonly ok: boolean;
    readonly detail: string;
  },
): void {
  if (ok)
    checks.passes
      .push(`${label}: ${detail}`,);
  else
    checks.failures
      .push(`${label}: ${detail}`,);
}

/**
 Sorted model ids one snapshot reports for the override route.

 @param snapshot - probe snapshot to read

 @returns the served ids in sorted order

 @example
 ```typescript
 const ids = servedIds(after);
 ```
 */
export function servedIds(
  snapshot: Snapshot,
): string[] {
  return snapshot.routeModels
    .map(function toId(model,) {
      return model.id;
    },)
    .toSorted();
}

/**
 Whether two sorted id lists name the same set.

 @param actual - ids the host served

 @param expected - ids the projection earns

 @returns `true` when both lists match position by position

 @example
 ```typescript
 sameIdSet({ actual: servedIds(after), expected: expectedIds });
 ```
 */
export function sameIdSet(
  {
    actual,
    expected,
  }: {
    readonly actual: readonly string[];
    readonly expected: readonly string[];
  },
): boolean {
  return (actual.length === expected.length)
    && actual.every(function matches(
      one,
      index,
    ) {
      return one === expected[index];
    },);
}

/**
 Assert the served rows against the projection of the checked-in listing.

 @param checks - collector the outcomes are written to

 @param snapshots - the two probe snapshots, before and after the refresh

 @param expected - projected rows the checked-in listing earns

 @param staleOnlyIds - ids the pinned builtin rows serve that the listing lacks

 @param mediaIds - ids the live listing returns that map to no serviceable row

 @example
 ```typescript
 assertSnapshots({ checks, snapshots, expected, staleOnlyIds, mediaIds });
 ```
 */
export function assertSnapshots(
  {
    checks,
    snapshots,
    expected,
    staleOnlyIds,
    mediaIds,
  }: {
    readonly checks: Checks;
    readonly snapshots: {
      readonly before: Snapshot;
      readonly after: Snapshot;
    };
    readonly expected: readonly QwenTokenPlanIndividualExpectedRow[];
    readonly staleOnlyIds: readonly string[];
    readonly mediaIds: readonly string[];
  },
): void {
  /**
   Expected ids in sorted order, the comparison's reference set.
   */
  const expectedIds = expected
    .map(function toId(row,) {
      return row.id;
    },)
    .toSorted();
  check({
    checks,
    label: 'snapshot catalog serves the checked-in listing set at session start',
    ok: sameIdSet({
      actual: servedIds(snapshots.before,),
      expected: expectedIds,
    },),
    detail: `${String(snapshots.before
      .routeModels
      .length,)} served against ${String(expectedIds.length,)} expected`,
  },);
  check({
    checks,
    label: 'refresh keeps the served set when the listing fetch cannot land',
    ok: sameIdSet({
      actual: servedIds(snapshots.after,),
      expected: servedIds(snapshots.before,),
    },),
    detail: `${String(snapshots.before
      .routeModels
      .length,)} -> ${String(snapshots.after
        .routeModels
        .length,)} served`,
  },);
  for (const liveOnly of [
    LIVE_ONLY_FLASH,
    LIVE_ONLY_PREVIEW,
  ]) {
    check({
      checks,
      label: `live-only model "${liveOnly}" is served`,
      ok: servedIds(snapshots.after,)
        .includes(liveOnly),
      detail: `present=${String(servedIds(snapshots.after,)
        .includes(liveOnly,),)}`,
    },);
  }
  check({
    checks,
    label: 'listing ids that map to no serviceable row are never served',
    ok: mediaIds.every(function mediaAbsent(id,) {
      return !servedIds(snapshots.after,)
        .includes(id);
    },),
    detail: mediaIds.length === 0
      ? 'the listing carries no media ids'
      : `absent: ${mediaIds.join(', ')}`,
  },);
  check({
    checks,
    label: 'builtin stale-only ids are gone',
    ok: staleOnlyIds.every(function staleAbsent(id,) {
      return !servedIds(snapshots.after,)
        .includes(id);
    },),
    detail: staleOnlyIds.length === 0
      ? 'the pinned builtin rows hold nothing the listing lacks'
      : `absent: ${staleOnlyIds.join(', ')}`,
  },);
  /**
   Served rows whose facts diverge from the projection.
   */
  const divergent = snapshots.after
    .routeModels
    .filter(function divergesFromProjection(model,) {
      /**
       Projected row the same id earns, absent when the host served an id the
       listing does not map.
       */
      const projected = expected.find(function sameId(row,) {
        return row.id === model.id;
      },);
      if (projected === undefined)
        return true;
      return (model.contextWindow !== projected.contextWindow)
        || (model.maxTokens !== projected.maxTokens)
        || (model.reasoning !== projected.reasoning)
        || (model.thinkingLevelCount !== projected.thinkingLevelCount);
    },);
  check({
    checks,
    label: 'every served row carries the projected facts',
    ok: divergent.length === 0,
    detail: divergent.length === 0
      ? `${String(snapshots.after
        .routeModels
        .length,)} rows match`
      : `divergent: ${divergent
        .map(function name(model,) {
          return model.id;
        },)
        .join(', ')}`,
  },);
}

//endregion Assertions
