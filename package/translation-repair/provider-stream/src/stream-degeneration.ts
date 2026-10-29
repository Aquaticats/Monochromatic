import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

//region Stream degeneration
// WHETHER A MODEL HAS STOPPED SAYING ANYTHING NEW, which is a different failure
// from the one `stream-idle-guard.ts` watches for and is invisible to it.
//
// The idle guard asks whether bytes are arriving. A degenerating model answers
// yes forever: it emits continuously, at full speed, repeating itself. Measured
// over 1677 completed streams in the 2026-08-17 pass, the largest streams ran
// with gaps of 202 to 858 ms and only 61 streams paused longer than 5 s, so no
// silence window can separate a degenerating stream from a working one. Both
// idle constants are set to 600000 precisely because silence does not
// discriminate here.
//
// THE PROVIDER DOES NOT STOP THESE CALLS, and no production stage sends a token
// cap, so nothing upstream ends them either. The only remaining place to end
// them is here.
//
// WHAT IS MEASURED is the share of recent windows of text that are distinct. A
// model producing prose emits almost entirely fresh windows; one cycling on a
// phrase emits the same few windows forever. The measure is taken over a
// TRAILING sample rather than the whole stream, so a reply that degenerates
// after a healthy opening is still caught rather than diluted by its own good
// beginning.
//
// THIS MODULE KNOWS NOTHING ABOUT THE WIRE FORMAT. It is fed generated text.
// Feeding it raw server-sent events would report every healthy stream as
// degenerate, because the JSON envelope around each token is identical by
// construction and would dominate the sample.

/**
 Width of one sampled window, in characters.

 Wide enough that ordinary prose almost never repeats a whole window by
 chance, and narrow enough that a short repeated phrase still fills several.
 */
const WINDOW_CHARS = 64;

/**
 Distance between the starts of consecutive sampled windows.

 Half a window, so every position is covered by a sample without paying for a
 sample at every character.
 */
const WINDOW_STRIDE = 32;

/**
 Windows kept in the trailing sample, which at `WINDOW_STRIDE` is about
 131000 characters of recent text.

 TRAILING RATHER THAN CUMULATIVE so late-onset degeneration is caught. A
 cumulative ratio over a reply that ran healthy for a long time cannot fall
 far enough to trip, no matter how long it then cycles.
 */
const TRAILING_WINDOWS = 4_096;

/**
 Windows required before any verdict is offered, which must not exceed
 {@link TRAILING_WINDOWS}: the sample is capped at that size, so a larger
 minimum would make every verdict unreachable and this guard silently inert.
 Equal to it here, so a verdict is offered exactly when the sample is full.

 SET SO THAT LENGTH ALONE NEVER CONDEMNS, and so that being verbose is not
 treated as being broken. Some models legitimately write a great deal. The
 ratio is what decides; this constant only decides when there is enough text
 for the ratio to mean anything, and it is deliberately far above any real
 reply. Across all 56 settled artifacts the longest recorded model output is
 8358 characters, so a bar at about 131000 sits roughly fifteen times above
 anything this pipeline has ever legitimately produced.

 IT IS ALSO WHAT KEEPS VERSE SAFE. A translated poem carrying a refrain is
 genuinely repetitive, and measured at 116800 characters one scored 0.036,
 which the ratio alone would condemn. No slice translation approaches this
 bar, so such a reply is never judged at all.

 DUE A REVISIT, since aborted streams now keep what they delivered. The bar
 is measured in characters of generated text, while the only length
 telemetry in production counts raw server-sent event bytes, envelope
 included. The two are related by a per-token envelope
 cost that has been estimated and never measured, so this bar is set from the
 artifact evidence rather than from that column.
 */
const MIN_WINDOWS_FOR_VERDICT = 4_096;

