import {
  alternateModelsFor,
  type LeftOutModel,
  readsImages,
  type ReaskClaims,
  type ReaskMode,
  type ReaskSeat,
  takenFor,
} from '@monochromatic-dev/translation-repair-provider/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Other readers of a cut picture
// THE MODELS A CUT PICTURE READING MAY BE RE-ASKED OF (ledger B1074, B1100), apart from `image-reading-past-refusal.ts`,
// which asks them in turn and passed its line bound with the transport failure that moves the order on (B1201).

/**
 The models a cut picture reading may be re-asked of, in order, with the seat they are drawn for and every model left
 out with why.

 @param modelId - reader whose reply was cut

 @param benchModelIds - every reader of this picture

 @param reask - whose answer the re-ask may give, which the seat carries

 @param claims - record of the models asked for the readers of this picture; absent for a reader alone on its picture

 @returns Models that read pictures in the order of their runaway shares, the seat the asks are claimed on, and the
 models left out: those the reader role or the bench keeps out, those that read no picture and those asked already

 @example
 ```ts
 const others = otherReadersOf({ modelId, benchModelIds, reask: 'other-model', },);
 ```
 */
export function otherReadersOf(
  {
    modelId,
    benchModelIds,
    reask,
    claims,
  }: {
    readonly modelId: RosterModelId;
    readonly benchModelIds: readonly RosterModelId[];
    readonly reask: ReaskMode;
    readonly claims?: ReaskClaims;
  },
): {
  readonly readers: readonly RosterModelId[];
  readonly seat: ReaskSeat;
  readonly leftOut: readonly LeftOutModel[];
} {
  /**
   The seat of this reader: the reader role, the readers of the picture and the record they share.
   */
  const seat: ReaskSeat = {
    role: 'reader',
    bench: benchModelIds,
    reask,
    // Conditional spread keeps the record absent for a reader that is alone on its picture.
    ...((claims === undefined) ? {} : { claims, }),
  };

  /**
   Models to ask in the order of their runaway shares, and the ones the reader role, the bench or an ask for another
   reader leaves out.
   */
  const {
    alternates,
    leftOut,
  } = alternateModelsFor({
    modelId,
    role: seat.role,
    bench: seat.bench,
    askedForOtherSeats: takenFor({ seat, },),
  },);

  /**
   The models that read pictures among them, in the same order.
   */
  const readers = alternates.filter(function readsPictures(alternate,): boolean {
    return readsImages({ modelId: alternate, },);
  },);
  return {
    readers,
    seat,
    leftOut: [
      ...leftOut,
      ...alternates
        .filter(function readsNoPicture(alternate,): boolean {
          return !readers.includes(alternate,);
        },)
        .map(function withReason(alternate,): LeftOutModel {
          return {
            modelId: alternate,
            why: 'reads no picture',
          };
        },),
    ],
  };
}

//endregion Other readers of a cut picture
