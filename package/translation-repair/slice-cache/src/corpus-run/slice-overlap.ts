import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  isNegativeWholeNumberText,
  isWholeNumberText,
  WHOLE_NUMBER_RULE,
} from '@monochromatic-dev/translation-repair-markup/ts';

//region Slice overlap
// How many independent units a driver may have in flight at once.
//
// Fallback belongs to caller. Editor calibration defaults to four by measured
// decision; corpus pass defaults to four since its own matched pairs decided
// (`doc/decision/translation-repair-pass-overlap.md`, 2026-09-06).
// One environment dial can override either without a rebuild.

/**
 Environment variable naming how many slices may run at once.
 */
const OVERLAP_VAR = 'TRANSLATION_REPAIR_SLICE_OVERLAP';

/**
 Fewest slices a driver can keep in flight and still do work.
 */
const MINIMUM_OVERLAP = 1;

/**
 Slices editor calibration keeps in flight when nobody said.

 Four was decided from matched 2026-08-26 arms recorded in
 `doc/decision/translation-repair-calibration-overlap.md`.
 */
export const CALIBRATION_OVERLAP = 4;

/**
 Where overlap value came from, for launch logs.

 @example
 ```ts
 const source: OverlapSettingSource = 'fallback';
 ```
 */
export type OverlapSettingSource = 'fallback' | typeof OVERLAP_VAR;

/**
 One overlap reading beside its source.

 @example
 ```ts
 const setting: OverlapSetting = { overlap: 4, source: 'fallback', };
 ```
 */
export type OverlapSetting = {
  /**
   Slices allowed in flight.
   */
  readonly overlap: number;

  /**
   Fallback or environment variable that supplied value.
   */
  readonly source: OverlapSettingSource;
};

/**
 Reads overlap and names where it came from.

 Refuses rather than falls back on invalid environment input because matched
 arms must not silently become identical after a typo. A variable unset,
 empty or blank says nothing, as it does for every numeric dial in the
 package, and takes the fallback (ledger B73).

 DIGITS ONLY, by the package's one count rule (ledger B73), which also reads
 `04` as four; a minus sign before digits is answered as a value below one.

 @param fallback - slices in flight when environment says nothing

 @param env - environment of the launch, which the entry file hands in, so
 the process's own variables are read there and in no driver it feeds

 @returns Valid overlap beside fallback or variable source

 @throws StatedRefusalError when variable is not a whole number written in
 digits, or is below one

 @example
 ```ts
 const setting = readOverlapSetting({ fallback: CALIBRATION_OVERLAP, env: process.env, },);
 ```
 */
export function readOverlapSetting(
  {
    fallback,
    env,
  }: {
    readonly fallback: number;
    readonly env: Readonly<NodeJS.ProcessEnv>;
  },
): OverlapSetting {
  /**
   Value as invoking environment wrote it, empty when unset or empty.
   */
  const written = env[OVERLAP_VAR] ?? '';
  if (written.trim() === '') {
    return {
      overlap: fallback,
      source: 'fallback',
    };
  }

  if ((!isWholeNumberText({ text: written, },)) && (!isNegativeWholeNumberText({ text: written, },))) {
    throw new StatedRefusalError({
      says: `${OVERLAP_VAR} must be ${WHOLE_NUMBER_RULE}, and ${written} is not one`,
    },);
  }

  /**
   Value as written, which `Number` now reads exactly: digits, with or
   without a minus sign before them.
   */
  const asked = Number(written,);
  if (asked < MINIMUM_OVERLAP) {
    throw new StatedRefusalError({
      says: `${OVERLAP_VAR} must be at least ${String(MINIMUM_OVERLAP,)}, and ${written} is not`,
    },);
  }
  return {
    overlap: asked,
    source: OVERLAP_VAR,
  };
}

/**
 Reads how many slices may be in flight at once.

 Compatibility wrapper for callers that do not log setting source.

 @param fallback - slices in flight when environment says nothing

 @param env - environment of the launch, which the entry file hands in

 @returns Valid overlap

 @throws StatedRefusalError when variable is not a whole number written in
 digits, or is below one

 @example
 ```ts
 const overlap = readOverlap({ fallback: CALIBRATION_OVERLAP, env: process.env, },);
 ```
 */
export function readOverlap(
  {
    fallback,
    env,
  }: {
    readonly fallback: number;
    readonly env: Readonly<NodeJS.ProcessEnv>;
  },
): number {
  return readOverlapSetting({
    fallback,
    env,
  },)
    .overlap;
}

//endregion Slice overlap
