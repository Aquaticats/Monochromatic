import { caught as caughtBy, DEFAULT_CONCURRENCY, describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import {
  FootnoteRewriteError,
  requireFootnoteRewriteRefusal,
} from '@monochromatic-dev/translation-repair-document-structure';
import { applyFootnoteRelabel, } from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'footnote operational rewrite boundaries',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        ...[
          { name: 'a changing map on a marker-free document', text: 'No markers.', map: [{ from: '1', to: '2' }], kind: 'missing-source' },
          { name: 'a missing changing identifier', text: 'Real[^1].\n\n[^1]: Note.', map: [{ from: 'missing', to: 'x' }], kind: 'missing-source' },
          { name: 'a stale map with both present and missing domains', text: 'Real[^1].\n\n[^1]: Note.',
            map: [{ from: '1', to: 'x' }, { from: 'missing', to: 'y' }], kind: 'missing-source' },
          { name: 'invalid label syntax before the post-rewrite graph guard', text: 'Real[^1].\n\n[^1]: Note.', map: [{ from: '1', to: 'x]outside' }], kind: 'label' },
          { name: 'conflicting normalized source mappings', text: 'Real[^Note].\n\n[^Note]: Note.',
            map: [{ from: 'NOTE', to: 'x' }, { from: 'note', to: 'y' }], kind: 'mapping' },
          { name: 'an empty logical source identifier', text: 'Real[^1].\n\n[^1]: Note.', map: [{ from: '', to: '2' }], kind: 'mapping' },
          { name: 'an unmoved occupied destination', text: 'A[^1] B[^2].\n\n[^1]: A.\n\n[^2]: B.', map: [{ from: '1', to: '2' }], kind: 'collision' },
          { name: 'malformed document syntax', text: '<Component value={broken>\n\nReal[^1].\n\n[^1]: Note.', map: [{ from: '1', to: '2' }], kind: 'syntax' },
          { name: 'a new JSX element replacing an unresolved reference', text: 'Missing[^1].', map: [{ from: '1', to: '<br/>' }], kind: 'graph' },
          { name: 'raw null bytes disagreeing with parser-normalized identifiers', text: 'Real[^a\0b].\n\n[^a\0b]: Note.', map: [{ from: 'a\0b', to: 'x' }], kind: 'position' },
          // micromark forms this call from an image label, whose label may hold
          // the space the raw marker grammar refuses (ledger T8).
          { name: 'a footnote call the parser forms from an image label holding a space, where the raw opening reads no marker',
            text: 'Cat![^ab ].\n\n[^ab]: Note.', map: [{ from: 'ab', to: 'cd' }], kind: 'position' },
        ].map(fixture => it({
          name: `refuses ${fixture.name}`,
          fn: async () => {
            let caught: unknown;
            try {
              applyFootnoteRelabel({ text: fixture.text, map: fixture.map });
            }
            catch (error) {
              caught = error;
            }
            expect(caught).toBeInstanceOf(FootnoteRewriteError);
            if (!(caught instanceof FootnoteRewriteError)) throw new Error('expected named footnote failure');
            expect(caught.kind).toBe(fixture.kind);
          },
        })),
        it({
          name: 'keeps equivalent identity spellings and duplicate equivalent mappings byte-stable',
          fn: async () => {
            const text = 'Real[^Note].\n\n[^NOTE]: Note.';
            expect(applyFootnoteRelabel({ text, map: [{ from: 'note', to: 'NOTE' }] })).toBe(text);
            expect(applyFootnoteRelabel({ text, map: [{ from: 'note', to: 'X' }, { from: 'NOTE', to: 'x' }] })).toBe('Real[^X].\n\n[^X]: Note.');
            expect(applyFootnoteRelabel({ text: '<unclosed', map: [] })).toBe('<unclosed');
          },
        }),
        it({
          name: 'matches Unicode-folded identifiers without using normalized lengths for edits',
          fn: async () => {
            expect(applyFootnoteRelabel({ text: 'Real[^İ].\n\n[^İ]: Note.', map: [{ from: 'i\u0307', to: 'x' }] })).toBe('Real[^x].\n\n[^x]: Note.');
            expect(applyFootnoteRelabel({ text: 'Real[^a\\[b].\n\n[^a\\[b]: Note.', map: [{ from: String.raw`a\[b`, to: 'x' }] })).toBe('Real[^x].\n\n[^x]: Note.');
          },
        }),
        it({
          name: 'accepts the tokenizer label-length boundary and refuses the next code unit',
          fn: async () => {
            const label = 'n'.repeat(999);
            const text = 'Real[^1].\n\n[^1]: Note.';
            expect(applyFootnoteRelabel({ text, map: [{ from: '1', to: label }] })).toBe(`Real[^${label}].\n\n[^${label}]: Note.`);
            expect(() => applyFootnoteRelabel({ text, map: [{ from: '1', to: `${label}n` }] })).toThrow();
          },
        }),
        it({
          name: 'keeps only fixed operation wording in diagnostics while retaining a cause separately',
          fn: async () => {
            const cause = new Error('private fixture content');
            const error = new FootnoteRewriteError({ kind: 'syntax', cause });
            expect(error.cause).toBe(cause);
            expect(error.message).not.toContain(cause.message);
            expect(error.messageNamesOnly).toBe(true);
            expect(error.name).toBe('FootnoteRewriteError');
            expect(new FootnoteRewriteError({ kind: 'label' }).message).toContain('label');
          },
        }),
      ],
    }),

    describe({
      name: requireFootnoteRewriteRefusal.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RETURNS the rewrite refusal a catch around a footnote rewrite holds, the same object',
          fn: async () => {
            const refusal = new FootnoteRewriteError({ kind: 'collision' });
            expect(requireFootnoteRewriteRefusal({ error: refusal })).toBe(refusal);
          },
        }),
        it({
          name: 'RETHROWS anything else unchanged, an error or not, since an unexpected state must keep propagating',
          fn: async () => {
            const stray = new TypeError('the cat knocked the footnotes off the table');
            expect(caughtBy(function narrowStray(): void {
              requireFootnoteRewriteRefusal({ error: stray });
            })).toBe(stray);
            expect(caughtBy(function narrowString(): void {
              requireFootnoteRewriteRefusal({ error: 'hairball' });
            })).toBe('hairball');
          },
        }),
      ],
    }),
  ],
},);
