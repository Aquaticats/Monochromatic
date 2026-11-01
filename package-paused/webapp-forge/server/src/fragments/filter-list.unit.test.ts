/**
 * Unit tests for the `renderFilterList` fragment.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core';

import {
  type FilterListData,
  renderFilterList,
} from './filter-list.ts';

/**
 * Builds a `FilterListData` for tests with sensible defaults.
 *
 * @param overrides - fields to override on the default fixture
 *
 * @returns ready-to-render view-model
 *
 * @example
 * ```ts
 * fixture({ issues: [{ ... }] });
 * ```
 */
function fixture(overrides: Partial<FilterListData> = {},): FilterListData {
  return {
    ownerLogin: 'alice',
    repoName: 'test',
    facetLabel: 'open',
    issues: [],
    ...overrides,
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: renderFilterList.name,
      children: [
        it({
          name: 'renders an empty-state message when no issues match',
          async fn() {
            const { html, } = renderFilterList(fixture(),);
            expect(html.includes('forge-empty',),).toBe(true,);
            expect(html.includes('No issues match',),).toBe(true,);
          },
        },),
        it({
          name: 'renders one row per issue with link, number, title, and time',
          async fn() {
            const { html, } = renderFilterList(fixture({
              issues: [
                {
                  id: 'i1',
                  number: 1,
                  title: 'First',
                  updatedAt: '2026-05-06T12:00:00Z',
                  state: 'open',
                },
                {
                  id: 'i2',
                  number: 2,
                  title: 'Second',
                  updatedAt: '2026-05-06T13:00:00Z',
                  state: 'open',
                },
              ],
            },),);
            expect(html.includes('href="/alice/test/issues/1"',),).toBe(true,);
            expect(html.includes('href="/alice/test/issues/2"',),).toBe(true,);
            expect(html.includes('First',),).toBe(true,);
            expect(html.includes('Second',),).toBe(true,);
            expect(html.includes('data-count="2"',),).toBe(true,);
          },
        },),
        it({
          name: 'escapes XSS payloads inside titles',
          async fn() {
            const { html, } = renderFilterList(fixture({
              issues: [
                {
                  id: 'i1',
                  number: 1,
                  title: '<script>alert(1)</script>',
                  updatedAt: '2026-05-06T12:00:00Z',
                  state: 'open',
                },
              ],
            },),);
            expect(html.includes('<script>alert(1)</script>',),).toBe(false,);
            expect(html.includes('&lt;script&gt;',),).toBe(true,);
          },
        },),
        it({
          name: 'escapes XSS payloads inside the facet label',
          async fn() {
            const { html, } = renderFilterList(fixture({
              facetLabel: '<img src=x onerror=alert(1)>',
            },),);
            expect(html.includes('<img src=x',),).toBe(false,);
            expect(html.includes('&lt;img',),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
