import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import { FootnoteRewriteError, } from '@monochromatic-dev/translation-repair-document-structure';
import { footnoteRelabelOf, } from '@monochromatic-dev/translation-repair-corpus-source';
import { prepareDocumentPair, } from '../dist/final/node/index.mjs';

function readFootnoteOutcome(input: Parameters<typeof footnoteRelabelOf>[0]): unknown {
  try {
    return footnoteRelabelOf(input);
  }
  catch (error) {
    return error;
  }
}

function expectScopeRefusal(input: Parameters<typeof footnoteRelabelOf>[0]): void {
  const caught = readFootnoteOutcome(input);
  expect(caught).toBeInstanceOf(FootnoteRewriteError);
  if (!(caught instanceof FootnoteRewriteError)) throw new Error('expected footnote scope refusal');
  expect(caught.kind).toBe('slice-scope');
}

await describe({
  name: 'footnote evidence projected from prepared documents',
  children: [
    ...['sourceText', 'targetText'].map(side => it({
      name: `refuses same-length stale ${side} without relying on marker overlap`,
      fn: async () => {
        const prepared = prepareDocumentPair({ sourceText: 'Source[^1].\n\n[^1]: Note.', targetText: 'Target[^2].\n\n[^2]: Note.' });
        const stale = { ...prepared, [side]: prepared[side as 'sourceText' | 'targetText'].toLowerCase() };
        expectScopeRefusal(stale);
      },
    })),
    it({
      name: 'refuses fractional offsets even when string slicing would round them to matching text',
      fn: async () => {
        const prepared = prepareDocumentPair({ sourceText: 'Source[^1].\n\n[^1]: Note.', targetText: 'Target[^2].\n\n[^2]: Note.' });
        expectScopeRefusal({ ...prepared, slices: prepared.slices.map(slice => ({ ...slice, source: { ...slice.source, startOffset: 0.5 } })) });
      },
    }),
    it({
      name: 'refuses a text-matching range that cuts through an active marker',
      fn: async () => {
        const prepared = prepareDocumentPair({ sourceText: 'Source[^1].\n\n[^1]: Note.', targetText: 'Target[^2].\n\n[^2]: Note.' });
        const endOffset = prepared.sourceText.indexOf(']');
        expectScopeRefusal({ ...prepared, slices: prepared.slices.map(slice => ({ ...slice,
          source: { ...slice.source, endOffset, text: prepared.sourceText.slice(slice.source.startOffset, endOffset) } })) });
      },
    }),
    it({
      name: 'reads a marker-bearing container half from its complete document instead of reparsing the slice',
      fn: async () => {
        const paragraphs = ['Reference[^1].', ...Array.from({ length: 39 }, (_, index) => `Paragraph ${index}.`)];
        const sourceText = `<details>\n\n${paragraphs.join('\n\n')}\n\n</details>\n\n[^1]: Source note.`;
        const targetText = sourceText.replaceAll('[^1]', '[^2]');
        const prepared = prepareDocumentPair({ sourceText, targetText, frontMatterAuthority: 'archive' });
        const markerSlice = prepared.slices.find(slice => slice.source.text.includes('Reference[^1]'));
        expect(prepared.slices.length).toBeGreaterThan(1);
        expect(markerSlice?.source.text.includes('<details>')).toBe(true);
        expect(markerSlice?.source.text.includes('</details>')).toBe(false);
        const reading = readFootnoteOutcome(prepared);
        expect(reading).toEqual({ kind: 'relabel', map: [{ from: '2', to: '1' }],
          correspondences: [{ from: '2', to: '1' }], skipped: [] });
      },
    }),
    ...['sourceText', 'targetText'].map(side => it({
      name: `refuses slice evidence from stale ${side} before interpreting label relations`,
      fn: async () => {
        const prepared = prepareDocumentPair({ sourceText: 'Source[^1].\n\n[^1]: Note.', targetText: 'Target[^2].\n\n[^2]: Note.' });
        const stale = { ...prepared, [side]: `Changed prefix.\n\n${prepared[side as 'sourceText' | 'targetText']}` };
        expectScopeRefusal(stale);
      },
    })),
  ],
});
