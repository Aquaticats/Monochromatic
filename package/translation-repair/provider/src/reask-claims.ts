import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { ReaskSeat, } from './chat-contract.ts';

//region Re-ask claims
// ONE MODEL GIVES AT MOST ONE VOICE PER QUESTION (ledger B1076, B1080; "one judge alone never decides", 2026-10-25;
// "Two judges must name it", 2026-10-25). The seats of one question that run away together are each re-asked of
// another model, and the plain question to a model is the digest of that model's own first ask, so a model asked for
// two seats answers both with one claimed reply: one model, two voices. The seats therefore share one record of the
// models already asked for the question, each takes a model from it BEFORE it asks, and a seat that finds the model
// taken goes on to the next. The take is one synchronous step (JavaScript runs it to the end before another seat
// runs), so it needs no lock.

/**
 The models already asked for seats of one question.

 One record per question, made by the gather (or the picture) that asks its seats and handed to every seat through
 `ReaskSeat.claims`; a seat of another question gets a record of its own.

 @example
 ```ts
 const claims: ReaskClaims = createReaskClaims();
 ```
 */
export type ReaskClaims = {
  /**
   Takes a model for the seat that asks, unless another seat of the question took it first; returns whether this call
   took it.
   */
  readonly take: (taking: { readonly modelId: RosterModelId; },) => boolean;

  /**
   Gives a model back, for a seat whose ask the router refused before any exchange (ledger B1201): the model was
   never asked, so it is no voice of the question and another seat may still take it. A model that was asked and
   failed stays taken.
   */
  readonly release: (releasing: { readonly modelId: RosterModelId; },) => void;

  /**
   Lists the models taken so far, in the order they were taken, as a copy.
   */
  readonly list: () => readonly RosterModelId[];
};

/**
 Makes the record of one question, empty.

 @returns Record whose `take` gives each model to the first seat that takes it

 @example
 ```ts
 const claims = createReaskClaims();
 if (claims.take({ modelId: 'glm-5.3', },)) await askIt();
 ```
 */
export function createReaskClaims(): ReaskClaims {
  /**
   Models taken so far, in the order they were taken.
   */
  const taken: RosterModelId[] = [];
  return Object.freeze({
    take: function take({ modelId, }: { readonly modelId: RosterModelId; },): boolean {
      if (taken.includes(modelId,))
        return false;
      taken.push(modelId,);
      return true;
    },
    release: function release({ modelId, }: { readonly modelId: RosterModelId; },): void {
      /**
       Place the model was taken at, absent when it was not taken.
       */
      const at = taken.indexOf(modelId,);
      if (at !== (-1)) {
        taken.splice(
          at,
          1,
        );
      }
    },
    list: function list(): readonly RosterModelId[] {
      return [...taken,];
    },
  },);
}

/**
 Models already asked for other seats of the seat's question, where the seat has a record.

 @param seat - seat that ran away

 @returns Models taken, or none where the seat is alone on its question

 @example
 ```ts
 const askedForOtherSeats = takenFor({ seat, },);
 ```
 */
export function takenFor({ seat, }: { readonly seat: ReaskSeat; },): readonly RosterModelId[] {
  /**
   Record the seat shares with the other seats of its question.
   */
  const { claims, } = seat;
  return (claims === undefined) ? [] : claims.list();
}

/**
 Takes a model for the seat that is about to ask it, where the seat has a record.

 @param seat - seat that ran away

 @param modelId - model the seat is about to ask

 @returns Whether the seat may ask it; always true for a seat that is alone on its question

 @example
 ```ts
 if (takeFor({ seat, modelId, },)) await ask({ modelId, },);
 ```
 */
export function takeFor(
  {
    seat,
    modelId,
  }: {
    readonly seat: ReaskSeat;
    readonly modelId: RosterModelId;
  },
): boolean {
  /**
   Record the seat shares with the other seats of its question.
   */
  const { claims, } = seat;
  return (claims === undefined) || claims.take({ modelId, },);
}

/**
 Gives back a model the seat took and the router then refused before any exchange, where the seat has a record.

 @param seat - seat that ran away

 @param modelId - model the seat took and was refused

 @example
 ```ts
 releaseFor({ seat, modelId, },);
 ```
 */
export function releaseFor(
  {
    seat,
    modelId,
  }: {
    readonly seat: ReaskSeat;
    readonly modelId: RosterModelId;
  },
): void {
  /**
   Record the seat shares with the other seats of its question.
   */
  const { claims, } = seat;
  claims?.release({ modelId, },);
}

//endregion Re-ask claims
