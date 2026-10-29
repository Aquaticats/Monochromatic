import type {
  BudgetView,
  MeterState,
  ModelClient,
  NeuralWattCause,
  NeuralWattReturn,
} from '@monochromatic-dev/translation-repair-provider/ts';
import type { ProviderRecord, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Run client contract
// THE CLIENT A CORPUS RUN HOLDS: the routed caller every stage calls, the
// first provider's meter that older callers still read, and the dryness view
// the seat reader derives benches from.
//
// A SEPARATE FILE so the entry drivers and the seat reader can name the type
// without importing `run-config.ts`, which loads the whole roster and refuses
// invalid role compositions at import time.

/**
 Client every corpus-run entrypoint is handed.

 @example
 ```ts
 const client: RunClient = createRunClient({ maxHoldMs: 25_200_000, },);
 ```
 */
export type RunClient = ModelClient & {
  /**
   Which providers are out of budget right now, holds folded in, as the
   router itself sees them.

   READ BEFORE EACH PHASE by `run-seats.ts`, which withholds a judge seat
   while the provider that would serve it is one that serves it too slowly
   for the round window, or one the owner declined to pay that model's rate
   on. The view is the router's own, so the seat reader and the router agree
   about where a call would go.
   */
  readonly providerDryness: (args: { readonly signal: AbortSignal; },) => Promise<BudgetView>;

  /**
   What each provider's meter said in the reading the dryness view is drawn
   from: `wet` or `dry` for a meter that answered, `unreadable` for one that
   did not. A provider refused for payment reads `dry` until its meter moves,
   whatever the meter said, since that refusal keeps it dry however long the
   hold it stated.

   READ BESIDE THE DRYNESS by `run-seats-read.ts`, so the `JUDGE SEATS` line
   can say that a provider the router treats as spendable is one whose meter
   said nothing, which the dryness bit alone cannot tell from a meter that
   answered wet.
   */
  readonly providerMeterStates: (args: { readonly signal: AbortSignal; },) => Promise<ProviderRecord<MeterState>>;

  /**
   How long each provider's last refusal still holds it out, in
   milliseconds, zero when it is not held.
   READ BESIDE THE DRYNESS by `run-seats-read.ts` since the thirteenth class:
   a bench that cannot reach quorum among the seats a wet provider serves
   waits out the shortest running hold once, as the router does when every
   provider reads dry, rather than seating a bench short of its quorum, which
   decides on a share of the seats a wet provider serves, or is an outage
   below two of them (`doc/decision/translation-repair-short-bench-share.md`,
   ledger B27).
   `pass-reseat-hook.ts` reads it too, to re-seat only while a hold runs.

   NEURALWATT'S SPENT DAY IS NO HOLD: nothing waits for its return (the owner's answer of 2026-10-29, "Never wait,
   relaunch").
   */
  readonly providerHolds: () => ProviderRecord<number>;

  /**
   What the run knows of when NeuralWatt's free energy returns, read by the seat reader when it ends the run on every
   provider dry, so the refusal states the account's own day start and never midnight. Absent on a client built over
   budgets that report none, which the refusal then words as an undecided day.
   */
  readonly neuralwattReturn?: () => NeuralWattReturn;

  /**
   Why NeuralWatt reads dry in the last reading, so the seat reader's refusal states only the true cause
   (`NeuralWattCause`). Absent on a client built over budgets that report none.
   */
  readonly neuralwattDryness?: NeuralWattCause['neuralwattDryness'];

  /**
   The providers whose wet reading keeps the run going, because they serve a model seated in a writing bench. The
   router counts a wet provider against "every provider dry" by this record, and the seat reader reads it to word the
   refusal and to decide between the run-ending refusal and the bench error. Absent on a client that carries none,
   which the seat reader reads as the roster's own (`RUN_WRITING_PROVIDERS`).
   */
  readonly writingProviders?: ProviderRecord<boolean>;

  /**
   Starts an entry with no provider given up on: a provider that refused right after a refusal cooldown ran out, with
   nothing served in between, stops being waited for until the entry ends, and the entry's name is written where it is
   given up on. Absent on a client built outside a pass, which keeps one set of marks for its whole life.
   */
  readonly beginEntry?: (args: { readonly entryId: string; },) => void;
};

//endregion Run client contract
