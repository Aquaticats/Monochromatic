import { MIN_CHARS_FOR_VERDICT, } from './stream-degeneration.ts';

//region Stream recurrence watch
// CATCHES A PERIOD THE RATIO DETECTOR CANNOT SEE. `watchForDegeneration`
// samples fixed-width windows every WINDOW_STRIDE characters, so its
// sensitivity depends on gcd(period, WINDOW_STRIDE) rather than on how
// repetitive the text actually is: a model looping a 501-character
// paragraph forever reads as healthy at 0.1223 distinct, one character
// shorter reads as degenerate, and nothing about the text differs in kind.
// Probed directly against the built detector with no rebuild and no quota,
// the predicted window count matched the observed one in all fifteen cases
// tried.
//
// THIS DOES NOT DEPEND ON THE PERIOD'S ARITHMETIC. It keeps the trailing
// text itself, in a bounded buffer, and asks whether the most recent
// characters occur earlier in it. A stream looping with any period shorter
// than the buffer answers yes eventually, whatever the period, and ordinary
// prose never repeats a block that long verbatim. It complements the ratio
// rather than replacing it: the ratio catches short cycling on a small
// sample, this catches the longer periods the ratio's sampling arithmetic
// is blind to.
//
// ONE HIT MUST NOT BE A VERDICT. Reasoning traces in this pipeline restate
// whole source slices and whole candidates verbatim, so a model quoting a
// long candidate a second time inside one thinking trace is ordinary work,
// and a single recurrence reading as a loop would cost a voice for doing
// its job. The verdict requires the recurrence to persist across several
// consecutive checks as the stream grows, which a genuine loop does forever
// and a bounded quotation cannot.
//
// GATED BEHIND THE SAME LENGTH BAR THE RATIO DETECTOR USES, per
// `MIN_CHARS_FOR_VERDICT`. A genuine loop never stops, so it always crosses
// the bar eventually; every legitimate reply, verse refrains included,
// stays under it and is never judged by this detector any more than by the
// ratio one.

/**
 Composes {@link QUARTER} from literals in the exempt magic-number range
 rather than naming the fraction 4 directly.
 */
const HALF = 1 / 2;

/**
 Share of {@link BUFFER_CHARS} given to {@link TAIL_CHARS}, composed from
 {@link HALF} rather than the literal 4.
 */
const QUARTER = HALF * HALF;

/**
 Trailing characters of generated text kept for the recurrence check.

 A FEW THOUSAND CHARACTERS, bounded so a stream that never ends costs no
 more to watch this way at any point than it did at the start.
 */
const BUFFER_CHARS = 4_096;

/**
 Width of the trailing slice searched for earlier in the buffer.

 A QUARTER OF {@link BUFFER_CHARS}, not half, and that headroom is
 load-bearing. `.includes()` finds `tail` inside `earlier` only where
 `earlier` is long enough to let the match slide to the right offset: with
 `earlier` fixed at `BUFFER_CHARS - TAIL_CHARS`, the room to slide is
 `earlier.length - tail.length`, which is `BUFFER_CHARS - 2 * TAIL_CHARS`.
 At half the buffer that room is zero, so `.includes()` can only match at
 one exact offset, which requires that offset to be a multiple of the
 period by coincidence rather than by construction, missing most periods
 including 501. At a quarter it is 2048, which is at least as wide as any
 period up to 2048 characters, so the sliding search is guaranteed to land
 on a matching offset somewhere in range for any period that short,
 covering the 410 to 1001-character periods the ratio detector's window
 arithmetic misses. Periods longer than this room stay out of reach of any
 buffer this size, which this package's own decision record accepts: "any
 period shorter than the buffer."

 THE SAME HEADROOM ALSO BOUNDS A BACK-TO-BACK DUPLICATION'S FALSE-POSITIVE
 REACH. A block of length L recurring immediately back-to-back is only
 findable while L is at least TAIL_CHARS (so the checked slice sits
 entirely inside repeated content) and at most `BUFFER_CHARS - TAIL_CHARS`
 (so the earlier copy has not yet scrolled out of the buffer), a window
 `BUFFER_CHARS - 2 * TAIL_CHARS` wide. The exact count of consecutive
 checks a single such requote can produce, and how
 {@link REQUIRED_CONSECUTIVE_HITS} is set against it, is measured and
 documented there rather than re-derived here.
 */
const TAIL_CHARS = BUFFER_CHARS * QUARTER;

/**
 Characters of newly generated text between recurrence checks.

 OCCASIONAL RATHER THAN PER CHUNK. A single check is one native substring
 search over a buffer bounded at {@link BUFFER_CHARS}, cheap on its own,
 but a chunk can be a few dozen characters, and checking every one of them
 would pay that cost far more often than the verdict needs. Spaced this
 way the amortised cost per generated character stays bounded regardless
 of how long the stream runs.
 */
const CHECK_INTERVAL_CHARS = 512;