/**
 `MIN_WINDOWS_FOR_VERDICT` in characters rather than windows, exported so a second
 detector fed the same generated text can be gated on the identical
 artifact-evidence bar rather than defining an independent one that could
 drift from it.

 DELIBERATELY THE SAME BAR, not merely a similar one. The reasoning that
 keeps verse and ordinary replies unjudged below {@link MIN_WINDOWS_FOR_VERDICT}
 windows applies unchanged to any other measure of the same generated text:
 no slice translation this pipeline produces approaches roughly 131000
 characters, so nothing legitimate is ever long enough to reach either
 detector's verdict.
 */
export const MIN_CHARS_FOR_VERDICT: number = MIN_WINDOWS_FOR_VERDICT * WINDOW_STRIDE;

/**
 Share of distinct windows at or below which the sample is called degenerate.

 SET WITH A WIDE MARGIN, deliberately. Scanned across all 56 settled
 artifacts, the most repetitive real string scored 0.998 distinct, and a
 synthetic control repeating one phrase scored 0.010. Nothing observed lies
 between 0.1 and 0.99, so this threshold is placed in empty space rather than
 fitted to a boundary.
 */
const DEGENERATE_RATIO = 0.1;

/**
 What the detector currently believes about a stream.

 A TAGGED UNION rather than a boolean plus numbers, because "not enough text
 to say" and "enough text, and it looks fine" are different answers and a
 caller that conflates them would abort short replies or trust empty ones.

 @example
 ```ts
 const verdict: DegenerationVerdict = { kind: 'undecided', windows: 12, };
 ```
 */
export type DegenerationVerdict = {
  readonly kind: 'undecided';

  /**
   Windows sampled so far, so a caller can say how far off a verdict is.
   */
  readonly windows: number;
} | {
  readonly kind: 'healthy';

  /**
   Share of the trailing sample that was distinct.
   */
  readonly distinctRatio: number;

  /**
   Windows the ratio was taken over.
   */
  readonly windows: number;
} | {
  readonly kind: 'degenerate';

  /**
   Share of the trailing sample that was distinct, at or below
   `DEGENERATE_RATIO`.
   */
  readonly distinctRatio: number;

  /**
   Windows the ratio was taken over.
   */
  readonly windows: number;

  /**
   Characters of generated text seen before the verdict, so the cost of
   letting it run this far is legible in the log.
   */
  readonly charsSeen: number;
};

/**
 A running detector over one stream's generated text.

 @example
 ```ts
 const detector = watchForDegeneration();
 detector.notifyText({ text: 'The cat naps. ', },);
 const verdict = detector.verdict();
 ```
 */
export type DegenerationDetector = {
  /**
   Feeds newly generated text, in arrival order.
   */
  readonly notifyText: (input: { readonly text: string; },) => void;

  /**
   Reads what the trailing sample currently says.
   */
  readonly verdict: () => DegenerationVerdict;

  /**
   Reads the running total of generated characters fed so far, unconditional
   on any verdict. `DegenerationVerdict` only carries this count on its
   `degenerate` case, which is silent on a stream that never trips it, and a
   progress line needs a figure for every stream rather than only for the
   ones this guard ends.
   */
  readonly charsSeen: () => number;
};

/**
 Builds a detector that reports when a stream has stopped producing new text.

 ONE LINEAR PASS AND BOUNDED MEMORY, per `RG2`: every character is examined a
 fixed number of times, and the sample never grows past `TRAILING_WINDOWS`
 entries regardless of how long the stream runs, which matters precisely
 because the streams this exists to stop are the ones that never end.

 NO REGEX, per `RG1`: the rule is "take a fixed-width slice every fixed number
 of characters", which slicing states directly.

 @returns Detector fed by `notifyText` and read by `verdict`

 @example
 ```ts
 const detector = watchForDegeneration();
 for (const chunk of chunks)
   detector.notifyText({ text: chunk, },);
 if (detector.verdict().kind === 'degenerate')
   throw new Error('the model is cycling',);
 ```
 */
