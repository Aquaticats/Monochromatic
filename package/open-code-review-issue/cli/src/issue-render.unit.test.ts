import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  capIssueTitle,
  renderIssue,
  renderIssueTitle,
  type NormalizedFinding,
} from '../dist/final/node/index.mjs';

await describe({
  name: renderIssue.name,
  children: [
    it({
      name: 'renders the complete deterministic Issue contract',
      fn: async () => {
        /**
         Finding carrying every rendered section and raw Markdown prose.
         */
        const finding: NormalizedFinding = {
          position: {
            kind: 'record',
            value: 1,
          },
          path: 'src/example.ts',
          content: '## Preserve this heading\n\nNotify @maintainer.',
          existingCode: 'return value + 1;\n',
          suggestionCode: 'return value;\n',
          startLine: 4,
          endLine: 5,
          category: 'bug',
          severity: 'high',
        };

        /**
         Fully rendered Issue draft without destination label.
         */
        const issue = renderIssue({
          finding,
          needsTriageLabel: false,
        },);

        expect(issue,).toStrictEqual({
          position: {
            kind: 'record',
            value: 1,
          },
          security: false,
          title: '[needs-triage] [bug] src/example.ts: ## Preserve this heading',
          body: [
            '## Finding',
            '',
            '## Preserve this heading',
            '',
            'Notify @maintainer.',
            '',
            '## Source',
            '',
            '- Location: src/example.ts:4-5',
            '- Category: `bug`',
            '- Severity: `high`',
            '',
            '## Existing code',
            '',
            '    return value + 1;',
            '    ',
            '',
            '## Suggested code',
            '',
            '    return value;',
            '    ',
            '',
            '## OpenCodeReview',
            '',
            'Generated from OpenCodeReview structured output.',
          ].join('\n',),
          labels: [],
        },);
      },
    },),
    it({
      name: 'uses code fallback and caps complete UTF-8 title bytes',
      fn: async () => {
        /**
         Finding whose content is empty and existing code supplies summary.
         */
        const finding: NormalizedFinding = {
          position: {
            kind: 'record',
            value: 2,
          },
          path: 'src/fallback.ts',
          content: '\n',
          existingCode: '  const secret = value;  ',
          suggestionCode: '',
          startLine: 1,
          endLine: 1,
        };

        expect(renderIssueTitle({ finding, needsTriageLabel: true, }),).toBe(
          '[uncategorized] src/fallback.ts: const secret = value;',
        );

        /**
         ASCII count that leaves exactly four bytes for final emoji.
         */
        const exactPrefixLength = 252;
        /**
         Complete title at exact byte boundary.
         */
        const exactTitle = `${'x'.repeat(exactPrefixLength,)}😀`;
        expect(capIssueTitle(exactTitle,),).toBe(exactTitle,);

        /**
         ASCII count that forces emoji removal and ellipsis suffix.
         */
        const overPrefixLength = 253;
        /**
         Deterministically truncated overlength title.
         */
        const capped = capIssueTitle(`${'x'.repeat(overPrefixLength,)}😀`,);
        expect(new TextEncoder().encode(capped,)).toHaveLength(256,);
        expect(capped.endsWith('…',),).toBe(true,);
        expect(capped.slice(0, -1,),).toBe('x'.repeat(overPrefixLength,),);
      },
    },),
    it({
      name: 'encodes commit-pinned source links at Markdown boundary',
      fn: async () => {
        /**
         Finding path containing Markdown and URL delimiters.
         */
        const finding: NormalizedFinding = {
          position: {
            kind: 'line',
            value: 4,
          },
          path: 'src/a] (bad)#.ts',
          content: 'Use the verified source link.',
          existingCode: '',
          suggestionCode: '',
          startLine: 9,
          endLine: 9,
          category: 'documentation',
        };

        /**
         Rendered linked Issue.
         */
        const issue = renderIssue({
          finding,
          needsTriageLabel: true,
          sourceLink: {
            repository: 'Aquaticat/issues-api',
            commit: 'abcdefabcdefabcdefabcdefabcdefabcdefabcd',
          },
        },);

        expect(issue.body,).toContain(
          `${String.raw`- Location: [src/a\] (bad)#.ts:9-9]`
           }(https://github.com/Aquaticat/issues-api/blob/`
            + `abcdefabcdefabcdefabcdefabcdefabcdefabcd/`
            + `src/a%5D%20%28bad%29%23.ts#L9-L9)`,
        );
      },
    },),
    it({
      name: 'marks other and missing categories explicitly',
      fn: async () => {
        /**
         Shared finding fields for category marker assertions.
         */
        const base = {
          position: { kind: 'record' as const, value: 1, },
          path: 'src/category.ts',
          content: 'Category marker.',
          existingCode: '',
          suggestionCode: '',
          startLine: 1,
          endLine: 1,
        };

        expect(renderIssue({
          finding: { ...base, category: 'other', },
          needsTriageLabel: true,
        },).classificationMarker,).toBe('OTHER',);
        expect(renderIssue({
          finding: base,
          needsTriageLabel: true,
        },).classificationMarker,).toBe('UNCATEGORIZED',);
      },
    },),
  ],
},);