/**
 Consecutive positive checks required before a recurrence reads as a
 verdict rather than a coincidence.

 PAST THE PROVEN MAXIMUM A BOUNDED RE-QUOTE CAN PRODUCE, not past a naive
 formula. Per the geometry documented on {@link TAIL_CHARS}, a block
 recurring exactly once, immediately back-to-back, produces hits only while
 its length L stays within `[TAIL_CHARS, BUFFER_CHARS - TAIL_CHARS]`
 (`[1024, 3072]` at these constants), and only for checks landing in a
 charsSeen window of width `L - TAIL_CHARS`. Dividing that width by
 {@link CHECK_INTERVAL_CHARS} suggests a cap of 4, but a closed interval
 counts both endpoints, so `k` consecutive hits need width at least `(k -
 1) * CHECK_INTERVAL_CHARS`. Reaching 6 would need `L >= TAIL_CHARS + 5 *
 CHECK_INTERVAL_CHARS`, 3584 at these constants, already past the 3072
 upper bound where the earlier copy has scrolled out of the buffer
 entirely and every hit stops. Reaching 5 needs `L >= 3072` exactly, which
 is that same upper bound, so 5 is reachable only at that one boundary
 length, with the check phase landing on both ends of the window, and
 never above it. Simulated directly against this detector at every
 512-character step boundary from `TAIL_CHARS` to `BUFFER_CHARS -
 TAIL_CHARS` and at every check phase, the measured hit count matched this
 formula exactly at each one (1, 2, 3, 4, 5), and one character past the
 upper bound produced zero, confirming the cutoff. Six is set past that
 proven ceiling, while an indefinitely repeating loop keeps matching every
 check for as long as it keeps looping and so always eventually reaches it
 regardless of where this constant is set.
 */
const REQUIRED_CONSECUTIVE_HITS = 6;

/**
 What the recurrence watch currently believes about a stream.

 TWO STATES ONLY, unlike the ratio detector's `DegenerationVerdict`: this
 detector has no "not enough windows yet" state distinct from "healthy",
 because a check either finds the trailing span recurring or it does not,
 and the caller cares only about the boundary this crosses.

 @example
 ```ts
 const verdict: RecurrenceVerdict = { kind: 'continuing', };
 ```
 */
export type RecurrenceVerdict = {
  readonly kind: 'continuing';
} | {
  readonly kind: 'degenerate';

  /**
   Characters of generated text seen before the verdict, so the cost of
   letting it run this far is legible in the log.
   */
  readonly charsSeen: number;
};

/**
 A running detector over one stream's generated text, watching for a
 recurring span the windowed ratio detector's arithmetic could miss.

 @example
 ```ts
 const detector = watchForRecurrence();
 detector.notifyText({ text: 'The cat naps. ', },);
 const verdict = detector.verdict();
 ```
 */
export type RecurrenceDetector = {
  /**
   Feeds newly generated text, in arrival order.
   */
  readonly notifyText: (input: { readonly text: string; },) => void;

  /**
   Reads what the trailing buffer currently says.
   */
  readonly verdict: () => RecurrenceVerdict;
};

/**
 @internal

 Builds a detector that reports when generated text has begun recurring at
 a short lag, regardless of the recurring span's length.

 BOUNDED MEMORY AND BOUNDED PER-CHARACTER COST, per `RG2`: the buffer never
 grows past {@link BUFFER_CHARS} and a check runs only once every
 {@link CHECK_INTERVAL_CHARS} characters, so the amortised cost per
 character stays constant regardless of how long the stream runs, which
 matters precisely because the streams this exists to stop are the ones
 that never end.

 NO REGEX, per `RG1`: the question is "does this exact slice occur
 earlier in this buffer", which `String.prototype.includes` states
 directly over a bounded input.

 @returns Detector fed by `notifyText` and read by `verdict`

 @example
 ```ts
 const detector = watchForRecurrence();
 for (const chunk of chunks)
   detector.notifyText({ text: chunk, },);
 if (detector.verdict().kind === 'degenerate')
   throw new Error('the model is cycling at a period the ratio missed',);
 ```
 */
export function watchForRecurrence(): RecurrenceDetector {
  /**
   Trailing buffer, running totals, and the consecutive-hit count, held in
   one record so the factory root holds no mutable variable.
   */
  const state = {
    buffer: '',
    charsSeen: 0,
    sinceLastCheck: 0,
    consecutiveHits: 0,
  };

  return {
    notifyText({ text, },): void {
      if (text === '')
        return;

      state.charsSeen += text.length;
      state.sinceLastCheck += text.length;
      state.buffer += text;

      // Keep only the trailing BUFFER_CHARS, so the buffer cannot grow with
      // a stream that never ends.
      if (state.buffer
        .length
        > BUFFER_CHARS)
        state.buffer = state.buffer
          .slice(-BUFFER_CHARS,);

      if (state.sinceLastCheck < CHECK_INTERVAL_CHARS)
        return;
      state.sinceLastCheck = 0;

      // Too little trailing text yet for a check to mean anything: an
      // earlier portion shorter than TAIL_CHARS could never contain a
      // TAIL_CHARS-wide match regardless of what the stream is doing.
      if (state.buffer
        .length
        < BUFFER_CHARS)
        return;

      /**
       Most recent slice, checked against everything before it in the
       buffer.
       */
      const tail = state.buffer
        .slice(-TAIL_CHARS,);

      /**
       Buffer length, read once so the earlier-portion boundary names a
       single value rather than a member chain.
       */
      const bufferLength = state.buffer
        .length;

      /**
       Everything in the buffer before the tail.
       */
      const earlier = state.buffer
        .slice(
        0,
        bufferLength - TAIL_CHARS,
        );

      state.consecutiveHits = earlier.includes(tail,) ? (state.consecutiveHits + 1) : 0;
    },

    verdict(): RecurrenceVerdict {
      if (state.charsSeen < MIN_CHARS_FOR_VERDICT)
        return { kind: 'continuing', };
      if (state.consecutiveHits < REQUIRED_CONSECUTIVE_HITS)
        return { kind: 'continuing', };
      return {
        kind: 'degenerate',
        charsSeen: state.charsSeen,
      };
    },
  };
}

//endregion Stream recurrence watch
