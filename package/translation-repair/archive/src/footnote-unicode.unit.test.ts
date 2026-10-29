import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import { scanGfmReferenceLiterals, } from '@monochromatic-dev/translation-repair-document-structure';
import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract';
import { applyFootnoteRelabel, } from '@monochromatic-dev/translation-repair-page-pass';
import { documentLabels, } from '../dist/final/node/index.mjs';

await describe({
  name: 'footnote tokenizer Unicode boundaries',
  children: [
    ...[
      { name: '499 astral scalars', label: '🐱'.repeat(499), valid: true },
      { name: '499 astral scalars and ASCII at 999 code units', label: `${'🐱'.repeat(499)}a`, valid: true },
      { name: '500 astral scalars at 1000 code units', label: '🐱'.repeat(500), valid: false },
      { name: '999 astral scalars', label: '🐱'.repeat(999), valid: false },
      { name: '1000 astral scalars', label: '🐱'.repeat(1_000), valid: false },
      { name: 'astral scalars and escaped bracket at 999 code units', label: `${'🐱'.repeat(498)}\\]a`, valid: true },
    ].map(fixture => it({
      name: `agrees with the actual parser for ${fixture.name}`,
      fn: async () => {
        const text = `Real[^${fixture.label}].\n\n[^${fixture.label}]: Note.`;
        const parsed = parseDocument({ text });
        expect(parsed.parseFindings.some(finding => finding.kind === 'mdx-downgraded')).toBe(false);
        expect(parsed.footnoteGraph.definitions).toHaveLength(fixture.valid ? 1 : 0);
        expect(parsed.footnoteGraph.references).toHaveLength(fixture.valid ? 1 : 0);
        expect(scanGfmReferenceLiterals({ slice: `[^${fixture.label}]` })).toHaveLength(fixture.valid ? 1 : 0);
        if (fixture.valid)
          expect(applyFootnoteRelabel({ text, map: [{ from: fixture.label, to: 'x' }] })).toBe('Real[^x].\n\n[^x]: Note.');
      },
    })),
    it({
      name: 'keeps NFC and NFD identifiers distinct instead of inventing canonical equivalence',
      fn: async () => {
        const text = 'NFC[^é] NFD[^e\u0301].\n\n[^é]: One.\n\n[^e\u0301]: Two.';
        expect(documentLabels({ text })).toEqual(['é', 'e\u0301']);
        expect(applyFootnoteRelabel({ text, map: [{ from: 'é', to: 'x' }] })).toBe(
          'NFC[^x] NFD[^e\u0301].\n\n[^x]: One.\n\n[^e\u0301]: Two.');
      },
    }),
  ],
});
