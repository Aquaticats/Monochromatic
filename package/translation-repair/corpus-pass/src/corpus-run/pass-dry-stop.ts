import {
  type Logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';
import { wait, } from '@monochromatic-dev/module-async-time/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import {
  durationWordsOf,
  EveryProviderDryError,
  type ModelClient,
  returnFitsWithin,
} from '@monochromatic-dev/translation-repair-provider/ts';
import { monotonicMs, } from '@monochromatic-dev/translation-repair-markup/ts';
import { refusalText, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { WritingBenchUnreachableError, } from '@monochromatic-dev/translation-repair-run/ts';
import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Pass dry stop
// THE RUN-LEVEL STOP FOR EVERY PROVIDER DRY. The owner ruled that the router's `EveryProviderDryError` "ENDS THE RUN"
// and that a run always ships what it has. XingZ6018 (2026-10-22) instead lost that error as one seat's voice in
// every round for 77 s, until the bench floor stopped it mid-entry with `status=ERROR`.
//
// THE STOP IS THE SPEND CEILING'S, NOT THE BENCH FLOOR'S. The queue asks `stopBeforeNext` before each attempt
// (`runAttemptQueue`), so a run stopped here starts no further entry, queues no second attempt, and the pages of
// every entry already settled stay shipped. The bench floor ends an entry from the inside as an error, which the
// queue then re-attempts or stalls on.
//
// THE MECHANISM IS AN ABORT. The pass hands every entry a signal that this gate aborts with the refusal as the
// reason, so every call in flight under it ends at once and each layer's "abort wins" rule carries the refusal out
// as it does a user's stop, none of them mistaking it for a timeout or a lost voice. The queue's seam then reads
// the gate and prints the one stop line.
//
// TWO WAYS IN, ONE TRIP. A guarded call that meets the dry router trips it before the refusal propagates. An entry
// the refusal reached some other way (the seat reading finds every provider dry before any chat call is made, or a
// layer re-raises it past the signal) hands it over at the entry's own catch (`trip`), so the pass stops however the
// refusal arrived. Either way the gate logs one line when it trips and hands out its stop note once.
//
// A REFUSAL THAT NAMES A RETURN THE PASS CAN USE PARKS THE PASS INSTEAD OF STOPPING IT (the review of 2026-10-22, M2,
// open to the owner's veto). The router bounds the sum of its waits at 300 s (the budget layer's own refusal cooldown;
// the 360 s call deadline is armed after routing and does not cut the router's wait), so a
// NeuralWatt day starting again 301 s away ended the call with a refusal that states the distance
// (`neuralwattReturnsInMs`). When that return lies inside what the pass has left, ending the run would throw away the
// energy that is about to come back, so the gate parks: the shared signal aborts so the entry in flight ends, the queue
// puts that entry back at the front with no verdict (`runAttemptQueue`'s `interrupted`), waits out the return before
// asking whether the pass goes on (`beforeNext`, which is `awaitPark`), and the attempt that follows starts on a live
// signal. The wait sits between entries, outside every exchange, so no call deadline cuts it, and it is for what is
// left of the return when the queue asks (the return is stored as an instant on the gate's clock, counted from the read
// that named it where the error carries the stamp, so a slice that unwinds late does not push the wait past the
// return). A refusal that names no return, or one past what the pass has left, trips as before and states the time.
// A refusal for the day start of a park that has ended trips, since the account has then not rolled to the new day it
// should have: parking again would wait a day in a loop.
//
// A WRITING BENCH ERROR THAT CARRIES THE RETURN PARKS THE SAME WAY (round 5, open to the owner's veto). With a provider
// still wet, a writing bench below its floor is the bench error and not the router's refusal; when NeuralWatt's return
// would lift that bench to its floor, the seat reader puts the return on the error (`run-seats-refusal.ts`) and the
// entry scheduler hands the error to this gate (`entry-error-outcome.ts`). The same checks apply (the return fits what
// the pass has left, its spent day was not already waited out), and the same requeue follows. A bench error that
// carries no return, or one the pass cannot wait for, is left alone: the entry ends as it did, and only the router's
// refusal ever stops the run.

/**
 Logger root for the gate, naming the entry whose call tripped it.
 */
const passDryStopLog = contextRoot({ tag: 'pass-dry-stop', },);

/**
 What a pass has left when it bounds nothing: no limit.

 @returns Infinity

 @example
 ```ts
 const left = unboundedPass();
 ```
 */
function unboundedPass(): number {
  return Number.POSITIVE_INFINITY;
}

/**
 Waits out a park.

 @param ms - milliseconds to wait

 @example
 ```ts
 await waitOutPark({ ms: 301_000, },);
 ```
 */
async function waitOutPark({ ms, }: { readonly ms: number; },): Promise<void> {
  await wait(ms,);
}

/**
 Gate that turns the first every-provider-dry refusal into a stop of the whole pass, or a park until the energy returns.

 @example
 ```ts
 const dryStop: PassDryStop = createPassDryStop();
 ```
 */
export type PassDryStop = {
  /**
   Aborted with the first refusal as its reason, which every entry's call deadline forwards; a fresh signal once a park
   has ended, so read it when an entry starts and not before.
   */
  readonly signal: AbortSignal;

  /**
   Wraps a client so a dry refusal from any of its calls trips this gate before the refusal propagates.
   */
  readonly guard: <ClientT extends ModelClient,>(args: { readonly client: ClientT; },) => ClientT;

  /**
   Hands the gate a failure that ended an entry, which trips it (or parks it) when the failure is the dry refusal and
   does nothing for any other; for a refusal that reached the entry without passing a guarded call.
   */
  readonly trip: (args: { readonly error: unknown; },) => void;

  /**
   Whether the dry router has been met and the pass stops, so it starts no further entry.
   */
  readonly tripped: () => boolean;

  /**
   Whether a dry refusal parked the pass and the park has not been waited out, so the entry in flight was cut short and
   goes back to the front of the queue.
   */
  readonly parked: () => boolean;

  /**
   Whether the park in force is for this very failure: the object the gate chose to park for, or another parking
   failure it saw while the park lasted (queue attempts run one at a time, so each belongs to the attempt that parked),
   and no other, so an entry that failed for another reason is not mistaken for the one the pass waits to resume.
   */
  readonly parkedFor: (args: { readonly error: unknown; },) => boolean;

  /**
   Waits until the return the park is for, logging the end, and gives the gate a live signal again; nothing where the
   pass is not parked.
   */
  readonly awaitPark: () => Promise<void>;

  /**
   The one line saying why the pass starts no further entry, the first time it is asked after the trip; none before
   the trip and none once it has been handed out, so the stop is announced exactly once.
   */
  readonly takeStopNotes: () => readonly string[];
};

/**
 The return a park waits for.
 */
type DryPark = {
  /**
   The failure the gate chose to park for, which `parkedFor` answers by identity.
   */
  readonly refusal: EveryProviderDryError | WritingBenchUnreachableError;

  /**
   The other parking failures the gate saw while the park lasted, in the order they arrived: a layer above the guard may
   rethrow any of them, so `parkedFor` answers for each of them as it does for `refusal`.
   */
  readonly seen: (EveryProviderDryError | WritingBenchUnreachableError)[];

  /**
   Milliseconds the refusal named between itself and the return, which the park log states.
   */
  readonly returnsInMs: number;

  /**
   The instant the return falls at on the gate's clock, so a park waits for what is left of it.
   */
  readonly atMs: number;

  /**
   The day start of the spent day the return is for, none where the refusal named no dryness.
   */
  readonly dayStart?: string;

  /**
   The time of day the return falls at, with its zone.
   */
  readonly returnsAt: string;
};

/**
 What the gate remembers of its trip and its parks.
 */
type DryStopState = {
  /**
   Owner of the abort every entry's calls forward, replaced once a park has ended.
   */
  controller: AbortController;

  /**
   The first refusal that tripped the gate, whose cause stays the reason.
   */
  cause?: EveryProviderDryError;

  /**
   Whether the stop note has been handed out.
   */
  noted: boolean;

  /**
   The sentence stating the return time that the stop note adds, empty where the refusal named none.
   */
  stopNote: string;

  /**
   The park in force: empty, or the one return the pass waits for.
   */
  readonly parks: DryPark[];

  /**
   The day start of each spent day a park has waited out: a refusal for one of them means the account has not shown the
   new day, and the gate trips.
   */
  readonly endedParks: string[];
};

/**
 Runs one call of a guarded client and hands any failure to the gate before it propagates.

 @param call - the client's own call

 @param trip - what the gate does with a failure

 @returns What the call returned

 @throws Whatever the call raised, unchanged

 @example
 ```ts
 return await tripping({ call: async () => await client.chatText(request,), trip, },);
 ```
 */
async function tripping<ResultT,>(
  {
    call,
    trip,
  }: {
    readonly call: () => Promise<ResultT>;
    readonly trip: (args: { readonly error: unknown; },) => void;
  },
): Promise<ResultT> {
  try {
    return await call();
  }
  catch (error) {
    trip({ error, },);
    throw error;
  }
}

/**
 States the return time of NeuralWatt's energy and why the pass did not wait for it, for the stop line.

 @param returnsInMs - milliseconds until the return, absent where the refusal named none

 @param returnsAt - time of day the return falls at, with its zone

 @param passLeftMs - milliseconds the pass has left

 @returns Sentence, empty where the refusal named no return

 @example
 ```ts
 const note = returnNoteOf({ returnsInMs: 301_000, returnsAt: '19:15:08 UTC', passLeftMs: 300_000, },);
 ```
 */
function returnNoteOf(
  {
    returnsInMs,
    returnsAt,
    passLeftMs,
  }: {
    readonly returnsInMs?: number;
    readonly returnsAt?: string;
    readonly passLeftMs: number;
  },
): string {
  if ((returnsInMs === undefined) || (returnsAt === undefined))
    return '';
  return ` NeuralWatt's energy returns at ${returnsAt}, in ${durationWordsOf({ ms: returnsInMs, },)}, `
    + `which is past the ${durationWordsOf({ ms: passLeftMs, },)} this pass has left.`;
}

/**
 Whether a failure is a refusal the gate acts on.

 @param error - what a guarded call raised, or what ended an entry

 @returns Whether it is the router's every-provider-dry refusal, or a writing bench error that
 carries a NeuralWatt return

 @example
 ```ts
 if (isParkingRefusal(error,))
   state.parks.push(parkFor(error,));
 ```
 */
function isParkingRefusal(error: unknown,): error is EveryProviderDryError | WritingBenchUnreachableError {
  return (error instanceof EveryProviderDryError)
    || ((error instanceof WritingBenchUnreachableError) && (error.neuralwattReturnsInMs !== undefined));
}

/**
 Parks the pass for the return a refusal or a bench error names, or trips the gate on the refusal.

 @param error - what a guarded call raised, or what ended an entry

 @param state - what the gate remembers of its trip and its parks

 @param l - logger the line is written on

 @param passLeftMs - milliseconds the pass has left before its soft budget or hard cap

 @param now - gate's clock

 @mutates state - holds the cause, or the park, and aborts the controller, the first time only; records a further
 parking failure on the park in force

 @example
 ```ts
 refuseOn({ error, state, l, passLeftMs, now, },);
 ```
 */
function refuseOn(
  {
    error,
    state,
    l,
    passLeftMs,
    now,
  }: {
    readonly error: unknown;
    readonly state: DryStopState;
    readonly l: Logger;
    readonly passLeftMs: () => number;
    readonly now: () => number;
  },
): void {
  if (!isParkingRefusal(error,))
    return;
  /**
   The park in force, none where the pass is not parked.
   */
  const [parked,] = state.parks;
  // A PARK OR A TRIP IS MADE ONCE: calls of the same entry that meet the dry router after the first are the same event,
  // but the failure each raises is kept, since the entry may end with any of them.
  if (parked !== undefined) {
    if (!parked.seen
      .includes(error,))
      parked.seen
        .push(error,);
    return;
  }
  if (state.cause !== undefined)
    return;
  /**
   The failure as a refusal the gate acts on: the router's every-provider-dry refusal, or a writing bench error that
   carries the NeuralWatt return which would lift the bench.
   */
  const refusal = error;

  /**
   Milliseconds the pass has left.
   */
  const left = passLeftMs();
  /**
   The day start of the spent day the refusal was made on, none where it names no dryness.
   */
  const dayStart = (refusal.neuralwattDryness
    ?.kind
    === 'spent-day') ? refusal.neuralwattDryness
      .dayStart : undefined;
  /**
   Whether the refusal names a return the pass can wait for, and not for a spent day a park has already waited out.
   */
  const parkable = (refusal.neuralwattReturnsInMs !== undefined) && (refusal.neuralwattReturnsAt !== undefined)
    && returnFitsWithin({
      returnsInMs: refusal.neuralwattReturnsInMs,
      boundMs: left,
    },)
    && ((dayStart === undefined) || (!state.endedParks
      .includes(dayStart,)));
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: refuseOn.name,
    l,
  },);
  if (parkable) {
    /**
     The instant the distance was read at: the seat reader's stamp, on its bench error and on its all-dry refusal
     alike, whose slices in flight finish before the failure reaches the gate, and the moment of the refusal for the
     router's own, which names the distance as it is raised.
     */
    const readAtMs = refusal.neuralwattReadAtMs ?? now();
    state.parks
      .push({
        refusal,
        seen: [],
        returnsInMs: nonNullishOrThrow(refusal.neuralwattReturnsInMs,),
        atMs: readAtMs + nonNullishOrThrow(refusal.neuralwattReturnsInMs,),
        returnsAt: nonNullishOrThrow(refusal.neuralwattReturnsAt,),
        ...((dayStart === undefined) ? {} : { dayStart, }),
      },);
    rl.warn(
      `${
        (refusal instanceof EveryProviderDryError)
          ? 'every provider is out of budget'
          : 'a writing bench is below its floor and NeuralWatt\'s energy would lift it'
      }, so the pass is parked until NeuralWatt's energy returns at `
        + `${nonNullishOrThrow(refusal.neuralwattReturnsAt,)} `
        + `(in ${durationWordsOf({ ms: nonNullishOrThrow(refusal.neuralwattReturnsInMs,), },)}); `
        + 'the entry in flight is left resumable and the pass resumes then '
        + `(cause: ${refusalText({ error: refusal, },)})`,
    );
    state.controller
      .abort(refusal,);
    return;
  }
  // A BENCH ERROR THE PASS CANNOT WAIT FOR ENDS ITS ENTRY AS IT DID BEFORE: only the router's refusal ends the run.
  if (!(refusal instanceof EveryProviderDryError))
    return;
  state.cause = refusal;
  state.stopNote = returnNoteOf({
    ...((refusal.neuralwattReturnsInMs === undefined) ? {} : { returnsInMs: refusal.neuralwattReturnsInMs, }),
    ...((refusal.neuralwattReturnsAt === undefined) ? {} : { returnsAt: refusal.neuralwattReturnsAt, }),
    passLeftMs: left,
  },);
  rl.error(
    `every provider is out of budget, so the pass stops starting entries (cause: ${refusalText({ error: refusal, },)})`,
  );
  state.controller
    .abort(refusal,);
}

