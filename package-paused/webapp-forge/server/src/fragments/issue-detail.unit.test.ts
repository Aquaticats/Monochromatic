/**
 * Unit tests for the `renderIssueDetail` fragment.
 *
 * Snapshot-style assertions on the rendered HTML, plus XSS-safety
 * verification for body, title, and comment content.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core';

import {
  type IssueDetailData,
  renderIssueDetail,
} from './issue-detail.ts';

/**
 * Builds an `IssueDetailData` for tests with sensible defaults.
 *
 * @param overrides - fields to override on the default fixture
 *
 * @returns ready-to-render view-model
 *
 * @example
 * ```ts
 * fixture({ title: 'My bug' });
 * ```
 */
function fixture(overrides: Partial<IssueDetailData> = {},): IssueDetailData {
  return {
    ownerLogin: 'alice',
    repoName: 'test',
    issueNumber: 1,
    title: 'Title',
    body: 'Body content',
    authorLogin: 'alice',
    createdAt: '2026-05-06T12:00:00Z',
    state: 'open',
    labels: [],
    comments: [],
    ...overrides,
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: renderIssueDetail.name,
      children: [
        it({
          name: 'renders title and number',
          async fn() {
            const { html, } = renderIssueDetail(fixture({
              title: 'My bug',
              issueNumber: 42,
            },),);
            expect(html.includes('My bug',),).toBe(true,);
            expect(html.includes('#42',),).toBe(true,);
          },
        },),
        it({
          name: 'escapes HTML in title to prevent XSS',
          async fn() {
            const { html, } = renderIssueDetail(fixture({
              title: '<script>alert(1)</script>',
            },),);
            expect(html.includes('<script>alert(1)</script>',),).toBe(false,);
            expect(html.includes('&lt;script&gt;',),).toBe(true,);
          },
        },),
        it({
          name: 'escapes HTML in body to prevent XSS',
          async fn() {
            const { html, } = renderIssueDetail(fixture({
              body: '<img src=x onerror=alert(1)>',
            },),);
            expect(html.includes('<img src=x',),).toBe(false,);
            expect(html.includes('&lt;img',),).toBe(true,);
          },
        },),
        it({
          name: 'renders labels with their colour',
          async fn() {
            const { html, } = renderIssueDetail(fixture({
              labels: [
                {
                  name: 'bug',
                  color: 'ff0000',
                },
                {
                  name: 'docs',
                  color: '00ff00',
                },
              ],
            },),);
            expect(html.includes('forge-label',),).toBe(true,);
            expect(html.includes('bug',),).toBe(true,);
            expect(html.includes('docs',),).toBe(true,);
            expect(html.includes('data-color="ff0000"',),).toBe(true,);
          },
        },),
        it({
          name: 'escapes XSS payloads inside label names',
          async fn() {
            const { html, } = renderIssueDetail(fixture({
              labels: [
                {
                  name: '<script>',
                  color: 'aaaaaa',
                },
              ],
            },),);
            expect(html.includes('<script>',),).toBe(false,);
            expect(html.includes('&lt;script&gt;',),).toBe(true,);
          },
        },),
        it({
          name: 'renders each comment with author, time, and body',
          async fn() {
            const { html, } = renderIssueDetail(fixture({
              comments: [
                {
                  id: 'c1',
                  authorLogin: 'bob',
                  body: 'first',
                  createdAt: '2026-05-06T13:00:00Z',
                },
                {
                  id: 'c2',
                  authorLogin: 'carol',
                  body: 'second',
                  createdAt: '2026-05-06T14:00:00Z',
                },
              ],
            },),);
            expect(html.includes('bob',),).toBe(true,);
            expect(html.includes('carol',),).toBe(true,);
            expect(html.includes('first',),).toBe(true,);
            expect(html.includes('second',),).toBe(true,);
          },
        },),
        it({
          name: 'escapes XSS payloads inside comment bodies',
          async fn() {
            const { html, } = renderIssueDetail(fixture({
              comments: [
                {
                  id: 'c1',
                  authorLogin: 'mallory',
                  body: '<svg/onload=alert(1)>',
                  createdAt: '2026-05-06T15:00:00Z',
                },
              ],
            },),);
            expect(html.includes('<svg',),).toBe(false,);
            expect(html.includes('&lt;svg',),).toBe(true,);
          },
        },),
        it({
          name: 'links to the owning repo in the meta line',
          async fn() {
            const { html, } = renderIssueDetail(fixture({
              ownerLogin: 'alice',
              repoName: 'test',
            },),);
            expect(html.includes('href="/alice/test"',),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
