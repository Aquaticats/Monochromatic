/**
 Tests for the check that says a slice's surplus translation may have been read
 out of a PICTURE rather than moved from its neighbour.

 WHAT THESE PIN is that the signal is the SAME component on both sides. Media
 in the translation alone is content the original lacks; media in the original
 alone is something the translation dropped. Only a shared component, with more
 prose beside it on one side, says the prose came out of the image.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { sharesMedia, } from '../../dist/final/node/index.mjs';

await describe({
  name: sharesMedia.name,
  children: [
    it({
      name: 'FLAGS a slice embedding the same component on both sides with a transcription beside '
        + 'it, which is the shape both hand-verified transcriptions take',
      fn: async () => {
        expect(sharesMedia({
          sourceText: '这是她留下的字条。\n\n<PhotoScroll photos={[\n  "note.webp",\n]} />\n',
          targetText: 'This is the note she left:\n\n<PhotoScroll photos={[\n  "note.webp",\n]} />\n\n'
            + 'Transcription of the note above:\n\n> Feed the cat at seven.\n',
        },),).toBe(true,);
      },
    },),
    it({
      name: 'does NOT flag a component present only in the translation, which is content the '
        + 'original does not carry rather than a reading of an image both sides show',
      fn: async () => {
        expect(sharesMedia({
          sourceText: '小猫在窗台上睡觉。\n',
          targetText: 'The cat sleeps on the windowsill.\n\n<PhotoScroll photos={[\n  "cat.webp",\n]} />\n',
        },),).toBe(false,);
      },
    },),
    it({
      name: 'does NOT flag a component present only in the original, which is something the '
        + 'translation dropped',
      fn: async () => {
        expect(sharesMedia({
          sourceText: '小猫在窗台上睡觉。\n\n![猫](cat.webp)\n',
          targetText: 'The cat sleeps on the windowsill.\n',
        },),).toBe(false,);
      },
    },),
    it({
      name: 'does NOT flag two slices of ordinary prose, which is the null it has to produce on '
        + 'every real relocation: neither verified case embeds media at all',
      fn: async () => {
        expect(sharesMedia({
          sourceText: '小猫在窗台上睡觉。她看着外面的鸟。\n',
          targetText: 'The cat sleeps on the windowsill each morning, watching the birds outside.\n',
        },),).toBe(false,);
      },
    },),
  ],
},);
