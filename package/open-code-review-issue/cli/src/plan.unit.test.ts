import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  buildNonInteractivePreview,
  buildPublicationPlan,
  SecurityAuthorityError,
  selectApplyPlan,
  type NormalizedInput,
} from '../dist/final/node/index.mjs';

await describe({
  name: buildPublicationPlan.name,
  children: [
    it({
      name: 'redacts security content from non-interactive preview',
      fn: async () => {
        /**
         Mixed input containing ordinary and secret security content.
         */
        const input: NormalizedInput = {
          inputKind: 'comments',
          findings: [
            {
              position: {
                kind: 'record',
                value: 1,
              },
              path: 'src/ordinary.ts',
              content: 'Ordinary finding.',
              existingCode: '',
              suggestionCode: '',
              startLine: 1,
              endLine: 1,
              category: 'bug',
              severity: 'medium',
            },
            {
              position: {
                kind: 'record',
                value: 2,
              },
              path: 'src/private.ts',
              content: 'SECRET SECURITY CONTENT',
              existingCode: 'SECRET EXISTING CODE',
              suggestionCode: '',
              startLine: 7,
              endLine: 8,
              category: 'security',
              severity: 'critical',
            },
          ],
        };

        /**
         Internal complete publication plan.
         */
        const plan = buildPublicationPlan({
          input,
          repository: 'https://github.com/Aquaticat/issues-api',
          needsTriageLabel: true,
        },);
        /**
         Machine-readable preview safe for standard output.
         */
        const preview = buildNonInteractivePreview(plan,);

        expect(preview,).toStrictEqual({
          outcome: 'preview',
          repository: 'https://github.com/Aquaticat/issues-api',
          labelStrategy: 'needs-triage-label',
          sourceReference: 'plain',
          issues: [
            {
              position: {
                kind: 'record',
                value: 1,
              },
              title: '[bug] src/ordinary.ts: Ordinary finding.',
              body: [
                '## Finding',
                '',
                'Ordinary finding.',
                '',
                '## Source',
                '',
                '- Location: src/ordinary.ts:1-1',
                '- Category: `bug`',
                '- Severity: `medium`',
                '',
                '## OpenCodeReview',
                '',
                'Generated from OpenCodeReview structured output.',
              ].join('\n',),
              labels: ['needs-triage',],
            },
          ],
          security: {
            count: 1,
            positions: [
              {
                kind: 'record',
                value: 2,
              },
            ],
          },
        },);
        expect(JSON.stringify(preview,),).not.toContain('SECRET',);
      },
    },),
    it({
      name: 'enforces non-interactive security authority atomically',
      fn: async () => {
        /**
         Complete plan with one ordinary and one security Issue.
         */
        const plan = buildPublicationPlan({
          input: {
            inputKind: 'comments',
            findings: [
              {
                position: { kind: 'record', value: 1, },
                path: 'src/ordinary.ts',
                content: 'Ordinary.',
                existingCode: '',
                suggestionCode: '',
                startLine: 1,
                endLine: 1,
                category: 'bug',
              },
              {
                position: { kind: 'record', value: 2, },
                path: 'src/security.ts',
                content: 'Private security content.',
                existingCode: '',
                suggestionCode: '',
                startLine: 2,
                endLine: 2,
                category: 'security',
              },
            ],
          },
          repository: 'https://github.com/Aquaticat/issues-api',
          needsTriageLabel: false,
        },);

        /**
         Captured bare-apply authority failure.
         */
        let caught: unknown;
        try {
          selectApplyPlan({ plan, authority: 'default', },);
        }
        catch (error: unknown) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(SecurityAuthorityError,);
        expect((caught as Error).message,).not.toContain('Private security content',);

        /**
         Non-security-only apply selection.
         */
        const ordinary = selectApplyPlan({
          plan,
          authority: 'non-security-only',
        },);
        expect(ordinary.issues,).toHaveLength(1,);
        expect(ordinary.withheldPositions,).toStrictEqual([
          { kind: 'record', value: 2, },
        ],);

        /**
         Explicit all-findings apply selection.
         */
        const all = selectApplyPlan({ plan, authority: 'all', },);
        expect(all.issues,).toHaveLength(2,);
        expect(all.withheldPositions,).toStrictEqual([],);
      },
    },),
    it({
      name: 'retains ordinary classification markers in preview',
      fn: async () => {
        /**
         Other and missing-category findings requiring visible markers.
         */
        const input: NormalizedInput = {
          inputKind: 'comments',
          findings: [
            {
              position: { kind: 'record', value: 1, },
              path: 'src/other.ts',
              content: 'Other category.',
              existingCode: '',
              suggestionCode: '',
              startLine: 1,
              endLine: 1,
              category: 'other',
            },
            {
              position: { kind: 'record', value: 2, },
              path: 'src/missing.ts',
              content: 'Missing category.',
              existingCode: '',
              suggestionCode: '',
              startLine: 2,
              endLine: 2,
            },
          ],
        };
        /**
         Safe preview retaining explicit ordinary markers.
         */
        const preview = buildNonInteractivePreview(buildPublicationPlan({
          input,
          repository: 'https://github.com/Aquaticat/issues-api',
          needsTriageLabel: true,
        },),);

        expect(preview.issues.map(function marker(issue,) {
          return issue.classificationMarker;
        },),).toStrictEqual(['OTHER', 'UNCATEGORIZED',],);
      },
    },),
  ],
},);
