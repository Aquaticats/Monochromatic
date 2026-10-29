/**
 Guarded poke delivery.

 @module
 */

import type { ExtensionAPI, } from '@earendil-works/pi-coding-agent';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { POKE_CUSTOM_TYPE, } from './constants.ts';
import { bashPokeLogger, } from './logger.ts';
import type { PokeDetails, } from './poke-text.ts';

//region Types

/**
 Outcome of one poke delivery attempt.
 
 Failure is reported rather than thrown because Pi invalidates an extension
 runtime on reload and on session replacement, so a job outliving either one
 cannot deliver. An escaping throw from a completion callback would surface as
 an unhandled rejection instead of a readable log line.
 */
type PokeDelivery =
  | {
    readonly delivered: true;
  }
    | {
      readonly delivered: false;
      readonly error: unknown;
    };

//endregion Types

//region Delivery

/**
 Sends one finished job's poke as a custom message that triggers a turn.
 
 A custom message rather than a user message keeps the transcript honest: Pi
 renders it through this package's renderer instead of showing text that looks
 typed. Turn scheduling is the same, immediate when idle and queued behind
 current work otherwise.
 
 @param pi - Pi extension API owning message delivery
 
 @param content - model-facing poke text
 
 @param details - renderer-only outcome data, excluded from model context
 
 @returns delivery state, or the captured failure for caller-owned reporting
 
 @example
 ```ts
 deliverPoke({ pi, content: '[bash finished]', details, },);
 ```
 */
function deliverPoke(
  {
    pi,
    content,
    details,
  }: {
    readonly pi: ForeignBorrowed<ExtensionAPI>;
    readonly content: string;
    readonly details: PokeDetails;
  },
): PokeDelivery {
  /**
   Function-scoped logger naming this boundary in every record.
   */
  const l = tagged({
    tag: deliverPoke.name,
    l: bashPokeLogger,
  }, );
  try {
    pi.sendMessage(
      {
        customType: POKE_CUSTOM_TYPE,
        content,
        display: true,
        details,
      },
      {
        triggerTurn: true,
        deliverAs: 'followUp',
      },
    );
    l.debug(`delivered poke for: ${details.command}`, );
    return { delivered: true, };
  }
  catch (error: unknown) {
    l.warn(`poke delivery failed for ${details.command}: ${caughtValueText(error, )}`, );
    return {
      delivered: false,
      error,
    };
  }
}

//endregion Delivery

export { deliverPoke, };

export type { PokeDelivery, };
