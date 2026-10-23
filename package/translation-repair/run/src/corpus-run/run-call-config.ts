import {
  requireStreamBoundsInsideCallTimeout,
  STREAM_FIRST_CONTENT_MS,
  STREAM_IDLE_MS,
} from '@monochromatic-dev/translation-repair-provider-stream/ts';
import {
  CLAUDE_COMPLETION_CAP,
  claudeCallDeadlineMs,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Run call config
// The deadline and the stream bounds one pass grants every model exchange, and the stamp that records them on each
// artifact. Split from `run-config.ts` for its line budget.

/**
 Deadline granted to one model exchange during a corpus run.

 Was 240_000, which measurably clipped real work. Run 013 sampled every call
 unfiltered: 748 succeeded and 35 were cut at the deadline, a 4.5 percent
 censoring rate, while the surviving time-to-first-byte distribution ran p50
 45_837 ms, p90 163_296 ms, p99 218_976 ms, and max 235_151 ms. Fifteen calls
 landed in the last 25 seconds before the cut, so the distribution had real
 density right up to the boundary with NO cliff ahead of it. That is the
 signature of clipping, not of connections hanging: a call completing at
 245_000 ms would be unremarkable beside the ones observed at 235_151 ms.

 Timeouts still arrive in correlated batches, about five per retry round,
 which once looked like evidence of hangs. It reconciles if the provider slows
 every concurrent call together under load, so a batch crosses the deadline
 together. That explains the correlation without hangs, and it means the added
 waiting falls during congested periods specifically.

 360_000 is chosen against the measurement rather than as a round multiple: it
 clears the observed p99 by 64 percent and the observed maximum by 53 percent,
 while keeping a worst-case stage bounded. `STAGE_RETRY_ROUNDS` allows four
 deadlines in one stage, so this caps a pathological stage near 24 minutes
 (the recovery round after them waits only a straggler window), where 480_000
 would put it past 32. The per-entry ceiling it was weighed against was 90
 minutes when this was written; it has been 420 since 2026-08-17
 (`HARD_CAP_MINUTES` in `corpus-pass.ts`), and this comment said 90 until
 ledger D6.
 Raising it should also REDUCE retry rounds by losing fewer voices, so the
 worst case gets rarer as well as no worse.

 Sampling stays unfiltered, so the next run reports how much tail still gets
 clipped at 360_000 and this can be tuned on evidence again.
 */
export const RUN_PER_CALL_TIMEOUT_MS = 360_000;

/**
 Call-timing knobs an artifact was produced under, so a pool spanning more
 than one configuration can still be analyzed per cohort.

 @example
 ```ts
 const config: RunCallConfig = {
   perCallTimeoutMs: 240_000,
   streamFirstContentMs: 150_000,
   streamIdleMs: 60_000,
   claudeCallTimeoutMs: 460_000,
 };
 ```
 */
export type RunCallConfig = {
  /**
   Total-duration deadline one model exchange was granted.
   */
  readonly perCallTimeoutMs: number;

  /**
   Total-duration deadline a Claude exchange ran under: the larger of
   `perCallTimeoutMs` and the deadline a Claude card's completion cap needs at
   the floor rate (`claudeCallDeadlineMs`), which the Claude client arms in
   place of a shorter one. Absent on artifacts stamped before this field
   existed, whose Claude calls ran under that same larger deadline without the
   stamp saying so.
   */
  readonly claudeCallTimeoutMs: number;

  /**
   Silence allowed before a stream's first content or reasoning delta. Named
   `streamFirstByteMs` on artifacts written before ledger B804, when the
   window counted bytes of any kind.
   */
  readonly streamFirstContentMs: number;

  /**
   Silence allowed between a stream's content or reasoning deltas once flowing.
   */
  readonly streamIdleMs: number;
};

/**
 Builds the call-timing knobs, refusing a set whose stream bounds cannot fire
 before the per-call deadline.

 @param perCallTimeoutMs - deadline one model exchange is granted

 @param streamFirstContentMs - silence allowed before the first generated text

 @param streamIdleMs - silence allowed between pieces of generated text

 @returns Three knobs, unchanged, and the deadline a Claude exchange runs
 under beside them, to stamp on an artifact

 @throws {@link StreamBoundsOutsideCallTimeoutError} when a bound is not a
 positive finite number shorter than the deadline (ledger B804)

 @example
 ```ts
 const config = runCallConfigOf({ perCallTimeoutMs: 360_000, streamFirstContentMs: 180_000, streamIdleMs: 120_000, },);
 ```
 */
export function runCallConfigOf(
  {
    perCallTimeoutMs,
    streamFirstContentMs,
    streamIdleMs,
  }: Omit<RunCallConfig, 'claudeCallTimeoutMs'>,
): RunCallConfig {
  requireStreamBoundsInsideCallTimeout({
    perCallTimeoutMs,
    streamFirstContentMs,
    streamIdleMs,
  },);
  return {
    perCallTimeoutMs,
    streamFirstContentMs,
    streamIdleMs,
    claudeCallTimeoutMs: Math.max(
      perCallTimeoutMs,
      claudeCallDeadlineMs({ maxTokens: CLAUDE_COMPLETION_CAP, },),
    ),
  };
}

/**
 Call-timing configuration stamped into every artifact this pass writes.

 The pool it labels is deliberately MIXED, by a decision the user made twice:
 keep already-settled entries rather than discard the compute.

 The version of this note in commit `e819e7ff2e` (2026-07-26) promised more than the stamp can deliver: it
 said precision could be split by cohort at analysis time, turning the
 confound into a number. RETRACTED in commit `6173cb82d1` (2026-07-27), because the arithmetic does not support
 it. The graded sample is 50 items and the pool at the coverage bar is about
 30 entries split near evenly between cohorts, so a per-cohort precision
 estimate rests on roughly 25 graded items and carries a standard error near
 8 points. A difference small enough to matter cannot resolve at that width,
 and the binding constraint is human grading effort, not compute, so widening
 the sample to the several hundred per cohort that would resolve it is not
 available. Claiming the number anyway would repeat the exact error retracted
 from the panel-coverage analysis: pooling across a noisy dimension and
 reading the result as signal.

 What the stamp is still for: identifying which cohort any artifact came from,
 so the mixed pool is disclosed QUALITATIVELY with the verdict rather than
 left unstated, and so a later analysis over a larger graded set is possible
 if one is ever funded.

 Three cohorts exist in the round-two pool, and the first two are equivalent
 for call timing even though they look different:

 -   Ten entries with NO `callConfig` field at all, settled before the field
     existed. Their absence identifies them exactly.
 -   Five entries stamped `perCallTimeoutMs: 240_000`, from run 013. The
     stream idle guard existed during this run but fired ZERO times, so these
     five ran under the same effective timing as the ten with no `callConfig`. Treat the
     fifteen as ONE cohort.
 -   Entries stamped `perCallTimeoutMs: 360_000` and later, which are the
     first to run without the deadline clipping roughly 4.5 percent of calls.

 Deliberately not surfaced on the grading sheet: a grader who could see which
 cohort an issue came from would be a worse instrument than one who could not.
 */
export const RUN_CALL_CONFIG: RunCallConfig = runCallConfigOf({
  perCallTimeoutMs: RUN_PER_CALL_TIMEOUT_MS,
  streamFirstContentMs: STREAM_FIRST_CONTENT_MS,
  streamIdleMs: STREAM_IDLE_MS,
},);

//endregion Run call config
