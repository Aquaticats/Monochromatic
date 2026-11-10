/**
 Tests for the issue printer and the ready-bug grouping.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  groupReadyBugs,
  renderIssue,
  renderReadyBugReport,
  type IssueListing,
  type IssueSnapshot,
} from '../dist/final/node/index.mjs';

/**
 One issue as the tracker reports it.
 
 @param body - Issue body.
 
 @param comments - Comment bodies on the issue.
 
 @returns The snapshot.
 
 @example
 ```ts
 const snapshot = snapshotOf({ body: '## Acceptance\n\nworks', comments: ['first'] });
 ```
 */
function snapshotOf({
  body = 'The body',
  comments = [],
}: {
  readonly body?: string;
  readonly comments?: readonly string[];
},): IssueSnapshot {
  return {
    number: 7,
    title: 'An issue',
    state: 'OPEN',
    body,
    comments,
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: renderIssue.name,
      children: [
        it({
          name: 'prints the whole issue with each comment under its marker',
          fn: async () => {
            /**
             Rendered block.
             */
            const block = renderIssue({
              snapshot: snapshotOf({
                body: 'The body',
                comments: ['first', 'second',],
              },),
              mode: 'full',
            },);
            expect(block,).toBe([
              '#7 [OPEN] An issue',
              'The body',
              '--- comment: first',
              '--- comment: second',
            ].join('\n',),);
          },
        },),
        it({
          name: 'prints only the sections naming what the issue asked for',
          fn: async () => {
            /**
             Rendered block.
             */
            const block = renderIssue({
              snapshot: snapshotOf({
                body: [
                  '## Context',
                  '',
                  'Long context nobody needs while closing.',
                  '',
                  '## Acceptance',
                  '',
                  'A scan of `doc/` finds no dash.',
                ].join('\n',),
              },),
              mode: 'acceptance',
            },);
            expect(block,).toContain('Acceptance',);
            expect(block,).toContain('A scan of `doc/` finds no dash.',);
            expect(block,).not.toContain('Long context',);
          },
        },),
        it({
          name: 'falls back to the opening of the body when no section is named',
          fn: async () => {
            /**
             Rendered block.
             */
            const block = renderIssue({
              snapshot: snapshotOf({ body: 'No headings here at all.', },),
              mode: 'acceptance',
            },);
            expect(block,).toContain('(no named section) No headings here at all.',);
          },
        },),
        it({
          name: 'cuts a named section to the reader budget',
          fn: async () => {
            /**
             Rendered block of a body whose acceptance runs long.
             */
            const block = renderIssue({
              snapshot: snapshotOf({ body: `## Acceptance\n\n${'x'.repeat(2_000,)}`, },),
              mode: 'acceptance',
            },);
            expect(block.length,).toBeLessThan(1_000,);
          },
        },),
      ],
    },),
    describe({
      name: groupReadyBugs.name,
      children: [
        it({
          name: 'groups each issue by the first package its text names',
          fn: async () => {
            /**
             Grouping under test.
             */
            const report = groupReadyBugs({
              listings: [
                { number: 1, title: 'package/a/x: one', body: 'package/b/y', },
                { number: 2, title: 'package/a/x: two', body: '', },
                { number: 3, title: 'no path here', body: 'package/c/z', },
              ],
            },);
            expect(report.considered,).toBe(3,);
            expect(report.groups,).toEqual([
              {
                packagePath: 'package/a/x',
                issues: [1, 2,],
              },
              {
                packagePath: 'package/c/z',
                issues: [3,],
              },
            ],);
          },
        },),
        it({
          name: 'leaves out issues whose subject is paused',
          fn: async () => {
            /**
             Grouping under test.
             */
            const report = groupReadyBugs({
              listings: [
                { number: 1, title: 'Windows path handling', body: 'package/a/x', },
                { number: 2, title: 'LFS pointer', body: 'package/a/x', },
                { number: 3, title: 'package/a/x: real', body: '', },
              ],
            },);
            expect(report.paused,).toBe(2,);
            expect(report.groups.map(function paths(group,): string {
              return group.packagePath;
            },),).toEqual(['package/a/x',],);
            expect(report.groups[0]?.issues,).toEqual([3,],);
          },
        },),
        it({
          name: 'collects issues that name no package under one heading',
          fn: async () => {
            /**
             Grouping under test.
             */
            const report = groupReadyBugs({
              listings: [{ number: 9, title: 'Nothing named', body: '', },],
            },);
            expect(report.groups[0]?.packagePath,).toBe('(no package named)',);
          },
        },),
        it({
          name: 'orders equally sized groups by path',
          fn: async () => {
            /**
             Grouping under test.
             */
            const report = groupReadyBugs({
              listings: [
                { number: 1, title: 'package/z/z', body: '', },
                { number: 2, title: 'package/a/a', body: '', },
              ],
            },);
            expect(report.groups.map(function paths(group,): string {
              return group.packagePath;
            },),).toEqual(['package/a/a', 'package/z/z',],);
          },
        },),
      ],
    },),
    describe({
      name: renderReadyBugReport.name,
      children: [
        it({
          name: 'prints one header line and one line per package',
          fn: async () => {
            /**
             Listings the report groups.
             */
            const listings: readonly IssueListing[] = [
              { number: 1, title: 'package/a/x', body: '', },
            ];
            /**
             Rendered lines.
             */
            const lines = renderReadyBugReport({
              report: groupReadyBugs({ listings, },),
            },);
            expect(lines,).toEqual([
              'issues 1, left out as paused 0, packages 1',
              '1\tpackage/a/x\t1',
            ],);
          },
        },),
      ],
    },),
  ],
},);
