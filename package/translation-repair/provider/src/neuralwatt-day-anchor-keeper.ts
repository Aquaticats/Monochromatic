import {
  decideNeuralWattDayAnchor,
  type NeuralWattDayAnchor,
} from './neuralwatt-day-anchor.ts';
import {
  NEURALWATT_QUOTA_RETRY_SECONDS,
  type NeuralWattDayStart,
  neuralWattSecondsAfter,
} from './neuralwatt-day.ts';
import type { NeuralWattQuotaSession, } from './neuralwatt-quota-session.ts';

//region NeuralWatt day anchor keeper
// ONLY A KNOWN ANCHOR IS KEPT. The owner wants the free energy used "front and center", and one transient quota
// failure at the start of a run (with no `TRANSLATION_REPAIR_NEURALWATT_DAY_START`) must not refuse NeuralWatt for the
// run and send every call to a paid provider. An unknown anchor is therefore tried again, from a fresh quota reading
// and not from the failed baseline, no sooner than a minute after the last failed attempt on the injected clock; the
// calls in between stay refused. The outage is warned about once (the first failed attempt), and the day that is
// finally learned is logged with where it came from (`neuralwatt-day-anchor.ts`).

/**
 Keeper of one client's day anchor.

 @example
 ```ts
 const keeper = createNeuralWattAnchorKeeper({ quota, setting, now, },);
 ```
 */
export type NeuralWattAnchorKeeper = {
  /**
   The anchor, learned on the first call that can and tried again, spaced, while it is unknown.
   */
  readonly anchorFor: (args: { readonly signal: AbortSignal; },) => Promise<NeuralWattDayAnchor>;

  /**
   The anchor this keeper has already learned, without learning one: unknown until a call of `anchorFor` has found it.
   NO QUOTA READ, so a message built while the run ends can ask it.
   */
  readonly decided: () => NeuralWattDayAnchor;
};

/**
 Builds the anchor keeper of one client.

 @param quota - the client's quota session, whose baseline carries the first attempt

 @param setting - the day-start setting, empty where the owner set none

 @param now - clock, injected so that tests wait for nothing

 @returns Keeper surface

 @example
 ```ts
 const keeper = createNeuralWattAnchorKeeper({ quota, setting, now, },);
 ```
 */
export function createNeuralWattAnchorKeeper(
  {
    quota,
    setting,
    now,
  }: {
    readonly quota: NeuralWattQuotaSession;
    readonly setting: NeuralWattDayStart;
    readonly now: () => Date;
  },
): NeuralWattAnchorKeeper {
  /**
   What the keeper holds: the anchor once known, the attempt in flight, and when the next attempt may be made.
   */
  const state: {
    known?: NeuralWattDayAnchor;
    deciding?: Promise<NeuralWattDayAnchor>;
    retryAfter?: Date;
    failedBefore: boolean;
  } = { failedBefore: false, };

  /**
   The anchor, learned on the first call that can and tried again, spaced, while it is unknown.

   @param signal - abort signal for the quota read

   @returns The anchor, or that it is unknown
   */
  async function anchorFor({ signal, }: { readonly signal: AbortSignal; },): Promise<NeuralWattDayAnchor> {
    if (state.known !== undefined)
      return state.known;
    if (state.deciding !== undefined)
      return await state.deciding;
    if ((state.retryAfter !== undefined) && (now() < state.retryAfter))
      return { known: false, };
    state.deciding = decideNeuralWattDayAnchor({
      quota,
      setting,
      now,
      signal,
      retry: state.failedBefore,
      warn: !state.failedBefore,
    },);

    /**
     Clears the attempt in flight when this function ends, however it ends.
     */
    using clearing = {
      [Symbol.dispose]: function clear(): void {
        delete state.deciding;
      },
    };
    void clearing;

    /**
     What the attempt found.
     */
    const anchor = await state.deciding;
    if (anchor.known) {
      state.known = anchor;
      return anchor;
    }
    state.failedBefore = true;
    state.retryAfter = neuralWattSecondsAfter({
      from: now(),
      seconds: NEURALWATT_QUOTA_RETRY_SECONDS,
    },);
    return anchor;
  }

  /**
   The anchor already learned, without learning one.

   @returns The kept anchor's time of day, without the fresh quota a retried anchor carries, or that none is known yet
   */
  function decided(): NeuralWattDayAnchor {
    /**
     The kept anchor, if one has been learned.
     */
    const kept = state.known;
    if (kept?.known !== true)
      return { known: false, };
    return {
      known: true,
      msOfDay: kept.msOfDay,
    };
  }
  return {
    anchorFor,
    decided,
  };
}

//endregion NeuralWatt day anchor keeper
