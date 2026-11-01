import type { ChunkPair, } from './chunk-document.ts';
import type { PairedReading, } from './image-reading-pair.ts';
import { mostCarriedReading, } from './most-carried-reading.ts';
import { slicePictures, } from './slice-pictures.ts';

//region Pairing pictures
// WHAT THE PAIRING SHEET IS SHOWN ABOUT A SECTION'S PICTURES (class
// thirty-four, 2026-09-16).
//
// ONE TRANSCRIPT PER PICTURE, the one the other readers carry most, the same
// choice the archive block review made on 2026-09-16: the pairing judge needs
// to recognise that an English block renders a picture, not to weigh six
// readings of it, and a sheet already carrying two whole sections is not the
// place to add five more.
//
// RENDERED BY `slicePictures` over the section as a one-slice window, so the
// PICTURE headings and the side legend read exactly as every other sheet has
// them.

/**
 Transcripts of the pictures one aligned section shows, for its pairing sheet.

 @param pair - aligned section whose original side names the pictures

 @param pictureReadings - what reading produced per asset name for this entry

 @returns Rendered picture context, empty when the section shows no picture
 anybody read

 @example
 ```ts
 const pictureContext = pairingPictureContext({ pair, pictureReadings, },);
 ```
 */
export function pairingPictureContext(
  {
    pair,
    pictureReadings,
  }: {
    readonly pair: ChunkPair;
    readonly pictureReadings: ReadonlyMap<string, PairedReading>;
  },
): string {
  /**
   Readings reduced to the one transcript per corroborated picture.
   */
  const reduced = new Map<string, PairedReading>();
  for (const [assetName, reading,] of pictureReadings) {
    if (reading.kind !== 'corroborated') {
      reduced.set(
        assetName,
        reading,
      );
      continue;
    }
    reduced.set(
      assetName,
      {
        ...reading,
        readings: [mostCarriedReading({ readings: reading.readings, },),],
      },
    );
  }

  /**
   Section rendered as a one-slice window.
   */
  const rendered = slicePictures({
    slices: [pair,],
    slicePosition: 0,
    readings: reduced,
  },);
  return rendered.context;
}

//endregion Pairing pictures
