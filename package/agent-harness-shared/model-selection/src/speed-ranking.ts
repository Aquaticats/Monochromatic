/**
 Name-heuristic speed ranking for model selection.
 
 The pi model registry exposes pricing and token limits, but no throughput or
 latency field. These helpers rank explicit model-name speed signals first,
 then fall back to the previous budget ordering through input price and
 version tie-breaks.
 
 @module
 */

import {
  compareVersions,
  findCheapestInMajorVersions,
} from './version.ts';
import { scoreModelSpeed, } from './speed-signals.ts';
import type { ModelPricing, } from './types.ts';

//region Public API

/**
 Compare two models by speed heuristic, then cost, then version.
 
 @param left - first model to compare
 
 @param right - second model to compare
 
 @returns negative when `left` ranks before `right`
 
 @example
 ```typescript
 compareModelSpeed({ left: flashModel, right: baseModel });
 ```
 */
export function compareModelSpeed<TModel extends ModelPricing,>(
  {
    left,
    right,
  }: {
    readonly left: TModel;
    readonly right: TModel;
  },
): number {
  /**
   Score delta with the right model first so higher speed scores sort earlier.
   */
  const speedDiff = scoreModelSpeed(right,) - scoreModelSpeed(left,);
  if (speedDiff !== 0)
    return speedDiff;

  /**
   Input-price tie-break preserving the previous budget-model fallback.
   */
  const inputCostDiff = left.cost
    .input
    - right
      .cost
      .input;
  if (inputCostDiff !== 0)
    return inputCostDiff;

  return compareVersions({
    a: left,
    b: right,
  },);
}

/**
 Find fastest models across top major-version groups.
 
 `majorVersions`: one keeps latest only, two keeps latest plus previous, zero
 keeps all major versions. Models with no recognized speed signal still rank
 deterministically by input price and version.
 
 @param models - models to filter and sort
 
 @param majorVersions - number of major-version families to keep
 
 @returns fastest models sorted by name heuristic, input cost, then version
 
 @example
 ```typescript
 findFastestInMajorVersions({ models, majorVersions: 1 });
 ```
 */
export function findFastestInMajorVersions<TModel extends ModelPricing,>(
  {
    models,
    majorVersions,
  }: {
    readonly models: readonly TModel[];
    readonly majorVersions: number;
  },
): TModel[] {
  /**
   Eligible models from the same major-version filter used by budget selection.
   */
  const eligible = findCheapestInMajorVersions({
    models,
    majorVersions,
  },);

  return eligible.toSorted(function bySpeedThenCostThenVersion(
    left,
    right,
  ) {
    return compareModelSpeed({
      left,
      right,
    },);
  },);
}

export { scoreModelSpeed, } from './speed-signals.ts';

//endregion Public API
