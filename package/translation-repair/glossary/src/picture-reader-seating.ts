import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Picture reader seating
// THE READERS A PER-PICTURE HOOK HANDS THE PICTURE READINGS (ledger X12), as
// the lanes', the lane contest's and the consolidation's hooks hand their
// slices a roster. The readings ask one bench, so the seating is that bench.

/**
 What a per-picture hook hands the picture readings.

 @example
 ```ts
 const seating: PictureReaderSeating = { readerModelIds: seats.readers, };
 ```
 */
export type PictureReaderSeating = {
  /**
   Readers read since the picture readings started, absent while the given
   ones stand.
   */
  readonly readerModelIds?: readonly RosterModelId[];
};

//endregion Picture reader seating
