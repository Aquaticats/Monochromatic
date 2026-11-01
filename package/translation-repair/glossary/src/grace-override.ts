import { StatedRefusalError, } from './stated-refusal.ts';
import { isWholeNumberText, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Grace override
// Lets one invocation run its stage rounds under a different straggler window
// than the built-in one, without rebuilding.
//
// ONE USE, and it is measurement. A measurement ranked lengthening the window as the
// first lever left against lost voices, since every cut stream in the last
// production run was a model still reasoning when the window closed, and the
// only way to price a longer window is a run under one matched against a run
// at the built-in window. The dial makes those two runs one build apart, the
// way `TRANSLATION_REPAIR_SLICE_OVERLAP` does for the overlap measurement.
//
// AN UNSET VARIABLE IS NOT AN OVERRIDE AND NEITHER IS AN EMPTY ONE, for the
// reason `cap-override.ts` gives: an exported-but-empty variable is a shell
// accident rather than an intention.
//
// ANYTHING ELSE REFUSES rather than falling back, and it refuses as a STATED
// refusal. The whole point of the dial is that two runs differ in it and in
// nothing else, so a mistyped value that quietly became the built-in window
// would produce two matched runs and a recorded conclusion that the window
// buys nothing. That is an operator's input to correct, not a fault to locate,
// so the command declines in one line and prints no frames for it.

/**
 Environment variable overriding the straggler window, in milliseconds.
 */
export const STRAGGLER_GRACE_VAR = 'TRANSLATION_REPAIR_STRAGGLER_GRACE_MS';

/**
 Straggler window the editor calibration runs under when nobody set one.

 FIVE MINUTES, DECIDED ON MEASUREMENT (`doc/decision/translation-repair-calibration-overlap.md`):
 under four slices in flight, arm D ran this window at the same normalized
 cost as arm B's built-in window and cut 2 voices against B's 7, because the
 wait a longer window adds is what overlap fills. The corpus pass keeps
 `STRAGGLER_GRACE_MS` (`doc/decision/translation-repair-straggler-grace.md`),
 though it too keeps four slices in flight since 2026-09-06.
 */
export const CALIBRATION_STRAGGLER_GRACE_MS = 300_000;

/**
 Where a calibration's window came from.
 */
export type CalibrationGrace = {
  /**
   Milliseconds the rounds wait on stragglers after quorum.
   */
  readonly effectiveMs: number;

  /**
   `calibration-default` when nothing was set and the calibration's own window
   was adopted; `override` when a launch set the variable.
   */
  readonly source: 'calibration-default' | 'override';
};

/**
 Reads the straggler window this invocation's rounds run under.

 @param fallback - built-in window, used when nothing overrides it

 @param raw - override text as the launch's environment holds it, `''` for an
 absent variable, since unset and empty are alike here; REQUIRED, so the
 environment is read where an entry file reads it and in no stage it feeds

 @returns Milliseconds a round keeps waiting on stragglers after quorum

 @throws {@link StatedRefusalError} when the override is present and is not
 a positive finite number of milliseconds

 @example
 ```ts
 const graceMs = resolveStragglerGraceMs({ fallback: STRAGGLER_GRACE_MS, raw: env[STRAGGLER_GRACE_VAR] ?? '', },);
 ```
 */
export function resolveStragglerGraceMs(
  {
    fallback,
    raw,
  }: {
    readonly fallback: number;
    readonly raw: string;
  },
): number {
  return readWindowDial({
    variable: STRAGGLER_GRACE_VAR,
    fallback,
    raw,
    unsetMeans: 'run under the built-in window',
  },);
}

/**
 Longest delay a JavaScript timer holds, 2^31 - 1 ms, about 24.8 days.

 `setTimeout` clamps anything above it to 1 ms with a TimeoutOverflowWarning,
 so a window past it would cut every straggler the instant quorum stood, the
 opposite of what an operator who typed a large number asked for. A fraction
 is refused for the same reason from the other side: timers round it, so the
 window that ran is not the window that was set.
 */
export const MAX_TIMER_DELAY_MS = 2_147_483_647;

/**
 Whether a number is a window a timer can hold as written.

 @param ms - candidate window

 @returns Whether it is a whole number of milliseconds a timer holds exactly

 @example
 ```ts
 isTimerWindow({ ms: 180_000, },);
 // => true
 ```
 */
export function isTimerWindow({ ms, }: { readonly ms: number; },): boolean {
  return Number.isInteger(ms,)
    && (ms > 0)
    && (ms <= MAX_TIMER_DELAY_MS);
}

/**
 Reads one window dial's text the way every window dial must read it.

 ONE READER FOR EVERY DIAL, because the refusal rule is the dial's whole
 value: `writer-grace-override.ts` reads a second variable by the same rule,
 and two hand-copied readers would be two places for `300s` to start meaning
 300 in one and a refusal in the other.

 @param variable - environment variable the text came from, named in the
 refusal so the operator corrects the right one

 @param fallback - window used when the text is blank; checked too, since a
 caller passing a window no timer holds is a defect this reader must not
 launder into a run

 @param raw - override text; blank means unset

 @param unsetMeans - what leaving the variable unset does, for the refusal

 @returns Milliseconds the dial names, or the fallback when it names nothing

 @throws {@link StatedRefusalError} when the text is present and is not a
 whole number of milliseconds a timer can hold

 @throws {@link RangeError} when the fallback itself is not such a number

 @example
 ```ts
 const ms = readWindowDial({ variable, fallback, raw, unsetMeans: 'run under the built-in window', },);
 ```
 */
export function readWindowDial(
  {
    variable,
    fallback,
    raw,
    unsetMeans,
  }: {
    readonly variable: string;
    readonly fallback: number;
    readonly raw: string;
    readonly unsetMeans: string;
  },
): number {
  if (!isTimerWindow({ ms: fallback, },))
    throw new RangeError(
      `fallback window for ${variable} must be a whole number of milliseconds from 1 to `
        + `${String(MAX_TIMER_DELAY_MS,)}, and ${String(fallback,)} is not`,
    );

  if (raw.trim() === '')
    return fallback;

  /**
   Override read as a number, which `Number` reports as NaN for anything that
   is not one. `Number` rather than `parseFloat`, because `parseFloat` reads
   a leading number out of `300s` and would accept a typo as 300.
   */
  const ms = Number(raw,);

  // DIGITS AS AN OPERATOR WRITES A COUNT OF MILLISECONDS: `Number` also read
  // `0x3E8`, `1e3`, `+1000`, ` 1000` and `1000.0` as one second (ledger B73).
  if ((!isWholeNumberText({ text: raw, },)) || (!isTimerWindow({ ms, },)))
    throw new StatedRefusalError({
      says: `${variable} must be a whole number of milliseconds from 1 to ${
        String(MAX_TIMER_DELAY_MS,)
      }, and ${JSON.stringify(raw,)} is not; leave it unset to ${unsetMeans}`,
    },);

  return ms;
}

/**
 Explains which window a run is under when it is not the built-in one.

 PRINTED BY THE DRIVERS RATHER THAN THE ROUND, because a run must never hide
 which window it ran under: a round's own log names the window only when it
 cuts a voice, and a run that cut nobody under a longer window is exactly the
 run the measurement wants to hear about.

 @param effectiveMs - window the rounds run under

 @param builtInMs - window the code ships with

 @returns Note naming both windows, or nothing when they agree

 @example
 ```ts
 const note = graceOverrideNote({ effectiveMs, builtInMs: STRAGGLER_GRACE_MS, },);
 ```
 */
export function graceOverrideNote(
  {
    effectiveMs,
    builtInMs,
  }: {
    readonly effectiveMs: number;
    readonly builtInMs: number;
  },
): string {
  if (effectiveMs === builtInMs)
    return '';

  return `STRAGGLER GRACE OVERRIDDEN by ${STRAGGLER_GRACE_VAR}: rounds abandon stragglers `
    + `${String(effectiveMs,)}ms after quorum rather than the built-in ${String(builtInMs,)}ms`;
}

/**
 Puts the editor calibration under its own window unless a launch set one.

 RETURNED, NEVER WRITTEN BACK. This once set the variable when it was unset,
 so every stage round, reading its window off the environment, picked the
 calibration's window up by the path a launch has. The rounds now take their
 windows as a value the entry hands down (`StageSettings`,
 `stage-settings.ts`), so the calibration hands its rounds the window read
 with this fallback (`readWriterGrace` with `CALIBRATION_STRAGGLER_GRACE_MS`)
 and nothing writes the process's environment. A launch that set the
 variable is honored as an override, refused if unreadable, exactly as before.

 @param env - environment of the launch, which the entry file hands in

 @returns Window the calibration's rounds run under, and where it came from

 @throws {@link StatedRefusalError} when a launch set the variable to
 something that is not a positive finite number of milliseconds

 @example
 ```ts
 const grace = adoptCalibrationGrace({ env: process.env, },);
 console.log(`straggler window ${String(grace.effectiveMs,)}ms (${grace.source})`,);
 ```
 */
export function adoptCalibrationGrace(
  { env, }: { readonly env: Readonly<NodeJS.ProcessEnv>; },
): CalibrationGrace {
  /**
   What the launch set, empty when it set nothing.
   */
  const written = env[STRAGGLER_GRACE_VAR] ?? '';

  if (written.trim() === '') {
    return {
      effectiveMs: CALIBRATION_STRAGGLER_GRACE_MS,
      source: 'calibration-default',
    };
  }
  return {
    effectiveMs: resolveStragglerGraceMs({
      fallback: CALIBRATION_STRAGGLER_GRACE_MS,
      raw: written,
    },),
    source: 'override',
  };
}

//endregion Grace override
