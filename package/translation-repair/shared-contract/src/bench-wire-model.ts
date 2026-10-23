import { MODEL_CARDS, } from './model-cards.ts';
import type { RosterModelId, } from './roster-id.ts';

//region Bench wire model
// ONE WIRE MODEL PER BENCH (the owner's Claude key, 2026-10-22). The eight Claude cards share two wire models: a model
// and its three effort cards are one model asked at four settings. Seated together on one bench they are one model
// heard more than once, which counts as several voices toward a quorum and overstates agreement. The rule was written
// in `claude-card-sides.ts` and `roster-id.ts` and enforced nowhere; this is the enforcement, called where a run's
// benches are fixed (`run/src/corpus-run/run-config.ts`).

/**
 Names the model a roster id sends a request for: the wire model of a Claude card, so the card of a model and its
 effort cards agree, and the roster id itself for any other model, whose id already names one model.

 @param modelId - roster model

 @returns The identity two seats must not share on one bench

 @example
 ```ts
 wireModelOf({ modelId: 'claude-sonnet-5-5-effort-high', },); // 'claude-sonnet-5-5'
 ```
 */
export function wireModelOf({ modelId, }: { readonly modelId: RosterModelId; },): string {
  return MODEL_CARDS[modelId]
    .anthropic
    ?.wireModel
    ?? modelId;
}

/**
 Raised when one bench seats the same wire model more than once.

 A THROW AT LOAD, because it is a seating written wrong in our own code and not a thing a model did.

 @example
 ```ts
 throw new WireModelRepeatedError({ bench: 'editors', wireModel: 'claude-sonnet-5-5', modelIds: [], },);
 ```
 */
export class WireModelRepeatedError extends Error {
  /**
   Declares this message safe to forward: it names a bench and models.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds failure naming the bench, the wire model and the seats that share it.

   @param bench - name of the bench

   @param wireModel - model seated more than once

   @param modelIds - the roster ids on the bench that send it, in bench order

   @example
   ```ts
   new WireModelRepeatedError({ bench: 'editors', wireModel: 'claude-sonnet-5-5', modelIds: ['claude-sonnet-5-5', 'claude-sonnet-5-5-effort-high',], },);
   ```
   */
  public constructor(
    {
      bench,
      wireModel,
      modelIds,
    }: {
      readonly bench: string;
      readonly wireModel: string;
      readonly modelIds: readonly RosterModelId[];
    },
  ) {
    super(
      `The ${bench} bench seats ${wireModel} more than once (${modelIds.join(', ',)}): one model heard twice counts as `
        + 'two voices toward a quorum; seat one card of it',
    );
    this.name = 'WireModelRepeatedError';
  }
}

/**
 Refuses a bench that seats one wire model more than once.

 @param bench - name of the bench, for the failure

 @param modelIds - the bench's seats

 @throws {@link WireModelRepeatedError} naming the first wire model seated twice

 @example
 ```ts
 assertOneWireModelPerBench({ bench: 'editors', modelIds: ['claude-sonnet-5-5', 'glm-5.3',], },);
 ```
 */
export function assertOneWireModelPerBench(
  {
    bench,
    modelIds,
  }: {
    readonly bench: string;
    readonly modelIds: readonly RosterModelId[];
  },
): void {
  /**
   The seats that send each wire model, in bench order.
   */
  const seatsByWireModel = Map.groupBy(
    modelIds,
    function wireModelOfSeat(modelId,): string {
      return wireModelOf({ modelId, },);
    },
  );
  for (const [wireModel, seats,] of seatsByWireModel) {
    if (seats.length > 1)
      throw new WireModelRepeatedError({
        bench,
        wireModel,
        modelIds: seats,
      },);
  }
}

//endregion Bench wire model
