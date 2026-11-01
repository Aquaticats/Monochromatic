//region Stream bound configuration
// A STREAM BOUND THAT CANNOT FIRE BEFORE THE DEADLINE ENDS NOTHING (ledger
// B804). The first-content and idle windows of the stream guard
// (`stream-idle-guard.ts`) were 600_000 ms beside a per-call deadline of
// 360_000 ms, so the deadline ended every call the guards were meant to end, and
// the artifact of zheermao22 stamped the pair without anything having read it.
// The two numbers are set in two packages, so the check that they agree lives
// where both are read together, and it throws when the configuration is built,
// before a run spends anything.

/**
 Name and value of one setting the check read.

 @example
 ```ts
 const setting: StreamBoundSetting = { key: 'streamIdleMs', valueMs: 600_000, setAt: 'STREAM_IDLE_MS', };
 ```
 */
export type StreamBoundSetting = {
  /**
   Key the setting carries in the call configuration stamped on an artifact.
   */
  readonly key: string;

  /**
   Value the setting held.
   */
  readonly valueMs: number;

  /**
   Constant or place where the setting is set, named in the remedy.
   */
  readonly setAt: string;
};

/**
 Whether a number is a usable duration: finite and above zero.

 @param valueMs - number to test

 @returns Whether it is positive and finite

 @example
 ```ts
 const usable = isUsableDuration({ valueMs: 120_000, },); // true
 ```
 */
function isUsableDuration({ valueMs, }: { readonly valueMs: number; },): boolean {
  return Number.isFinite(valueMs,) && (valueMs > 0);
}

/**
 Raised when a stream bound is not shorter than the per-call deadline, or when
 a bound or the deadline is not a positive finite number.

 Names every offending setting with its value and the deadline, and says what
 to change, so the operator need not read the guard to learn why a run was
 refused before it began.

 @example
 ```ts
 throw new StreamBoundsOutsideCallTimeoutError({ perCallTimeoutMs: 360_000, offenders: [], },);
 ```
 */
export class StreamBoundsOutsideCallTimeoutError extends Error {
  /**
   Declares this message safe to forward: it names settings and durations.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds the refusal.

   @param perCallTimeoutMs - deadline the bounds were read against

   @param offenders - settings that are not shorter than the deadline, or are
   not usable durations, each with where it is set
   */
  constructor(
    {
      perCallTimeoutMs,
      offenders,
    }: {
      readonly perCallTimeoutMs: number;
      readonly offenders: readonly StreamBoundSetting[];
    },
  ) {
    /**
     What each offending setting is wrong with.
     */
    const faults = offenders
      .map(function describe({
        key,
        valueMs,
      },): string {
        return isUsableDuration({ valueMs, },)
          ? `${key} is ${String(valueMs,)} ms, which is not shorter than the deadline`
          : `${key} is ${String(valueMs,)}, which is not a positive finite number of milliseconds`;
      },)
      .join('; ',);
    /**
     Where the offending settings are set.
     */
    const places = offenders
      .map(function placeOf({ setAt, },): string {
        return setAt;
      },)
      .join(' and ',);
    super(
      `The stream bounds of this configuration cannot fire before the per-call deadline of ${
        String(perCallTimeoutMs,)
      } ms (perCallTimeoutMs), or the numbers are not usable: ${faults}. A guard whose window is not shorter than the deadline ends nothing the `
        + `deadline would not. Either lower ${places} or raise the per-call deadline (RUN_PER_CALL_TIMEOUT_MS in `
        + 'package/translation-repair/run/src/corpus-run/run-config.ts), so that each bound is a positive number of '
        + 'milliseconds shorter than the deadline, and the deadline is a positive finite number of milliseconds.',
    );
    this.name = 'StreamBoundsOutsideCallTimeoutError';
  }
}

/**
 Refuses a configuration whose stream bounds cannot fire before the per-call
 deadline.

 @param perCallTimeoutMs - deadline one model exchange is granted

 @param streamFirstContentMs - silence allowed before the first generated text

 @param streamIdleMs - silence allowed between pieces of generated text

 @throws {@link StreamBoundsOutsideCallTimeoutError} when a bound is equal to
 or longer than the deadline, when a bound is not a positive finite number
 (not a number, zero, negative, infinite), or when the deadline is not

 @example
 ```ts
 requireStreamBoundsInsideCallTimeout({ perCallTimeoutMs: 360_000, streamFirstContentMs: 180_000, streamIdleMs: 120_000, },);
 ```
 */
export function requireStreamBoundsInsideCallTimeout(
  {
    perCallTimeoutMs,
    streamFirstContentMs,
    streamIdleMs,
  }: {
    readonly perCallTimeoutMs: number;
    readonly streamFirstContentMs: number;
    readonly streamIdleMs: number;
  },
): void {
  if (!isUsableDuration({ valueMs: perCallTimeoutMs, },))
    throw new StreamBoundsOutsideCallTimeoutError({
      perCallTimeoutMs,
      offenders: [{
        key: 'perCallTimeoutMs',
        valueMs: perCallTimeoutMs,
        setAt: 'RUN_PER_CALL_TIMEOUT_MS',
      },],
    },);

  /**
   Both bounds, with where each is set.
   */
  const bounds: readonly StreamBoundSetting[] = [
    {
      key: 'streamFirstContentMs',
      valueMs: streamFirstContentMs,
      setAt: 'STREAM_FIRST_CONTENT_MS (package/translation-repair/provider-stream/src/stream-idle-guard.ts)',
    },
    {
      key: 'streamIdleMs',
      valueMs: streamIdleMs,
      setAt: 'STREAM_IDLE_MS (package/translation-repair/provider-stream/src/stream-idle-guard.ts)',
    },
  ];

  /**
   Bounds that are not usable durations or not shorter than the deadline.
   */
  const offenders = bounds.filter(function offends({ valueMs, },): boolean {
    return (!isUsableDuration({ valueMs, },)) || (valueMs >= perCallTimeoutMs);
  },);
  if (offenders.length > 0)
    throw new StreamBoundsOutsideCallTimeoutError({
      perCallTimeoutMs,
      offenders,
    },);
}

//endregion Stream bound configuration
