/**
 Tests what the pairing sheet is shown about the pictures of one aligned
 section: one transcript per corroborated picture, the one the other readers
 carry most, and nothing for a picture no reading may be used for. Fixtures are
 cat-themed invention; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type PairedReading,
  pairingPictureContext,
} from '../dist/final/node/index.mjs';
import { sliceOf, } from './content-slice-of.test-fixture.ts';
import {
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Placeholder corpus pages write for an entry's own directory, escaped so the
 characters are the ones a page carries.
 */
const ENTRY = `\${path}`;

/**
 What the shorter reader transcribed from the sunbeam picture.
 */
const SUNBEAM_SHORT = 'A tabby cat dozes in a sunbeam.';

/**
 What the longer reader transcribed from the same picture, carrying more than
 the shorter reading without contradicting it.
 */
const SUNBEAM_LONG = 'A tabby cat lies curled and dozing in a warm patch of sun by the window.';

/**
 One section naming two pictures, the sunbeam one and the nap one.
 */
const SECTION = sliceOf({
  text: `Tabby dozes in a sunbeam.\n\n<PhotoScroll photos={[ '${ENTRY}/photos/sunbeam.webp', '${ENTRY}/photos/nap.webp' ]} />\n`,
  sliceIndex: 0,
},);

await describe({
  name: pairingPictureContext.name,
  children: [
    it({
      name: 'SHOWS ONE TRANSCRIPT OF A CORROBORATED PICTURE, the longer of two readings that carry each other, '
        + 'and SHOWS NOTHING OF A PICTURE THAT CARRIES NO TEXT',
      fn: async () => {
        /**
         What reading produced for each picture of the section.
         */
        const pictureReadings = new Map<string, PairedReading>([
          [
            'sunbeam.webp',
            {
              kind: 'corroborated',
              readings: [
                {
                  modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
                  text: SUNBEAM_SHORT,
                },
                {
                  modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
                  text: SUNBEAM_LONG,
                },
              ],
              overlap: 0.82,
            },
          ],
          [
            'nap.webp',
            {
              kind: 'no-text',
              characters: 0,
            },
          ],
        ],);

        expect(pairingPictureContext({
          pair: SECTION,
          pictureReadings,
        },),).toBe(`PICTURE sunbeam.webp\n${SEAT_SYNTHETIC_VISION_NO_OPENROUTER}:\n${SUNBEAM_LONG}`,);
      },
    },),
    it({
      name: 'SHOWS NOTHING WHEN NO PICTURE OF THE SECTION CARRIES A READING THE SHEET MAY SHOW, so a section '
        + 'whose pictures all carry no text reads as showing no picture to the pairing judge',
      fn: async () => {
        expect(pairingPictureContext({
          pair: SECTION,
          pictureReadings: new Map<string, PairedReading>([
            [
              'sunbeam.webp',
              {
                kind: 'no-text',
                characters: 0,
                confirmedBy: [SEAT_HYPER_VISION,],
              },
            ],
          ],),
        },),).toBe('',);
      },
    },),
  ],
},);
