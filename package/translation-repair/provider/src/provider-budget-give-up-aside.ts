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
// GIVING A PROVIDER'S COOLDOWNS UP NARROWS WHERE A CALL MAY GO; IT MUST NEVER BE THE CAUSE OF A RUN ENDING (the lead's
// ruling of 2026-10-25, open to the owner's veto; the owner's rule is that a run always ships). A given-up provider
// reads dry for the rest of the entry, and when it was the only writing provider not dry that leaves every provider dry:
// the router and the seat reader raise `EveryProviderDryError`, which ends the run, for a provider that would serve
// again after its cooldown. So the give-up holds only while another provider that keeps the run going reads wet.
// Without one, the give-up is SET ASIDE: the provider's cooldown is waited out as before and the provider is asked
// again, whatever it did after the last cooldown.
//
// "ANOTHER PROVIDER THAT KEEPS THE RUN GOING" IS A WRITING PROVIDER (`WritingReach`) that reads wet by its meter, is not
// held by a refusal or a refused credential or a payment refusal, and is not itself given up. A wet provider that
// writes nothing (Xiaomi, whose one seat is a checker) is no alternative, as it is none against "every provider dry".
// A provider given up with the first one counts as dry here, since the give-up would otherwise keep both out and leave
// no provider to ask.
//
// THE DECISION IS THE BUDGET LAYER'S, TAKEN FROM THE LAST METER READING AND THE HOLDS AS THEY STAND NOW, so the router
// and the seat reader, which read only the dryness and the holds, change nothing: with the give-up set aside the
// provider reads dry while its cooldown runs and wet after it, and its hold counts for a wait. The give-up comes back
// in force the moment another writing provider reads wet, and goes aside again if that provider goes dry or is held.
// Before any reading nothing is known of the others, so the give-up is set aside.
//
// WHEN THE GIVE-UP IS SET ASIDE WHILE A CALL FOR A MODEL ONLY THE GIVEN-UP PROVIDER SERVES IS ROUTED, nothing here
// changes the call: the provider reads dry during its cooldown, so the router ends that call as
// `NoProviderForModelError` with no request sent, which the stage reads as that seat unreachable and not as the end of
// the run (`attemptStageCall`). This module decides only the one question: is another writing provider wet?

/**
 What the give-up decision needs of the refusal holds around it.
 */
type AsideInputs = {
  /**
   The providers whose wet reading keeps the run going.
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
 const aside: GiveUpAside = createGiveUpAside({ writing, given, left, entryWords, l, },);
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
   Whether the give-ups are set aside now: some provider is given up, and no other writing provider reads wet.

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

 @param writing - the providers whose wet reading keeps the run going

 @param given - the providers the entry has given up, read live

 @param left - how much longer a refusal holds a provider out

 @param entryWords - the entry the decision is for, worded for a log line

 @param l - logger the decision is written to

 @returns A decision that sets every give-up aside until a reading finds another writing provider wet

 @example
 ```ts
 const aside = createGiveUpAside({ writing, given, left, entryWords, l, },);
 ```
 */
export function createGiveUpAside(
  {
    writing,
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
   The writing providers that read wet and that nothing else keeps out, in spending order.

   @returns Providers a call can still go to that keep the run going
   */
  function wetAlternatives(): readonly ProviderName[] {
    return PROVIDER_ORDER.filter(function isAlternative(provider,): boolean {
      return writing[provider]
        && (!given.has(provider,))
        && (!reading.dry[provider])
        && (left({ provider, },) === 0);
    },);
  }

  /**
   Whether the give-ups are set aside now.

   @returns Whether a provider is given up and no other writing provider reads wet
   */
  function isAside(): boolean {
    /**
     The writing providers a call can still go to.
     */
    const alternatives = wetAlternatives();
    return (given.size > 0) && (alternatives.length === 0);
  }

  /**
   How each other writing provider reads, for the line that sets a give-up aside.

   @returns One clause per writing provider that is not given up
   */
  function othersWords(): readonly string[] {
    /**
     The writing providers that are not given up.
     */
    const others = PROVIDER_ORDER.filter(function isOther(provider,): boolean {
      return writing[provider] && (!given.has(provider,));
    },);
    return others.map(function wordOf(provider,): string {
      /**
       How long its refusal still holds it out.
       */
      const heldMs = left({ provider, },);
      return (heldMs > 0) ? `${provider} held for ${String(heldMs,)}ms` : `${provider} dry`;
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
       How each other writing provider reads.
       */
      const others = othersWords();
      rl.warn(
        `${names.join(', ',)}: ${entry} sets its give-up aside and waits out the cooldowns of ${names.join(', ',)} `
          + `as before, because no other writing provider reads wet (${others.join(', ',)}), and reading `
          + `${names.join(', ',)} dry would end the run for a provider that serves again after its cooldown`,
      );
      return;
    }
    if (before.length > 0) {
      /**
       The writing providers that read wet again.
       */
      const wet = wetAlternatives();
      rl.info(
        `${before.join(', ',)}: ${entry} puts its give-up back in force, because ${wet.join(', ',)} reads wet `
          + `again, so ${before.join(', ',)} stays unreachable until the entry ends`,
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
