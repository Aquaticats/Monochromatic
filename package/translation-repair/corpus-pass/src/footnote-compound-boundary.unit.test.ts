import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import { reorderFootnoteDefinitions, } from '@monochromatic-dev/translation-repair-archive';
import {
  parseMarkdownBody,
  parseMdxBody,
} from '@monochromatic-dev/translation-repair-document-structure';
import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract';
import { applyFootnoteRelabel, } from '@monochromatic-dev/translation-repair-page-pass';
import { relabelArchiveFootnotes, } from '../dist/final/node/index.mjs';

await describe({
  name: 'compound footnote syntax preservation',
  children: [
    it({
      name: 'treats indentation as prose under actual MDX grammar while plain Markdown recognizes code',
      fn: async () => {
        const body = '    Indented[^1].\n\n[^1]: Note.';
        expect(parseMdxBody({ body }).children[0]?.type).toBe('paragraph');
        expect(parseMarkdownBody({ body }).children[0]?.type).toBe('code');
        expect(applyFootnoteRelabel({ text: body, map: [{ from: '1', to: '2' }] })).toBe('    Indented[^2].\n\n[^2]: Note.');
      },
    }),
    it({
      name: 'distinguishes ESM, expressions and JSX attributes from JSX children and definition references',
      fn: async () => {
        const literalPrefix = 'export const sample = "[^1]";\n\n{"[^1]"}\n\n';
        const text = `${literalPrefix}<span title={"[^1]"}>Child[^1]</span>\n\nReal[^2].\n\n[^1]: First note.\n\n[^2]: See[^1] and \`[^1]\`.`;
        const expected = `${literalPrefix}<span title={"[^1]"}>Child[^11]</span>\n\nReal[^22].\n\n[^11]: First note.\n\n[^22]: See[^11] and \`[^1]\`.`;
        expect(applyFootnoteRelabel({ text, map: [{ from: '1', to: '11' }, { from: '2', to: '22' }] })).toBe(expected);
        expect(parseDocument({ text: expected }).footnoteGraph.references).toHaveLength(3);
        expect(parseDocument({ text: expected }).footnoteGraph.findings).toEqual([]);
      },
    }),
    it({
      name: 'preserves bare-CR separators through both renaming and reordering',
      fn: async () => {
        const text = 'First[^b] and [^a].\r\r[^b]: Second.\r\r[^a]: First.\r';
        const renamed = applyFootnoteRelabel({ text, map: [{ from: 'b', to: '2' }, { from: 'a', to: '1' }] });
        expect(reorderFootnoteDefinitions({ text: renamed, order: ['1', '2'] }).text).toBe(
          'First[^2] and [^1].\r\r[^1]: First.\r\r[^2]: Second.\r');
      },
    }),
    it({
      name: 'moves complete multiline definition bodies without reformatting their continuation paragraphs',
      fn: async () => {
        const first = '[^a]: First introduction.\n    Continued first.';
        const second = '[^b]: Second introduction.\n    Continued second.\n\n    Second paragraph.';
        const text = `Body[^b] and [^a].\n\n${second}\n\n${first}`;
        expect(reorderFootnoteDefinitions({ text, order: ['a', 'b'] }).text).toBe(`Body[^b] and [^a].\n\n${first}\n\n${second}`);
      },
    }),
    it({
      name: 'keeps separate definition runs in place rather than moving across intervening prose',
      fn: async () => {
        const text = 'Body[^b] and [^a].\n\n[^b]: Second.\n\nIntervening prose.\n\n[^a]: First.';
        const result = reorderFootnoteDefinitions({ text, order: ['a', 'b'] });
        expect(result.text).toBe(text);
        expect(result.changed).toBe(false);
        expect(result.note).toContain('interleaved');
      },
    }),
    it({
      name: 'retains multiple unmatched notes in relative order through the complete archive operation',
      fn: async () => {
        const archiveText = 'Extra[^1] and extra[^2]; first[^a] and second[^b].\n\n[^1]: Extra first.\n\n[^2]: Extra second.\n\n[^a]: Main first.\n\n[^b]: Main second.';
        const result = relabelArchiveFootnotes({ entryId: 'invented-compound', slices: [],
          definitionPairs: [{ sourceLabel: '1', targetLabel: 'a' }, { sourceLabel: '2', targetLabel: 'b' }],
          sourceText: 'First[^1] and second[^2].\n\n[^1]: Source first.\n\n[^2]: Source second.', archiveText,
          l: tagged({ tag: 'compound-footnote-test' }) });
        expect(result.archiveText).toBe('Extra[^3] and extra[^4]; first[^1] and second[^2].\n\n[^1]: Main first.\n\n[^2]: Main second.\n\n[^3]: Extra first.\n\n[^4]: Extra second.');
        expect(result.changed).toBe(true);
        expect(result.withheld).toBeUndefined();
        const graph = parseDocument({ text: result.archiveText }).footnoteGraph;
        expect(graph.references).toHaveLength(4);
        expect(graph.definitions).toHaveLength(4);
        expect(graph.findings).toEqual([]);
      },
    }),
  ],
});
