/**
 Tests for deterministic refusal-shape detection.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  detectRefusalShape,
  REFUSAL_SCAN_WINDOW,
} from '../dist/final/node/index.mjs';

await describe({
  name: detectRefusalShape.name,
  children: [
    it({
      name: 'flags an opening refusal and reports the marker',
      fn: async () => {
        /** Scan of an apologetic opening. */
        const scan = detectRefusalShape({
          text: "I'm sorry, but I can't help with reviewing this document.",
        },);
        expect(scan.refusalShaped,).toBe(true,);
        // "i can't help" precedes "i'm sorry, but" in the marker list,
        // and first marker in list order wins.
        expect(
          scan.refusalShaped
            ? scan.marker
            : '',
        ).toBe("i can't help",);
      },
    },),

    it({
      name: 'flags refusals regardless of letter case',
      fn: async () => {
        expect(detectRefusalShape({ text: 'AS AN AI, I MUST DECLINE.', },).refusalShaped,)
          .toBe(true,);
      },
    },),

    it({
      name: 'flags Chinese refusal phrasing',
      fn: async () => {
        expect(detectRefusalShape({ text: '很抱歉，我无法协助完成这个请求。', },).refusalShaped,)
          .toBe(true,);
      },
    },),

    it({
      name: 'passes ordinary JSON content untouched',
      fn: async () => {
        expect(detectRefusalShape({ text: '{"issues":[],"verdict":"pass"}', },).refusalShaped,)
          .toBe(false,);
      },
    },),

    it({
      name: 'ignores markers past the opening window as quoted content',
      fn: async () => {
        /** Marker buried beyond the scanned opening. */
        const buried = `${'喵'.repeat(REFUSAL_SCAN_WINDOW,)} i cannot help`;
        expect(detectRefusalShape({ text: buried, },).refusalShaped,).toBe(false,);
      },
    },),

    it({
      name: 'READS A MARKER AS WORDS, never as letters inside longer ones (ledger B23): a cat that served as an '
        + 'aide is no reply written as an AI',
      fn: async () => {
        expect(detectRefusalShape({ text: 'The cat served as an aide to the shelter for years.', },).refusalShaped,)
          .toBe(false,);
      },
    },),

    it({
      name: 'FLAGS A REFUSAL WRITTEN WITH A TYPOGRAPHIC APOSTROPHE, as models often write one (ledger B23)',
      fn: async () => {
        expect(detectRefusalShape({ text: 'I\u{2019}m sorry, but I can\u{2019}t help with this.', },).refusalShaped,)
          .toBe(true,);
      },
    },),
  ],
},);
