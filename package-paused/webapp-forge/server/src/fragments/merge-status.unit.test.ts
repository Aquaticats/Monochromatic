/**
 * Unit tests for the merge-status renderer.
 *
 * Pure function. Each case asserts the summary line for a given
 * `(mergeable, approvedCount, changesRequestedCount, requiredApprovals)`
 * tuple, plus the data-attributes used by client-side styling.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core';

import { renderMergeStatus, } from './merge-status.ts';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: renderMergeStatus.name,
      concurrency: 1,
      children: [
        it({
          name: 'reports conflicts when mergeable === conflicts',
          async fn() {
            await Promise.resolve();
            const { html, } = renderMergeStatus({
              prNumber: 42,
              mergeable: 'conflicts',
              approvedCount: 5,
              changesRequestedCount: 0,
              requiredApprovals: 1,
            },);
            expect(html.includes('Merge conflicts must be resolved',),).toBe(true,);
            expect(html.includes('data-mergeable="conflicts"',),).toBe(true,);
          },
        },),
        it({
          name: 'reports changes-requested when blocking reviews exist',
          async fn() {
            await Promise.resolve();
            const { html, } = renderMergeStatus({
              prNumber: 42,
              mergeable: 'unknown',
              approvedCount: 1,
              changesRequestedCount: 1,
              requiredApprovals: 1,
            },);
            expect(html.includes('Changes have been requested',),).toBe(true,);
          },
        },),
        it({
          name: 'reports approval shortfall',
          async fn() {
            await Promise.resolve();
            const { html, } = renderMergeStatus({
              prNumber: 42,
              mergeable: 'clean',
              approvedCount: 0,
              changesRequestedCount: 0,
              requiredApprovals: 2,
            },);
            expect(html.includes('Needs 2 more approval',),).toBe(true,);
          },
        },),
        it({
          name: 'reports ready-to-merge when state is clean and approvals met',
          async fn() {
            await Promise.resolve();
            const { html, } = renderMergeStatus({
              prNumber: 42,
              mergeable: 'clean',
              approvedCount: 1,
              changesRequestedCount: 0,
              requiredApprovals: 1,
            },);
            expect(html.includes('Ready to merge.',),).toBe(true,);
          },
        },),
        it({
          name: 'reports computing-mergeability when state is unknown and no blockers',
          async fn() {
            await Promise.resolve();
            const { html, } = renderMergeStatus({
              prNumber: 42,
              mergeable: 'unknown',
              approvedCount: 1,
              changesRequestedCount: 0,
              requiredApprovals: 1,
            },);
            expect(html.includes('Mergeability is being computed.',),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
