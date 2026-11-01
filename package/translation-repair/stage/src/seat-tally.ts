import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ChatTextReply,
  ChatTextRequest,
  DecisionReply,
  DecisionRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider/ts';
import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup/ts';
import { deliveredNoContent, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import type {
  ReadonlyEntry,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { StragglerCutError, } from './straggler-cut.ts';

//region Seat tally
// WHAT EVERY SEAT WAS ASKED AND WHAT IT GAVE BACK, counted at the client seam
// and said out loud once, when the command ends.
//
// A four-slice calibration once ran with five of ten seats failing every
// call they made, 25 of 25 each, and the command exited 0. Quorum is five of
// ten and was met on the nose by the other five. The calibration's closing
// coverage sentence did name the five seats, but as "wrote nothing at all",
// which is also what a seat that was outvoted every round looks like from
// there, and the pass and the other thirty-six commands print nothing of the
// kind. A per-call `warn` and a per-round `5/10 heard` were both correct and
// both invisible to a reader who was not grepping.
//
// THIS IS THE ONE PLACE EVERY CALL PASSES. The client every corpus-run command
// uses is built by one factory, so wrapping it here reaches all of them at
// once, and the report is printed by the refusal boundary they all share. IT
// WRAPS THE OUTERMOST CLIENT, the one callers hold (ledger P6): a wrapper that
// buys JSON through `chatText` and reads the reply itself hides every reading
// from a tally beneath it, which then counts each reply that arrived as
// usable. A
// seat is DARK when it was asked at least once and never once produced a usable
// answer, which is the signature of a provider that errors on it or a model
// that answers nothing readable; a seat that simply lost some rounds is not dark
// and is not reported here, and neither is a seat no provider could take at all
// (no key, or no wet provider), which the router refuses before any request is
// sent and the tally counts `unreachable`, not asked.
//
// A REFUSAL THE ROUTER GIVES BEFORE ANY REQUEST IS SENT IS NOT AN ASK
// (ledger B711). The provider router throws `NoProviderForModelError` for a seat
// no wet provider serves, and the zheermao20 run counted four such refusals of
// `glm-5.3` as "asked=4 threw=4" and withheld the seat for a failure it never
// had. They are counted in a column of their own, `unreachable`, which is in
// neither `asked` nor `threw`, so a seat the router only refuses is never dark
// and never withheld here: the router refuses it cheaply each time, the stages
// read the refusal as an unreachable seat, and the seat comes back by itself
// when a provider is wet again. The `JUDGE SEATS` lines name such seats as
// unserved while they sit on a bench.
//
// PROCESS-SCOPED, LIKE `process.exitCode`. A calibration builds one client per
// slice, so a tally held by a client would end with the slice; the fact being
// counted is about the whole command, and the singleton is what a whole-command
// fact looks like. Tests that need isolation pass their own tally.

/**
 What one call to a seat came back as.

 `usable` is text returned or a JSON outcome of kind `ok`; `unusable` is a
 JSON outcome the guard rejected (refusal-shaped or off-schema); `threw` is a
 call that never returned an outcome at all and was not abandoned by its
 caller; `silent` is a call the round's own grace window cut, after quorum
 stood, that had delivered no answer characters and no reasoning characters:
 an ask the seat failed, counted in `asked` (ledger B761); `abandoned` is any
 other call whose caller aborted its signal before it
 settled, which is counted beside the others and in none of them: it is not an
 ask the seat answered or failed (ledger B421), a slow reasoner cut with
 reasoning on its way among them; `unreachable` is a call the
 provider router refused with `NoProviderForModelError` before any request was
 sent, which is counted beside the others and in none of them for the same
 reason: no provider was asked, so the seat neither answered nor failed
 (ledger B711).

 @example
 ```ts
 const outcome: SeatOutcome = 'threw';
 ```
 */
export type SeatOutcome = 'usable' | 'unusable' | 'threw' | 'silent' | 'abandoned' | 'unreachable';

/**
 Counts for one seat over the life of the tally.

 @example
 ```ts
 const dark = counts.filter(function isDark(count,) { return count.usable === 0; },);
 ```
 */
export type SeatCount = {
  /**
   Seat these counts describe.
   */
  readonly modelId: RosterModelId;

  /**
   Calls that settled, whatever came back, and the calls the round's grace
   window cut with nothing delivered (`silent`); a call the caller abandoned
   or the round cut with content on its way is not one, and neither is a call
   the router refused, so none of those can raise a seat toward its
   withholding or lower it.
   */
  readonly asked: number;

  /**
   Calls that returned text or an `ok` outcome.
   */
  readonly usable: number;

  /**
   Calls that returned an outcome the guard rejected.
   */
  readonly unusable: number;

  /**
   Calls that threw instead of returning.
   */
  readonly threw: number;

  /**
   Calls the round's grace window cut after quorum stood, that had delivered
   no answer characters and no reasoning characters: a dead stream, which the
   seat failed to answer inside the window. In `asked`, and a failed ask for
   the withholding rule beside `threw`.
   */
  readonly silent: number;

  /**
   Calls the caller abandoned before they settled: a straggler a round cut
   off after quorum with answer or reasoning characters on its way, a stage
   deadline, an entry stop. Not in `asked`.
   */
  readonly abandoned: number;

  /**
   Calls the provider router refused with `NoProviderForModelError` before any
   request was sent, because no wet provider serves the seat. Not in `asked`.
   */
  readonly unreachable: number;
};

/**
 Running counts per seat, with the two readings the report needs.

 @example
 ```ts
 const tally = createSeatTally();
 tally.record({ modelId: 'minimax-m3', outcome: 'threw', },);
 ```
 */
export type SeatTally = {
  /**
   Counts one settled call.
   */
  readonly record: (args: {
    readonly modelId: RosterModelId;
    readonly outcome: SeatOutcome;
  },) => void;

  /**
   Every seat a call was counted against, in first-counted order: seats asked,
   and seats whose calls were all abandoned or refused by the router.
   */
  readonly counts: () => readonly SeatCount[];

  /**
   Seats asked at least once that never produced a usable answer. A seat only
   the router refused was never asked, so it is not here; its `unreachable`
   count and the `JUDGE SEATS` lines name it.
   */
  readonly dark: () => readonly SeatCount[];

  /**
   Seats asked at least `minAsks` times whose every ask failed: threw, or was
   cut by the round's grace window with nothing delivered, so no call they
   made ever returned an outcome. Calls the router refused and calls cut with
   content on their way are not asks, so they neither count toward `minAsks`
   nor break a run of failures.

   @param args - the number of asks a seat must have been given before it counts

   @returns Counts of the withheld seats, in first-asked order
   */
  readonly withheld: (args: {
    readonly minAsks: number;
  },) => readonly SeatCount[];

  /**
   Forgets every count, for a new command in the same process.
   */
  readonly reset: () => void;
};

/**
 Mutable counts behind one seat.
 */
type SeatCounter = {
  asked: number;
  usable: number;
  unusable: number;
  threw: number;
  silent: number;
  abandoned: number;
  unreachable: number;
};

/**
 Builds an empty tally.

 @returns Tally counting from zero

 @example
 ```ts
 const tally = createSeatTally();
 ```
 */
export function createSeatTally(): SeatTally {
  /**
   Counter per seat, keyed by model id, in first-asked order.
   */
  const counters = new Map<RosterModelId, SeatCounter>();

  /**
   Returns the seat's counter, creating it on first use.

   @param modelId - seat being counted

   @returns Counter to increment

   @example
   ```ts
   counterFor('minimax-m3',).asked += 1;
   ```
   */
  function counterFor(modelId: RosterModelId,): SeatCounter {
    /**
     Existing counter when this seat was asked before.
     */
    const existing = counters.get(modelId,);
    if (existing !== undefined)
      return existing;

    /**
     Fresh counter for a seat's first call.
     */
    const created: SeatCounter = {
      asked: 0,
      usable: 0,
      unusable: 0,
      threw: 0,
      silent: 0,
      abandoned: 0,
      unreachable: 0,
    };
    counters.set(
      modelId,
      created,
    );
    return created;
  }

  /**
   Snapshot of every counter as immutable counts.

   @returns Counts in first-asked order

   @example
   ```ts
   const all = snapshot();
   ```
   */
  function snapshot(): readonly SeatCount[] {
    return [...counters.entries(),].map(function toCount([modelId, counter,]: ReadonlyEntry<RosterModelId, Readonly<SeatCounter>>,): SeatCount {
      return {
        modelId,
        asked: counter.asked,
        usable: counter.usable,
        unusable: counter.unusable,
        threw: counter.threw,
        silent: counter.silent,
        abandoned: counter.abandoned,
        unreachable: counter.unreachable,
      };
    },);
  }

  return {
    record({
      modelId,
      outcome,
    },): void {
      /**
       Counter for the seat this call went to.
       */
      const counter = counterFor(modelId,);
      if (outcome === 'abandoned') {
        counter.abandoned += 1;
        return;
      }
      if (outcome === 'unreachable') {
        counter.unreachable += 1;
        return;
      }
      counter.asked += 1;
      if (outcome === 'silent')
        counter.silent += 1;
      else if (outcome === 'usable')
        counter.usable += 1;
      else if (outcome === 'unusable')
        counter.unusable += 1;
      else
        counter.threw += 1;
    },
    counts: snapshot,
    dark(): readonly SeatCount[] {
      return snapshot()
        .filter(function producedNothingUsable(count,): boolean {
          return (count.asked > 0) && (count.usable === 0);
        },);
    },
    withheld({ minAsks, },): readonly SeatCount[] {
      return snapshot()
        .filter(function failedEveryAsk(count,): boolean {
          return (count.asked >= minAsks) && ((count.threw + count.silent) === count.asked);
        },);
    },
    reset(): void {
      counters.clear();
    },
  };
}

/**
 Tally shared by every client the corpus-run factory builds in this process.

 @example
 ```ts
 const darkSeats = RUN_SEATS.dark();
 ```
 */
export const RUN_SEATS: SeatTally = createSeatTally();

/**
 Wraps a client so every call it makes is counted against its seat.

 COUNTS AFTER THE CALL SETTLES, never before, so a call still in flight when
 the process ends is not counted as anything. The wrapped client is otherwise
 untouched: the same reply, the same outcome, the same throw.

 A SEAT THE TALLY WITHHOLDS IS REFUSED BEFORE THE INNER CLIENT IS ASKED, with
 a `NoProviderForModelError` that a stage reads as an unreachable seat. Such a
 refusal is not an ask, so it leaves the counts as they were.

 A `NoProviderForModelError` THE INNER CLIENT THROWS IS COUNTED AS UNREACHABLE,
 AND AS NOTHING ELSE (ledger B711). The router gives it for a seat no wet
 provider serves and the owner's cull gives it for a culled seat, both before
 any request is sent, so it is no more an ask than the tally's own refusal is.
 It is counted whether or not the caller had aborted, since the refusal is a
 fact about the seat and no call was left behind. A seat only refused this way
 is never dark and never withheld by the tally; it is refused again, cheaply,
 at every call.

 A CALL THE CALLER ABORTED IS COUNTED AS ABANDONED WHEN IT THROWS, AND AS
 NOTHING ELSE. The abort is the caller leaving the call, not the seat failing
 to answer, so a seat its callers cut off after quorum is not counted as asked
 and cannot be withheld for it, but its census line says how many replies were
 cut: the TianqiChen66626 pass cut 58 Qwen3.8-27B replies and read
 `asked=299 usable=299`, which hid them. A call still in flight when the
 process ends is not counted at all.

 A PROVIDER'S REJECTED CREDENTIAL (HTTP 401) IS NOT SAID OR ACTED ON HERE. The
 provider router beneath holds that provider out for the run and goes on to the
 next provider serving the seat (`provider-router.ts`), so a seat is thrown for
 only when no provider that serves it is left.

 @param inner - client whose calls are counted

 @param tally - where the counts go

 @param minAsks - asks a seat must have been given, all of them thrown, before
 its calls are refused

 @returns Client with the same surface, counting into `tally`

 @throws {@link NoProviderForModelError} when the seat is withheld

 @example
 ```ts
 const counted = seatTallyClient({ inner: client, tally: RUN_SEATS, minAsks: 4, },);
 ```
 */
export function seatTallyClient(
  {
    inner,
    tally,
    minAsks,
  }: {
    readonly inner: ModelClient;
    readonly tally: SeatTally;
    readonly minAsks: number;
  },
): ModelClient {
  /**
   Typed exchange the wrapped client offers, bound once so the closure
   this function returns keeps the narrowing.
   */
  const innerDecide = inner.decide;

  /**
   Throws when the tally has withheld the seat, before any provider is asked.

   @param modelId - seat a call names

   @throws {@link NoProviderForModelError} when the seat is withheld

   @example
   ```ts
   refuseWithheld({ modelId: request.modelId, },);
   ```
   */
  function refuseWithheld({ modelId, }: { readonly modelId: RosterModelId; },): void {
    /**
     Counts of every seat the tally has withheld.
     */
    const withheldCounts = tally.withheld({ minAsks, },);
    /**
     The seat's counts when the tally has withheld it, or none.
     */
    const withheldCount = withheldCounts.find(function isThisSeat(count,): boolean {
      return count.modelId === modelId;
    },);
    if (withheldCount === undefined)
      return;
    /**
     The seat's asks, counted and named by the word for the count.
     */
    const asks = `${String(withheldCount.asked,)} ${
      wordForCount({
        count: withheldCount.asked,
        one: 'ask',
        many: 'asks',
      },)
    }`;
    // THE WORDS FOR A SEAT THAT ONLY THREW STAY AS THEY WERE (ledger B711); a seat with asks the round cut with nothing
    // delivered says both ways its asks failed.
    throw new NoProviderForModelError({
      modelId,
      reason: (withheldCount.silent === 0)
        ? `the run withheld it after it threw on all ${asks} and does not ask it again`
        : `the run withheld it after all ${asks} failed (${String(withheldCount.threw,)} threw, ${
          String(withheldCount.silent,)
        } ended with nothing delivered) and does not ask it again`,
    },);
  }

  /**
   Counts a call that threw: as unreachable when the inner client refused the
   seat with `NoProviderForModelError`, since no request was sent and so no
   ask was made; otherwise, when the signal had aborted, as silent where the
   abort was the round's grace window closing on a stream that had delivered
   no answer and no reasoning (a failed ask of the seat, ledger B761), and as
   abandoned in every other case, since an abort is the caller leaving the call
   (a straggler cut off after quorum with content on its way, a stage
   deadline, an entry stop) and not the seat failing to answer, so it is not an
   ask either; otherwise as threw.

   @param modelId - seat the call went to

   @param signal - signal the call was made with

   @param error - what the call threw, whose class says whether the router refused the seat

   @example
   ```ts
   recordThrow({ modelId: request.modelId, signal: request.signal, error, },);
   ```
   */
  function recordThrow({
    modelId,
    signal,
    error,
  }: {
    readonly modelId: RosterModelId;
    readonly signal: AbortSignal;
    readonly error: unknown;
  },): void {
    if (error instanceof NoProviderForModelError) {
      tally.record({
        modelId,
        outcome: 'unreachable',
      },);
      return;
    }
    if (signal.aborted) {
      // ONLY THE ROUND'S OWN GRACE CUT, WITH NOTHING DELIVERED, IS THE SEAT'S FAILURE. A caller's stop, an entry's hard
      // cap or an operator's abort lands on every call in flight, empty or not, and says nothing of any seat; a stream
      // cut with reasoning on its way is a model still working (Qwen, 43,372 reasoning characters and no answer yet).
      tally.record({
        modelId,
        outcome: ((signal.reason instanceof StragglerCutError) && deliveredNoContent({ error, },))
          ? 'silent'
          : 'abandoned',
      },);
      return;
    }
    tally.record({
      modelId,
      outcome: 'threw',
    },);
  }

  return {
    async chatText(request: ForeignBorrowed<ChatTextRequest>,): Promise<ChatTextReply> {
      refuseWithheld({ modelId: request.modelId, },);
      try {
        /**
         Reply from the wrapped client.
         */
        const reply = await inner.chatText(request,);
        tally.record({
          modelId: request.modelId,
          outcome: 'usable',
        },);
        return reply;
      }
      catch (error) {
        recordThrow({
          modelId: request.modelId,
          signal: request.signal,
          error,
        },);
        throw error;
      }
    },
    async chatJson<ValueT,>(
      request: ForeignBorrowed<ChatJsonRequest<ValueT>>,
    ): Promise<ChatJsonOutcome<ValueT>> {
      refuseWithheld({ modelId: request.modelId, },);
      try {
        /**
         Outcome from the wrapped client, usable only when `ok`.
         */
        const outcome = await inner.chatJson(request,);
        tally.record({
          modelId: request.modelId,
          outcome: (outcome.kind === 'ok') ? 'usable' : 'unusable',
        },);
        return outcome;
      }
      catch (error) {
        recordThrow({
          modelId: request.modelId,
          signal: request.signal,
          error,
        },);
        throw error;
      }
    },
    quotas: inner.quotas,
    // THE TYPED EXCHANGE IS A SEAT ASKED TOO, tallied under the same
    // outcomes, so the SEAT lines count a decision seat beside the rest and a
    // decision seat that never answers usably is named dark like any other.
    ...((innerDecide === undefined)
      ? {}
      : {
        decide: async function talliedDecide(request: ForeignBorrowed<DecisionRequest>,): Promise<DecisionReply> {
          refuseWithheld({ modelId: request.modelId, },);
          try {
            /**
             Reply from the wrapped client.
             */
            const reply = await innerDecide(request,);
            tally.record({
              modelId: request.modelId,
              outcome: 'usable',
            },);
            return reply;
          }
          catch (error) {
            recordThrow({
              modelId: request.modelId,
              signal: request.signal,
              error,
            },);
            throw error;
          }
        },
      }),
  };
}

/**
 Renders the tally as the lines a command prints when it ends.

 ONE LINE PER SEAT, then one line naming the dark seats when there are any.
 Nothing at all when no call was counted, so a command that never built a
 client prints nothing extra. A seat the router only refused gets its line
 (`asked=0`, `unreachable=N`) and is not named dark, since it was never asked.
 Model ids and numbers only.

 @param tally - counts to render

 @returns Lines in print order, empty when nothing was asked

 @example
 ```ts
 for (const line of seatReportLines({ tally: RUN_SEATS, },)) console.error(line,);
 ```
 */
export function seatReportLines(
  { tally, }: { readonly tally: SeatTally; },
): readonly string[] {
  /**
   Every seat a call was counted against this run.
   */
  const counts = tally.counts();
  if (counts.length === 0)
    return [];

  /**
   Seats that never produced a usable answer.
   */
  const dark = tally.dark();

  /**
   One line per seat, whatever it produced.
   */
  const perSeat = counts.map(function toLine(count,): string {
    return `SEAT ${count.modelId} asked=${String(count.asked,)} usable=${String(count.usable,)} `
      + `unusable=${String(count.unusable,)} threw=${String(count.threw,)} silent=${String(count.silent,)} `
      + `abandoned=${String(count.abandoned,)} `
      + `unreachable=${String(count.unreachable,)}`;
  },);

  if (dark.length === 0)
    return perSeat;

  /**
   Dark seats with the counts that make them dark.
   */
  const named = dark
    .map(function describe(count,): string {
      /**
       The silent count, named only where there is one so the line reads as it did for every seat that has none.
       */
      const silent = (count.silent === 0) ? '' : `, silent ${String(count.silent,)}`;
      return `${count.modelId} (asked ${String(count.asked,)}, unusable ${
        String(count.unusable,)
      }, threw ${String(count.threw,)}${silent})`;
    },)
    .join('; ',);

  /**
   Seats that were asked at least once, the denominator the dark line states;
   a seat only abandoned or refused by the router was never asked.
   */
  const askedSeats = counts.filter(function wasAsked(count,): boolean {
    return count.asked > 0;
  },);
  /**
   How many seats were asked.
   */
  const askedCount = askedSeats.length;

  /**
   The one line a reader who is not grepping must see.
   */
  const darkLine = `SEATS DARK: ${String(dark.length,)} of ${String(askedCount,)} ${
    wordForCount({
      count: askedCount,
      one: 'seat',
      many: 'seats',
    },)
  } asked `
    + `produced nothing usable this run: ${named}. A seat that fails every call is a provider that `
    + 'errors on it or a model that answers nothing readable; the run log names which, and writes each '
    + 'failed call\'s error text on a warn line "<stage> <model>: <error>, voice lost". A seat the router '
    + 'refused before any request was sent is on its SEAT line as unreachable=N and is not counted here. '
    + 'Do not read this run as a comparison of the roster.';

  return [
    ...perSeat,
    darkLine,
  ];
}

//endregion Seat tally