export function watchForDegeneration(): DegenerationDetector {
  // An unreachable verdict would leave this guard inert while still looking
  // installed, which is the quietest way it could fail. The sample is capped at
  // TRAILING_WINDOWS, so a larger minimum can never be met.
  if (MIN_WINDOWS_FOR_VERDICT > TRAILING_WINDOWS)
    throw new Error(
      `degeneration guard is inert: a verdict needs ${String(MIN_WINDOWS_FOR_VERDICT,)} windows `
        + `but the sample holds at most ${String(TRAILING_WINDOWS,)}`,
    );

  /**
   Text not yet consumed into a window, plus the running totals.

   A RECORD RATHER THAN LOOSE BINDINGS so the factory root holds no mutable
   variable, and so every piece of the detector's state is named in one place.
   */
  const state = {
    pending: '',
    charsSeen: 0,
  };

  /**
   Windows in the trailing sample, oldest first, so the oldest can be evicted
   when the sample is full.
   */
  const order: string[] = [];

  /**
   How many times each window appears in `order`.

   Counted rather than merely present, because eviction must know when the
   last copy of a window has left the sample.
   */
  const counts = new Map<string, number>();

  /**
   Drops the oldest window from the sample, keeping `counts` in step.

   CALLED ONLY FROM `admit` UNDER `order.length > TRAILING_WINDOWS`, so
   `order` holds at least one entry here and `order.shift()` always returns
   one: `admit` pushes before calling this, so the array it reads from is
   never empty. The window it returns is therefore always one `admit` just
   counted into `counts`, so `counts.get(gone)` is its positive remaining
   count, never undefined.

   @example
   ```ts
   evictOldest();
   ```
   */
  function evictOldest(): void {
    /**
     Window leaving the sample, which `order` always has one of here.
     */
    const gone = nonNullishOrThrow(order.shift(),);

    /**
     Copies of it that remain after this one leaves, read from the count
     `admit` set when it pushed this very occurrence.
     */
    const left = nonNullishOrThrow(counts.get(gone,),) - 1;
    if (left <= 0)
      counts.delete(gone,);
    else
      counts.set(
        gone,
        left,
      );
  }

  /**
   Adds one sampled window to the trailing sample.

   @param window - fixed-width slice of generated text

   @example
   ```ts
   admit({ window: 'the cat naps on the mat', },);
   ```
   */
  function admit({ window, }: { readonly window: string; },): void {
    order.push(window,);
    counts.set(
      window,
      (counts.get(window,) ?? 0) + 1,
    );
    if (order.length > TRAILING_WINDOWS)
      evictOldest();
  }

  return {
    notifyText({ text, },): void {
      if (text === '')
        return;
      state.charsSeen += text.length;

      /**
       Everything not yet cut into windows, this arrival included.
       */
      const buffer = state.pending + text;

      /**
       Cursor over the buffer, advanced one stride per window taken.
       */
      const cut = { at: 0, };
      while ((cut.at + WINDOW_CHARS) <= buffer.length) {
        admit({
          window: buffer.slice(
            cut.at,
            cut.at + WINDOW_CHARS,
          ),
        },);
        cut.at += WINDOW_STRIDE;
      }

      // Keep only what a later window might still need, so the buffer cannot
      // grow with the stream.
      state.pending = buffer.slice(cut.at,);
    },

    verdict(): DegenerationVerdict {
      /**
       Windows currently in the trailing sample.
       */
      const windows = order.length;
      if (windows < MIN_WINDOWS_FOR_VERDICT)
        return {
          kind: 'undecided',
          windows,
        };

      /**
       Share of the sample that is distinct.
       */
      const distinctRatio = counts.size / windows;
      if (distinctRatio > DEGENERATE_RATIO)
        return {
          kind: 'healthy',
          distinctRatio,
          windows,
        };

      return {
        kind: 'degenerate',
        distinctRatio,
        windows,
        charsSeen: state.charsSeen,
      };
    },

    charsSeen(): number {
      return state.charsSeen;
    },
  };
}

//endregion Stream degeneration
