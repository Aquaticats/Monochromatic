import {
  type Logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';
import {
  PROVIDER_ORDER,
  type ProviderName,
  type ProviderRecord,
  providerRecord,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { WritingReach, } from './budget-routing.ts';

//region Provider budget give-up aside
// WHEN AN ENTRY'S GIVE-UP OF A PROVIDER IS SET ASIDE, split out of `provider-budget-refusal-holds.ts`.
//
// GIVING A PROVIDER'S COOLDOWNS UP NARROWS WHERE A CALL MAY GO; IT MUST NEVER COST THE ENTRY ANYTHING THAT WAITING WOULD
// HAVE SAVED (the lead's rulings of 2026-10-25, open to the owner's veto; the owner's rule is that a run always ships).
// A given-up provider reads dry for the rest of the entry. When the providers left (not dry, not held, not given up)
// cannot meet every bench floor the entry's stages need, that reading costs the entry its benches: round 2 found the
// run ended by `EveryProviderDryError` where the given-up provider was the only one not dry, and round 3 found the
// entry ended by `WritingBenchUnreachableError` where a wet provider was too small to lift a writing bench (DeepSeek
// serves one of eight translators where the floor is two). So the give-up holds only while the providers left CAN meet
// every floor. Otherwise it is SET ASIDE: the provider's cooldown is waited out as before and the provider is asked
// again, whatever it did after the last cooldown.
//
// "EVERY BENCH FLOOR THE ENTRY NEEDS" IS THE RUN'S, NOT THIS PACKAGE'S. This package knows no roster, so the floors come
// in as `UnmetFloors`: given the dryness a reading would have with only the providers left wet, it names each bench
// floor that would stay unmet, the writing floors and the judging quorums, counted by the seat reader's own helpers
// (`run-seats-give-up-floors.ts`). A caller that knows no roster gets the floor of the round-2 ruling
// (`writingProviderFloor`): some writing provider (`WritingReach`) can still be asked. A provider given up with the
// first one counts as unavailable here, since the give-up would otherwise keep both out and leave no provider to ask.
//
// THE DECISION IS THE BUDGET LAYER'S, TAKEN FROM THE LAST METER READING AND THE HOLDS AS THEY STAND NOW, so the router
// and the seat reader, which read only the dryness and the holds, change nothing: with the give-up set aside the
// provider reads dry while its cooldown runs and wet after it, and its hold counts for a wait. The give-up comes back
// in force the moment the providers left meet every floor, and goes aside again if one of them goes dry or is held.
// Before any reading nothing is known of the others, so the give-up is set aside.
//
// WHEN THE GIVE-UP IS SET ASIDE WHILE A CALL FOR A MODEL ONLY THE GIVEN-UP PROVIDER SERVES IS ROUTED, nothing here
// changes the call: the provider reads dry during its cooldown, so the router ends that call as
// `NoProviderForModelError` with no request sent, which the stage reads as that seat unreachable and not as the end of
// the run (`attemptStageCall`). This module decides only the one question: do the providers left meet every floor?

/**
 Names the bench floors the entry's stages need that would stay unmet if only some providers were asked.

 PURE: the same dryness gives the same clauses, since the decision may ask it again at every read of the budgets.

 @param dry - per provider, whether it cannot be asked in the view under question

 @returns One clause per bench floor that stays unmet, as the `JUDGE SEATS` lines word it; none when every floor is met

 @example
 ```ts
 const unmetFloors: UnmetFloors = function unmetFloors({ dry, }) {
   return dry.hyper ? ['translators 1 of 8 reachable, floor 2',] : [];
 };
 ```
 */
export type UnmetFloors = (args: { readonly dry: ProviderRecord<boolean>; },) => readonly string[];

/**
 The clause of a floor that no writing provider can be asked.
 */
const NO_WRITING_PROVIDER_LEFT = 'no writing provider can be asked';

/**
 The floor of a caller that knows no roster of benches: some writing provider can still be asked.

 @param writing - the providers whose wet reading keeps the run going

 @returns A floor source that finds the one floor unmet while every writing provider is dry

 @example
 ```ts
 const unmetFloors = writingProviderFloor({ writing: EVERY_PROVIDER_COUNTS, },);
 ```
 */
export function writingProviderFloor({ writing, }: { readonly writing: WritingReach; },): UnmetFloors {
  return function noWritingProviderLeft({ dry, },): readonly string[] {
    return PROVIDER_ORDER.some(function canWrite(provider,): boolean {
      return writing[provider] && (!dry[provider]);
    },)
      ? []
      : [NO_WRITING_PROVIDER_LEFT,];
  };
}

/**
 What the give-up decision needs of the refusal holds around it.
 */
type AsideInputs = {
  /**
   The bench floors the providers left must meet.
   */
  readonly unmetFloors: UnmetFloors;

  /**
   The providers the entry has given up.
   */
  readonly given: ReadonlySet<ProviderName>;

  /**
   How much longer a refusal holds a provider out.
   */
  readonly left: (args: { readonly provider: ProviderName; },) => number;

  /**
   The entry the decision is for, worded for a log line ("entry XingZ6020", or "the run" where none is named).
   */
  readonly entryWords: () => string;

  /**
   Logger the decision is written to.
   */
  readonly l: Logger;
};

/**
 The give-up decision of one budget layer.

 @example
 ```ts
 const aside: GiveUpAside = createGiveUpAside({ unmetFloors, given, left, entryWords, l, },);
 ```
 */
export type GiveUpAside = {
  /**
   Records which providers the last meter reading found dry, apart from any refusal hold and any give-up, and decides
   again whether the give-ups are set aside.

   @param dry - per provider, whether its meter, a refused credential or a payment refusal keeps it dry

   @mutates aside - replaces the reading the decision is taken from, and the decision last logged
   */
  readonly noteReading: (args: { readonly dry: ProviderRecord<boolean>; },) => void;

  /**
   Decides again from the reading and the holds as they stand, logging a change.

   @mutates aside - replaces the decision last logged
   */
  readonly reconsider: () => void;

  /**
   Whether the give-ups are set aside now: some provider is given up, and the providers left cannot meet every floor.

   @returns Whether a given-up provider is to be waited for and asked as before
   */
  readonly isAside: () => boolean;

  /**
   Forgets the decision last logged, for an entry that begins with no provider given up.

   @mutates aside - clears the decision last logged
   */
  readonly forget: () => void;
};

/**
 The dryness of a provider before any reading: dry, so nothing is known to keep a give-up in force.

 @returns True

 @example
 ```ts
 const dry = providerRecord({ of: dryBeforeAnyReading, },);
 ```
 */
function dryBeforeAnyReading(): boolean {
  return true;
}

/**
 The providers a decision is about, joined so two decisions can be compared.

 @param names - given-up providers in spending order

 @returns The names joined with commas

 @example
 ```ts
 const signature = signatureOf({ names: ['synthetic', 'hyper',], },); // => 'synthetic,hyper'
 ```
 */
function signatureOf({ names, }: { readonly names: readonly ProviderName[]; },): string {
  return names.join(',',);
}

/**
 Builds the give-up decision of one budget layer.

 @param unmetFloors - the bench floors the providers left must meet

 @param given - the providers the entry has given up, read live

 @param left - how much longer a refusal holds a provider out

 @param entryWords - the entry the decision is for, worded for a log line

 @param l - logger the decision is written to

 @returns A decision that sets every give-up aside until a reading finds the providers left meeting every floor

 @example
 ```ts
 const aside = createGiveUpAside({ unmetFloors, given, left, entryWords, l, },);
 ```
 */
export function createGiveUpAside(
  {
    unmetFloors,
    given,
    left,
    entryWords,
    l,
  }: AsideInputs,
): GiveUpAside {
  /**
   Per provider, whether the last reading found it dry apart from refusal holds and give-ups.
   */
  const reading: { dry: ProviderRecord<boolean>; } = { dry: providerRecord({ of: dryBeforeAnyReading, },), };

  /**
   The providers the decision last logged was about; none when no give-up was set aside.
   */
  const logged: { names: readonly ProviderName[]; } = { names: [], };

  /**
   The providers a call can still go to: not given up, not dry by the last reading, not held by a refusal.

   @returns The providers left, in spending order
   */
  function providersLeft(): readonly ProviderName[] {
    return PROVIDER_ORDER.filter(function isLeft(provider,): boolean {
      return (!given.has(provider,))
        && (!reading.dry[provider])
        && (left({ provider, },) === 0);
    },);
  }

  /**
   The bench floors the providers left cannot meet.

   @returns One clause per unmet floor; none when the providers left meet every floor
   */
  function unmet(): readonly string[] {
    /**
     The providers a call can still go to.
     */
    const remaining = providersLeft();
    return unmetFloors({
      dry: providerRecord({
        of: function dryUnlessLeft(provider,): boolean {
          return !remaining.includes(provider,);
        },
      },),
    },);
  }

  /**
   Whether the give-ups are set aside now.

   @returns Whether a provider is given up and the providers left cannot meet every floor
   */
  function isAside(): boolean {
    // Nothing is weighed for an entry that has given no provider up, which is nearly always.
    if (given.size === 0)
      return false;
    /**
     The bench floors the providers left cannot meet.
     */
    const clauses = unmet();
    return clauses.length > 0;
  }

  /**
   How each provider that is not given up reads, for the line that sets a give-up aside.

   @returns One clause per provider that is not given up
   */
  function othersWords(): readonly string[] {
    /**
     The providers a call can still go to.
     */
    const remaining = providersLeft();
    return PROVIDER_ORDER
      .filter(function isOther(provider,): boolean {
        return !given.has(provider,);
      },)
      .map(function wordOf(provider,): string {
        /**
         How long its refusal still holds it out.
         */
        const heldMs = left({ provider, },);
        if (heldMs > 0)
          return `${provider} held for ${String(heldMs,)}ms`;
        return remaining.includes(provider,) ? `${provider} wet` : `${provider} dry`;
      },);
  }

  /**
   The providers given up, in spending order.

   @returns The given-up providers
   */
  function givenInOrder(): readonly ProviderName[] {
    return PROVIDER_ORDER.filter(function isGiven(provider,): boolean {
      return given.has(provider,);
    },);
  }

  /**
   Decides again from the reading and the holds as they stand, logging a change.
   */
  function reconsider(): void {
    /**
     Logger pre-tagged with this function's name.
     */
    const rl = tagged({
      tag: reconsider.name,
      l,
    },);
    /**
     The entry, worded for the line.
     */
    const entry = entryWords();
    /**
     The given-up providers a set-aside is about, none when the give-ups are in force.
     */
    const names = isAside() ? givenInOrder() : [];
    /**
     The given-up providers this decision is about, joined for comparison.
     */
    const signature = signatureOf({ names, },);
    /**
     The ones last logged, joined for comparison.
     */
    const loggedSignature = signatureOf({ names: logged.names, },);
    if (signature === loggedSignature)
      return;
    /**
     The providers the line last logged was about.
     */
    const before = logged.names;
    logged.names = names;
    if (names.length > 0) {
      /**
       How each provider that is not given up reads.
       */
      const others = othersWords();
      /**
       The bench floors the providers left cannot meet.
       */
      const clauses = unmet();
      rl.warn(
        `${names.join(', ',)}: ${entry} sets its give-up aside and waits out the cooldowns of ${names.join(', ',)} as `
          + `before, because the providers left (${others.join(', ',)}) cannot meet every bench floor without `
          + `${names.join(', ',)}: ${clauses.join('; ',)}; reading ${names.join(', ',)} dry would cost the entry what `
          + 'waiting would have saved',
      );
      return;
    }
    if (before.length > 0) {
      /**
       The providers left, which meet every floor again.
       */
      const wet = providersLeft();
      rl.info(
        `${before.join(', ',)}: ${entry} puts its give-up back in force, because the providers left `
          + `(${wet.join(', ',)}) meet every bench floor again, so ${before.join(', ',)} stays `
          + 'unreachable until the entry ends',
      );
    }
  }

  return {
    isAside,

    reconsider,

    forget: function forget(): void {
      logged.names = [];
    },

    noteReading: function noteReading({ dry, },): void {
      reading.dry = dry;
      reconsider();
    },
  };
}

//endregion Provider budget give-up aside
