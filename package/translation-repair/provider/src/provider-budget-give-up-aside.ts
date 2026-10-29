import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import {
  PROVIDER_ORDER,
  type ProviderName,
  type ProviderRecord,
  providerRecord,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { WritingReach, } from './budget-routing.ts';

//region Provider budget give-up aside
// WHEN AN ENTRY'S GIVE-UP OF A PROVIDER HOLDS AND WHEN IT IS SET ASIDE, split out of `provider-budget-refusal-holds.ts`.
//
// WAITING FOR A GIVEN-UP PROVIDER BUYS SOMETHING ONLY WHEN ITS RETURN WOULD MAKE THE ENTRY FINISHABLE (the lead's
// ruling of 2026-10-25 as corrected in round 4, open to the owner's veto; round 3's wording, that the providers left
// must meet every floor, was the lead's error and is superseded). A given-up provider reads dry for the rest of the
// entry, which costs the entry nothing when waiting could not have helped it: XingZ6020 refused five benches short with
// Synthetic back as well as without it, so its 58 waits bought a refusal each time and no finished entry. The give-up
// is SET ASIDE, and the cooldown waited out as before, only when
//
//   (a) the given-up provider is the only writing provider not dry (round 2's guard: a give-up never ends the run,
//       which the owner's rule that a run always ships requires), or
//   (b) with that provider back no bench floor the entry needs would be left unmet, while without it at least one is.
//
// Otherwise the give-up HOLDS: no further waits for that provider's cooldowns in the entry. Reading (b) as "the return
// lifts any unmet floor" would set the give-up aside in XingZ6020, where Synthetic lifts two benches and leaves five
// short, and so is not the rule.
//
// "EVERY BENCH FLOOR THE ENTRY NEEDS" IS THE RUN'S, NOT THIS PACKAGE'S. This package knows no roster, so the floors come
// in as `UnmetFloors`: given the dryness a reading would have with only some providers wet, it names each bench floor
// that would stay unmet, the writing floors and the judging quorums, counted by the seat reader's own helpers
// (`run-seats-give-up-floors.ts`). The writing roster (`WritingReach`) comes in beside it, for (a) alone. A caller that
// knows no roster of benches gets the floor of the round-2 ruling (`writingProviderFloor`): some writing provider can be
// asked, which makes (a) and (b) one question.
//
// THE FLOORS ARE READ WITH THE GIVEN-UP PROVIDER ABSENT AND THEN WITH IT BACK, and nothing else differs between the two
// views: the providers left (not dry by the last meter reading, not held by a refusal, not given up) are wet in both.
// A provider given up with the first counts as unavailable in both, since the give-up would otherwise keep both out and
// leave no provider to ask. A provider whose own meter reads dry does not come back by waiting, so it is not added to
// the view with it back.
//
// THE DECISION IS THE BUDGET LAYER'S, TAKEN FROM THE LAST METER READING AND THE HOLDS AS THEY STAND NOW, so the router
// and the seat reader, which read only the dryness and the holds, change nothing: with the give-up set aside the
// provider reads dry while its cooldown runs and wet after it, and its hold counts for a wait. It is taken for each
// given-up provider, since (b) asks about that provider's own return. Before any reading nothing is known of the
// others, so a writing provider given up is the only one not dry and (a) sets its give-up aside.
//
// WHEN THE GIVE-UP IS SET ASIDE WHILE A CALL FOR A MODEL ONLY THE GIVEN-UP PROVIDER SERVES IS ROUTED, nothing here
// changes the call: the provider reads dry during its cooldown, so the router ends that call as
// `NoProviderForModelError` with no request sent, which the stage reads as that seat unreachable and not as the end of
// the run (`attemptStageCall`).

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
 What the entry does about a given-up provider's cooldowns: `held` stops waiting for them, `only-writer` and
 `return-cures` set the give-up aside and wait them out as before, for the reason the name gives.
 */
export type GiveUpVerdictKind = 'held' | 'only-writer' | 'return-cures';

/**
 What the decision found for one given-up provider, and the floors that make the reason.
 */
type GiveUpVerdict = {
  /**
   The decision.
   */
  readonly kind: GiveUpVerdictKind;

  /**
   For `return-cures`, the floors unmet without the provider; for `held`, the floors that stay unmet with it back, none
   when the providers left meet every floor without it; none for `only-writer`.
   */
  readonly floors: readonly string[];
};

/**
 What the give-up decision needs of the refusal holds around it.
 */
type AsideInputs = {
  /**
   The bench floors the entry's stages need.
   */
  readonly unmetFloors: UnmetFloors;

  /**
   The providers whose wet reading keeps the run going, which (a) reads.
   */
  readonly writing: WritingReach;

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
 const aside: GiveUpAside = createGiveUpAside({ unmetFloors, writing, given, left, entryWords, l, },);
 ```
 */
export type GiveUpAside = {
  /**
   Records which providers the last meter reading found dry, apart from any refusal hold and any give-up, and decides
   again for every provider given up.

   @param dry - per provider, whether its meter, a refused credential or a payment refusal keeps it dry

   @mutates aside - replaces the reading the decision is taken from, and the decisions last logged
   */
  readonly noteReading: (args: { readonly dry: ProviderRecord<boolean>; },) => void;

  /**
   Decides again from the reading and the holds as they stand, logging each change.

   @mutates aside - replaces the decisions last logged
   */
  readonly reconsider: () => void;

  /**
   Whether a provider's give-up is set aside now: the provider is given up, and it is the only writing provider not
   dry, or its return would leave no bench floor unmet where without it one is.

   @param provider - provider to check

   @returns Whether the provider is to be waited for and asked as before
   */
  readonly isAside: (args: { readonly provider: ProviderName; },) => boolean;

  /**
   Whether waiting for a provider's return is what lets the entry finish, given up or not: the provider is the only
   writing provider not dry, or its return would leave no bench floor unmet where without it one is. The same decision
   as `isAside` without the give-up it is about, so a return that served nothing yields to the same two exceptions.

   @param provider - provider to check

   @returns Whether a wait for the provider's return is worth taking
   */
  readonly waitingHelps: (args: { readonly provider: ProviderName; },) => boolean;

  /**
   Forgets the decisions last logged, for an entry that begins with no provider given up.

   @mutates aside - clears the decisions last logged
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
 The decision of a provider that has none logged.

 @returns The name of no decision

 @example
 ```ts
 const logged = providerRecord({ of: nothingLoggedYet, },);
 ```
 */
function nothingLoggedYet(): GiveUpVerdictKind | 'none' {
  return 'none';
}

/**
 Builds the give-up decision of one budget layer.

 @param unmetFloors - the bench floors the entry's stages need

 @param writing - the providers whose wet reading keeps the run going

 @param given - the providers the entry has given up, read live

 @param left - how much longer a refusal holds a provider out

 @param entryWords - the entry the decision is for, worded for a log line

 @param l - logger the decision is written to

 @returns A decision that sets a give-up aside only when the given-up provider is the only writing provider not dry or
 its return would leave no floor unmet where without it one is

 @example
 ```ts
 const aside = createGiveUpAside({ unmetFloors, writing, given, left, entryWords, l, },);
 ```
 */
export function createGiveUpAside(
  {
    unmetFloors,
    writing,
    given,
    left,
    entryWords,
    l,
  }: AsideInputs,
): GiveUpAside {
  /**
   Per provider, whether the last reading found it dry apart from refusal holds and give-ups, and whether there has been
   a reading at all.
   */
  const reading: {
    dry: ProviderRecord<boolean>;
    taken: boolean;
  } = {
    dry: providerRecord({ of: dryBeforeAnyReading, },),
    taken: false,
  };

  /**
   The decision last logged for each provider, `none` for one that has none.
   */
  const logged: Record<ProviderName, GiveUpVerdictKind | 'none'> = providerRecord({ of: nothingLoggedYet, },);

  /**
   The providers a call can still go to: not given up, not dry by the last reading, not held by a refusal.

   @param except - provider the question is about, which is never among the providers left (default: none)

   @returns The providers left, in spending order
   */
  function providersLeft({ except, }: { readonly except?: ProviderName; } = {},): readonly ProviderName[] {
    return PROVIDER_ORDER.filter(function isLeft(provider,): boolean {
      return (provider !== except)
        && (!given.has(provider,))
        && (!reading.dry[provider])
        && (left({ provider, },) === 0);
    },);
  }

  /**
   The bench floors that stay unmet when only some providers can be asked.

   @param wet - the providers that can be asked

   @returns One clause per unmet floor; none when every floor is met
   */
  function floorsWith({ wet, }: { readonly wet: readonly ProviderName[]; },): readonly string[] {
    return unmetFloors({
      dry: providerRecord({
        of: function dryUnlessWet(provider,): boolean {
          return !wet.includes(provider,);
        },
      },),
    },);
  }

  /**
   What the entry does about a given-up provider's cooldowns.

   @param provider - provider the entry has given up

   @returns `only-writer` when the provider is the only writing provider not dry, `return-cures` when its return would
   leave no floor unmet where without it one is, `held` otherwise
   */
  function verdictOf({ provider, }: { readonly provider: ProviderName; },): GiveUpVerdict {
    /**
     The providers a call can still go to, the provider itself not among them.
     */
    const remaining = providersLeft({ except: provider, },);
    /**
     Whether the provider's own meter reads dry, which waiting does not cure; before any reading nothing is known against
     it.
     */
    const meterDry = reading.taken && reading.dry[provider];
    // (a) A GIVE-UP NEVER ENDS THE RUN: the provider is the only writing provider not dry.
    if (writing[provider]
      && (!meterDry)
      && (!remaining.some(function writes(other,): boolean {
        return writing[other];
      },)))
      return {
        kind: 'only-writer',
        floors: [],
      };
    /**
     The floors that stay unmet with the provider absent.
     */
    const without = floorsWith({ wet: remaining, },);
    // Nothing to cure: the providers left meet every floor, so the provider's return buys nothing.
    if (without.length === 0) {
      return {
        kind: 'held',
        floors: [],
      };
    }
    /**
     The providers that can be asked with the provider back, the providers left unchanged.
     */
    const wetWithIt = meterDry
      ? remaining
      : [
        ...remaining,
        provider,
      ];
    /**
     The floors that stay unmet with the provider back.
     */
    const back = floorsWith({ wet: wetWithIt, },);
    // (b) ITS RETURN WOULD LEAVE NO FLOOR UNMET WHERE WITHOUT IT ONE IS.
    if (back.length === 0) {
      return {
        kind: 'return-cures',
        floors: without,
      };
    }
    return {
      kind: 'held',
      floors: back,
    };
  }

  /**
   How each provider that is not given up reads, for the lines that say what the decision found.

   @returns One clause per provider that is not given up, joined with commas
   */
  function othersWords(): string {
    /**
     The providers a call can still go to.
     */
    const remaining = providersLeft();
    /**
     The providers that are not given up.
     */
    const others = PROVIDER_ORDER.filter(function isOther(provider,): boolean {
      return !given.has(provider,);
    },);
    /**
     One clause for each of them.
     */
    const clauses = others.map(function wordOf(provider,): string {
      /**
       How long its refusal still holds it out.
       */
      const heldMs = left({ provider, },);
      if (heldMs > 0)
        return `${provider} held for ${String(heldMs,)}ms`;
      return remaining.includes(provider,) ? `${provider} wet` : `${provider} dry`;
    },);
    return clauses.join(', ',);
  }

  /**
   The reason a decision gives, worded for a log line.

   @param provider - provider the decision is about

   @param verdict - the decision

   @returns The reason, which names the condition that set the give-up aside or the floors that stay unmet with the
   provider back
   */
  function reasonOf(
    {
      provider,
      verdict,
    }: {
      readonly provider: ProviderName;
      readonly verdict: GiveUpVerdict;
    },
  ): string {
    /**
     How each provider that is not given up reads.
     */
    const others = othersWords();
    /**
     What the decision found and the floors that make the reason.
     */
    const {
      kind,
      floors,
    } = verdict;
    if (kind === 'only-writer') {
      return `${provider} is the only writing provider not dry (${others}), so reading it dry would end the run for a `
        + 'provider that serves again after its cooldown';
    }
    /**
     The floors, joined for the line.
     */
    const unmet = floors.join('; ',);
    if (kind === 'return-cures') {
      return `with ${provider} back no bench floor the entry needs would be left unmet, while without it these stay `
        + `unmet: ${unmet} (${others})`;
    }
    if (floors.length === 0) {
      return `the providers left (${others}) meet every bench floor without ${provider}, so its return buys nothing`;
    }
    return `with ${provider} back the providers left (${others}) would still leave these bench floors unmet: `
      + `${unmet}; waiting for ${provider} would not make the entry finishable`;
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
   Decides again from the reading and the holds as they stand, logging each change.
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
    for (const provider of givenInOrder()) {
      /**
       What the entry does about this provider now.
       */
      const verdict = verdictOf({ provider, },);
      /**
       What was last logged about it.
       */
      const before = logged[provider];
      if (before === verdict.kind)
        continue;
      logged[provider] = verdict.kind;
      /**
       Why the decision is what it is.
       */
      const reason = reasonOf({
        provider,
        verdict,
      },);
      if (verdict.kind !== 'held') {
        rl.warn(
          `${provider}: ${entry} sets its give-up aside and waits out the cooldowns of ${provider} as before, because `
            + `${reason}; reading ${provider} dry would cost the entry what waiting would have saved`,
        );
        continue;
      }
      if (before === 'none') {
        rl.warn(
          `${provider}: ${entry} keeps its give-up in force and stops waiting for ${provider}'s cooldowns, because ${reason}`,
        );
        continue;
      }
      rl.info(
        `${provider}: ${entry} puts its give-up back in force, because ${reason}; ${provider} stays unreachable until `
          + 'the entry ends',
      );
    }
  }

  return {
    reconsider,

    isAside: function isAside({ provider, },): boolean {
      // Nothing is weighed for a provider the entry has not given up, which is nearly always.
      if (!given.has(provider,))
        return false;
      /**
       What the entry does about the provider now.
       */
      const { kind, } = verdictOf({ provider, },);
      return kind !== 'held';
    },

    waitingHelps: function waitingHelps({ provider, },): boolean {
      /**
       What the entry does about the provider now.
       */
      const { kind, } = verdictOf({ provider, },);
      return kind !== 'held';
    },

    forget: function forget(): void {
      for (const provider of PROVIDER_ORDER) {
        logged[provider] = 'none';
      }
    },

    noteReading: function noteReading({ dry, },): void {
      reading.dry = dry;
      reading.taken = true;
      reconsider();
    },
  };
}

//endregion Provider budget give-up aside
