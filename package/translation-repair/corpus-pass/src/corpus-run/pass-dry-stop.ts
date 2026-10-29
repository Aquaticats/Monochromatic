import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  EveryProviderDryError,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider/ts';
import { refusalText, } from '@monochromatic-dev/translation-repair-document-structure/ts';
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
// THE PASS NEVER PARKS FOR NEURALWATT'S DAY TO START AGAIN (the owner's answer of 2026-10-29, "Never wait, relaunch":
// "Stop at once and relaunch later."). When the refusal says stop, the pass stops, whatever the return time; the stop
// note carries the refusal's own message, which states the time of day in UTC NeuralWatt's day starts again, so the
// operator knows when to relaunch. A writing bench error never reaches this gate: it ends its entry and not the run.

/**
 Logger root for the gate, naming the entry whose call tripped it.
 */
const passDryStopLog = contextRoot({ tag: 'pass-dry-stop', },);

/**
 Gate that turns the first every-provider-dry refusal into a stop of the whole pass.

 @example
 ```ts
 const dryStop: PassDryStop = createPassDryStop();
 ```
 */
export type PassDryStop = {
  /**
   Aborted with the first refusal as its reason, which every entry's call deadline forwards.
   */
  readonly signal: AbortSignal;

  /**
   Wraps a client so a dry refusal from any of its calls trips this gate before the refusal propagates.
   */
  readonly guard: <ClientT extends ModelClient,>(args: { readonly client: ClientT; },) => ClientT;

  /**
   Hands the gate a failure that ended an entry, which trips it when the failure is the dry refusal and does nothing
   for any other; for a refusal that reached the entry without passing a guarded call.
   */
  readonly trip: (args: { readonly error: unknown; },) => void;

  /**
   Whether the dry router has been met and the pass stops, so it starts no further entry.
   */
  readonly tripped: () => boolean;

  /**
   The one line saying why the pass starts no further entry, the first time it is asked after the trip; none before
   the trip and none once it has been handed out, so the stop is announced exactly once.
   */
  readonly takeStopNotes: () => readonly string[];
};

/**
 What the gate remembers of its trip.
 */
type DryStopState = {
  /**
   The first refusal that tripped the gate, whose cause stays the reason.
   */
  cause?: EveryProviderDryError;

  /**
   Whether the stop note has been handed out.
   */
  noted: boolean;
};

/**
 Runs one call of a guarded client and hands any failure to the gate before it propagates.

 @param call - the client's own call

 @param trip - what the gate does with a failure

 @returns What the call returned

 @throws Whatever the call raised, unchanged

 @example
 ```ts
 return tripping({ call: async () => await client.chatText(request,), trip, },);
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
 Trips the gate on the router's every-provider-dry refusal, once.

 @param error - what a guarded call raised, or what ended an entry

 @param state - what the gate remembers of its trip

 @param controller - owner of the abort every entry's calls forward

 @param l - logger the line is written on

 @mutates state - holds the cause, the first time only

 @example
 ```ts
 refuseOn({ error, state, controller, l, },);
 ```
 */
function refuseOn(
  {
    error,
    state,
    controller,
    l,
  }: {
    readonly error: unknown;
    readonly state: DryStopState;
    readonly controller: AbortController;
    readonly l: Logger;
  },
): void {
  // A TRIP IS MADE ONCE: calls of the same entry that meet the dry router after the first are the same event.
  if ((!(error instanceof EveryProviderDryError)) || (state.cause !== undefined))
    return;
  state.cause = error;
  tagged({
    tag: refuseOn.name,
    l,
  },)
    .error(
    `every provider is out of budget, so the pass stops starting entries (cause: ${refusalText({ error, },)})`,
    );
  controller.abort(error,);
}

/**
 Builds the gate one pass shares across its entries.

 @param l - logger the trip is written on; the gate's own, naming the entry in flight, when absent

 @returns Gate whose signal aborts on the first every-provider-dry refusal a guarded client meets

 @example
 ```ts
 const dryStop = createPassDryStop();
 const client = dryStop.guard({ client: input.newClient({ promptPayloadDir, }), },);
 ```
 */
export function createPassDryStop({ l = passDryStopLog, }: { readonly l?: Logger; } = {},): PassDryStop {
  /**
   What the gate remembers of its trip.
   */
  const state: DryStopState = { noted: false, };

  /**
   Owner of the abort every entry's calls forward.
   */
  const controller = new AbortController();

  /**
   Hands a failure of a guarded call to the gate.

   @param error - what the call raised
   */
  function trip({ error, }: { readonly error: unknown; },): void {
    refuseOn({
      error,
      state,
      controller,
      l,
    },);
  }

  return {
    signal: controller.signal,

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

    takeStopNotes: function takeStopNotes(): readonly string[] {
      if ((state.cause === undefined) || state.noted)
        return [];
      state.noted = true;
      return [
        'STOP every provider is out of budget, so no further entry starts; the entries already settled keep '
          + `their pages and the entry in flight is left resumable. Cause: ${state.cause
            .message}`,
      ];
    },
  };
}

//endregion Pass dry stop
