import type {
  BoundOwner,
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
 const client: RunClient = createRunClient();
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
   did not.

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

   A NEURALWATT RETURN IS A HOLD AT ANY DISTANCE unless the caller names a bound (`returnBoundMs`): the seat reader sits
   outside every exchange and bounds it by what the pass has left (`passLeftMs`).
   */
  readonly providerHolds: (args?: { readonly returnBoundMs?: number; },) => ProviderRecord<number>;

  /**
   What the run knows of when NeuralWatt's free energy returns, read by the seat reader when it ends the run on every
   provider dry, so the refusal states the account's own day start and never midnight. Absent on a client built over
   budgets that report none, which the refusal then words as an undecided day.
   */
  readonly neuralwattReturn?: () => NeuralWattReturn;

  /**
   Why NeuralWatt reads dry in the last reading, so the seat reader's refusal states only the true cause, and whether
   the hold that keeps it out is its day start (`NeuralWattCause`). Absent on a client built over
   budgets that report none.
   */
  readonly neuralwattDryness?: NeuralWattCause['neuralwattDryness'];

  /**
   Whether NeuralWatt's hold is its day start and not a refusal's; absent where the budgets report none, which the wait
   words as the day start.
   */
  readonly neuralwattHeldByReturn?: NeuralWattCause['neuralwattHeldByReturn'];

  /**
   The providers whose wet reading keeps the run going, because they serve a model seated in a writing bench. The
   router counts a wet provider against "every provider dry" by this record, and the seat reader reads it to word the
   refusal and to decide between the run-ending refusal and the bench error. Absent on a client that carries none,
   which the seat reader reads as the roster's own (`RUN_WRITING_PROVIDERS`).
   */
  readonly writingProviders?: ProviderRecord<boolean>;

  /**
   Milliseconds the seat reader may wait for NeuralWatt's return: the pass's soft budget left or its hard cap, whichever
   is shorter, and inside an entry also what that entry's own cap has left (`pass-entry-bound.ts`), so a wait for a
   NeuralWatt return never outlasts the entry's timer. A return inside it is waited for, one past it is refused at
   once with its time stated, since the pass or the entry would end first. Absent for a client built outside a pass,
   which bounds the wait by the entry's abort alone.

   ONLY A NEURALWATT RETURN IS FILTERED BY THIS BOUND. A stated hold (Hyper's daily limit, a rate limit's) and a refusal
   cooldown are waited for whatever the bound is and are cut by the entry's abort when the entry's timer ends first.
   */
  readonly passLeftMs?: () => number;

  /**
   Whose bound `passLeftMs` is: the entry when its own hard cap has less left than the pass, the pass otherwise, so a
   refusal past the bound says "the entry has left" or "this pass has left" for the bound that won. Absent where the
   client carries no entry bound, which the sentence reads as the pass's.
   */
  readonly passBoundOwner?: () => BoundOwner;
};

//endregion Run client contract
