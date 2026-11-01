import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { isDecimalText, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Cap override
// Lets one invocation run under a different per-entry ceiling than the built-in
// one, without rebuilding.
//
// TWO USES, and only one of them is testing. A second way out for the largest entries is to
// raise the cap for entries above some slice count, and any such rule needs the
// ceiling to be settable before it can be a rule at all. The other is that the
// re-attempt queue only does anything to an entry the cap CUTS, so verifying it
// against the ceiling it ships with means finding an entry that needs thirteen
// hours. A lowered ceiling reproduces the same path in one.
//
// AN UNSET VARIABLE IS NOT AN OVERRIDE AND NEITHER IS AN EMPTY ONE. An
// exported-but-empty variable is an ordinary shell accident, and `run-config.ts`
// already carries the scar from treating one as meaningful.
//
// ANYTHING ELSE THROWS rather than falling back. A ceiling is what stops a
// runaway entry, so a misspelled value silently becoming the default would let
// an operator believe they had bounded a run that is not bounded the way they
// think. The failure has to be loud.

/**
 Environment variable overriding the per-entry ceiling, in minutes.
 */
export const HARD_CAP_VAR = 'TRANSLATION_REPAIR_HARD_CAP_MINUTES';

/**
 Milliseconds in a minute, which converts the override to the whole milliseconds a ceiling is stated in.
 */
const MS_PER_MINUTE = 60_000;

/**
 Smallest figure the override may state, written as the decimal minutes the message gives an operator: half a
 millisecond is the least that rounds to one whole millisecond, and this figure is a little over it so that
 floating-point multiplication cannot round it back to zero.
 */
const SMALLEST_FIGURE_MINUTES = '0.0000083334';

/**
 Raised when the override is present but is not a usable number of minutes.
 */
export class HardCapOverrideError extends StatedRefusalError {
  /**
   Declares this message safe to forward: it names the variable and repeats the value the operator set in it.
   */
  override readonly messageNamesOnly: true = true;

  /**
   Names the variable, what it held, and why that cannot be a ceiling.

   @param value - what the variable held, quoted back

   @param roundsToZero - whether the value is a positive number of minutes finer than half a millisecond, which
   the message then says along with the smallest figure accepted

   @example
   ```ts
   throw new HardCapOverrideError({ value: 'soon', },);
   ```
   */
  constructor(
    {
      value,
      roundsToZero = false,
    }: {
      readonly value: string;
      readonly roundsToZero?: boolean;
    },
  ) {
    super({
      says: roundsToZero
        ? `${HARD_CAP_VAR} must be a positive number of minutes, and a ceiling is read in whole milliseconds;`
          + ` received ${JSON.stringify(value,)}, which rounds to 0 milliseconds. The smallest figure accepted is`
          + ` half a millisecond, written as ${SMALLEST_FIGURE_MINUTES} minutes. This is the ceiling that stops one`
          + ' entry running away with a whole pass, so a figure it cannot state is refused rather than quietly'
          + ' replaced by another: an operator who set it believes the run is bounded the way they asked for.'
        : `${HARD_CAP_VAR} must be a positive number of minutes; received ${JSON.stringify(value,)}.`
          + ' This is the ceiling that stops one entry running away with a whole pass, so an'
          + ' unreadable value is refused rather than quietly replaced by the default: an operator'
          + ' who set it believes the run is bounded the way they asked for.',
    },);
    this.name = 'HardCapOverrideError';
  }
}

/**
 Reads the per-entry ceiling this invocation runs under.

 @param fallback - built-in ceiling, used when nothing overrides it

 @param raw - override text, which the entry file reads from its launch's
 environment and hands in, so no library code reads the process's own (ledger
 B398). UNSET AND EMPTY COLLAPSE TO ONE VALUE here rather than being
 distinguished, because this module already treats them alike: neither is an
 override. That is why the parameter is a plain string and the entry file
 supplies `''` for an absent variable

 @returns Minutes one entry may run before its exchanges abort

 @throws {@link HardCapOverrideError} when the override is present and is not
 a positive finite number, or is positive and finer than half a millisecond,
 which the pass rounds to zero whole milliseconds and the provider budgets'
 hold bound refuses. The figure is refused here, with the smallest figure
 accepted, so no figure an operator wrote is replaced by another

 @example
 ```ts
 const minutes = resolveHardCapMinutes({ fallback: HARD_CAP_MINUTES, raw: process.env[HARD_CAP_VAR] ?? '', },);
 ```
 */
export function resolveHardCapMinutes(
  {
    fallback,
    raw,
  }: {
    readonly fallback: number;
    readonly raw: string;
  },
): number {
  if (raw.trim() === '')
    return fallback;

  /**
   Override read as a number, which `Number` reports as NaN for anything that
   is not one. `Number` rather than `parseFloat`, because `parseFloat` reads
   a leading number out of `30minutes` and would accept a typo as 30.
   */
  const minutes = Number(raw,);

  // A PLAIN DECIMAL, as an operator writes minutes: `Number` also read `0x10`,
  // `1e1`, `+15`, ` 15` and `.5` as minutes nobody wrote that way (ledger B73).
  if ((!isDecimalText({ text: raw, },))
    || (!Number.isFinite(minutes,))
    || (minutes <= 0))
    throw new HardCapOverrideError({ value: raw, },);

  // A POSITIVE FIGURE THAT ROUNDS TO ZERO WHOLE MILLISECONDS names no ceiling a timer can state. The pass rounds
  // the product once to whole milliseconds (`corpus-pass.ts`), and a floor of one millisecond there would print a
  // figure the operator never wrote, so the figure is refused here instead.
  if (Math.round(minutes * MS_PER_MINUTE,) < 1)
    throw new HardCapOverrideError({
      value: raw,
      roundsToZero: true,
    },);

  return minutes;
}

/**
 Whether a ceiling leaves room for one full-length model exchange.

 MEASURED THE HARD WAY on 2026-08-24. A verification run set the ceiling to
 five minutes while `RUN_PER_CALL_TIMEOUT_MS` allowed six for a single
 exchange. Both attempts were cut mid-exchange, so no exchange ever returned,
 so no slice ever cached, so the queue read no progress and dropped the entry
 as stalled. Nothing in the run said why, and the ceiling looked reasonable.

 @param capMs - ceiling one attempt runs under

 @param perCallMs - deadline one exchange is allowed

 @param claudeCallMs - deadline a Claude exchange is allowed, which the ceiling must outlast too

 @returns Whether the longest exchange can finish inside an attempt

 @example
 ```ts
 const roomy = capOutlastsOneCall({ capMs, perCallMs, claudeCallMs, },);
 ```
 */
export function capOutlastsOneCall(
  {
    capMs,
    perCallMs,
    claudeCallMs,
  }: {
    readonly capMs: number;
    readonly perCallMs: number;
    readonly claudeCallMs: number;
  },
): boolean {
  return capMs > Math.max(
    perCallMs,
    claudeCallMs,
  );
}

/**
 Explains a ceiling no exchange can finish inside.

 WARNED RATHER THAN REFUSED, and the reason is that cutting mid-exchange is
 exactly what a test of the stall path wants. Refusing would have blocked the
 run that found this. An operator who meant it keeps their ceiling and reads
 why every attempt will report no progress.

 @param capMs - ceiling one attempt runs under

 @param perCallMs - deadline one exchange is allowed

 @param claudeCallMs - deadline a Claude exchange is allowed

 @returns Line naming the three numbers and what follows from them

 @example
 ```ts
 console.log(capTooTightNote({ capMs, perCallMs, claudeCallMs, },),);
 ```
 */
export function capTooTightNote(
  {
    capMs,
    perCallMs,
    claudeCallMs,
  }: {
    readonly capMs: number;
    readonly perCallMs: number;
    readonly claudeCallMs: number;
  },
): string {
  /**
   Deadline of the longest exchange, which the ceiling must outlast.
   */
  const longestMs = Math.max(
    perCallMs,
    claudeCallMs,
  );
  return `CAP TOO TIGHT: an attempt runs ${String(capMs,)}ms, which is not longer than the `
    + `${String(longestMs,)}ms one model exchange is allowed `
    + `(${String(perCallMs,)}ms by default, ${String(claudeCallMs,)}ms for a Claude exchange). Attempts are cut `
    + 'before an exchange can return, so no slice caches, every attempt reports no progress, and the '
    + 'queue drops the entry as stalled after its second try. Raise the ceiling above one '
    + 'exchange to buy anything, or keep it here to exercise the stall path deliberately.';
}

//endregion Cap override
