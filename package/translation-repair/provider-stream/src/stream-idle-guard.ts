import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  contextRoot,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import { monotonicMs, } from '@monochromatic-dev/translation-repair-markup/ts';
import type { ProviderName, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Stream idle guard
// Distinguishes a dead stream from a long one, which a total-duration deadline
// cannot do.
//
// Measured on the twelve pass-7 corpus logs: 14 stage rounds of 417 expired at
// the 240 s per-call deadline, and every one of them lost 4, 5, 6, or 7 of its
// 7 voices at once, never just one or two. Model-specific slowness does not
// have that shape. The retry settles it: of 13 retry rounds, 12 recovered a
// full 7/7 in 27 to 233 s, median 88 s, so the very voices that could not
// answer inside 240 s answered inside 88 s on a fresh dispatch. The wait was
// not buying generation progress, so a LARGER total deadline would only scale
// the waste; what is needed is noticing that nothing is arriving.
//
// An idle guard aborts on silence rather than on elapsed time, so a healthy
// long generation runs as long as it keeps emitting while a stalled one is cut
// early. Because the abort fires on a locally owned controller and never on the
// caller's signal, `attemptExchange` sees a non-abort failure and
// `exchangeWithRetry` retries it at transport level on a ~1 s backoff, instead
// of the stall escalating into another whole stage round.
//
// SILENCE IS THE ABSENCE OF GENERATED TEXT, NOT OF BYTES (ledger B804). A
// content delta or a reasoning delta resets the window; a comment line
// (OpenRouter writes `: OPENROUTER PROCESSING` while it waits for the upstream
// provider), a blank keep-alive line and an event with an empty delta do not.
// In zheermao22 eleven `inception/mercury-2.5` streams sent only such bytes, so
// the guard that timed bytes never fired and each was cut by the 120 s
// straggler grace after quorum.

/**
 Silence allowed before the first generated text, 180_000 ms, set BELOW
 `RUN_PER_CALL_TIMEOUT_MS` (360_000 in `corpus-run/run-config.ts`) so it can fire
 before the deadline does; `requireStreamBoundsInsideCallTimeout` refuses a
 configuration where it cannot. It was 600_000 under the name
 `STREAM_FIRST_BYTE_MS`, above the deadline, so it never fired.

 MEASURED, 2026-10-21 (ledger B804), over every pass log in the agent
 directory from CuspariaKLSY16 to zheermao22 except the one then being written:
 19542 completed streams that delivered text. The logs record the first BYTE
 and not the first text, so the first byte is a lower bound on the wait this
 bound must clear: on the 11748 streams of providers that send no comment
 lines (Synthetic, Hyper, NeuralWatt, Bedrock) the first byte ran p50 1915 ms,
 p99 11755 ms, p99.9 88110 ms and at most 115725 ms (a `minimax-m3` call
 on Hyper), with 8 streams past 90 s, 5 past 100 s and none past 120 s; the
 7794 OpenRouter streams ran at most 11219 ms to the first byte, which a
 comment line can be, so for them the first byte says nothing of the wait for text.

 THE MARGIN OVER A LOWER BOUND CAN HIDE A NEGATIVE ONE. A window of 120_000 ms
 stood 3.7 percent over the largest first byte, and the wait for text it
 bounds is at least that long, so a healthy slow stream could have been cut.
 At 180_000 ms the margin over the largest first byte is 56 percent. The
 99.9th percentile (88 s) would have cut 8 healthy streams to be sent again
 into a queue that had already cost them 90 to 116 s, and a cut here costs a
 transport re-dispatch of that one call, not a voice. The sample is censored
 from above by the 120 s straggler grace and the 360 s deadline, and the cut
 streams that delivered text show a first byte of at most 48854 ms, so the
 censored part does not hide a slower healthy start. The real figure is the
 `firstContentMs` of the progress line `reportStreamProgress` writes for each
 stream, which the next reading of the logs sets this bound against.

 THE OLDER TAIL EVIDENCE IS RETIRED, not contradicted. Two completions of
 retired rosters, a `hf:zai-org/GLM-5.2` stream at 183_755 ms and a pass-7
 completion at 347_099 ms
 (`doc/audit/stream-guards-first-production-traffic.md`,
 `doc/handover/translation-repair-history.md`), would still be cut by this
 bound. The roster that produced them is gone, and the deadline they were set
 against is the one this bound must remain shorter than. See
 `doc/decision/translation-repair-runaway-call-termination.md` for the
 re-arming question this answers.

 A MODEL'S CARD MAY NAME A SHORTER WINDOW OF ITS OWN (ledger B809): an
 OpenRouter card's `firstContentMs` reaches the guard through
 `TransportExchange.firstContentMs`, and only `inception/mercury-2.5` names one
 (90_000 ms). In `XingZ6017` twenty-six rounds waited on its streams that
 sent only comment lines (twenty-eight such streams, two rounds holding two
 each); a round whose quorum is every live seat cannot
 start its straggler grace, so this guard is the only bound that ends such a
 call, and the 4,636 streams of that model that delivered text took at most
 56,772 ms. Every other model keeps this window.
 */
export const STREAM_FIRST_CONTENT_MS = 180_000;

/**
 Silence allowed between generated pieces of text once the first has arrived,
 120_000 ms, also below `RUN_PER_CALL_TIMEOUT_MS`.

 MEASURED over the same 19542 streams: the largest silence between two chunks
 of a completed stream was 93736 ms (a `minimax-m3` call on Hyper), p99.9
 59008 ms, with one stream past 90 s and none past 100 s. The cut streams that
 delivered text show at most 61642 ms (a `hf:Qwen/Qwen3.8-27B` pause that
 recurs near 60 s), so the censored part adds no longer healthy silence. The
 bound is the largest observed silence plus 28 percent. The largest silence
 the older logs recorded, 124_992 ms in a completed `hf:zai-org/GLM-5.2`
 stream of a retired roster, would be cut now; each earlier larger sample
 found a larger gap (733 ms, 24_673 ms, 124_992 ms), and a tail not yet seen
 costs one re-dispatch of that call.
 */
export const STREAM_IDLE_MS = 120_000;

/**
 Logger root for the stream guard.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Characters of generated text a stream has delivered, per channel.

 @example
 ```ts
 const generated: GeneratedChars = { content: 8, reasoning: 0, };
 ```
 */
export type GeneratedChars = {
  /**
   Characters of the answer channel.
   */
  readonly content: number;

  /**
   Characters of the reasoning channel.
   */
  readonly reasoning: number;
};

/**
 A count of characters with the kind it counts, the noun agreeing with the
 count.

 @param count - characters counted

 @param kind - which characters

 @returns Phrase such as `8 content chars`

 @example
 ```ts
 const phrase = describeChars({ count: 1, kind: 'raw', },); // '1 raw char'
 ```
 */
function describeChars(
  {
    count,
    kind,
  }: {
    readonly count: number;
    readonly kind: 'content' | 'raw' | 'reasoning';
  },
): string {
  return `${String(count,)} ${kind} ${
    wordForCount({
      count,
      one: 'char',
      many: 'chars',
    },)
  }`;
}

/**
 Raised when a stream produced no generated text, content or reasoning, for
 longer than its window. Bytes that carry none (a comment line, a blank
 keep-alive line, an event with an empty delta) do not end the silence.
 Distinct from a caller abort so the retry layer treats it as transient
 weather and re-dispatches, which is the response the measurements support.

 @example
 ```ts
 throw new StreamStalledError({ label: 'hf:model', idleMs: 60_000, phase: 'body', },);
 ```
 */
export class StreamStalledError extends Error {
  /**
   Declares this message safe to forward: it names the model, the phase and a duration.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds the stalled-stream failure.

   @param label - names the stalled call

   @param idleMs - silence window that expired

   @param phase - whether the silence preceded the first generated text or
   interrupted a flowing stream
   */
  constructor(
    {
      label,
      idleMs,
      phase,
    }: {
      readonly label: string;
      readonly idleMs: number;
      readonly phase: 'first-content' | 'body';
    },
  ) {
    super(
      `Stalled: ${label} emitted no content or reasoning for ${String(idleMs,)}ms (${phase})`,
    );
    this.name = 'StreamStalledError';
  }
}

/**
 What one drained stream did over time, for tuning the idle windows.

 @example
 ```ts
 const progress: StreamProgress = { firstByteMs: 812, firstContentMs: 1_340, maxGapMs: 43, chars: 9_211, elapsedMs: 4_210, };
 ```
 */
export type StreamProgress = {
  /**
   Milliseconds from arming to the first body byte; negative when no byte
   ever arrived, which is itself the diagnostic.
   */
  readonly firstByteMs: number;

  /**
   Milliseconds from arming to the first generated text, content or
   reasoning; negative when none ever arrived, which with a first byte that
   did is the diagnostic of a stream that sent only comment lines or empty
   events.

   THE QUANTITY THE FIRST-CONTENT WINDOW BOUNDS (ledger B804). The first byte
   is only a lower bound on it, so the window was set against a figure that
   could understate the wait; this reading is the wait itself, on the same
   clock as the first byte.
   */
  readonly firstContentMs: number;

  /**
   Longest silence observed between consecutive body chunks.
   */
  readonly maxGapMs: number;

  /**
   Total decoded characters received.
   */
  readonly chars: number;

  /**
   Milliseconds from arming the guard to this reading.

   ANSWERS WHERE A RUN'S HOURS WENT, which nothing could before the timing work.
   Read on `monotonicMs`, like the first byte and the gaps, so setting the
   system clock mid-call cannot move it (ledger B78).
   Dispatch is logged at `debug` and production runs emit `info` and `warn`
   only, so a stream's start time was unrecoverable and the only concurrency
   figure derivable was a clustering of completion timestamps, which cannot
   tell calls that overlapped from calls that merely finished near each other.

   CARRIED HERE RATHER THAN LOGGED AT DISPATCH because it answers strictly
   more for no extra lines: start time follows by subtraction, while a
   dispatch line cannot be paired to its completion when one model has
   several calls in flight.
   */
  readonly elapsedMs: number;
};

/**
 Live guard over one exchange: a signal to hand the request, one notification
 per chunk of bytes, one per piece of generated text, and the progress it
 observed.
 */
export type IdleGuard = Disposable & {
  /**
   Signal that aborts when the idle window expires; its abort reason is the
   {@link StreamStalledError} describing which window expired.
   */
  readonly signal: AbortSignal;

  /**
   Records that bytes arrived: the first byte, the gaps between chunks and the
   raw characters, which are the wire's bookkeeping. THE WINDOW IS NOT RESET
   BY THIS, since a comment line, a blank keep-alive line and an event with an
   empty delta are bytes that carry no text (ledger B804).
   */
  readonly notify: (chars: number,) => void;

  /**
   Records that generated text arrived, content or reasoning, resetting the
   window. Called with the running totals of each channel, so the line that
   names a cut can say how much text there had been; call it only when a total
   has grown.
   */
  readonly notifyGenerated: (generated: GeneratedChars,) => void;

  /**
   Snapshot of what the stream has done so far.
   */
  readonly progress: () => StreamProgress;
};

/**
 Arms an idle guard. Arm it BEFORE the request so the window also covers a
 provider that never sends response headers at all, which a body-only guard
 would miss entirely.

 @param label - names the call in the stall error, which for a chat exchange
 is the model id

 @param provider - names the provider that serves the call in the line that
 reports a cut, absent where the caller does not know it

 @param firstContentMs - silence allowed before the first generated text, the
 window and not the `firstContentMs` reading of {@link StreamProgress}

 @param idleMs - silence allowed between pieces of generated text afterwards

 @returns Guard whose disposal clears its timer

 @example
 ```ts
 using guard = armIdleGuard({ label: modelId, },);
 ```
 */
export function armIdleGuard(
  {
    label,
    provider,
    firstContentMs = STREAM_FIRST_CONTENT_MS,
    idleMs = STREAM_IDLE_MS,
  }: {
    readonly label: string;
    readonly provider?: ProviderName;
    readonly firstContentMs?: number;
    readonly idleMs?: number;
  },
): IdleGuard {
  /**
   Controller aborted when silence outlasts the current window.
   */
  const controller = new AbortController();

  /**
   Stream bookkeeping, held in one named record so no binding at function
   root has to be reassignable.
   */
  const state = {
    armedAt: monotonicMs(),
    lastChunkAt: 0,
    firstByteMs: -1,
    firstContentMs: -1,
    maxGapMs: 0,
    chars: 0,
    content: 0,
    reasoning: 0,
  };

  /**
   Aborts the exchange, naming which silence window expired. The abort reason
   carries the error so the transport can rethrow it instead of the opaque
   abort the platform would otherwise surface.

   @param phase - which window expired

   @param expiredMs - that window's length
   */
  function trip(
    {
      phase,
      expiredMs,
    }: {
      readonly phase: 'first-content' | 'body';
      readonly expiredMs: number;
    },
  ): void {
    tagged({
      tag: armIdleGuard.name,
      l,
    },)
      .warn(
      `${(provider === undefined) ? '' : `${provider} `}${label}: no content or reasoning for ${
          String(expiredMs,)
        }ms during ${phase}; ${describeChars({
          count: state.chars,
          kind: 'raw',
        })} on the wire, ${
          describeChars({
            count: state.content,
            kind: 'content',
          })
        } and ${describeChars({
          count: state.reasoning,
          kind: 'reasoning',
        })} so far; aborting for retry`,
      );
    controller.abort(new StreamStalledError({
      label,
      idleMs: expiredMs,
      phase,
    },),);
  }

  /**
   Starts one silence window, returning its timer handle.

   @param phase - window being armed

   @param windowMs - silence allowed before tripping

   @returns Timer handle to clear when activity arrives
   */
  function armWindow(
    {
      phase,
      windowMs,
    }: {
      readonly phase: 'first-content' | 'body';
      readonly windowMs: number;
    },
  ): ReturnType<typeof setTimeout> {
    return setTimeout(
      function onSilence() {
        trip({
          phase,
          expiredMs: windowMs,
        },);
      },
      windowMs,
    );
  }

  /**
   Timer handle of the currently armed window, replaced on every piece of
   generated text.
   */
  const timers = {
    handle: armWindow({
      phase: 'first-content',
      windowMs: firstContentMs,
    },),
  };

  return {
    signal: controller.signal,
    notify(chars: number,): void {
      /**
       When this chunk landed, for gap bookkeeping.
       */
      const now = monotonicMs();
      if (state.firstByteMs < 0)
        state.firstByteMs = now - state.armedAt;
      else
        state.maxGapMs = Math.max(
          state.maxGapMs,
          now - state.lastChunkAt,
        );
      state.lastChunkAt = now;
      state.chars += chars;
    },
    notifyGenerated({
      content,
      reasoning,
    }: GeneratedChars,): void {
      state.content = content;
      state.reasoning = reasoning;
      // The first text is stamped on the same clock as the first byte, so the two readings subtract.
      if ((state.firstContentMs < 0) && ((content + reasoning) > 0))
        state.firstContentMs = monotonicMs() - state.armedAt;
      // A guard that has tripped stays tripped: a late piece of text must not arm a second window that logs a second cut.
      if (controller.signal
        .aborted)
        return;
      clearTimeout(timers.handle,);
      timers.handle = armWindow({
        phase: 'body',
        windowMs: idleMs,
      },);
    },
    progress(): StreamProgress {
      return {
        firstByteMs: state.firstByteMs,
        firstContentMs: state.firstContentMs,
        maxGapMs: state.maxGapMs,
        chars: state.chars,
        elapsedMs: monotonicMs() - state.armedAt,
      };
    },
    [Symbol.dispose](): void {
      clearTimeout(timers.handle,);
    },
  };
}

//endregion Stream idle guard
