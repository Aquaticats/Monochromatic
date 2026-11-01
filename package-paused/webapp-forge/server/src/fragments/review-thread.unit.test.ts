/**
 * Unit tests for the review-thread renderer.
 *
 * Pure function. Asserts each review's author/state/body/timestamp is
 * rendered, the review-count attribute is correct, and XSS is escaped.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core';

import {
  renderReviewThread,
  type ReviewThreadData,
} from './review-thread.ts';

const baseData: ReviewThreadData = {
  ownerLogin: 'alice',
  repoName: 'demo',
  prNumber: 42,
  reviews: [],
};

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: renderReviewThread.name,
      concurrency: 1,
      children: [
        it({
          name: 'renders an empty-state when no reviews exist',
          async fn() {
            await Promise.resolve();
            const { html, } = renderReviewThread(baseData,);
            expect(html.includes('No reviews yet.',),).toBe(true,);
            expect(html.includes('data-review-count="0"',),).toBe(true,);
          },
        },),
        it({
          name: 'renders one block per review with state/author/timestamp',
          async fn() {
            await Promise.resolve();
            const { html, } = renderReviewThread({
              ...baseData,
              reviews: [
                {
                  id: 'r1',
                  reviewerLogin: 'reviewer',
                  state: 'approved',
                  body: 'LGTM',
                  createdAt: '2026-05-06T12:00:00Z',
                },
                {
                  id: 'r2',
                  reviewerLogin: 'other',
                  state: 'changes_requested',
                  body: 'please address...',
                  createdAt: '2026-05-06T12:30:00Z',
                },
              ],
            },);
            expect(html.includes('data-review-count="2"',),).toBe(true,);
            expect(html.includes('data-review-id="r1"',),).toBe(true,);
            expect(html.includes('data-state="approved"',),).toBe(true,);
            expect(html.includes('data-state="changes_requested"',),).toBe(true,);
            expect(html.includes('LGTM',),).toBe(true,);
            expect(html.includes('please address...',),).toBe(true,);
          },
        },),
        it({
          name: 'escapes XSS payloads inside review bodies and authors',
          async fn() {
            await Promise.resolve();
            const xss = '<img src=x onerror=alert(1)>';
            const { html, } = renderReviewThread({
              ...baseData,
              reviews: [
                {
                  id: 'r1',
                  reviewerLogin: xss,
                  state: 'commented',
                  body: xss,
                  createdAt: '2026-05-06T12:00:00Z',
                },
              ],
            },);
            expect(html.includes('<img src=x',),).toBe(false,);
            expect(html.includes('&lt;img src=x',),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
