/**
 * Unit tests for the PR detail renderer.
 *
 * Pure function; no DB, no IO. Each case asserts that produced HTML
 * contains the expected branches/state, escapes XSS, and surfaces the
 * mergeable/state attributes for client-side styling.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core';

import {
  type PrDetailData,
  renderPrDetail,
} from './pr-detail.ts';

const baseData: PrDetailData = {
  ownerLogin: 'alice',
  repoName: 'demo',
  prNumber: 42,
  title: 'Add feature',
  body: 'Description',
  authorLogin: 'alice',
  createdAt: '2026-05-06T12:00:00Z',
  state: 'open',
  baseRef: 'refs/heads/main',
  headRef: 'refs/heads/feat-x',
  headSha: 'abcdef0123456789',
  mergeable: 'unknown',
  approvedCount: 0,
  changesRequestedCount: 0,
};

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: renderPrDetail.name,
      concurrency: 1,
      children: [
        it({
          name: 'emits state, mergeable, and PR number on the root element',
          async fn() {
            await Promise.resolve();
            const { html, } = renderPrDetail(baseData,);
            expect(html.includes('data-state="open"',),).toBe(true,);
            expect(html.includes('data-mergeable="unknown"',),).toBe(true,);
            expect(html.includes('data-pr-number="42"',),).toBe(true,);
          },
        },),
        it({
          name: 'renders base + head refs and head SHA',
          async fn() {
            await Promise.resolve();
            const { html, } = renderPrDetail(baseData,);
            expect(html.includes('refs/heads/main',),).toBe(true,);
            expect(html.includes('refs/heads/feat-x',),).toBe(true,);
            expect(html.includes('abcdef0123456789',),).toBe(true,);
          },
        },),
        it({
          name: 'escapes XSS payloads in title, body, and refs',
          async fn() {
            await Promise.resolve();
            const xss = '<script>alert(1)</script>';
            const { html, } = renderPrDetail({
              ...baseData,
              title: xss,
              body: xss,
              baseRef: xss,
              headRef: xss,
            },);
            expect(html.includes('<script>',),).toBe(false,);
            expect(html.includes('&lt;script&gt;',),).toBe(true,);
          },
        },),
        it({
          name: 'shows review-summary counts',
          async fn() {
            await Promise.resolve();
            const { html, } = renderPrDetail({
              ...baseData,
              approvedCount: 2,
              changesRequestedCount: 1,
            },);
            expect(html.includes('2 approved',),).toBe(true,);
            expect(html.includes('1 changes requested',),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
