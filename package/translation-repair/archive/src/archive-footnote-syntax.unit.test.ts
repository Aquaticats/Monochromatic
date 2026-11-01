import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract';
import { applyFootnoteRelabel, } from '@monochromatic-dev/translation-repair-page-pass';
import { documentLabels, } from '../dist/final/node/index.mjs';

const syntaxLines = [
  '---', 'name: "Literal [^1]"', '---', '', 'Real[^1].', '', '`[^1]`', '', String.raw`\[^1]`, '',
  '```md', '[^1]', '```', '', '<!-- [^1] -->', '', '[ordinary](https://example.test/x[^1])', '',
  '<span title="[^1]">Literal text</span>', '', '[^1]: Actual note.',
];

await describe({
  name: 'footnote rewrite syntax boundaries',
  children: [
    it({
      name: 'rewrites real markers without touching metadata, code, comments, escapes, URLs or JSX attributes',
      fn: async () => {
        const text = syntaxLines.join('\n');
        const expected = syntaxLines.map(line => line === 'Real[^1].' ? 'Real[^2].' : line === '[^1]: Actual note.' ? '[^2]: Actual note.' : line).join('\n');
        expect(applyFootnoteRelabel({ text, map: [{ from: '1', to: '2' }] })).toBe(expected);
      },
    }),
    it({
      name: 'does not reserve inactive strings as footnote namespace members',
      fn: async () => {
        const text = ['---', 'name: "[^meta]"', '---', '', 'Real[^1].', '', '`[^inline]`', '', String.raw`\[^escaped]`, '',
          '```md', '[^fence]', '```', '', '<!-- [^comment] -->', '', '[ordinary](https://example.test/x[^url])', '',
          '<span title="[^attribute]">Literal text</span>', '', '[^1]: Actual note.'].join('\n');
        expect(documentLabels({ text })).toEqual(['1']);
      },
    }),
    it({
      name: 'matches case-equivalent markers while retaining the first encountered spelling in inventories',
      fn: async () => {
        const text = 'Real[^Note] and [^note].\n\n[^NOTE]: Actual note.';
        expect(applyFootnoteRelabel({ text, map: [{ from: 'NOTE', to: 'x' }] })).toBe('Real[^x] and [^x].\n\n[^x]: Actual note.');
        expect(documentLabels({ text })).toEqual(['Note']);
      },
    }),
    it({
      name: 'uses exact raw label spans for escaped closing brackets',
      fn: async () => {
        const text = 'Real[^a\\]b].\n\n[^a\\]b]: Actual note.';
        expect(documentLabels({ text })).toEqual([String.raw`a\]b`]);
        expect(applyFootnoteRelabel({ text, map: [{ from: String.raw`a\]b`, to: String.raw`x\]y` }] })).toBe('Real[^x\\]y].\n\n[^x\\]y]: Actual note.');
      },
    }),
    it({
      name: 'distinguishes an escaped opening from a marker after an escaped backslash',
      fn: async () => {
        const text = 'Literal \\[^1], active \\\\[^1].\n\n[^1]: Actual note.';
        expect(applyFootnoteRelabel({ text, map: [{ from: '1', to: '2' }] })).toBe('Literal \\[^1], active \\\\[^2].\n\n[^2]: Actual note.');
        const graph = parseDocument({ text }).footnoteGraph;
        expect(graph.references).toHaveLength(1);
      },
    }),
    it({
      name: 'keeps unresolved unescaped references visible while ignoring escaped lookalikes',
      fn: async () => {
        const text = String.raw`Missing[^9], literal \[^8].`;
        expect(documentLabels({ text })).toEqual(['9']);
        expect(parseDocument({ text }).footnoteGraph.references.map(reference => reference.identifier)).toEqual(['9']);
      },
    }),
    ...['', 'two words', 'x]outside', 'x\n[^injected]: text', 'x[other'].map(label => it({
      name: `refuses destination label ${JSON.stringify(label)} before emitting invalid markup`,
      fn: async () => {
        expect(() => applyFootnoteRelabel({ text: 'Real[^1].\n\n[^1]: Actual note.', map: [{ from: '1', to: label }] })).toThrow();
      },
    })),
    it({
      name: 'refuses a composed rewrite that would merge definitions',
      fn: async () => {
        expect(() => applyFootnoteRelabel({ text: 'A[^1] B[^2].\n\n[^1]: A.\n\n[^2]: B.',
          map: [{ from: '1', to: 'x' }, { from: '2', to: 'x' }] })).toThrow();
      },
    }),
  ],
});
