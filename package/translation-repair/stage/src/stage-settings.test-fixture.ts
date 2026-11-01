import type { StageSettings, } from '../dist/final/node/index.mjs';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

//region Stage settings fixture
// THE SETTINGS A CASE OVER SCRIPTED WORK HANDS EVERY STAGE it drives when the
// case is not about them. The stages take the straggler windows and the
// contest ledger as a value the entry files read from the environment, so a
// case says which windows its rounds wait under rather than inheriting
// whatever the shell running the suite exported, and records no contest
// unless it names where.
//
// BOTH WINDOWS ARE THE HANG STOP, the one real-clock bound a case may set over
// scripted work (`hang-stop.test-fixture.ts`): every scripted seat answers on
// its own, so no round waits out its window, and a seat that never answered
// would fail the case within a minute. The stages read these windows where
// they once read the built-in ones off the environment, 120000 ms for every
// round and 180000 ms for the writers, which no scripted case reached either.
// A case whose rounds must wait some other window names it as `graceMs`, the
// key `real-clock-bounds.unit.test.ts` reads, through `scriptedStageSettingsWaiting`.

/**
 Settings whose every round, writers' included, waits the window a case names,
 and which keep no contest.

 @param graceMs - window every round waits on stragglers after quorum

 @returns Settings to hand the stages a case drives

 @example
 ```ts
 const stageSettings = scriptedStageSettingsWaiting({ graceMs: 0, },);
 ```
 */
export function scriptedStageSettingsWaiting({ graceMs, }: { readonly graceMs: number; },): StageSettings {
  return {
    grace: {
      writerMs: graceMs,
      roundMs: graceMs,
      source: 'round-window',
    },
    contestLedger: { kind: 'unkept', },
  };
}

/**
 Hang-stop windows for every round, writers' included, and no contest kept.

 @example
 ```ts
 const outcome = await selectBestCandidate({ ...request, stageSettings: SCRIPTED_STAGE_SETTINGS, },);
 ```
 */
export const SCRIPTED_STAGE_SETTINGS: StageSettings = scriptedStageSettingsWaiting({ graceMs: HANG_STOP_MS, },);

//endregion Stage settings fixture