/**
 Builds the gate one pass shares across its entries.

 @param l - logger the trip is written on; the gate's own, naming the entry in flight, when absent

 @param passLeftMs - milliseconds the pass has left before its soft budget or hard cap, which a park must fit inside;
 unbounded when absent

 @param waitMs - waits out a park, injectable so a test does not wait; `wait` by default

 @param now - gate's clock, on which a park's return is an instant and what is left of it is waited for;
 `monotonicMs` by default

 @returns Gate whose signal aborts on the first every-provider-dry refusal a guarded client meets

 @example
 ```ts
 const dryStop = createPassDryStop();
 const client = dryStop.guard({ client: input.newClient({ promptPayloadDir, }), },);
 ```
 */
export function createPassDryStop(
  {
    l = passDryStopLog,
    passLeftMs = unboundedPass,
    waitMs = waitOutPark,
    now = monotonicMs,
  }: {
    readonly l?: Logger;
    readonly passLeftMs?: () => number;
    readonly waitMs?: (args: { readonly ms: number; },) => Promise<void>;
    readonly now?: () => number;
  } = {},
): PassDryStop {
  /**
   What the gate remembers of its trip and its parks.
   */
  const state: DryStopState = {
    controller: new AbortController(),
    noted: false,
    stopNote: '',
    parks: [],
    endedParks: [],
  };

  /**
   Hands a failure of a guarded call to the gate.

   @param error - what the call raised
   */
  function trip({ error, }: { readonly error: unknown; },): void {
    refuseOn({
      error,
      state,
      l,
      passLeftMs,
      now,
    },);
  }

  return {
    /**
     The live signal, which a park replaces.
     */
    get signal(): AbortSignal {
      return state.controller
        .signal;
    },

    trip,

    guard: function guard<ClientT extends ModelClient,>(
      { client, }: { readonly client: ClientT; },
    ): ClientT {
      /**
       The chat calls, each failing through the gate.
       */
      const chat: Pick<ModelClient, 'chatJson' | 'chatText'> = {
        chatText: async function guardedText(request,) {
          return await tripping({
            call: async function ask() {
              return await client.chatText(request,);
            },
            trip,
          },);
        },
        chatJson: async function guardedJson(request,) {
          return await tripping({
            call: async function ask() {
              return await client.chatJson(request,);
            },
            trip,
          },);
        },
      };

      /**
       The typed-decision call, present only where the client carries one.
       */
      const decisions: Pick<ModelClient, 'decide'> = (client.decide === undefined)
        ? {}
        : {
          decide: async function guardedDecide(request,) {
            return await tripping({
              call: async function ask() {
                return await nonNullishOrThrow(client.decide,)(request,);
              },
              trip,
            },);
          },
        };
      return {
        ...client,
        ...chat,
        ...decisions,
      };
    },

    tripped: function tripped(): boolean {
      return state.cause !== undefined;
    },

    parked: function parked(): boolean {
      return state.parks
        .length
        > 0;
    },

    parkedFor: function parkedFor({ error, }: { readonly error: unknown; },): boolean {
      /**
       The one park in force, none where the pass is not parked.
       */
      const [park,] = state.parks;
      if (park === undefined)
        return false;
      /**
       Every parking failure the park answers for, typed unknown so any failure can be looked up in it.
       */
      const answered: readonly unknown[] = [
        park.refusal,
        ...park.seen,
      ];
      return answered.includes(error,);
    },

    awaitPark: async function awaitPark(): Promise<void> {
      /**
       The return the park waits for, none where the pass is not parked.
       */
      const [park,] = state.parks;
      if (park === undefined)
        return;
      await waitMs({ ms: Math.max(
        0,
        park.atMs - now(),
      ), },);
      state.parks
        .splice(0,);
      if (park.dayStart !== undefined)
        state.endedParks
          .push(park.dayStart,);
      state.controller = new AbortController();
      tagged({
        tag: createPassDryStop.name,
        l,
      },)
        .info(
        `the pass resumes: NeuralWatt's energy was due at ${park.returnsAt} `
          + `and the park of ${durationWordsOf({ ms: park.returnsInMs, },)} has ended`,
        );
    },

    takeStopNotes: function takeStopNotes(): readonly string[] {
      if ((state.cause === undefined) || state.noted)
        return [];
      state.noted = true;
      return [
        'STOP every provider is out of budget, so no further entry starts; the entries already settled keep '
          + `their pages and the entry in flight is left resumable. Cause: ${state.cause
            .message}${state.stopNote}`,
      ];
    },
  };
}

//endregion Pass dry stop
