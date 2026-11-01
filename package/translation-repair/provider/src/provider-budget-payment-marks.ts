import {
  type ProviderName,
  type ProviderRecord,
  providerRecord,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type PaymentMark,
  unmarkedPayment,
} from './provider-budget-defaults.ts';

//region Provider budget payment marks
// SPLIT OUT OF `provider-budget.ts` at its line budget: which providers last refused us for payment and have not moved
// since, beside the meter level each read at that refusal. A provider reads dry while it is marked and its meter still
// reads that level.

/**
 The payment refusals of one budget layer.

 @example
 ```ts
 const { paidOut, markPaid, } = createPaymentMarks();
 ```
 */
export type PaymentMarks = {
  /**
   Whether a payment refusal still holds a provider dry: its meter has not moved since. A meter that has moved clears
   the mark, whichever way it moved, since the refusal was about the balance it read then.

   DOWNWARD IS A CLEARING MOVE, NOT A MISTAKE (checked for the ledger's P13, 2026-09-28). Of 230 payment refusals in the
   run logs, 198 came from OpenRouter while its meter read wet, at 0.01 to 1.94 USD, and they refuse what the balance
   leaves once the calls in flight have reserved their share: 8 bodies name `in_flight_budget_exhausted` ("Retry after
   in-flight requests settle"), and the rest ask for up to 131,072 tokens and are told 1,844 to 84,651 are affordable. A
   settling call lowers the balance by what it cost and frees the larger reservation it held, so holding the mark until
   the balance rises would keep OpenRouter dry for the rest of a run that can still afford smaller calls. The cost
   measured the same day: 28 refusals came back after a downward move, each a refused call that bought nothing.

   @param provider - provider to check

   @param levels - what every meter reads now

   @returns Whether the provider reads dry for payment
   */
  readonly paidOut: (
    args: {
      readonly provider: ProviderName;
      readonly levels: ProviderRecord<string>;
    },
  ) => boolean;

  /**
   Marks a provider as refused for payment at the level its meter reads now.

   @param provider - provider that was refused

   @param level - what its meter reads now
   */
  readonly markPaid: (
    args: {
      readonly provider: ProviderName;
      readonly level: string;
    },
  ) => void;
};

/**
 Builds the payment marks of one budget layer.

 @returns Marks that hold no provider dry until a refusal marks one

 @example
 ```ts
 const { paidOut, markPaid, } = createPaymentMarks();
 ```
 */
export function createPaymentMarks(): PaymentMarks {
  /**
   Which providers last refused us for payment and have not moved since, beside the meter level each read at that
   refusal.
   */
  const paidOutAt: Record<ProviderName, PaymentMark> = providerRecord({ of: unmarkedPayment, },);

  return {
    paidOut: function paidOut(
      {
        provider,
        levels,
      },
    ): boolean {
      /**
       The mark and the level at the refusal.
       */
      const refused = paidOutAt[provider];
      if (!refused.marked)
        return false;
      if (refused.level === levels[provider])
        return true;
      refused.marked = false;
      return false;
    },

    markPaid: function markPaid(
      {
        provider,
        level,
      },
    ): void {
      paidOutAt[provider] = {
        marked: true,
        level,
      };
    },
  };
}

//endregion Provider budget payment marks
